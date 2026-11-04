#![allow(clippy::unwrap_used)]

use crate::expressions::parser::TOO_MANY_NESTED_CALLS_MESSAGE;
use crate::test::util::new_empty_model;

fn nested_abs(calls: usize) -> String {
    format!("={}1{}", "ABS(".repeat(calls), ")".repeat(calls))
}

// Excel's `names/add` accepts a body of 65 nested calls and refuses 66 (probes/t3264-api-name-65,
// t3264-api-name-66, T-3264): the name is then undefined, so a cell using it is #NAME?.
#[test]
fn a_name_defined_through_the_api_takes_65_nested_calls_and_not_66() {
    let mut model = new_empty_model();
    model.define_name("Deep", None, &nested_abs(65)).unwrap();
    let refused = model
        .define_name("Deeper", None, &nested_abs(66))
        .unwrap_err();
    assert!(refused.contains(TOO_MANY_NESTED_CALLS_MESSAGE), "{refused}");
    model._set("A1", "=Deep");
    model._set("A2", "=Deeper");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"#NAME?");
}

// A loaded body is held to the cap by the importer, which refuses the workbook (T-3143), so the
// loading path itself does not refuse it.
#[test]
fn a_loaded_name_is_not_refused_here() {
    let mut model = new_empty_model();
    model
        .define_loaded_name("Deeper", None, &nested_abs(66))
        .unwrap();
}

// A cell's calls and the calls of a name it uses do not add up: 65 calls around a name whose body
// nests 65 evaluate (probes/t3264-name65-cell65 and -typed, T-3264).
#[test]
fn a_cells_calls_and_its_names_do_not_add_up() {
    let mut model = new_empty_model();
    model.define_name("N", None, &nested_abs(65)).unwrap();
    let cell = format!("={}N{}", "ABS(".repeat(65), ")".repeat(65));
    model._set("A1", &cell);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1");
}

// Graph's `worksheets('S')/names/add` holds a sheet-scoped name to the same cap: 65 calls are
// added (S!N is 1 from another sheet) and 66 refused (probes/t4014-sheet-api-name-65, -66, T-4014).
#[test]
fn a_sheet_scoped_name_takes_65_nested_calls_and_not_66() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.define_name("Deep", Some(0), &nested_abs(65)).unwrap();
    let refused = model
        .define_name("Deeper", Some(0), &nested_abs(66))
        .unwrap_err();
    assert!(refused.contains(TOO_MANY_NESTED_CALLS_MESSAGE), "{refused}");
    model._set("Sheet2!A1", "=Sheet1!Deep");
    model._set("Sheet2!A2", "=Sheet1!Deeper");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!A1"), *"1");
    assert_eq!(model._get_text("Sheet2!A2"), *"#NAME?");
}
