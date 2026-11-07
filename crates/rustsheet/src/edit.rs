//! Atomic edits (SPEC-0001 "Edits and change sets"): [`Workbook::edit`], [`Txn`], [`ChangeSet`].
//!
//! Every op applies to the engine immediately and records what it overwrote in an undo log;
//! an `Err` from the closure replays the log in reverse, so a failed edit never clones the
//! workbook. Nothing recalculates inside the closure: one `evaluate` runs after it returns
//! `Ok` (sheet-level engine ops reparse and so evaluate on their own, see [`Txn::add_sheet`]).
//!
//! The [`ChangeSet`] is computed by diffing (SPEC-0001): before the closure runs the facade
//! captures every formula and spill cell, ops add the cells they touch, and after the recalc
//! the captured values are compared with the current ones (see `diff`).

mod calculated_column;
mod diff;
mod pivot;
mod rollback;
mod sheets;
mod table;
mod table_filter;

pub use self::pivot::{
    PivotDateGroupBy, PivotError, PivotFunction, PivotParts, PivotRangeGroup, PivotTableInput,
};

use rustsheet_engine::clock::Clock;
use rustsheet_engine::types::{Cell, Link, Table};
use rustsheet_engine::Model;
use rustsheet_engine::TakenSheet;

use self::diff::{capture_derived, input_changes, value_changes, Before, Key, KeySet};
use self::rollback::roll_back;
use crate::error::{EditError, RefError};
use crate::refs::{CellRef, SheetId, Target};
use crate::value::Value;
use crate::workbook::{engine_rc, resolve_range, sheet_index, sheet_metas, Workbook};

/// What to store in a cell.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Input {
    /// A typed value, stored as is (text is never reinterpreted as a number or formula).
    Value(Value),
    /// A formula as a user types it; must start with `=`. Excel refuses some typed formulas it
    /// reads when stored, and so does this: a LET name or LAMBDA parameter spelled like a cell
    /// (`=LET(f1,1,f1+1)`) is [`EditError::InvalidInput`] (decision 0103). Typed into a table's
    /// data rows it fills the column as a calculated column where Excel would, as
    /// [`Input::Parse`] does (Excel's API and UI both type the formula; T-2958).
    Formula(String),
    /// A formula as an xlsx file stores it, read as xlsx import reads a dynamic-array `<f>`; must
    /// start with `=`. LET names and LAMBDA parameters spelled like a cell bind, as Excel binds
    /// them stored as `_xlpm.f1` (decision 0103). Use it to replay stored formulas; it never
    /// fills a table column.
    StoredFormula(String),
    /// A formula as a pre-dynamic-array workbook stores it; must start with `=`. Functions
    /// that expect a scalar implicitly intersect a range, so `=A1:A2` is stored as `=@A1:A2`,
    /// exactly as xlsx import reads a plain `<f>`.
    LegacyFormula(String),
    /// Text as a user would type it: `=` starts a formula, numbers, booleans, errors and
    /// percentages/currencies are recognised, a leading `'` forces text. A formula typed into a
    /// table's data rows fills the column when its other data cells are all blank or all hold
    /// one formula (a calculated column, T-2958); the filled cells are part of the edit.
    Parse(String),
    /// [`Input::Parse`] typed in R1C1 style (`=R[-1]C+1` in A3 is `=A2+1`; T-4288).
    ParseR1C1(String),
    /// Empties the cell (its style is kept).
    Clear,
}

impl From<f64> for Input {
    fn from(n: f64) -> Self {
        Self::Value(Value::Number(n))
    }
}

impl From<bool> for Input {
    fn from(b: bool) -> Self {
        Self::Value(Value::Bool(b))
    }
}

impl From<&str> for Input {
    fn from(text: &str) -> Self {
        Self::Parse(text.to_string())
    }
}

impl From<String> for Input {
    fn from(text: String) -> Self {
        Self::Parse(text)
    }
}

impl From<Value> for Input {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

/// A cell whose stored input (formula or literal text) changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputChange {
    /// The cell.
    pub cell: CellRef,
    /// The input before the edit, `None` for an empty cell.
    pub before: Option<String>,
    /// The input after the edit, `None` for an empty cell.
    pub after: Option<String>,
}

/// A cell whose computed value changed.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueChange {
    /// The cell.
    pub cell: CellRef,
    /// The value before the edit.
    pub before: Value,
    /// The value after the edit.
    pub after: Value,
}

/// A change to the sheet list.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SheetChange {
    /// A sheet was added.
    Added(SheetId),
    /// A sheet was renamed.
    Renamed {
        /// The sheet.
        id: SheetId,
        /// The old name.
        from: String,
        /// The new name.
        to: String,
    },
    /// A sheet was moved in the tab order.
    Moved {
        /// The sheet.
        id: SheetId,
        /// Its 0-based position before the move.
        from: u32,
        /// Its 0-based position after the move.
        to: u32,
    },
    /// A sheet was deleted.
    Deleted {
        /// The sheet.
        id: SheetId,
        /// Its name when it was deleted.
        name: String,
    },
}

/// Everything an edit changed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChangeSet {
    /// Cells whose input changed, sorted by (sheet order, row, column). Cells of a sheet deleted
    /// in the edit are reported by [`SheetChange::Deleted`] only.
    pub inputs: Vec<InputChange>,
    /// Cells whose value changed, sorted by (sheet order, row, column), cells of deleted sheets
    /// last. Empty when change tracking is off.
    pub values: Vec<ValueChange>,
    /// Sheet-list changes in the order they happened.
    pub sheets: Vec<SheetChange>,
}

/// Largest rectangle a single `set` snapshots for its spill footprint. Imported array ranges are
/// untrusted, so the loop must not follow them blindly. A DEVIATION from Excel:
/// docs/excel-semantics/arrays.md ("The edit API refuses a cell inside a spill extent").
const MAX_FOOTPRINT_CELLS: i64 = 1 << 22;

enum Undo {
    /// Cells (and the target's link) as they were before a `set`.
    Cells {
        sheet_id: u32,
        cells: Vec<(i32, i32, Option<Cell>)>,
        link: ((i32, i32), Option<Link>),
    },
    AddedSheet(u32),
    Renamed {
        sheet_id: u32,
        old_name: String,
    },
    DeletedSheet {
        index: u32,
        sheet: Box<TakenSheet>,
    },
    /// A sheet moved by `move_sheet`, with its position before the move.
    Moved {
        sheet_id: u32,
        index: u32,
    },
    /// A defined name added by `define_name` (`sheet_id: None`) or `define_sheet_name`.
    DefinedName {
        name: String,
        sheet_id: Option<u32>,
    },
    /// A table added by `add_table`.
    Table(String),
    /// A table as it was before a typed formula grew it (T-2968) or made one of its columns a
    /// calculated column (T-3024).
    TableBefore(Box<Table>),
    /// A data table added by `add_data_table`: its sheet and result range.
    DataTable {
        sheet_id: u32,
        range: (i32, i32, i32, i32),
    },
    /// A pivot added, refreshed or laid out (`edit::pivot`).
    Pivot(pivot::PivotUndo),
    /// A row's hidden flag as it was before `set_row_hidden`.
    RowHidden {
        sheet_id: u32,
        row: i32,
        hidden: bool,
    },
    /// The clock as it was before `set_clock`.
    Clock(Clock),
}

/// The write-only handle passed to [`Workbook::edit`].
pub struct Txn<'a> {
    model: &'a mut Model,
    undo: Vec<Undo>,
    before: Option<Before>,
    /// First-seen input text of each written cell, in first-write order.
    inputs: Vec<(Key, Option<String>)>,
    input_seen: KeySet,
    /// Cells a `set` already snapshotted for undo; a repeat write skips the footprint walk.
    logged: KeySet,
    sheets: Vec<SheetChange>,
    /// A sheet-level engine op reparses and evaluates, so rollback must re-evaluate.
    sheet_op: bool,
}

impl Txn<'_> {
    /// Stores `input` in a cell. A range target must be a single cell.
    ///
    /// # Errors
    /// [`EditError::Ref`] when the target does not resolve to one cell,
    /// [`EditError::InvalidInput`] when the engine rejects the input (a merged or array-formula
    /// cell, a formula without `=`, a non-finite number).
    pub fn set<'t>(
        &mut self,
        r: impl Into<Target<'t>>,
        input: impl Into<Input>,
    ) -> Result<(), EditError> {
        let range = resolve_range(self.model, r.into())?;
        if range.top != range.bottom || range.left != range.right {
            return Err(RefError::Unsupported.into());
        }
        let sheet = sheet_index(self.model, range.sheet)?;
        let (row, col) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        let cell = CellRef {
            sheet: range.sheet,
            row: range.top,
            col: range.left,
        };
        let key = (range.sheet.get(), row, col);
        self.log_cell(sheet, key, cell)?;
        let input = input.into();
        match &input {
            // `Input::Formula` without `=` is refused in `apply` before it writes anything.
            Input::Formula(text) if !text.starts_with('=') => {}
            Input::Parse(text) | Input::Formula(text) | Input::ParseR1C1(text) => {
                let r1c1 = matches!(input, Input::ParseR1C1(_));
                self.log_typed_fill(sheet, key, cell, text, r1c1)?;
            }
            _ => {}
        }
        self.apply(sheet, row, col, input, cell)
    }

    /// Records the input text, the undo entry and the pre-edit values for a write to `key`.
    fn log_cell(&mut self, sheet: u32, key: Key, cell: CellRef) -> Result<(), EditError> {
        let (sheet_id, row, col) = key;
        if self.input_seen.insert(key) {
            let text = self.input_text(sheet, row, col);
            self.inputs.push((key, text));
        }
        // Rollback replays the first snapshot last, so a cell already logged needs no second
        // one; a sheet-level op re-evaluates and can reshape spills, so it voids that.
        if self.logged.contains(&key) && !self.sheet_op {
            return Ok(());
        }
        let ws = self
            .model
            .workbook
            .worksheet(sheet)
            .map_err(EditError::Engine)?;
        // A spill member belongs to the anchor's footprint; an anchor owns its array range.
        let anchor = match ws.cell(row, col) {
            Some(Cell::SpillCell { a, .. }) => *a,
            _ => (row, col),
        };
        let mut positions = vec![(row, col)];
        if let Some(Cell::ArrayFormula {
            r: (width, height), ..
        }) = ws.cell(anchor.0, anchor.1)
        {
            if i64::from(*width) * i64::from(*height) > MAX_FOOTPRINT_CELLS {
                return Err(EditError::InvalidInput {
                    cell,
                    message: "spill extent too large to edit".to_string(),
                });
            }
            for r in anchor.0..anchor.0.saturating_add(*height) {
                for c in anchor.1..anchor.1.saturating_add(*width) {
                    if (r, c) != (row, col) {
                        positions.push((r, c));
                    }
                }
            }
        }
        let cells = positions
            .iter()
            .map(|&(r, c)| (r, c, ws.cell(r, c).cloned()))
            .collect();
        let link = ((row, col), ws.links.get(&(row, col)).cloned());
        self.logged
            .extend(positions.iter().map(|&(r, c)| (sheet_id, r, c)));
        if let Some(before) = &mut self.before {
            for &(r, c) in &positions {
                before.touch((sheet_id, r, c), ws, self.model);
            }
        }
        self.undo.push(Undo::Cells {
            sheet_id,
            cells,
            link,
        });
        Ok(())
    }

    fn input_text(&self, sheet: u32, row: i32, col: i32) -> Option<String> {
        self.model
            .get_localized_cell_content(sheet, row, col)
            .ok()
            .filter(|s| !s.is_empty())
    }

    fn apply(
        &mut self,
        sheet: u32,
        row: i32,
        col: i32,
        input: Input,
        cell: CellRef,
    ) -> Result<(), EditError> {
        let model = &mut *self.model;
        let result = match input {
            Input::Clear | Input::Value(Value::Empty) => {
                model.set_user_input(sheet, row, col, String::new())
            }
            Input::Formula(text) | Input::StoredFormula(text) | Input::LegacyFormula(text)
                if !text.starts_with('=') =>
            {
                return Err(EditError::InvalidInput {
                    cell,
                    message: format!("\"{text}\" is not a valid formula"),
                });
            }
            Input::Parse(text) | Input::Formula(text) => {
                model.set_typed_user_input(sheet, row, col, text)
            }
            Input::ParseR1C1(text) => model.set_typed_r1c1_user_input(sheet, row, col, text),
            Input::StoredFormula(text) => model.set_loaded_user_input(sheet, row, col, text),
            Input::LegacyFormula(text) => model.set_loaded_legacy_formula(sheet, row, col, &text),
            Input::Value(value) => {
                // Typed setters skip the spill/array/merge checks that `set_user_input` does.
                model
                    .prepare_cell_for_user_input(sheet, row, col)
                    .and_then(|()| match value {
                        Value::Number(n) if n.is_finite() => {
                            model.update_cell_with_number(sheet, row, col, n)
                        }
                        Value::Number(_) => Err("numbers must be finite".to_string()),
                        Value::Bool(b) => model.update_cell_with_bool(sheet, row, col, b),
                        Value::Text(s) => model.update_cell_with_text(sheet, row, col, &s),
                        Value::Error(e) => {
                            model.update_cell_with_error(sheet, row, col, e.kind.to_engine())
                        }
                        Value::Empty => Ok(()),
                    })
            }
        };
        result.map_err(|m| EditError::from_engine_message(m, Some(cell)))
    }

    /// The first sheet in workbook order.
    ///
    /// # Errors
    /// [`EditError::Engine`] if the workbook has no sheet (it never does).
    pub fn first_sheet(&self) -> Result<SheetId, EditError> {
        sheet_metas(self.model)
            .first()
            .map(|s| s.id)
            .ok_or_else(|| EditError::Engine("workbook has no sheet".to_string()))
    }

    /// Adds a workbook-scoped defined name: any formula that parses, `=` optional (`=S!$A$1:$B$3
    /// S!$A$2:$B$3`, an intersection); an unqualified reference binds to [`crate::Workbook::active_sheet`],
    /// as `names/add` does. Like a sheet-level op it reparses and re-evaluates the workbook, so define
    /// names before many cells. Names are not reported in the [`ChangeSet`]; changed cells are.
    ///
    /// # Errors
    /// [`EditError::DefinedName`] when the name is not a valid identifier, already exists, or the
    /// formula does not parse or nests more than 65 function calls, which Excel's `names/add`
    /// refuses (probes/t3264-api-name-66; docs/excel-semantics/syntax.md "Formula nesting depth cap").
    pub fn define_name(&mut self, name: &str, formula: &str) -> Result<(), EditError> {
        self.add_defined_name(name, None, formula, false)
    }

    /// Adds a defined name as a workbook being loaded defines it (`sheet` `None` is workbook
    /// scope). Unlike [`Txn::define_name`] it accepts a name spelled like an A1 cell (`NLG1`):
    /// Excel opens such a workbook, and a formula still reads the cell, not the name.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet; otherwise as [`Txn::define_name`], except for the
    /// cell-like spelling.
    pub fn load_defined_name(
        &mut self,
        sheet: Option<SheetId>,
        name: &str,
        formula: &str,
    ) -> Result<(), EditError> {
        self.add_defined_name(name, sheet, formula, true)
    }

    /// Adds a sheet-scoped defined name, as Excel's `Sheet1!Print_Area` or a name whose scope is
    /// one sheet: formulas on that sheet see it, and it shadows a workbook name of the same name
    /// there. Otherwise as [`Txn::define_name`].
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet; otherwise as [`Txn::define_name`].
    pub fn define_sheet_name(
        &mut self,
        sheet: SheetId,
        name: &str,
        formula: &str,
    ) -> Result<(), EditError> {
        self.add_defined_name(name, Some(sheet), formula, false)
    }

    fn add_defined_name(
        &mut self,
        name: &str,
        sheet: Option<SheetId>,
        formula: &str,
        loading: bool,
    ) -> Result<(), EditError> {
        let scope = sheet.map(|id| sheet_index(self.model, id)).transpose()?;
        // Stored without the `=`, as xlsx `definedName` text is: the engine reads a name body as
        // a reference only in that form.
        let body = formula.trim();
        let body = body.strip_prefix('=').unwrap_or(body);
        let defined = if loading {
            self.model.define_loaded_name(name, scope, body)
        } else {
            self.model.define_name(name, scope, body)
        };
        defined.map_err(EditError::DefinedName)?;
        self.sheet_op = true;
        self.undo.push(Undo::DefinedName {
            name: name.to_string(),
            sheet_id: sheet.map(SheetId::get),
        });
        Ok(())
    }

    /// Hides or shows a row (0-based): SUBTOTAL 101-111 and AGGREGATE's odd options skip it, and
    /// on a sheet with a filtered table (see [`Txn::add_table`]) SUBTOTAL 1-11 skip it too. The
    /// usual recalculation after the closure picks up functions that read the flag.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet or a row outside the Excel limits.
    pub fn set_row_hidden(
        &mut self,
        sheet: SheetId,
        row: u32,
        hidden: bool,
    ) -> Result<(), EditError> {
        let index = sheet_index(self.model, sheet)?;
        let (row, _) = engine_rc(row, 0).ok_or(RefError::OutOfBounds)?;
        let was_hidden = self
            .model
            .is_row_hidden(index, row)
            .map_err(|_| RefError::OutOfBounds)?;
        self.model
            .set_row_hidden(index, row, hidden)
            .map_err(|_| RefError::OutOfBounds)?;
        self.undo.push(Undo::RowHidden {
            sheet_id: sheet.get(),
            row,
            hidden: was_hidden,
        });
        Ok(())
    }

    /// Sets the workbook clock (SPEC-0008): NOW, TODAY and year-less date text read it, and
    /// `to_xlsx_bytes` stamps `dcterms:modified` from it. The usual recalculation after the closure
    /// applies it, even when `clock` equals the current one. Undone with the transaction.
    ///
    /// # Errors
    /// Never fails today; the `Result` keeps the op uniform with the other `Txn` methods.
    pub fn set_clock(&mut self, clock: Clock) -> Result<(), EditError> {
        self.undo.push(Undo::Clock(self.model.clock()));
        self.model.set_clock(clock);
        Ok(())
    }

    /// Gives one cell the number format `format` (Excel format code, e.g. `"[h]:mm"`), keeping
    /// its value and the rest of its style. It changes no value; the cell displays through it
    /// (`Workbook::formatted`). Undone with the transaction like a `set`.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, a target outside the sheet or a multi-cell
    /// target; [`EditError::Engine`] if the engine rejects the style.
    pub fn set_number_format<'t>(
        &mut self,
        r: impl Into<Target<'t>>,
        format: &str,
    ) -> Result<(), EditError> {
        let range = resolve_range(self.model, r.into())?;
        if range.top != range.bottom || range.left != range.right {
            return Err(RefError::Unsupported.into());
        }
        let sheet = sheet_index(self.model, range.sheet)?;
        let (row, col) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        let cell = CellRef {
            sheet: range.sheet,
            row: range.top,
            col: range.left,
        };
        self.log_cell(sheet, (range.sheet.get(), row, col), cell)?;
        let mut style = self
            .model
            .get_style_for_cell(sheet, row, col)
            .map_err(EditError::Engine)?;
        style.num_fmt = format.to_string();
        self.model
            .set_cell_style(sheet, row, col, &style)
            .map_err(EditError::Engine)
    }
}

impl Workbook {
    /// The workbook's clock.
    #[must_use]
    pub fn clock(&self) -> Clock {
        self.model.clock()
    }

    /// One-op [`Workbook::edit`]: sets the clock, recalculates once and reports the values that
    /// changed (SPEC-0008).
    ///
    /// # Errors
    /// As [`Txn::set_clock`].
    pub fn set_clock(&mut self, clock: Clock) -> Result<ChangeSet, EditError> {
        self.edit(|tx| tx.set_clock(clock))
            .map(|((), changes)| changes)
    }

    /// Turns the per-edit value diff on (default) or off. Off makes `edit`/`set` return a
    /// [`ChangeSet`] with empty `values`, for bulk loads that do not need the diff.
    pub fn set_track_changes(&mut self, on: bool) {
        self.track_changes = on;
    }

    /// Runs `f` as one transaction: all of its ops apply and the workbook recalculates once, or
    /// (when `f` returns `Err`) nothing changes.
    ///
    /// # Errors
    /// Whatever `f` returns; the workbook is then exactly as it was before the call.
    pub fn edit<T>(
        &mut self,
        f: impl FnOnce(&mut Txn<'_>) -> Result<T, EditError>,
    ) -> Result<(T, ChangeSet), EditError> {
        let before = self.track_changes.then(|| capture_derived(&self.model));
        let mut txn = Txn {
            model: &mut self.model,
            undo: Vec::new(),
            before,
            inputs: Vec::new(),
            input_seen: KeySet::default(),
            logged: KeySet::default(),
            sheets: Vec::new(),
            sheet_op: false,
        };
        let result = f(&mut txn);
        let Txn {
            undo,
            before,
            inputs,
            sheets,
            sheet_op,
            ..
        } = txn;
        let out = match result {
            Ok(out) => out,
            Err(e) => {
                roll_back(&mut self.model, undo, sheet_op);
                return Err(e);
            }
        };
        self.model.evaluate();
        let changes = ChangeSet {
            inputs: input_changes(&self.model, inputs),
            values: before.map_or_else(Vec::new, |mut b| value_changes(&self.model, &mut b)),
            sheets,
        };
        Ok((out, changes))
    }

    /// Stores `input` in one cell; shorthand for a one-op [`Workbook::edit`].
    ///
    /// # Errors
    /// As [`Txn::set`].
    pub fn set<'a>(
        &mut self,
        r: impl Into<Target<'a>>,
        input: impl Into<Input>,
    ) -> Result<ChangeSet, EditError> {
        self.edit(|tx| tx.set(r, input))
            .map(|((), changes)| changes)
    }
}

#[cfg(test)]
mod tests;
