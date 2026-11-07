#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// `Data!Local` reads the name scoped to Data from another sheet, and keeps reading it after Data
// is renamed (probes/sheet-ops-rename-sheet-scoped-name, excel-graph, T-1608, T-1693).
fn model_with_scoped_name() -> crate::Model {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet1", "Calc").unwrap();
    model.rename_sheet("Sheet2", "Data").unwrap();
    model
        .define_name("Local", Some(1), "=Data!$A$1:$A$2")
        .unwrap();
    model.set_a1("Data!A1", "1");
    model.set_a1("Data!A2", "2");
    model.set_a1("Calc!A1", "=Data!Local");
    model.set_a1("Calc!A2", "=SUM(Data!Local)");
    model.evaluate();
    model
}

#[test]
fn qualified_sheet_scoped_name_evaluates() {
    let model = model_with_scoped_name();
    assert_eq!(model.get_text_a1("Calc!A1"), "#SPILL!");
    assert_eq!(model.get_text_a1("Calc!A2"), "3");
}

#[test]
fn qualified_sheet_scoped_name_follows_a_sheet_rename() {
    let mut model = model_with_scoped_name();
    model.rename_sheet("Data", "Inputs").unwrap();
    model.evaluate();
    assert_eq!(model.get_formula_a1("Calc!A2"), "=SUM(Inputs!Local)");
    assert_eq!(model.get_text_a1("Calc!A1"), "#SPILL!");
    assert_eq!(model.get_text_a1("Calc!A2"), "3");
}
