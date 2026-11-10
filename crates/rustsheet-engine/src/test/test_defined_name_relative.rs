#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A relative name is stored relative to A1 and read at the same offset from the cell that uses
// it (docs/excel-semantics/references.md "Relative references in defined names", gen-m8a case
// 051f538767ab0cfb, T-1639). The bodies are spelled as xlsx stores them, with no `=`.
#[test]
fn relative_name_is_read_from_the_using_cell() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet2", "Jan").unwrap();
    for (name, formula) in [
        ("rel_same", "Jan!A1"),
        ("rel_right", "Jan!B1"),
        ("rel_down", "Jan!A2"),
        ("rel_col_abs", "Jan!$A1"),
        ("rel_row_abs", "Jan!A$1"),
        ("rel_box", "Jan!A1:B2"),
        ("rel_col", "Jan!A:A"),
        ("f_rel_sum", "=rel_same+rel_right"),
    ] {
        model.define_name(name, None, formula).unwrap();
    }
    for (cell, value) in [
        ("A1", "1"),
        ("A6", "2"),
        ("K1", "10"),
        ("L1", "20"),
        ("K2", "30"),
        ("K6", "40"),
        ("K7", "50"),
        ("L6", "60"),
        ("L7", "70"),
    ] {
        model.set_a1(&format!("Jan!{cell}"), value);
    }
    model.set_a1("K1", "=rel_same+1");
    model.set_a1("K2", "=rel_right");
    model.set_a1("K3", "=rel_col_abs");
    model.set_a1("K4", "=rel_row_abs");
    model.set_a1("K5", "=f_rel_sum");
    model.set_a1("K6", "=rel_down");
    model.set_a1("M6", "=SUM(rel_box)");
    model.set_a1("M7", "=ROWS(rel_box)*10+COLUMNS(rel_box)");
    model.set_a1("K8", "=ISREF(rel_same)");
    model.set_a1("K9", "=SHEET(rel_same)");
    model.set_a1("K10", "=SUM(rel_col)");
    model.set_a1("L6", "=SUM(INDIRECT(\"rel_box\"))");
    model.set_a1("A6", "=rel_col_abs");
    model.evaluate();
    // From K1: Jan!K1; from K2: Jan!L2 (empty); from K3: Jan!$A3 (empty); from K4: Jan!K$1.
    assert_eq!(model.get_text_a1("K1"), "11");
    assert_eq!(model.get_text_a1("K2"), "0");
    assert_eq!(model.get_text_a1("K3"), "0");
    assert_eq!(model.get_text_a1("K4"), "10");
    // From K5: Jan!K5 + Jan!L5, both empty.
    assert_eq!(model.get_text_a1("K5"), "0");
    // From K6: Jan!K7.
    assert_eq!(model.get_text_a1("K6"), "50");
    // From M6: Jan!M6:N7, empty; from M7: still 2x2.
    assert_eq!(model.get_text_a1("M6"), "0");
    assert_eq!(model.get_text_a1("M7"), "22");
    assert_eq!(model.get_text_a1("K8"), "TRUE");
    assert_eq!(model.get_text_a1("K9"), "2");
    // From K10: Jan!K:K.
    assert_eq!(model.get_text_a1("K10"), "130");
    // From L6: Jan!L6:M7. INDIRECT of a relative name is not probed; the engine reads it from
    // the INDIRECT's cell like any other use.
    assert_eq!(model.get_text_a1("L6"), "130");
    // From A6: Jan!$A6.
    assert_eq!(model.get_text_a1("A6"), "2");
}

// An absolute name is the same cell wherever it is used.
#[test]
fn absolute_name_ignores_the_using_cell() {
    let mut model = new_empty_model();
    model.define_name("abs", None, "Sheet1!$A$1").unwrap();
    model.set_a1("A1", "7");
    model.set_a1("C5", "=abs*2");
    model.evaluate();
    assert_eq!(model.get_text_a1("C5"), "14");
}

// Used on the last row, a relative name points past the grid. Excel's answer there is not probed
// (references.md "Relative references in defined names": it may wrap to row 1); the engine reads
// the cells past the grid as empty, with no error and no panic.
#[test]
fn relative_name_past_the_last_row_is_bounded() {
    let mut model = new_empty_model();
    model.define_name("rel_down", None, "Sheet1!A2").unwrap();
    model
        .define_name("rel_tall", None, "Sheet1!$A1:$A1048576")
        .unwrap();
    model.set_a1("A1", "5");
    model.set_a1("C1048576", "=rel_down");
    model.set_a1("D1048576", "=SUM(rel_tall)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1048576"), "0");
    assert_eq!(model.get_text_a1("D1048576"), "0");
}

// A name whose corners have different anchors (`$B$5:$B1`) used above its anchor row is the range
// between its corners, B1:B5, not an upside-down B5:B1.
#[test]
fn relative_name_corners_stay_in_order() {
    let mut model = new_empty_model();
    model.define_name("inv", None, "Sheet1!$B$5:$B1").unwrap();
    for row in 1..=5 {
        model.set_a1(&format!("B{row}"), &row.to_string());
    }
    model.set_a1("A1", "=ROWS(inv)");
    model.set_a1("C1", "=SUM(inv)");
    model.set_a1("D1", "=INDEX(inv,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "5");
    assert_eq!(model.get_text_a1("C1"), "15");
    assert_eq!(model.get_text_a1("D1"), "1");
}

/// The stored formula of each of `names` after `edit` on sheet `Data` (index 1).
fn names_after(names: &[(&str, &str)], edit: impl FnOnce(&mut crate::Model)) -> Vec<String> {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet2", "Data").unwrap();
    for (name, formula) in names {
        model.define_name(name, None, formula).unwrap();
    }
    edit(&mut model);
    let list = model.get_defined_name_list();
    names
        .iter()
        .map(|(name, _)| {
            list.iter()
                .find(|(n, _, _)| n == name)
                .map(|(_, _, formula)| formula.clone())
                .unwrap()
        })
        .collect()
}

// A row or column insert or delete leaves a name's relative rows and columns as they are and
// moves only its absolute ones (references.md "Relative references in a name";
// probes/sheet-ops/t4747-name-relative-*, t4760-name-*, excel-graph, T-4760). A name all
// absolute moves with its cells (T-4702).
#[test]
fn row_and_column_edits_move_only_absolute_parts_of_a_name() {
    let names = [
        ("Rel", "Data!A5"),
        ("RowRel", "Data!$A5"),
        ("RowAbs", "Data!A$5"),
        ("Abs", "Data!$E$5"),
        ("Box", "Data!$A$3:A6"),
    ];
    let insert_row = names_after(&names, |m| m.insert_rows(1, 4, 1).unwrap());
    assert_eq!(
        insert_row,
        [
            "Data!A5",
            "Data!$A5",
            "Data!A$6",
            "Data!$E$6",
            "Data!$A$3:A6"
        ]
    );
    // A relative row on the deleted line is left alone, not `#REF!`.
    let delete_row = names_after(&names, |m| m.delete_rows(1, 5, 1).unwrap());
    assert_eq!(
        delete_row,
        ["Data!A5", "Data!$A5", "#REF!", "#REF!", "Data!$A$3:A6"]
    );
    // `Box` becomes `$B$3:A6`, whose corners print in order (the same cells, B3:A6 is A3:B6).
    let insert_column = names_after(&names, |m| m.insert_columns(1, 1, 1).unwrap());
    assert_eq!(
        insert_column,
        [
            "Data!A5",
            "Data!$B5",
            "Data!A$5",
            "Data!$F$5",
            "Data!A$3:$B6"
        ]
    );
    let delete_column = names_after(&names, |m| m.delete_columns(1, 2, 1).unwrap());
    assert_eq!(
        delete_column,
        [
            "Data!A5",
            "Data!$A5",
            "Data!A$5",
            "Data!$D$5",
            "Data!$A$3:A6"
        ]
    );
}

// Used from the edited sheet itself, the name still reads its literal address after the edit
// (probes/sheet-ops/t4760-name-relative-own-sheet-insert-row, excel-graph: C2 50, C21 240).
#[test]
fn relative_name_used_from_the_edited_sheet_stays() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model.define_name("Rel", None, "Data!A5").unwrap();
    for row in (1..=40).filter(|row| *row != 2 && *row != 20) {
        model.set_a1(&format!("C{row}"), &(row * 10).to_string());
    }
    model.set_a1("C2", "=Rel");
    model.set_a1("C20", "=Rel");
    model.evaluate();
    model.insert_rows(0, 4, 1).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("C2"), "50");
    assert_eq!(model.get_text_a1("C21"), "240");
}
