//! A workbook's pivots as its file saved them (T-4906): the engine's model of the package loaded
//! without a recalculation, so no `refreshOnLoad` cache has been refreshed yet and every pivot
//! still holds the cache and definition Excel saved. The facade's evaluated workbook cannot serve
//! here: its first recalculation refreshes the `refreshOnLoad` caches from their sources
//! (docs/excel-semantics/pivots.md "Excel for the web re-renders a refreshOnLoad pivot when it
//! opens the file"), which is check (c)'s subject, not the file's own evidence.
//!
//! The model is the importer's, as the round-trip suites load it (`roundtrip::semantic`), on the
//! harness clock, so a relative date filter reads one fixed day.

use rustsheet_engine::pivot::{
    compute_on, lay_out, AxisLine, CacheField, PivotBudget, PivotCache, PivotCell, RenderedCell,
    MAX_PIVOT_WORK,
};
use rustsheet_engine::Model;

use super::records::{self, RecordCheck};
use super::SavedPivot;
use crate::cellref::Addr;
use crate::value::Value;
use crate::xlsx_snapshot::PivotLine;

/// A refresh of a `refreshOnLoad` pivot's cache from its source: the cells it writes, and
/// whether the source's records are the ones the file's cache saved.
pub(super) struct Refreshed {
    pub(super) cells: Vec<(Addr, Value)>,
    pub(super) same_records: bool,
}

/// The loaded model.
pub(super) struct SavedModel {
    model: Model,
}

/// A rendered pivot cell as the harness scores it, 1-based. The facade's `Value::from_pivot_cell`
/// and `engine::harness_value` take the facade's values, and this model is the engine's, below the
/// facade; an error is its en-US literal, as the facade's `stored_literal` writes it.
fn cell_value(cell: RenderedCell) -> (Addr, Value) {
    let at = Addr {
        row: cell.row.saturating_add(1),
        col: cell.column.saturating_add(1),
    };
    let value = match cell.value {
        PivotCell::Text(t) => Value::Text(t),
        PivotCell::Number(n) => Value::Number(n),
        PivotCell::Error(e) => Value::Error(e.to_string()),
    };
    (at, value)
}

/// Whether `new`'s records are `old`'s: the same count and, for every field the source gives
/// (no calculated field and none Excel added to group another), the same value in each record.
/// Bounded by the caches' record values (`MAX_RECORD_VALUES` each, directive 5).
fn same_records(old: &PivotCache, new: &PivotCache) -> bool {
    if old.record_count != new.record_count {
        return false;
    }
    let database = |f: &&CacheField| f.formula.is_none() && f.added_group.is_none();
    old.fields.iter().filter(database).all(|field| {
        let Some(other) = new.fields.iter().find(|f| f.name == field.name) else {
            return false;
        };
        field.records.len() == other.records.len()
            && (field.records.iter().zip(&other.records)).all(|(&a, &b)| {
                let (a, b) = (field.items.get(a as usize), other.items.get(b as usize));
                a.is_some() && a == b
            })
    })
}

impl SavedModel {
    /// Loads `bytes`.
    ///
    /// # Errors
    /// When the importer or the model refuses the package.
    pub(super) fn load(bytes: &[u8]) -> Result<SavedModel, String> {
        let clock = crate::engine::harness_options(None)?.clock;
        let model = crate::roundtrip::semantic::load_model_on(bytes, "", clock)?;
        Ok(SavedModel { model })
    }

    /// The index of the first pivot named `name` on the sheet named `sheet`, the one every
    /// facade call naming a sheet and a pivot means.
    pub(super) fn table(&self, sheet: &str, name: &str) -> Option<usize> {
        let workbook = &self.model.workbook;
        let id = (workbook.worksheets.iter().find(|w| w.name == sheet))?.sheet_id;
        (workbook.pivot_tables.iter()).position(|p| p.sheet_id == id && p.name == name)
    }

    /// The pivot's saved `dataCaption`, the Values field's caption, which Excel writes in its UI
    /// language (docs/excel-semantics/pivots.md "A non-English Excel saves its own UI language's
    /// default captions").
    pub(super) fn data_caption(&self, table: usize) -> Option<&str> {
        let pivot = self.model.workbook.pivot_tables.get(table)?;
        pivot.options.data_caption.as_deref()
    }

    /// The records test of `saved`, the pivot at `table` ([`records::check`]).
    ///
    /// # Errors
    /// As [`records::check`], or when the pivot's cache is missing.
    pub(super) fn records(&self, table: usize, saved: &SavedPivot) -> Result<RecordCheck, String> {
        let workbook = &self.model.workbook;
        let pivot = workbook.pivot_tables.get(table).ok_or("no such pivot")?;
        let cache = (workbook.pivot_caches.get(pivot.cache as usize)).ok_or("no cache")?;
        records::check(saved, pivot, cache, self.model.pivot_today())
    }

    /// The `rowItems` and `colItems` the engine's layout of the pivot at `table` gives
    /// (`rustsheet_engine::pivot::lay_out`, T-4147), in the snapshot's form.
    ///
    /// # Errors
    /// Why the pivot has no layout.
    pub(super) fn lines(&self, table: usize) -> Result<(Vec<PivotLine>, Vec<PivotLine>), String> {
        let workbook = &self.model.workbook;
        let pivot = workbook.pivot_tables.get(table).ok_or("no such pivot")?;
        let cache = (workbook.pivot_caches.get(pivot.cache as usize)).ok_or("no cache")?;
        let mut work = MAX_PIVOT_WORK;
        let today = self.model.pivot_today();
        let result = compute_on(pivot, cache, today, &mut work).map_err(|e| e.to_string())?;
        let layout = lay_out(pivot, cache, &result, &mut work).map_err(|e| e.to_string())?;
        let line = |l: &AxisLine| PivotLine {
            t: records::line_type(l.kind).map(str::to_string),
            r: l.repeat,
            i: l.data_field,
            x: l.items.clone(),
        };
        Ok((
            layout.rows.iter().map(line).collect(),
            layout.columns.iter().map(line).collect(),
        ))
    }

    /// The cells the engine lays the pivot at `table` out into from its saved definition and
    /// cache (`Model::plan_pivot_render`, which `Workbook::pivot_table_cells` also calls),
    /// changing nothing; empty cells left out.
    ///
    /// # Errors
    /// Why the pivot does not lay out.
    pub(super) fn render(&self, table: usize) -> Result<Vec<(Addr, Value)>, String> {
        let plan = (self.model)
            .plan_pivot_render(table, &mut PivotBudget::default())
            .map_err(|e| e.to_string())?;
        Ok((plan.placements.into_iter())
            .flat_map(|p| p.cells)
            .map(cell_value)
            .collect())
    }

    /// For a pivot over a `refreshOnLoad` cache, its refresh from the cache's source
    /// (`Model::plan_pivot_refresh`, what the facade's load runs; changing nothing); `None` for
    /// any other pivot.
    pub(super) fn refresh(&self, table: usize) -> Option<Result<Refreshed, String>> {
        let workbook = &self.model.workbook;
        let pivot = workbook.pivot_tables.get(table)?;
        let old =
            (workbook.pivot_caches.get(pivot.cache as usize)).filter(|c| c.refresh_on_load)?;
        let plan = self
            .model
            .plan_pivot_refresh(pivot.cache, &mut PivotBudget::default());
        Some(plan.map_err(|e| e.to_string()).map(|plan| {
            Refreshed {
                same_records: plan
                    .cache
                    .as_ref()
                    .is_some_and(|new| same_records(old, new)),
                cells: (plan.placements.into_iter())
                    .filter(|p| p.table == table)
                    .flat_map(|p| p.cells)
                    .map(cell_value)
                    .collect(),
            }
        }))
    }
}
