#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn issue_155() {
    let mut model = new_empty_model();
    model.set_a1("A1", "123");
    model.set_a1("D2", "=-(A1^1.22)");
    model.evaluate();

    assert_eq!(model.get_formula_a1("D2"), "=-(A1^1.22)".to_string());
}
