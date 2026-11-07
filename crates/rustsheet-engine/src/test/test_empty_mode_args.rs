//! A written-empty `match_mode`/`search_mode` of XLOOKUP and XMATCH, and an empty `rows` or
//! `columns` of EXPAND, read as omitted, not as 0 (T-4871; probes/t4871-empty-mode-args,
//! excel-graph, 2026-11-07). Sheet1 A1:A3 = 1..3, B1:B3 = 10..30, C1:C3 = 3..1.

use crate::test::util::new_empty_model;

#[test]
fn empty_lookup_modes_and_expand_sizes_are_omitted() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "2"),
        ("A3", "3"),
        ("B1", "10"),
        ("B2", "20"),
        ("B3", "30"),
        ("C1", "3"),
        ("C2", "2"),
        ("C3", "1"),
    ] {
        model._set(cell, value);
    }
    let cells = [
        ("E5", "=XLOOKUP(2,A1:A3,B1:B3,,,)", "20"),
        ("E6", "=XLOOKUP(5,A1:A3,B1:B3,,,)", "#N/A"),
        ("E9", "=XLOOKUP(2.5,A1:A3,B1:B3,,-1,)", "20"),
        ("E10", "=XLOOKUP(2.5,A1:A3,B1:B3,,1,)", "30"),
        ("E14", "=XMATCH(2,A1:A3,0,)", "2"),
        ("E16", "=XMATCH(2.5,C1:C3,1,)", "1"),
        ("E17", "=XMATCH(2,C1:C3,,)", "2"),
        ("E20", "=LET(e,,XMATCH(2,A1:A3,0,e))", "2"),
        ("E21", "=ROWS(EXPAND(A1:B1,3,,0))", "3"),
        ("E22", "=COLUMNS(EXPAND(A1:B1,3,,0))", "2"),
        ("E23", "=ROWS(EXPAND(A1:A2,,3,0))", "2"),
        ("E26", "=COLUMNS(EXPAND(A1:A2,,,0))", "1"),
        ("E27", "=INDEX(EXPAND(A1:B1,3,,\"p\"),3,2)", "p"),
        ("E28", "=INDEX(EXPAND(A1:A2,,3),2,3)", "#N/A"),
        ("E31", "=ROWS(EXPAND(A1:B1,,))", "1"),
    ];
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), expected, "{cell} {formula}");
    }
}
