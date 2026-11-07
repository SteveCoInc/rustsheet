//! A cell keeps the whitespace its formula was typed with: FORMULATEXT and the formula text print
//! it, structural edits keep it, and no value changes (docs/excel-semantics/syntax.md "Formula
//! text keeps typed whitespace…", SPEC-0023).

#![allow(clippy::unwrap_used)]

use crate::expressions::types::{Area, CellReferenceIndex};
use crate::test::util::new_empty_model;

#[test]
fn whitespace_prints_back_and_leaves_values_alone() {
    let mut model = new_empty_model();
    model.set_a1("B1", "1");
    model.set_a1("B2", "2");
    let cases = [
        ("=1 + 2", "=1+2"),
        ("=SUM( B1:B2 )", "=SUM(B1:B2)"),
        ("=- 5 %", "=-5%"),
        ("=LET(x, 1, y, B2, x + y)", "=LET(x,1,y,B2,x+y)"),
        ("=LAMBDA(x, ROW(x))(B2)", "=LAMBDA(x,ROW(x))(B2)"),
        ("=ISREF( B1 )", "=ISREF(B1)"),
        ("=@ B1:B2", "=@B1:B2"),
        ("=ROWS( (B1, B2) )", "=ROWS((B1,B2))"),
    ];
    for (row, (spaced, bare)) in (1..).zip(cases) {
        model.set_a1(&format!("C{row}"), spaced);
        model.set_a1(&format!("D{row}"), bare);
        model.set_a1(&format!("E{row}"), &format!("=FORMULATEXT(C{row})"));
    }
    model.evaluate();
    for (row, (spaced, _)) in (1..).zip(cases) {
        let (c, d) = (format!("C{row}"), format!("D{row}"));
        assert_eq!(model.get_text_a1(&c), model.get_text_a1(&d), "{spaced}");
        assert_eq!(model.get_formula_a1(&c), spaced);
        assert_eq!(model.get_text_a1(&format!("E{row}")), spaced);
    }
}

#[test]
fn edits_keep_the_whitespace() {
    let mut model = new_empty_model();
    model.set_a1("B1", "4");
    model.set_a1("A1", "=Sheet1!B1 + 1");
    model.evaluate();
    assert_eq!(model.insert_rows(0, 1, 1), Ok(()));
    assert_eq!(model.rename_sheet("Sheet1", "Data"), Ok(()));
    model.evaluate();
    assert_eq!(model.get_formula_a1("A2"), "=Data!B2 + 1");
    assert_eq!(model.get_text_a1("A2"), "5");

    let source = &CellReferenceIndex {
        sheet: 0,
        row: 2,
        column: 1,
    };
    let target = &CellReferenceIndex {
        sheet: 0,
        row: 3,
        column: 3,
    };
    let area = &Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 2,
        height: 2,
    };
    let moved = model.move_cell_value_to_area("=SUM( B2 ) + 1", source, target, area);
    assert_eq!(moved.unwrap(), "=SUM( D3 ) + 1");
}
