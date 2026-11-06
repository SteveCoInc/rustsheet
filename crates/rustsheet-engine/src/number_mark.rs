//! The "mark" of a number from ERROR.TYPE or `CELL("col"|"row",..)`: some array-taking arguments
//! reject it where the same number from anywhere else is accepted. F.TEST, the paired functions and
//! SUMXMY2 give `#NUM!` for it, LINEST and IRR's flows `#VALUE!` (statistics.md "ERROR.TYPE and
//! CELL results are #NUM! in the paired functions", financial.md "IRR's error order"). LET, IF,
//! CHOOSE, IFERROR, a LAMBDA call and INDEX hand it on; `+0`, `@`, SINGLE, MAX and a cell holding
//! the value drop it (probes/t2576-error-type-cell-paired D1:D28). Why Excel does this is unknown.
//!
//! The mark is a flag on a value, not a form of the formula: it is kept beside the evaluation,
//! not in [`CalcResult`]. Each [`Model::evaluate_node_in_context`] call gets a serial number, and
//! [`NumberMark`] remembers the latest call whose value carries the mark, by its serial and the
//! node it evaluated. A reader asks whether the call it just made is that one
//! ([`Model::evaluate_marked`]); a function that hands an argument on unchanged moves the mark from
//! that argument's call to its own ([`Model::pass_number_mark`]). Only the kinds listed in
//! [`passes_mark`] keep a mark handed to them, so a call made by any other function, however it was
//! reached, never looks marked. A LET name or LAMBDA parameter bound to a marked value is marked
//! where it is read (`VariableStack::mark`). Constant memory: two serials and a flag.

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
    variable_stack::Binding,
};

/// One `evaluate_node_in_context` call: its serial and the address of the node it evaluated.
type Call = (u64, usize);

/// Where the mark is (module docs).
#[derive(Clone, Copy, Default)]
pub(crate) struct NumberMark {
    /// The serial the next `evaluate_node_in_context` call gets.
    next: u64,
    /// The call running now.
    current: Call,
    /// The latest call whose value carries the mark.
    marked: Option<Call>,
    /// Whether the last [`Model::bind_argument`] bound a marked value, for the LET or LAMBDA
    /// call that binds it ([`Model::take_bound_mark`]).
    bound: bool,
}

fn address(node: &Node) -> usize {
    std::ptr::from_ref(node.ungrouped()) as usize
}

/// Whether `node` (ungrouped) is a call that makes its number marked: ERROR.TYPE, or CELL with
/// "col" or "row" typed as its info type. `CELL("contents",..)` of a number, N, SHEET, COLUMNS,
/// MATCH and the rest do not (probes/t2576-error-type-cell-paired D6:D26).
fn makes_mark(node: &Node) -> bool {
    match node {
        Node::FunctionKind {
            kind: Function::ErrorType,
            ..
        } => true,
        Node::FunctionKind {
            kind: Function::Cell,
            args,
        } => matches!(
            args.first(),
            Some(Node::StringKind(info)) if info.eq_ignore_ascii_case("col") || info.eq_ignore_ascii_case("row")
        ),
        _ => false,
    }
}

/// Whether `node` (ungrouped) hands on the mark of the argument it returns unchanged: LET, IF,
/// CHOOSE, IFERROR, INDEX and a LAMBDA called in place (probes/t2576-error-type-cell-paired D1:D5,
/// D19, D20). IFNA, IFS, SWITCH and a called LET name are unprobed and drop it.
fn passes_mark(node: &Node) -> bool {
    matches!(
        node,
        Node::FunctionKind {
            kind: Function::Let
                | Function::If
                | Function::Choose
                | Function::Iferror
                | Function::Index,
            ..
        } | Node::LambdaCallKind { .. }
    )
}

/// Whether a value can carry the mark: a number, or an array (`ERROR.TYPE({#REF!})`, D24; an
/// array of references read through a LET name, probes/t2577-indirect-ref-array A64).
fn can_carry(value: &CalcResult) -> bool {
    matches!(value, CalcResult::Number(_) | CalcResult::Array(_))
}

impl NumberMark {
    /// Starts the call evaluating `node`; returns the call it interrupts, for [`Self::end`].
    pub(crate) fn begin(&mut self, node: &Node) -> Call {
        let call = (self.next, address(node));
        self.next += 1;
        std::mem::replace(&mut self.current, call)
    }

    /// Whether the latest marked call was made since `since` (a [`Self::next_serial`]) and
    /// evaluated `node`.
    fn is_marked(&self, since: u64, node: &Node) -> bool {
        self.marked
            .is_some_and(|(serial, at)| serial >= since && at == address(node))
    }

    /// The serial the next call gets.
    pub(crate) fn next_serial(&self) -> u64 {
        self.next
    }
}

impl Model {
    /// Ends the call [`NumberMark::begin`] started for `node` (ungrouped), whose value is `value`,
    /// and resumes `outer`: the call is marked when `node` makes the mark, reads a marked LET
    /// name or LAMBDA parameter, or passes on a mark handed to it ([`Self::pass_number_mark`]).
    pub(crate) fn end_number_mark(&mut self, node: &Node, value: &CalcResult, outer: Call) {
        let mark = &mut self.number_mark;
        let call = std::mem::replace(&mut mark.current, outer);
        let handed = mark.marked == Some(call);
        let marked = can_carry(value)
            && (makes_mark(node)
                || (handed && passes_mark(node))
                || self.variable_stack.is_marked_variable(node));
        let mark = &mut self.number_mark;
        if marked {
            mark.marked = Some(call);
        } else if handed {
            mark.marked = None;
        }
    }

    /// The value of `node` and whether it carries the mark.
    pub(crate) fn evaluate_marked(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> (CalcResult, bool) {
        let since = self.number_mark.next_serial();
        let value = self.evaluate_node_in_context(node, cell);
        let marked = self.number_mark.is_marked(since, node);
        (value, marked)
    }

    /// Hands the mark of `node`, evaluated since `since` (a [`NumberMark::next_serial`]), to the
    /// running call, which keeps it if it is one that [`passes_mark`]. Called by LET, IF, CHOOSE,
    /// IFERROR, INDEX and a LAMBDA call right after reading the argument they return.
    pub(crate) fn pass_number_mark(&mut self, since: u64, node: &Node) {
        let mark = &mut self.number_mark;
        if mark.is_marked(since, node) {
            mark.marked = Some(mark.current);
        }
    }

    /// [`Self::evaluate_node_in_context`] of the argument the running call returns unchanged,
    /// handing its mark on ([`Self::pass_number_mark`]).
    pub(crate) fn evaluate_passing_mark(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let since = self.number_mark.next_serial();
        let value = self.evaluate_node_in_context(node, cell);
        self.pass_number_mark(since, node);
        value
    }

    /// Records whether `binding`, which [`Self::bind_argument`] made of `node` from calls since
    /// `since`, holds a marked value.
    pub(crate) fn record_bound_mark(&mut self, since: u64, node: &Node, binding: &Binding) {
        // A name bound to a marked name is marked too: `LET(x,ERROR.TYPE(#REF!),y,x,y)`.
        self.number_mark.bound = matches!(binding, Binding::Value(value) if can_carry(value))
            && (self.number_mark.is_marked(since, node)
                || self.variable_stack.is_marked_variable(node.ungrouped()));
    }

    /// Whether the last binding made holds a marked value; clears it for the next.
    pub(crate) fn take_bound_mark(&mut self) -> bool {
        std::mem::take(&mut self.number_mark.bound)
    }
}
