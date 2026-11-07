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
    model.set_a1("S!A1", "=N");
    model.set_a1("S!A2", "=ISERROR(N)");
    model.set_a1("S!A3", "=S!N");
    model.set_a1("T!A1", "=S!N");
    model.set_a1("T!A2", "=N");
    model.define_name("N", Some(0), "=1").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("S!A1"), "#NAME?");
    assert_eq!(model.get_formula_a1("S!A1"), "=N");
    assert_eq!(model.get_text_a1("S!A2"), "TRUE");
    assert_eq!(model.get_text_a1("S!A3"), "1");
    assert_eq!(model.get_formula_a1("S!A3"), "=S!N");
    assert_eq!(model.get_text_a1("T!A1"), "1");
    assert_eq!(model.get_text_a1("T!A2"), "#NAME?");

    // The binding is stored: another add re-parses every formula and S!A1 stays unresolved, while
    // `=N` entered on S now reads the sheet-scoped name (ours, binding by its text; unprobed, the
    // harness cannot type after an API add: T-4348).
    model.define_name("Other", None, "=2").unwrap();
    model.set_a1("S!B1", "=N");
    model.evaluate();
    assert_eq!(model.get_text_a1("S!A1"), "#NAME?");
    assert_eq!(model.get_formula_a1("S!A1"), "=N");
    assert_eq!(model.get_text_a1("S!B1"), "1");
}

#[test]
fn a_workbook_name_read_on_the_sheet_moves_to_the_sheet_name() {
    let mut model = model_with_sheets();
    model.define_name("N", None, "=10").unwrap();
    model.set_a1("S!A1", "=N");
    model.set_a1("S!A3", "=S!N");
    model.set_a1("T!A1", "=N");
    model.set_a1("T!A2", "=S!N");
    model.evaluate();
    assert_eq!(model.get_text_a1("S!A3"), "10");
    model.define_name("N", Some(0), "=1").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("S!A1"), "1");
    assert_eq!(model.get_formula_a1("S!A1"), "=N");
    assert_eq!(model.get_text_a1("S!A3"), "1");
    assert_eq!(model.get_formula_a1("S!A3"), "=S!N");
    assert_eq!(model.get_text_a1("T!A1"), "10");
    assert_eq!(model.get_text_a1("T!A2"), "10");
}

// Which place an unqualified use took at load decides whether the add fills it (T-4618,
// probes/t4618-*, excel-graph 2026-11-06; name_load_order.rs).
#[test]
fn an_earlier_loaded_qualified_use_makes_the_sheets_uses_read_the_added_name() {
    for (active, expected) in [(0, "1"), (1, "#NAME?")] {
        let mut model = model_with_sheets();
        // T is the second tab here: S's =S!N loads first only while S is active.
        model.set_a1("S!A1", "=T!N");
        model.set_a1("T!A1", "=N");
        model.set_a1("T!A2", "=N+1");
        model.set_active_sheet(active).unwrap();
        model.define_name("N", Some(1), "=1").unwrap();
        model.evaluate();
        assert_eq!(model.get_text_a1("T!A1"), expected, "active sheet {active}");
        assert_eq!(model.get_formula_a1("T!A1"), "=N");
        assert_eq!(model.get_text_a1("S!A1"), "1");
    }
}

#[test]
fn an_unqualified_use_no_qualified_use_fixes_reads_the_added_name() {
    // probes/t4394-api-name-active-s2 S2!A3 =SUM(Span): no `S!Span` anywhere.
    let mut model = model_with_sheets();
    model.set_a1("S!A1", "=N");
    model.set_a1("T!A1", "=N");
    model.define_name("N", Some(0), "=1").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("S!A1"), "1");
    assert_eq!(model.get_text_a1("T!A1"), "#NAME?");
}

#[test]
fn a_formula_s_names_are_read_in_written_order() {
    // probes/t4618-same-formula-qualified-first: `S!N` before `N` in one formula.
    let mut model = model_with_sheets();
    model.set_a1("S!A1", "=S!N+N*10");
    model.set_a1("S!A2", "=N");
    model.define_name("N", Some(0), "=1").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("S!A1"), "11");
    assert_eq!(model.get_text_a1("S!A2"), "1");
    // Written the other way round the bare use comes first, and the qualified one fixes it.
    let mut model = model_with_sheets();
    model.set_a1("S!A1", "=N*10+S!N");
    model.define_name("N", Some(0), "=1").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("S!A1"), "#NAME?");
}
