#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The rendered cells of every oracle-rendered probe pivot in scope, against the `pivots` cells
//! Excel saved in the probes' sidecars: the definitions come from the case files through the
//! shared probe harness (`pivot::probe_cases`), the expectations from the sidecars, never from
//! the engine. Every sidecar cell is compared, an empty one included, and every rendered cell
//! must lie in the sidecar's capture window.
//!
//! Out of scope, and why:
//! - probes/t4128-pivot-spike: S is the `#SPILL!` collapse (T-4151), R's source holds a
//!   formula the case file does not evaluate (T-4152), G is grouped (T-4171) and K has a
//!   calculated field, a Top-N filter and a sort (T-4174, T-4165);
//! - t4134 I3, t4582, t4585 and t4627: formulas in their sources (T-4152 runs them through the
//!   engine), and t4582 J3, J4 date items against cached numbers (T-4612).

use super::*;
use crate::pivot::probe_cases::{
    probe_pivot, probe_pivot_over, read, sidecar_differences, T4634_SHEETS, T4699_SHEETS,
    T4745_SHEETS,
};
use crate::pivot::{compute, lay_out, MAX_PIVOT_WORK};

/// Checks the cells of sheet `name`'s pivot of probe `stem`, leaving out the cells `skip`.
fn check(stem: &str, name: &str, skip: &[&str]) {
    let wrong = differences(stem, name, skip);
    assert!(wrong.is_empty(), "{stem} {name}:\n{}", wrong.join("\n"));
}

/// The cells of sheet `name`'s pivot of probe `stem` that are not Excel's, but the cells `skip`.
fn differences(stem: &str, name: &str, skip: &[&str]) -> Vec<String> {
    let case = read(stem, "case");
    let (cache, table) = probe_pivot(&case, name);
    sidecar_differences(stem, name, &cache, &table, None, skip)
}

fn check_all(stem: &str, sheets: &[&str]) {
    for sheet in sheets {
        check(stem, sheet, &[]);
    }
}

#[test]
fn the_capture_probes_cells_are_excels() {
    // T to T4 hold `Pct`, a percentOfTotal field beside Sum and Count (T-4163).
    check_all("t4123-pivot-capture", &["C", "T", "T2", "T3", "T4"]);
}

/// The sheets of every show-values-as probe (pivots.md "Show-values-as modes as Excel for the web
/// renders them", "Show-values-as edge cases (T-4161)").
const SHOW_AS: &[(&str, &[&str])] = &[
    (
        "t4129-pivot-show-as",
        &["N", "DF", "DP", "PC", "PD", "RT", "PR", "PCo", "PT", "IX"],
    ),
    (
        "t4129-pivot-show-as-x14",
        &["PP", "PPR", "PPC", "PRT", "RA", "RD"],
    ),
    (
        "t4161-pivot-show-as-base-items",
        &[
            "DYP", "DYN", "PYP", "PYN", "QYP", "QYN", "PY1", "DY1", "DPP", "PPN", "QPP", "PPC",
            "DPC", "QPC", "DRP", "PR0", "QRN",
        ],
    ),
    (
        "t4161-pivot-show-as-running-rank",
        &[
            "RAP", "RDP", "RAR", "RDR", "RDY", "RTR", "RTY", "PTR", "PTY", "RTT",
        ],
    ),
    (
        "t4161-pivot-show-as-zeros",
        &[
            "N", "PC", "PD", "PY", "PR", "PCo", "PT", "IX", "PP", "PPR", "PPC", "PRT", "RA", "RD",
        ],
    ),
    (
        "t4161-pivot-show-as-edges",
        &[
            "D0", "P0", "Q0", "PR", "PCo", "PT", "IX", "PPR", "PPC", "PP", "PRY",
        ],
    ),
    ("t4161-pivot-show-as-running-zero", &["PRY", "PRP"]),
    (
        "t4161-pivot-show-as-nested-parent",
        &["PPR_", "PPP", "PPK", "PPRow", "PPCol", "PPY", "IX3"],
    ),
];

#[test]
fn the_show_values_as_probes_cells_are_excels() {
    let wrong: Vec<String> = SHOW_AS
        .iter()
        .flat_map(|(stem, sheets)| {
            sheets
                .iter()
                .flat_map(|sheet| differences(stem, sheet, &[]))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
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
    // Column and page fields at the default (L1, L2), compact versus outline on the outer row
    // and column fields (L3, L4, L7, L8), and rowHeaderCaption losing to the field name (L5).
    check_all(
        "t4609-pivot-row-labels-2",
        &["L1", "L2", "L3", "L4", "L5", "L6", "L7", "L8"],
    );
    // Custom countA and count subtotals, alone and both, under a Sum field over an error (the
    // count ends at the error within its inner-item cell), with no error, and under Count.
    check_all(
        "t4622-pivot-count-subtotals",
        &["CA", "CN", "B1", "B2", "B3", "B4", "B5"],
    );
}

#[test]
fn the_layout_option_probes_cells_are_excels() {
    // Every custom subtotal function's label and value, tabular and compact (F1, F2), custom
    // subtotals over a Count field (F3), insertBlankRow (BR, BR2) and a collapsed item (SD, SD2).
    check_all(
        "t4168-pivot-layout-options",
        &["F1", "F2", "F3", "BR", "BR2", "SD", "SD2"],
    );
    // Blank rows and collapsed items beyond an outer row field (pivots.md "Layout lines").
    for &sheet in T4699_SHEETS.iter().filter(|&&s| s != "CV4") {
        check("t4699-pivot-blank-collapsed", sheet, &[]);
    }
    // CV4!C3 is the Values field's `Values` header in tabular form (T-4914).
    check("t4699-pivot-blank-collapsed", "CV4", &["C3"]);
    // Repeat item labels: per field in tabular (X14F, T3R to TBL), outline (OST, OB, O3), compact
    // (CF, MX) form and on columns (CC); the table-wide default alone fills nothing (X14, TW,
    // TWE, OW).
    check_all("t4129-pivot-layout-flags", &["X14", "X14F"]);
    // Classic layout (gridDropZones, GZ, GC, GV, G1) and subtotalCaption (SC, SO, SB, SK, SF,
    // SCC): pivots.md "Classic layout (gridDropZones)", "subtotalCaption labels a field's
    // subtotal lines".
    check_all("t4129-pivot-layout-flags", &["GZ", "SC"]);
    check_all("t4745-pivot-classic-subtotal-caption", T4745_SHEETS);
    check_all(
        "t4698-pivot-repeat-labels",
        &[
            "T3R", "T3P", "T3A", "TND", "TBR", "TSD", "TIN", "TW", "TWE", "OST", "OB", "O3", "OW",
            "CF", "MX", "CC", "TBL",
        ],
    );
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
    check_all("t4634-pivot-total-order-4", T4634_SHEETS);
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
        value: PivotCell::Text(long),
        number_format: None,
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

#[test]
fn a_subtotal_captions_item_copies_are_bounded_before_they_are_made() {
    // Rows R (tabular, so its subtotals have lines of their own) and C, with `subtotalCaption`
    // on R: each `?` copies the item (t4745 SQ).
    let (cache, mut table) = crate::pivot::probe_cases::diagonal(3, true);
    table.row_fields.push(crate::pivot::AxisField::Field(1));
    table.column_fields.clear();
    table.fields[1].axis = Some(crate::pivot::Axis::Row);
    table.fields[0].outline = false;
    table.fields[0].subtotal_caption = Some("?-?".into());
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let cells = render(&table, &cache, &result, &layout).unwrap();
    let label = |text: &str| {
        cells
            .iter()
            .any(|c| c.value == PivotCell::Text(text.to_string()))
    };
    assert!(label("0-0") && label("2-2"));
    table.fields[0].subtotal_caption = Some("?".repeat(1_000));
    assert_eq!(
        render_within(&table, &cache, &result, &layout, 999),
        Err(PivotComputeError::Limit)
    );
}

/// probes/t4767-pivot-group-items GD and GDA: the manual group Product2 of Product (Group2 = A,
/// B; Group1 = C, D; E alone; F ungrouped) with the group item of E written `<d
/// v="2020-01-01T00:00:00"/>`. The sidecar cannot tell a saved `d` from an `s` (its
/// `group_items` read both as text), so the date item is the case's, as the serial Excel's A6
/// holds; its label cell is that number shown in m/d/yyyy, as Excel's A6 text reads.
#[test]
fn a_date_group_item_shows_as_a_date() {
    use crate::number_format::{format_number, get_num_fmt};
    use crate::pivot::{group_by_discrete, probe_cases::source_cache, AddedGroup};

    let case = read("t4767-pivot-group-items", "case");
    let oracle = read("t4767-pivot-group-items", "oracle");
    for sheet in ["GD", "GDA"] {
        let pivot = (oracle["pivots"].as_array().unwrap().iter())
            .find(|p| p["sheet"] == sheet)
            .unwrap();
        let serial = pivot["cells"]["A6"]["v"].as_f64().unwrap();
        let mut cache = source_cache(&case, "Groups!A1:B9");
        let labels = ["F", "Group2", "Group1"]
            .map(|t| CacheItem::Text(t.to_string()))
            .into_iter()
            .chain([CacheItem::Date(serial)])
            .collect();
        let discrete: Vec<u32> = (cache.fields[0].items.iter())
            .map(|item| match item {
                CacheItem::Text(t) if t == "A" || t == "B" => 1,
                CacheItem::Text(t) if t == "C" || t == "D" => 2,
                CacheItem::Text(t) if t == "E" => 3,
                _ => 0,
            })
            .collect();
        let mut added = group_by_discrete(&cache.fields[0], &discrete, labels).unwrap();
        added.name = "Product2".to_string();
        added.added_group = Some(AddedGroup {
            base: 0,
            discrete: Some(discrete),
        });
        cache.fields.push(added);
        let (cache, table) = probe_pivot_over(&case, sheet, cache);
        let mut work = MAX_PIVOT_WORK;
        let result = compute(&table, &cache, &mut work).unwrap();
        let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
        let cells = render(&table, &cache, &result, &layout).unwrap();
        let label = (cells.iter())
            .find(|c| c.column == 0 && c.value == PivotCell::Number(serial))
            .unwrap();
        let Some(CellFormat::Id(id)) = label.number_format else {
            panic!("{sheet}: a date group item's label has a numFmtId");
        };
        let id = i32::try_from(id).unwrap();
        let text = format_number(serial, &get_num_fmt(id, &[]), "en").text;
        assert_eq!(text, pivot["text"]["A6"].as_str().unwrap(), "{sheet}");
        // Every other label is text, and keeps its cell's format.
        assert!((cells.iter())
            .filter(|c| c.column == 0 && c != &label)
            .all(|c| matches!(c.value, PivotCell::Text(_)) && c.number_format.is_none()));
    }
}

/// An ungrouped field's date item takes a short-date format code (T-4807: m/d/yyyy whatever the
/// source cells' format), not a group item's `numFmtId`.
#[test]
fn an_ungrouped_date_item_takes_no_group_item_format() {
    let (mut cache, table) = crate::pivot::probe_cases::diagonal(3, true);
    cache.fields[0].items[0] = CacheItem::Date(43831.0);
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let cells = render(&table, &cache, &result, &layout).unwrap();
    let label = (cells.iter())
        .find(|c| c.value == PivotCell::Number(43831.0))
        .unwrap();
    assert_eq!(label.number_format, Some(CellFormat::Code("m/d/yyyy")));
}
