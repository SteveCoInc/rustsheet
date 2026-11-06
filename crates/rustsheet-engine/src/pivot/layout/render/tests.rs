#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The rendered cells of every oracle-rendered probe pivot in scope, against the `pivots` cells
//! Excel saved in the probes' sidecars: the definitions come from the case files through the
//! shared probe harness (`pivot::probe_cases`), the expectations from the sidecars, never from
//! the engine. Every sidecar cell is compared, an empty one included, and every rendered cell
//! must lie in the sidecar's capture window.
//!
//! Out of scope, and why:
//! - probes/t4123-pivot-capture T to T4 column D (`Pct`, `showDataAs="percentOfTotal"`): show
//!   values as is T-4163's; the caption D4 is compared;
//! - probes/t4128-pivot-spike: S is the `#SPILL!` collapse (T-4151), R's source holds a
//!   formula the case file does not evaluate (T-4152), G is grouped (T-4171) and K has a
//!   calculated field, a Top-N filter and a sort (T-4174, T-4165);
//! - probes/t4622-pivot-count-subtotals East's custom Count subtotal values over an error under
//!   a Sum field (CA, CN C6; B1, B4, B5 C6:C7): the result's count rule is T-4168's (T-4636);
//! - t4134 I3, t4582, t4585 and t4627: formulas in their sources (T-4152 runs them through the
//!   engine), and t4582 J3, J4 date items against cached numbers (T-4612).

use std::collections::HashMap;

use serde_json::Value as Json;

use super::*;
use crate::expressions::token::get_error_by_english_name;
use crate::pivot::probe_cases::{address, probe_pivot, read};
use crate::pivot::{compute, lay_out, MAX_PIVOT_WORK};

fn rendered(table: &PivotTable, cache: &PivotCache) -> HashMap<String, PivotCell> {
    let mut work = MAX_PIVOT_WORK;
    let result = compute(table, cache, &mut work).unwrap();
    let layout = lay_out(table, cache, &result, &mut work).unwrap();
    let mut cells = HashMap::new();
    for cell in render(table, cache, &result, &layout).unwrap() {
        let at = address(cell.column, cell.row);
        assert!(
            cells.insert(at.clone(), cell.value).is_none(),
            "{at} rendered twice"
        );
    }
    cells
}

/// A sidecar cell as a rendered one; `None` for an empty cell.
fn expected(cell: &Json) -> Option<PivotCell> {
    match cell["t"].as_str().unwrap() {
        "empty" => None,
        "s" => Some(PivotCell::Text(cell["v"].as_str().unwrap().to_string())),
        "n" => Some(PivotCell::Number(cell["v"].as_f64().unwrap())),
        "e" => Some(PivotCell::Error(
            get_error_by_english_name(cell["v"].as_str().unwrap()).unwrap(),
        )),
        other => panic!("cell type {other}"),
    }
}

/// Checks the cells of sheet `name`'s pivot of probe `stem`, leaving out the cells `skip`.
fn check(stem: &str, name: &str, skip: &[&str]) {
    let (case, oracle) = (read(stem, "case"), read(stem, "oracle"));
    let (cache, table) = probe_pivot(&case, name);
    let mut actual = rendered(&table, &cache);
    let pivot = oracle["pivots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["sheet"] == name)
        .unwrap();
    let mut wrong = Vec::new();
    for (at, cell) in pivot["cells"].as_object().unwrap() {
        let mine = actual.remove(at);
        if skip.contains(&at.as_str()) {
            continue;
        }
        let excel = expected(cell);
        if mine != excel {
            wrong.push(format!("{at}: Excel {excel:?}, engine {mine:?}"));
        }
    }
    for (at, cell) in actual {
        wrong.push(format!("{at}: outside the capture window, engine {cell:?}"));
    }
    wrong.sort();
    assert!(wrong.is_empty(), "{stem} {name}:\n{}", wrong.join("\n"));
}

fn check_all(stem: &str, sheets: &[&str]) {
    for sheet in sheets {
        check(stem, sheet, &[]);
    }
}

#[test]
fn the_capture_probes_cells_are_excels() {
    check("t4123-pivot-capture", "C", &[]);
    // Column D is `Pct`, a show-values-as field (T-4163).
    let pct: Vec<String> = (5..=12).map(|row| format!("D{row}")).collect();
    let pct: Vec<&str> = pct.iter().map(String::as_str).collect();
    for sheet in ["T", "T2", "T3", "T4"] {
        check("t4123-pivot-capture", sheet, &pct);
    }
}

#[test]
fn the_geometry_probes_cells_are_excels() {
    // Compact, outline and tabular rows, two column fields, Values on rows and on columns, no
    // row field, no axis field, and wrapped page fields above the location (G8).
    check_all(
        "t4138-pivot-geometry",
        &["G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9"],
    );
}

#[test]
fn the_caption_probes_cells_are_excels() {
    // K1 caption attributes, K2 missingCaption, K3 errorCaption, K4 page fields, K5 custom
    // subtotal labels.
    check_all(
        "t4137-pivot-captions",
        &["K1", "K2", "K3", "K4", "K5", "K6"],
    );
    check_all(
        "t4583-pivot-row-labels",
        &["R1", "R2", "R3", "R4", "R5", "R6", "C1", "C2"],
    );
    // A custom Count subtotal under a Sum field over an error is a count in Excel and the error
    // in the engine's result (T-4168, T-4636): East's subtotal values are left out, their
    // labels compared.
    check("t4622-pivot-count-subtotals", "B2", &[]);
    check("t4622-pivot-count-subtotals", "B3", &[]);
    for sheet in ["CA", "CN"] {
        check("t4622-pivot-count-subtotals", sheet, &["C6"]);
    }
    for sheet in ["B1", "B4", "B5"] {
        check("t4622-pivot-count-subtotals", sheet, &["C6", "C7"]);
    }
}

#[test]
fn the_item_and_aggregate_probes_cells_are_excels() {
    // Booleans, errors and blanks as labels; text that reads as a number.
    check_all("t4134-pivot-items", &["I1", "I2", "I4", "I5"]);
    check_all(
        "t4135-pivot-aggregates",
        &[
            "A_sum",
            "A_count",
            "A_average",
            "A_max",
            "A_min",
            "A_product",
            "A_countNums",
            "A_stdDev",
            "A_stdDevp",
            "A_var",
            "A_varp",
            "EC",
        ],
    );
    check_all("t4589-pivot-total-order", &["S1", "E1", "E2", "E3", "E4"]);
    check_all(
        "t4614-pivot-total-order-2",
        &["S2", "S3", "P2", "C1", "C2", "C3", "C4"],
    );
    check_all(
        "t4616-pivot-total-order-3",
        &[
            "S1", "S2", "S3", "S4", "S5", "S6", "V1", "V2", "V3", "V4", "P1", "P2", "I1", "I2",
            "I3", "I4", "M1", "M2", "M3",
        ],
    );
}

#[test]
fn the_cells_text_is_bounded() {
    // A long item: every cell that shows it holds a copy, and the copies count.
    let (mut cache, table) = crate::pivot::probe_cases::diagonal(3, true);
    let long = "x".repeat(1_000);
    cache.fields[0].items[0] = CacheItem::Text(long.clone());
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let cells = render(&table, &cache, &result, &layout).unwrap();
    let bytes: usize = cells
        .iter()
        .map(|c| match &c.value {
            PivotCell::Text(text) => text.len(),
            _ => 0,
        })
        .sum();
    assert!(cells.contains(&RenderedCell {
        row: 2,
        column: 0,
        value: PivotCell::Text(long)
    }));
    assert_eq!(
        render_within(&table, &cache, &result, &layout, bytes),
        Ok(cells)
    );
    assert_eq!(
        render_within(&table, &cache, &result, &layout, bytes - 1),
        Err(PivotComputeError::Limit)
    );
}
