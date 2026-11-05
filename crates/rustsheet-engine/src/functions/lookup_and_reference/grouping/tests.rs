//! Unit tests of the grouping core (`super`).

use super::{Axis, Grouping, Line, ParentOrder};
use crate::expressions::{parser::ArrayNode, types::CellReferenceIndex};
use std::cmp::Ordering;

const CELL: CellReferenceIndex = CellReferenceIndex {
    sheet: 0,
    row: 1,
    column: 1,
};

fn text(s: &str) -> ArrayNode {
    ArrayNode::String(s.into())
}

/// The first key of each group and its rows, after sorting with `ascending`.
fn sorted(keys: &[ArrayNode], ascending: bool) -> Vec<(ArrayNode, Vec<usize>)> {
    let keys: Vec<Vec<ArrayNode>> = keys.iter().map(|key| vec![key.clone()]).collect();
    let mut grouping = Grouping::new(&keys, 0..keys.len(), &[], |_| None, |_| Ok(()), CELL)
        .unwrap_or(Grouping {
            groups: Vec::new(),
            ascending: Vec::new(),
        });
    grouping.sort_by_keys(&[ascending], CELL);
    grouping
        .groups
        .into_iter()
        .map(|group| (group.keys[0].clone(), group.rows))
        .collect()
}

/// T-4088: keys group by Excel's `=`, the prefix rule included, each joining the first group
/// it equals (probes/t4086-groupby-weightless Calc!B1, B4:B6, D4:D6, B11).
#[test]
fn keys_group_by_excel_equality() {
    let keys = [
        text("ab"),
        text("ab\u{ad}"),
        text("\u{e9}"),
        text("\u{e9}\u{ad}"),
        text("\u{ad}ab"),
        text("AB"),
        text("\u{df}"),
        text("ss"),
    ];
    assert_eq!(
        sorted(&keys, true),
        [
            (text("ab"), vec![0, 4, 5]),
            (text("ab\u{ad}"), vec![1]),
            (text("\u{e9}"), vec![2, 3]),
            (text("\u{df}"), vec![6, 7]),
        ]
    );
}

/// T-4195: on an intransitive run a key joins the distinct key a binary search of the sorted
/// distinct keys finds, not its first equal one, and the groups follow `<`'s order
/// (probes/t4195-groupby-intransitive Calc!B4, B32).
#[test]
fn intransitive_keys_join_the_key_a_binary_search_finds() {
    // `ab`&U+00AD, U+00AD&`ab`, `ab`: the middle one equals both, and joins `ab`.
    assert_eq!(
        sorted(&[text("ab\u{ad}"), text("\u{ad}ab"), text("ab")], true),
        [(text("ab"), vec![1, 2]), (text("ab\u{ad}"), vec![0])]
    );
    // `AB` < `ab`&U+00AD by the prefix rule, so its group comes first.
    assert_eq!(
        sorted(
            &[text("AB"), text("ab"), text("ab\u{ad}"), text("\u{ad}ab")],
            true
        ),
        [(text("AB"), vec![0, 1, 3]), (text("ab\u{ad}"), vec![2])]
    );
}

/// T-4195: a later field is classed within its parent, a group shows each class's first key, and
/// a group none of whole-row `=`'s first rows is in is not shown (probes/t4195-groupby-intransitive
/// Calc!B55, probes/t4088-groupby-multifield Calc!B30, probes/t4195-groupby-shown-key Calc!B2).
#[test]
fn later_fields_class_within_their_parent() {
    let group = |keys: Vec<Vec<ArrayNode>>| {
        let mut grouping = Grouping::new(&keys, 0..keys.len(), &[], |_| None, |_| Ok(()), CELL)
            .unwrap_or(Grouping {
                groups: Vec::new(),
                ascending: Vec::new(),
            });
        grouping.sort_by_keys(&[], CELL);
        grouping
            .groups
            .into_iter()
            .map(|group| (group.keys, group.rows))
            .collect::<Vec<_>>()
    };
    let x = || text("x");
    let y = || text("y");
    assert_eq!(
        group(vec![
            vec![x(), text("\u{ad}ab")],
            vec![y(), text("ab")],
            vec![y(), text("ab\u{ad}")],
        ]),
        [
            (vec![x(), text("\u{ad}ab")], vec![0]),
            (vec![y(), text("ab")], vec![1]),
            (vec![y(), text("ab\u{ad}")], vec![2]),
        ]
    );
    assert_eq!(
        group(vec![
            vec![text("ab"), x()],
            vec![text("ab\u{ad}"), y()],
            vec![text("\u{ad}ab"), y()],
        ]),
        [
            (vec![text("ab"), x()], vec![0]),
            (vec![text("ab\u{ad}"), y()], vec![1]),
        ]
    );
    assert_eq!(
        group(vec![vec![text("ABC"), x()], vec![text("abc"), y()]]),
        [
            (vec![text("ABC"), x()], vec![0]),
            (vec![text("ABC"), y()], vec![1]),
        ]
    );
}

/// The tree over UNIQUE's sorted keys (T-4241, T-4342; docs/excel-semantics/coercion.md "GROUPBY on
/// an intransitive run"): A = `ab`, B = `ab`&U+00AD, C = U+00AD&`ab`.
#[test]
fn multi_field_keys_group_by_the_tree_over_sorted_unique_keys() {
    let group = |keys: Vec<Vec<ArrayNode>>, ascending: &[bool]| {
        Grouping::new(&keys, 0..keys.len(), ascending, |_| None, |_| Ok(()), CELL)
            .map(|grouping| {
                grouping
                    .groups
                    .into_iter()
                    .map(|group| (group.keys, group.rows))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let (a, b, c) = (|| text("ab"), || text("ab\u{ad}"), || text("\u{ad}ab"));
    let (x, y) = (|| text("x"), || text("y"));
    // (A,y), (B,x), (C,y): row 1 is in two groups, and the leaf no row reaches, (C,y) shown as
    // (B,y), holds every row (probes/t4195-groupby-intransitive Calc!C8).
    assert_eq!(
        group(vec![vec![a(), y()], vec![b(), x()], vec![c(), y()]], &[]),
        [
            (vec![a(), y()], vec![0, 2]),
            (vec![b(), x()], vec![1]),
            (vec![b(), y()], vec![0, 1, 2]),
        ]
    );
    // Cx, Ay, Cy, Bx, By: the leaves repeat (C,x), (C,y), and the search over [0, n-1] puts the
    // y rows in the first (C,y) (probes/t4342-groupby-tree-holdout Calc!B14).
    assert_eq!(
        group(
            vec![
                vec![c(), x()],
                vec![a(), y()],
                vec![c(), y()],
                vec![b(), x()],
                vec![b(), y()],
            ],
            &[],
        ),
        [
            (vec![c(), x()], vec![0, 3]),
            (vec![c(), y()], vec![1, 2, 4]),
            (vec![c(), x()], vec![0, 1, 2, 3, 4]),
            (vec![c(), y()], vec![0, 1, 2, 3, 4]),
        ]
    );
    // A group shows the first spelling in whole-row sorted order, ASCII too: (ABC,y), (abc,x)
    // shows `abc` twice (t4342 Calc!B51).
    assert_eq!(
        group(vec![vec![text("ABC"), y()], vec![text("abc"), x()]], &[]),
        [
            (vec![text("abc"), x()], vec![1]),
            (vec![text("abc"), y()], vec![0]),
        ]
    );
    // A descending first field reverses the whole-row sort the tree is built on (t4241 Calc!C).
    assert_eq!(
        group(vec![vec![text("p"), x()], vec![text("q"), y()]], &[false]),
        [
            (vec![text("q"), y()], vec![1]),
            (vec![text("p"), x()], vec![0])
        ]
    );
}

#[test]
fn case_folded_first_spelling_and_blank_last_both_ways() {
    let keys = [
        text("West"),
        ArrayNode::Empty,
        text("east"),
        ArrayNode::Number(2.0),
        text("East"),
        ArrayNode::Boolean(true),
    ];
    let ascending = sorted(&keys, true);
    assert_eq!(
        ascending,
        [
            (ArrayNode::Number(2.0), vec![3]),
            (text("east"), vec![2, 4]),
            (text("West"), vec![0]),
            (ArrayNode::Boolean(true), vec![5]),
            (ArrayNode::Empty, vec![1]),
        ]
    );
    // Descending reverses every type but keeps the blank key last (G!A145).
    let descending = sorted(&keys, false);
    assert_eq!(
        descending
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>(),
        [
            ArrayNode::Boolean(true),
            text("West"),
            text("east"),
            ArrayNode::Number(2.0),
            ArrayNode::Empty,
        ]
    );
}

#[test]
fn two_key_fields_sort_lexicographically() {
    let keys = vec![
        vec![text("b"), ArrayNode::Number(2.0)],
        vec![text("a"), ArrayNode::Number(3.0)],
        vec![text("b"), ArrayNode::Number(1.0)],
        vec![text("a"), ArrayNode::Number(3.0)],
    ];
    let mut grouping = Grouping::new(&keys, 0..keys.len(), &[], |_| None, |_| Ok(()), CELL)
        .unwrap_or(Grouping {
            groups: Vec::new(),
            ascending: Vec::new(),
        });
    grouping.sort_by_keys(&[true, false], CELL);
    let rows: Vec<Vec<usize>> = grouping.groups.into_iter().map(|g| g.rows).collect();
    assert_eq!(rows, [vec![1, 3], vec![0], vec![2]]);
}

#[test]
fn value_sort_keeps_parents_with_equal_aggregates_together() {
    // Parents a and b both sum to 4: they keep key order and their children do not
    // interleave; b's children tie at 2 and keep key order (w before y).
    let keys = vec![
        vec![text("a"), text("x")],
        vec![text("b"), text("y")],
        vec![text("a"), text("z")],
        vec![text("b"), text("w")],
    ];
    let values = [1.0, 2.0, 3.0, 2.0];
    let mut grouping = Grouping::new(&keys, 0..keys.len(), &[], |_| None, |_| Ok(()), CELL)
        .unwrap_or(Grouping {
            groups: Vec::new(),
            ascending: Vec::new(),
        });
    let mut calls = 0;
    let order = (ParentOrder::ByValue, Axis::Rows);
    let sums = grouping.sort_by_aggregate(true, order, CELL, |rows, _| {
        calls += 1;
        ArrayNode::Number(rows.iter().map(|&row| values[row]).sum())
    });
    let leaves: Vec<_> = (0..4)
        .map(|i| sums.of_line(Line::Group(i)).cloned())
        .collect();
    let rows: Vec<Vec<usize>> = grouping.groups.iter().map(|g| g.rows.clone()).collect();
    assert_eq!(rows, [vec![0], vec![2], vec![3], vec![1]]);
    assert_eq!(
        leaves,
        [1.0, 3.0, 2.0, 2.0].map(|n| Some(ArrayNode::Number(n)))
    );
    // One call per group and one per parent, and the subtotal lines get the parents' sums
    // without another call.
    assert_eq!(calls, 6);
    let subtotals: Vec<_> = grouping
        .lines(1, false)
        .into_iter()
        .filter(|line| matches!(line, Line::Subtotal { .. }))
        .map(|line| sums.of_line(line).cloned())
        .collect();
    assert_eq!(
        subtotals,
        [Some(ArrayNode::Number(4.0)), Some(ArrayNode::Number(4.0))]
    );
    assert_eq!(calls, 6);
}

/// Three-key groups a/x/1, a/x/2, a/y/1, b/x/1, already in key order.
fn three_keys() -> Grouping {
    let keys: Vec<Vec<ArrayNode>> = [
        ("a", "x", 1.0),
        ("a", "x", 2.0),
        ("a", "y", 1.0),
        ("b", "x", 1.0),
    ]
    .iter()
    .map(|(a, b, c)| vec![text(a), text(b), ArrayNode::Number(*c)])
    .collect();
    Grouping::new(&keys, 0..keys.len(), &[], |_| None, |_| Ok(()), CELL).unwrap_or(Grouping {
        groups: Vec::new(),
        ascending: Vec::new(),
    })
}

#[test]
fn subtotal_lines_after_and_before_their_groups() {
    let grouping = three_keys();
    let subtotal = |level, start, end| Line::Subtotal { level, start, end };
    // Depth 2: the first level only (probes/groupby-subtotals G!A1).
    assert_eq!(
        grouping.lines(1, false),
        [
            Line::Group(0),
            Line::Group(1),
            Line::Group(2),
            subtotal(1, 0, 3),
            Line::Group(3),
            subtotal(1, 3, 4)
        ]
    );
    // Depth 3: the deepest subtotal first below the groups (G!A61) ...
    assert_eq!(
        grouping.lines(2, false),
        [
            Line::Group(0),
            Line::Group(1),
            subtotal(2, 0, 2),
            Line::Group(2),
            subtotal(2, 2, 3),
            subtotal(1, 0, 3),
            Line::Group(3),
            subtotal(2, 3, 4),
            subtotal(1, 3, 4),
        ]
    );
    // ... and the shallowest first above them (G!A91).
    assert_eq!(
        grouping.lines(2, true),
        [
            subtotal(1, 0, 3),
            subtotal(2, 0, 2),
            Line::Group(0),
            Line::Group(1),
            subtotal(2, 2, 3),
            Line::Group(2),
            subtotal(1, 3, 4),
            subtotal(2, 3, 4),
            Line::Group(3),
        ]
    );
    assert_eq!(
        grouping.lines(0, true),
        (0..4).map(Line::Group).collect::<Vec<_>>()
    );
    assert_eq!(grouping.rows_of(0, 3), [0, 1, 2]);
    assert_eq!(
        grouping.subtotal_label_cells(1, 3, 3),
        [text("b"), text(""), text("")]
    );
}

/// Every kind of key cell a sort meets: each `ArrayNode` variant, numbers at the edges of the
/// 15-digit comparison (-0/+0, subnormals, 0.1+0.2 against 0.3, infinities, NaN), texts that
/// differ only in case, accents, expansions (`æ`, `ß`), skipped characters, the ASCII fast
/// path against the general one and lone surrogates, and every error kind.
fn sort_pool() -> Vec<ArrayNode> {
    use crate::{expressions::token::Error, text::Text};
    let numbers = [
        f64::NEG_INFINITY,
        -f64::MAX,
        -1.0,
        -1e-300,
        -5e-324,
        -0.0,
        0.0,
        5e-324,
        1e-300,
        0.1 + 0.2,
        0.3,
        0.999_999_999_999_999_9,
        1.0,
        1.0 + f64::EPSILON,
        1.000_000_000_000_001,
        123_456_789_012_345.6,
        123_456_789_012_345.7,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
        -f64::NAN,
    ];
    let texts = [
        "", " ", "\u{a0}", "\t", "\n", "\u{1}", "a", "A", "b", "B", "z", "ä", "Ä", "á", "à", "ã",
        "å", "ª", "ae", "AE", "æ", "Æ", "ss", "SS", "ß", "th", "þ", "o", "ö", "ø", "d", "đ", "ð",
        "i", "I", "ı", "İ", "ab", "a'b", "a-b", "a–b", "a—b", "-a", "'a", "a-", "1", "-1", "'1",
        "10", "½", "¹", "~", "+", "€", "<", "😀", "\u{fffd}", "a\u{a0}b",
    ];
    let units: [&[u16]; 5] = [
        &[0xD83D],
        &[0xDE00],
        &[0xD83D, 0xDE00],
        &[0x61, 0xD800],
        &[0xDC00, 0xD800],
    ];
    numbers
        .into_iter()
        .map(ArrayNode::Number)
        .chain(texts.into_iter().map(text))
        .chain(
            units
                .into_iter()
                .map(|u| ArrayNode::String(Text::from_utf16(u.to_vec()))),
        )
        .chain([ArrayNode::Boolean(false), ArrayNode::Boolean(true)])
        .chain(Error::ALL.into_iter().map(ArrayNode::Error))
        .chain([ArrayNode::Empty])
        .collect()
}

/// Checks that `cmp` is a total order on `pool`: reflexive, antisymmetric and transitive
/// (`slice::sort_by` may panic otherwise, since Rust 1.81). Exhaustive over all triples, on
/// a precomputed table.
fn assert_total_order(
    pool: &[ArrayNode],
    name: &str,
    cmp: impl Fn(&ArrayNode, &ArrayNode) -> Ordering,
) {
    let n = pool.len();
    let table: Vec<Vec<Ordering>> = pool
        .iter()
        .map(|a| pool.iter().map(|b| cmp(a, b)).collect())
        .collect();
    for i in 0..n {
        assert_eq!(
            table[i][i],
            Ordering::Equal,
            "{name}: {:?} not equal to itself",
            pool[i]
        );
        for j in 0..n {
            assert_eq!(
                table[i][j],
                table[j][i].reverse(),
                "{name}: not antisymmetric on {:?}, {:?}",
                pool[i],
                pool[j]
            );
            for k in 0..n {
                let (ij, jk) = (table[i][j], table[j][k]);
                // a <= b <= c implies a <= c, strictly when either step is strict.
                let implied = match (ij, jk) {
                    (Ordering::Equal, step) | (step, Ordering::Equal) => Some(step),
                    (x, y) if x == y => Some(x),
                    _ => None,
                };
                if let Some(implied) = implied {
                    assert_eq!(
                        table[i][k], implied,
                        "{name}: not transitive on {:?}, {:?}, {:?}",
                        pool[i], pool[j], pool[k]
                    );
                }
            }
        }
    }
}

#[test]
fn sort_comparators_are_total_orders_on_mixed_types() {
    use crate::functions::spill_functions::sort_key_cmp;
    let pool = sort_pool();
    for ascending in [true, false] {
        // SORT and SORTBY (texts aside: `compare_values_pairwise`), and GROUPBY/PIVOTBY.
        assert_total_order(&pool, "sort_key_cmp", |a, b| {
            sort_key_cmp(a, b, ascending, CELL)
        });
        assert_total_order(&pool, "key_cmp", |a, b| {
            super::super::group_order::key_cmp(a, b, ascending, CELL)
        });
    }
}

/// T-4195 review: a table sort naming a later field first compares texts by rank under one parent
/// and by `cmp_text` across parents, which can cycle; the sort must still end with every group.
#[test]
fn table_sort_by_a_later_field_mixing_ranks_does_not_panic() {
    let spellings = ["AB", "ab\u{ad}", "ab", "\u{ad}ab", "ab\u{ad}\u{ad}", "Ab"];
    let firsts = ["x", "y", "z"];
    let keys: Vec<Vec<ArrayNode>> = firsts
        .iter()
        .flat_map(|first| {
            spellings
                .iter()
                .map(move |second| vec![text(first), text(second)])
        })
        .collect();
    for ascending in [true, false] {
        let mut grouping = Grouping::new(&keys, 0..keys.len(), &[], |_| None, |_| Ok(()), CELL)
            .unwrap_or(Grouping {
                groups: Vec::new(),
                ascending: Vec::new(),
            });
        let before: usize = grouping.groups.iter().map(|group| group.rows.len()).sum();
        grouping.sort_table_by_keys(&[true, ascending], &[1], CELL);
        let after: usize = grouping.groups.iter().map(|group| group.rows.len()).sum();
        assert_eq!(before, after);
        assert!(!grouping.groups.is_empty());
    }
}

/// A leaf no row reaches holds every row, and that copy is charged to the work budget per row
/// (directive 5): crafted runs with many such leaves end as the budget's error.
#[test]
fn a_leaf_no_row_reaches_is_charged_per_row() {
    let (a, b, c) = (|| text("ab"), || text("ab\u{ad}"), || text("\u{ad}ab"));
    let (x, y) = (|| text("x"), || text("y"));
    let mut keys = vec![vec![a(), y()], vec![b(), x()], vec![c(), y()]];
    keys.extend((0..7).map(|_| vec![text("p"), x()]));
    let mut charges = Vec::new();
    let grouping = Grouping::new(
        &keys,
        0..keys.len(),
        &[],
        |_| None,
        |units| {
            charges.push(units);
            Ok(())
        },
        CELL,
    );
    let all = grouping
        .map(|grouping| {
            grouping
                .groups
                .iter()
                .filter(|group| !group.reached)
                .count()
        })
        .unwrap_or(0);
    assert_eq!(all, 1);
    assert!(charges.contains(&10), "{charges:?}");
    let refused = Grouping::new(
        &keys,
        0..keys.len(),
        &[],
        |_| None,
        |units| {
            if units >= 10 {
                Err(super::value_error(CELL, "budget"))
            } else {
                Ok(())
            }
        },
        CELL,
    );
    assert!(refused.is_err());
}
