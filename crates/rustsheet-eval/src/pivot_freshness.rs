//! Whether each pivot an xlsx case's file holds is fresh or stale, and the scores of its re-render
//! and its refresh (T-4131, T-4906; docs/research/pivot-tables-2026-11-04/plan.md, checks (b) and
//! (c)).
//!
//! **Freshness** is a property of the file: a pivot is *fresh* when its saved cache records
//! reproduce every value Excel saved for it, read at the cells Excel's own `<rowItems>` and
//! `<colItems>` place them (`records`). It does not depend on the renderer: labels and headers
//! are left out, and the values come from the engine's result tree, not its layout. Anything
//! else is *stale*: a value the records do not give (an edited sheet, a cache saved without
//! records), or a pivot the test cannot read (no data field, no layout lines, a pivot the
//! importer skips).
//!
//! **Check (b), the re-render.** Every pivot is laid out again from its saved definition and
//! cache (`Model::plan_pivot_render`) and compared with Excel's saved area, cell by cell, labels
//! included: T-4131's comparison, now the render score. A fresh pivot's area cells are scored
//! cells of the case, ratcheted by the baseline like any other; a stale pivot's render is
//! reported and never scored, since its saved cells are no evidence of what Excel renders from
//! its cache.
//!
//! **Check (c), the refresh.** A pivot over a `refreshOnLoad` cache is refreshed from its source
//! (`Model::plan_pivot_refresh`, what the facade's load runs) and the refresh's cells compared
//! with Excel's saved area. Excel re-renders such a pivot on open, so its saved cells are the
//! render before that refresh (docs/excel-semantics/pivots.md "Excel for the web re-renders a
//! refreshOnLoad pivot when it opens the file"). They are compared only when the pivot is fresh
//! and the source still holds the records the cache saved: then they are Excel's render of the
//! very records the refresh reads. Otherwise the check is reported as not comparable. It is
//! reported, not scored: where it is comparable its cells are check (b)'s.
//!
//! Only a workbook whose cached values its suite trusts (Excel-saved, `use_cached_values`) is
//! classified: another application's cells are no evidence of what Excel renders.

mod records;
mod saved_model;
mod summary;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::cellref::{parse_range, Addr};
use crate::closeness::Visible;
use crate::runner::{CellKind, CellResult, Target};
use crate::value::{compare, Value};
use crate::xlsx_snapshot::{PivotDef, PivotLine, WorkbookSnapshot};
use saved_model::SavedModel;
pub use summary::FreshRenderSuite;
pub(crate) use summary::{summarize, write_markdown};

/// The largest saved area compared, as the oracle fill caps a captured one
/// (`tools/src/rustsheet_tools/snapshot.py` `MAX_PIVOT_AREA_CELLS`).
const MAX_AREA_CELLS: u64 = 100_000;

/// The saved `dataCaption`s of a pivot whose default captions are English.
const EN_US_DATA_CAPTIONS: [&str; 2] = ["Values", "Data"];

/// A pivot as Excel saved it: its place, its layout lines and its area's cached values.
#[derive(Debug, Clone)]
pub(crate) struct SavedPivot {
    sheet: String,
    name: String,
    location: String,
    /// The area compared (1-based, inclusive), or why there is none.
    area: Result<(Addr, Addr), String>,
    /// `<location>`'s top-left cell, which `first_data_row` and `first_data_col` count from.
    anchor: Addr,
    first_data_row: u32,
    first_data_col: u32,
    row_items: Vec<PivotLine>,
    col_items: Vec<PivotLine>,
    /// Excel's non-empty cached values inside `area`.
    cells: BTreeMap<Addr, Value>,
    /// Whether the workbook is Excel's, so its cells are Excel evidence.
    from_excel: bool,
}

/// The saved area of `pivot`, 1-based and inclusive: its `location` and the page-field rows above
/// it (`row_page_count` rows and the blank row before the location), at least as wide as the page
/// fields. The same area the oracle fill captures (`snapshot.py` `pivot_area`).
fn saved_area(pivot: &PivotDef) -> Option<(Addr, Addr)> {
    let (mut first, mut last) = parse_range(&pivot.location)?;
    if pivot.row_page_count > 0 {
        first.row = first
            .row
            .saturating_sub(pivot.row_page_count.saturating_add(1))
            .max(1);
        let page_cols = pivot.col_page_count.max(1).saturating_mul(3);
        last.col = last
            .col
            .max(first.col.saturating_add(page_cols).saturating_sub(2));
    }
    Some((first, last))
}

/// Every pivot of `snapshot` with its saved area's values.
pub(crate) fn saved_pivots(snapshot: &WorkbookSnapshot) -> Vec<SavedPivot> {
    let from_excel = snapshot.is_excel_authored();
    snapshot
        .pivots
        .iter()
        .map(|p| {
            let area = saved_area(p)
                .ok_or_else(|| format!("location {:?} does not parse", p.location))
                .and_then(|(first, last)| {
                    let rows = u64::from(last.row.saturating_sub(first.row)) + 1;
                    let cols = u64::from(last.col.saturating_sub(first.col)) + 1;
                    if rows * cols > MAX_AREA_CELLS {
                        Err(format!("area over {MAX_AREA_CELLS} cells, not compared"))
                    } else {
                        Ok((first, last))
                    }
                });
            let cells = match (&area, snapshot.sheets.iter().find(|s| s.name == p.sheet)) {
                (Ok((first, last)), Some(sheet)) => (sheet.cells.iter())
                    .filter(|(a, c)| inside(**a, *first, *last) && c.value != Value::Empty)
                    .map(|(a, c)| (*a, c.value.clone()))
                    .collect(),
                _ => BTreeMap::new(),
            };
            SavedPivot {
                sheet: p.sheet.clone(),
                name: p.name.clone(),
                location: p.location.clone(),
                area,
                anchor: parse_range(&p.location).map_or(Addr { row: 1, col: 1 }, |(a, _)| a),
                first_data_row: p.first_data_row,
                first_data_col: p.first_data_col,
                row_items: p.row_items.clone(),
                col_items: p.col_items.clone(),
                cells,
                from_excel,
            }
        })
        .collect()
}

fn inside(a: Addr, first: Addr, last: Addr) -> bool {
    (first.row..=last.row).contains(&a.row) && (first.col..=last.col).contains(&a.col)
}

/// Fresh or stale; see the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Freshness {
    /// The saved cache records give every value Excel saved, where Excel's lines place it.
    Fresh,
    /// They do not, or the pivot cannot be checked.
    Stale,
}

/// Check (b) for one pivot: its re-render against Excel's saved area.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderScore {
    /// Cells compared: Excel's saved area and every cell of the engine's layout.
    pub compared: usize,
    /// Of those, the ones whose values differ.
    pub differing: usize,
    /// Of those, the ones visibly off (decision 0083).
    pub visibly_off: usize,
    /// Whether the engine's `rowItems` and `colItems` (T-4147) are the ones Excel saved; `None`
    /// when the engine has no layout of the pivot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines_equal: Option<bool>,
    /// The first differing cell, or why the pivot was not laid out or compared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
}

/// What check (c) could do for a `refreshOnLoad` pivot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshOutcome {
    /// The refresh's cells were compared with Excel's saved area.
    Compared,
    /// Excel's saved cells are no evidence of the refresh: the pivot is stale, or its source no
    /// longer holds the records its cache saved.
    NotComparable,
    /// The engine did not refresh the pivot.
    Failed,
}

/// Check (c) for one `refreshOnLoad` pivot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefreshCheck {
    pub outcome: RefreshOutcome,
    /// Cells compared, when [`RefreshOutcome::Compared`].
    pub compared: usize,
    /// Of those, the ones whose values differ.
    pub differing: usize,
    /// The first differing cell, or why the check was not compared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
}

/// One saved pivot's classification and scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotFreshness {
    pub sheet: String,
    pub name: String,
    /// Excel's `<location ref>`.
    pub location: String,
    pub freshness: Freshness,
    /// Value cells of Excel's layout the records test compared.
    pub compared: usize,
    /// Of those, the ones whose values differ.
    pub differing: usize,
    /// Why a stale pivot is stale: its first differing value cell, or why it was not checked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Check (b); `None` when the pivot has no saved area to compare.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render: Option<RenderScore>,
    /// Why a fresh pivot's render is reported but not scored: it was saved by a non-en-US Excel,
    /// whose default captions are no evidence of what en-US Excel renders (decision 0030).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_unscored: Option<String>,
    /// Check (c), for a pivot over a `refreshOnLoad` cache.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh: Option<RefreshCheck>,
}

/// Every cell of `p`'s saved area against `engine`'s value there, then every cell of `engine`
/// outside the area against empty (Excel saved it empty, or the pivot holds no such cell): (cell,
/// Excel's value, the engine's).
fn compare_area(
    p: &SavedPivot,
    (first, last): (Addr, Addr),
    engine: &[(Addr, Value)],
) -> Vec<(Addr, Value, Value)> {
    let engine: BTreeMap<Addr, &Value> = engine.iter().map(|(a, v)| (*a, v)).collect();
    let mut out = Vec::new();
    for row in first.row..=last.row {
        for col in first.col..=last.col {
            let at = Addr { row, col };
            let excel = p.cells.get(&at).cloned().unwrap_or(Value::Empty);
            let actual = engine.get(&at).map_or(Value::Empty, |v| (*v).clone());
            out.push((at, excel, actual));
        }
    }
    for (at, actual) in engine.iter().filter(|(a, _)| !inside(**a, first, last)) {
        out.push((*at, Value::Empty, (*actual).clone()));
    }
    out
}

/// The first of `compared` whose values differ, as `A1: Excel x, engine y`.
fn first_difference(compared: &[(Addr, Value, Value)], engine: &str) -> Option<String> {
    (compared.iter())
        .find(|(_, excel, actual)| !compare(excel, actual).is_pass())
        .map(|(at, excel, actual)| format!("{at}: Excel {excel}, {engine} {actual}"))
}

/// `compared` as scored cells of `p`'s sheet.
fn render_cells(p: &SavedPivot, compared: Vec<(Addr, Value, Value)>) -> Vec<CellResult> {
    (compared.into_iter())
        .map(|(at, expected, actual)| CellResult {
            sheet: p.sheet.clone(),
            cell: at.to_string(),
            formula: None,
            anchor_formula: None,
            verdict: compare(&expected, &actual),
            expected,
            actual,
            from_excel: p.from_excel,
            listed_deviation: false,
            graph_subnormal: false,
            kind: CellKind::Scored,
            math: None,
            target: Target::Excel,
        })
        .collect()
}

/// Classifies each of `saved`, pivots of the xlsx `bytes`, and scores its render and refresh.
/// Returns the classifications and the scored render cells of the fresh pivots.
pub(crate) fn classify(
    saved: &[SavedPivot],
    bytes: &[u8],
) -> (Vec<PivotFreshness>, Vec<CellResult>) {
    if saved.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let model = SavedModel::load(bytes);
    let mut cells = Vec::new();
    let pivots = (saved.iter())
        .map(|p| {
            let (freshness, render) = match &model {
                Ok(model) => classify_one(p, model),
                Err(e) => (stale(p, format!("import: {e}")), None),
            };
            if let Some(render) = render.filter(|_| freshness.freshness == Freshness::Fresh) {
                cells.extend(render);
            }
            freshness
        })
        .collect();
    (pivots, cells)
}

/// A stale classification of `p`, unchecked, for `reason`.
fn stale(p: &SavedPivot, reason: String) -> PivotFreshness {
    PivotFreshness {
        sheet: p.sheet.clone(),
        name: p.name.clone(),
        location: p.location.clone(),
        freshness: Freshness::Stale,
        compared: 0,
        differing: 0,
        reason: Some(reason),
        render: None,
        render_unscored: None,
        refresh: None,
    }
}

/// `p`'s classification and scores, and its render's cells.
fn classify_one(p: &SavedPivot, model: &SavedModel) -> (PivotFreshness, Option<Vec<CellResult>>) {
    let Some(table) = model.table(&p.sheet, &p.name) else {
        return (stale(p, "the importer did not load the pivot".into()), None);
    };
    let mut out = match model.records(table, p) {
        Ok(check) => PivotFreshness {
            freshness: if check.differing == 0 && check.compared > 0 {
                Freshness::Fresh
            } else {
                Freshness::Stale
            },
            compared: check.compared,
            differing: check.differing,
            reason: check
                .first
                .or_else(|| (check.compared == 0).then(|| "no value cell to compare".into())),
            ..stale(p, String::new())
        },
        Err(e) => stale(p, e),
    };
    let Ok(area) = p.area.clone() else {
        out.reason = out.reason.or_else(|| p.area.clone().err());
        return (out, None);
    };
    // A pivot the engine does not lay out writes none of its cells: a fresh one's are scored so.
    let (engine, why) = match model.render(table) {
        Ok(engine) => (engine, None),
        Err(e) => (Vec::new(), Some(format!("engine: {e}"))),
    };
    let compared = compare_area(p, area, &engine);
    let first = why.or_else(|| first_difference(&compared, "engine"));
    let scored = render_cells(p, compared);
    // Excel writes an item's line without `t` or as `t="data"`.
    let plain = |lines: &[PivotLine]| -> Vec<PivotLine> {
        let mut lines = lines.to_vec();
        for line in &mut lines {
            line.t = line.t.take().filter(|t| t != "data");
        }
        lines
    };
    let lines_equal = (model.lines(table).ok())
        .map(|(rows, columns)| rows == plain(&p.row_items) && columns == plain(&p.col_items));
    out.render = Some(RenderScore {
        compared: scored.len(),
        differing: scored.iter().filter(|c| !c.verdict.is_pass()).count(),
        visibly_off: scored.iter().filter(|c| Visible::is_off(c)).count(),
        lines_equal,
        first,
    });
    out.refresh = model.refresh(table).map(|refreshed| {
        let not = |outcome, why: &str| RefreshCheck {
            outcome,
            compared: 0,
            differing: 0,
            first: Some(why.to_string()),
        };
        match refreshed {
            Err(e) => not(RefreshOutcome::Failed, &e),
            Ok(_) if out.freshness == Freshness::Stale => not(
                RefreshOutcome::NotComparable,
                "stale: the saved cells are not a render of the saved records",
            ),
            Ok(r) if !r.same_records => not(
                RefreshOutcome::NotComparable,
                "the source no longer holds the saved records, so the saved cells predate the refresh",
            ),
            Ok(r) => {
                let compared = compare_area(p, area, &r.cells);
                RefreshCheck {
                    outcome: RefreshOutcome::Compared,
                    compared: compared.len(),
                    differing: (compared.iter())
                        .filter(|(_, e, a)| !compare(e, a).is_pass())
                        .count(),
                    first: first_difference(&compared, "refresh"),
                }
            }
        }
    });
    // `Values` is the en-US data caption (`Data` in a file first saved by an Excel before 2007
    // or by LibreOffice, whose other captions are English too); a non-English Excel saves its
    // own (pivots.md "A non-English Excel saves its own UI language's default captions").
    if let Some(caption) = (model.data_caption(table)).filter(|c| !EN_US_DATA_CAPTIONS.contains(c))
    {
        out.render_unscored = Some(format!(
            "dataCaption {caption:?}: saved by a non-en-US Excel, whose default captions are no \
             en-US evidence (decision 0030)"
        ));
        return (out, None);
    }
    (out, Some(scored))
}
