#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;

#[test]
fn formulatext_english() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "=SUM(1, 2, 3)").unwrap();
    model.set_user_input(0, 1, 2, "=FORMULATEXT(A1)").unwrap();

    model.set_language("de").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("=SUM(1, 2, 3)".to_string())
    );
}

// T-4770: FORMULATEXT of a reference a row delete removed is `#REF!`
// (probes/table-edits/t4274-calc-column-delete-row-2 F!A1).
#[test]
fn formulatext_of_deleted_cell_is_ref_error() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 3, "=1+1").unwrap();
    model.set_user_input(0, 1, 1, "=FORMULATEXT(C2)").unwrap();
    model.delete_rows(0, 2, 1).unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("#REF!".to_string())
    );
}

#[test]
fn formulatext_of_cell_deleted_on_another_sheet_is_ref_error() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();
    model.set_user_input(0, 2, 3, "=1+1").unwrap();
    model
        .set_user_input(1, 1, 1, "=FORMULATEXT(Sheet1!C2)")
        .unwrap();
    model.delete_rows(0, 2, 1).unwrap();
    assert_eq!(
        model.get_formatted_cell_value(1, 1, 1),
        Ok("#REF!".to_string())
    );
}
