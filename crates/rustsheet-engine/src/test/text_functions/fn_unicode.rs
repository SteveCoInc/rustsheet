#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=UNICODE(\"1,00\")");
    model.set_a1("A2", "=UNICODE(\"1\")");
    model.set_a1("A3", "=UNICODE(1)");
    model.set_a1("A4", "=UNICODE(\"T\")");
    model.set_a1("A5", "=UNICODE(\"TRUE\")");
    model.set_a1("A6", "=UNICODE(TRUE)");
    model.set_a1("A7", "=UNICODE(FALSE)");
    model.set_a1("A8", "=UNICODE(\"の\")");
    model.set_a1("A9", "=UNICODE(\" \")");

    model.set_a1("A10", "=_xlfn.UNICODE(\"T\")");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"49");
    assert_eq!(model.get_text_a1("A2"), *"49");
    assert_eq!(model.get_text_a1("A3"), *"49");
    assert_eq!(model.get_text_a1("A4"), *"84");
    assert_eq!(model.get_text_a1("A5"), *"84");
    assert_eq!(model.get_text_a1("A6"), *"84");
    assert_eq!(model.get_text_a1("A7"), *"70");
    assert_eq!(model.get_text_a1("A8"), *"12398");
    assert_eq!(model.get_text_a1("A9"), *"32");
    assert_eq!(model.get_text_a1("A10"), *"84");
}

#[test]
fn test_error_cases() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=UNICODE(\"\")");
    model.set_a1("A2", "=UNICODE(#CALC!)");
    model.set_a1("A3", "=UNICODE(#NAME?)");
    model.set_a1("A4", "=UNICODE(#VALUE!)");
    model.set_a1("A5", "=UNICODE(#REF!)");
    model.set_a1("A6", "=UNICODE(#DIV/0!)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#CALC!");
    assert_eq!(model.get_text_a1("A3"), *"#NAME?");
    assert_eq!(model.get_text_a1("A4"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A5"), *"#REF!");
    assert_eq!(model.get_text_a1("A6"), *"#DIV/0!");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=UNICODE()");
    model.set_a1("A2", "=UNICODE(\"B\",\"A\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
}

/// UNICODE lifts over an array or a range, as CODE does, so a lifted INDIRECT spills and a
/// blocked spill is `#SPILL!` (gen-m9/675763979c1defb6 Calc!A25; T-2090). The element values
/// are each element's scalar UNICODE (not oracle-probed).
#[test]
fn unicode_lifts_over_arrays_and_ranges() {
    let mut model = new_empty_model();
    model.set_a1("A1", "A");
    model.set_a1("A2", "b");
    model.set_a1("C1", "=ROWS(UNICODE(INDIRECT({\"A1\";\"A2\"})))");
    model.set_a1("D1", "=UNICODE(A1:A3)");
    model.set_a1("E1", "=UNICODE({\"A\",\"\"})");
    model.set_a1("G1", "=UNICODE(A1:A2)");
    model.set_a1("G2", "blocker");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"2");
    assert_eq!(model.get_text_a1("D1"), *"65");
    assert_eq!(model.get_text_a1("D2"), *"98");
    assert_eq!(model.get_text_a1("D3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("E1"), *"65");
    assert_eq!(model.get_text_a1("F1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("G1"), *"#SPILL!");
}
