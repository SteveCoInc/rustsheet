#![allow(clippy::unwrap_used)]

use crate::{expressions::types::Area, test::util::new_empty_model};

#[test]
fn set_values() {
    let mut model = new_empty_model();
    // set a 2x2 array formula with value "Hello, world!" in A5:B6
    model
        .set_user_array_formula(0, 5, 1, 2, 2, "Hello, world!")
        .unwrap();
    model.evaluate();

    // check that the value is correctly set in the 4 cells
    assert_eq!(model.get_text_by_index(0, 5, 1), "Hello, world!");
    assert_eq!(model.get_text_by_index(0, 5, 2), "Hello, world!");
    assert_eq!(model.get_text_by_index(0, 6, 1), "Hello, world!");
    assert_eq!(model.get_text_by_index(0, 6, 2), "Hello, world!");

    // check there is no formula in A5
    assert!(!model.has_formula_a1("A5"));
    assert_eq!(model.get_text_a1("A5"), "Hello, world!");

    // check we can delete A6
    model.set_user_input(0, 6, 1, String::new()).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_by_index(0, 6, 1), "");
}

#[test]
fn set_simple_formula() {
    let mut model = new_empty_model();
    // set a 2x2 array formula with value "=1+1" in A5:B6
    model.set_user_array_formula(0, 5, 1, 2, 2, "=1+1").unwrap();
    assert_eq!(model.get_text_by_index(0, 5, 2), "");
    model.evaluate();

    // check that the value is correctly set in the 4 cells
    assert_eq!(model.get_text_by_index(0, 5, 1), "2");
    assert_eq!(model.get_text_by_index(0, 5, 2), "2");
    assert_eq!(model.get_text_by_index(0, 6, 1), "2");
    assert_eq!(model.get_text_by_index(0, 6, 2), "2");

    // check there is a formula in A5
    assert!(model.has_formula_a1("A5"));
    assert_eq!(model.get_formula_a1("A5"), "=1+1");

    // check we cannot delete A6
    assert!(model.set_user_input(0, 6, 1, String::new()).is_err());
    assert!(model.clear_contents_at(0, 6, 1).is_err());
    assert!(model.clear_all_at(0, 6, 1).is_err());

    // assert we cannot delete A5
    assert!(model.set_user_input(0, 5, 1, String::new()).is_err());

    model.evaluate();
    assert_eq!(model.get_text_by_index(0, 5, 1), "2");
    assert_eq!(model.get_text_by_index(0, 5, 2), "2");
    assert_eq!(model.get_text_by_index(0, 6, 1), "2");
    assert_eq!(model.get_text_by_index(0, 6, 2), "2");
}

#[test]
fn delete_full_array_formula() {
    let mut model = new_empty_model();
    // set a 2x2 array formula with value "=1+1" in A5:B6
    model.set_user_array_formula(0, 5, 1, 2, 2, "=1+1").unwrap();
    model.evaluate();

    // clear the whole formula using range clear
    let area = Area {
        sheet: 0,
        row: 5,
        column: 1,
        width: 2,
        height: 2,
    };
    model.range_clear_contents(&area).unwrap();
    model.evaluate();

    // check that all cells are cleared
    assert_eq!(model.get_text_by_index(0, 5, 1), "");
    assert_eq!(model.get_text_by_index(0, 5, 2), "");
    assert_eq!(model.get_text_by_index(0, 6, 1), "");
    assert_eq!(model.get_text_by_index(0, 6, 2), "");
}
