#![allow(clippy::unwrap_used, reason = "tests")]
//! A loaded sheet name holding `*` is refused: the engine keeps a reference to a deleted sheet
//! under the name `*#REF!`, and Excel cannot write a sheet name with `*` (T-4655).

use std::io::{Cursor, Read, Write};

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

/// An xlsx of one sheet named `Sx`, with `Sx` replaced by `name` in `xl/workbook.xml`.
fn workbook_with_sheet_name(name: &str) -> Vec<u8> {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.rename_sheet_by_index(0, "Sx").unwrap();
    let bytes = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let file_name = file.name().unwrap().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        if file_name == "xl/workbook.xml" {
            let xml = String::from_utf8(data).unwrap();
            assert!(xml.contains(r#"name="Sx""#), "{xml}");
            data = xml
                .replace(r#"name="Sx""#, &format!(r#"name="{name}""#))
                .into_bytes();
        }
        out.start_file(file_name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&data).unwrap();
    }
    out.finish().unwrap().into_inner()
}

#[test]
fn a_sheet_named_like_the_deleted_sheet_sentinel_is_refused() {
    assert!(load_from_xlsx_bytes(&workbook_with_sheet_name("Sy"), "m", "en", "UTC").is_ok());
    for name in ["*#REF!", "A*B"] {
        let err = load_from_xlsx_bytes(&workbook_with_sheet_name(name), "m", "en", "UTC");
        assert!(err.is_err(), "{name}");
    }
}
