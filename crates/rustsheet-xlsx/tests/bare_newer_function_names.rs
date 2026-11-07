#![allow(clippy::unwrap_used, reason = "tests")]
//! A function added after Excel 2007 is recognised in a file only under its `_xlfn.` prefix:
//! `PERCENTOF(1,2)` stored bare is `#NAME?` and `_xlfn.PERCENTOF(1,2)` is 0.5
//! (docs/excel-semantics/xlsx-storage.md "Newer functions need `_xlfn.` in the file",
//! probes/catalog/function-name-forms A1-A2, excel-graph, T-0040; T-1089). Typed text keeps
//! reading the bare name as the function.

use std::io::{Cursor, Read, Write};

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

/// The `<f>` texts of rows 2.. of column A, in order, after the value 1 in A1.
const FORMULAS: [&str; 8] = [
    "PERCENTOF(1,2)",
    "_xlfn.PERCENTOF(1,2)",
    "percentof(1,2)",
    "SUM(1,2)",
    "_xlfn.SUM(1,2)",
    "IFERROR(ERROR.TYPE(XLOOKUP(1,A1,A1)),0)",
    "IFERROR(ERROR.TYPE(_xlfn.XLOOKUP(1,A1,A1)),0)",
    "IFERROR(ERROR.TYPE(_xlfn._xlws.SORT(A1)),0)",
];

/// An xlsx file whose first sheet holds 1 in A1 and [`FORMULAS`] from A2 down, as written.
fn workbook() -> Vec<u8> {
    let mut model = Model::new_empty("bare", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "1".to_string()).unwrap();
    let saved = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    let mut rows = String::new();
    for (f, row) in FORMULAS.iter().zip(2..) {
        rows.push_str(&format!(
            r#"<row r="{row}"><c r="A{row}"><f>{f}</f></c></row>"#
        ));
    }
    let mut src = zip::ZipArchive::new(Cursor::new(saved)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i).unwrap();
        let name = file.name().unwrap().into_owned();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if name == "xl/worksheets/sheet1.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert!(xml.contains("</sheetData>"), "{xml}");
            bytes = xml
                .replace("</sheetData>", &format!("{rows}</sheetData>"))
                .into_bytes();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&bytes).unwrap();
    }
    out.finish().unwrap().into_inner()
}

fn load(bytes: &[u8]) -> Model {
    let workbook = load_from_xlsx_bytes(bytes, "bare", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    model
}

fn values(model: &Model) -> Vec<String> {
    (2..=9)
        .map(|row| model.get_formatted_cell_value(0, row, 1).unwrap())
        .collect()
}

/// A bare newer name is `#NAME?`, in any case; the prefixed one calculates, and a classic
/// function calculates bare or prefixed (function-name-forms-2 A3 `_xlfn.SUM(1,2)` = 3).
const EXPECTED: [&str; 8] = ["#NAME?", "0.5", "#NAME?", "3", "3", "5", "0", "0"];

#[test]
fn a_bare_newer_function_name_in_a_file_is_name_error() {
    let model = load(&workbook());
    assert_eq!(values(&model), EXPECTED);
    // The call keeps the name the file wrote. A plain `<f>` is a legacy formula, which adds `@`
    // to a call of a name (static_analysis.rs `add_legacy_intersection`); whether Excel shows
    // `=@PERCENTOF(1,2)` for an unknown one is unprobed, so only the call is checked.
    let shown = model.get_cell_formula(0, 2, 1).unwrap().unwrap();
    assert!(shown.ends_with("PERCENTOF(1,2)"), "{shown}");
    assert_eq!(
        model.get_cell_formula(0, 3, 1).unwrap().unwrap(),
        "=PERCENTOF(1,2)"
    );
}

/// The unknown call survives the engine's own storage and a save: the bare name is written
/// bare again, the function with its prefix, so a reload reads each the same.
#[test]
fn a_bare_newer_function_name_round_trips() {
    let model = load(&workbook());
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    assert_eq!(values(&reloaded), EXPECTED);

    let saved = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(saved.clone())).unwrap();
    let mut xml = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    assert!(xml.contains("<f>PERCENTOF(1,2)</f>"), "{xml}");
    assert!(xml.contains("<f>_xlfn.PERCENTOF(1,2)</f>"), "{xml}");
    assert_eq!(values(&load(&saved)), EXPECTED);
}

/// Typed and API text is not file text: a bare newer name there is the function.
#[test]
fn a_typed_bare_newer_function_name_is_the_function() {
    let mut model = load(&workbook());
    model
        .set_user_input(0, 2, 1, "=PERCENTOF(1,2)".to_string())
        .unwrap();
    model
        .set_loaded_user_input(0, 3, 1, "=PERCENTOF(1,2)".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "0.5");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "0.5");
}
