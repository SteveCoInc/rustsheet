//! Whether each pivot an xlsx case's file holds is fresh or stale (T-4131, pivot-corpus).
//!
//! A pivot is *fresh* when its cache reproduces the cells Excel saved for it: the engine lays it
//! out again from its saved definition and cache (`Workbook::pivot_table_cells`, which computes
//! what a refresh would write and changes nothing) and every cell of that layout, and of Excel's
//! saved area, equals Excel's cached value. Otherwise it is *stale*: Excel saved cells its cache
//! no longer gives (an edited sheet, a cache saved without records), or the engine cannot yet lay
//! the pivot out as Excel did. Until the PV1 renderer reaches parity both kinds are stale; the
//! re-render check (docs/research/pivot-tables-2026-11-04/plan.md, check (b)) scores fresh pivots
//! only, so a stale one is reported and never scored for render.
//!
//! Only a workbook whose cached values its suite trusts (Excel-saved, `use_cached_values`) is
//! classified: another application's cells are no evidence of what Excel renders.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::cellref::{parse_range, Addr};
use crate::engine::Evaluated;
use crate::runner::CaseResult;
use crate::value::{compare, Value};
use crate::xlsx_snapshot::{PivotDef, WorkbookSnapshot};

/// The largest saved area compared, as the oracle fill caps a captured one
/// (`tools/src/rustsheet_tools/snapshot.py` `MAX_PIVOT_AREA_CELLS`).
const MAX_AREA_CELLS: u64 = 100_000;

/// A pivot as Excel saved it: its place and its area's cached values.
#[derive(Debug, Clone)]
pub(crate) struct SavedPivot {
    sheet: String,
    name: String,
    location: String,
    /// The area compared (1-based, inclusive), or why there is none.
    area: Result<(Addr, Addr), String>,
    /// Excel's non-empty cached values inside `area`.
    cells: BTreeMap<Addr, Value>,
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
                cells,
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
    /// The engine's layout from the saved cache gives every cell Excel saved, and no other.
    Fresh,
    /// It does not, or the pivot cannot be laid out.
    Stale,
}

/// One saved pivot's classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotFreshness {
    pub sheet: String,
    pub name: String,
    /// Excel's `<location ref>`.
    pub location: String,
    pub freshness: Freshness,
    /// Cells compared: Excel's saved area and every cell of the engine's layout.
    pub compared: usize,
    /// Of those, the ones whose values differ.
    pub differing: usize,
    /// Why a stale pivot is stale: its first differing cell, or why it was not compared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Classifies each of `saved` against the engine's layout of it in `evaluated`.
pub(crate) fn classify(saved: &[SavedPivot], evaluated: &dyn Evaluated) -> Vec<PivotFreshness> {
    saved.iter().map(|p| classify_one(p, evaluated)).collect()
}

fn classify_one(p: &SavedPivot, evaluated: &dyn Evaluated) -> PivotFreshness {
    let stale = |reason: String| PivotFreshness {
        sheet: p.sheet.clone(),
        name: p.name.clone(),
        location: p.location.clone(),
        freshness: Freshness::Stale,
        compared: 0,
        differing: 0,
        reason: Some(reason),
    };
    let (first, last) = match &p.area {
        Ok(area) => *area,
        Err(e) => return stale(e.clone()),
    };
    let engine: BTreeMap<Addr, Value> = match evaluated.pivot_cells(&p.sheet, &p.name) {
        Ok(cells) => cells.into_iter().collect(),
        Err(e) => return stale(format!("engine: {e}")),
    };
    let mut compared = 0;
    let mut differing = 0;
    let mut reason = None;
    let mut check = |at: Addr, expected: &Value, actual: &Value| {
        compared += 1;
        if !compare(expected, actual).is_pass() {
            differing += 1;
            reason.get_or_insert_with(|| format!("{at}: Excel {expected}, engine {actual}"));
        }
    };
    for row in first.row..=last.row {
        for col in first.col..=last.col {
            let at = Addr { row, col };
            let expected = p.cells.get(&at).unwrap_or(&Value::Empty);
            check(at, expected, engine.get(&at).unwrap_or(&Value::Empty));
        }
    }
    // A cell of the engine's layout outside Excel's area: Excel saved it empty, or the pivot
    // holds no such cell at all.
    for (at, actual) in engine.iter().filter(|(a, _)| !inside(**a, first, last)) {
        check(*at, &Value::Empty, actual);
    }
    PivotFreshness {
        sheet: p.sheet.clone(),
        name: p.name.clone(),
        location: p.location.clone(),
        freshness: if differing == 0 {
            Freshness::Fresh
        } else {
            Freshness::Stale
        },
        compared,
        differing,
        reason,
    }
}

/// report.md's "Pivot freshness" section: the counts by suite, and every stale pivot.
pub(crate) fn write_markdown(cases: &[CaseResult], md: &mut String) {
    let rows: Vec<(&CaseResult, &PivotFreshness)> = cases
        .iter()
        .flat_map(|c| c.pivot_freshness.iter().map(move |p| (c, p)))
        .collect();
    if rows.is_empty() {
        return;
    }
    let mut by_suite: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (case, p) in &rows {
        let counts = by_suite.entry(case.suite.as_str()).or_default();
        match p.freshness {
            Freshness::Fresh => counts.0 += 1,
            Freshness::Stale => counts.1 += 1,
        }
    }
    let _ = writeln!(
        md,
        "## Pivot freshness\n\nEach pivot of an Excel-saved workbook whose cache the engine lays out \
         again into exactly Excel's saved cells is fresh; any other is stale, reported here and \
         never scored for render (T-4131). Stale includes pivots the engine does not yet lay out as \
         Excel did, not only outdated caches (T-4906).\n"
    );
    let _ = writeln!(md, "| suite | fresh | stale |\n|---|---|---|");
    for (suite, (fresh, stale)) in &by_suite {
        let _ = writeln!(md, "| {suite} | {fresh} | {stale} |");
    }
    let _ = writeln!(
        md,
        "\n| case | pivot | location | differing / compared | reason |\n|---|---|---|---|---|"
    );
    for (case, p) in rows.iter().filter(|(_, p)| p.freshness == Freshness::Stale) {
        let reason = p.reason.as_deref().unwrap_or("-").replace('|', "\\|");
        let _ = writeln!(
            md,
            "| {} | {}!{} | {} | {} / {} | {reason} |",
            case.id, p.sheet, p.name, p.location, p.differing, p.compared
        );
    }
    let _ = writeln!(md);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::CaseStatus;

    struct Laid(Result<Vec<(Addr, Value)>, String>);
    impl Evaluated for Laid {
        fn value(&self, _: &str, _: Addr) -> Option<Value> {
            None
        }
        fn stored_cells(&self, _: &str) -> Vec<Addr> {
            Vec::new()
        }
        fn pivot_cells(&self, _: &str, _: &str) -> Result<Vec<(Addr, Value)>, String> {
            self.0.clone()
        }
    }

    fn a(s: &str) -> Addr {
        Addr::parse(s).expect("a1")
    }

    fn def(location: &str, pages: u32) -> PivotDef {
        PivotDef {
            name: "Pv".into(),
            sheet: "P".into(),
            location: location.into(),
            first_header_row: 1,
            first_data_row: 1,
            first_data_col: 1,
            row_page_count: pages,
            col_page_count: u32::from(pages > 0),
            row_items: Vec::new(),
            col_items: Vec::new(),
            cache_fields: Vec::new(),
        }
    }

    fn saved(cells: &[(&str, Value)]) -> SavedPivot {
        SavedPivot {
            sheet: "P".into(),
            name: "Pv".into(),
            location: "A3:B4".into(),
            area: saved_area(&def("A3:B4", 0)).ok_or_else(String::new),
            cells: cells.iter().map(|(c, v)| (a(c), v.clone())).collect(),
        }
    }

    fn excel() -> Vec<(&'static str, Value)> {
        vec![
            ("A3", Value::Text("Row Labels".into())),
            ("B3", Value::Text("Sum of x".into())),
            ("A4", Value::Text("Grand Total".into())),
            ("B4", Value::Number(6.0)),
        ]
    }

    fn engine(cells: &[(&str, Value)]) -> Laid {
        Laid(Ok(cells.iter().map(|(c, v)| (a(c), v.clone())).collect()))
    }

    #[test]
    fn the_saved_area_takes_the_page_rows_above_the_location() {
        assert_eq!(saved_area(&def("A3:B4", 0)), Some((a("A3"), a("B4"))));
        // One page field: its row and the blank row before the location.
        assert_eq!(saved_area(&def("A4:B6", 1)), Some((a("A2"), a("B6"))));
        assert_eq!(saved_area(&def("A1:B2", 3)), Some((a("A1"), a("B2"))));
        assert_eq!(saved_area(&def("nope", 0)), None);
    }

    #[test]
    fn a_pivot_whose_layout_gives_exactly_excels_cells_is_fresh() {
        let got = classify(&[saved(&excel())], &engine(&excel()));
        assert_eq!(got[0].freshness, Freshness::Fresh);
        assert_eq!((got[0].compared, got[0].differing), (4, 0));
        assert_eq!(got[0].reason, None);
    }

    #[test]
    fn a_different_missing_or_extra_cell_makes_it_stale() {
        let mut other = excel();
        other[3].1 = Value::Number(7.0);
        let got = classify(&[saved(&excel())], &engine(&other));
        assert_eq!(got[0].freshness, Freshness::Stale);
        assert_eq!(got[0].differing, 1);
        assert_eq!(got[0].reason.as_deref(), Some("B4: Excel 6.0, engine 7.0"));
        // A cell the engine leaves out.
        let got = classify(&[saved(&excel())], &engine(&excel()[..3]));
        assert_eq!(got[0].differing, 1);
        // A cell beyond Excel's area.
        let mut longer = excel();
        longer.push(("A5", Value::Text("x".into())));
        let got = classify(&[saved(&excel())], &engine(&longer));
        assert_eq!((got[0].compared, got[0].differing), (5, 1));
    }

    #[test]
    fn a_pivot_the_engine_cannot_lay_out_or_too_large_to_compare_is_stale() {
        let got = classify(&[saved(&excel())], &Laid(Err("no records".into())));
        assert_eq!(got[0].freshness, Freshness::Stale);
        assert_eq!(got[0].reason.as_deref(), Some("engine: no records"));
        let mut huge = saved(&[]);
        huge.area = Err("area over 100000 cells, not compared".into());
        assert_eq!(
            classify(&[huge], &engine(&[]))[0].freshness,
            Freshness::Stale
        );
    }

    #[test]
    fn the_report_counts_by_suite_and_lists_stale_pivots() {
        let mut md = String::new();
        write_markdown(&[CaseResult::empty("s/x", "s", CaseStatus::Pass)], &mut md);
        assert!(md.is_empty());
        let mut case = CaseResult::empty("s/x.xlsx", "s", CaseStatus::Pass);
        let mut stale_cells = excel();
        stale_cells[3].1 = Value::Number(7.0);
        case.pivot_freshness = classify(&[saved(&excel())], &engine(&excel()));
        case.pivot_freshness
            .extend(classify(&[saved(&excel())], &engine(&stale_cells)));
        write_markdown(&[case], &mut md);
        assert!(md.contains("| s | 1 | 1 |"), "{md}");
        assert!(md.contains("| s/x.xlsx | P!Pv | A3:B4 | 1 / 4 | B4: Excel 6.0, engine 7.0 |"));
    }
}
