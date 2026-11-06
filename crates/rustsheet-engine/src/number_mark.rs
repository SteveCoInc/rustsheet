//! The "mark" of a number from ERROR.TYPE or `CELL("col"|"row",..)`: some array-taking arguments
//! reject it where the same number from anywhere else is accepted. F.TEST, the paired functions,
//! CHITEST and SUMXMY2 give `#NUM!` for it, LINEST and IRR's flows `#VALUE!` (statistics.md
//! "ERROR.TYPE and CELL results are #NUM! in the paired functions", financial.md "IRR's error
//! order"). LET, IF, CHOOSE, IFERROR, IFNA, INDEX, a LAMBDA call and a called name hand it on,
//! nested in any order and for an array too; `+0`, `@`, SINGLE, MAX, IFS, SWITCH, IF over an
//! array condition and a cell holding the value drop it (probes/t2576-error-type-cell-paired
//! D1:D28, t2766-mark-positions, t2766-mark-positions-2). Why Excel does this is unknown.
//!
//! The mark is a flag on a value, not a form of the formula: it is kept beside the evaluation,
//! not in [`CalcResult`]. An evaluation of a node is bracketed ([`Model::marking`]): each
//! [`Model::evaluate_node_in_context`] call is, and so are the reference-context entry points
//! that read a node without it (`evaluate_node_with_reference`, `evaluate_reference_arg`,
//! `evaluate_picked_reference_arg`), so a LET, CHOOSE or LAMBDA call read for its reference is
//! bracketed too. Each bracket gets a serial number; [`NumberMark`] keeps the bracket running now
//! and the one that finished last, with whether its value carries the mark. A bracket's value
//! carries it when its node makes the mark, reads a marked LET name or LAMBDA parameter, or is
//! one of the kinds in [`passes_mark`] and was handed the mark of the argument it returns
//! ([`Model::passing_mark`]). Since the bracket of an evaluation is the last to finish within it,
//! "the evaluation just made carries the mark" is a check of the last bracket's serial and node
//! ([`Model::evaluate_marked`]): exact under recursion, where the same node is evaluated again
//! inside itself. Constant memory: a few serials and flags.

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
    reference_operators::RefArg,
    variable_stack::Binding,
};

/// The bracket running now: its serial, shifted left one bit, and in the low bit whether it was
/// handed a mark. One word, as it is saved across every `evaluate_node_in_context` call.
#[derive(Clone, Copy, Default)]
pub(crate) struct Frame(u64);

impl Frame {
    fn serial(self) -> u64 {
        self.0 >> 1
    }

    fn handed(self) -> bool {
        self.0 & 1 == 1
    }
}

/// The bracket that finished last.
#[derive(Clone, Copy, Default)]
struct Finished {
    serial: u64,
    /// The address of the node it evaluated (ungrouped).
    node: usize,
    marked: bool,
}

/// Where the mark is (module docs).
#[derive(Clone, Copy, Default)]
pub(crate) struct NumberMark {
    /// The serial the next bracket gets.
    next: u64,
    current: Frame,
    last: Finished,
    /// Whether a bracket carrying the mark has finished in this calculation pass. Until one has,
    /// no value is marked, so a bracket's end only looks for one that makes the mark and keeps
    /// no record: a workbook without ERROR.TYPE, `CELL("col"|"row")` or a LET-bound array of
    /// references pays two counter updates a node.
    armed: bool,
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
/// CHOOSE, IFERROR, IFNA, INDEX, a LAMBDA called in place and a called name
/// (probes/t2576-error-type-cell-paired D1:D5, D19, D20; t2766-mark-positions A23, A28;
/// t2766-mark-positions-2 A1:A15). IFS and SWITCH drop it (`FTEST({1;2;3},IFS(TRUE,
/// ERROR.TYPE(#REF!)))` is `#DIV/0!`, t2766-mark-positions A24, A25).
fn passes_mark(node: &Node) -> bool {
    matches!(
        node,
        Node::FunctionKind {
            kind: Function::Let
                | Function::If
                | Function::Choose
                | Function::Iferror
                | Function::Ifna
                | Function::Index,
            ..
        } | Node::LambdaCallKind { .. }
            | Node::NamedFunctionKind { .. }
    )
}

/// A result that may carry the mark: a number, or an array (`ERROR.TYPE({#REF!})`, D24; an
/// array of references read through a LET name, probes/t2577-indirect-ref-array A64).
pub(crate) trait MarkCarrier {
    /// Whether this value can carry the mark.
    fn can_carry(&self) -> bool;
}

impl MarkCarrier for CalcResult {
    fn can_carry(&self) -> bool {
        matches!(self, CalcResult::Number(_) | CalcResult::Array(_))
    }
}

impl MarkCarrier for RefArg {
    fn can_carry(&self) -> bool {
        matches!(self, RefArg::Value(value) if value.can_carry())
    }
}

impl NumberMark {
    /// Starts a bracket; returns the one it interrupts, for [`Model::end_number_mark`].
    #[inline]
    pub(crate) fn begin(&mut self) -> Frame {
        let frame = Frame(self.next << 1);
        self.next += 1;
        std::mem::replace(&mut self.current, frame)
    }

    /// Whether a bracket of `node` begun at or after `since` (a [`Self::next_serial`]) is the
    /// last to have finished and carries the mark: the evaluation of `node` made since `since`
    /// carries it.
    fn was_marked(&self, since: u64, node: &Node) -> bool {
        self.last.marked && self.last.serial >= since && self.last.node == address(node)
    }

    /// The serial the next bracket gets.
    pub(crate) fn next_serial(&self) -> u64 {
        self.next
    }
}

impl Model {
    /// Ends the bracket [`NumberMark::begin`] started for `node`, whose value `can_carry` says
    /// may carry the mark, and resumes `outer`.
    #[inline]
    pub(crate) fn end_number_mark(&mut self, node: &Node, can_carry: bool, outer: Frame) {
        let frame = std::mem::replace(&mut self.number_mark.current, outer);
        // Only a number or an array is ever marked.
        if can_carry || self.number_mark.armed {
            self.record_number_mark(node, can_carry, frame);
        }
    }

    /// [`Self::end_number_mark`] of a bracket that may carry the mark or may need recording.
    #[inline(never)]
    fn record_number_mark(&mut self, node: &Node, can_carry: bool, frame: Frame) {
        let node = node.ungrouped();
        let marked = can_carry
            && match node {
                Node::NamedVariableKind { id: Some(id), .. } => {
                    self.variable_stack.is_marked(*id as usize)
                }
                // Nothing was handed on before the first mark.
                _ if !self.number_mark.armed => makes_mark(node),
                // Read twice over (a reference-context entry point, then
                // `evaluate_node_in_context`), the inner bracket's mark is this one's.
                _ if passes_mark(node) => {
                    let last = self.number_mark.last;
                    frame.handed()
                        || (last.marked
                            && last.node == address(node)
                            && last.serial > frame.serial())
                }
                _ => makes_mark(node),
            };
        let mark = &mut self.number_mark;
        mark.armed |= marked;
        if mark.armed {
            mark.last = Finished {
                serial: frame.serial(),
                node: address(node),
                marked,
            };
        }
    }

    /// `evaluate(self)`, the evaluation of `node`, bracketed (module docs).
    pub(crate) fn marking<T: MarkCarrier>(
        &mut self,
        node: &Node,
        evaluate: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let outer = self.number_mark.begin();
        let result = evaluate(self);
        self.end_number_mark(node, result.can_carry(), outer);
        result
    }

    /// The value of `node` and whether it carries the mark.
    pub(crate) fn evaluate_marked(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> (CalcResult, bool) {
        let since = self.number_mark.next_serial();
        let value = self.evaluate_node_in_context(node, cell);
        (value, self.number_mark.was_marked(since, node))
    }

    /// `read(self)`, the reading of `node`, the argument the running bracket returns unchanged,
    /// handing on its mark: LET's body, the branch IF picks, the argument CHOOSE picks, IFERROR's
    /// value or fallback, INDEX's array and a LAMBDA's body. The bracket keeps it if its node is
    /// one that [`passes_mark`].
    pub(crate) fn passing_mark<T>(&mut self, node: &Node, read: impl FnOnce(&mut Self) -> T) -> T {
        let since = self.number_mark.next_serial();
        let value = read(self);
        self.hand_on_mark(since, node);
        value
    }

    /// Hands the running bracket the mark of `node`, read since `since`, if it carries it.
    fn hand_on_mark(&mut self, since: u64, node: &Node) {
        if self.number_mark.was_marked(since, node) {
            self.number_mark.current.0 |= 1;
        }
    }

    /// [`Self::evaluate_node_in_context`] of `node` handing its mark on only when it is a
    /// number: IFERROR and IFNA rebuild an array, which drops it
    /// (probes/t2766-mark-positions A27).
    pub(crate) fn evaluate_passing_number_mark(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let since = self.number_mark.next_serial();
        let value = self.evaluate_node_in_context(node, cell);
        if matches!(value, CalcResult::Number(_)) {
            self.hand_on_mark(since, node);
        }
        value
    }

    /// [`Self::evaluate_node_in_context`] of `node` through [`Self::passing_mark`].
    pub(crate) fn evaluate_passing_mark(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.passing_mark(node, |model| model.evaluate_node_in_context(node, cell))
    }

    /// `bind(self)`, the binding of a LET value or LAMBDA argument `node`, and whether the value
    /// bound carries the mark. A name bound to a marked name is marked too:
    /// `LET(x,ERROR.TYPE(#REF!),y,x,y)` (probes/t2766-mark-positions A26).
    pub(crate) fn binding_marked(
        &mut self,
        node: &Node,
        bind: impl FnOnce(&mut Self) -> Binding,
    ) -> (Binding, bool) {
        let since = self.number_mark.next_serial();
        let binding = bind(self);
        let marked = matches!(&binding, Binding::Value(value) if value.can_carry())
            && (self.number_mark.was_marked(since, node)
                || matches!(node.ungrouped(), Node::NamedVariableKind { id: Some(id), .. }
                    if self.variable_stack.is_marked(*id as usize)));
        (binding, marked)
    }
}
