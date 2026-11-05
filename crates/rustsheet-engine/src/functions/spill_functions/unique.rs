//! UNIQUE's grouping of lines, and the bucketed grouping GROUPBY and PIVOTBY share with it.

use std::cmp::Ordering;
use std::collections::HashMap;

use super::{cell_key, sort_key_cmp, CellKey};
use crate::area_budget::text_units;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    functions::util::{
        coalesce_equal_lines,
        collation::{cmp_text_pairwise, collation_key},
    },
    model::Model,
};

/// For each of `count` lines, the first line Excel's `=` finds it equal to (itself when none
/// before it is): UNIQUE's distinct lines and GROUPBY's and PIVOTBY's groups. Lines are bucketed by
/// `key`, which every pair `same` finds equal must share (texts by [`collation_key`]), and a line
/// joins the first kept line of its bucket that `same` finds equal to it, so an intransitive run
/// (U+00AD&`"ab"` equals both `"ab"` and `"ab"`&U+00AD, which differ) joins the first
/// (probes/t4086-groupby-weightless Calc!B4, T-4088). Each comparison is charged to `spend` first
/// (`cost` units; directive 5, SPEC-0024), so a crafted bucket ends as the budget's error instead
/// of running quadratically.
fn first_equal_lines<K: std::hash::Hash + Eq, E>(
    count: usize,
    key: impl Fn(usize) -> K,
    same: impl Fn(usize, usize) -> bool,
    cost: impl Fn(usize, usize) -> u64,
    mut spend: impl FnMut(u64) -> Result<(), E>,
) -> Result<Vec<usize>, E> {
    let mut first = Vec::with_capacity(count);
    let mut buckets: HashMap<K, Vec<usize>> = HashMap::with_capacity(count);
    for line in 0..count {
        let bucket = buckets.entry(key(line)).or_default();
        let mut found = None;
        for &kept in bucket.iter() {
            spend(cost(kept, line))?;
            if same(kept, line) {
                found = Some(kept);
                break;
            }
        }
        first.push(found.unwrap_or_else(|| {
            bucket.push(line);
            line
        }));
    }
    Ok(first)
}

/// For each of `count` lines of `length` cells (`at(line, k)`, `None` past the data), the line
/// UNIQUE keeps for it: the smallest line of its group. Two texts are one value when Excel's `=`
/// finds them equal ([`cmp_text_pairwise`]). Only texts outside ASCII compare intransitively
/// (accents, weightless characters), so lines without one group by equality through the hash
/// buckets ([`first_equal_lines`]); with one, lines group as Excel's merge meets them
/// ([`coalesce_equal_lines`]; probes/t4200-unique-cycle-permutations). Every comparison is
/// charged to `spend` ([`text_units`] of the texts compared, at least one unit): [`unique_of`],
/// and GROUPBY's and PIVOTBY's keys (`Grouping::new`, T-4342).
pub(crate) fn unique_first_lines<'a, E>(
    count: usize,
    length: usize,
    at: impl Fn(usize, usize) -> Option<&'a ArrayNode>,
    mut spend: impl FnMut(u64) -> Result<(), E>,
    cell: CellReferenceIndex,
) -> Result<Vec<usize>, E> {
    let same_texts = |a: usize, b: usize| {
        (0..length).all(|k| match (at(a, k), at(b, k)) {
            (Some(ArrayNode::String(x)), Some(ArrayNode::String(y))) => {
                cmp_text_pairwise(x, y) == Ordering::Equal
            }
            _ => true,
        })
    };
    let compare_units = |a: usize, b: usize| {
        (0..length)
            .map(|k| match (at(a, k), at(b, k)) {
                (Some(ArrayNode::String(x)), Some(ArrayNode::String(y))) => {
                    text_units(x.len() + y.len())
                }
                _ => 0,
            })
            .fold(1, u64::saturating_add)
    };
    let intransitive = (0..count).any(|line| {
        (0..length)
            .any(|k| matches!(at(line, k), Some(ArrayNode::String(text)) if !text.is_ascii()))
    });
    if intransitive {
        let compare_lines = |a: usize, b: usize| {
            (0..length)
                .map(|k| match (at(a, k), at(b, k)) {
                    (Some(x), Some(y)) => unique_cell_cmp(x, y, cell),
                    _ => Ordering::Equal,
                })
                .find(|ordering| *ordering != Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        };
        coalesce_equal_lines(count, |a, b| {
            spend(compare_units(a, b))?;
            Ok(compare_lines(a, b))
        })
    } else {
        first_equal_lines(
            count,
            |line| {
                (0..length)
                    .map(|k| at(line, k).map(unique_cell_key))
                    .collect::<Vec<_>>()
            },
            same_texts,
            compare_units,
            spend,
        )
    }
}

/// One UNIQUE call over `data`: its distinct rows (or columns), or those that occur once.
///
/// Two texts are one value when Excel's `=` finds them equal ([`cmp_text_pairwise`]): case is
/// folded, `"ß"` is `"ss"` and `"á"` is `"a"&UNICHAR(769)`, but `"a"` and `"a"` plus 16 U+0302
/// (equal wrapped accent sums) are two (probes/t3211-collation-sort-cycles G1:J15,
/// t3181-collation-prefix-arrays B1, B2; docs/excel-semantics/coercion.md "Not a total order").
/// Lines are bucketed by their [`CellKey`]s, texts by [`collation_key`], which every pair the
/// rule finds equal shares, and a line joins the first kept line of its bucket the rule finds
/// equal to it. That is Excel's grouping only while `=` is transitive: data holding a text outside
/// ASCII groups as Excel's merge meets its lines instead ([`coalesce_equal_lines`], T-4200), so
/// `{a̧;a͂;a͂͂}` keeps three rows (probes/t3211-collation-sort-cycles G4).
///
/// Texts can share a key and still be unequal (`"a"` plus 16, 32, 48 ... U+0302), so a bucket may
/// hold many kept lines and the walk may compare each line with all of them: every comparison is
/// charged to the formula's work budget ([`text_units`] of the texts compared, at least one unit),
/// so a crafted column ends as the budget's `#VALUE!` instead of running quadratically
/// (directive 5; SPEC-0024).
pub(super) fn unique_of(
    model: &mut Model,
    data: &[Vec<ArrayNode>],
    by_col: bool,
    exactly_once: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    let num_rows = data.len();
    let num_cols = data.first().map_or(0, Vec::len);
    let (count, length) = if by_col {
        (num_cols, num_rows)
    } else {
        (num_rows, num_cols)
    };
    let at = |line: usize, k: usize| {
        if by_col {
            &data[k][line]
        } else {
            &data[line][k]
        }
    };
    let first = unique_first_lines(
        count,
        length,
        |line, k| Some(at(line, k)),
        |units| model.spend_work(units, cell),
        cell,
    );
    let first = match first {
        Ok(first) => first,
        Err(error) => return error,
    };
    // The kept lines, in order, with how many lines each stands for.
    let mut stands_for = vec![0_usize; count];
    for &line in &first {
        if let Some(n) = stands_for.get_mut(line) {
            *n += 1;
        }
    }
    let kept: Vec<(usize, usize)> = (0..count)
        .filter(|&line| first.get(line) == Some(&line))
        .map(|line| (line, stands_for.get(line).copied().unwrap_or(0)))
        .collect();
    let lines: Vec<usize> = kept
        .into_iter()
        .filter(|&(_, n)| !exactly_once || n == 1)
        .map(|(line, _)| line)
        .collect();
    if lines.is_empty() {
        let what = if by_col { "columns" } else { "rows" };
        return CalcResult::new_error(
            Error::CALC,
            cell,
            format!("UNIQUE found no matching {what}"),
        );
    }
    if by_col {
        CalcResult::Array(
            data.iter()
                .map(|row| lines.iter().map(|&c| row[c].clone()).collect())
                .collect(),
        )
    } else {
        CalcResult::Array(lines.iter().map(|&r| data[r].clone()).collect())
    }
}

/// How UNIQUE orders two cells of a line when it merges ([`unique_of`]): two texts as Excel's `<`,
/// `=`, `>` do ([`cmp_text_pairwise`]); other cells are equal exactly when their
/// [`unique_cell_key`]s are, and otherwise ordered as SORT orders them ([`sort_key_cmp`]), ties
/// such as 0 and -0 by their kind and bits, so that unequal cells never compare equal.
fn unique_cell_cmp(a: &ArrayNode, b: &ArrayNode, cell: CellReferenceIndex) -> Ordering {
    if let (ArrayNode::String(x), ArrayNode::String(y)) = (a, b) {
        return cmp_text_pairwise(x, y);
    }
    let (key_a, key_b) = (unique_cell_key(a), unique_cell_key(b));
    if key_a == key_b {
        return Ordering::Equal;
    }
    let rank = |key: &CellKey| match key {
        CellKey::Number(bits) => (0, *bits),
        CellKey::Boolean(b) => (1, u64::from(*b)),
        CellKey::Error(e) => (2, u64::from(*e)),
        CellKey::Lambda(id) => (3, u64::try_from(*id).unwrap_or(u64::MAX)),
        CellKey::Empty => (4, 0),
        CellKey::Str(_) | CellKey::Collated(_) => (5, 0),
    };
    sort_key_cmp(a, b, true, cell).then_with(|| rank(&key_a).cmp(&rank(&key_b)))
}

/// UNIQUE's [`cell_key`]: a text is keyed by its [`collation_key`] (see [`unique_of`]).
pub(crate) fn unique_cell_key(node: &ArrayNode) -> CellKey {
    match node {
        ArrayNode::String(text) => CellKey::Collated(collation_key(text)),
        other => cell_key(other),
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::first_equal_lines;
    use crate::functions::util::collation::{cmp_text_pairwise, collation_key};
    use crate::Text;

    #[test]
    fn t4234_ascii_unique_hash_and_pairwise_group_alike() {
        use crate::functions::util::coalesce_equal_lines;
        let mut texts = crate::test::test_collation::ascii_texts();
        // A deterministic shuffle (an LCG), with duplicates added so groups have several lines.
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = |bound: usize| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            usize::try_from(state >> 33).unwrap_or(0) % bound
        };
        let extra: Vec<Text> = (0..texts.len() / 2)
            .map(|_| texts[next(texts.len())].clone())
            .collect();
        texts.extend(extra);
        for i in (1..texts.len()).rev() {
            texts.swap(i, next(i + 1));
        }
        let hashed = first_equal_lines(
            texts.len(),
            |i| collation_key(&texts[i]),
            |i, j| cmp_text_pairwise(&texts[i], &texts[j]) == Ordering::Equal,
            |_, _| 1,
            |_| Ok::<(), ()>(()),
        );
        let coalesced = coalesce_equal_lines(texts.len(), |i, j| {
            Ok::<_, ()>(cmp_text_pairwise(&texts[i], &texts[j]))
        });
        assert_eq!(hashed, coalesced);
    }
}
