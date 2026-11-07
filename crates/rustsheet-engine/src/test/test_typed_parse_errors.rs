//! T-3213: Excel refuses a typed formula that does not parse (Graph PATCH 400), and the cell keeps
//! its content; it closes a call's missing `)` but not a group's (probes/t3213-typed-parse-errors).
//! Stored text keeps today's parse-error cell.

use crate::test::util::{new_empty_model, typed_r1c1};
use crate::typed_input::open_parenthesis_is_a_call;
use crate::TYPED_FORMULA_DOES_NOT_PARSE;

#[test]
fn typed_formulas_that_do_not_parse_are_refused() {
    let mut model = new_empty_model();
    model._set("A1", "7");
    for formula in [
        "=1+",
        "=(1",
        "=1)",
        "=+",
        "=1 2",
        "=\"a",
        "=NoSuchTable[Col]",
    ] {
        let result = model.set_typed_user_input(0, 1, 1, formula.to_string());
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{formula}: {message}"
        );
    }
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
}

#[test]
fn typed_calls_missing_their_parenthesis_are_closed() {
    let mut model = new_empty_model();
    for (row, formula) in [(1, "=SUM(1"), (2, "=SUM(1,,"), (3, "=LOG10(100")] {
        let result = model.set_typed_user_input(0, row, 1, formula.to_string());
        assert_eq!(result, Ok(()), "{formula}");
    }
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=SUM(1)");
    assert_eq!(model._get_formula("A2"), "=SUM(1,,)");
    assert_eq!(model._get_text("A2"), "1");
    assert_eq!(model._get_text("A3"), "2");
}

#[test]
fn stored_formulas_that_do_not_parse_keep_their_text() {
    let mut model = new_empty_model();
    model._set("A1", "=1+");
    model._set("A2", "=(1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "1");
}

#[test]
fn only_a_calls_open_parenthesis_is_closed() {
    for (formula, is_call) in [
        ("SUM(1", true),
        ("LOG10(2", true),
        ("SUM(1,,", true),
        ("\"(\"&SUM(1", true),
        ("'a(b'!A1+SUM(1", true),
        ("T[(x]+SUM(1", true),
        ("(1", false),
        ("1+(2", false),
        // Two open: Excel refuses them (probes/t3283-typed-missing-paren).
        ("SUM(1+(2", false),
        ("SUM((1", false),
        ("SUM(SUM(1", false),
        ("ABS(-(1", false),
        ("SUM(1,(2", false),
        ("LAMBDA(x,x)(1", false),
        ("SUM(1)", false),
        ("\"SUM(\"", false),
    ] {
        assert_eq!(open_parenthesis_is_a_call(formula), is_call, "{formula}");
    }
}

/// T-3283 (probes/t3283-typed-double-star, t3213-typed-parse-errors A5): typed, `**` in a number is
/// its exponent marker `E`, and `**` anywhere an `E` would not be is refused. Stored text is unprobed
/// and reads `**` as before.
#[test]
fn typed_double_star_in_a_number_is_its_exponent() {
    let mut model = new_empty_model();
    model._set("Z1", "2");
    for (row, typed, formula, value) in [
        (1, "=1**2", "=100", "100"),
        (2, "=2**3", "=2000", "2000"),
        (3, "=1.5**2", "=150", "150"),
        (4, "=2**-1", "=0.2", "0.2"),
        (5, "=-2**2", "=-200", "-200"),
        (6, "=2**3*2", "=2000*2", "4000"),
        (7, "=SUM(1**2)", "=SUM(100)", "100"),
    ] {
        let result = model.set_typed_user_input(0, row, 1, typed.to_string());
        assert_eq!(result, Ok(()), "{typed}");
        model.evaluate();
        assert_eq!(
            model.get_cell_formula(0, row, 1).unwrap().as_deref(),
            Some(formula),
            "{typed}"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, row, 1).unwrap(),
            value,
            "{typed}"
        );
    }
    for typed in [
        "=1**",
        "=Z1**2",
        "=1***2",
        "=2** 3",
        "=(2)**3",
        "=2**0.5",
        "=1E2**1",
        "=\"a\"**2",
        "=2**3**2",
    ] {
        let result = model.set_typed_user_input(0, 9, 1, typed.to_string());
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{typed}: {message}"
        );
    }
    // Stored, `**` is not read as an exponent: the text stays a parse error (unprobed).
    model._set("A10", "=1**2");
    model.evaluate();
    assert_eq!(model._get_text("A10"), "#ERROR!");
}

/// T-3338: a typed formula past Excel's sign cap is refused as one that does not parse, the cell
/// left blank, as Excel's Graph PATCH 400 leaves it (probes/t3266-minus-paren-1023-typed,
/// -minus-paren2-, -plus-paren-, -minus-abs-1023-typed: A1 blank, `ISFORMULA(A1)` FALSE); one
/// sign fewer is accepted.
#[test]
fn typed_formulas_past_the_sign_cap_are_refused_as_not_parsing() {
    for (sign, operand) in [("-", "(1)"), ("-", "((1))"), ("+", "(1)"), ("-", "ABS(1)")] {
        let mut model = new_empty_model();
        let typed = |count: usize| format!("={}{operand}", sign.repeat(count));
        let result = model.set_typed_user_input(0, 1, 1, typed(1023));
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{sign}{operand}: {message}"
        );
        assert_eq!(model.set_typed_user_input(0, 2, 1, typed(1022)), Ok(()));
        model._set("B1", "=ISFORMULA(A1)");
        model.evaluate();
        assert_eq!(model._get_text("A1"), "");
        assert_eq!(model._get_text("B1"), "FALSE");
        // An even count of minus signs: 1 either way.
        assert_eq!(model._get_text("A2"), "1", "{sign}{operand}");
    }
}

/// T-3262: Graph PATCH refuses a typed `ANCHORARRAY(` call at any depth (probes/t3143-anchor-typed-*)
/// while the `#` operator is accepted (probes/t3143-anchor-inner-64-typed); stored text keeps it.
#[test]
fn typed_anchorarray_call_is_refused_but_the_hash_operator_is_not() {
    let mut model = new_empty_model();
    model._set("A1", "7");
    for formula in ["=ANCHORARRAY(C1)", "=SUM(ANCHORARRAY(C1))"] {
        let result = model.set_typed_user_input(0, 1, 1, formula.to_string());
        assert!(result.is_err(), "{formula}");
    }
    assert_eq!(model._get_text("A1"), "7");
    assert_eq!(
        model.set_typed_user_input(0, 2, 1, "=SUM(C1#)".to_string()),
        Ok(())
    );
    model._set("A3", "=ANCHORARRAY(C1)");
    assert_eq!(model._get_formula("A3"), "=C1#");
}

/// T-3689: typed past Excel's 256-parenthesis cap or its 65-call cap, a formula is refused as one
/// that does not parse and the cell keeps its content, as Excel's Graph PATCH 400 leaves it
/// (probes/t3172-paren-257-typed, t3266-nested-paren-257-typed, t3097-nesting-*-66-typed: A1
/// blank, `ISFORMULA(A1)` FALSE); one level fewer is accepted.
#[test]
fn typed_formulas_past_the_parenthesis_or_call_cap_are_refused_as_not_parsing() {
    let nested = |open: &str, close: &str, levels: usize| {
        format!("={}1{}", open.repeat(levels), close.repeat(levels))
    };
    for (open, close, cap) in [("(", ")", 256), ("-(", ")", 256), ("ABS(", ")", 65)] {
        let mut model = new_empty_model();
        model._set("A1", "7");
        let result = model.set_typed_user_input(0, 1, 1, nested(open, close, cap + 1));
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{open}: {message}"
        );
        assert_eq!(
            model.set_typed_user_input(0, 2, 1, nested(open, close, cap)),
            Ok(()),
            "{open}"
        );
        model.evaluate();
        assert_eq!(model._get_text("A1"), "7", "{open}");
        // An even count of minus signs: 1 either way.
        assert_eq!(model._get_text("A2"), "1", "{open}");
    }
}

// T-3102: typed, Excel refuses a TAB or an NBSP between tokens (Graph PATCH 400, the cell left
// blank; T-2949, probes/t3102-r1c1-ws-after-bang A3), in A1 and R1C1 mode. A TAB inside a string
// literal is text, and stored text keeps loading.
#[test]
fn typed_tab_or_nbsp_between_tokens_is_refused() {
    let mut model = new_empty_model();
    model._set("A1", "7");
    model._set("D1", "4");
    for formula in ["=1+\t2", "=\t1+2", "=1\t+2", "=1+\u{a0}2", "=Sheet1!\tD1"] {
        let result = model.set_typed_user_input(0, 1, 1, formula.to_string());
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{formula:?}: {message}"
        );
    }
    for (row, formula) in [(2, "=LEN(\"a\tb\")"), (3, "=1+ \n2")] {
        let result = model.set_typed_user_input(0, row, 1, formula.to_string());
        assert_eq!(result, Ok(()), "{formula:?}");
    }
    model._set("A4", "=1+\t2");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "3");
    assert_eq!(model._get_text("A4"), "3");
    assert_eq!(
        typed_r1c1(&mut model, 0, "=Sheet1! R1C4", 10, 1).as_deref(),
        Some("=Sheet1!$D$1")
    );
    assert_eq!(typed_r1c1(&mut model, 0, "=Sheet1!\tR1C4", 11, 1), None);
}

/// Excel's short R1C1 forms typed in R1C1 mode, as probes/t3212-r1c1-bare-offsets shows them
/// (FORMULATEXT of the typed cell, excel-graph, 2026-11-03, T-3212); `None` is a refusal.
#[test]
fn short_r1c1_forms_typed_in_r1c1_mode() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "S").unwrap();
    model.add_sheet("T").unwrap();
    // (typed text, row, column of the typed cell in S, the A1 text Excel shows)
    let cases: [(&str, i32, i32, Option<&str>); 27] = [
        ("=R[-8]C[-6]", 10, 8, Some("=B2")),
        ("=SUM(R2)", 11, 8, Some("=SUM($2:$2)")),
        ("=SUM(R[-9])", 12, 8, Some("=SUM(3:3)")),
        ("=SUM(C3)", 13, 8, Some("=SUM($C:$C)")),
        ("=SUM(C[-5])", 14, 8, Some("=SUM(C:C)")),
        ("=R2C", 15, 8, Some("=H$2")),
        ("=R[-12]C[-5]+RC[-100]", 16, 8, Some("=C4+XBP16")),
        ("=RC2", 18, 8, Some("=$B18")),
        ("=R2C[-5]", 19, 8, Some("=C$2")),
        ("=R[-17]C2", 20, 8, Some("=$B3")),
        ("=SUM(R2C:R4C)", 22, 8, Some("=SUM(H$2:H$4)")),
        ("=rc[-6]", 23, 8, Some("=B23")),
        ("=ROW(RC)", 24, 8, Some("=ROW(H24)")),
        ("=ROWS(R)", 26, 8, Some("=ROWS(26:26)")),
        ("=COLUMNS(C)", 27, 8, Some("=COLUMNS(H:H)")),
        ("=T!RC", 28, 8, Some("=T!H28")),
        ("=LET(rc,5,rc+1)", 30, 8, None),
        ("=LET(r,5,r+1)", 31, 8, None),
        ("=LAMBDA(c,c+1)(2)", 33, 8, None),
        ("=LET(r1c1,5,r1c1+1)", 35, 8, None),
        ("=LET(x,5,x+1)", 36, 8, Some("=LET(x,5,x+1)")),
        ("=R0C1", 38, 8, Some("=R0C1")),
        ("=SUM(R[-29]:R[-27])", 40, 8, Some("=SUM(11:13)")),
        ("=SUM(C2:C3)", 41, 8, Some("=SUM($B:$C)")),
        ("=R[-100]C[-7]", 45, 8, Some("=A1048521")),
        ("=RC[16384]", 46, 8, None),
        ("=R2C16385", 47, 8, None),
    ];
    for (typed, row, column, a1) in cases {
        assert_eq!(
            typed_r1c1(&mut model, 0, typed, row, column).as_deref(),
            a1,
            "{typed}"
        );
    }
    // Quoted text and bracketed parts are copied as they are.
    assert_eq!(
        typed_r1c1(&mut model, 0, "=\"RC\"&'T'!RC", 1, 1).as_deref(),
        Some("=\"RC\"&T!A1")
    );
}

// T-4299: typed, a LET name or a LAMBDA parameter written with the `_xlpm.` prefix is refused in
// any letter case, and a use of it spelled with the prefix is the undefined name, its prefix
// written back lowercase (probes/t4299-typed-xlpm-extrapolations A1:A5).
#[test]
fn typed_xlpm_declarations_are_refused_in_any_case_and_uses_read_back_lowercase() {
    let mut model = new_empty_model();
    for formula in [
        "=LET(_xlpm.x,1,_xlpm.x+1)",
        "=LET(_XLPM.x,1,_XLPM.x+1)",
        "=LAMBDA(_xlpm.x,_xlpm.x*2)(3)",
    ] {
        let result = model.set_typed_user_input(0, 1, 1, formula.to_string());
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{formula}: {message}"
        );
    }
    for (row, formula) in [
        (2, "=LET(x,2,_XLPM.x*3)"),
        (3, "=LAMBDA(x,_xlpm.x*2)(3)"),
        (4, "=LAMBDA(x,x*2)(3)"),
    ] {
        let result = model.set_typed_user_input(0, row, 1, formula.to_string());
        assert_eq!(result, Ok(()), "{formula}");
    }
    model._set("B2", "=FORMULATEXT(A2)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "#NAME?");
    assert_eq!(model._get_text("B2"), "=LET(x,2,_xlpm.x*3)");
    assert_eq!(model._get_text("A3"), "#NAME?");
    assert_eq!(model._get_text("A4"), "6");
}
