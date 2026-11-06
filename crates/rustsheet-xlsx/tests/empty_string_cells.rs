//! A cell holding the empty string stays a text cell through load and save (T-4148).
//!
//! The four XML shapes below are read as the text "" by the importer. That mapping is UNVERIFIED
//! against Excel (xlsx-storage.md "Empty `t=\"str\"` and empty inline-string cells", probe T-4723):
//! this test pins the round trip, not Excel's reading. ISBLANK FALSE and COUNTA counting a cell
//! holding "" are verified (references.md, criteria.md).

use std::error::Error;
use std::io::{Cursor, Read, Write};

use rustsheet_xlsx::base::types::CellType;
use rustsheet_xlsx::base::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

type TestResult = Result<(), Box<dyn Error>>;

/// `tests/basic_text.xlsx` with row 100 holding the empty-text cell shapes of poi/bug66675.xlsx.
fn book() -> Result<Vec<u8>, Box<dyn Error>> {
    let row = concat!(
        "<sheetData><row r=\"100\">",
        "<c r=\"A100\" t=\"str\"></c>",
        "<c r=\"B100\" t=\"str\"><v></v></c>",
        "<c r=\"C100\" t=\"inlineStr\"><is><t xml:space=\"preserve\"></t></is></c>",
        "<c r=\"D100\" t=\"inlineStr\"><is><t/></is></c>",
        "</row>"
    );
    let bytes = std::fs::read("tests/basic_text.xlsx")?;
    let mut src = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        if file.name()? == "xl/worksheets/sheet1.xml" {
            content = content.replacen("<sheetData>", row, 1);
        }
        out.start_file(
            file.name()?.into_owned(),
            zip::write::SimpleFileOptions::default(),
        )?;
        out.write_all(content.as_bytes())?;
    }
    Ok(out.finish()?.into_inner())
}

fn assert_empty_text(model: &Model, stage: &str) -> TestResult {
    for column in 1..=4 {
        assert!(
            !model.is_empty_cell(0, 100, column)?,
            "{stage}: column {column} is blank"
        );
        assert_eq!(
            model.get_cell_type(0, 100, column)?,
            CellType::Text,
            "{stage}: column {column}"
        );
    }
    Ok(())
}

#[test]
fn empty_string_cells_survive_load_and_save() -> TestResult {
    let model = Model::from_workbook(load_from_xlsx_bytes(&book()?, "case", "en", "UTC")?, "en")?;
    assert_empty_text(&model, "load")?;
    let saved = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))?.into_inner();
    let mut reloaded =
        Model::from_workbook(load_from_xlsx_bytes(&saved, "case", "en", "UTC")?, "en")?;
    assert_empty_text(&reloaded, "reload")?;
    // A cell holding "" is not blank: ISBLANK is FALSE and COUNTA counts it (references.md).
    reloaded.set_user_input(0, 101, 1, "=OR(ISBLANK(A100:D100))".to_string())?;
    reloaded.set_user_input(0, 101, 2, "=COUNTA(A100:D100)".to_string())?;
    reloaded.evaluate();
    assert_eq!(reloaded.get_formatted_cell_value(0, 101, 1)?, "FALSE");
    assert_eq!(reloaded.get_formatted_cell_value(0, 101, 2)?, "4");
    Ok(())
}
