//! Tests of the pivot freshness test and its render and refresh scores (T-4131, T-4906).

use super::*;
use crate::runner::{CaseResult, CaseStatus};
use crate::xlsx_snapshot::read_snapshot;

/// An Excel-saved workbook (Microsoft Macintosh Excel) with one compact pivot of three row
/// fields over the Table `Table1`, Sheet4!A3:B13.
const EXAMPLE: &[u8] = include_bytes!("../../../rustsheet-xlsx/tests/example.xlsx");
/// A workbook with two `refreshOnLoad` pivots over P's source range: `PivotSum` (P!A3:E7, a
/// column field) and `PivotAvg` (P!A20:B23).
const REFRESH_ON_LOAD: &[u8] = include_bytes!("../../../rustsheet-xlsx/tests/pivot_tables.xlsx");

fn a(s: &str) -> Addr {
    Addr::parse(s).expect("a1")
}

fn saved_of(bytes: &[u8]) -> Vec<SavedPivot> {
    saved_pivots(&read_snapshot(bytes).expect("snapshot"))
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

#[test]
fn the_saved_area_takes_the_page_rows_above_the_location() {
    assert_eq!(saved_area(&def("A3:B4", 0)), Some((a("A3"), a("B4"))));
    // One page field: its row and the blank row before the location.
    assert_eq!(saved_area(&def("A4:B6", 1)), Some((a("A2"), a("B6"))));
    assert_eq!(saved_area(&def("A1:B2", 3)), Some((a("A1"), a("B2"))));
    assert_eq!(saved_area(&def("nope", 0)), None);
}

#[test]
fn a_pivot_whose_records_give_its_saved_values_is_fresh_and_its_render_scored() {
    let saved = saved_of(EXAMPLE);
    let (pivots, cells) = classify(&saved, EXAMPLE);
    let p = &pivots[0];
    assert_eq!(
        (p.sheet.as_str(), p.name.as_str()),
        ("Sheet4", "PivotTable1")
    );
    assert_eq!(p.freshness, Freshness::Fresh, "{p:?}");
    // The leaf lines and the grand total; the outer items' lines are compact parent rows whose
    // subtotals Excel shows at the top, so they are compared too.
    assert_eq!(
        (p.compared, p.differing, p.reason.as_deref()),
        (10, 0, None)
    );
    let render = p.render.as_ref().expect("rendered");
    assert_eq!(render.compared, 22, "every cell of A3:B13");
    assert_eq!(cells.len(), render.compared);
    assert!(cells
        .iter()
        .all(|c| c.sheet == "Sheet4" && c.formula.is_none() && c.from_excel));
    assert_eq!(
        cells.iter().filter(|c| c.verdict.is_pass()).count(),
        render.compared - render.differing
    );
    assert_eq!(
        render.lines_equal,
        Some(true),
        "the engine's rowItems and colItems are Excel's"
    );
    assert_eq!(p.refresh, None, "no refreshOnLoad cache");
}

#[test]
fn a_value_the_records_do_not_give_makes_it_stale_and_unscored() {
    let mut saved = saved_of(EXAMPLE);
    let (at, value) = (saved[0].cells.iter())
        .find(|(_, v)| matches!(v, Value::Number(_)))
        .map(|(a, v)| (*a, v.clone()))
        .expect("a value cell");
    let Value::Number(n) = value else {
        unreachable!()
    };
    saved[0].cells.insert(at, Value::Number(n + 1.0));
    let (pivots, cells) = classify(&saved, EXAMPLE);
    assert_eq!(pivots[0].freshness, Freshness::Stale);
    assert_eq!(pivots[0].differing, 1);
    let reason = pivots[0].reason.as_deref().expect("why");
    assert!(
        reason.starts_with(&format!("{at}: Excel {}", n + 1.0)),
        "{reason}"
    );
    // The render is still reported, never scored.
    assert!(pivots[0].render.as_ref().is_some_and(|r| r.differing >= 1));
    assert!(cells.is_empty());
}

#[test]
fn a_label_does_not_decide_freshness_but_fails_the_render_score() {
    let mut saved = saved_of(EXAMPLE);
    let label = (saved[0].cells.iter())
        .find(|(_, v)| matches!(v, Value::Text(t) if t != "Row Labels"))
        .map(|(a, _)| *a)
        .expect("a label");
    saved[0].cells.insert(label, Value::Text("renamed".into()));
    let (pivots, cells) = classify(&saved, EXAMPLE);
    assert_eq!(pivots[0].freshness, Freshness::Fresh);
    let failing: Vec<_> = cells.iter().filter(|c| !c.verdict.is_pass()).collect();
    assert!(
        failing.iter().any(|c| c.cell == label.to_string()),
        "{failing:?}"
    );
    assert!(pivots[0]
        .render
        .as_ref()
        .is_some_and(|r| r.visibly_off >= 1));
}

#[test]
fn a_pivot_that_cannot_be_checked_is_stale() {
    // An axis with fields and no saved lines.
    let mut saved = saved_of(EXAMPLE);
    saved[0].row_items.clear();
    let (pivots, _) = classify(&saved, EXAMPLE);
    assert_eq!(pivots[0].freshness, Freshness::Stale);
    assert_eq!(
        pivots[0].reason.as_deref(),
        Some("Excel saved no layout lines for an axis with fields")
    );
    // Lines that do not fit the location.
    let mut saved = saved_of(EXAMPLE);
    let extra = saved[0].row_items[0].clone();
    saved[0].row_items.push(extra);
    let (pivots, _) = classify(&saved, EXAMPLE);
    assert_eq!(
        pivots[0].reason.as_deref(),
        Some("Excel's layout lines do not fit its location")
    );
    // A pivot the importer does not hold, and a package it refuses.
    let mut saved = saved_of(EXAMPLE);
    saved[0].name = "Other".into();
    let (pivots, _) = classify(&saved, EXAMPLE);
    assert_eq!(
        pivots[0].reason.as_deref(),
        Some("the importer did not load the pivot")
    );
    let (pivots, cells) = classify(&saved_of(EXAMPLE), b"not a zip");
    assert!(pivots[0]
        .reason
        .as_deref()
        .is_some_and(|r| r.starts_with("import: ")));
    assert!(cells.is_empty());
    let (none, no_cells) = classify(&[], EXAMPLE);
    assert!(none.is_empty() && no_cells.is_empty());
}

#[test]
fn a_refresh_on_load_pivot_is_refreshed_and_compared_when_its_source_holds_its_records() {
    let saved = saved_of(REFRESH_ON_LOAD);
    let (pivots, _) = classify(&saved, REFRESH_ON_LOAD);
    let got: Vec<_> = (pivots.iter())
        .map(|p| {
            let r = p.refresh.as_ref().expect("a refreshOnLoad cache");
            (
                p.name.as_str(),
                p.freshness,
                r.outcome,
                r.compared,
                r.differing,
            )
        })
        .collect();
    let compared = RefreshOutcome::Compared;
    assert_eq!(
        got,
        vec![
            ("PivotSum", Freshness::Fresh, compared, 25, 0),
            ("PivotAvg", Freshness::Fresh, compared, 8, 0),
        ]
    );
    // A stale pivot's saved cells are no evidence of the refresh.
    let mut stale = saved_of(REFRESH_ON_LOAD);
    stale[0].row_items.clear();
    let refresh = classify(&stale, REFRESH_ON_LOAD).0[0].refresh.clone();
    assert_eq!(
        refresh.map(|r| r.outcome),
        Some(RefreshOutcome::NotComparable)
    );
}

#[test]
fn a_non_english_data_caption_leaves_a_fresh_render_unscored() {
    let mut pivots = classify(&saved_of(EXAMPLE), EXAMPLE).0;
    assert_eq!(pivots[0].render_unscored, None);
    pivots[0].render_unscored = Some("dataCaption \"Werte\"".into());
    let mut case = CaseResult::empty("s/x.xlsx", "s", CaseStatus::Pass);
    case.pivot_freshness = pivots;
    let summary = summarize(&[case]);
    assert_eq!((summary[0].fresh, summary[0].fresh_unscored), (1, 1));
    assert_eq!(summary[0].render_cells, 0);
}

#[test]
fn the_report_counts_by_suite_and_lists_stale_pivots() {
    let mut md = String::new();
    write_markdown(&[CaseResult::empty("s/x", "s", CaseStatus::Pass)], &mut md);
    assert!(md.is_empty());
    assert!(summarize(&[]).is_empty());
    let mut case = CaseResult::empty("s/x.xlsx", "s", CaseStatus::Pass);
    let fresh = classify(&saved_of(EXAMPLE), EXAMPLE).0;
    let mut stale = saved_of(EXAMPLE);
    stale[0].row_items.clear();
    case.pivot_freshness = fresh;
    case.pivot_freshness.extend(classify(&stale, EXAMPLE).0);
    let rendered = case.pivot_freshness[0].render.clone().expect("rendered");
    let summary = summarize(std::slice::from_ref(&case));
    assert_eq!(
        summary,
        vec![FreshRenderSuite {
            suite: "s".into(),
            fresh: 1,
            stale: 1,
            fresh_rendered_exactly: usize::from(rendered.differing == 0),
            render_cells: rendered.compared,
            render_passed: rendered.compared - rendered.differing,
            fresh_lines_equal: usize::from(rendered.lines_equal == Some(true)),
            render_visibly_off: rendered.visibly_off,
            ..FreshRenderSuite::default()
        }]
    );
    write_markdown(&[case], &mut md);
    assert!(md.contains("## Pivot freshness"), "{md}");
    assert!(md.contains(
        "| s/x.xlsx | Sheet4!PivotTable1 | A3:B13 | 0 / 0 | Excel saved no layout lines for an axis with fields |"
    ));
}

#[test]
fn an_empty_parent_row_is_compared_unless_its_fields_flags_leave_it_blank() {
    // Sheet4!B4 is the first outer item's own line, which shows its subtotal at the top.
    let mut saved = saved_of(EXAMPLE);
    assert!(matches!(
        saved[0].cells.get(&a("B4")),
        Some(Value::Number(_))
    ));
    saved[0].cells.remove(&a("B4"));
    let (pivots, _) = classify(&saved, EXAMPLE);
    assert_eq!(pivots[0].freshness, Freshness::Stale);
    assert!(pivots[0]
        .reason
        .as_deref()
        .is_some_and(|r| r.starts_with("B4: Excel <empty>")));
    // With the outer field's subtotal at the bottom, Excel leaves that line blank: not compared.
    let mut model = crate::roundtrip::semantic::load_model(EXAMPLE, "").expect("loads");
    let today = model.pivot_today();
    let cache = model.workbook.pivot_caches[0].clone();
    let table = &mut model.workbook.pivot_tables[0];
    let rustsheet_engine::pivot::AxisField::Field(outer) = table.row_fields[0] else {
        panic!("a field")
    };
    table.fields[outer as usize].subtotal_top = false;
    let check = records::check(&saved[0], table, &cache, today).expect("checks");
    assert_eq!((check.compared, check.differing), (9, 0));
}

/// The sidecar `pivots` an oracle fill of `bytes` would capture if Excel rendered each pivot on
/// open as it saved it: every cell of each saved area, empty ones included (T-4972).
fn on_open_of(bytes: &[u8]) -> Vec<OraclePivot> {
    let snapshot = read_snapshot(bytes).expect("snapshot");
    (snapshot.pivots.iter())
        .map(|d| {
            let (first, last) = saved_area(d).expect("an area");
            let sheet = (snapshot.sheets.iter().find(|s| s.name == d.sheet)).expect("its sheet");
            let mut cells = BTreeMap::new();
            for row in first.row..=last.row {
                for col in first.col..=last.col {
                    let at = Addr { row, col };
                    let v = sheet
                        .cells
                        .get(&at)
                        .map_or(Value::Empty, |c| c.value.clone());
                    cells.insert(at.to_string(), v);
                }
            }
            OraclePivot {
                definition: d.clone(),
                cells,
                text: BTreeMap::new(),
            }
        })
        .collect()
}

#[test]
fn excel_s_on_open_render_is_check_c_s_evidence_and_its_cells_are_scored() {
    let mut saved = saved_of(REFRESH_ON_LOAD);
    let mut on_open = on_open_of(REFRESH_ON_LOAD);
    // Excel's on-open render of PivotSum shows another value at its first number cell.
    let sum = (on_open.iter_mut()).find(|o| o.definition.name == "PivotSum");
    let sum = sum.expect("PivotSum");
    let (at, n) = (sum.cells.iter())
        .find_map(|(a, v)| match v {
            Value::Number(n) => Some((a.clone(), *n)),
            _ => None,
        })
        .expect("a number");
    sum.cells.insert(at.clone(), Value::Number(n + 1.0));
    attach_on_open(&mut saved, &on_open);
    // A stale pivot is compared too: the on-open render does not depend on the saved cells.
    saved[1].row_items.clear();
    let (pivots, cells) = classify(&saved, REFRESH_ON_LOAD);
    let got: Vec<_> = (pivots.iter())
        .map(|p| {
            let r = p.refresh.as_ref().expect("a refreshOnLoad cache");
            (
                p.name.as_str(),
                r.outcome,
                r.on_open,
                r.compared,
                r.differing,
            )
        })
        .collect();
    let compared = RefreshOutcome::Compared;
    assert_eq!(
        got,
        vec![
            ("PivotSum", compared, true, 25, 1),
            ("PivotAvg", compared, true, 8, 0)
        ]
    );
    // Their cells are the case's, and check (b) leaves the same cells unscored.
    assert_eq!(cells.len(), 25 + 8);
    let failing: Vec<_> = cells.iter().filter(|c| !c.verdict.is_pass()).collect();
    assert_eq!(failing.len(), 1);
    assert_eq!(failing[0].cell, at);
    assert!(failing[0].from_excel);
    assert!(pivots[0].render_unscored.is_some() && pivots[0].render.is_some());
    let mut case = CaseResult::empty("s/x.xlsx", "s", CaseStatus::Fail);
    case.pivot_freshness = pivots;
    let summary = &summarize(std::slice::from_ref(&case))[0];
    assert_eq!(
        (
            summary.refresh_on_open,
            summary.refresh_cells,
            summary.refresh_passed,
            summary.refresh_compared,
            summary.render_cells
        ),
        (2, 33, 32, 2, 0)
    );
    let mut md = String::new();
    write_markdown(&[case], &mut md);
    assert!(
        md.contains("| s/x.xlsx | P!PivotSum | compared | on open | 1 / 25 |"),
        "{md}"
    );
}

#[test]
fn a_refresh_that_fails_scores_excel_s_on_open_area_against_nothing() {
    let mut saved = saved_of(REFRESH_ON_LOAD);
    attach_on_open(&mut saved, &on_open_of(REFRESH_ON_LOAD));
    let open = saved[1].on_open.clone().expect("attached");
    let (check, cells) = refresh_on_open(&saved[1], &open, Err("no source".into()));
    assert_eq!(check.outcome, RefreshOutcome::Failed);
    assert_eq!(check.first.as_deref(), Some("no source"));
    assert_eq!((check.compared, check.differing), (8, open.cells.len()));
    assert_eq!(cells.len(), 8);
    // A sidecar of other pivots, or an area that does not parse, attaches nothing.
    let mut other = on_open_of(REFRESH_ON_LOAD);
    other[0].definition.name = "Elsewhere".into();
    other[1].definition.location = "nope".into();
    let mut saved = saved_of(REFRESH_ON_LOAD);
    attach_on_open(&mut saved, &other);
    assert!(saved.iter().all(|p| p.on_open.is_none()));
}

#[test]
fn a_pivot_the_engine_cannot_load_scores_excel_s_on_open_area_as_a_failed_refresh() {
    let mut saved = saved_of(REFRESH_ON_LOAD);
    attach_on_open(&mut saved, &on_open_of(REFRESH_ON_LOAD));
    // A pivot the importer does not hold, and a package it refuses (decision 0349).
    saved[1].name = "Other".into();
    for (bytes, avg_why) in [
        (REFRESH_ON_LOAD, "the importer did not load the pivot"),
        (&b"not a zip"[..], "import: "),
    ] {
        let (pivots, cells) = classify(&saved, bytes);
        let r = pivots[1].refresh.as_ref().expect("scored against nothing");
        assert_eq!(
            (r.outcome, r.on_open, r.compared),
            (RefreshOutcome::Failed, true, 8)
        );
        assert!(r.first.as_deref().is_some_and(|w| w.starts_with(avg_why)));
        assert!(pivots[1].render_unscored.is_some() && pivots[1].render.is_none());
        // PivotSum loads and refreshes (25 cells); only a refused package fails it too.
        assert_eq!(cells.len(), 25 + 8);
    }
    // Without Excel's on-open render, an unloaded pivot scores nothing.
    let (pivots, cells) = classify(&saved_of(REFRESH_ON_LOAD), b"not a zip");
    assert!(pivots.iter().all(|p| p.refresh.is_none()) && cells.is_empty());
}
