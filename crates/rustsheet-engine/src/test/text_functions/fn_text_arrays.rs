// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

//! LEN, TRIM, PROPER, CLEAN, FIND, SEARCH, REPT, SUBSTITUTE and REPLACE work
//! element by element when any of their arguments is a range or an array.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A1:C1 holds three texts
fn model_with_texts() -> Model {
    let mut model = new_empty_model();
    model.set_a1("A1", "Hello World");
    model.set_a1("B1", "' ab  c ");
    model.set_a1("C1", "x-y-z");
    model
}

fn row(model: &Model, row: i32, columns: i32) -> Vec<String> {
    (1..=columns)
        .map(|column| model.get_text_by_index(0, row, column))
        .collect()
}

#[test]
fn len() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=LEN(A1:C1)");
    model.set_a1("A4", "=SUM(LEN(A1:C1))");
    model.set_a1("A5", "=LEN({\"a\",\"bb\";\"ccc\",\"\"})");
    model.set_a1("A8", "=LEN({1234,TRUE})");
    model.evaluate();

    assert_eq!(row(&model, 3, 4), ["11", "7", "5", ""]);
    assert_eq!(model.get_text_a1("A4"), *"23");
    assert_eq!(row(&model, 5, 3), ["1", "2", ""]);
    assert_eq!(row(&model, 6, 3), ["3", "0", ""]);
    assert_eq!(row(&model, 8, 3), ["4", "4", ""]);
}

#[test]
fn trim_proper_clean() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=TRIM(A1:C1)");
    model.set_a1("A4", "=PROPER({\"hello world\",\"FOO bar\"})");
    // A bell at the end of each text, and then without it
    model.set_a1("A5", "=LEN(A1:C1&CHAR(7))");
    model.set_a1("A6", "=LEN(CLEAN(A1:C1&CHAR(7)))");
    model.set_a1("A7", "=CLEAN(A1:C1&CHAR(7))=A1:C1");
    model.evaluate();

    assert_eq!(row(&model, 3, 4), ["Hello World", "ab c", "x-y-z", ""]);
    assert_eq!(row(&model, 4, 3), ["Hello World", "Foo Bar", ""]);
    assert_eq!(row(&model, 5, 4), ["12", "8", "6", ""]);
    assert_eq!(row(&model, 6, 4), ["11", "7", "5", ""]);
    assert_eq!(row(&model, 7, 4), ["TRUE", "TRUE", "TRUE", ""]);
}

#[test]
fn find() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=FIND(\"o\", A1:C1)");
    model.set_a1("A4", "=FIND({\"H\",\"W\",\"h\"}, \"Hello World\")");
    model.set_a1("A5", "=FIND(\"o\", \"Hello World\", {1,6,9})");
    // A column of texts to find against a row of texts
    model.set_a1("A6", "=FIND({\"l\";\"-\"}, A1:C1)");
    model.evaluate();

    assert_eq!(row(&model, 3, 4), ["5", "#VALUE!", "#VALUE!", ""]);
    // FIND is case sensitive
    assert_eq!(row(&model, 4, 4), ["1", "7", "#VALUE!", ""]);
    assert_eq!(row(&model, 5, 4), ["5", "8", "#VALUE!", ""]);
    assert_eq!(row(&model, 6, 4), ["3", "#VALUE!", "#VALUE!", ""]);
    assert_eq!(row(&model, 7, 4), ["#VALUE!", "#VALUE!", "2", ""]);
}

#[test]
fn search() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=SEARCH(\"O\", A1:C1)");
    // With wildcards
    model.set_a1("A4", "=SEARCH(\"?-\", A1:C1)");
    model.set_a1("A5", "=SEARCH({\"h\",\"w*d\"}, \"Hello World\")");
    model.set_a1("A6", "=SEARCH(\"o\", \"Hello World\", {1,6})");
    model.evaluate();

    assert_eq!(row(&model, 3, 4), ["5", "#VALUE!", "#VALUE!", ""]);
    assert_eq!(row(&model, 4, 4), ["#VALUE!", "#VALUE!", "1", ""]);
    assert_eq!(row(&model, 5, 3), ["1", "7", ""]);
    assert_eq!(row(&model, 6, 3), ["5", "8", ""]);
}

#[test]
fn rept() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=REPT({\"a\",\"b\"}, 2)");
    model.set_a1("A2", "=REPT(\"ab\", {0,1,2})");
    model.set_a1("A3", "=REPT({\"a\",\"b\"}, {1;2})");
    model.set_a1("A5", "=REPT(\"a\", {1,-1})");
    model.evaluate();

    assert_eq!(row(&model, 1, 3), ["aa", "bb", ""]);
    assert_eq!(row(&model, 2, 4), ["", "ab", "abab", ""]);
    assert_eq!(row(&model, 3, 3), ["a", "b", ""]);
    assert_eq!(row(&model, 4, 3), ["aa", "bb", ""]);
    assert_eq!(row(&model, 5, 3), ["a", "#VALUE!", ""]);
}

#[test]
fn substitute() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=SUBSTITUTE(A1:C1, \"-\", \"+\")");
    model.set_a1("A4", "=SUBSTITUTE(\"a-b-c\", \"-\", \"+\", {1,2,3})");
    model.set_a1("A5", "=SUBSTITUTE(\"abc\", {\"a\",\"b\"}, {\"X\",\"Y\"})");
    model.set_a1("A6", "=SUBSTITUTE(\"a-b-c\", \"-\", \"+\", {1,0})");
    model.evaluate();

    assert_eq!(row(&model, 3, 4), ["Hello World", " ab  c ", "x+y+z", ""]);
    assert_eq!(row(&model, 4, 4), ["a+b-c", "a-b+c", "a-b-c", ""]);
    assert_eq!(row(&model, 5, 3), ["Xbc", "aYc", ""]);
    assert_eq!(row(&model, 6, 3), ["a+b-c", "#VALUE!", ""]);
}

#[test]
fn replace() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=REPLACE(A1:C1, 1, 1, \"Z\")");
    model.set_a1("A4", "=REPLACE(\"abcdef\", {1,3,9}, 2, \"-\")");
    model.set_a1(
        "A5",
        "=REPLACE(\"abcdef\", 2, {0,1,9}, {\"X\",\"Y\",\"Z\"})",
    );
    model.set_a1("A6", "=REPLACE(\"abcdef\", {1,0}, {-1,1}, \"-\")");
    model.evaluate();

    assert_eq!(row(&model, 3, 4), ["Zello World", "Zab  c ", "Z-y-z", ""]);
    assert_eq!(row(&model, 4, 4), ["-cdef", "ab-ef", "abcdef-", ""]);
    assert_eq!(row(&model, 5, 4), ["aXbcdef", "aYcdef", "aZ", ""]);
    assert_eq!(row(&model, 6, 3), ["#VALUE!", "#VALUE!", ""]);
}

#[test]
fn errors_are_per_element() {
    let mut model = model_with_texts();
    model.set_a1("B1", "=1/0");
    model.set_a1("A3", "=LEN(A1:C1)");
    model.set_a1("A4", "=TRIM(A1:C1)");
    model.set_a1("A5", "=PROPER(A1:C1)");
    model.set_a1("A6", "=CLEAN(A1:C1)");
    model.set_a1("A7", "=FIND(\"-\", A1:C1)");
    model.set_a1("A8", "=SEARCH(\"-\", A1:C1)");
    model.set_a1("A9", "=REPT(A1:C1, 1)");
    model.set_a1("A10", "=SUBSTITUTE(A1:C1, \"-\", \"\")");
    model.set_a1("A11", "=REPLACE(A1:C1, 1, 1, \"\")");
    model.evaluate();

    for r in 3..=11 {
        assert_eq!(model.get_text_by_index(0, r, 2), "#DIV/0!", "row {r}");
        assert_ne!(model.get_text_by_index(0, r, 3), "#DIV/0!", "row {r}");
    }
    assert_eq!(row(&model, 3, 4), ["11", "#DIV/0!", "5", ""]);
    assert_eq!(row(&model, 7, 4), ["#VALUE!", "#DIV/0!", "2", ""]);
}

#[test]
fn arrays_of_different_sizes() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=FIND({\"a\",\"b\"}, {\"ab\",\"ab\",\"ab\"})");
    model.set_a1("A2", "=REPT({\"a\",\"b\",\"c\"}, {1,2})");
    model.evaluate();

    assert_eq!(row(&model, 1, 4), ["1", "2", "#N/A", ""]);
    assert_eq!(row(&model, 2, 4), ["a", "bb", "#N/A", ""]);
}

#[test]
fn single_values_still_work() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=LEN(A1)");
    model.set_a1("B3", "=LEN(1234)");
    model.set_a1("C3", "=LEN(D1)");
    model.set_a1("A4", "=TRIM(B1)");
    model.set_a1("B4", "=PROPER(\"it's a test\")");
    model.set_a1("A5", "=FIND(\"l\", A1, \"4\")");
    model.set_a1("B5", "=SEARCH(\"L\", A1, TRUE)");
    model.set_a1("A6", "=REPT(\"ab\", 2.9)");
    model.set_a1("B6", "=REPT(\"ab\", 2000000000)");
    model.set_a1("A7", "=SUBSTITUTE(C1, \"-\", \"+\", 2)");
    model.set_a1("B7", "=SUBSTITUTE(C1, \"\", \"+\")");
    model.set_a1("A8", "=REPLACE(C1, 2, 3, \"\")");
    // An error argument is the result
    model.set_a1("A9", "=LEN(1/0)");
    model.set_a1("B9", "=FIND(\"a\", NA())");
    // Implicit intersection
    model.set_a1("B10", "=LEN(@A1:C1)");
    model.evaluate();

    assert_eq!(row(&model, 3, 3), ["11", "4", "0"]);
    assert_eq!(row(&model, 4, 2), ["ab c", "It'S A Test"]);
    assert_eq!(row(&model, 5, 2), ["4", "3"]);
    assert_eq!(row(&model, 6, 2), ["abab", "#VALUE!"]);
    assert_eq!(row(&model, 7, 2), ["x-y+z", "x-y-z"]);
    assert_eq!(model.get_text_a1("A8"), *"xz");
    assert_eq!(row(&model, 9, 2), ["#DIV/0!", "#N/A"]);
    assert_eq!(model.get_text_a1("B10"), *"7");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LEN()");
    model.set_a1("A2", "=TRIM(\"a\", \"b\")");
    model.set_a1("A3", "=FIND(\"a\")");
    model.set_a1("A4", "=SEARCH(\"a\", \"a\", 1, 1)");
    model.set_a1("A5", "=SUBSTITUTE(\"a\", \"b\")");
    model.set_a1("A6", "=SUBSTITUTE(\"a\", \"b\", \"c\", 1, 1)");
    model.set_a1("A7", "=REPLACE(\"a\", 1, 1)");
    model.set_a1("A8", "=PROPER()");
    model.set_a1("A9", "=CLEAN()");
    model.evaluate();

    for r in 1..=9 {
        assert_eq!(model.get_text_by_index(0, r, 1), "#ERROR!", "row {r}");
    }
}

// Where one array is shorter than the other the missing elements are #N/A,
// and like any other error they wait their turn: the error of an earlier
// argument comes first.

/// A1:A9 holds texts, with an error in A6 and another one in A8
fn model_with_a_column() -> Model {
    let mut model = new_empty_model();
    for (r, value) in [
        "Hello",
        "Adios",
        "Vamos",
        "x-y-z",
        "abcde",
        "=1/0",
        "seven",
        "=SQRT(-1)",
        "nine9",
    ]
    .iter()
    .enumerate()
    {
        model
            .set_user_input(0, r as i32 + 1, 1, value.to_string())
            .unwrap();
    }
    model
}

fn column(model: &Model, column: i32, rows: i32) -> Vec<String> {
    (1..=rows)
        .map(|row| model.get_text_by_index(0, row, column))
        .collect()
}

#[test]
fn missing_elements_and_errors_left_right_mid() {
    let mut model = model_with_a_column();
    // Nine texts and five numbers
    model.set_a1("C1", "=LEFT(A1:A9, SEQUENCE(5))");
    model.set_a1("D1", "=RIGHT(A1:A9, SEQUENCE(5))");
    model.set_a1("E1", "=MID(A1:A9, SEQUENCE(5), 1)");
    model.set_a1("F1", "=MID(A1:A9, 1, SEQUENCE(5))");
    model.evaluate();

    let tail = ["#DIV/0!", "#N/A", "#NUM!", "#N/A", ""];
    assert_eq!(column(&model, 3, 5), ["H", "Ad", "Vam", "x-y-", "abcde"]);
    assert_eq!(column(&model, 4, 5), ["o", "os", "mos", "-y-z", "abcde"]);
    assert_eq!(column(&model, 5, 5), ["H", "d", "m", "-", "e"]);
    for c in 3..=6 {
        assert_eq!(column(&model, c, 10)[5..], tail, "column {c}");
    }
}

#[test]
fn missing_elements_and_errors_other_functions() {
    let mut model = model_with_a_column();
    model.set_a1("C1", "=FIND(\"o\", A1:A9, {1;1;1;1;1})");
    model.set_a1("D1", "=SEARCH(\"O\", A1:A9, {1;1;1;1;1})");
    model.set_a1("E1", "=REPT(A1:A9, {1;1;1;1;1})");
    model.set_a1(
        "F1",
        "=SUBSTITUTE(A1:A9, \"o\", {\"0\";\"0\";\"0\";\"0\";\"0\"})",
    );
    model.set_a1(
        "G1",
        "=REPLACE(A1:A9, 1, 1, {\"Z\";\"Z\";\"Z\";\"Z\";\"Z\"})",
    );
    model.set_a1("H1", "=EXACT(A1:A9, {\"Hello\";\"b\";\"c\";\"d\";\"e\"})");
    model.evaluate();

    let tail = ["#DIV/0!", "#N/A", "#NUM!", "#N/A", ""];
    assert_eq!(column(&model, 3, 2), ["5", "4"]);
    assert_eq!(column(&model, 5, 2), ["Hello", "Adios"]);
    assert_eq!(column(&model, 6, 2), ["Hell0", "Adi0s"]);
    assert_eq!(column(&model, 7, 2), ["Zello", "Zdios"]);
    assert_eq!(column(&model, 8, 2), ["TRUE", "FALSE"]);
    for c in 3..=8 {
        assert_eq!(column(&model, c, 10)[5..], tail, "column {c}");
    }
}

#[test]
fn a_missing_element_in_an_earlier_argument_comes_first() {
    let mut model = model_with_a_column();
    // Five texts and nine numbers, some of them errors: there is no text to
    // take anything from, whatever the number is
    model.set_a1("C1", "=LEFT({\"a\";\"b\";\"c\";\"d\";\"e\"}, LEN(A1:A9))");
    model.set_a1("D1", "=FIND({\"a\";\"b\";\"c\";\"d\";\"e\"}, A1:A9)");
    model.evaluate();

    assert_eq!(
        column(&model, 3, 10)[5..],
        ["#N/A", "#N/A", "#N/A", "#N/A", ""]
    );
    assert_eq!(
        column(&model, 4, 10)[5..],
        ["#N/A", "#N/A", "#N/A", "#N/A", ""]
    );
}

// poi ElapsedFormatTests A52:A215, FormatChoiceTests A27:A53: `=TEXT(Cn, Bn)` over blanks is "".
#[test]
fn text_blank_value_with_empty_format_is_empty() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXT(C1,B1)");
    model.set_a1("A2", "=TEXT(C1,\"\")");
    model.set_a1("A3", "=ISTEXT(TEXT(C1,B1))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "");
    assert_eq!(model.get_text_a1("A2"), "");
    assert_eq!(model.get_text_a1("A3"), "TRUE");
}

/// An error argument beats a start or length range failure (probes/error-arg-vs-range-check, T-1056).
#[test]
fn replace_error_argument_beats_range_check() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=REPLACE(\"abc\",0,NA(),\"x\")");
    model.set_a1("A2", "=REPLACE(\"abc\",1,-1,NA())");
    model.set_a1("A3", "=REPLACE(\"abc\",0,1,\"x\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#N/A");
    assert_eq!(model.get_text_a1("A2"), "#N/A");
    assert_eq!(model.get_text_a1("A3"), "#VALUE!");
}

/// The *B functions delegate to their non-B counterparts, so they spill over ranges like them
/// (T-0271: `=LENB(A21:A24)` hit the scalar-context `debug_assert` from the fuzz target).
#[test]
fn byte_functions_spill_like_the_non_byte_functions() {
    let mut model = model_with_texts();
    model.set_a1("A3", "=LENB(A1:C1)");
    model.set_a1("A4", "=LEFTB(A1:C1,2)");
    model.set_a1("A5", "=RIGHTB(A1:C1,2)");
    model.set_a1("A6", "=MIDB(A1:C1,2,2)");
    model.set_a1("A7", "=FINDB(\"o\",A1:C1)");
    model.set_a1("A8", "=SEARCHB(\"o\",A1:C1)");
    model.set_a1("A9", "=REPLACEB(A1:C1,1,1,\"Z\")");
    model.set_a1("A10", "=LENB(A21:A24)");
    model.evaluate();

    assert_eq!(row(&model, 3, 4), ["11", "7", "5", ""]);
    assert_eq!(row(&model, 4, 4), ["He", " a", "x-", ""]);
    assert_eq!(row(&model, 5, 4), ["ld", "c ", "-z", ""]);
    assert_eq!(row(&model, 6, 4), ["el", "ab", "-y", ""]);
    assert_eq!(row(&model, 7, 4), ["5", "#VALUE!", "#VALUE!", ""]);
    assert_eq!(row(&model, 8, 4), ["5", "#VALUE!", "#VALUE!", ""]);
    assert_eq!(row(&model, 9, 4), ["Zello World", "Zab  c ", "Z-y-z", ""]);
    assert_eq!(model.get_text_a1("A10"), "0");
    assert_eq!(model.get_text_a1("A11"), "0");
}

#[test]
fn text_over_an_array_of_formats() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXT(1,{\"0\",#N/A})");
    model.set_a1("A2", "=TEXT({1,2},{\"0.0\",\"00\"})");
    model.evaluate();
    assert_eq!(row(&model, 1, 2), ["1", "#N/A"]);
    assert_eq!(row(&model, 2, 2), ["1.0", "02"]);
}

#[test]
fn text_nested_in_the_format_is_not_reevaluated() {
    let mut model = new_empty_model();
    let mut f = String::from("\"0\"");
    for _ in 0..40 {
        f = format!("TEXT(0,{f})");
    }
    model.set_a1("A1", &format!("=TEXT(1,{f})"));
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"1");
}

#[test]
fn text_error_value_with_an_array_of_formats() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXT(NA(),{\"0\",\"0\"})");
    model.set_a1("A2", "=TEXT({1,#N/A},{\"0\",#DIV/0!})");
    model.evaluate();
    assert_eq!(row(&model, 1, 2), ["#N/A", "#N/A"]);
    assert_eq!(row(&model, 2, 2), ["1", "#N/A"]);
}

/// One result per format, following the format array's orientation (probes/text-format-array A20
/// and A24, excel-graph, T-1458).
#[test]
fn text_over_a_column_array_and_a_range_of_formats() {
    let mut model = new_empty_model();
    model.set_a1("C1", "'0");
    model.set_a1("C2", "'0.0");
    model.set_a1("A1", "=ROWS(TEXT(1,{\"0\";\"0.0\"}))");
    model.set_a1("A2", "=TEXT(1,C1:C2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(column(&model, 1, 3), ["2", "1", "1.0"]);
}

/// A 32,767-character code is over the 255-character limit, so every element of an array is
/// `#VALUE!` before any formatting: the call stays linear in the code's length (number-formats.md
/// "TEXT format code complexity limit"; probes/t2842-dollar-run-contexts J, K; T-2952).
#[test]
fn text_with_a_32767_character_code_over_an_array_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        "=SUM(--ISERROR(TEXT(SEQUENCE(1024),REPT(\"0\",32767))))",
    );
    model.set_a1("A2", "=INDEX(TEXT(SEQUENCE(4,4),REPT(\"a\",32767)),4,4)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"1024");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
}
