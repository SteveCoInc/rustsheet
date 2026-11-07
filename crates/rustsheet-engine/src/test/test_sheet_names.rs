#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn sheet_number_name() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.set_a1("A1", "7");
    model.set_a1("A2", "=Sheet2!C3");
    model.evaluate();
    model.rename_sheet("Sheet2", "2024").unwrap();
    model.evaluate();
    assert_eq!(model.workbook.get_worksheet_names(), ["Sheet1", "2024"]);
    assert_eq!(model.get_text_a1("A2"), "0");
}

// A workbook with no view for the model's `view_id` (one decoded from bytes may have none) still
// takes the active sheet: the view is created rather than the choice dropped (T-4554).
#[test]
fn set_active_sheet_creates_a_missing_view() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.workbook.views.clear();
    model.set_active_sheet(1).unwrap();
    assert_eq!(model.active_sheet(), 1);
    assert!(model.set_active_sheet(5).is_err());
    assert_eq!(model.active_sheet(), 1);
}
