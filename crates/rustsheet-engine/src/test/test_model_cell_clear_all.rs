#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;

#[test]
fn test_cell_clear_all_non_existing_sheet() {
    let mut model = new_empty_model();
    assert_eq!(
        model.clear_all_at(13, 1, 1),
        Err("Invalid sheet index".to_string())
    );
}

#[test]
fn test_cell_clear_all_unset_cell() {
    let mut model = new_empty_model();
    assert!(model.clear_all_at(0, 1, 1).is_ok());
}

#[test]
fn test_cell_clear_all_with_value() {
    let mut model = new_empty_model();
    model.set_a1("A1", "hello");
    model.evaluate();

    assert_eq!(model.get_text_by_index(0, 1, 1), "hello");
    assert_eq!(model.is_empty_cell(0, 1, 1), Ok(false));

    model.clear_all_at(0, 1, 1).unwrap();
    model.evaluate();

    assert_eq!(model.get_text_by_index(0, 1, 1), "");
    assert_eq!(model.is_empty_cell(0, 1, 1), Ok(true));
}

#[test]
fn test_cell_clear_all_referenced_elsewhere() {
    let mut model = new_empty_model();
    model.set_a1("A1", "35");
    model.set_a1("A2", "=2*A1");
    model.evaluate();

    assert_eq!(model.get_text_by_index(0, 1, 1), "35");
    assert_eq!(model.get_text_by_index(0, 2, 1), "70");
    assert_eq!(model.is_empty_cell(0, 1, 1), Ok(false));
    assert_eq!(model.is_empty_cell(0, 2, 1), Ok(false));

    model.clear_all_at(0, 1, 1).unwrap();
    model.evaluate();

    assert_eq!(model.get_text_by_index(0, 1, 1), "");
    assert_eq!(model.get_text_by_index(0, 2, 1), "0");
    assert_eq!(model.is_empty_cell(0, 1, 1), Ok(true));
    assert_eq!(model.is_empty_cell(0, 2, 1), Ok(false));
}
