#![allow(clippy::unwrap_used)]

// XLOOKUP under a lifted lookup_value with a range if_not_found (SPEC-0014 amendment "Mixed
// grids", T-4570): its hits are values and each miss is the range, a reference element.
// Evidence: probes/t4563-xlookup-not-found-ref-element, probes/t4569-xlookup-ref-element-max-min.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A1:A3 = 1, 2, 3; B1:B3 = 10, 20, 30; D1:D2 = x, y; F1:F2 = 3, 4; G1:G2 = 50, 60.
fn data() -> Model {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "2"),
        ("A3", "3"),
        ("B1", "10"),
        ("B2", "20"),
        ("B3", "30"),
        ("D1", "x"),
        ("D2", "y"),
        ("F1", "3"),
        ("F2", "4"),
        ("G1", "50"),
        ("G2", "60"),
    ] {
        model._set(cell, value);
    }
    model
}

const X_F: &str = "XLOOKUP({2;9},$A$1:$A$3,$B$1:$B$3,$F$1:$F$2)";
const X_G: &str = "XLOOKUP({2;9},$A$1:$A$3,$B$1:$B$3,$G$1:$G$2)";

#[test]
fn readers_see_the_hit_as_a_value_and_the_miss_as_the_range() {
    let mut model = data();
    let cases = [
        // INDEX picks the reference element itself, or the value (t4563 A1, A13, A29, A45).
        (format!("=ISREF(INDEX({X_F},2,1))"), "TRUE"),
        (format!("=ROWS(INDEX({X_F},2,1))"), "2"),
        (format!("=INDEX({X_F},1,1)"), "20"),
        (format!("=AREAS(INDEX({X_F},2,1))"), "1"),
        // The value view: 20 and #VALUE! (t4563 A57, A89).
        (format!("=INDEX({X_F}+0,2,1)"), "#VALUE!"),
        (format!("=INDEX({X_F}+0,1,1)"), "20"),
        (format!("=ISREF({X_F})"), "FALSE"),
        // Top-left aggregates read F1 for the miss; COUNT and COUNTA see a value and an error
        // item; SUMPRODUCT reads the reference as 0 (t4563 A73, A77, A81; t4569 A1, A21).
        (format!("=SUM({X_F})"), "23"),
        (format!("=MAX({X_G})"), "50"),
        (format!("=COUNT({X_F})"), "1"),
        (format!("=COUNTA({X_F})"), "2"),
        (format!("=SUMPRODUCT({X_G})"), "20"),
        // Per-element callers lift over both elements (t4569 C24:D34).
        (format!("=INDEX(ROW({X_G}),1,1)"), "#VALUE!"),
        (format!("=INDEX(ROW({X_G}),2,1)"), "1"),
        (format!("=INDEX(N({X_G}),1,1)"), "20"),
        (format!("=INDEX(N({X_G}),2,1)"), "50"),
        // One index into the column of elements is a one-element array (t4563 T25, W25).
        (format!("=ROWS(INDEX({X_F},2))"), "1"),
        (format!("=INDEX(INDEX({X_F},2),2,1)"), "#REF!"),
    ];
    for (row, (formula, _)) in (1..).zip(&cases) {
        model._set(&format!("K{row}"), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in (1..).zip(&cases) {
        assert_eq!(model._get_text(&format!("K{row}")), *expected, "{formula}");
    }
}

// OFFSET lifts over the elements of a mixed grid: a 2x1 result, `#VALUE!` for the hit (a value)
// and for the moved miss (a two-cell range read as a value), whatever the offset
// (probes/t4704-mixed-xlookup-grid-readers A4:C8, T-4739).
#[test]
fn offset_lifts_over_a_mixed_grid() {
    let mut model = data();
    model._set("K1", &format!("=OFFSET({X_G},0,0)"));
    model._set("L1", &format!("=ROWS(OFFSET({X_G},0,0))"));
    model._set("M1", &format!("=COLUMNS(OFFSET({X_G},0,0))"));
    model._set("K4", &format!("=OFFSET({X_G},1,0)"));
    model._set("L4", &format!("=ROWS(OFFSET({X_G},1,0))"));
    // A scalar XLOOKUP is still the reference it picks.
    model._set(
        "K7",
        "=OFFSET(XLOOKUP(2,$A$1:$A$3,$B$1:$B$3,$G$1:$G$2),1,0)",
    );
    model.evaluate();
    for (cell, expected) in [
        ("K1", "#VALUE!"),
        ("K2", "#VALUE!"),
        ("L1", "2"),
        ("M1", "1"),
        ("K4", "#VALUE!"),
        ("K5", "#VALUE!"),
        ("L4", "2"),
        ("K7", "30"),
    ] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
}

// The RANK family lifts over the elements of a mixed grid: the hit (a value) is #VALUE!, the miss
// ranks against its range G1:G2 = 50, 60 (probes/t4704-mixed-xlookup-grid-readers A25:C38,
// T-4740).
#[test]
fn rank_lifts_over_a_mixed_grid() {
    let mut model = data();
    model._set("K1", &format!("=RANK(60,{X_G})"));
    model._set("K4", &format!("=RANK.EQ(60,{X_G})"));
    model._set("K7", &format!("=RANK.AVG(60,{X_G})"));
    model._set("K10", &format!("=RANK(20,{X_G})"));
    model._set("K13", &format!("=RANK(50,{X_G})"));
    model._set("L1", &format!("=ROWS(RANK(60,{X_G}))"));
    model.evaluate();
    for (cell, expected) in [
        ("K1", "#VALUE!"),
        ("K2", "1"),
        ("K4", "#VALUE!"),
        ("K5", "1"),
        ("K7", "#VALUE!"),
        ("K8", "1"),
        ("K10", "#VALUE!"),
        ("K11", "#N/A"),
        ("K13", "#VALUE!"),
        ("K14", "2"),
        ("L1", "2"),
    ] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
}

// LARGE, SMALL and MEDIAN over references on another sheet are #N/A, on the formula's own sheet
// the value view's #VALUE! (t4569 A12, A13, A22, A31, A43, Local!E1:E3).
#[test]
fn order_statistics_are_na_only_for_references_on_another_sheet() {
    let mut model = data();
    model.add_sheet("Sheet2").unwrap();
    model._set("Sheet2!A1", "=LARGE(OFFSET(Sheet1!$G$1,{0;1},0),1)");
    model._set("Sheet2!A2", "=SMALL(OFFSET(Sheet1!$G$1,{0;1},0),1)");
    model._set("Sheet2!A3", "=MEDIAN(OFFSET(Sheet1!$G$1,{0;1},0))");
    model._set(
        "Sheet2!A4",
        "=LARGE(XLOOKUP({2;9},Sheet1!$A$1:$A$3,Sheet1!$B$1:$B$3,Sheet1!$G$1:$G$2),1)",
    );
    model._set("K1", "=LARGE(OFFSET($G$1,{0;1},0),1)");
    model._set("K2", "=MEDIAN(OFFSET($G$1,{0;1},0))");
    model._set("K3", &format!("=LARGE({X_G},1)"));
    model.evaluate();
    for cell in ["Sheet2!A1", "Sheet2!A2", "Sheet2!A3", "Sheet2!A4"] {
        assert_eq!(model._get_text(cell), *"#N/A", "{cell}");
    }
    for cell in ["K1", "K2", "K3"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
}

// Over references on both sheets the first element decides: own sheet first is #VALUE!, another
// sheet first #N/A (T-4714; t4697 Local!H1:H6, H9, H16, H17).
#[test]
fn order_statistics_over_mixed_sheets_follow_the_first_element() {
    let mut model = data();
    model.add_sheet("Sheet2").unwrap();
    let own_first = "{\"Sheet2!A1\";\"Sheet1!G1\"}";
    let other_first = "{\"Sheet1!G1\";\"Sheet2!A1\"}";
    model._set("Sheet2!B1", &format!("=LARGE(INDIRECT({own_first}),1)"));
    model._set("Sheet2!B2", &format!("=SMALL(INDIRECT({own_first}),1)"));
    model._set("Sheet2!B3", &format!("=MEDIAN(INDIRECT({own_first}))"));
    model._set("Sheet2!B4", &format!("=LARGE(INDIRECT({other_first}),1)"));
    model._set(
        "Sheet2!B5",
        "=LARGE(INDIRECT({\"Sheet2!A1\";\"Sheet1!G1\";\"Sheet1!G2\"}),1)",
    );
    model.evaluate();
    for cell in ["Sheet2!B1", "Sheet2!B2", "Sheet2!B3", "Sheet2!B5"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
    assert_eq!(model._get_text("Sheet2!B4"), *"#N/A");
}

// Over an error element beside an off-sheet reference the first element decides: the reference
// first is #N/A for LARGE, SMALL and MEDIAN, the error first is the error; MAX takes the error
// either way (T-4741; t4704 Calc!A46:A61, A88, A91).
#[test]
fn order_statistics_over_an_error_and_a_reference_follow_the_first_element() {
    let mut model = data();
    model._set("I1", "5");
    model._set("I2", "=1/0");
    model._set("I3", "7");
    model.add_sheet("Sheet2").unwrap();
    let grid =
        |keys: &str| format!("XLOOKUP({keys},Sheet1!$A$1:$A$3,Sheet1!$I$1:$I$3,Sheet1!$G$1:$G$2)");
    let (reference_first, error_first) = (grid("{9;2}"), grid("{2;9}"));
    for (row, grid) in [(1, &reference_first), (2, &error_first)] {
        model._set(&format!("Sheet2!A{row}"), &format!("=LARGE({grid},1)"));
        model._set(&format!("Sheet2!B{row}"), &format!("=SMALL({grid},1)"));
        model._set(&format!("Sheet2!C{row}"), &format!("=MEDIAN({grid})"));
        model._set(&format!("Sheet2!D{row}"), &format!("=MAX({grid})"));
    }
    model.evaluate();
    for cell in ["Sheet2!A1", "Sheet2!B1", "Sheet2!C1"] {
        assert_eq!(model._get_text(cell), *"#N/A", "{cell}");
    }
    for cell in [
        "Sheet2!D1",
        "Sheet2!A2",
        "Sheet2!B2",
        "Sheet2!C2",
        "Sheet2!D2",
    ] {
        assert_eq!(model._get_text(cell), *"#DIV/0!", "{cell}");
    }
}

// A lifted XLOOKUP in a per-element caller's range argument is a grid of its elements whatever
// its if_not_found: each value element is #VALUE!, an error element its error, a 1x1 lookup a 1x1
// result and a scalar lookup the reference it picks (T-4742; t4704 A64:C74, t4742 A10:C37).
#[test]
fn criteria_calls_lift_over_every_xlookup_element() {
    let mut model = data();
    model._set(
        "K1",
        "=COUNTIF(XLOOKUP({2;9},$A$1:$A$3,$B$1:$B$3,\"x\"),\">0\")",
    );
    model._set(
        "L1",
        "=ROWS(SUMIF(XLOOKUP({2;3},$A$1:$A$3,$B$1:$B$3,\"x\"),\">0\"))",
    );
    model._set("M1", "=COUNTIF(XLOOKUP({2;9},$A$1:$A$3,$B$1:$B$3),\">0\")");
    model._set(
        "N1",
        "=COLUMNS(COUNTIF(XLOOKUP({2},$A$1:$A$3,$B$1:$B$3,\"x\"),\">0\"))",
    );
    model._set(
        "O1",
        "=COUNTIF(XLOOKUP(2,$A$1:$A$3,$B$1:$B$3,\"x\"),\">0\")",
    );
    model._set(
        "P1",
        "=ROWS(SUMIF($A$1:$A$2,\">0\",XLOOKUP({2;3},$A$1:$A$3,$B$1:$B$3,\"x\")))",
    );
    model.evaluate();
    for (cell, expected) in [
        ("K1", "#VALUE!"),
        ("K2", "#VALUE!"),
        ("L1", "2"),
        ("M1", "#VALUE!"),
        ("M2", "#N/A"),
        ("N1", "1"),
        ("O1", "1"),
        ("P1", "2"),
    ] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
}

// A lift with no miss builds no grid: the hits are an ordinary array, so ISREF of an element is
// FALSE (SPEC-0014 amendment "Producer").
#[test]
fn an_all_hit_lift_is_an_array_of_values() {
    let mut model = data();
    model._set(
        "K1",
        "=ISREF(INDEX(XLOOKUP({2;3},$A$1:$A$3,$B$1:$B$3,$F$1:$F$2),1,1))",
    );
    model._set("K2", "=SUM(XLOOKUP({2;3},$A$1:$A$3,$B$1:$B$3,$F$1:$F$2))");
    model._set(
        "K3",
        "=COUNTIF(XLOOKUP({2;9},$A$1:$A$3,$B$1:$B$3,$G$1:$G$2),\">0\")",
    );
    model.evaluate();
    assert_eq!(model._get_text("K1"), *"FALSE");
    assert_eq!(model._get_text("K2"), *"50");
    assert_eq!(model._get_text("K3"), *"#VALUE!");
    assert_eq!(model._get_text("K4"), *"2");
}

// The mixed grid's element stays within the size the grids were built for (T-1956).
#[test]
fn a_ref_element_holding_a_value_stays_small() {
    assert!(std::mem::size_of::<crate::reference_operators::RefElement>() <= 32);
}
