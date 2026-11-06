//! A cell holding the empty string (`t="s"` to an empty shared string) survives load and save as
//! a text cell, not a blank one (T-4148; ISBLANK is FALSE and COUNTA counts it in Excel). These
//! shapes already round-trip; poi/bug66675's loss is merged-range cover (see the task's Handoff).

use std::error::Error;
use std::io::{Cursor, Read, Write};

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

type TestResult = Result<(), Box<dyn Error>>;

/// `tests/basic_text.xlsx` with its first shared string ("Bold", used by A1) replaced by `si` and
/// A1's cell element replaced by `cell` when given.
fn workbook_with_empty_a1(si: &str, cell: Option<&str>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut src = zip::ZipArchive::new(Cursor::new(std::fs::read("tests/basic_text.xlsx")?))?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i)?;
        let name = file.name()?.into_owned();
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        if name == "xl/sharedStrings.xml" {
            content = content.replace("<si><t>Bold</t></si>", si);
        }
        if let (Some(cell), "xl/worksheets/sheet1.xml") = (cell, name.as_str()) {
            content = content.replace(r#"<c r="A1" s="1" t="s"><v>0</v></c>"#, cell);
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())?;
        out.write_all(content.as_bytes())?;
    }
    Ok(out.finish()?.into_inner())
}

fn a1_after_round_trip(
    si: &str,
    cell: Option<&str>,
) -> Result<(Option<String>, String), Box<dyn Error>> {
    let bytes = workbook_with_empty_a1(si, cell)?;
    let workbook = load_from_xlsx_bytes(&bytes, "m", "en", "UTC")?;
    let mut model = Model::from_workbook(workbook, "en")?;
    model.evaluate();
    let saved = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))?.into_inner();
    let mut zip = zip::ZipArchive::new(Cursor::new(saved.clone()))?;
    let mut sheet = String::new();
    zip.by_name("xl/worksheets/sheet1.xml")?
        .read_to_string(&mut sheet)?;
    let a1 = sheet
        .split("<c ")
        .find(|c| c.starts_with("r=\"A1\""))
        .map(str::to_string);
    let reloaded = load_from_xlsx_bytes(&saved, "m", "en", "UTC")?;
    let mut again = Model::from_workbook(reloaded, "en")?;
    again.set_user_input(0, 20, 1, "=ISBLANK(A1)".to_string())?;
    again.evaluate();
    Ok((a1, again.get_formatted_cell_value(0, 20, 1)?))
}

#[test]
fn empty_shared_string_cell_stays_a_text_cell() -> TestResult {
    for si in ["<si><t></t></si>", "<si><t/></si>", "<si/>"] {
        let (a1, isblank) = a1_after_round_trip(si, None)?;
        let a1 = a1.ok_or("A1 was not written")?;
        assert!(a1.contains("t=\"s\""), "{si}: {a1}");
        assert_eq!(isblank, "FALSE", "{si}");
    }
    Ok(())
}

/// The shapes poi/bug66675.xlsx stores its empty strings in (`WGT-D01 By Production Day` B9 and
/// E9): `t="str"` with no `<v>`, and an empty inline string.
#[test]
fn empty_str_and_inline_string_cells_stay_text_cells() -> TestResult {
    for cell in [
        r#"<c r="A1" s="1" t="str"></c>"#,
        r#"<c s="1" t="str" r="A1"></c>"#,
        r#"<c r="A1" s="1" t="str"><v></v></c>"#,
        r#"<c r="A1" s="1" t="inlineStr"><is><t xml:space="preserve"></t></is></c>"#,
        r#"<c r="A1" s="1" t="inlineStr"><is><t/></is></c>"#,
    ] {
        let (a1, isblank) = a1_after_round_trip("<si><t>Bold</t></si>", Some(cell))?;
        let a1 = a1.ok_or("A1 was not written")?;
        assert!(
            a1.contains("t=\"s\"") || a1.contains("t=\"str\"") || a1.contains("inlineStr"),
            "{cell}: {a1}"
        );
        assert_eq!(isblank, "FALSE", "{cell}: {a1}");
    }
    Ok(())
}
