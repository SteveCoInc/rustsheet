#![allow(clippy::unwrap_used, reason = "tests")]
//! A defined name whose stored body is a padded bare column `T[ Col ]` loads as `#REF!`, as Excel
//! loads it (probes/t3258-name-padded-column-0..2, T-3258, T-4022); `T[ [Col] ]` is the column.

use std::io::{Cursor, Read, Write};

use rustsheet_engine::Model;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

/// `tests/example.xlsx` with a name `N` of body `body` and `SUM(N)` in `Table!H50`.
fn workbook(body: &str) -> Vec<u8> {
    let mut src =
        zip::ZipArchive::new(Cursor::new(std::fs::read("tests/example.xlsx").unwrap())).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i).unwrap();
        let name = file.name().unwrap().into_owned();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if name == "xl/workbook.xml" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "<definedNames>",
                    &format!(r#"<definedNames><definedName name="N">{body}</definedName>"#),
                )
                .into_bytes();
        } else if name == "xl/worksheets/sheet5.xml" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "</sheetData>",
                    r#"<row r="50"><c r="H50"><f>SUM(N)</f></c></row></sheetData>"#,
                )
                .into_bytes();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&bytes).unwrap();
    }
    out.finish().unwrap().into_inner()
}

fn value(body: &str) -> String {
    let workbook = load_from_xlsx_bytes(&workbook(body), "case", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    let sheet = model
        .workbook
        .get_worksheet_names()
        .iter()
        .position(|n| n == "Table")
        .unwrap();
    model.get_formatted_cell_value(sheet as u32, 50, 8).unwrap()
}

#[test]
fn padded_bare_column_names_load_as_ref() {
    for body in ["Table1[ Year ]", "Table1[ Year]", "Table1[Year ]"] {
        assert_eq!(value(body), "#REF!", "{body}");
    }
}

#[test]
fn bracketed_padded_column_name_is_the_column() {
    assert_eq!(value("Table1[ [Year] ]"), "5935");
    assert_eq!(value("Table1[Year]"), "5935");
}
