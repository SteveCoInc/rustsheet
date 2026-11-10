#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_quote_prefix_formula() {
    let mut model = new_empty_model();
    model.set_a1("A1", "'= 1 + 3");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"= 1 + 3");
    assert!(!model.has_formula_a1("A1"));
}

#[test]
fn test_quote_prefix_number() {
    let mut model = new_empty_model();
    model.set_a1("A1", "'13");
    model.set_a1("A2", "=ISNUMBER(A1)");
    model.set_a1("A3", "=A1+1");
    model.set_a1("A4", "=ISNUMBER(A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"13");
    assert!(!model.has_formula_a1("A1"));

    assert_eq!(model.get_text_a1("A2"), *"FALSE");
    assert_eq!(model.get_text_a1("A3"), *"14");

    assert_eq!(model.get_text_a1("A4"), *"TRUE");
}

#[test]
fn test_quote_prefix_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "'#N/A");
    model.set_a1("A2", "=ISERROR(A1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#N/A");

    assert_eq!(model.get_text_a1("A2"), *"FALSE");
}

#[test]
fn test_quote_prefix_boolean() {
    let mut model = new_empty_model();
    model.set_a1("A1", "'FALSE");
    model.set_a1("A2", "=ISTEXT(A1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"FALSE");

    assert_eq!(model.get_text_a1("A2"), *"TRUE");
}

#[test]
fn test_quote_prefix_enter() {
    let mut model = new_empty_model();
    model.set_a1("A1", "'123");
    model.set_a1("A2", "=ISTEXT(A1)");
    model.evaluate();
    // We introduce a value with a "quote prefix" index
    model.set_user_input(0, 1, 3, "'=A1".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"=A1");

    // But if we enter with a quote_prefix but without the "'" it won't be quote_prefix
    model.set_user_input(0, 1, 4, "=A1".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), *"123");
}

#[test]
fn test_quote_prefix_reenter() {
    let mut model = new_empty_model();
    model.set_a1("A1", "'123");
    model.set_a1("A2", "=ISTEXT(A1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    // We introduce a value with a "quote prefix" index
    model.set_user_input(0, 1, 1, "123".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"FALSE");
}

#[test]
fn test_update_cell_quote() {
    let mut model = new_empty_model();
    model.update_cell_with_text(0, 1, 1, "= 1 + 3").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"= 1 + 3");
    assert!(!model.has_formula_a1("A1"));
}

#[test]
fn test_update_quote_prefix_reenter() {
    let mut model = new_empty_model();
    model.update_cell_with_text(0, 1, 1, "123").unwrap();
    model.set_a1("A2", "=ISTEXT(A1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    // We reenter as a number
    model.update_cell_with_number(0, 1, 1, 123.0).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"FALSE");
}

#[test]
fn test_update_quote_prefix_reenter_bool() {
    let mut model = new_empty_model();
    model.update_cell_with_text(0, 1, 1, "TRUE").unwrap();
    model.set_a1("A2", "=ISTEXT(A1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    // We enter a bool
    model.update_cell_with_bool(0, 1, 1, true).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"FALSE");
}

#[test]
fn test_update_quote_prefix_reenter_text() {
    let mut model = new_empty_model();
    model.update_cell_with_text(0, 1, 1, "123").unwrap();
    model.set_a1("A2", "=ISTEXT(A1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().quote_prefix);
    // We enter a string
    model.update_cell_with_text(0, 1, 1, "Hello").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    assert!(!model.get_style_for_cell(0, 1, 1).unwrap().quote_prefix);
}

#[test]
fn test_update_quote_prefix_reenter_text_2() {
    let mut model = new_empty_model();
    model.update_cell_with_text(0, 1, 1, "123").unwrap();
    model.set_a1("A2", "=ISTEXT(A1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().quote_prefix);
    // We enter another number
    model.update_cell_with_text(0, 1, 1, "42").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().quote_prefix);
}

#[test]
fn normal_string_with_quote_prefix() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "'Hello".to_string()).unwrap();
    model.evaluate();
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().quote_prefix);
    assert_eq!(model.get_text_a1("A1"), *"Hello");
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        *"'Hello"
    );
}

#[test]
fn dates_with_quote_prefix() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "'2024-01-01".to_string())
        .unwrap();
    model.evaluate();
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().quote_prefix);
    assert_eq!(model.get_text_a1("A1"), *"2024-01-01");
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        *"'2024-01-01"
    );
}

// `update_cell_with_text` quotes whatever typed input would read as something other than text,
// by the typed-input reader itself (T-4851): percentages, currencies, dates and times as well as
// plain numbers, booleans, errors and a leading formula sign.
#[test]
fn test_update_cell_with_text_quotes_typed_number_text() {
    let mut model = new_empty_model();
    let quoted = [
        "12", "50%", "$5", "1/2/2020", "10:30", "1e-309", "true", "False", "#REF!", "=A1", "-x",
    ];
    for text in quoted {
        assert!(model.text_needs_quote_prefix(text), "{text}");
    }
    // Typed `1e-310` stays text (probes/typed-number-tiny, T-1100), as `1€` does in en-US
    // (probes/typed-currency-percent-input B23:C26) and #BLOCKED! (SPEC-0003).
    for text in [
        "",
        "hello",
        "1e-310",
        "1e-400",
        "0e309",
        "1€",
        "#BLOCKED!",
        "1.5$",
    ] {
        assert!(!model.text_needs_quote_prefix(text), "{text}");
    }
    for (column, text) in ["50%", "1/2/2020", "hello"].into_iter().enumerate() {
        let column = i32::try_from(column).unwrap() + 1;
        model.update_cell_with_text(0, 1, column, text).unwrap();
        // Typed back in through its content, a quoted cell stays text.
        let content = model.get_localized_cell_content(0, 1, column).unwrap();
        model.set_user_input(0, 2, column, content).unwrap();
    }
    model.set_a1("A3", "=ISTEXT(A2)");
    model.set_a1("B3", "=ISTEXT(B2)");
    model.set_a1("C3", "=ISTEXT(C2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A3"), *"TRUE");
    assert_eq!(model.get_text_a1("B3"), *"TRUE");
    assert_eq!(model.get_text_a1("C3"), *"TRUE");
}
