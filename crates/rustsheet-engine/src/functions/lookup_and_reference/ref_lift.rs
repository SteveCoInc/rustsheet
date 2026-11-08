//! What OFFSET, INDIRECT and TRIMRANGE share when they lift into an array of references
//! (SPEC-0014 "Producers"): which argument values lift the call, and their conversion to the
//! driver's arguments; and where IF, CHOOSE, LET and variables hand such an array on (rule 7).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::match_same_arms)]
// LINT-DEBT-END

use std::borrow::Cow;

use super::drop_take::Edge;
use crate::calc_result::{CalcResult, Range};
use crate::cast::calc_result_to_array_node;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::aggregate::value_node;
use crate::functions::lift::{ArgBuf, LiftedArg};
use crate::functions::util::{array_node_to_calc_result, top_left_of_array};
use crate::functions::Function;
use crate::limits::check_array_size;
use crate::model::Model;
use crate::reference_operators::{Picked, Ranges, RefArg, RefElement, RefGrid};

/// The signature of a function implementation, as `Model::evaluate_function` calls it.
type Implementation = fn(&mut Model, &[Node], CellReferenceIndex) -> CalcResult;

/// What one call of a per-element consumer costs (SPEC-0014 "Bounds"), charged for every element
/// to the lift's visit budget (`limits::MAX_LIFTED_CELL_VISITS`) before any element runs.
#[derive(Clone, Copy)]
pub(crate) enum ElementCost {
    /// One visit: ROW, COLUMN, AREAS, CELL, ISFORMULA, N and T read no cells or one.
    Unit,
    /// The cells of the element's references inside the used area, times this many walks:
    /// SUBTOTAL, COUNTIF, SUMIF, AVERAGEIF and COUNTBLANK walk them once, SUMIFS, COUNTIFS,
    /// AVERAGEIFS, MAXIFS and MINIFS once per criteria pair.
    Cells(u64),
}

/// A criteria array that broadcasts with the grids of a per-element call: its argument position
/// and its values ([`PerElement::broadcasts_criteria`]).
type CriteriaGrid = (usize, Vec<Vec<ArrayNode>>);

/// A consumer that calls itself once per element of an array of references (SPEC-0014 rule 6):
/// which of its arguments take a reference, and what an element costs.
#[derive(Clone, Copy)]
pub(crate) struct PerElement {
    /// Whether the argument at this position takes a reference.
    pub(crate) takes_reference: fn(usize) -> bool,
    cost: ElementCost,
    /// Whether an array of values in a criteria position broadcasts with the grids instead of
    /// being handed to every element whole: the criteria functions
    /// (probes/t4742-countif-array-range-arg A40:C40, T-4826).
    pub(crate) broadcasts_criteria: bool,
}

impl Function {
    /// The consumers that call themselves once per element of an array of references given
    /// where they take a reference (SPEC-0014 rule 6), for a call of `count` arguments.
    /// `SUBTOTAL(9,OFFSET(A1,{0;1},0))` spills 1, 100; `ROW(OFFSET(A1,{0;1},0))` 1, 2
    /// (probes/reference-lift-consumers E57-E78, E97, E109, E165, E169, E185; -b1c E85-E93;
    /// -b1d E97; -b2 E125-E133, E177, E193, E197). COUNTIFS, AVERAGEIF and MAXIFS too:
    /// `COUNTIFS(OFFSET(A1,{0;1},0),">0")` spills 1, 1, `AVERAGEIF(OFFSET(A1,0,0,{1;2}),">0")`
    /// 1, 50.5 (probes/t2065-lift-countifs, -averageif, -maxifs). AVERAGEIFS and MINIFS lift
    /// like SUMIFS and MAXIFS: `MINIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")` spills 1,
    /// 100, and a criteria range alone lifts too (probes/t4396-lift-*).
    pub(crate) fn per_reference_element(&self, count: usize) -> Option<PerElement> {
        let first: fn(usize) -> bool = |index| index == 0;
        // One walk per criteria pair (SUMIFS, COUNTIFS, AVERAGEIFS, MAXIFS, MINIFS).
        let per_pair = ElementCost::Cells(u64::try_from(count / 2).unwrap_or(u64::MAX).max(1));
        let (takes_reference, cost): (fn(usize) -> bool, ElementCost) = match self {
            Function::Row
            | Function::Column
            | Function::Areas
            | Function::Isformula
            | Function::N
            | Function::T => (first, ElementCost::Unit),
            Function::Cell => (|index| index == 1, ElementCost::Unit),
            Function::Countif | Function::Countblank => (first, ElementCost::Cells(1)),
            // Every criteria range.
            Function::Countifs => (|index| index % 2 == 0, per_pair),
            Function::Subtotal => (|index| index > 0, ElementCost::Cells(1)),
            // The criteria range and the sum range.
            Function::Sumif | Function::Averageif => (|index| index != 1, ElementCost::Cells(1)),
            // The sum (average, max, min) range and every criteria range.
            Function::Sumifs | Function::Averageifs | Function::Maxifs | Function::Minifs => {
                (|index| index == 0 || index % 2 == 1, per_pair)
            }
            _ => return None,
        };
        let broadcasts_criteria = matches!(
            self,
            Function::Countif
                | Function::Countifs
                | Function::Sumif
                | Function::Sumifs
                | Function::Averageif
                | Function::Averageifs
                | Function::Maxifs
                | Function::Minifs
        );
        Some(PerElement {
            takes_reference,
            cost,
            broadcasts_criteria,
        })
    }
}

/// An absolute reference to `range`, the argument a per-element call reads in place of the
/// array of references it came from, or of a reference argument read once.
pub(crate) fn range_node(range: &Range) -> Node {
    Node::RangeKind {
        sheet_name: None,
        sheet_index: range.left.sheet,
        absolute_row1: true,
        absolute_column1: true,
        row1: range.left.row,
        column1: range.left.column,
        absolute_row2: true,
        absolute_column2: true,
        row2: range.right.row,
        column2: range.right.column,
    }
}

/// Whether `node` is an OFFSET or INDIRECT call, which lifts into an array of references when an
/// argument lifts it: the only argument a per-element consumer reads ahead of its call (SPEC-0014
/// rule 6). A TRIMRANGE is not, as SUBTOTAL and AREAS read one as the cells it picks
/// (`SUBTOTAL(9,TRIMRANGE(A1:A2))` is 3, probes/subtotal-reference-operands-b H17); nor are IF,
/// CHOOSE and LET handing a grid on (unprobed).
pub(crate) fn lifts_into_references(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::FunctionKind {
            kind: Function::Offset | Function::Indirect,
            ..
        }
    )
}

/// The arguments a per-element consumer reads ahead of its call (SPEC-0014 rule 6): those of
/// [`lifts_into_references`], and an XLOOKUP, which a lifted `lookup_value` makes an array of
/// its elements, a range `if_not_found` the reference element of each miss (SPEC-0014 amendment
/// rule 6; `COUNTIF(XLOOKUP({2;9},A1:A3,B1:B3,G1:G2),">0")` spills `#VALUE!`, 2;
/// probes/t4569-xlookup-ref-element-max-min A26, C24:D34), any other element a value or an
/// error ([`Model::xlookup_reference_elements`]).
pub(crate) fn lifts_into_reference_elements(node: &Node) -> bool {
    match node.ungrouped() {
        Node::FunctionKind {
            kind: Function::Xlookup,
            ..
        } => true,
        node => lifts_into_references(node),
    }
}

/// A node that reads again what `read` holds at no more cost than its cells: one range as an
/// absolute reference, several areas as their union, an array of references as its value view
/// (`#VALUE!` per reference element), an array as a literal, a scalar as a literal. `None` for a
/// 3-D range, an empty value and a LAMBDA.
pub(super) fn put_back(read: &RefArg) -> Option<Node> {
    match read {
        RefArg::Ranges(Ranges::One(range)) if range.left.sheet == range.right.sheet => {
            Some(range_node(range))
        }
        RefArg::Ranges(Ranges::One(_)) => None,
        RefArg::Ranges(areas) => union_node(areas),
        RefArg::Grid(grid) => Some(Node::ArrayKind(
            grid.to_array(ArrayNode::Error(Error::VALUE)),
        )),
        RefArg::Value(CalcResult::Array(rows)) => Some(Node::ArrayKind(rows.clone())),
        RefArg::Value(CalcResult::EmptyCell | CalcResult::EmptyArg) => None,
        RefArg::Value(value) => value_node(value),
    }
}

/// The union of `areas` as a balanced tree of absolute references, so reading it nests no
/// deeper than the logarithm of the count.
fn union_node(areas: &[Range]) -> Option<Node> {
    match areas {
        [] => None,
        [range] => Some(range_node(range)),
        _ => {
            let (left, right) = areas.split_at(areas.len() / 2);
            Some(Node::OpUnionKind {
                left: Box::new(union_node(left)?),
                right: Box::new(union_node(right)?),
                bare: false,
            })
        }
    }
}

/// Whether `node` is read again at no more cost than its cells: a literal or a written
/// reference.
fn reads_cheaply(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::NumberKind(_)
            | Node::StringKind(_)
            | Node::BooleanKind(_)
            | Node::ErrorKind(_)
            | Node::EmptyArgKind
            | Node::ArrayKind(_)
            | Node::ImplicitIntersection { .. }
            | Node::ReferenceKind { .. }
            | Node::RangeKind { .. }
            | Node::ThisRowKind { .. }
    )
}

/// The element of `grid` at (`row`, `column`) of a broadcast: a grid of one row or one column
/// repeats it, and a position past a shorter grid is `None`.
fn broadcast_element(grid: &RefGrid, row: usize, column: usize) -> Option<&RefElement> {
    let (rows, columns) = grid.dims();
    let at = |i: usize, n: usize| if n == 1 { 0 } else { i };
    grid.get(at(row, rows), at(column, columns))
}

/// Cells in an array value: what reading it once more costs.
fn array_cells(rows: &[Vec<ArrayNode>]) -> u64 {
    rows.iter()
        .map(|row| u64::try_from(row.len()).unwrap_or(u64::MAX))
        .fold(0, u64::saturating_add)
}

/// Whether an argument's value lifts TRIMRANGE: an array or a range of more than one cell. A
/// one-cell range and a 1x1 array are their value, as before (whether Excel lifts a 1x1 array
/// there is unprobed, TODO(T-2347); OFFSET, INDIRECT, TAKE and DROP do, [`lifts_one_by_one`]).
pub(super) fn lifts(value: &CalcResult) -> bool {
    match value {
        CalcResult::Range { left, right } => left != right,
        CalcResult::Array(rows) => rows.len() > 1 || rows.first().is_some_and(|row| row.len() > 1),
        _ => false,
    }
}

/// Whether an argument's value lifts OFFSET, INDIRECT, TAKE or DROP: what [`lifts`] accepts, and
/// a 1x1 array too, which gives one element: `TAKE(A1:B3,{2})` is one element, A1:B2, so ROWS of
/// it is 1 and spilled directly it is `#VALUE!` (probes/m7-lift-take-drop-sortby Sheet1 F13, G13,
/// I13:K13, W65); ROWS of `DROP(A1:B3,{1})` and of `OFFSET(A1:B2,{1},0)` is 1 too
/// (probes/m7-one-cell-operand-lift F6, F7, F16, F17; T-2298), and `INDIRECT("B1",{1})` is
/// `#VALUE!` (probes/t2577-indirect-ref-array A1, A25; T-2577). A one-cell range is still its
/// value.
pub(super) fn lifts_one_by_one(value: &CalcResult) -> bool {
    lifts(value) || matches!(value, CalcResult::Array(_))
}

/// Whether `node` may stand for an array of references (SPEC-0014 rule 7): a call to a producer
/// (OFFSET, INDIRECT, TRIMRANGE), an IF, CHOOSE or LET that may hand one on, or a variable. Read
/// without evaluating, so a consumer asks for the grid only where one can come.
pub(super) fn may_hand_on_grid(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::NamedVariableKind { .. }
            | Node::FunctionKind {
                kind: Function::Offset
                    | Function::Indirect
                    | Function::Trimrange
                    | Function::Take
                    | Function::Drop
                    | Function::If
                    | Function::Choose
                    | Function::Let,
                ..
            }
    )
}

impl Model {
    /// The reference arguments of `args` (positions `takes_reference`) that are a lifting call
    /// ([`lifts_into_references`]), each read once: each one that is an array of references is
    /// returned with its position, and every other one is put back as what it gave
    /// ([`put_back`]), so the call never evaluates it again (a nest of such calls stays linear).
    /// What cannot be put back (a 3-D range, an empty value) is `#VALUE!`, as one sheet's
    /// reference is wanted.
    pub(crate) fn reference_grid_args<'a>(
        &mut self,
        args: &'a [Node],
        takes_reference: fn(usize) -> bool,
        cell: CellReferenceIndex,
    ) -> (Cow<'a, [Node]>, Vec<(usize, RefGrid)>) {
        let mut resolved = Cow::Borrowed(args);
        let mut grids = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            if !takes_reference(index) || !lifts_into_reference_elements(arg) {
                continue;
            }
            // An XLOOKUP is read in its reference form, so one that gives no grid is put back
            // as the reference it picks, as the call reads it (`ROW(XLOOKUP(2,A1:A3,B1:B3,C1))`).
            let read = match arg.ungrouped() {
                Node::FunctionKind {
                    kind: Function::Xlookup,
                    args,
                } => self.xlookup_reference_elements(args, cell),
                _ => self.evaluate_reference_arg(arg, cell),
            };
            if let RefArg::Grid(grid) = read {
                grids.push((index, grid));
            } else if let Some(slot) = resolved.to_mut().get_mut(index) {
                *slot = put_back(&read).unwrap_or(Node::ErrorKind(Error::VALUE));
            }
        }
        (resolved, grids)
    }

    /// Calls the implementation of `kind`: through [`Self::call_per_reference_element`] for a
    /// per-element consumer ([`Function::per_reference_element`]), directly otherwise. Every
    /// built-in call's value leaves here, so this is where an array element's error tag is
    /// dropped ([`CalcResult::untagged`], T-3080).
    pub(crate) fn call_by_reference_elements(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
        implementation: Implementation,
    ) -> CalcResult {
        let value = match kind.per_reference_element(args.len()) {
            Some(per_element) => {
                self.call_per_reference_element(args, per_element, cell, implementation)
            }
            None => implementation(self, args, cell),
        };
        // After its arguments ran, the call may empty AMORDEGRC's left-over value
        // (`Function::empties_amor_left_over`; probes/amordegrc-fraction-state-cx to -da).
        if kind.empties_amor_left_over() {
            self.empty_amor_left_over();
        }
        value.untagged()
    }

    /// Calls `implementation` once per element of every array of references among its reference
    /// arguments (SPEC-0014 rule 6), the element in the argument's place, and spills the results
    /// in the grids' broadcast shape (a position past a shorter grid is `#N/A`); an element that
    /// is an error is that error in the argument's place, and an element's array result is its
    /// top-left value. Without such an argument it is one call.
    ///
    /// The other arguments are the same for every element, so each is read once, before any
    /// element runs, and put back as a literal, an array or an absolute reference; then the shape
    /// (`check_array_size`) and the cost of every element are checked (directive 5): the cells
    /// its references walk ([`ElementCost`]) and those the other arguments read. An argument that
    /// cannot be put back (a 3-D range, a LAMBDA, an empty value) makes the call `#VALUE!`
    /// (unprobed in Excel). Out of line: inlined into `evaluate_function`, its locals made the
    /// frame of every function call, and of every nested LAMBDA call, larger (T-0444).
    #[inline(never)]
    pub(crate) fn call_per_reference_element(
        &mut self,
        args: &[Node],
        per_element: PerElement,
        cell: CellReferenceIndex,
        implementation: Implementation,
    ) -> CalcResult {
        let (resolved, grids) = self.reference_grid_args(args, per_element.takes_reference, cell);
        if grids.is_empty() {
            return implementation(self, &resolved, cell);
        }
        let walks = match per_element.cost {
            ElementCost::Unit => 0,
            ElementCost::Cells(walks) => walks,
        };
        let mut element_args = resolved.into_owned();
        // What the arguments that are not grids cost each element.
        let mut shared: u64 = 1;
        for (index, arg) in element_args.iter_mut().enumerate() {
            if grids.iter().any(|(at, _)| *at == index) {
                continue;
            }
            let Some(cells) = self.read_once(arg, cell) else {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "An argument beside an array of references".to_string(),
                );
            };
            // A reference argument is walked; a value argument is read.
            let reads = if (per_element.takes_reference)(index) {
                cells.saturating_mul(walks)
            } else {
                cells
            };
            shared = shared.saturating_add(reads);
        }
        // A criteria array beside the grids broadcasts with them, each element taking its own
        // criterion: `COUNTIF(XLOOKUP({2;9},..,"x"),{">0",">5"})` is 2x2 (probes/
        // t4742-countif-array-range-arg A40:C40, T-4826; probes/t4826-criteria-array-beside-grid
        // shows the shapes and that every element is `#VALUE!`, so the pairing of grid and
        // criterion elements is UNVERIFIED).
        let values: Vec<CriteriaGrid> = if per_element.broadcasts_criteria {
            element_args
                .iter()
                .enumerate()
                .filter(|(index, _)| !(per_element.takes_reference)(*index))
                .filter_map(|(index, arg)| match arg {
                    Node::ArrayKind(rows)
                        if rows.len() > 1 || rows.first().is_some_and(|r| r.len() > 1) =>
                    {
                        Some((index, rows.clone()))
                    }
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        };
        let (rows, columns) = grids
            .iter()
            .map(|(_, grid)| grid.dims())
            .chain(
                values
                    .iter()
                    .map(|(_, rows)| (rows.len(), rows.first().map_or(0, Vec::len))),
            )
            .fold((1, 1), |(rows, columns), (r, c)| {
                (rows.max(r), columns.max(c))
            });
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        let mut visits: u64 = 0;
        for row in 0..rows {
            for column in 0..columns {
                let mut element_cost = shared;
                for (_, grid) in &grids {
                    if let (Some(RefElement::Reference(range)), true) =
                        (broadcast_element(grid, row, column), walks > 0)
                    {
                        let value = CalcResult::Range {
                            left: range.left,
                            right: range.right,
                        };
                        let cells = self.range_walk_cost(&value);
                        element_cost = element_cost.saturating_add(cells.saturating_mul(walks));
                    }
                }
                visits = visits.saturating_add(element_cost);
            }
        }
        if let Some(error) = self.spend_visits(visits, cell) {
            return error;
        }
        let mut output = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut output_row = Vec::with_capacity(columns);
            for column in 0..columns {
                output_row.push(self.call_on_element(
                    &mut element_args,
                    (&grids, &values),
                    (row, column),
                    cell,
                    implementation,
                ));
            }
            output.push(output_row);
        }
        CalcResult::Array(output)
    }

    /// One element's call: each grid's element at `at` put in its argument's place.
    fn call_on_element(
        &mut self,
        element_args: &mut [Node],
        (grids, values): (&[(usize, RefGrid)], &[CriteriaGrid]),
        (row, column): (usize, usize),
        cell: CellReferenceIndex,
        implementation: Implementation,
    ) -> ArrayNode {
        for (index, array) in values {
            let at = |i: usize, n: usize| if n == 1 { 0 } else { i };
            let element = array
                .get(at(row, array.len()))
                .and_then(|r| r.get(at(column, r.len())));
            let (Some(element), Some(slot)) = (element, element_args.get_mut(*index)) else {
                return ArrayNode::Error(Error::NA);
            };
            *slot = value_node(&array_node_to_calc_result(element, cell))
                .unwrap_or(Node::ErrorKind(Error::VALUE));
        }
        for (index, grid) in grids {
            let (Some(element), Some(slot)) = (
                broadcast_element(grid, row, column),
                element_args.get_mut(*index),
            ) else {
                return ArrayNode::Error(Error::NA);
            };
            *slot = match element {
                RefElement::Reference(range) => range_node(range),
                RefElement::Error(error) => Node::ErrorKind(error.clone()),
                // A value element is the scalar argument it is: ROW of it is `#VALUE!`, N of it
                // the value (SPEC-0014 amendment rule 6; probes/t4569-xlookup-ref-element-max-min
                // C24:D34).
                RefElement::Value(value) => value_node(&array_node_to_calc_result(value, cell))
                    .unwrap_or(Node::ErrorKind(Error::VALUE)),
            };
        }
        match implementation(self, element_args, cell) {
            CalcResult::Array(array) => calc_result_to_array_node(top_left_of_array(&array, cell)),
            value => calc_result_to_array_node(value),
        }
    }

    /// Reads an argument that is the same for every element once, and puts it back as what it
    /// gave ([`put_back`]). Returns the cells a further read costs (a reference's cells inside
    /// the used area, an array's cells, 1 for a literal), or `None` when the value cannot be put
    /// back.
    fn read_once(&mut self, arg: &mut Node, cell: CellReferenceIndex) -> Option<u64> {
        let value = if reads_cheaply(arg) {
            self.evaluate_node_with_reference(arg, cell)
        } else {
            let value = self.evaluate_node_in_context(arg, cell);
            let read = RefArg::from(value.clone());
            *arg = put_back(&read)?;
            value
        };
        Some(match &value {
            CalcResult::Range { left, right } if left.sheet != right.sheet => return None,
            CalcResult::Range { .. } => self.range_walk_cost(&value),
            CalcResult::Array(rows) => array_cells(rows).max(1),
            _ => 1,
        })
    }

    /// A value argument whose consumer reads an array of references by its elements (SPEC-0014
    /// "Consumers"): a lifted producer is its grid, anything else its value as
    /// `evaluate_node_in_context` gives it, a range as `RefArg::Ranges`. The producers are
    /// OFFSET, INDIRECT, TRIMRANGE, TAKE and DROP ([`Model::reference_producer`]); IF, CHOOSE,
    /// LET and a variable hand a producer's grid on, as they do to an aggregate:
    /// `SUMPRODUCT(IF(TRUE,OFFSET(A1,{0;1},0)))` is 0 as the bare OFFSET is (rule 7;
    /// probes/reference-lift-pass-through E5-E7). Each picks its argument as its value form does
    /// (`if_branch_value`, `choose_arm_value`, LET's body), so nothing else changes.
    pub(crate) fn evaluate_value_or_grid(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        self.value_or_grid(node, cell, false)
    }

    /// [`Self::evaluate_value_or_grid`], where `keep_cell_variables` reads a variable bound to
    /// one cell as that reference ([`Model::cell_variable_reference`]), also as the arm IF or
    /// CHOOSE picks or a LET body: TAKE's and DROP's array (T-2622).
    pub(crate) fn value_or_grid(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        keep_cell_variables: bool,
    ) -> RefArg {
        match node.ungrouped() {
            Node::FunctionKind {
                kind:
                    kind @ (Function::Offset
                    | Function::Indirect
                    | Function::Trimrange
                    | Function::Take
                    | Function::Drop),
                args,
            } => self.reference_producer(kind, args, cell),
            // SUMPRODUCT reads XLOOKUP's reference element as 0 (SPEC-0014 amendment rule 4;
            // probes/t4569-xlookup-ref-element-max-min A21).
            Node::FunctionKind {
                kind: Function::Xlookup,
                args,
            } => self.xlookup_reference(args, cell, false),
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => match self.if_pick(args, cell) {
                Picked::Node(arm) if may_hand_on_grid(arm) => {
                    self.value_or_grid(arm, cell, keep_cell_variables)
                }
                Picked::Node(arm) => RefArg::from(self.if_branch_value(arm, cell)),
                Picked::Value(value) => RefArg::Value(value),
            },
            Node::FunctionKind {
                kind: Function::Choose,
                args,
            } => match self.choose_pick(args, cell) {
                Picked::Node(arm) if may_hand_on_grid(arm) => {
                    self.value_or_grid(arm, cell, keep_cell_variables)
                }
                Picked::Node(arm) => RefArg::from(self.choose_arm_value(arm, cell)),
                Picked::Value(value) => RefArg::Value(value),
            },
            Node::FunctionKind {
                kind: Function::Let,
                args,
            } => self
                .let_scope(args, cell, |model, body| {
                    model.value_or_grid(body, cell, keep_cell_variables)
                })
                .unwrap_or_else(RefArg::Value),
            node @ Node::NamedVariableKind { .. } if keep_cell_variables => {
                match self.cell_variable_reference(node) {
                    Some(reference) => RefArg::from(reference),
                    None => self.value_or_grid(node, cell, false),
                }
            }
            node @ Node::NamedVariableKind { .. } if self.is_grid_variable(node) => {
                self.variable_reference(node, cell)
            }
            _ => RefArg::from(self.evaluate_node_in_context(node, cell)),
        }
    }

    /// OFFSET, INDIRECT, TRIMRANGE, TAKE, DROP or VSTACK read as a reference argument: one
    /// reference (an unlifted TAKE or DROP: its value, as a value argument reads it), or an array
    /// of references when an argument lifts the call, or VSTACK stacks such arrays
    /// ([`Model::vstack_reference`]; otherwise VSTACK's value). `kind` is one of the six.
    pub(crate) fn reference_producer(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> RefArg {
        let value = match kind {
            Function::Indirect => self.indirect_reference(args, cell),
            Function::Trimrange => self.trimrange_reference(args, cell, false),
            Function::Take => self.take_drop_reference(args, cell, Edge::Take, false),
            Function::Drop => self.take_drop_reference(args, cell, Edge::Drop, false),
            Function::Vstack => match self.vstack_reference(args, cell) {
                Some(stacked) => stacked,
                None => RefArg::from(self.fn_vstack(args, cell)),
            },
            _ => self.offset_reference(args, cell),
        };
        // Read as a reference, TAKE, DROP and VSTACK still empty AMORDEGRC's left-over value
        // (`Function::empties_amor_left_over`; probes/amordegrc-fraction-state-bo to -bq).
        if kind.empties_amor_left_over() {
            self.empty_amor_left_over();
        }
        value
    }

    /// The values of `args`, held inline (no heap allocation for the scalar call, SPEC-0014
    /// "Producers"); a written empty argument is `EmptyArg`.
    pub(super) fn reference_lift_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> ArgBuf<CalcResult> {
        self.reference_lift_values_read(args, cell, Self::evaluate_node_in_context)
    }

    /// [`Self::reference_lift_values`] with each argument read by `read`.
    pub(super) fn reference_lift_values_read(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        read: fn(&mut Self, &Node, CellReferenceIndex) -> CalcResult,
    ) -> ArgBuf<CalcResult> {
        let mut values = ArgBuf::filled(args.len(), || CalcResult::EmptyArg);
        for (slot, arg) in values.as_mut_slice().iter_mut().zip(args) {
            if !matches!(arg, Node::EmptyArgKind) {
                *slot = read(self, arg, cell);
            }
        }
        values
    }

    /// The driver's arguments for `values`: those that `lifts_value` accepts ([`lifts`] or
    /// [`lifts_one_by_one`]: the reason it is a parameter) read into their cells, the rest
    /// scalars. An unreadable one (a range across sheets) is the call's error.
    pub(super) fn reference_lift_args(
        &mut self,
        values: ArgBuf<CalcResult>,
        lifts_value: fn(&CalcResult) -> bool,
        cell: CellReferenceIndex,
    ) -> Result<Vec<LiftedArg>, CalcResult> {
        let mut lifted = Vec::with_capacity(values.as_slice().len());
        for value in values {
            lifted.push(if lifts_value(&value) {
                LiftedArg::read(self, value, cell)?
            } else {
                LiftedArg::Scalar(value)
            });
        }
        Ok(lifted)
    }
}
