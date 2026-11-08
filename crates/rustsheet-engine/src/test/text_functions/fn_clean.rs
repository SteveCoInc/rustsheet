#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_clean_basic() {
    let mut model = new_empty_model();
    // Tab (char 9) should be removed
    model.set_a1("A1", "=CLEAN(\"hello\tworld\")");
    model.set_a1("A2", "=CLEAN(\"no control chars\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "helloworld");
    assert_eq!(model.get_text_a1("A2"), "no control chars");
}
