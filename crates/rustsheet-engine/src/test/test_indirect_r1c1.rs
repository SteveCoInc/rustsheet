#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn indirect_reads_r1c1_references() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("C2", "20");
    model.set_a1("D5", "=INDIRECT(\"R2C3\",FALSE)");
    model.set_a1("D6", "=INDIRECT(\"R[-4]C[-1]\",FALSE)");
    model.set_a1("D7", "=SUM(INDIRECT(\"R1C1:R2C1\",FALSE))");
    model.evaluate();

    // an absolute R1C1 reference
    assert_eq!(model.get_text_a1("D5"), "20");
    // a relative R1C1 reference, relative to the cell it is written in
    assert_eq!(model.get_text_a1("D6"), "20");
    // an R1C1 range
    assert_eq!(model.get_text_a1("D7"), "3");
}
