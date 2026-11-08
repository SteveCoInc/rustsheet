//! A written-empty argument (`F(x,)`, `F(,x)`, or a LET or LAMBDA variable bound to nothing),
//! told apart from a value by the formula's text: the node, not what it evaluates to. IF, CHOOSE,
//! IFERROR and IFNA hand a written-empty branch they return on as one, so for them the branch
//! they pick decides ([`Model::is_empty_argument`]). The value-level reading of the dynamic-array
//! lookups' modes is `mode_or_default` (`lookup_and_reference/xmatch.rs`, T-4871).

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::{math_and_trigonometry::RANDOM_DRAWS, Function},
    model::Model,
    reference_operators::Picked,
};

/// How many probes of a wrapper's pick ([`Model::is_empty_argument`]) may run inside one another.
/// A probe evaluates the condition, index or IFERROR value it picks by, and the call evaluates it
/// again, so a reader inside that condition that probes too doubles the work at each level. Past
/// this depth a reader reads only the node, as before T-4912: a formula nesting readers in the
/// conditions of written-empty wrappers more deeply than this reads the omission as a value there.
/// Excel has no such limit: DEVIATION (CLAUDE.md directive 5, decision 0014; numbers.md "Which
/// wrappers hand a written-empty branch on as an omission").
const MAX_OMISSION_PROBES: u8 = 2;

/// Whether `node` (ungrouped) is an IF, CHOOSE, IFERROR or IFNA call one of whose branches is
/// written empty, there or in such a call nested in it: the only calls that may hand an omission
/// on, so the only ones [`Model::is_empty_argument`] evaluates anything for. A variable bound to
/// nothing as a branch is not looked for: it is rare, and the probe would miss only it.
fn may_hand_omission(node: &Node) -> bool {
    let Node::FunctionKind { kind, args } = node else {
        return false;
    };
    let branches = match kind {
        Function::If | Function::Choose => args.get(1..).unwrap_or_default(),
        Function::Iferror | Function::Ifna => args.as_slice(),
        _ => return false,
    };
    branches.iter().any(|branch| {
        let branch = branch.ungrouped();
        matches!(branch, Node::EmptyArgKind) || may_hand_omission(branch)
    })
}

impl Model {
    /// Whether the argument `node` is empty as written: an empty argument (`f(1,)`), a variable
    /// bound to an omitted LAMBDA parameter or an empty LET value, which hands the emptiness on
    /// (probes/eng-imsub-imdiv-lambda-let-empty, t4873-empty-arg-let-lambda), or an IF, CHOOSE,
    /// IFERROR or IFNA (in parentheses or under a unary plus too) whose picked branch is one of
    /// these: `MROUND(IF(TRUE,),1)`, `MROUND(IFERROR(1/0,),1)` and `MROUND(IF(TRUE,IF(TRUE,)),1)`
    /// are `#N/A` (probes/t4901-mround-blank-let-lambda-roundings A3, A10, A12:A16,
    /// t5067-written-empty-branch-wrappers, t4912-written-empty-branch-readers). A formula that
    /// evaluates to a blank (`IFERROR(<blank>,0)`, `IF(TRUE,Z1,)`) is not one, nor are IFS, SWITCH,
    /// an operator or IF over an array condition (t5067 A4:A10, A13, A17:A19).
    ///
    /// For a wrapper the condition, index or IFERROR value is evaluated here, before the caller
    /// evaluates the argument: a parameter read once there is copied first, as IF over an array
    /// condition does, so the second read finds it ([`MAX_OMISSION_PROBES`] bounds the rest).
    pub(crate) fn is_empty_argument(&mut self, node: &Node, cell: CellReferenceIndex) -> bool {
        let node = node.ungrouped();
        if matches!(node, Node::EmptyArgKind) || self.is_omitted_variable(node) {
            return true;
        }
        if self.omission_probes >= MAX_OMISSION_PROBES || !may_hand_omission(node) {
            return false;
        }
        // The probe evaluates what the wrapper picks by, and the call evaluates it again: a random
        // draw there, or a call whose body the walk cannot see (a name call, a defined name), could
        // pick differently the second time and give a value Excel, which evaluates the wrapper once,
        // cannot (`IF(RAND()<0.5,,1)` read as 0). Such a wrapper is not probed, so its written-empty
        // branch reads as a value, 0: a known gap until the pick is read once (TODO(T-5099); numbers.md
        // "Which wrappers hand a written-empty branch on as an omission", Ours). A LAMBDA written in
        // the formula is walked like any other node.
        if node.contains_function(&RANDOM_DRAWS)
            || node.contains_node(|n| {
                matches!(
                    n,
                    Node::NamedFunctionKind { .. } | Node::DefinedNameKind(..)
                )
            })
        {
            return false;
        }
        self.variable_stack.copy_read_once();
        self.omission_probes += 1;
        let handed = self.picks_omission(node, cell);
        self.omission_probes -= 1;
        handed
    }

    /// Whether the wrapper `node` ([`may_hand_omission`]) picks a branch that is written empty or
    /// hands an omission on itself: IF and CHOOSE through their own pick steps, IFERROR and IFNA
    /// when their value hands one on or is a scalar error they replace with a fallback that does
    /// (`MROUND(IFNA(1/0,),1)` is `#DIV/0!`, t4912 A18). The value is read as `if_error_like`
    /// reads it, but without handing on its ERROR.TYPE/CELL mark, which only the call's own
    /// evaluation does.
    fn picks_omission(&mut self, node: &Node, cell: CellReferenceIndex) -> bool {
        let Node::FunctionKind { kind, args } = node else {
            return false;
        };
        let handed = |model: &mut Self, branch: &Node| {
            let branch = branch.ungrouped();
            matches!(branch, Node::EmptyArgKind)
                || model.is_omitted_variable(branch)
                || (may_hand_omission(branch) && model.picks_omission(branch, cell))
        };
        match kind {
            Function::If | Function::Choose => match self.pick_step(kind, args, cell) {
                Some(Picked::Node(arm)) => handed(self, arm),
                _ => false,
            },
            Function::Iferror | Function::Ifna => {
                let [value, fallback] = args.as_slice() else {
                    return false;
                };
                if handed(self, value) {
                    return true;
                }
                let fallback_may = matches!(fallback.ungrouped(), Node::EmptyArgKind)
                    || may_hand_omission(fallback.ungrouped());
                if !fallback_may || self.is_sheet_span_name(value.ungrouped()) {
                    return false;
                }
                match self.evaluate_node_in_context(value, cell) {
                    CalcResult::Error { error, .. } if replaces_error(kind, &error) => {
                        handed(self, fallback)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// [`Model::is_empty_argument`] for each of the first `N` arguments (`false` past the last).
    pub(crate) fn empty_arguments<const N: usize>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> [bool; N] {
        let mut empty = [false; N];
        for (slot, node) in empty.iter_mut().zip(args) {
            *slot = self.is_empty_argument(node, cell);
        }
        empty
    }
}

/// Whether IFERROR or IFNA (`kind`) replaces `error` with its fallback: IFERROR every error, IFNA
/// only `#N/A`.
pub(crate) fn replaces_error(kind: &Function, error: &Error) -> bool {
    *kind == Function::Iferror || *error == Error::NA
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
