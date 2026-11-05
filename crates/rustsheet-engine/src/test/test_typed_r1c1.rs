use crate::test::util::{new_empty_model, typed_r1c1};

/// A model whose sheets are `Sheet1` and `S`.
fn model() -> crate::Model {
    let mut model = new_empty_model();
    model.add_sheet("S").unwrap();
    model
}

#[test]
fn absolute_and_relative_references_resolve_against_the_cell() {
    let mut model = model();
    let mut a1 = |text: &str, row, column| typed_r1c1(&mut model, 0, text, row, column);
    assert_eq!(a1("=R1C2+1", 3, 1).as_deref(), Some("=$B$1+1"));
    assert_eq!(a1("=R[-1]C[1]", 4, 1).as_deref(), Some("=B3"));
    assert_eq!(
        a1("=SUM(R1C1:R[1]C[2])", 5, 1).as_deref(),
        Some("=SUM($A$1:C6)")
    );
    assert_eq!(a1("=S!R1C2", 1, 1).as_deref(), Some("=S!$B$1"));
}

#[test]
fn constants_are_typed_as_in_a1_mode_and_unparsable_formulas_are_refused() {
    let mut model = model();
    model
        .set_typed_r1c1_user_input(0, 1, 1, "abc".to_string())
        .unwrap();
    assert_eq!(model._get_text("A1"), "abc");
    assert_eq!(typed_r1c1(&mut model, 0, "=$A$1", 2, 1), None);
    assert_eq!(typed_r1c1(&mut model, 0, "=1+", 3, 1), None);
    // A refused formula leaves the cell as it was.
    assert_eq!(model._get_text("A1"), "abc");
    assert!(model
        .set_typed_r1c1_user_input(0, 1, 1, "=1+".to_string())
        .is_err());
    assert_eq!(model._get_text("A1"), "abc");
}

/// A LET name or LAMBDA parameter spelled like an A1 cell is no reference typed in R1C1 mode,
/// and Excel binds it (probes/t3897-r1c1-let-cell-names, excel-graph, T-3897), where typed in A1
/// mode it refuses the formula (decision 0103). The parsed formula is stored as it is, so its A1
/// text is the bare name, and it reloads from the R1C1 storage form the same (T-4288).
#[test]
fn let_and_lambda_names_spelled_like_a1_cells_bind() {
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
        ("=LET(x,5,x+1)", "6"),
    ];
    let mut model = new_empty_model();
    model._set("F1", "7");
    for (row, (typed, _)) in (1..).zip(cases) {
        assert_eq!(
            typed_r1c1(&mut model, 0, typed, row, 8).as_deref(),
            Some(typed),
            "{typed}"
        );
    }
    model.evaluate();
    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for m in [&model, &reloaded] {
        for (row, (typed, value)) in (1..).zip(cases) {
            let cell = format!("H{row}");
            assert_eq!(m._get_text(&cell), value, "{typed}");
            assert_eq!(m._get_formula(&cell), typed);
        }
    }
    // Typed in A1 mode, the A1 text of these formulas is refused, and `_xlpm.` is part of the
    // name there: no A1 text binds such a name (T-4288).
    for typed in ["=LET(f1,5,f1+1)", "=LET(_xlpm.f1,5,_xlpm.f1+1)"] {
        assert!(model
            .set_typed_user_input(0, 30, 1, typed.to_string())
            .is_err());
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
        model
            .set_typed_r1c1_user_input(0, row, 1, typed.to_string())
            .unwrap_or_else(|e| panic!("{typed}: {e}"));
    }
    model.evaluate();
    for (row, (typed, formula, value)) in (1..).zip(cases) {
        let cell = format!("A{row}");
        assert_eq!(model._get_formula(&cell), formula, "{typed}");
        assert_eq!(model._get_text(&cell), value, "{typed}");
    }
    // The quoted spelling is refused typed, in either mode.
    assert_eq!(typed_r1c1(&mut model, 0, "='f1'", 21, 1), None);
    assert!(model
        .set_typed_user_input(0, 20, 1, "='f1'".to_string())
        .is_err());
}

/// An unbound name written with `_xlpm.` keeps the prefix in its formula text, any name, from a
/// file or typed, and a bound one drops it (probes/t4099-file-xlpm-unbound-names, excel-graph,
/// T-4099); a reload from the R1C1 storage form keeps both.
#[test]
fn an_unbound_prefixed_name_keeps_its_prefix() {
    let cases = [
        ("=_xlpm.f1", "=_xlpm.f1", "#NAME?"),
        ("=_xlpm.abc+_xlpm.f1", "=_xlpm.abc+_xlpm.f1", "#NAME?"),
        ("=_xlpm.R1C1", "=_xlpm.R1C1", "#NAME?"),
        ("=_xlpm.f1(2)", "=_xlpm.f1(2)", "#NAME?"),
        ("=LET(x,1,x+_xlpm.f1)", "=LET(x,1,x+_xlpm.f1)", "#NAME?"),
        ("=ISREF(_xlpm.f1)", "=ISREF(_xlpm.f1)", "FALSE"),
        ("=LET(_xlpm.x,1,_xlpm.x+1)", "=LET(x,1,x+1)", "2"),
        ("=LET(_xlpm.f1,5,_xlpm.f1*2)", "=LET(f1,5,f1*2)", "10"),
        ("=LET(x,2,_xlpm.x*3)", "=LET(x,2,x*3)", "6"),
        ("=R1C1", "=R1C1", "#NAME?"),
    ];
    let mut model = crate::test::util::new_empty_model();
    model._set("F1", "7");
    for (row, (stored, ..)) in (1..).zip(cases) {
        model._set(&format!("A{row}"), stored);
    }
    // Typed, the prefix stays as well (A10).
    model
        .set_typed_user_input(0, 20, 1, "=_xlpm.abc".to_string())
        .unwrap_or_else(|e| panic!("{e}"));
    model.evaluate();
    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for m in [&model, &reloaded] {
        for (row, (stored, formula, value)) in (1..).zip(cases) {
            let cell = format!("A{row}");
            assert_eq!(m._get_formula(&cell), formula, "{stored}");
            assert_eq!(m._get_text(&cell), value, "{stored}");
        }
        assert_eq!(m._get_formula("A20"), "=_xlpm.abc");
    }
}

/// A file's `_xlpm.` is one mark: a doubled prefix leaves one on the name, an undefined name
/// that FORMULATEXT shows as written, and `_xlpm._xlpm.x` declares the name `_xlpm.x`, which a
/// use binds only spelled the same (probes/t4224-doubled-xlpm-prefix A1:C2, A7:C8, A13:C13,
/// excel-graph, T-4224); a reload from the R1C1 storage form and an export keep each.
#[test]
fn a_files_doubled_prefix_is_one_mark_and_the_name() {
    let cases = [
        ("=_xlpm._xlpm.f1", "=_xlpm._xlpm.f1", "#NAME?"),
        ("=_xlpm._xlpm.abc", "=_xlpm._xlpm.abc", "#NAME?"),
        ("=LET(_xlpm.x,2,_xlpm.x*3)", "=LET(x,2,x*3)", "6"),
        (
            "=LET(_xlpm._xlpm.x,1,_xlpm._xlpm.x+1)",
            "=LET(_xlpm.x,1,_xlpm.x+1)",
            "2",
        ),
        (
            "=LET(_xlpm.x,2,_xlpm._xlpm.x*3)",
            "=LET(x,2,_xlpm._xlpm.x*3)",
            "#NAME?",
        ),
    ];
    let mut model = crate::test::util::new_empty_model();
    model._set("F1", "7");
    for (row, (stored, ..)) in (1..).zip(cases) {
        model._set(&format!("A{row}"), stored);
    }
    model.evaluate();
    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for m in [&model, &reloaded] {
        for (row, (stored, formula, value)) in (1..).zip(cases) {
            let cell = format!("A{row}");
            assert_eq!(m._get_formula(&cell), formula, "{stored}");
            assert_eq!(m._get_text(&cell), value, "{stored}");
        }
    }
    // The export writes the file's text back.
    let context = crate::expressions::types::CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let exported: Vec<String> = model.parsed_formulas[0]
        .iter()
        .map(|f| crate::expressions::parser::stringify::to_excel_string(&f.evaluated, &context))
        .collect();
    for text in [
        "_xlpm._xlpm.f1",
        "_xlfn.LET(_xlpm._xlpm.x,1,_xlpm._xlpm.x+1)",
        "_xlfn.LET(_xlpm.x,2,_xlpm._xlpm.x*3)",
    ] {
        assert!(exported.iter().any(|e| e == text), "{text}: {exported:?}");
    }
}

/// Typed, in A1 or R1C1 mode, `_xlpm.` is no mark but part of the name: a doubled prefix is the
/// undefined name FORMULATEXT shows as typed, a use `_xlpm.x` is not the LET's `x`, and a LET
/// name written with it is refused (probes/t4224-doubled-xlpm-prefix A3:C6, A9:C12, A14:C14,
/// excel-graph, T-4288); a reload from the R1C1 storage form keeps each.
#[test]
fn typed_xlpm_is_part_of_the_name() {
    let cases = [
        ("=_xlpm._xlpm.f1", "#NAME?"),
        ("=_xlpm._xlpm.abc", "#NAME?"),
        ("=LET(x,2,_xlpm.x*3)", "#NAME?"),
        ("=LET(x,2,_xlpm._xlpm.x*3)", "#NAME?"),
        ("=_xlpm.abc", "#NAME?"),
    ];
    let mut model = new_empty_model();
    model._set("F1", "7");
    for (row, (typed, _)) in (1..).zip(cases) {
        model
            .set_typed_user_input(0, row, 1, typed.to_string())
            .unwrap_or_else(|e| panic!("{typed}: {e}"));
        assert_eq!(
            typed_r1c1(&mut model, 0, typed, row, 2).as_deref(),
            Some(typed),
            "{typed}"
        );
    }
    model.evaluate();
    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for m in [&model, &reloaded] {
        for (row, (typed, value)) in (1..).zip(cases) {
            for cell in [format!("A{row}"), format!("B{row}")] {
                assert_eq!(m._get_formula(&cell), typed, "{cell}");
                assert_eq!(m._get_text(&cell), value, "{cell}");
            }
        }
    }
    // A LET name written with the prefix is refused typed, in either mode (A10, A12).
    let declared = "=LET(_xlpm.x,1,_xlpm.x+1)";
    assert!(model
        .set_typed_user_input(0, 20, 1, declared.to_string())
        .is_err());
    assert_eq!(typed_r1c1(&mut model, 0, declared, 20, 2), None);
    assert_eq!(model._get_formula("A20"), "");
}
