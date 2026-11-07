#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A bare name in the body of a name added through the API binds to its sheet-scoped definition on
// the alphabetically first sheet name, else to the workbook name, whichever sheet is active
// (probes/t4554-api-name-scoped-name-choice, -first, -last, probes/t4554-api-name-local-name-body;
// docs/excel-semantics/syntax.md "A bare name in an API-added name's body", T-4587).

/// Tabs First, S2, Last with B1 = 1, 7, 9 and B2 = 10, 70, 90, as the t4554 probes.
fn three_sheet_model() -> crate::Model {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_sheet();
    model.rename_sheet("Sheet1", "First").unwrap();
    model.rename_sheet("Sheet2", "S2").unwrap();
    model.rename_sheet("Sheet3", "Last").unwrap();
    for (sheet, b1, b2) in [("First", 1, 10), ("S2", 7, 70), ("Last", 9, 90)] {
        model.set_a1(&format!("{sheet}!B1"), &b1.to_string());
        model.set_a1(&format!("{sheet}!B2"), &b2.to_string());
    }
    model
}

/// The value `W` reads on each sheet, `W` = `=Dup` added with `active` active, after `Dup` is
/// defined in `scopes` (`None` the workbook, reading First!B2; a sheet its own B2, or B1 for
/// First, as the probes).
fn bound_values(scopes: &[Option<u32>], active: u32) -> [String; 3] {
    let mut model = three_sheet_model();
    for &scope in scopes {
        let body = match scope {
            None => "=First!$B$2",
            Some(0) => "=First!$B$1",
            Some(1) => "=S2!$B$2",
            Some(_) => "=Last!$B$2",
        };
        model.define_name("Dup", scope, body).unwrap();
    }
    model.set_active_sheet(active).unwrap();
    model.define_name("W", None, "=Dup").unwrap();
    for sheet in ["First", "S2", "Last"] {
        model.set_a1(&format!("{sheet}!D1"), "=W");
    }
    model.evaluate();
    ["First", "S2", "Last"].map(|sheet| model.get_text_a1(&format!("{sheet}!D1")))
}

#[test]
fn bare_name_binds_alphabetically_first_sheet_definition() {
    // (scopes, value on every sheet), probes/t4554-api-name-scoped-name-choice rows A..I.
    let cases: [(&[Option<u32>], &str); 9] = [
        (&[None, Some(1)], "70"),
        (&[None, Some(2)], "90"),
        (&[None, Some(0), Some(1)], "1"),
        (&[Some(0), Some(2)], "1"),
        (&[Some(2)], "90"),
        (&[Some(0)], "1"),
        (&[None, Some(1), Some(2)], "90"),
        (&[Some(1), Some(2)], "90"),
        (&[None, Some(0)], "1"),
    ];
    for (scopes, expected) in cases {
        for active in 0..3 {
            let expected = expected.to_string();
            assert_eq!(
                bound_values(scopes, active),
                [expected.clone(), expected.clone(), expected],
                "scopes {scopes:?}, active {active}"
            );
        }
    }
}

#[test]
fn bare_name_with_only_a_workbook_definition_stays_bare() {
    let mut model = three_sheet_model();
    model.define_name("Dup", None, "=First!$B$2").unwrap();
    model.set_active_sheet(1).unwrap();
    model.define_name("W", None, "=Dup*2").unwrap();
    let body = model
        .get_defined_name_list()
        .into_iter()
        .find(|(name, ..)| name == "W")
        .map(|(_, _, body)| body);
    assert_eq!(body.as_deref(), Some("=Dup*2"));
}

#[test]
fn bound_body_is_qualified_and_follows_a_sheet_rename() {
    let mut model = three_sheet_model();
    model.define_name("Dup", Some(1), "=S2!$B$2").unwrap();
    model.define_name("Dup", Some(2), "=Last!$B$2").unwrap();
    // Sheet-scoped too: Last's own `LD` = `=Dup` reads Last's (probes/t4554-api-name-local-name-body).
    model.define_name("LD", Some(2), "=Dup+1").unwrap();
    model.set_a1("First!D1", "=Last!LD");
    model.evaluate();
    assert_eq!(model.get_text_a1("First!D1"), "91");
    model.rename_sheet("Last", "Final").unwrap();
    model.evaluate();
    let body = model
        .get_defined_name_list()
        .into_iter()
        .find(|(name, ..)| name == "LD")
        .map(|(_, _, body)| body);
    assert_eq!(body.as_deref(), Some("=Final!Dup+1"));
    assert_eq!(model.get_text_a1("First!D1"), "91");
}

#[test]
fn a_let_variable_is_not_bound_to_a_sheet_name() {
    let mut model = three_sheet_model();
    model.define_name("x", Some(2), "=Last!$B$2").unwrap();
    model.define_name("W", None, "=LET(x,3,x*2)").unwrap();
    model.set_a1("First!D1", "=W");
    model.evaluate();
    assert_eq!(model.get_text_a1("First!D1"), "6");
}

#[test]
fn a_lambda_parameter_is_not_bound_to_a_sheet_name() {
    let mut model = three_sheet_model();
    model.define_name("x", Some(2), "=Last!$B$2").unwrap();
    model.define_name("W", None, "=LAMBDA(x,x*2)(4)").unwrap();
    model.set_a1("First!D1", "=W");
    model.evaluate();
    assert_eq!(model.get_text_a1("First!D1"), "8");
}

// The name order ignores letter case and puts `_` before the letters
// (probes/t4587-api-name-sheet-order-case: tabs Z, B, a, _u with B1 = 1, 2, 3, 4).
#[test]
fn sheet_name_order_ignores_case_and_puts_underscore_first() {
    let sheets = ["Z", "B", "a", "_u"];
    let mut model = new_empty_model();
    for _ in 1..sheets.len() {
        model.new_sheet();
    }
    for (index, sheet) in sheets.iter().enumerate() {
        model
            .rename_sheet(&format!("Sheet{}", index + 1), sheet)
            .unwrap();
        model.set_a1(&format!("{sheet}!B1"), &(index + 1).to_string());
    }
    // Each name's two defining sheets, by tab index.
    let pairs: [(&str, [u32; 2]); 4] = [
        ("Dup_1", [1, 2]),
        ("Dup_2", [2, 3]),
        ("Dup_3", [1, 3]),
        ("Dup_4", [0, 2]),
    ];
    for (name, scopes) in pairs {
        for scope in scopes {
            let body = format!("={}!$B$1", sheets[scope as usize]);
            model.define_name(name, Some(scope), &body).unwrap();
        }
    }
    for (row, (name, _)) in pairs.iter().enumerate() {
        let alias = format!("W{name}");
        model
            .define_name(&alias, None, &format!("={name}"))
            .unwrap();
        model.set_a1(&format!("Z!D{}", row + 1), &format!("={alias}"));
    }
    model.evaluate();
    let values = ["Z!D1", "Z!D2", "Z!D3", "Z!D4"].map(|cell| model.get_text_a1(cell));
    assert_eq!(values, ["3", "4", "4", "3"]);
}
