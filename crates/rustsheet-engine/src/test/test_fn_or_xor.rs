// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::items_after_statements)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]
#![allow(clippy::print_stdout)]
use crate::test::util::new_empty_model;

// These tests are grouped because in many cases XOR and OR have similar behaviour.

// Test specific to xor
#[test]
fn fn_xor() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=XOR(1, 1, 1, 0, 0)");
    model.set_a1("A2", "=XOR(1, 1, 0, 0, 0)");
    model.set_a1("A3", "=XOR(TRUE, TRUE, TRUE, FALSE, FALSE)");
    model.set_a1("A4", "=XOR(TRUE, TRUE, FALSE, FALSE, FALSE)");
    model.set_a1("A5", "=XOR(FALSE, FALSE, FALSE, FALSE, FALSE)");
    model.set_a1("A6", "=XOR(TRUE, TRUE)");
    model.set_a1("A7", "=XOR(0,0,0)");
    model.set_a1("A8", "=XOR(0,0,1)");
    model.set_a1("A9", "=XOR(0,1,0)");
    model.set_a1("A10", "=XOR(0,1,1)");
    model.set_a1("A11", "=XOR(1,0,0)");
    model.set_a1("A12", "=XOR(1,0,1)");
    model.set_a1("A13", "=XOR(1,1,0)");
    model.set_a1("A14", "=XOR(1,1,1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"FALSE");
    assert_eq!(model.get_text_a1("A3"), *"TRUE");
    assert_eq!(model.get_text_a1("A4"), *"FALSE");
    assert_eq!(model.get_text_a1("A5"), *"FALSE");
    assert_eq!(model.get_text_a1("A6"), *"FALSE");
    assert_eq!(model.get_text_a1("A7"), *"FALSE");
    assert_eq!(model.get_text_a1("A8"), *"TRUE");
    assert_eq!(model.get_text_a1("A9"), *"TRUE");
    assert_eq!(model.get_text_a1("A10"), *"FALSE");
    assert_eq!(model.get_text_a1("A11"), *"TRUE");
    assert_eq!(model.get_text_a1("A12"), *"FALSE");
    assert_eq!(model.get_text_a1("A13"), *"FALSE");
    assert_eq!(model.get_text_a1("A14"), *"TRUE");
}

#[test]
fn fn_or() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=OR(1, 1, 1, 0, 0)");
    model.set_a1("A2", "=OR(1, 1, 0, 0, 0)");
    model.set_a1("A3", "=OR(TRUE, TRUE, TRUE, FALSE, FALSE)");
    model.set_a1("A4", "=OR(TRUE, TRUE, FALSE, FALSE, FALSE)");
    model.set_a1("A5", "=OR(FALSE, FALSE, FALSE, FALSE, FALSE)");
    model.set_a1("A6", "=OR(TRUE, TRUE)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    assert_eq!(model.get_text_a1("A3"), *"TRUE");
    assert_eq!(model.get_text_a1("A4"), *"TRUE");
    assert_eq!(model.get_text_a1("A5"), *"FALSE");
    assert_eq!(model.get_text_a1("A6"), *"TRUE");
}

#[test]
fn fn_or_xor() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();

        // Text args
        model.set_a1("A1", &format!(r#"={func}("")"#));
        model.set_a1("A2", &format!(r#"={func}("", "")"#));
        model.set_a1("A3", &format!(r#"={func}("", TRUE)"#));
        model.set_a1("A4", &format!(r#"={func}("", FALSE)"#));

        model.set_a1("A5", &format!("={func}(FALSE, TRUE)"));
        model.set_a1("A6", &format!("={func}(FALSE, FALSE)"));
        model.set_a1("A7", &format!("={func}(TRUE, FALSE)"));

        // Reference to empty cell, plus true argument
        model.set_a1("A8", &format!("={func}(Z99, 1)"));

        // Reference to empty cell/range
        model.set_a1("A9", &format!("={func}(Z99)"));
        model.set_a1("A10", &format!("={func}(X99:Z99"));

        // Reference to text cell
        model.set_a1("B13", "some_text");
        model.set_a1("A13", &format!("={func}(B13)"));
        model.set_a1("A14", &format!("={func}(B13, 0)"));
        model.set_a1("A15", &format!("={func}(B13, 1)"));

        // Reference to Implicit intersection
        model.set_a1("X16", "1");
        model.set_a1("B16", "=@X15:X16");
        model.set_a1("A16", &format!("={func}(B16)"));

        // Non-empty range
        model.set_a1("B17", "1");
        model.set_a1("A17", &format!("={func}(B17:C17)"));

        // Non-empty range with text
        model.set_a1("B18", "text");
        model.set_a1("A18", &format!("={func}(B18:C18)"));

        // Non-empty range with text and number
        model.set_a1("B19", "text");
        model.set_a1("C19", "1");
        model.set_a1("A19", &format!("={func}(B19:C19)"));

        // range with error
        model.set_a1("B20", "=1/0");
        model.set_a1("A20", &format!("={func}(B20:C20)"));

        model.evaluate();

        assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
        assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
        assert_eq!(model.get_text_a1("A3"), *"TRUE");
        assert_eq!(model.get_text_a1("A4"), *"FALSE");

        assert_eq!(model.get_text_a1("A5"), *"TRUE");
        assert_eq!(model.get_text_a1("A6"), *"FALSE");
        assert_eq!(model.get_text_a1("A7"), *"TRUE");

        assert_eq!(model.get_text_a1("A8"), *"TRUE");

        assert_eq!(model.get_text_a1("A9"), *"#VALUE!");
        assert_eq!(model.get_text_a1("A10"), *"#VALUE!");

        assert_eq!(model.get_text_a1("A13"), *"#VALUE!");
        assert_eq!(model.get_text_a1("A14"), *"FALSE");
        assert_eq!(model.get_text_a1("A15"), *"TRUE");

        assert_eq!(model.get_text_a1("A17"), *"TRUE");

        assert_eq!(model.get_text_a1("A18"), *"#VALUE!");

        assert_eq!(model.get_text_a1("A19"), *"TRUE");

        assert_eq!(model.get_text_a1("A20"), *"#DIV/0!");
    }
}

#[test]
fn fn_or_xor_arrays() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();

        // Boolean array literals
        model.set_a1("A1", &format!("={func}({{TRUE, FALSE, FALSE}})"));
        model.set_a1("A2", &format!("={func}({{FALSE, FALSE, FALSE}})"));

        // Number array literals (0 is FALSE, anything else TRUE)
        model.set_a1("A3", &format!("={func}({{0, 0, 1}})"));
        model.set_a1("A4", &format!("={func}({{0, 0, 0}})"));

        // Strings in arrays are ignored
        model.set_a1("A5", &format!(r#"={func}({{"text", TRUE}})"#));
        model.set_a1("A6", &format!(r#"={func}({{"text", FALSE}})"#));

        // Two-dimensional array combined with a scalar argument
        model.set_a1("A7", &format!("={func}({{FALSE; FALSE}}, TRUE)"));

        // Errors inside an array are propagated
        model.set_a1("A8", &format!(r"={func}({{#DIV/0!, TRUE}})"));

        model.evaluate();

        assert_eq!(model.get_text_a1("A1"), *"TRUE");
        assert_eq!(model.get_text_a1("A2"), *"FALSE");
        assert_eq!(model.get_text_a1("A3"), *"TRUE");
        assert_eq!(model.get_text_a1("A4"), *"FALSE");
        assert_eq!(model.get_text_a1("A5"), *"TRUE");
        assert_eq!(model.get_text_a1("A6"), *"FALSE");
        assert_eq!(model.get_text_a1("A7"), *"TRUE");
        assert_eq!(model.get_text_a1("A8"), *"#DIV/0!");
    }
}

#[test]
fn fn_and_arrays() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=AND({TRUE, TRUE, TRUE})");
    model.set_a1("A2", "=AND({TRUE, FALSE, TRUE})");
    model.set_a1("A3", "=AND({1, 2, 3})");
    model.set_a1("A4", "=AND({1, 0, 3})");
    // Strings ignored
    model.set_a1("A5", r#"=AND({"text", TRUE})"#);
    // Error propagated
    model.set_a1("A6", "=AND({TRUE, #N/A})");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"FALSE");
    assert_eq!(model.get_text_a1("A3"), *"TRUE");
    assert_eq!(model.get_text_a1("A4"), *"FALSE");
    assert_eq!(model.get_text_a1("A5"), *"TRUE");
    assert_eq!(model.get_text_a1("A6"), *"#N/A");
}

#[test]
#[ignore = "not yet implemented"]
fn spill_behaviour() {
    let mut model = new_empty_model();

    // Reference to cell with reference to empty range
    model.set_a1("B11", "=X99:Z99");
    model.set_a1("A11", "=OR(B11)");

    // Reference to cell with non-empty range
    model.set_a1("X12", "1");
    model.set_a1("B12", "=X12:Z12");
    model.set_a1("A12", "=OR(B12)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A11"), *"FALSE");

    assert_eq!(model.get_text_a1("A12"), *"TRUE");
}

#[test]
fn fn_or_xor_no_arguments() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();
        model.set_a1("A1", &format!("={func}()"));
        model.evaluate();
        assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    }
}

#[test]
fn fn_or_xor_missing_arguments() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();
        model.set_a1("A1", &format!("={func}(,)"));
        model.set_a1("A2", &format!("={func}(,1)"));
        model.set_a1("A3", &format!("={func}(1,)"));
        model.set_a1("A4", &format!("={func}(,B1)"));
        model.set_a1("A5", &format!("={func}(,B1:B4)"));
        model.evaluate();
        assert_eq!(model.get_text_a1("A1"), *"FALSE");
        assert_eq!(model.get_text_a1("A2"), *"TRUE");
        assert_eq!(model.get_text_a1("A3"), *"TRUE");
        assert_eq!(model.get_text_a1("A4"), *"FALSE");
        assert_eq!(model.get_text_a1("A5"), *"FALSE");
    }
}

// AND/OR evaluate every argument: an error after a decisive value still wins
// (T-0215; docs/excel-semantics/coercion.md).
#[test]
fn test_and_or_do_not_short_circuit_past_errors() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=OR(TRUE,1/0)");
    model.set_a1("A2", "=AND(FALSE,1/0)");
    model.set_a1("A3", "=OR(TRUE,NA())");
    model.set_a1("A4", "=AND(FALSE,NA())");
    model.set_a1("B1", "1");
    model.set_a1("B2", "=1/0");
    // A5/A6 (an error inside a range argument) are UNVERIFIED: no oracle case, probe in T-0229.
    model.set_a1("A5", "=OR(B1:B2)");
    model.set_a1("A6", "=OR(TRUE,B1:B2)");
    model.set_a1("A7", "=AND(FALSE,TRUE)");
    model.set_a1("A8", "=OR(TRUE,FALSE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A2"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A3"), *"#N/A");
    assert_eq!(model.get_text_a1("A4"), *"#N/A");
    assert_eq!(model.get_text_a1("A5"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A6"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A7"), *"FALSE");
    assert_eq!(model.get_text_a1("A8"), *"TRUE");
}

// NOT lifts over arrays and ranges, one result per element (T-0225).
#[test]
fn test_not_lifts_over_arrays_and_ranges() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=NOT({TRUE,FALSE})");
    model.set_a1("A3", "TRUE");
    model.set_a1("A4", "0");
    model.set_a1("C1", "=NOT(A3:A4)");
    model.set_a1("E1", "=NOT(TRUE)");
    // Two whole columns exceed the shared array allocation bound (DEVIATION, decision 0014).
    model.set_a1("G1", "=NOT(Z:AA)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"FALSE");
    assert_eq!(model.get_text_a1("B1"), *"TRUE");
    assert_eq!(model.get_text_a1("C1"), *"FALSE");
    assert_eq!(model.get_text_a1("C2"), *"TRUE");
    assert_eq!(model.get_text_a1("E1"), *"FALSE");
    assert_eq!(model.get_text_a1("G1"), *"#ERROR!");
}

/// Nightly fuzz OOM (T-0230): `IFERROR` hands `OR` the reference `F:LSE` (8,606 whole
/// columns). The walk is clipped to the used area, so this is quick and bounded, and the
/// result is that of an empty range: no logical values, `#VALUE!`.
#[test]
fn fuzz_oom_or_over_whole_columns() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=OR(IFERROR(ERROR.TYPE(TRUNC($Z$1))=5,F:LSE))");
    model.set_a1("A2", "=OR(F:LSE)");
    let start = std::time::Instant::now();
    model.evaluate();
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
}

/// Several errors: the first one, arguments left to right and ranges row by row, wins for
/// AND, OR and XOR (probes/and-or-xor-error-order, excel-graph, T-0229).
#[test]
fn first_error_wins_row_by_row() {
    let mut model = new_empty_model();
    // D1:F2 = TRUE, #DIV/0!, FALSE / #N/A, TRUE, #NUM!
    model.set_a1("D1", "TRUE");
    model.set_a1("E1", "=1/0");
    model.set_a1("F1", "FALSE");
    model.set_a1("D2", "=NA()");
    model.set_a1("E2", "TRUE");
    model.set_a1("F2", "=SQRT(-1)");
    model.set_a1("A1", "=OR(TRUE,NA(),1/0)");
    model.set_a1("A2", "=AND(FALSE,1/0,NA())");
    model.set_a1("A3", "=XOR(D1:F2)");
    model.set_a1("A4", "=OR(TRANSPOSE(D1:F2))");
    model.set_a1("A5", "=AND(D1:F2,NA())");
    model.set_a1("A6", "=OR({TRUE,#DIV/0!,#N/A})");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#N/A");
    assert_eq!(model.get_text_a1("A2"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A3"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A4"), *"#N/A");
    assert_eq!(model.get_text_a1("A5"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A6"), *"#DIV/0!");
}

// AND, OR and XOR read through the SPEC-0009 walker with today's results kept (T-1677).
#[test]
fn logical_walker_keeps_text_empty_and_error_rules() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=\"TRUE\"");
    model.set_a1("A2", "x");
    model.new_sheet();
    // Direct text counts in any case; referenced text, also "TRUE", is skipped.
    model.set_a1("B1", "=AND(\"true\")");
    model.set_a1("B2", "=OR(\"FALSE\",\"tRuE\")");
    model.set_a1("B3", "=AND(A1)");
    model.set_a1("B4", "=OR(A1:A2,FALSE)");
    model.set_a1("B5", "=OR({\"TRUE\"},FALSE)");
    // An empty argument is FALSE only when nothing came before.
    model.set_a1("B6", "=AND(TRUE,)");
    model.set_a1("B7", "=AND(,TRUE)");
    model.set_a1("B8", "=OR(,)");
    // The first error wins after a decisive value.
    model.set_a1("B9", "=AND(FALSE,1/0)");
    model.set_a1("B10", "=OR(TRUE,{1,#N/A},1/0)");
    // A whole column on an empty sheet has no logical value.
    model.set_a1("B11", "=OR(Sheet2!C:C)");
    // A direct LAMBDA is `#VALUE!`, held so a direct error beats it (probes/and-or-xor-picked-text).
    model.set_a1("B13", "=AND(LAMBDA(x,x),1/0)");
    model.set_a1("B14", "=XOR(A1:A2)");
    // A text "TRUE" cell picked by a function is a referenced cell: skipped (T-1685).
    model.set_a1("B15", "=AND(IF(TRUE,A1))");
    model.set_a1("B16", "=OR(INDEX(A1:A2,1))");
    model.set_a1("B17", "=XOR(LET(x,A1,x))");
    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), "TRUE");
    assert_eq!(model.get_text_a1("B2"), "TRUE");
    assert_eq!(model.get_text_a1("B3"), "#VALUE!");
    assert_eq!(model.get_text_a1("B4"), "FALSE");
    assert_eq!(model.get_text_a1("B5"), "FALSE");
    assert_eq!(model.get_text_a1("B6"), "TRUE");
    assert_eq!(model.get_text_a1("B7"), "FALSE");
    assert_eq!(model.get_text_a1("B8"), "FALSE");
    assert_eq!(model.get_text_a1("B9"), "#DIV/0!");
    assert_eq!(model.get_text_a1("B10"), "#N/A");
    assert_eq!(model.get_text_a1("B11"), "#VALUE!");
    assert_eq!(model.get_text_a1("B13"), "#DIV/0!");
    assert_eq!(model.get_text_a1("B14"), "#VALUE!");
    assert_eq!(model.get_text_a1("B15"), "#VALUE!");
    assert_eq!(model.get_text_a1("B16"), "#VALUE!");
    assert_eq!(model.get_text_a1("B17"), "#VALUE!");
}
