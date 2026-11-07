//! A written-empty argument (`F(x,)`, `F(,x)`, or a LET or LAMBDA variable bound to nothing),
//! told apart from a value by the formula's text: the node, not what it evaluates to. The
//! value-level reading of the dynamic-array lookups' modes is `mode_or_default`
//! (`lookup_and_reference/xmatch.rs`, T-4871).

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    /// Whether the argument `node` is empty as written: an empty argument (`f(1,)`), or a
    /// variable bound to an omitted LAMBDA parameter or an empty LET value, which hands the
    /// emptiness on (probes/eng-imsub-imdiv-lambda-let-empty, t4873-empty-arg-let-lambda). A
    /// formula that evaluates to a blank (`IFERROR(<blank>,0)`) is not one.
    pub(crate) fn is_empty_argument(&self, node: &Node) -> bool {
        matches!(node, Node::EmptyArgKind) || self.is_omitted_variable(node)
    }

    /// [`Model::is_empty_argument`] for each of the first `N` arguments (`false` past the last).
    pub(crate) fn empty_arguments<const N: usize>(&self, args: &[Node]) -> [bool; N] {
        std::array::from_fn(|index| {
            args.get(index)
                .is_some_and(|node| self.is_empty_argument(node))
        })
    }
}

/// The Analysis `ToolPak` rule for a written-empty required argument (T-4870): the first arguments'
/// values, each replaced by `na` where that argument is written empty (`empty` from
/// [`Model::empty_arguments`]). A kernel swaps its arguments through this before reading them
/// in Excel's order, so it meets the `#N/A` at that argument's turn and a lifted call gives it per
/// element, as MROUND and GCD read theirs (arrays.md "MROUND keeps its scalar rules", coercion.md):
/// `EDATE(,1)` and `EDATE(,1/0)` are `#N/A`, `EDATE(1/0,)` `#DIV/0!`, `YEARFRAC(1,,9)` `#NUM!`
/// (basis is read first) and `ROWS(EDATE({1;2},))` 2 (probes/t4870-empty-date-args,
/// t4870-r3-order-and-lift). Nothing is copied: the other arguments, holidays included, are read
/// where they are.
pub(crate) fn written_empty_or<'a, const N: usize>(
    values: [&'a CalcResult; N],
    empty: [bool; N],
    na: &'a CalcResult,
) -> [&'a CalcResult; N] {
    std::array::from_fn(|index| if empty[index] { na } else { values[index] })
}

/// The `#N/A` [`written_empty_or`] hands a kernel.
pub(crate) fn written_empty_na(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NA, cell, String::new())
}
