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
    /// Whether the argument `node` is written empty (`F(x,)`) or is a LET or LAMBDA variable bound
    /// to nothing: the two read alike wherever Excel tells an empty argument from a value
    /// (`IMSUB("1",)` and `LET(e,,IMSUB("1",e))`, `WORKDAY(1,5,)` and `LET(h,,WORKDAY(1,5,h))`).
    pub(crate) fn is_written_empty(&self, node: &Node) -> bool {
        matches!(node, Node::EmptyArgKind) || self.is_omitted_variable(node)
    }

    /// The positions among the first `required` arguments that are written empty
    /// ([`Model::is_written_empty`]), read once from the formula's text before lifting: an argument
    /// that only evaluates to nothing is not one (`EDATE(<#VALUE! start>,IFNA(<blank>,..))` stays
    /// the start's `#VALUE!`, gen-m6/b54448af3fde07ca A297).
    pub(crate) fn written_empty_required(&self, args: &[Node], required: usize) -> Vec<usize> {
        args.iter()
            .take(required)
            .enumerate()
            .filter(|(_, arg)| self.is_written_empty(arg))
            .map(|(index, _)| index)
            .collect()
    }
}

/// The Analysis `ToolPak` rule for a written-empty required argument (T-4870): argument `index`'s
/// value, or `na` where that argument is written empty (`empty` from
/// [`Model::written_empty_required`]). A kernel swaps its arguments through this before reading them
/// in Excel's order, so it meets the `#N/A` at that argument's turn and a lifted call gives it per
/// element, as MROUND and GCD read theirs (arrays.md "MROUND keeps its scalar rules", coercion.md):
/// `EDATE(,1)` and `EDATE(,1/0)` are `#N/A`, `EDATE(1/0,)` `#DIV/0!`, `YEARFRAC(1,,9)` `#NUM!`
/// (basis is read first) and `ROWS(EDATE({1;2},))` 2 (probes/t4870-empty-date-args,
/// t4870-r3-order-and-lift). Nothing is copied: the other arguments, holidays included, are read
/// where they are.
pub(crate) fn written_empty_or<'a>(
    value: &'a CalcResult,
    index: usize,
    empty: &[usize],
    na: &'a CalcResult,
) -> &'a CalcResult {
    if empty.contains(&index) {
        na
    } else {
        value
    }
}

/// The `#N/A` [`written_empty_or`] hands a kernel.
pub(crate) fn written_empty_na(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NA, cell, String::new())
}
