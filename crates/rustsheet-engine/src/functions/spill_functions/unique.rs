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
pub(crate) fn first_equal_lines<K: std::hash::Hash + Eq, E>(
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
    let same_texts = |a: usize, b: usize| {
        (0..length).all(|k| match (at(a, k), at(b, k)) {
            (ArrayNode::String(x), ArrayNode::String(y)) => {
                cmp_text_pairwise(x, y) == Ordering::Equal
            }
            _ => true,
        })
    };
    let compare_units = |a: usize, b: usize| {
        (0..length)
            .map(|k| match (at(a, k), at(b, k)) {
                (ArrayNode::String(x), ArrayNode::String(y)) => text_units(x.len() + y.len()),
                _ => 0,
            })
            .fold(1, u64::saturating_add)
    };
    // Only texts outside ASCII compare intransitively (accents, weightless characters), so data
    // without one groups by equality through the hash buckets; with one, lines group as Excel's
    // merge meets them ([`coalesce_equal_lines`]; probes/t4200-unique-cycle-permutations).
    let intransitive = data
        .iter()
        .flatten()
        .any(|node| matches!(node, ArrayNode::String(text) if !text.is_ascii()));
    let first = if intransitive {
        let compare_lines = |a: usize, b: usize| {
            (0..length)
                .map(|k| unique_cell_cmp(at(a, k), at(b, k), cell))
                .find(|ordering| *ordering != Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        };
        coalesce_equal_lines(count, |a, b| {
            model.spend_work(compare_units(a, b), cell)?;
            Ok(compare_lines(a, b))
        })
    } else {
        first_equal_lines(
            count,
            |line| {
                (0..length)
                    .map(|k| unique_cell_key(at(line, k)))
                    .collect::<Vec<_>>()
            },
            same_texts,
            compare_units,
            |units| model.spend_work(units, cell),
        )
    };
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
