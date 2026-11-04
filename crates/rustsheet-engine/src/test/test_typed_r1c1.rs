use crate::r1c1_formula_to_a1;

fn a1(text: &str, row: i32, column: i32) -> Option<String> {
    let sheets = ["Sheet1".to_string(), "S".to_string()];
    r1c1_formula_to_a1(text, &sheets, "Sheet1", row, column)
}

#[test]
fn absolute_and_relative_references_resolve_against_the_cell() {
    assert_eq!(a1("=R1C2+1", 3, 1).as_deref(), Some("=$B$1+1"));
    assert_eq!(a1("=R[-1]C[1]", 3, 1).as_deref(), Some("=B2"));
    assert_eq!(
        a1("=SUM(R1C1:R[1]C[2])", 3, 1).as_deref(),
        Some("=SUM($A$1:C4)")
    );
    assert_eq!(a1("=S!R1C2", 1, 1).as_deref(), Some("=S!$B$1"));
}

#[test]
fn constants_pass_through_and_unparsable_formulas_are_refused() {
    assert_eq!(a1("(1)", 1, 1).as_deref(), Some("(1)"));
    assert_eq!(a1("=$A$1", 1, 1), None);
    assert_eq!(a1("=1+", 1, 1), None);
}

/// A LET name or LAMBDA parameter spelled like an A1 cell is no reference typed in R1C1 mode,
/// and Excel binds it (probes/t3897-r1c1-let-cell-names, excel-graph, T-3897): the A1 text writes
/// it `_xlpm.`, which the typed A1 entry binds where it refuses a bare `f1` (decision 0103).
#[test]
fn let_and_lambda_names_spelled_like_a1_cells_keep_their_prefix() {
    assert_eq!(
        a1("=LET(f1,5,f1+1)", 1, 1).as_deref(),
        Some("=LET(_xlpm.f1,5,_xlpm.f1+1)")
    );
    assert_eq!(
        a1("=LAMBDA(x,[b2],ISOMITTED(b2))(1)", 1, 1).as_deref(),
        Some("=LAMBDA(x,[_xlpm.b2],ISOMITTED(_xlpm.b2))(1)")
    );
    // Unbound, such a name is Excel's undefined name (`#NAME?`, A8:A10, A13), not the cell bare
    // A1 text would read: the A1 text writes it `_xlpm.` too (T-4040).
    assert_eq!(
        a1("=LET(f1,5,f1+1)+F1", 1, 1).as_deref(),
        Some("=LET(_xlpm.f1,5,_xlpm.f1+1)+_xlpm.F1")
    );
    // Other names stay bare.
    assert_eq!(a1("=LET(x,5,x+1)", 1, 1).as_deref(), Some("=LET(x,5,x+1)"));
    let cases = [
        ("=LET(f1,5,f1+1)", "6"),
        ("=LAMBDA(a1,a1*2)(3)", "6"),
        ("=LET(xy1,3,xy1+1)", "4"),
        ("=LET(ab12,2,f1,ab12*10,f1+ab12)", "22"),
        ("=LAMBDA(x,[b2],ISOMITTED(b2))(1)", "TRUE"),
        ("=LET(f1,LAMBDA(x,x+1),f1(2))", "3"),
        ("=LET(A1,5,A1*2)", "10"),
        ("=LET(F1,5,F1+1)", "6"),
        ("=LAMBDA(A1,A1*2)(3)", "6"),
    ];
    let mut model = crate::test::util::new_empty_model();
    model._set("F1", "7");
    for (row, (typed, _)) in (1..).zip(cases) {
        let text = a1(typed, row, 8).unwrap_or_default();
        model
            .set_typed_user_input(0, row, 8, text)
            .unwrap_or_else(|e| panic!("{typed}: {e}"));
    }
    model.evaluate();
    for (row, (typed, value)) in (1..).zip(cases) {
        let cell = format!("H{row}");
        assert_eq!(model._get_text(&cell), value, "{typed}");
        assert_eq!(model._get_formula(&cell), typed);
    }
}

/// An unbound name spelled like an A1 cell, typed in R1C1 mode, is Excel's undefined name:
/// `#NAME?`, and FORMULATEXT quotes it like a sheet name in the workbook's first spelling of the
/// name (probes/t3897-r1c1-let-cell-names A8:C10, A13:C13; probes/t4040-r1c1-unbound-cell-names
/// A7:C13, excel-graph, T-4040). A name that is no cell (`Xfe1`) stays bare, and `='f1'` typed is
/// refused in either mode (A1:A6).
#[test]
fn unbound_names_spelled_like_a1_cells_are_quoted_undefined_names() {
    let cases = [
        ("=f1", "='f1'", "#NAME?"),
        ("=F1+1", "='f1'+1", "#NAME?"),
        ("=LET(x,1,x+F1)", "=LET(x,1,x+'f1')", "#NAME?"),
        ("=LET(f1,5,f1+1)+F1", "=LET(f1,5,f1+1)+'f1'", "#NAME?"),
        ("=xy1", "='xy1'", "#NAME?"),
        ("=Xfe1", "=Xfe1", "#NAME?"),
        ("=AB12*2", "='AB12'*2", "#NAME?"),
        ("=SUM(f1,2)", "=SUM('f1',2)", "#NAME?"),
        ("=f1(2)", "='f1'(2)", "#NAME?"),
        ("=LAMBDA(x,x+F1)(1)", "=LAMBDA(x,x+'f1')(1)", "#NAME?"),
        ("=ISREF(f1)", "=ISREF('f1')", "FALSE"),
    ];
    let mut model = crate::test::util::new_empty_model();
    model._set("F1", "7");
    for (row, (typed, ..)) in (1..).zip(cases) {
        let text = a1(typed, row, 1).unwrap_or_else(|| panic!("{typed} refused"));
        model
            .set_typed_user_input(0, row, 1, text)
            .unwrap_or_else(|e| panic!("{typed}: {e}"));
    }
    model.evaluate();
    for (row, (typed, formula, value)) in (1..).zip(cases) {
        let cell = format!("A{row}");
        assert_eq!(model._get_formula(&cell), formula, "{typed}");
        assert_eq!(model._get_text(&cell), value, "{typed}");
    }
    // The quoted spelling is refused typed, in either mode.
    assert_eq!(a1("='f1'", 1, 1), None);
    assert!(model
        .set_typed_user_input(0, 20, 1, "='f1'".to_string())
        .is_err());
}
