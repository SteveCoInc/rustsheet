//! A written-empty argument (`F(x,)`, `F(,x)`, or a LET or LAMBDA variable bound to nothing),
//! told apart from a value by the formula's text: the node, not what it evaluates to. The
//! value-level reading of the dynamic-array lookups' modes is `mode_or_default`
//! (lookup_and_reference/xmatch.rs, T-4871).

use std::borrow::Cow;

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

/// The Analysis `ToolPak` rule for a written-empty required argument (T-4870): `values` with each
/// position of `empty` ([`Model::written_empty_required`]) read as `#N/A`. A kernel that reads its
/// arguments in Excel's order then meets the `#N/A` at that argument's turn, and a lifted call gives
/// it per element, as MROUND and GCD read theirs (arrays.md "MROUND keeps its scalar rules",
/// coercion.md): `EDATE(,1)` and `EDATE(,1/0)` are `#N/A`, `EDATE(1/0,)` `#DIV/0!`,
/// `YEARFRAC(1,,9)` `#NUM!` (basis is read first) and `ROWS(EDATE({1;2},))` 2
/// (probes/t4870-empty-date-args, t4870-r3-order-and-lift).
pub(crate) fn na_at_written_empty<'a>(
    values: &'a [CalcResult],
    empty: &[usize],
    cell: CellReferenceIndex,
) -> Cow<'a, [CalcResult]> {
    if empty.is_empty() {
        return Cow::Borrowed(values);
    }
    let mut values = values.to_vec();
    for &index in empty {
        if let Some(value) = values.get_mut(index) {
            *value = CalcResult::new_error(Error::NA, cell, String::new());
        }
    }
    Cow::Owned(values)
}
