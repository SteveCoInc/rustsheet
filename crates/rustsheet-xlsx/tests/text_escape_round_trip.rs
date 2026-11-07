//! Text with a carriage return, a tab, a control character or a literal `_x000D_` keeps its value
//! through load and save (T-4150), and an inline string's `_xHHHH_` escapes decode like a shared
//! string's (poi/60289.xlsx TABLE!B2 stores `Rich Text_x000d_\nTest` inline).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
// LINT-DEBT-END

use std::error::Error;
use std::io::{Cursor, Read, Write};

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

type TestResult = Result<(), Box<dyn Error>>;

fn reload(model: &Model) -> Result<Model, Box<dyn Error>> {
    let saved = save_xlsx_to_writer(model, Cursor::new(Vec::new()))?.into_inner();
    let workbook = load_from_xlsx_bytes(&saved, "m", "en", "UTC")?;
    Ok(Model::from_workbook(workbook, "en")?)
}

#[test]
fn text_with_cr_tab_control_and_escape_lookalike_survives_save_and_load() -> TestResult {
    let mut model = Model::new_empty("m", "en", "UTC", "en")?;
    let texts = [
        "Rich Text\r\nTest",
        "a\tb",
        "x\u{1}y",
        "_x000D_",
        "lit _x005F_ and _x000d_",
    ];
    for (i, text) in texts.iter().enumerate() {
        model.set_user_input(0, i as i32 + 1, 1, (*text).to_string())?;
    }
    let again = reload(&model)?;
    for (i, text) in texts.iter().enumerate() {
        assert_eq!(
            again.get_formatted_cell_value(0, i as i32 + 1, 1)?,
            *text,
            "{text:?}"
        );
    }
    Ok(())
}

/// `tests/basic_text.xlsx` with A1 turned into an inline string holding `inline`.
fn workbook_with_inline_a1(inline: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut src = zip::ZipArchive::new(Cursor::new(std::fs::read("tests/basic_text.xlsx")?))?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i)?;
        let name = file.name()?.into_owned();
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        if name == "xl/worksheets/sheet1.xml" {
            content = content.replace(
                r#"<c r="A1" s="1" t="s"><v>0</v></c>"#,
                &format!(r#"<c r="A1" s="1" t="inlineStr"><is><t>{inline}</t></is></c>"#),
            );
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())?;
        out.write_all(content.as_bytes())?;
    }
    Ok(out.finish()?.into_inner())
}

#[test]
fn inline_string_escapes_decode_and_survive_a_round_trip() -> TestResult {
    for (stored, expected) in [
        ("Rich Text_x000d_\nTest", "Rich Text\r\nTest"),
        ("a_x0001_b", "a\u{1}b"),
        ("_x005F_x000D_", "_x000D_"),
    ] {
        let workbook = load_from_xlsx_bytes(&workbook_with_inline_a1(stored)?, "m", "en", "UTC")?;
        let model = Model::from_workbook(workbook, "en")?;
        assert_eq!(
            model.get_formatted_cell_value(0, 1, 1)?,
            expected,
            "{stored}"
        );
        let again = reload(&model)?;
        assert_eq!(
            again.get_formatted_cell_value(0, 1, 1)?,
            expected,
            "{stored}"
        );
    }
    Ok(())
}
