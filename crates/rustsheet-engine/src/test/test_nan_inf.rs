// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::unreadable_literal
)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

//! "NaN", "inf" and "infinity" are words, and a number too large to hold,
//! like 1e400, is not a number either. None of them is ever read as a number,
//! and no cell ever holds a number that is not finite.

use crate::model::Model;
use crate::number_format::parse_finite_number;
use crate::test::util::new_empty_model;
use crate::types::CellType;

/// The texts that the parser of numbers of the language takes for numbers
const WORDS: [&str; 12] = [
    "NaN",
    "nan",
    "NAN",
    "inf",
    "Inf",
    "INF",
    "infinity",
    "Infinity",
    "+inf",
    "-inf",
    "+NaN",
    "-infinity",
];

fn row(model: &Model, row: i32, columns: i32) -> Vec<String> {
    (1..=columns)
        .map(|column| model.get_text_by_index(0, row, column))
        .collect()
}

/// Sets each formula in column A, from row 1, and returns what each one shows.
fn evaluate(formulas: &[&str]) -> Vec<String> {
    let mut model = new_empty_model();
    for (r, formula) in formulas.iter().enumerate() {
        model
            .set_user_input(0, r as i32 + 1, 1, formula.to_string())
            .unwrap();
    }
    model.evaluate();
    (1..=formulas.len() as i32)
        .map(|r| model.get_text_by_index(0, r, 1))
        .collect()
}

#[test]
fn reading_numbers() {
    for (text, number) in [
        ("12", 12.0),
        ("-3.5", -3.5),
        ("+7", 7.0),
        (".5", 0.5),
        ("5.", 5.0),
        ("1e5", 100000.0),
        ("1E-3", 0.001),
        ("9.99999999999999e307", 9.99999999999999e307),
        ("0", 0.0),
        // Too small to hold is zero, which is a number.
        // `"1e-400"+0` is #VALUE! (probes/text-number-words A9); this reader serves conditional formats,
        // whose behavior is not observable through the oracle (numbers.md, T-1100).
        ("1e-400", 0.0),
    ] {
        assert_eq!(parse_finite_number(text), Some(number), "{text}");
    }
    for text in WORDS {
        assert_eq!(parse_finite_number(text), None, "{text}");
    }
    for text in [
        "1e400", "-1e400", "1e309", "1e308", "-1e308", "", " ", "abc", "1e", "--1", "1,5",
    ] {
        assert_eq!(parse_finite_number(text), None, "{text}");
    }
}

#[test]
fn the_words_typed_in_a_cell_are_text() {
    for word in ["NaN", "nan", "inf", "Inf", "infinity", "Infinity"] {
        let mut model = new_empty_model();
        model.set_a1("A1", word);
        model.set_a1("B1", "=A1+1");
        model.set_a1("C1", "=ISNUMBER(A1)");
        model.set_a1("D1", "=ISTEXT(A1)");
        model.set_a1("E1", "=A1&\"!\"");
        model.evaluate();

        assert_eq!(model.get_cell_type(0, 1, 1), Ok(CellType::Text), "{word}");
        assert_eq!(
            row(&model, 1, 5),
            [word, "#VALUE!", "FALSE", "TRUE", &format!("{word}!")],
            "{word}"
        );
        // It is shown as it was typed when edited
        assert_eq!(model.get_localized_cell_content(0, 1, 1).unwrap(), word);
    }
}

#[test]
fn a_number_too_large_typed_in_a_cell_is_text() {
    for text in ["1e400", "1E400", "1e309", "1e400%", "$1e400"] {
        let mut model = new_empty_model();
        model.set_a1("A1", text);
        model.set_a1("B1", "=A1+1");
        model.set_a1("C1", "=ISNUMBER(A1)");
        model.evaluate();

        assert_eq!(model.get_cell_type(0, 1, 1), Ok(CellType::Text), "{text}");
        assert_eq!(row(&model, 1, 3), [text, "#VALUE!", "FALSE"], "{text}");
        assert_eq!(model.get_localized_cell_content(0, 1, 1).unwrap(), text);
    }
    // The largest typeable number is still a number; from 1E+308 on, Excel reads number text as
    // text (probes/text-number-words A7, H2; docs/excel-semantics/numbers.md)
    let mut model = new_empty_model();
    model.set_a1("A1", "9.99999999999999e307");
    model.set_a1("B1", "=ISNUMBER(A1)");
    model.set_a1("A2", "1e308");
    model.set_a1("B2", "=ISNUMBER(A2)");
    model.evaluate();
    assert_eq!(model.get_cell_type(0, 1, 1), Ok(CellType::Number));
    assert_eq!(model.get_text_a1("B1"), "TRUE");
    assert_eq!(model.get_cell_type(0, 2, 1), Ok(CellType::Text));
    assert_eq!(model.get_text_a1("B2"), "FALSE");
}

#[test]
fn a_text_cell_with_the_words_needs_no_quote() {
    let mut model = new_empty_model();
    model.update_cell_with_text(0, 1, 1, "inf").unwrap();
    model.update_cell_with_text(0, 2, 1, "NaN").unwrap();
    model.update_cell_with_text(0, 3, 1, "1e400").unwrap();
    // A text that reads as a number does: without the quote it would be one
    model.update_cell_with_text(0, 4, 1, "12").unwrap();
    model.update_cell_with_text(0, 5, 1, "1e5").unwrap();

    let content = |r: i32| model.get_localized_cell_content(0, r, 1).unwrap();
    assert_eq!(content(1), "inf");
    assert_eq!(content(2), "NaN");
    assert_eq!(content(3), "1e400");
    assert_eq!(content(4), "'12");
    assert_eq!(content(5), "'1e5");
}

#[test]
fn the_words_are_not_numbers_for_the_operators() {
    for word in WORDS.iter().chain(&["1e400", "-1e400"]) {
        let results = evaluate(&[
            &format!("=\"{word}\"+1"),
            &format!("=1-\"{word}\""),
            &format!("=\"{word}\"*2"),
            &format!("=2/\"{word}\""),
            &format!("=\"{word}\"^2"),
            &format!("=-\"{word}\""),
            &format!("=\"{word}\"%"),
        ]);
        for result in results {
            assert_eq!(result, "#VALUE!", "{word}");
        }
    }
    // They are still texts for everything else
    assert_eq!(
        evaluate(&[
            "=\"inf\"=\"INF\"",
            "=\"inf\"&\"inity\"",
            "=\"nan\">1",
            "=LEN(\"NaN\")"
        ]),
        ["TRUE", "infinity", "TRUE", "3"]
    );
}

#[test]
fn the_words_inside_arrays() {
    let mut model = new_empty_model();
    model.set_a1("A1", "={\"NaN\",1,\"inf\",\"2\"}+1");
    model.set_a1("A2", "={\"1e400\",3}*{2,\"-inf\"}");
    model.set_a1("A3", "=SUM({\"NaN\",1,\"inf\"})");
    model.evaluate();

    assert_eq!(row(&model, 1, 5), ["#VALUE!", "2", "#VALUE!", "3", ""]);
    assert_eq!(row(&model, 2, 3), ["#VALUE!", "#VALUE!", ""]);
    // Texts in an array are not counted
    assert_eq!(model.get_text_a1("A3"), "1");
}

#[test]
fn the_words_are_not_numbers_for_the_functions() {
    for word in ["NaN", "inf", "-inf", "infinity", "1e400"] {
        let results = evaluate(&[
            &format!("=SUM(\"{word}\", 1)"),
            &format!("=PRODUCT(\"{word}\", 2)"),
            &format!("=AVERAGE(\"{word}\", 1)"),
            &format!("=AVERAGEA(\"{word}\", 1)"),
            &format!("=SIN(\"{word}\")"),
            &format!("=ABS(\"{word}\")"),
            &format!("=ROUND(\"{word}\", 1)"),
            &format!("=INT(\"{word}\")"),
            &format!("=MOD(\"{word}\", 2)"),
            &format!("=POWER(\"{word}\", 2)"),
            &format!("=DATE(\"{word}\", 1, 1)"),
            &format!("=DATE(2020, 1, \"{word}\")"),
            &format!("=VALUE(\"{word}\")"),
            &format!("=NUMBERVALUE(\"{word}\")"),
            &format!("=REPT(\"a\", \"{word}\")"),
            &format!("=LEFT(\"abc\", \"{word}\")"),
        ]);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result, "#VALUE!", "{word}, formula {i}");
        }
    }
}

#[test]
fn the_words_are_not_counted_as_numbers() {
    assert_eq!(
        evaluate(&[
            "=COUNT(\"NaN\", 1)",
            "=COUNT(\"inf\", \"1e400\", \"2\", 1)",
            "=COUNTA(\"NaN\", 1)",
            "=N(\"NaN\")",
            "=ISNUMBER(\"inf\"*1)",
            "=TEXT(\"inf\", \"0\")",
            "=T(\"NaN\")",
        ]),
        ["1", "2", "2", "0", "FALSE", "inf", "NaN"]
    );
}

#[test]
fn criteria_with_the_words_are_texts() {
    let mut model = new_empty_model();
    model.update_cell_with_text(0, 1, 1, "inf").unwrap();
    model.update_cell_with_text(0, 2, 1, "nan").unwrap();
    model.update_cell_with_text(0, 3, 1, "abc").unwrap();
    model.update_cell_with_text(0, 4, 1, "Infinity").unwrap();
    model.set_a1("A5", "5");
    model.set_a1("B1", "10");
    model.set_a1("B2", "20");
    model.set_a1("B3", "30");
    model.set_a1("B4", "40");
    model.set_a1("B5", "50");
    model.set_a1("D1", "=COUNTIF(A1:A5, \"inf\")");
    model.set_a1("D2", "=COUNTIF(A1:A5, \"NaN\")");
    model.set_a1("D3", "=COUNTIF(A1:A5, \"=infinity\")");
    model.set_a1("D4", "=SUMIF(A1:A5, \"nan\", B1:B5)");
    model.set_a1("D5", "=SUMIFS(B1:B5, A1:A5, \"inf\")");
    model.set_a1("D6", "=AVERAGEIF(A1:A5, \"inf\", B1:B5)");
    model.set_a1("D7", "=COUNTIF(A1:A5, \"inf*\")");
    // Numbers are still numbers
    model.set_a1("D8", "=COUNTIF(A1:A5, \"<10\")");
    model.set_a1("D9", "=COUNTIF(A1:A5, \"5\")");
    // A number too large to hold is a text too: no number is below it
    model.set_a1("D10", "=COUNTIF(A1:A5, \"<1e400\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("D2"), "1");
    assert_eq!(model.get_text_a1("D3"), "1");
    assert_eq!(model.get_text_a1("D4"), "20");
    assert_eq!(model.get_text_a1("D5"), "10");
    assert_eq!(model.get_text_a1("D6"), "10");
    assert_eq!(model.get_text_a1("D7"), "2");
    assert_eq!(model.get_text_a1("D8"), "1");
    assert_eq!(model.get_text_a1("D9"), "1");
    assert_eq!(model.get_text_a1("D10"), "0");
}

#[test]
fn a_number_too_large_in_a_formula_is_an_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=1e400");
    model.set_a1("A2", "=SUM(1e400, 1)");
    // Excel does not load a formula with a literal of 1E+308 or more
    // (docs/excel-semantics/numbers.md); it is no number here either
    model.set_a1("A3", "=1e308");
    // Below 1E-309 Excel does not load the formula either; from 1E-309 up the literal is 0
    // (numbers.md, probes/literal-number-tiny)
    model.set_a1("A4", "=1e-400");
    model.set_a1("A5", "=1e-309");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
    assert_eq!(model.get_text_a1("A2"), "#ERROR!");
    assert_eq!(model.get_text_a1("A3"), "#ERROR!");
    assert_eq!(model.get_text_a1("A4"), "#ERROR!");
    assert_eq!(model.get_text_a1("A5"), "0");
    // The formula is kept as it was written, not as an infinity
    assert_eq!(model.get_formula_a1("A1"), "=1e400");
    assert_eq!(model.get_formula_a1("A2"), "=SUM(1e400, 1)");
}

#[test]
fn a_calculation_that_overflows_is_an_error() {
    let mut model = new_empty_model();
    // In a single cell
    model.set_a1("A1", "=EXP(1000)");
    model.set_a1("A2", "=1e307*100");
    model.set_a1("A3", "=-1e307*100");
    // And in every element of an array
    model.set_a1("A5", "=EXP({1000,1})");
    model.set_a1("A6", "={1e307,1}*100");
    model.set_a1("A7", "=EXP(SEQUENCE(1,3,708,1))");
    model.set_a1("A8", "={-1,-1e307}*100");
    model.set_a1("A9", "=SUM(EXP({1000,1}))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "#NUM!");
    assert_eq!(model.get_text_a1("A2"), "#NUM!");
    assert_eq!(model.get_text_a1("A3"), "#NUM!");
    assert_eq!(row(&model, 5, 3), ["#NUM!", "2.718281828", ""]);
    assert_eq!(row(&model, 6, 3), ["#NUM!", "100", ""]);
    assert_eq!(row(&model, 7, 4)[2..], ["#NUM!", ""]);
    assert_eq!(row(&model, 8, 3), ["-100", "#NUM!", ""]);
    assert_eq!(model.get_text_a1("A9"), "#NUM!");
    // No cell holds a number that is not finite
    for r in 1..=9 {
        for c in 1..=3 {
            if let Ok(crate::cell::CellValue::Number(v)) = model.get_cell_value_by_index(0, r, c) {
                assert!(v.is_finite(), "row {r} column {c}");
            }
        }
    }
}

#[test]
fn a_number_that_is_not_finite_cannot_be_set_in_a_cell() {
    let mut model = new_empty_model();
    model.set_a1("A1", "7");
    model.set_a1("B1", "=A1+1");
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            model.update_cell_with_number(0, 1, 1, value).is_err(),
            "{value}"
        );
        // Also in an empty cell
        assert!(
            model.update_cell_with_number(0, 5, 5, value).is_err(),
            "{value}"
        );
        // And straight in the worksheet
        assert!(
            model
                .workbook
                .worksheet_mut(0)
                .unwrap()
                .set_cell_with_number(1, 1, value, 0)
                .is_err(),
            "{value}"
        );
    }
    model.evaluate();

    // The cells are as they were
    assert_eq!(model.get_text_a1("A1"), "7");
    assert_eq!(model.get_text_a1("B1"), "8");
    assert_eq!(model.is_empty_cell(0, 5, 5), Ok(true));

    // Any other number can
    for value in [0.0, -0.0, 1e308, -1e308, f64::MIN_POSITIVE, f64::MAX] {
        assert_eq!(model.update_cell_with_number(0, 1, 1, value), Ok(()));
    }
    model.update_cell_with_number(0, 1, 1, 41.0).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "42");
}

#[test]
fn a_number_too_large_in_a_number_format_is_an_error() {
    assert_eq!(
        evaluate(&[
            "=TEXT(5, \"[>1e400]0.0;0\")",
            "=TEXT(5, \"[<-1e400]0.0;0\")",
            "=TEXT(5, \"[>1e308]0.0;0\")",
            "=TEXT(5, \"[>]0.0;0\")",
        ]),
        // probes/text-number-words J1, L4, J2
        ["#VALUE!", "#VALUE!", "#VALUE!", "5.0"]
    );
}
