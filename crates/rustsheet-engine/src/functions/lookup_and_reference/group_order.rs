//! How GROUPBY and PIVOTBY compare two keys or two aggregates when they sort the groups
//! (`Grouping::sort_by_keys`, `Grouping::sort_by_aggregate` in grouping.rs). Excel:
//! docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY".

use std::cmp::Ordering;

use crate::{
    expressions::{parser::ArrayNode, types::CellReferenceIndex},
    functions::spill_functions::sort_key_cmp,
    functions::util::collation::cmp_text,
};

/// [`key_cmp`] for two aggregates that may be missing (they never are: the indices come from
/// the same groups).
pub(super) fn compare_aggregates(
    a: Option<&ArrayNode>,
    b: Option<&ArrayNode>,
    ascending: bool,
    cell: CellReferenceIndex,
) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => aggregate_cmp(a, b, ascending, cell),
        _ => Ordering::Equal,
    }
}

/// [`key_cmp`] for two aggregates, one of which may be an array result (a nested cell, shown as
/// `#VALUE!`). Excel sorts that cell by the array, not by the `#VALUE!` it shows: after every
/// number, text and TRUE and before every error, and two arrays element by element in row-major
/// order with the values' order, a blank element greatest, and a shorter array first when one
/// is the start of the other (`LAMBDA(x,x)` sorts groups by their first record; probes/t3893-
/// pivotby-error-tie-order A461-A651, the blank element and the prefix: t3893-pivotby-error-
/// sort-header A61, A71). A blank element against text, booleans, errors and numbers (greatest):
/// probes/t3946-aggregate-array-element-kinds. Unprobed: a one-cell reference result (compared
/// like an array).
fn aggregate_cmp(
    a: &ArrayNode,
    b: &ArrayNode,
    ascending: bool,
    cell: CellReferenceIndex,
) -> Ordering {
    let ordering = match (a, b) {
        (ArrayNode::Nested(x), ArrayNode::Nested(y)) => {
            let mut left = x.elements();
            let mut right = y.elements();
            // Bounded by the smaller array's cells; an inner nested element recurses at most
            // `MAX_NESTED_DEPTH` levels (`NestedArray::new`).
            loop {
                match (left.next(), right.next()) {
                    (None, None) => break Ordering::Equal,
                    (None, Some(_)) => break Ordering::Less,
                    (Some(_), None) => break Ordering::Greater,
                    (Some(p), Some(q)) => {
                        let order = match (p, q) {
                            (ArrayNode::Empty, ArrayNode::Empty) => Ordering::Equal,
                            (ArrayNode::Empty, _) => Ordering::Greater,
                            (_, ArrayNode::Empty) => Ordering::Less,
                            _ => aggregate_cmp(p, q, true, cell),
                        };
                        if order != Ordering::Equal {
                            break order;
                        }
                    }
                }
            }
        }
        (ArrayNode::Nested(_), other) => nested_against(other),
        (other, ArrayNode::Nested(_)) => nested_against(other).reverse(),
        _ => return key_cmp(a, b, ascending, cell),
    };
    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

/// How an array aggregate compares with the scalar aggregate `other` ([`aggregate_cmp`]): below
/// an error, above anything else.
fn nested_against(other: &ArrayNode) -> Ordering {
    if matches!(other, ArrayNode::Error(_)) {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// [`sort_key_cmp`] with numbers compared exactly (see `Grouping::sort_by_keys`), and text
/// compared in place with the comparator `sort_key_cmp` reaches (`cmp_text`, through
/// `compare_values`) instead of converting and cloning both strings on every comparison.
pub(super) fn key_cmp(
    a: &ArrayNode,
    b: &ArrayNode,
    ascending: bool,
    cell: CellReferenceIndex,
) -> Ordering {
    let ordering = match (a, b) {
        (ArrayNode::Number(x), ArrayNode::Number(y)) => x.total_cmp(y),
        (ArrayNode::String(x), ArrayNode::String(y)) => cmp_text(x, y),
        _ => return sort_key_cmp(a, b, ascending, cell),
    };
    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

#[cfg(test)]
mod tests {
    use super::aggregate_cmp;
    use crate::expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex};
    use crate::nested_array::{NestedArray, NestedKind};
    use std::cmp::Ordering;
    use std::sync::Arc;

    const CELL: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };

    fn text(s: &str) -> ArrayNode {
        ArrayNode::String(s.into())
    }

    /// An array aggregate as `aggregate` returns it: a nested cell of one column.
    fn column(cells: &[ArrayNode]) -> ArrayNode {
        let rows = cells.iter().map(|cell| vec![cell.clone()]).collect();
        NestedArray::new(rows, NestedKind::Array).map_or(ArrayNode::Error(Error::ERROR), |n| {
            ArrayNode::Nested(Arc::new(n))
        })
    }

    #[test]
    fn array_aggregates_sort_by_their_elements_between_true_and_errors() {
        // probes/t3893-pivotby-error-tie-order A461-A651, t3893-pivotby-error-sort-header A61.
        let n = ArrayNode::Number;
        let ordered = [
            n(5.0),
            text("t"),
            ArrayNode::Boolean(true),
            column(&[n(-0.5)]),
            column(&[n(1.0), n(6.0)]),
            column(&[n(2.0), n(4.0)]),
            column(&[text("a"), n(1.0)]),
            column(&[ArrayNode::Boolean(true), n(1.0)]),
            column(&[ArrayNode::Empty]),
            column(&[ArrayNode::Empty, n(-0.5)]),
            ArrayNode::Error(Error::NA),
        ];
        for (i, a) in ordered.iter().enumerate() {
            for (j, b) in ordered.iter().enumerate() {
                let up = aggregate_cmp(a, b, true, CELL);
                assert_eq!(up, i.cmp(&j), "{i} vs {j}");
                assert_eq!(aggregate_cmp(a, b, false, CELL), up.reverse(), "{i} vs {j}");
            }
        }
        // Equal arrays tie, so the stable sort keeps key order both ways.
        let same = column(&[n(1.0), n(2.0)]);
        assert_eq!(
            aggregate_cmp(&same, &column(&[n(1.0), n(2.0)]), false, CELL),
            Ordering::Equal
        );
    }
}
