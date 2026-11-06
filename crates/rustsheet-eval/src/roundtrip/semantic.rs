//! The semantic items of SPEC-0033 D2 and D3: what the engine's typed model holds, compared as
//! meaning, with every index resolved (shared strings, style records, number-format ids). Built
//! from the importer's own model (CLAUDE.md conventions), never from a second xlsx parser.
//!
//! A workbook is flattened into facts: a key (`cell Sheet!A1 style`, `merge Sheet!A1:B2`, ...)
//! and a canonical text. Two fact sets compare key by key: a missing key is `lost`, a new one
//! `added`, a different text `changed`.
//!
//! Which pair a fact compares (T-4513):
//! - inputs (formula text or constant), styles and structure compare the source as loaded
//!   against the output as loaded;
//! - cached values compare the recalculated workbook that was saved against the output as
//!   loaded.
//!
//! An engine-parity difference is the cell suites' to score, never an I/O loss here.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;

use rustsheet::{Value, Workbook};
use rustsheet_engine::language::get_default_language;
use rustsheet_engine::types::Cell;
use rustsheet_engine::Model;

use super::package::Problem;
use super::xmldiff::clip;
use super::Verdict;
use crate::cellref::Addr;
use crate::xlsx_snapshot::{FormulaKind, WorkbookSnapshot};

/// A workbook's facts: key → canonical text.
pub(crate) type Facts = BTreeMap<String, String>;

/// `A1` for 1-based `row` and `column`, through the harness's one address formatter (`Addr`). A
/// coordinate outside `u32` cannot come from a loaded sheet; it prints as row or column 0.
fn a1(row: i32, column: i32) -> String {
    Addr {
        row: u32::try_from(row).unwrap_or(0),
        col: u32::try_from(column).unwrap_or(0),
    }
    .to_string()
}

/// Loads `bytes` into the engine's model the way the facade does, without recalculating.
///
/// # Errors
/// When the importer refuses the package.
pub(crate) fn load_model(bytes: &[u8], name: &str) -> Result<Model, String> {
    let workbook = rustsheet_xlsx::import::load_from_xlsx_bytes(bytes, name, "en", "UTC")
        .map_err(|e| e.to_string())?;
    Model::from_workbook(workbook, "en")
}

/// The input, style and structure facts of a loaded model (D3.1–D3.5).
pub(crate) fn model_facts(model: &Model) -> Facts {
    let mut facts = Facts::new();
    let wb = &model.workbook;
    for (index, sheet) in wb.worksheets.iter().enumerate() {
        let Ok(idx) = u32::try_from(index) else {
            continue;
        };
        let name = &sheet.name;
        facts.insert(format!("sheet {name} order"), index.to_string());
        facts.insert(format!("sheet {name} state"), format!("{:?}", sheet.state));
        facts.insert(
            format!("sheet {name} tab-color"),
            format!("{:?}", sheet.color),
        );
        facts.insert(
            format!("sheet {name} frozen"),
            format!("{},{}", sheet.frozen_rows, sheet.frozen_columns),
        );
        facts.insert(
            format!("sheet {name} grid-lines"),
            sheet.show_grid_lines.to_string(),
        );
        for (row, column, cell) in sheet.sheet_data.cells() {
            let at = format!("{name}!{}", a1(row, column));
            // Which cells hold a computed value (a formula or a spill member), for `value_facts`.
            // Their stored text and constants come from the file itself (`stored_facts`), so a
            // printer bug cannot hide behind the engine printing both sides the same way.
            if matches!(cell, Cell::SpillCell { .. })
                || matches!(model.get_cell_formula(idx, row, column), Ok(Some(_)))
            {
                facts.insert(format!("cell {at} computed"), String::new());
            }
            if let Ok(style) = model.get_style_for_cell(idx, row, column) {
                // A style compares as its resolved properties, never its index (D3.4).
                let text = serde_json::to_string(&style).unwrap_or_default();
                facts.insert(format!("cell {at} style"), text);
            }
        }
        for m in &sheet.merged_cells {
            let range = format!(
                "{}:{}",
                a1(m.row, m.column),
                a1(
                    m.row.saturating_add(m.height).saturating_sub(1),
                    m.column.saturating_add(m.width).saturating_sub(1),
                )
            );
            facts.insert(format!("merge {name}!{range}"), String::new());
        }
        for ((row, column), link) in &sheet.links {
            facts.insert(
                format!("link {name}!{}", a1(*row, *column)),
                format!("{link:?}"),
            );
        }
        for cf in &sheet.conditional_formatting {
            let rule = serde_json::to_string(&cf.cf_rule).unwrap_or_default();
            facts.insert(
                format!("cf {name}!{} priority {}", cf.range, cf.priority),
                rule,
            );
        }
    }
    for name in &wb.defined_names {
        let scope = name
            .sheet_id
            .and_then(|id| wb.worksheets.iter().find(|s| s.sheet_id == id))
            .map_or_else(String::new, |s| format!("{}!", s.name));
        facts.insert(format!("name {scope}{}", name.name), name.formula.clone());
    }
    for table in wb.tables.values() {
        let columns: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
        facts.insert(
            format!("table {}", table.name),
            format!(
                "{}!{} header {} totals {} filters {} columns {columns:?}",
                table.sheet_name,
                table.reference,
                table.header_row_count,
                table.totals_row_count,
                table.has_filters
            ),
        );
    }
    facts
}

/// The stored formula text and constant value of every cell, as the snapshot reader (the
/// harness's one reader of stored cells) reads the file (D3.1). Formula text drops the leading
/// spaces, TABs and LFs Excel itself drops on import (SPEC-0033 X5) and compares `_xlfn.` prefixes
/// as Excel stores them ([`canonical_prefixes`]); an empty-string constant is
/// not a blank (T-4148). Array members carry no input of their own.
pub(crate) fn stored_facts(snapshot: &WorkbookSnapshot) -> Facts {
    let mut facts = Facts::new();
    for sheet in &snapshot.sheets {
        for (addr, cell) in &sheet.cells {
            let at = format!("{}!{addr}", sheet.name);
            match (&cell.formula_kind, &cell.formula) {
                (None, _) => {
                    facts.insert(format!("cell {at} input"), format!("{:?}", cell.value));
                }
                // An array member, or a formula cell with no text: no input of its own.
                (Some(FormulaKind::ArrayMember { .. }), _) | (Some(_), None) => {}
                (Some(_), Some(text)) => {
                    let text = text.trim_start_matches([' ', '\t', '\n']);
                    facts.insert(format!("cell {at} formula"), canonical_prefixes(text));
                }
            }
        }
    }
    facts
}

/// `text` with each `_xlfn.` (or `_xlfn._xlws.`) prefix dropped where the function's stored form has
/// none: Excel re-stores a hand-written prefix on a function that needs none without it
/// (xlsx-storage.md "Classic CEILING is stored without `_xlfn.`", T-0172), so the prefix is no
/// difference. The engine's one list of stored forms (`Function::to_xlsx_string`) decides; a
/// prefix it requires, or one the export adds wrongly (T-0417), still compares. A prefix inside a
/// string literal is canonicalized too; no fixture holds one.
fn canonical_prefixes(text: &str) -> String {
    static PREFIXED: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"_xlfn\.(?:_xlws\.)?([A-Za-z][A-Za-z0-9_.]*)\(")
            .unwrap_or_else(|e| unreachable!("static regex: {e}"))
    });
    let functions = &get_default_language().functions;
    PREFIXED
        .replace_all(text, |c: &regex::Captures<'_>| {
            let whole = &c[0];
            match functions.lookup_stored(&whole[..whole.len() - 1]) {
                Some(f) if !f.to_xlsx_string().starts_with('_') => format!("{}(", &c[1]),
                _ => whole.to_string(),
            }
        })
        .into_owned()
}

/// The cached value of every computed cell (a formula or a spill member, as `facts` marks them) of
/// a facade workbook (D3.1), by `Sheet!A1`. An error compares by its kind: the engine's detail
/// text is not stored in the file.
pub(crate) fn value_facts(workbook: &Workbook, facts_of: &Facts) -> Facts {
    let mut facts = Facts::new();
    for meta in workbook.sheets() {
        let Ok(cells) = workbook.stored_cells(meta.id) else {
            continue;
        };
        for r in cells {
            let at = format!("{}!{r}", meta.name);
            if !facts_of.contains_key(&format!("cell {at} computed")) {
                continue;
            }
            if let Ok(value) = workbook.value(r) {
                let text = match value {
                    Value::Error(e) => format!("Error({:?})", e.kind),
                    other => format!("{other:?}"),
                };
                facts.insert(format!("value {at}"), text);
            }
        }
    }
    facts
}

/// The differences between two fact sets: `(key, verdict, detail)`, and how many source facts
/// were compared.
pub(crate) fn compare(
    source: &Facts,
    output: &Facts,
) -> (Vec<(String, Verdict, Option<String>)>, usize) {
    let mut out = Vec::new();
    for (key, was) in source {
        match output.get(key) {
            None => out.push((key.clone(), Verdict::Lost, None)),
            Some(now) if now != was => {
                let detail = format!("{} became {}", clip(was), clip(now));
                out.push((key.clone(), Verdict::Changed, Some(detail)));
            }
            Some(_) => {}
        }
    }
    for key in output.keys().filter(|k| !source.contains_key(*k)) {
        out.push((key.clone(), Verdict::Added, None));
    }
    (out, source.len())
}

/// V7 (D6): every cell's style index names a style record, and every shared-string cell names a
/// shared string.
pub(crate) fn validate_indexes(model: &Model) -> Vec<Problem> {
    let wb = &model.workbook;
    let styles = wb.styles.cell_xfs.len();
    let strings = wb.shared_strings.len();
    let mut out = Vec::new();
    for sheet in &wb.worksheets {
        for (row, column, cell) in sheet.sheet_data.cells() {
            let at = format!("{}!{}", sheet.name, a1(row, column));
            let s = cell.get_style();
            if usize::try_from(s).map_or(true, |s| s >= styles) {
                out.push(Problem {
                    check: "V7",
                    detail: format!("{at} uses style {s} of {styles}"),
                });
            }
            if let Cell::SharedString { si, .. } = cell {
                if usize::try_from(*si).map_or(true, |i| i >= strings) {
                    out.push(Problem {
                        check: "V7",
                        detail: format!("{at} uses shared string {si} of {strings}"),
                    });
                }
            }
        }
    }
    out
}
