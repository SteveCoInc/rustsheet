//! Calculated fields (`cacheField@formula`, `databaseField="0"`; T-4174): a data field over one
//! shows, in every cell, its formula over that cell's sums of the fields it names.
//!
//! The rules are docs/excel-semantics/pivots.md "A calculated field applies its formula to the
//! field sums" and "Calculated fields as Excel for the web renders them" (probes/t4128-pivot-spike
//! K, t4505-pivot-calc-fields, t4505-pivot-calc-field-functions, t4130-pivot-calc-items CF):
//! - every cell, subtotals and grand totals included, computes the formula over its own sums, so
//!   an error stays where its sums make it and never reaches a total;
//! - a field named in the formula is that cell's Sum of it (a text field's is 0), whatever the
//!   field's place in the pivot, and a calculated field named in it is that field's value at the
//!   same cell, one listed after it included; a cycle, or a field that names itself, is `#NAME?`;
//! - the data field's summary function is ignored: Count, Average, Max and Min show the Sum;
//! - the formula's functions see each field as one number, as the worksheet functions see a
//!   number; a text result is `#VALUE!` and a boolean shows as 1 or 0;
//! - an intersection no record reaches shows 0 (LC B7), which the engine reads as the formula over
//!   sums of 0 (UNVERIFIED, T-4897).
//!
//! The formula is read by the engine's own parser, with a quoted name (`'Unit Cost'`) read as a
//! name ([`Parser::set_quoted_names`]), and checked before anything is evaluated: it may name
//! fields of the cache (bare, quoted, or spelled like a cell, `Rev2`) and hold constants,
//! operators and function calls, and nothing else. Functions are a denylist
//! ([`REFUSED_FUNCTIONS`]), not an allowlist: a unit test pins how many functions it lets
//! through, so a new `Function` variant fails it until someone decides whether the sandbox may
//! run it. Excel for the web refuses a file whose
//! calculated field calls NOW, TODAY, RAND or INDIRECT, holds an array constant or a cell
//! reference, or names no field (pivots.md "Pivot flags Excel for the web refuses"), so the
//! import skips such a cache ([`check_calculated_fields`]); the engine also refuses the other
//! volatile, reference and external-data functions (RANDBETWEEN, RANDARRAY, OFFSET, CELL, INFO,
//! GETPIVOTDATA, IMAGE, STOCKHISTORY, FIELDVALUE, FILTERXML) and LET, LAMBDA,
//! ranges, names and calls of unknown functions, which are unprobed (an engine choice:
//! UNVERIFIED, T-4897). Each cell's formula is then the checked tree with every field replaced by the cell's
//! number for it, evaluated by the engine's own evaluator in a sandbox: an empty workbook of its
//! own on a fixed clock, which no cell of the pivot's workbook reaches and which reads no clock,
//! file or network (CLAUDE.md directive 5). Its work is drawn from the pivot's budget
//! ([`super::MAX_PIVOT_WORK`] or the grid writer's `PivotBudget`), and the formulas are bounded
//! in length ([`MAX_CALCULATED_FORMULA_CHARS`]), in number ([`MAX_CALCULATED_FIELDS`]) and, by the
//! parser, in depth; the fields' dependencies are ordered without recursion.

use std::collections::{HashMap, HashSet};

use thiserror::Error as ThisError;

use crate::calc_result::CalcResult;
use crate::clock::Clock;
use crate::expressions::parser::{new_parser_english, Node};
use crate::expressions::token::Error;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::expressions::utils::number_to_column;
use crate::functions::util::collation::fold_case;
use crate::functions::Function;
use crate::model::Model;
use crate::recalc_budget::RecalcBudget;

use super::result::{charge, units};
use super::{CacheField, PivotComputeError};

/// Most characters in a calculated field's formula: Excel's cell formula limit. What Excel does
/// with a longer one is unprobed (UNVERIFIED, T-4897); the import skips its cache.
pub const MAX_CALCULATED_FORMULA_CHARS: usize = 8_192;

/// Most calculated fields in one cache (DEVIATION, decision 0014: pivots.md "A pivot result is
/// bounded in work and nodes"). Excel's own limit is unprobed.
pub const MAX_CALCULATED_FIELDS: usize = 256;

/// The functions a calculated field may not call, a denylist: the four Excel for the web refuses
/// (NOW, TODAY, RAND, INDIRECT, T-4505) and, unprobed (UNVERIFIED, T-4897), the other volatile,
/// reference and external-data ones and the name binders (see the module documentation).
const REFUSED_FUNCTIONS: [Function; 16] = [
    Function::Now,
    Function::Today,
    Function::Rand,
    Function::Indirect,
    Function::Randbetween,
    Function::Randarray,
    Function::Offset,
    Function::Cell,
    Function::Info,
    Function::Getpivotdata,
    Function::Image,
    Function::Stockhistory,
    Function::Fieldvalue,
    Function::Filterxml,
    Function::Let,
    Function::Lambda,
];

/// The work units charged for making a plan's sandbox workbook ([`Sandbox::new`]): a flat cost,
/// about what a few hundred nodes evaluated cost (`area_budget::NODE_UNITS` each).
const SANDBOX_UNITS: u64 = 4_096;

/// The sheet the formulas are parsed and evaluated on: the sandbox's only one.
const SANDBOX_SHEET: &str = "Sheet1";

/// Why a calculated field's formula is refused.
#[derive(ThisError, Debug, Clone, PartialEq, Eq)]
pub enum CalculatedFieldError {
    /// The formula is longer than [`MAX_CALCULATED_FORMULA_CHARS`].
    #[error(
        "a calculated field's formula is longer than {MAX_CALCULATED_FORMULA_CHARS} characters"
    )]
    TooLong,
    /// The cache has more than [`MAX_CALCULATED_FIELDS`] calculated fields.
    #[error("a pivot cache has more than {MAX_CALCULATED_FIELDS} calculated fields")]
    TooMany,
    /// The formula does not parse.
    #[error("a calculated field's formula does not parse")]
    Parse,
    /// The formula names something that is not a field of the cache (`Foo*2`).
    #[error("a calculated field's formula names no field: {0:?}")]
    UnknownField(String),
    /// The formula holds a cell or range reference (`A1*2`, `Data!C2*2`).
    #[error("a calculated field's formula holds a cell reference")]
    Reference,
    /// The formula holds an array constant (`SUM({1,2})+Units`).
    #[error("a calculated field's formula holds an array constant")]
    ArrayConstant,
    /// The formula calls a function a calculated field may not call ([`REFUSED_FUNCTIONS`]).
    #[error("a calculated field's formula calls {0}")]
    Function(String),
    /// The formula holds another node a calculated field may not hold.
    #[error("a calculated field's formula holds {0}")]
    Unsupported(&'static str),
}

/// Checks every calculated field of a cache's `fields` as [`compute`](super::compute) reads it,
/// so that the import can skip a cache Excel would refuse, or the engine could not compute.
///
/// # Errors
/// The first field's [`CalculatedFieldError`].
pub fn check_calculated_fields(fields: &[CacheField]) -> Result<(), CalculatedFieldError> {
    let names = FieldNames::of(fields);
    let mut count = 0_usize;
    for field in fields {
        if let Some(formula) = &field.formula {
            count += 1;
            if count > MAX_CALCULATED_FIELDS {
                return Err(CalculatedFieldError::TooMany);
            }
            compile(formula, &names)?;
        }
    }
    Ok(())
}

/// The cache's fields by case-folded name, the first of a repeated name. Whether Excel matches a
/// formula's field names ignoring case is unprobed (UNVERIFIED, T-4897), as a refresh matches
/// fields by case-folded name (`refresh::fields_by_name`).
struct FieldNames(HashMap<String, u32>);

impl FieldNames {
    fn of(fields: &[CacheField]) -> FieldNames {
        let mut names = HashMap::with_capacity(fields.len());
        for (index, field) in (0u32..).zip(fields) {
            names.entry(fold_case(&field.name)).or_insert(index);
        }
        FieldNames(names)
    }

    fn get(&self, name: &str) -> Option<u32> {
        self.0.get(&fold_case(name)).copied()
    }
}

/// A checked formula: its tree with every field it names a [`Node::NamedVariableKind`] whose
/// `id` is the field's index, and those indices, in the order first named.
struct Compiled {
    tree: Node,
    fields: Vec<u32>,
}

/// Parses and checks `formula` (see the module documentation).
fn compile(formula: &str, names: &FieldNames) -> Result<Compiled, CalculatedFieldError> {
    if formula.chars().count() > MAX_CALCULATED_FORMULA_CHARS {
        return Err(CalculatedFieldError::TooLong);
    }
    let mut parser = new_parser_english(vec![SANDBOX_SHEET.to_string()], vec![], HashMap::new());
    parser.set_loaded(true);
    parser.set_quoted_names(true);
    let context = CellReferenceRC {
        sheet: SANDBOX_SHEET.to_string(),
        row: 1,
        column: 1,
    };
    let mut tree = parser.parse(formula, &context);
    if tree.has_parse_error() {
        return Err(CalculatedFieldError::Parse);
    }
    // Evaluation reads no whitespace (`ParsedFormula`).
    tree.strip_spaces();
    let mut fields = Vec::new();
    let mut seen = HashSet::new();
    let mut pending = vec![&mut tree];
    while let Some(node) = pending.pop() {
        let named = match node {
            Node::NamedVariableKind { name, .. } => Some(
                names
                    .get(name)
                    .ok_or_else(|| CalculatedFieldError::UnknownField(name.clone()))?,
            ),
            // The parser reads a field spelled like a cell (`Rev2`) as that cell; any other
            // reference is refused.
            Node::ReferenceKind {
                sheet_name: None,
                absolute_row: false,
                absolute_column: false,
                row,
                column,
                ..
            } => Some(
                spelled_cell(*row, *column)
                    .and_then(|spelled| names.get(&spelled))
                    .ok_or(CalculatedFieldError::Reference)?,
            ),
            _ => None,
        };
        if let Some(field) = named {
            if seen.insert(field) {
                fields.push(field);
            }
            *node = Node::NamedVariableKind {
                name: String::new(),
                id: Some(field),
            };
            continue;
        }
        check_node(node)?;
        node.push_children_mut(&mut pending);
    }
    Ok(Compiled { tree, fields })
}

/// The A1 text of a relative reference read at `A1` (`row` and `column` are 0-based offsets).
fn spelled_cell(row: i32, column: i32) -> Option<String> {
    let letters = number_to_column(column.checked_add(1)?)?;
    Some(format!("{letters}{}", row.checked_add(1)?))
}

/// Whether a node other than a field may stand in a calculated field's formula.
fn check_node(node: &Node) -> Result<(), CalculatedFieldError> {
    use CalculatedFieldError::{ArrayConstant, Function as Call, Reference, Unsupported};
    match node {
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::OpConcatenateKind { .. }
        | Node::CompareKind { .. }
        | Node::UnaryKind { .. }
        | Node::UnaryPlus { .. }
        | Node::Group { .. }
        | Node::Spaced { .. } => Ok(()),
        Node::FunctionKind { kind, .. } if REFUSED_FUNCTIONS.contains(kind) => {
            Err(Call(kind.to_xlsx_string()))
        }
        Node::FunctionKind { .. } => Ok(()),
        Node::ArrayKind(_) => Err(ArrayConstant),
        Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::OpRangeKind { .. }
        | Node::OpIntersectKind { .. }
        | Node::OpUnionKind { .. }
        | Node::StructuredReferenceKind { .. }
        | Node::ThisRowKind { .. }
        | Node::SpillRangeOperator { .. }
        | Node::SheetRefErrorKind { .. } => Err(Reference),
        Node::DefinedNameKind(_) => Err(Unsupported("a defined name")),
        Node::NamedFunctionKind { .. } => Err(Unsupported("a call of an unknown function")),
        _ => Err(Unsupported("a node a calculated field may not hold")),
    }
}

/// What a pivot's calculated data fields need, ready to evaluate cell by cell ([`Plan::values`]).
///
/// Each field a formula names reads one input per cell: a database field's Sum, from an
/// accumulator slot after the data fields' ([`Plan::operands`]), or a calculated field's value,
/// evaluated earlier in the cell ([`Plan::steps`] are in dependency order). A formula's tree
/// names its inputs by their index: operands first, then steps.
pub(super) struct Plan {
    /// The database fields the formulas read, in slot order.
    pub(super) operands: Vec<u32>,
    /// The calculated fields to evaluate, in order; `None` for one in or behind a cycle.
    steps: Vec<Option<Step>>,
    /// Per data field, the step whose value it shows, when it is over a calculated field.
    pub(super) shows: Vec<Option<usize>>,
    sandbox: Sandbox,
}

/// One calculated field's formula, its fields renamed to inputs.
struct Step {
    tree: Node,
    /// What cloning the tree costs ([`crate::area_budget::NODE_UNITS`] aside: a unit a node).
    nodes: u64,
}

impl Plan {
    /// The plan for the data fields `data` (their cache field indices) over `fields`; `None`
    /// when none of them is a calculated field.
    ///
    /// # Errors
    /// [`PivotComputeError::Malformed`] for a formula [`check_calculated_fields`] refuses or an
    /// index past the fields, and [`PivotComputeError::Limit`] past `work`.
    pub(super) fn new(
        data: &[u32],
        fields: &[CacheField],
        work: &mut u64,
    ) -> Result<Option<Plan>, PivotComputeError> {
        let formula = |field: u32| {
            fields
                .get(field as usize)
                .map(|f| f.formula.as_deref())
                .ok_or(PivotComputeError::Malformed)
        };
        let mut wanted = Vec::new();
        for &field in data {
            if formula(field)?.is_some() && !wanted.contains(&field) {
                wanted.push(field);
            }
        }
        if wanted.is_empty() {
            return Ok(None);
        }
        // Every calculated field the data fields reach, compiled once, without recursion.
        let names = FieldNames::of(fields);
        let mut compiled: HashMap<u32, Compiled> = HashMap::new();
        let mut order = Vec::new();
        while let Some(field) = wanted.pop() {
            if compiled.contains_key(&field) {
                continue;
            }
            if compiled.len() >= MAX_CALCULATED_FIELDS {
                return Err(PivotComputeError::Limit);
            }
            let text = formula(field)?.ok_or(PivotComputeError::Malformed)?;
            charge(work, units(text.len()))?;
            let formula = compile(text, &names).map_err(|_| PivotComputeError::Malformed)?;
            for &named in &formula.fields {
                if formula_of(fields, named).is_some() && !compiled.contains_key(&named) {
                    wanted.push(named);
                }
            }
            order.push(field);
            compiled.insert(field, formula);
        }
        // The database fields the formulas read, in the order first met.
        let mut operands = Vec::new();
        let mut seen = HashSet::new();
        for field in &order {
            for &named in compiled.get(field).map_or(&[][..], |c| &c.fields) {
                if formula_of(fields, named).is_none() && seen.insert(named) {
                    operands.push(named);
                }
            }
        }
        let evaluation = dependency_order(&order, &compiled);
        let mut input: HashMap<u32, u32> = HashMap::new();
        for (slot, &field) in (0u32..).zip(&operands) {
            input.insert(field, slot);
        }
        let base = u32::try_from(operands.len()).map_err(|_| PivotComputeError::Limit)?;
        for (step, (field, _)) in (base..).zip(&evaluation) {
            input.insert(*field, step);
        }
        let mut steps = Vec::with_capacity(evaluation.len());
        let mut step_of: HashMap<u32, usize> = HashMap::new();
        for (index, (field, ordered)) in evaluation.iter().enumerate() {
            step_of.insert(*field, index);
            let tree = match (ordered, compiled.remove(field)) {
                (true, Some(formula)) => {
                    let mut tree = formula.tree;
                    rename_inputs(&mut tree, &input);
                    Some(Step {
                        nodes: tree.node_count(),
                        tree,
                    })
                }
                _ => None,
            };
            steps.push(tree);
        }
        let shows = data
            .iter()
            .map(|field| step_of.get(field).copied())
            .collect();
        charge(work, SANDBOX_UNITS)?;
        Ok(Some(Plan {
            operands,
            steps,
            shows,
            sandbox: Sandbox::new()?,
        }))
    }

    /// The value of every step at one cell whose operands' Sums are `operands`, in step order.
    ///
    /// # Errors
    /// [`PivotComputeError::Limit`] past `work`.
    pub(super) fn values(
        &mut self,
        operands: Vec<Result<f64, Error>>,
        work: &mut u64,
    ) -> Result<Vec<Result<f64, Error>>, PivotComputeError> {
        let first = operands.len();
        let mut inputs = operands;
        for step in &self.steps {
            let value = match step {
                // A cycle, a field naming itself, or one behind them (pivots.md: EC, ES).
                None => Err(Error::NAME),
                Some(step) => {
                    charge(work, step.nodes)?;
                    let mut tree = step.tree.clone();
                    substitute(&mut tree, &inputs);
                    self.sandbox.evaluate(&tree, work)?
                }
            };
            inputs.push(value);
        }
        Ok(inputs.split_off(first))
    }
}

/// The formula of field `field`, if it is a calculated field.
fn formula_of(fields: &[CacheField], field: u32) -> Option<&str> {
    fields.get(field as usize)?.formula.as_deref()
}

/// The calculated fields `fields` in an order that evaluates each after the calculated fields it
/// names, each with whether it could be ordered: one in a cycle, or naming one that could not,
/// could not (Kahn's algorithm, which needs no recursion).
fn dependency_order(fields: &[u32], compiled: &HashMap<u32, Compiled>) -> Vec<(u32, bool)> {
    let depends = |field: &u32| -> Vec<u32> {
        compiled.get(field).map_or_else(Vec::new, |c| {
            (c.fields.iter())
                .copied()
                .filter(|named| compiled.contains_key(named))
                .collect()
        })
    };
    let mut waiting: HashMap<u32, usize> = HashMap::new();
    let mut dependents: HashMap<u32, Vec<u32>> = HashMap::new();
    for field in fields {
        let named = depends(field);
        waiting.insert(*field, named.len());
        for on in named {
            dependents.entry(on).or_default().push(*field);
        }
    }
    let mut ready: Vec<u32> = (fields.iter().rev())
        .copied()
        .filter(|f| waiting.get(f) == Some(&0))
        .collect();
    let mut ordered = Vec::with_capacity(fields.len());
    let mut done = HashSet::new();
    while let Some(field) = ready.pop() {
        ordered.push((field, true));
        done.insert(field);
        for dependent in dependents.get(&field).map_or(&[][..], Vec::as_slice) {
            if let Some(count) = waiting.get_mut(dependent) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.push(*dependent);
                }
            }
        }
    }
    for field in fields {
        if !done.contains(field) {
            ordered.push((*field, false));
        }
    }
    ordered
}

/// Renames every field of `tree` ([`compile`]) to its input index.
fn rename_inputs(tree: &mut Node, input: &HashMap<u32, u32>) {
    let mut pending = vec![tree];
    while let Some(node) = pending.pop() {
        if let Node::NamedVariableKind { id, .. } = node {
            *id = id.and_then(|field| input.get(&field).copied());
            continue;
        }
        node.push_children_mut(&mut pending);
    }
}

/// Replaces every input of `tree` by its value in `inputs`.
fn substitute(tree: &mut Node, inputs: &[Result<f64, Error>]) {
    let mut pending = vec![tree];
    while let Some(node) = pending.pop() {
        if let Node::NamedVariableKind { id, .. } = node {
            let value = id.and_then(|i| inputs.get(i as usize));
            *node = match value {
                Some(Ok(number)) => Node::NumberKind(*number),
                Some(Err(error)) => Node::ErrorKind(error.clone()),
                None => Node::ErrorKind(Error::NAME),
            };
            continue;
        }
        node.push_children_mut(&mut pending);
    }
}

/// The empty workbook a pivot's formulas are evaluated in (see the module documentation).
struct Sandbox(Box<Model>);

/// Where the sandbox evaluates: its first sheet's `A1`, as the formulas were parsed.
const SANDBOX_CELL: CellReferenceIndex = CellReferenceIndex {
    sheet: 0,
    row: 1,
    column: 1,
};

impl Sandbox {
    fn new() -> Result<Sandbox, PivotComputeError> {
        // A fixed clock: nothing the sandbox may evaluate reads it, and none reads the host's.
        let clock = Clock::fixed_unix_millis(0).map_err(|_| PivotComputeError::Malformed)?;
        Model::new_empty_with_clock("pivot", "en", "UTC", "en", clock)
            .map(|model| Sandbox(Box::new(model)))
            .map_err(|_| PivotComputeError::Malformed)
    }

    /// The value of `tree`, a checked formula over numbers, drawing at most what `work` holds:
    /// the run's work comes from a recalculation budget of that size, which bounds it below its
    /// own limit, and what it spent is then charged to `work`.
    fn evaluate(
        &mut self,
        tree: &Node,
        work: &mut u64,
    ) -> Result<Result<f64, Error>, PivotComputeError> {
        let model = &mut self.0;
        let left = *work;
        model.evaluation.recalc = Some(RecalcBudget::new(left, left));
        let result = model.evaluate_outside_cell(tree, SANDBOX_CELL);
        let budget = model.evaluation.recalc.take();
        let (after, overrun) = budget.map_or((0, true), |b| (b.left, b.overrun.is_some()));
        charge(work, left.saturating_sub(after))?;
        if overrun {
            *work = 0;
            return Err(PivotComputeError::Limit);
        }
        Ok(scalar(result))
    }
}

/// A formula's value as a pivot cell shows it: a boolean as 1 or 0 (`AND(Units>0,Price>10)`,
/// pivots.md FA), a text as `#VALUE!` (`"a"`, ET). An empty value is 0, and an array or a
/// reference `#VALUE!` (both unprobed: UNVERIFIED, T-4897).
fn scalar(result: CalcResult) -> Result<f64, Error> {
    match result {
        CalcResult::Number(n) if n.is_finite() => Ok(n),
        CalcResult::Number(_) => Err(Error::NUM),
        CalcResult::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
        CalcResult::Error { error, .. } => Err(error),
        CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(0.0),
        CalcResult::String(_)
        | CalcResult::Range { .. }
        | CalcResult::Array(_)
        | CalcResult::Lambda(_) => Err(Error::VALUE),
    }
}

#[cfg(test)]
mod tests;
