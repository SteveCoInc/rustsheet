//! Workbook defined names: creating, validating, renaming and deleting them.

use crate::{
    calc_result::{CalcResult, Range},
    constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH},
    evaluation_limits::stack_address,
    expressions::{
        parser::{
            is_multi_sheet_span,
            stringify::{
                pin_unresolved_name, rebind_workbook_name_to_sheet, rename_defined_name_in_node,
                sheet_edits, to_english_string,
            },
            Node, TOO_MANY_NESTED_CALLS_MESSAGE,
        },
        token::Error,
        types::{CellReferenceIndex, CellReferenceRC},
        utils::{is_cell_spelled_name, is_valid_identifier},
    },
    functions::logical::SharedLambda,
    model::Model,
    reference_operators::RefArg,
    types::{DefinedName, WorkbookView},
    utils as common,
};

/// Where a defined name comes from, which decides the checks [`Model::insert_defined_name`] makes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NameSource {
    /// Added through the API: held to Excel's 65-call cap.
    Api,
    /// Read from a workbook: a cell-like spelling is accepted, the importer checks the cap.
    Loaded,
    /// Put back by an undo: it existed already, so the cap is not applied.
    Restored,
}

/// The `#NAME?` of a name whose body runs out of depth or budget ([`Model::within_defined_name`]).
fn name_refers_to_itself(name: &str, cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::NAME,
        cell,
        format!("Defined name \"{name}\" refers to itself."),
    )
}

/// A defined name's reference body (`=Jan!$A$1`, `=Jan!A1:B2`). xlsx stores a relative row or
/// column relative to A1, and Excel reads it at the same offset from the cell that uses the name:
/// `=Jan!A2` used in Calc!K6 is Jan!K7, `=Jan!$A1` there Jan!$A6 (docs/excel-semantics/
/// references.md "Relative references in defined names", gen-m8a case 051f538767ab0cfb; T-1639).
#[derive(Clone)]
pub(crate) struct NameReference {
    /// The corners as written, relative parts counted from A1.
    written: Range,
    /// The `(row, column)` absolute (`$`) flags of the left corner, then of the right one.
    absolute: [(bool, bool); 2],
}

impl NameReference {
    pub(crate) fn new(
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        absolute: [(bool, bool); 2],
    ) -> Self {
        NameReference {
            written: Range { left, right },
            absolute,
        }
    }

    /// The sheet the reference is on; using the name elsewhere does not move it.
    pub(crate) fn sheet(&self) -> u32 {
        self.written.left.sheet
    }

    /// The range the reference stands for when the name is used in `cell`.
    pub(crate) fn at(&self, cell: CellReferenceIndex) -> Range {
        let corner = |written: CellReferenceIndex,
                      (absolute_row, absolute_column): (bool, bool)| {
            CellReferenceIndex {
                sheet: written.sheet,
                row: if absolute_row {
                    written.row
                } else {
                    written.row - 1 + cell.row
                },
                column: if absolute_column {
                    written.column
                } else {
                    written.column - 1 + cell.column
                },
            }
        };
        // Mixed anchors can move one corner past the other (`Jan!$B$5:$B1` used in row 1 is
        // B5:B1); the range is the one between them, B1:B5, as an inline reference is.
        Range::between(
            corner(self.written.left, self.absolute[0]),
            corner(self.written.right, self.absolute[1]),
        )
    }

    /// The reference as a reference-context result, used in `cell`.
    pub(crate) fn range_at(&self, cell: CellReferenceIndex) -> CalcResult {
        let Range { left, right } = self.at(cell);
        CalcResult::Range { left, right }
    }
}

/// A parsed formula for a defined name
#[derive(Clone)]
pub(crate) enum ParsedDefinedName {
    /// `CellReference` (`=C4`)
    CellReference(NameReference),
    /// A Range (`=C4:D6`)
    RangeReference(NameReference),
    /// `=LAMBDA(params..., body)`, shared with the lambdas a call creates (T-0440)
    LambdaDefinition(SharedLambda),
    /// An error literal, possibly sheet-qualified (`=Sheet1!#REF!`): the name evaluates to it
    ErrorLiteral(Error),
    /// Any other formula (`=SUM(Sheet1!$A$1:$A$3)`, `=Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2`,
    /// `=0.5`): evaluated where the name is used, in that cell's context
    Formula(std::sync::Arc<Node>),
    /// A body that does not parse
    InvalidDefinedNameFormula,
}

/// How many defined-name bodies may be evaluated inside one another. A name whose body
/// reaches itself, directly (`=selfref+1`) or through others (`a` is `=b`, `b` is `=a`), is
/// `#NAME?` in Excel (probes/defined-name-formula-bodies H11, H13); the engine gives that by
/// running out of depth or budget. A chain of distinct names deeper than this is `#NAME?` too,
/// where Excel would evaluate it (DEVIATION, decision 0014: references.md "A defined name may be
/// a formula").
const MAX_DEFINED_NAME_DEPTH: usize = 64;

/// How many name bodies one cell's formula may evaluate in all. The depth limit alone
/// does not bound the work: `x` = `=x+x` doubles it at every level. Past this the use is
/// `#NAME?` (same DEVIATION).
const MAX_DEFINED_NAME_EVALUATIONS: usize = 10_000;

/// The state of one run of a cell's formula (`Model::run_cell_formula`): the defined-name bodies
/// being evaluated right now (`Model::within_defined_name`) and the D-function formula condition.
#[derive(Default)]
pub(crate) struct DefinedNameRun {
    /// Bodies evaluated inside one another.
    depth: usize,
    /// Bodies evaluated so far in this cell's formula, or in this outermost use outside a cell.
    evaluations: usize,
    /// Where the stack was when the outermost one started.
    stack_base: usize,
    /// Whether a D-function formula condition is being evaluated again for a record
    /// (`Model::db_computed_condition`), which does not nest. A cell first computed inside one
    /// is its own run, so its D-functions re-evaluate their own conditions.
    pub(crate) db_condition_running: bool,
}

impl Model {
    /// Runs `f` one defined-name body deeper, or gives `None` past [`MAX_DEFINED_NAME_DEPTH`],
    /// [`MAX_DEFINED_NAME_EVALUATIONS`] or the evaluation's stack budget, so that names
    /// referring to themselves stay bounded in depth, time and stack.
    pub(crate) fn within_defined_name<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> Option<T> {
        let here = stack_address();
        // Inside a cell, measure from where its recursion started, so that names and cells
        // share one stack budget (evaluation.rs).
        let in_cell = self.evaluation.depth > 0;
        let cell_base = in_cell.then_some(self.evaluation.stack_base);
        let run = &mut self.defined_names_running;
        if run.depth == 0 {
            // A cell's formula has one budget however many names it uses (`=x+x+x`), set up by
            // `run_cell_formula`; outside a cell (conditional formats, `evaluate_formula`) every
            // outermost use starts afresh.
            if !in_cell {
                run.evaluations = 0;
            }
            run.stack_base = cell_base.unwrap_or(here);
        }
        if run.depth >= MAX_DEFINED_NAME_DEPTH
            || run.evaluations >= MAX_DEFINED_NAME_EVALUATIONS
            || run.stack_base.abs_diff(here) >= self.evaluation.max_stack
        {
            return None;
        }
        run.depth += 1;
        run.evaluations += 1;
        let result = f(self);
        self.defined_names_running.depth -= 1;
        Some(result)
    }

    /// Evaluates the formula `node` of the cell `cell` with a name budget and an area budget of
    /// its own, so that the budgets belong to this one run of this one formula: a cell evaluated
    /// inside another cell's name body, or the same cell run again after a restart, does not
    /// inherit them. `spills` is set for a dynamic-array anchor ([`Model::evaluate_formula_root`]).
    pub(crate) fn run_cell_formula(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        spills: bool,
    ) -> CalcResult {
        let outer = std::mem::take(&mut self.defined_names_running);
        let result = self.with_area_budget(cell, |model| {
            model.evaluate_formula_root(node, cell, spills)
        });
        self.defined_names_running = outer;
        result
    }

    /// The value of the defined name `name` whose body is the formula `body`, used in `cell`. The
    /// body is snapped at its own root like a cell formula (docs/excel-semantics/numbers.md
    /// "Near-zero snapping", probes/near-zero-snap-groups C1-C5).
    pub(crate) fn evaluate_defined_name_formula(
        &mut self,
        name: &str,
        body: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // A name bound to a 3-D span is `#VALUE!` as a value, where the span written inline is
        // `#REF!`: `=span3`, `span3+100`, `ROWS(span3)` (gen-m8a b221d72cc501675a; T-1640).
        if is_multi_sheet_span(body) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("Defined name \"{name}\" is a 3-D reference."),
            );
        }
        self.within_defined_name(|model| model.evaluate_formula_root(body, cell, false))
            .unwrap_or_else(|| name_refers_to_itself(name, cell))
    }

    /// The parsed defined name `name` of `scope`, as written in a formula: any `Sheet!` or `[0]`
    /// prefix is not part of the name.
    pub(crate) fn parsed_defined_name(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Option<&ParsedDefinedName> {
        let bare = crate::expressions::utils::bare_defined_name(name);
        crate::name_key::get_name(&self.parsed_defined_names, scope, bare)
    }

    /// The body of the defined name `node` when it is a formula (T-0158), shared, not copied;
    /// `None` for any other node or name.
    pub(crate) fn defined_name_formula(&self, node: &Node) -> Option<std::sync::Arc<Node>> {
        let Node::DefinedNameKind((name, scope, _)) = node else {
            return None;
        };
        match self.parsed_defined_name(name, *scope) {
            Some(ParsedDefinedName::Formula(body)) => Some(std::sync::Arc::clone(body)),
            _ => None,
        }
    }

    /// Whether `node` is a defined name bound to a 3-D span over several sheets
    /// (`span3` = `=Jan:Mar!$A$1:$B$2`).
    pub(crate) fn is_sheet_span_name(&self, node: &Node) -> bool {
        self.defined_name_formula(node)
            .is_some_and(|body| is_multi_sheet_span(&body))
    }

    /// The error a defined-name `node` is by itself: an error literal (`=Sheet1!#REF!`) or a body
    /// that does not parse (`#NAME?`), else `None`.
    pub(crate) fn defined_name_error_literal(&self, node: &Node) -> Option<Error> {
        let Node::DefinedNameKind((name, scope, _)) = node else {
            return None;
        };
        match self.parsed_defined_name(name, *scope) {
            Some(ParsedDefinedName::ErrorLiteral(error)) => Some(error.clone()),
            Some(ParsedDefinedName::InvalidDefinedNameFormula) => Some(Error::NAME),
            _ => None,
        }
    }

    /// The formula-bodied defined name `node` as an operand of `:`: its body read as a `:`
    /// operand, under the same depth and budget as its value read.
    pub(crate) fn defined_name_range_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (Node::DefinedNameKind((name, ..)), Some(body)) =
            (node, self.defined_name_formula(node))
        else {
            return self.evaluate_node_with_reference(node, cell);
        };
        self.within_defined_name(|model| model.range_operand(&body, cell))
            .unwrap_or_else(|| name_refers_to_itself(name, cell))
    }

    /// The defined name `node` where a reference is accepted. A name whose body is a formula
    /// stands for what the body is in reference context, so a union body is its areas
    /// (probes/union-defined-name H1, H4, H5) and a cross-sheet one `#VALUE!`
    /// (probes/multiarea-xname). Any other name is read as [`Model::evaluate_node_with_reference`]
    /// reads it. The body runs under the same depth and budget as its value read.
    pub(crate) fn defined_name_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        if let Node::DefinedNameKind((name, ..)) = node {
            if let Some(body) = self.defined_name_formula(node) {
                // Only a body that may stand for several areas, a 3-D span included, leaves
                // the value path, which snaps the body at its root (`SUM(nz)` with `nz` =
                // `0.1+0.2-0.3` is 0, probes/near-zero-snap-groups C5).
                if !self.is_multi_area(&body) && !is_multi_sheet_span(&body) {
                    return RefArg::from(self.evaluate_node_with_reference(node, cell));
                }
                return self
                    .within_defined_name(|model| model.evaluate_reference_arg(&body, cell))
                    .unwrap_or_else(|| RefArg::Value(name_refers_to_itself(name, cell)));
            }
        }
        RefArg::from(self.evaluate_node_with_reference(node, cell))
    }

    /// Adds a new defined name.
    /// If scope is None it is a global defined name, otherwise it is local to the sheet with index scope.
    pub fn new_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        self.is_valid_defined_name(name, scope, formula)?;
        self.define_name(name, scope, formula)
    }

    /// As [`Model::define_name`], for a name that existed before, which an undo of its delete puts
    /// back: the 65-call cap of an API add is not applied, so a body edited past it (an
    /// update is unchecked, since Graph has no endpoint that sets a body) comes back too instead
    /// of failing the undo (T-4014).
    ///
    /// The body is not held to [`Model::new_defined_name`]'s reference-or-LAMBDA rule either: a
    /// name loaded from a workbook may have any body.
    ///
    /// # Errors
    /// As [`Model::define_name`], except for the call cap.
    pub(crate) fn restore_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        self.insert_defined_name(name, scope, formula, NameSource::Restored)
    }

    /// Adds a defined name whose formula is any formula that parses, as Excel stores them: an
    /// intersection is a valid name body (docs/excel-semantics/references.md "A defined name may
    /// be an intersection"). [`Model::new_defined_name`] additionally restricts the body to a
    /// reference or a LAMBDA.
    ///
    /// If scope is None it is a global defined name, otherwise it is local to the sheet with
    /// index scope. Reparses and re-evaluates the workbook.
    ///
    /// # Errors
    /// An invalid or duplicate name, an invalid scope, or a formula that does not parse.
    pub fn define_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        self.insert_defined_name(name, scope, formula, NameSource::Api)
    }

    /// As [`Model::define_name`], for a name read from a workbook: one spelled like an A1 cell
    /// (`NLG1`) is accepted too, as Excel opens such a workbook.
    ///
    /// # Errors
    /// As [`Model::define_name`], except for the cell-like spelling.
    pub fn define_loaded_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        self.insert_defined_name(name, scope, formula, NameSource::Loaded)
    }

    /// Translates a formula the user typed (in the active language and locale)
    /// into the canonical English representation that is stored internally.
    ///
    /// The formula is first parsed in the active language/locale. If that fails
    /// it is parsed as English — this lets internally generated formulas (which
    /// are already English, e.g. produced by undo/redo or cut & paste) round
    /// trip unchanged regardless of the active language. Returns an error if the
    /// formula parses in neither. Any leading `=` is preserved.
    pub(crate) fn user_formula_to_internal(
        &mut self,
        formula: &str,
        context: &CellReferenceRC,
    ) -> Result<String, String> {
        self.formula_to_internal_between(formula, context, context)
    }

    /// As [`Model::user_formula_to_internal`], but read in `read_on` and written as seen from
    /// `write_from`: an unqualified reference read on another sheet comes out qualified.
    pub(crate) fn formula_to_internal_between(
        &mut self,
        formula: &str,
        read_on: &CellReferenceRC,
        write_from: &CellReferenceRC,
    ) -> Result<String, String> {
        let context = read_on;
        let trimmed = formula.trim();
        let had_equals = trimmed.starts_with('=');
        let body = trimmed.strip_prefix('=').unwrap_or(trimmed);
        let mut node = self.parser.parse(body, context);
        if let Node::ParseErrorKind { .. } = node {
            // The user's language could not parse it: it might already be in the
            // internal English form.
            node = self.parse_internal_formula(body, context);
        }
        if let Node::ParseErrorKind { .. } = node {
            return Err(format!("Invalid formula: '{formula}'"));
        }
        if read_on.sheet != write_from.sheet {
            sheet_edits::qualify_unqualified_references(&mut node, &read_on.sheet);
        }
        let english = to_english_string(&node, write_from);
        Ok(if had_equals {
            format!("={english}")
        } else {
            english
        })
    }

    fn insert_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
        source: NameSource,
    ) -> Result<(), String> {
        let loading = source == NameSource::Loaded;
        let sheet_id = self.defined_name_slot(name, scope, loading)?;
        // Defined-name formulas are stored internally in English so they keep
        // working when the user switches language/locale.
        let context = self.defined_name_context();
        // Excel's `names/add` binds an unqualified reference of the body to the active sheet
        // (probes/t4394-api-name-active-s2, -last; syntax.md "An unqualified reference in an
        // API-defined name binds to the active sheet"; T-4445): the body is read there and stored
        // qualified, as the defined-name context would write a reference to that sheet.
        let read_on = if source == NameSource::Api {
            self.active_sheet_context()
        } else {
            context.clone()
        };
        // A loaded body is read as a file's stored text: a structured reference Excel cannot
        // resolve, such as a padded bare column `T[ Col ]`, loads as `#REF!`
        // (probes/t3258-name-padded-column-0..2, T-3258).
        self.parser.set_loaded(loading);
        let internal_formula = self.formula_to_internal_between(formula, &read_on, &context);
        self.parser.set_loaded(false);
        let internal_formula = internal_formula?;
        // Excel's `names/add` refuses a body nesting more than 65 calls (400 `InvalidArgument`;
        // probes/t3264-api-name-65 is accepted, t3264-api-name-66 refused, T-3264; syntax.md
        // "Formula nesting depth cap"), and so does `worksheets('S')/names/add` for a sheet-scoped
        // name (probes/t4014-sheet-api-name-65, -66, T-4014). A loaded body is checked by the
        // importer, which refuses the whole workbook as Excel does (T-3143), and a restored one
        // existed already ([`Model::restore_defined_name`]).
        if source == NameSource::Api && self.parser.nests_too_many_calls() {
            return Err(format!("Formula: {TOO_MANY_NESTED_CALLS_MESSAGE}"));
        }
        if let (NameSource::Api, Some(sheet)) = (source, scope) {
            self.bind_added_sheet_name(name, sheet);
        }
        self.workbook.defined_names.push(DefinedName {
            name: name.to_string(),
            formula: internal_formula,
            sheet_id,
        });
        self.reset_parsed_structures();

        Ok(())
    }

    /// Rewrites the formulas of sheet `sheet` that a sheet-scoped `name` added through the API
    /// must not re-bind by their text alone (T-4049; docs/excel-semantics/syntax.md "A
    /// sheet-scoped name added after load"). Excel binds a formula's names when it is entered or
    /// loaded. With no workbook-scoped `name`, an unqualified use on the sheet stays bound to the
    /// undefined workbook name and is `#NAME?` (probes/t4049-sheet-api-name-plain S!A1:A3, as
    /// probes/t4014-sheet-api-name-65 S!A1); with one, the sheet's uses of it move to the new name,
    /// `S!N` included, while other sheets keep the workbook name
    /// (probes/t4049-sheet-api-name-over-book). An unqualified use of a defined workbook name
    /// needs no rewrite: its text already reads the new sheet-scoped name. In most other books
    /// Excel re-binds the unresolved use instead (probes/t4391-api-rebind-*), and no rule yet
    /// tells those books apart from T-4049's, so those cells are known failures (T-4617).
    fn bind_added_sheet_name(&mut self, name: &str, sheet: u32) {
        let name_upper = name.to_uppercase();
        let workbook_name_defined = self
            .workbook
            .defined_names
            .iter()
            .any(|df| df.sheet_id.is_none() && df.name.to_uppercase() == name_upper);
        let Ok(sheet_name) = self.workbook.worksheet(sheet).map(|ws| ws.get_name()) else {
            return;
        };
        self.rewrite_shared_formulas(Some(sheet), |node| {
            if workbook_name_defined {
                rebind_workbook_name_to_sheet(node, name, sheet, &sheet_name);
            } else {
                pin_unresolved_name(node, name);
            }
        });
    }

    /// The index of the workbook's active sheet (the sheet of the current workbook view; an xlsx
    /// import takes the sheet marked `tabSelected`). The first sheet when the view names none.
    #[must_use]
    pub fn active_sheet(&self) -> u32 {
        self.workbook
            .views
            .get(&self.view_id)
            .map(|view| view.sheet)
            .filter(|&sheet| self.workbook.worksheet(sheet).is_ok())
            .unwrap_or(0)
    }

    /// Makes the sheet at index `sheet` the active one: a name added through the API afterwards
    /// binds its unqualified references to it.
    ///
    /// # Errors
    /// An invalid sheet index.
    pub fn set_active_sheet(&mut self, sheet: u32) -> Result<(), String> {
        self.workbook.worksheet(sheet)?;
        // A workbook decoded from bytes may have no view for `view_id`; create it rather than
        // drop the choice, so `active_sheet` returns `sheet` afterwards (T-4554).
        self.workbook
            .views
            .entry(self.view_id)
            .or_insert(WorkbookView {
                sheet,
                window_width: DEFAULT_WINDOW_WIDTH,
                window_height: DEFAULT_WINDOW_HEIGHT,
            })
            .sheet = sheet;
        Ok(())
    }

    /// A1 of the active sheet, where an API-added name's body is read.
    fn active_sheet_context(&self) -> CellReferenceRC {
        let sheet = self.active_sheet();
        CellReferenceRC {
            sheet: self
                .workbook
                .worksheet(sheet)
                .map(|ws| ws.get_name())
                .unwrap_or_else(|_| self.defined_name_context().sheet),
            row: 1,
            column: 1,
        }
    }

    /// The context used to parse/stringify defined-name formulas. Defined names
    /// have no natural anchor cell, so we use the first worksheet's A1.
    pub(crate) fn defined_name_context(&self) -> CellReferenceRC {
        CellReferenceRC {
            sheet: self
                .workbook
                .worksheets
                .first()
                .map(|ws| ws.get_name())
                .unwrap_or_else(|| "Sheet1".to_string()),
            row: 1,
            column: 1,
        }
    }

    /// The `sheet_id` a new defined name `name` in `scope` would get, or an error when the name
    /// is not a valid identifier, the scope is not a sheet index, or the name already exists there.
    ///
    /// `loading` admits a name spelled like an A1 cell (`NLG1`): Excel opens a workbook defining
    /// one (its Name Manager refusing one is unverified), so only the loading path
    /// (`define_loaded_name`) takes it. The lexer reads such a spelling as the cell, so the name never shadows it
    /// (docs/excel-semantics/references.md "Legacy N and T marker spelling").
    fn defined_name_slot(
        &self,
        name: &str,
        scope: Option<u32>,
        loading: bool,
    ) -> Result<Option<u32>, String> {
        if !(is_valid_identifier(name) || (loading && is_cell_spelled_name(name))) {
            return Err("Name: Invalid defined name".to_string());
        }
        let name_upper = name.to_uppercase();
        let sheet_id = match scope {
            Some(index) => match self.workbook.worksheet(index) {
                Ok(ws) => Some(ws.sheet_id),
                Err(_) => return Err("Scope: Invalid sheet index".to_string()),
            },
            None => None,
        };
        if self
            .workbook
            .defined_names
            .iter()
            .any(|df| df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id)
        {
            return Err("Name: Defined name already exists".to_string());
        }
        Ok(sheet_id)
    }

    /// Validates if a defined name can be created
    pub fn is_valid_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<Option<u32>, String> {
        let sheet_id = self.defined_name_slot(name, scope, false)?;

        // Make sure the formula is valid: a cell/range reference, a LAMBDA definition or an error
        // literal such as `Sheet1!#REF!`.
        let is_reference =
            common::ParsedReference::parse_reference_formula(None, formula, self.locale, |name| {
                self.get_sheet_index_by_name(name)
            })
            .is_ok();

        if !is_reference {
            // Try the full parser to see if it is a LAMBDA definition.
            // Defined-name formulas may carry a leading '='; strip it before parsing.
            let formula_body = formula.strip_prefix('=').unwrap_or(formula);
            let dummy_ref = self.defined_name_context();
            // Accept the formula whether it is written in the active language
            // or already in the internal English form (e.g. generated by
            // undo/redo or cut & paste).
            let mut node = self.parser.parse(formula_body, &dummy_ref);
            if let Node::ParseErrorKind { .. } = node {
                node = self.parse_internal_formula(formula_body, &dummy_ref);
            }
            if !matches!(
                node.ungrouped(),
                Node::LambdaDefKind { .. } | Node::ErrorKind(_) | Node::SheetRefErrorKind { .. }
            ) {
                return Err("Formula: Invalid defined name formula".to_string());
            }
        }

        Ok(sheet_id)
    }

    /// Delete defined name of name and scope
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let name_upper = name.to_uppercase();
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(self.workbook.worksheet(index)?.sheet_id),
            None => None,
        };
        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        if let Some(i) = index {
            self.workbook.defined_names.remove(i);
            self.reset_parsed_structures();
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }

    /// Update defined name
    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
        new_scope: Option<u32>,
        new_formula: &str,
    ) -> Result<(), String> {
        if !is_valid_identifier(new_name) {
            return Err("Name: Invalid defined name".to_string());
        };
        let name_upper = name.to_uppercase();
        let new_name_upper = new_name.to_uppercase();

        if name_upper != new_name_upper || scope != new_scope {
            for key in self.parsed_defined_names.keys() {
                if key.lower().to_uppercase() == new_name_upper && key.scope() == new_scope {
                    return Err("Name: Defined name already exists".to_string());
                }
            }
        }
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            ),
            None => None,
        };

        let new_sheet_id = match new_scope {
            Some(index) => Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            ),
            None => None,
        };

        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        // Defined-name formulas are stored internally in English.
        let context = self.defined_name_context();
        let internal_formula = self.user_formula_to_internal(new_formula, &context)?;
        if let Some(i) = index {
            let renamed = self
                .workbook
                .defined_names
                .get(i)
                .is_some_and(|df| df.name != new_name);
            if renamed {
                // Every formula that uses the old name is rewritten to the new one.
                self.rewrite_shared_formulas(None, |t| {
                    rename_defined_name_in_node(t, name, scope, new_name);
                });
            }
            if let Some(df) = self.workbook.defined_names.get_mut(i) {
                df.name = new_name.to_string();
                df.sheet_id = new_sheet_id;
                df.formula = internal_formula;
                self.reset_parsed_structures();
            }
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }
}
