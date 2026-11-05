#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A sheet-scoped name added through the API after the formulas exist (T-4049,
// probes/t4049-sheet-api-name-plain and -over-book, excel-graph 2026-11-05; syntax.md "A
// sheet-scoped name added after load").
fn model_with_sheets() -> crate::Model {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet1", "S").unwrap();
    model.rename_sheet("Sheet2", "T").unwrap();
    model
}

#[test]
fn an_unresolved_name_on_its_sheet_stays_unresolved() {
    let mut model = model_with_sheets();
    model._set("S!A1", "=N");
    model._set("S!A2", "=ISERROR(N)");
    model._set("S!A3", "=S!N");
    model._set("T!A1", "=S!N");
    model._set("T!A2", "=N");
    model.define_name("N", Some(0), "=1").unwrap();
    model.evaluate();
    assert_eq!(model._get_text("S!A1"), "#NAME?");
    assert_eq!(model._get_formula("S!A1"), "=N");
    assert_eq!(model._get_text("S!A2"), "TRUE");
    assert_eq!(model._get_text("S!A3"), "1");
    assert_eq!(model._get_formula("S!A3"), "=S!N");
    assert_eq!(model._get_text("T!A1"), "1");
    assert_eq!(model._get_text("T!A2"), "#NAME?");

    // The binding is stored: another add re-parses every formula and S!A1 stays unresolved, while
    // `=N` entered on S now reads the sheet-scoped name.
    model.define_name("Other", None, "=2").unwrap();
    model._set("S!B1", "=N");
    model.evaluate();
    assert_eq!(model._get_text("S!A1"), "#NAME?");
    assert_eq!(model._get_formula("S!A1"), "=N");
    assert_eq!(model._get_text("S!B1"), "1");
}

#[test]
fn a_workbook_name_read_on_the_sheet_moves_to_the_sheet_name() {
    let mut model = model_with_sheets();
    model.define_name("N", None, "=10").unwrap();
    model._set("S!A1", "=N");
    model._set("S!A3", "=S!N");
    model._set("T!A1", "=N");
    model._set("T!A2", "=S!N");
    model.evaluate();
    assert_eq!(model._get_text("S!A3"), "10");
    model.define_name("N", Some(0), "=1").unwrap();
    model.evaluate();
    assert_eq!(model._get_text("S!A1"), "1");
    assert_eq!(model._get_formula("S!A1"), "=N");
    assert_eq!(model._get_text("S!A3"), "1");
    assert_eq!(model._get_formula("S!A3"), "=S!N");
    assert_eq!(model._get_text("T!A1"), "10");
    assert_eq!(model._get_text("T!A2"), "10");
}
