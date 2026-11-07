//! A written-empty argument (`F(x,)`, `F(,x)`, or a LET or LAMBDA variable bound to nothing),
//! told apart from a value by the formula's text: the node, not what it evaluates to. The
//! value-level reading of the dynamic-array lookups' modes is `mode_or_default`
//! (lookup_and_reference/xmatch.rs, T-4871).

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

    /// The Analysis `ToolPak` rule for a written-empty required argument (T-4870): `#N/A` where one of
    /// the first `required` arguments is written empty ([`Model::is_written_empty`]): `EDATE(,1)`,
    /// `WORKDAY(1,)` and `LET(e,,EDATE(1,e))` (probes/t4870-empty-date-args), as for MROUND,
    /// RANDBETWEEN, IMSUB and IMDIV. It is a rule of the formula's text: a blank cell, and a value
    /// that only evaluates to empty (`EDATE(1/0,IFNA(<blank>,..))` is the start's `#VALUE!`,
    /// gen-m6/b54448af3fde07ca A297), are read as before. Excel's order against another argument's
    /// error when both are written that way is unprobed.
    pub(crate) fn toolpak_empty_required(
        &self,
        args: &[Node],
        required: usize,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        args.iter()
            .take(required)
            .any(|arg| self.is_written_empty(arg))
            .then(|| CalcResult::new_error(Error::NA, cell, String::new()))
    }
}
