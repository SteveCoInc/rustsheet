#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_evaluation() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(x, y, x + y)(1, 2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"3");
}

#[test]
fn evaluation_in_let() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LET(x, 1, y, LAMBDA(a, b, a + b), y(x, 22))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"23");
}

#[test]
fn evaluation_with_defined_name() {
    let mut model = new_empty_model();
    model
        .new_defined_name("MySum", None, "=LAMBDA(x, y, x + y)")
        .unwrap();
    model.set_a1("A1", "=MySum(1, 2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"3");
}

// Regression: a defined-name LAMBDA that returns an array must spill.
// Calling it (a `NamedFunctionKind`) was statically classified as a scalar,
// so the cell was never marked dynamic and the array result reached a
// scalar context instead of spilling.
#[test]
fn defined_name_lambda_spills_array() {
    let mut model = new_empty_model();
    model
        .new_defined_name("MySeq", None, "=LAMBDA(n, SEQUENCE(n))")
        .unwrap();
    model.set_a1("A1", "=MySeq(3)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"2");
    assert_eq!(model.get_text_a1("A3"), *"3");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(x, y, x + y)(1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
}

#[test]
fn returns_calculation_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(x, y, x + y)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#CALC!");
}

#[test]
fn sheet_local_name_takes_precedence_over_global() {
    // Global MyFn = LAMBDA(x, x * 2)
    // Sheet2-local MyFn = LAMBDA(x, x * 3)
    // Sheet1!A1 calls MyFn(5) => uses global => 10
    // Sheet2!A1 calls MyFn(5) => uses local  => 15
    let mut model = new_empty_model();
    model.new_sheet(); // adds Sheet2 at index 1

    model
        .new_defined_name("MyFn", None, "=LAMBDA(x, x * 2)")
        .unwrap();
    model
        .new_defined_name("MyFn", Some(1), "=LAMBDA(x, x * 3)")
        .unwrap();

    model.set_a1("A1", "=MyFn(5)");
    model.set_a1("Sheet2!A1", "=MyFn(5)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"10");
    assert_eq!(model.get_text_a1("Sheet2!A1"), *"15");
}

#[test]
fn let_bound_variable_captured_in_lambda_body() {
    let mut model = new_empty_model();
    // `a` is bound by LET; the LAMBDA body references `a` as a closure variable.
    // f(2) should return 2 + 1 = 3.
    model.set_a1("A1", "=LET(a, 1, f, LAMBDA(x, x + a), f(2))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"3");
}

#[test]
fn optional_parameter_omitted() {
    let mut model = new_empty_model();
    // b is optional; calling with one arg should use default (a/2 = 6).
    model.set_a1("A1", "=LAMBDA(a, [b], IF(ISOMITTED(b), a/2, a*b))(12)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"6");
}

#[test]
fn optional_parameter_provided() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(a, [b], IF(ISOMITTED(b), a/2, a*b))(12, 3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"36");
}

#[test]
fn optional_parameter_too_many_args_is_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(a, [b], a+1)(1, 2, 3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
}

#[test]
fn required_arg_missing_is_error() {
    let mut model = new_empty_model();
    // `a` is required; calling with no args is an error.
    model.set_a1("A1", "=LAMBDA(a, [b], a+1)()");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
}

// A reference given to a LAMBDA, or bound by a LET, stays a reference, also
// when it is a single cell. Where a value is wanted it is the cell's value.
#[test]
fn a_single_cell_is_passed_as_a_reference() {
    let mut model = new_empty_model();
    model.set_a1("A1", "text");
    model.set_a1("A2", "7");
    model.set_a1("B5", "3");
    model.set_a1("C1", "=LAMBDA(x, MAX(x))(A1)");
    model.set_a1("C2", "=LAMBDA(x, MAX(x))(A2)");
    model.set_a1("C3", "=LAMBDA(x, ROW(x))(B5)");
    model.set_a1("C4", "=LAMBDA(x, COLUMN(x))(B5)");
    model.set_a1("C5", "=LAMBDA(x, OFFSET(x, 1, 0))(A1)");
    model.set_a1("C6", "=LAMBDA(x, ROWS(x))(B5)");
    model.set_a1("C9", "=LET(x, A1, MAX(x))");
    model.set_a1("C10", "=LET(x, B5, ROW(x))");
    model.set_a1("C11", "=LET(x, A2, y, B5, OFFSET(x, 0, 0)+y)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"0");
    assert_eq!(model.get_text_a1("C2"), *"7");
    assert_eq!(model.get_text_a1("C3"), *"5");
    assert_eq!(model.get_text_a1("C4"), *"2");
    assert_eq!(model.get_text_a1("C5"), *"7");
    assert_eq!(model.get_text_a1("C6"), *"1");
    assert_eq!(model.get_text_a1("C9"), *"0");
    assert_eq!(model.get_text_a1("C10"), *"5");
    assert_eq!(model.get_text_a1("C11"), *"10");
}

#[test]
fn a_single_cell_parameter_is_a_value_where_a_value_is_wanted() {
    let mut model = new_empty_model();
    model.set_a1("A1", "text");
    model.set_a1("A2", "7");
    model.set_a1("C1", "=LAMBDA(x, x)(A1)");
    model.set_a1("C2", "=LAMBDA(x, x)(A2)");
    model.set_a1("C3", "=LAMBDA(x, x&\"!\")(A1)");
    model.set_a1("C4", "=LAMBDA(x, x+1)(A2)");
    model.set_a1("C5", "=LAMBDA(x, IF(x=\"text\", 1, 2))(A1)");
    model.set_a1("C6", "=LAMBDA(x, ISBLANK(x))(A3)");
    model.set_a1("C7", "=LAMBDA(x, ISTEXT(x))(A1)");
    model.set_a1("C8", "=LAMBDA(x, LEN(x))(A1)");
    model.set_a1("C9", "=LET(x, A2, x*2)");
    model.set_a1("C10", "=LAMBDA(x, TYPE(x))(A2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"text");
    assert_eq!(model.get_text_a1("C2"), *"7");
    assert_eq!(model.get_text_a1("C3"), *"text!");
    assert_eq!(model.get_text_a1("C4"), *"8");
    assert_eq!(model.get_text_a1("C5"), *"1");
    assert_eq!(model.get_text_a1("C6"), *"TRUE");
    assert_eq!(model.get_text_a1("C7"), *"TRUE");
    assert_eq!(model.get_text_a1("C8"), *"4");
    assert_eq!(model.get_text_a1("C9"), *"14");
    assert_eq!(model.get_text_a1("C10"), *"1");
    // Nothing spills from a single cell
    assert_eq!(model.get_text_a1("D1"), *"");
    assert_eq!(model.get_text_a1("C11"), *"");
}

// A defined name that stands for a cell is a reference wherever one is
// wanted, and the cell's value wherever a value is wanted
#[test]
fn a_defined_name_for_a_single_cell_is_a_reference() {
    let mut model = new_empty_model();
    model.set_a1("K1", "text");
    model.set_a1("K2", "7");
    model
        .new_defined_name("TextCell", None, "Sheet1!$K$1")
        .unwrap();
    model
        .new_defined_name("Later", None, "=LAMBDA(x, x+1)")
        .unwrap();
    model.set_a1("A1", "=ROW(TextCell)");
    model.set_a1("A2", "=COLUMN(TextCell)");
    model.set_a1("A3", "=OFFSET(TextCell, 1, 0)");
    model.set_a1("A4", "=ROWS(TextCell)");
    model.set_a1("A5", "=TextCell&\"!\"");
    model.set_a1("A6", "=LEN(TextCell)");
    model.set_a1("A7", "=IF(TextCell=\"text\", 1, 2)");
    model.set_a1("A8", "=TextCell");
    model.set_a1("A9", "=Later(1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"11");
    assert_eq!(model.get_text_a1("A3"), *"7");
    assert_eq!(model.get_text_a1("A4"), *"1");
    assert_eq!(model.get_text_a1("A5"), *"text!");
    assert_eq!(model.get_text_a1("A6"), *"4");
    assert_eq!(model.get_text_a1("A7"), *"1");
    assert_eq!(model.get_text_a1("A8"), *"text");
    assert_eq!(model.get_text_a1("A9"), *"2");
}

// A called LET name or LAMBDA parameter that holds a value: a number or boolean is #NAME?, text
// is 0, an error is itself, an array is called per element and a reference (INDEX's too) is
// #REF! (probes/let-name-call*, lambda-param-call; T-1092).
#[test]
fn called_variable_holding_a_value() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("C1", "=LET(f, 1, f(2))");
    model.set_a1("C2", "=LET(f, \"a\", f(2))");
    model.set_a1("C3", "=LET(f, 1/0, f(2))");
    model.set_a1("C4", "=LET(f, INDEX(A1:A2, 2), f(2))");
    model.set_a1("C5", "=LAMBDA(f, f(2))(TRUE)");
    model.set_a1("C6", "=LET(f, {1;\"a\"}, f(2))");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"#NAME?");
    assert_eq!(model.get_text_a1("C2"), *"0");
    assert_eq!(model.get_text_a1("C3"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C4"), *"#REF!");
    assert_eq!(model.get_text_a1("C5"), *"#NAME?");
    assert_eq!(model.get_text_a1("C6"), *"#NAME?");
    // C7 is C6's spill.
    assert_eq!(model.get_text_a1("C7"), *"0");
}

#[test]
fn duplicate_parameters_are_refused_in_any_case() {
    // Excel for the web refuses `LAMBDA(x,x,x)(1,2)`, `LAMBDA(x,X,x+X)(1,2)` and
    // `LAMBDA(ä,Ä,ä+Ä)(1,2)` (docs/excel-semantics/references.md, T-1387).
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(x,x,x)(1,2)");
    model.set_a1("A2", "=LAMBDA(x,X,x+X)(1,2)");
    model.set_a1("A3", "=LAMBDA(ä,Ä,ä+Ä)(1,2)");
    model.set_a1("A4", "=LAMBDA(x,[X],x)(1)");
    model.set_a1("A5", "=LAMBDA(ß,ß*2)(3)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model.get_text_a1(cell), *"#ERROR!", "{cell}");
    }
    assert_eq!(model.get_text_a1("A5"), *"6");
}

// Messages are not Excel-visible; the value stays `#NAME?`.
#[test]
fn calling_a_number_or_boolean_variable_is_name_with_a_message() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LET(f, 1, f(2))");
    model.set_a1("A2", "=LET(f, TRUE, f(2))");
    model.evaluate();

    for cell in ["A1", "A2"] {
        assert_eq!(model.get_text_a1(cell), *"#NAME?");
        assert_eq!(model.get_error_message_a1(cell), "A value cannot be called");
    }
}

#[test]
fn a_call_of_a_call_is_chained() {
    // probes/lambda-escaping-closures H8 and H10 (excel-graph): a LAMBDA returned by a call is
    // called by the next `(...)` (T-2038).
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(a,LAMBDA(b,a*b))(3)(4)");
    model.set_a1("A2", "=LAMBDA(a,LAMBDA(b,LAMBDA(c,a*100+b*10+c)))(1)(2)(3)");
    model.set_a1("A4", "=(LAMBDA(a,LAMBDA(b,a&b)))(\"x\")(\"y\")");
    model.set_a1("A3", "=LET(f,LAMBDA(a,LAMBDA(b,a-b)),f(10)(4))");
    model.set_a1("A5", "=LAMBDA(x,x)(1)(2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "12");
    assert_eq!(model.get_text_a1("A2"), "123");
    assert_eq!(model.get_text_a1("A4"), "xy");
    assert_eq!(model.get_text_a1("A3"), "6");
    // A returned value is called like a LET-bound one (probes/chained-calls-lambda B4).
    assert_eq!(model.get_text_a1("A5"), "#NAME?");
    model.set_a1("A6", "=SUM(LAMBDA(x,x)(1)(2))");
    model.set_a1("A7", "=LAMBDA(x,x)(\"a\")(2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A6"), "#NAME?");
    assert_eq!(model.get_text_a1("A7"), "0");
    // Excel for the web refuses a workbook holding a call followed by a space and `(4)`, or a
    // call of SUM's result (probes T-2038, docs/excel-semantics/references.md "Calling a call's
    // result"): the engine reads them as syntax errors.
    for (cell, formula) in [
        ("B1", "=LAMBDA(a,LAMBDA(b,a*b))(3) (4)"),
        ("B2", "=SUM(1)(2)"),
    ] {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "#ERROR!");
    assert_eq!(model.get_text_a1("B2"), "#ERROR!");
    // The text round-trips.
    assert_eq!(model.get_formula_a1("A1"), "=LAMBDA(a,LAMBDA(b,a*b))(3)(4)");
    assert_eq!(
        model.get_formula_a1("A2"),
        "=LAMBDA(a,LAMBDA(b,LAMBDA(c,a*100+b*10+c)))(1)(2)(3)"
    );
    assert_eq!(
        model.get_formula_a1("A3"),
        "=LET(f,LAMBDA(a,LAMBDA(b,a-b)),f(10)(4))"
    );
    assert_eq!(
        model.get_formula_a1("A4"),
        "=(LAMBDA(a,LAMBDA(b,a&b)))(\"x\")(\"y\")"
    );
}

#[test]
fn a_call_of_a_reference_function_result_is_chained() {
    // probes/chained-calls-builtin-* and chained-calls-ref-4, -6 (excel-graph, T-2405, T-2499):
    // the result of a built-in that can return a reference (or a LAMBDA) may be called; Excel
    // refuses the others (`SUM(1)(2)`, `ABS(1)(2)`, `IFERROR`, `IFNA`), which stay syntax errors.
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3")] {
        model.set_a1(cell, value);
    }
    let cases = [
        ("=IF(TRUE,LAMBDA(x,x*2))(3)", "6"),
        ("=CHOOSE(1,LAMBDA(x,x*2))(3)", "6"),
        ("=IFS(TRUE,LAMBDA(x,x*2))(3)", "6"),
        ("=SWITCH(1,1,LAMBDA(x,x*2))(3)", "6"),
        ("=LET(f,LAMBDA(x,x*2),f)(3)", "6"),
        ("=INDEX(A1:A3,1)(2)", "#REF!"),
        ("=OFFSET(A1,0,0)(2)", "#REF!"),
        ("=INDIRECT(\"A1\")(2)", "#REF!"),
        ("=CHOOSE(1,A1:A3)(2)", "#REF!"),
        // A called TAKE, DROP, TRIMRANGE or XLOOKUP over a range is the range it picks (T-2499).
        ("=TAKE(A1:A3,1)(2)", "#REF!"),
        ("=DROP(A1:A3,1)(2)", "#REF!"),
        ("=TRIMRANGE(A1:A3)(2)", "#REF!"),
        ("=XLOOKUP(1,A1:A3,A1:A3)(2)", "#REF!"),
        ("=INDEX({1,2},1)(2)", "#NAME?"),
        ("=SUM(1)(2)", "#ERROR!"),
        ("=ABS(1)(2)", "#ERROR!"),
        ("=IFERROR(LAMBDA(x,x*2),0)(3)", "#ERROR!"),
        ("=IFNA(LAMBDA(x,x*2),0)(3)", "#ERROR!"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("C{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, value)) in cases.iter().enumerate() {
        let cell = format!("C{}", row + 1);
        assert_eq!(model.get_text_a1(&cell), *value, "{formula}");
        if *value != "#ERROR!" {
            // FORMULATEXT prints the callee bare (probes/chained-calls-builtin-if-text C1).
            assert_eq!(model.get_formula_a1(&cell), *formula);
        }
    }
}

#[test]
fn a_long_call_chain_is_bounded_like_nesting() {
    // Each chained call is a level of the tree, so a chain past the depth limit is refused
    // rather than overflowing the stack (CLAUDE.md directive 5).
    let mut model = new_empty_model();
    let chain = "(1)".repeat(10_000);
    let refused = model.set_user_input(0, 1, 1, format!("=LAMBDA(x,x){chain}"));
    assert!(refused.unwrap_err().contains("nested too deeply"));
    let short = "(1)".repeat(20);
    model.set_a1("A2", &format!("=LAMBDA(x,x){short}"));
    model.evaluate();
    assert_eq!(model.get_formula_a1("A2"), format!("=LAMBDA(x,x){short}"));
}

// T-2925: no LAMBDA outlives the formula that made it, so a root cell's LAMBDAs are dropped once it
// is evaluated, and a later formula's calls do not slow down with what earlier ones made.
#[test]
fn a_root_cells_lambdas_are_dropped_once_it_is_evaluated() {
    let mut model = new_empty_model();
    model
        .new_defined_name("AddOne", None, "=LAMBDA(x, x + 1)")
        .unwrap();
    model.set_a1(
        "A1",
        "=SUM(MAP(SEQUENCE(10),LAMBDA(z,MAP(z,LAMBDA(y,y+z)))))",
    );
    model.set_a1(
        "A2",
        "=A1+SUM(MAP(SEQUENCE(5),LAMBDA(z,AddOne(z))))+REDUCE(0,{1,2},_xleta.SUM)",
    );
    model.set_a1("A3", "=LET(f,LAMBDA(x,LAMBDA(y,x*y)),f(3)(4))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"110");
    assert_eq!(model.get_text_a1("A2"), *"133");
    assert_eq!(model.get_text_a1("A3"), *"12");
    assert!(model.lambdas.is_empty());
    assert!(model.eta_lambdas.is_empty());
    assert!(model.lambda_captures.is_empty());
    assert!(model.named_lambdas.is_empty());
    // Ids keep growing across root cells: a patched body's mark compares them.
    assert!(model.last_lambda_id > 0);

    // A second evaluation gives the same values from a model with no LAMBDA left over.
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), *"133");
    assert!(model.lambdas.is_empty());
}

/// An uncalled LAMBDA joined as text is `#VALUE!` on either side of `&`
/// (probes/t2622-take-let-reference-lambda-text Q1-Q10; T-2622).
#[test]
fn uncalled_lambda_joined_as_text_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LAMBDA(x,x)&\"\"");
    model.set_a1("A2", "=\"a\"&LET(f,LAMBDA(x,x),f)");
    model.set_a1("A3", "=ERROR.TYPE(LAMBDA(x,x)&LAMBDA(y,y))");
    model.set_a1("A4", "=IFERROR(LAMBDA(x,x)&\"\",\"e\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
    assert_eq!(model.get_text_a1("A2"), "#VALUE!");
    assert_eq!(model.get_text_a1("A3"), "3");
    assert_eq!(model.get_text_a1("A4"), "e");
}

/// CONCAT, TEXTJOIN and CONCATENATE of an uncalled LAMBDA are `#VALUE!`, not `#N/IMPL!`
/// (probes/t3130-concat-lambda-trimrange-cell-variable Q1-Q12; T-3130).
#[test]
fn uncalled_lambda_read_as_text_by_functions_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CONCAT(\"a\",LAMBDA(x,x))");
    model.set_a1("A2", "=TEXTJOIN(\",\",TRUE,LAMBDA(x,x))");
    model.set_a1("A3", "=TEXTJOIN(LAMBDA(x,x),TRUE,\"a\",\"b\")");
    model.set_a1("A4", "=ERROR.TYPE(CONCAT(LET(f,LAMBDA(x,x),f),\"\"))");
    model.set_a1("A5", "=ERROR.TYPE(CONCATENATE(LAMBDA(x,x),\"\"))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"3");
    assert_eq!(model.get_text_a1("A5"), *"3");
}

/// A one-cell TRIMRANGE of a cell or a LET variable bound to one is a one-cell reference, and a
/// MAKEARRAY LAMBDA returning one gives its value (probes/t3130-concat-lambda-trimrange-cell-variable
/// Q13-Q16; gen-m7 a470dc9e165bdb20 A465; T-3130).
#[test]
fn one_cell_trimrange_is_a_reference() {
    let mut model = new_empty_model();
    model.set_a1("A2", "5");
    model.set_a1("A3", "t");
    model.set_a1("B1", "=TYPE(TRIMRANGE(A2))");
    model.set_a1("B2", "=TYPE(TRIMRANGE(LET(v,A2,v)))");
    model.set_a1("B3", "=ROWS(A2-TRIMRANGE(A3))");
    model.set_a1("B4", "=MAKEARRAY(1,2,LAMBDA(r,c,TRIMRANGE(A2,2,2)))");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), *"1");
    assert_eq!(model.get_text_a1("B2"), *"1");
    assert_eq!(model.get_text_a1("B3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B4"), *"5");
    assert_eq!(model.get_text_a1("C4"), *"5");
}

#[test]
fn names_starting_with_a_mark_or_an_uppercase_circled_letter_are_refused() {
    // Excel for the web refuses (createSession 501 FileCorruptTryRepair) a single-cell workbook
    // whose formula starts a name with a mark or an uppercase circled letter, and loads every
    // other form below (`rustsheet-oracle recalc -b excel-graph`, 2026-11-06, T-1856, each LET
    // name written `_xlpm.`; refused inputs cannot be committed: docs/excel-semantics/references.md
    // "Characters of a name").
    let refused = [
        "=LET(Ⓐ,1,Ⓐ+1)",
        "=LET(Ⓑ,1,Ⓑ+1)",
        "=LET(Ⓜ,1,Ⓜ+1)",
        "=LET(Ⓩ,1,Ⓩ+1)",
        "=LET(Ⓐx,1,Ⓐx+1)",
        "=LAMBDA(Ⓐ,Ⓐ+1)(1)",
        "=LET(\u{345}a,1,\u{345}a+1)",
        "=LET(\u{93E}a,1,\u{93E}a+1)",
        "=LET(\u{301}a,1,\u{301}a+1)",
        "=LET(\u{307}x,1,\u{307}x+1)",
        "=Ⓑ+1",
    ];
    // (formula, Excel's value)
    let accepted = [
        ("=LET(ⓐ,1,ⓐ+1)", "2"),
        ("=LET(ⓩ,1,ⓩ+1)", "2"),
        ("=LET(\u{1F130},1,\u{1F130}+1)", "2"),
        ("=LET(\u{1F130}x,1,\u{1F130}x+1)", "2"),
        ("=LET(\u{1F150},1,\u{1F150}+1)", "2"),
        ("=LET(aⒶ,1,aⒶ+1)", "2"),
        ("=LET(İ,1,İ+1)", "2"),
        ("=LET(xİ,1,xİ+1)", "2"),
        ("=LET(İ,1,i+1)", "#NAME?"),
        ("=LAMBDA(İ,İ+1)(1)", "2"),
        ("=LET(नाम,1,नाम+1)", "2"),
        ("=LET(क\u{93E},1,क\u{93E}+1)", "2"),
        ("=LET(a\u{345},1,a\u{345}+1)", "2"),
        ("=LET(a\u{301},1,a\u{301}+1)", "2"),
        ("=LET(i\u{307},1,i\u{307}+1)", "2"),
        ("=LET(ǅ,1,ǅ+1)", "2"),
        ("=LET(ʰ,1,ʰ+1)", "2"),
        ("=LET(ー,1,ー+1)", "2"),
        ("=LET(ª,1,ª+1)", "2"),
        ("=LET(Ⅰ,1,Ⅰ+1)", "2"),
        ("=LET(ⅰ,1,ⅰ+1)", "2"),
        ("=LET(ﬃ,1,ﬃ+1)", "2"),
        ("=LET(ŉ,1,ŉ+1)", "2"),
        ("=LET(ǰ,1,ǰ+1)", "2"),
        ("=LET(ΐ,1,ΐ+1)", "2"),
        ("=LET(שם,1,שם+1)", "2"),
        ("=LET(اسم,1,اسم+1)", "2"),
        ("=LET(a²,1,a²+1)", "2"),
        ("=LET(a٣,1,a٣+1)", "2"),
        ("=LET(a①,1,a①+1)", "2"),
        ("=LET(aⅫ,1,aⅫ+1)", "2"),
        ("=ⓩ+1", "#NAME?"),
        ("=aⒶ+1", "#NAME?"),
    ];
    let mut model = new_empty_model();
    for (row, formula) in refused
        .iter()
        .copied()
        .chain(accepted.iter().map(|(formula, _)| *formula))
        .enumerate()
    {
        model.set_a1(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, formula) in refused.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", row + 1)),
            *"#ERROR!",
            "{formula}"
        );
    }
    for (row, (formula, expected)) in accepted.iter().enumerate() {
        let cell = format!("A{}", refused.len() + row + 1);
        assert_eq!(model.get_text_a1(&cell), *expected, "{formula}");
    }
}

#[test]
fn a_sheet_name_starting_with_a_refused_name_character_is_quoted() {
    // The lexer refuses a name starting with Ⓐ-Ⓩ or a mark (T-1856), so a reference to such a
    // sheet must be written quoted to read back. How Excel writes it is unprobed (T-4446).
    let mut model = new_empty_model();
    model.add_sheet("Ⓐ").unwrap();
    model.add_sheet("ⓑ").unwrap();
    // `set_a1` parses its sheet prefix byte by byte, so these go in by sheet index.
    model.set_user_input(1, 1, 1, "5".to_string()).unwrap();
    model.set_user_input(2, 1, 1, "7".to_string()).unwrap();
    model.set_a1("A1", "='Ⓐ'!A1+1");
    model.set_a1("A2", "=FORMULATEXT(A1)");
    model.set_a1("A3", "=ⓑ!A1+1");
    model.set_a1("A4", "=FORMULATEXT(A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"6");
    assert_eq!(model.get_text_a1("A2"), *"='Ⓐ'!A1+1");
    assert_eq!(model.get_text_a1("A3"), *"8");
    assert_eq!(model.get_text_a1("A4"), *"=ⓑ!A1+1");
}

/// Special forms and reference functions passed by name (probes/t4450-eta-special-forms):
/// LET and LAMBDA are not functions by name (`#NAME?`, a helper's `#VALUE!`), IF and CHOOSE
/// read a reference value argument as a blank, IFERROR and INDEX keep their values.
#[test]
fn special_forms_passed_by_name() {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "3"), ("A3", "5")] {
        model.set_a1(cell, value);
    }
    for (cell, value) in [("B1", "2"), ("B2", "4"), ("B3", "6")] {
        model.set_a1(cell, value);
    }
    model.set_a1("D1", "=ERROR.TYPE(_xleta.LAMBDA)");
    model.set_a1("D2", "=LET(f,_xleta.LAMBDA,f(1))");
    model.set_a1("D3", "=ROWS(MAP(A1:A3,_xleta.LAMBDA))");
    model.set_a1("D4", "=ROWS(BYROW(A1:B3,_xleta.LET))");
    model.set_a1("D5", "=ISBLANK(LET(f,_xleta.IF,f(TRUE,A1)))");
    model.set_a1("D6", "=ARRAYTOTEXT(MAP(A1:A3,B1:B3,_xleta.CHOOSE))");
    model.set_a1("D7", "=ARRAYTOTEXT(MAP(A1:A3,{7;8;9},_xleta.IF))");
    model.set_a1("D8", "=ARRAYTOTEXT(MAP(A1:A3,B1:B3,_xleta.IFERROR))");
    model.set_a1("D9", "=ARRAYTOTEXT(MAP(A1:A3,{1;2;1},_xleta.INDEX))");
    model.set_a1("D10", "=ROWS(LET(f,_xleta.IF,f(TRUE,A1:A3)))");
    model.set_a1("D11", "=ISBLANK(LET(f,_xleta.IF,f(TRUE,(A1,B1))))");
    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), *"5");
    assert_eq!(model.get_text_a1("D2"), *"#NAME?");
    assert_eq!(model.get_text_a1("D3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("D4"), *"#VALUE!");
    assert_eq!(model.get_text_a1("D5"), *"TRUE");
    assert_eq!(model.get_text_a1("D6"), *", #VALUE!, #VALUE!");
    assert_eq!(model.get_text_a1("D7"), *"7, 8, 9");
    assert_eq!(model.get_text_a1("D8"), *"1, 3, 5");
    assert_eq!(model.get_text_a1("D9"), *"1, #REF!, 5");
    assert_eq!(model.get_text_a1("D10"), *"#VALUE!");
    assert_eq!(model.get_text_a1("D11"), *"TRUE");
}

/// A function passed by name and called through LET with a count it does not take is `#VALUE!`;
/// one that takes none runs (probes/t4450-eta-special-forms E85:F103, L1:L13).
#[test]
fn function_by_name_called_without_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LET(f,_xleta.SUM,f())");
    model.set_a1("A2", "=LET(f,_xleta.PI,f())");
    model.set_a1("A3", "=LET(f,_xleta.ROW,f())");
    model.set_a1("A4", "=LET(f,_xleta.SUM,f(1,2))");
    model.set_a1("A5", "=LET(f,_xleta.ABS,f(1,2))");
    model.set_a1("A6", "=ERROR.TYPE(LET(f,_xleta.IF,f()))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"3.141592654");
    assert_eq!(model.get_text_a1("A3"), *"3");
    assert_eq!(model.get_text_a1("A4"), *"3");
    assert_eq!(model.get_text_a1("A5"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A6"), *"3");
}

/// A wrong argument count at any link of a chained call is `#VALUE!`, ahead of an error argument;
/// an optional parameter at a later link may be omitted, but a link with no arguments is still
/// too few (probes/t4482-chained-call-arity A1:E10, -optional A1:E3; T-4482).
#[test]
fn a_chained_call_with_a_wrong_count_at_a_link_is_value_error() {
    let mut model = new_empty_model();
    model
        .new_defined_name("MYFN", None, "=LAMBDA(x,LAMBDA(y,x*y))")
        .unwrap();
    let cases = [
        ("=LAMBDA(x,LAMBDA(y,x+y))(1)()", "#VALUE!"),
        ("=LAMBDA(x,LAMBDA(y,x+y))(1)(2,3)", "#VALUE!"),
        ("=LAMBDA(x,LAMBDA(y,x+y))(1,2)(3)", "#VALUE!"),
        ("=LAMBDA(x,LAMBDA(y,x+y))()(3)", "#VALUE!"),
        ("=LAMBDA(x,LAMBDA(y,x+y))(1/0)()", "#VALUE!"),
        ("=LAMBDA(x,LAMBDA(y,x+y))(1)(NA(),2)", "#VALUE!"),
        ("=LET(f,LAMBDA(x,LAMBDA(y,x*y)),f(3)())", "#VALUE!"),
        ("=MYFN(3)(4,5)", "#VALUE!"),
        ("=MYFN(3)()", "#VALUE!"),
        ("=IFERROR(LAMBDA(x,LAMBDA(y,x+y))(1)(),\"e\")", "e"),
        ("=LAMBDA(x,LAMBDA(y,x+y))(1)(2)", "3"),
        (
            "=LAMBDA(x,LAMBDA(y,[z],x+y+IF(ISOMITTED(z),0,z)))(1)(2)",
            "3",
        ),
        (
            "=LAMBDA(x,LAMBDA(y,[z],x+y+IF(ISOMITTED(z),0,z)))(1)()",
            "#VALUE!",
        ),
        (
            "=LAMBDA(x,LAMBDA(y,[z],x+y+IF(ISOMITTED(z),0,z)))(1)(2,3)",
            "6",
        ),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}
