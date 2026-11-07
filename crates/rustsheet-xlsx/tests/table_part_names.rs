// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_possible_truncation)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used, reason = "tests")]
//! A table part may name itself by `name`, `displayName` or both; formulas reach it through whichever
//! exists (T-2208, docs/excel-semantics/xlsx-storage.md). Table parts are untrusted: never a panic.

use std::io::{Cursor, Read, Write};

use rustsheet_engine::Model;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

const TABLE_ATTRS: &str = r#"name="Table1" displayName="Table1""#;

/// `tests/example.xlsx` with its table part's name attributes replaced by `attrs` and a
/// `SUM(Table1[Year])` formula in `Table!H50`.
fn workbook(attrs: &str) -> Vec<u8> {
    workbook_with_formula(attrs, "SUM(Table1[Year])")
}

/// As [`workbook`], with `formula` in `Table!H50` and `FORMULATEXT(H50)` in `Table!H51`.
fn workbook_with_formula(attrs: &str, formula: &str) -> Vec<u8> {
    let mut src =
        zip::ZipArchive::new(Cursor::new(std::fs::read("tests/example.xlsx").unwrap())).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i).unwrap();
        let name = file.name().unwrap().into_owned();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if name == "xl/tables/table1.xml" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(TABLE_ATTRS, attrs)
                .into_bytes();
        } else if name == "xl/worksheets/sheet5.xml" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "</sheetData>",
                    &format!(
                        r#"<row r="50"><c r="H50"><f>{formula}</f></c></row><row r="51"><c r="H51"><f>FORMULATEXT(H50)</f></c></row></sheetData>"#
                    ),
                )
                .into_bytes();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&bytes).unwrap();
    }
    out.finish().unwrap().into_inner()
}

fn cell(bytes: &[u8], row: i32) -> Result<String, String> {
    let workbook = load_from_xlsx_bytes(bytes, "case", "en", "UTC").map_err(|e| e.to_string())?;
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    let sheet = model
        .workbook
        .get_worksheet_names()
        .iter()
        .position(|n| n == "Table")
        .unwrap();
    Ok(model
        .get_formatted_cell_value(sheet as u32, row, 8)
        .unwrap())
}

#[test]
fn both_attributes_resolve() {
    assert_eq!(cell(&workbook(TABLE_ATTRS), 50).unwrap(), "5935");
}

#[test]
fn display_name_alone_names_the_table() {
    assert_eq!(
        cell(&workbook(r#"displayName="Table1""#), 50).unwrap(),
        "5935"
    );
}

#[test]
fn name_alone_names_the_table() {
    assert_eq!(cell(&workbook(r#"name="Table1""#), 50).unwrap(), "5935");
}

#[test]
fn neither_attribute_is_an_import_error() {
    assert!(cell(&workbook(""), 50)
        .unwrap_err()
        .contains("missing table name"));
}

#[test]
fn odd_display_names_never_panic() {
    for attrs in [
        r#"name="Table1" displayName="""#,
        r#"displayName="""#,
        r#"displayName="Таблица1""#,
        r#"name="Table1" displayName="Таблица1""#,
        r#"name="Table1" displayName="Table1" displayName="Table1""#,
    ] {
        let _ = cell(&workbook(attrs), 50);
    }
}

/// A loaded formula naming a table or column the file lacks is `#REF!` and prints `#REF!`
/// (probes/table-edits/table-unknown-name-load and -column-load, T-2197, T-2322).
#[test]
fn unknown_table_or_column_loads_as_ref() {
    for formula in ["SUM(Nope[Year])", "SUM(Table1[Nope])"] {
        let bytes = workbook_with_formula(TABLE_ATTRS, formula);
        assert_eq!(cell(&bytes, 50).unwrap(), "#REF!", "{formula}");
        assert_eq!(cell(&bytes, 51).unwrap(), "=SUM(#REF!)", "{formula}");
    }
}

/// A table named `Ⓐ`, `ⓐ` or a combining mark + `a` loads as Excel loads it while no formula
/// uses it: the table and the sheet's cells are kept (references.md "Characters of a name",
/// "Table names", T-4446, T-4753), although `add_table` refuses such a name.
#[test]
fn a_table_named_with_a_circled_letter_or_a_mark_loads() {
    for table_name in ["Ⓐ", "ⓐ", "\u{301}a"] {
        let attrs = format!(r#"name="{table_name}" displayName="{table_name}""#);
        let bytes = workbook_with_formula(&attrs, "SUM(B2:B4)");
        let workbook = load_from_xlsx_bytes(&bytes, "case", "en", "UTC").unwrap();
        let table = &workbook.tables[table_name];
        assert_eq!(
            (table.sheet_name.as_str(), table.reference.as_str()),
            ("Table", "A1:D4"),
            "{table_name}"
        );
        assert_eq!(cell(&bytes, 50).unwrap(), "5935", "{table_name}");
    }
}

/// A loaded structured reference to such a table: `ⓐ[Year]` reads it, as Excel's `SUM(ⓐ[h])`
/// does; Excel refuses the workbook at load for `Ⓐ[h]` and a first mark, which the engine loads as
/// a parse error (`#ERROR!`), its stand-in for every refused name (references.md "Characters of a
/// name", T-4753).
#[test]
fn a_structured_reference_to_a_circled_or_marked_table_name() {
    for (table_name, value) in [("ⓐ", "5935"), ("Ⓐ", "#ERROR!"), ("\u{301}a", "#ERROR!")] {
        let attrs = format!(r#"name="{table_name}" displayName="{table_name}""#);
        let formula = format!("SUM({table_name}[Year])");
        let bytes = workbook_with_formula(&attrs, &formula);
        assert_eq!(cell(&bytes, 50).unwrap(), value, "{table_name}");
        assert_eq!(
            cell(&bytes, 51).unwrap(),
            format!("={formula}"),
            "{table_name}"
        );
    }
}
