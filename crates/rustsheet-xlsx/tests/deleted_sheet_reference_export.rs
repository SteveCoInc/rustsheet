#![allow(clippy::unwrap_used, reason = "tests")]
//! A plain reference to a deleted sheet is saved as Excel's file holds it, `#REF!C4:C7`, and a
//! sheet added under the old name does not revive it (T-2612,
//! probes/table-edits/t2612-delete-sheet-then-add).

use std::io::{Cursor, Read};

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;

#[test]
fn a_reference_to_a_deleted_sheet_is_saved_as_ref() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.add_sheet("S").unwrap();
    model
        .set_user_input(0, 1, 1, "=SUM(S!C4:C7)+S!$A$1".to_string())
        .unwrap();
    model.delete_sheet(1).unwrap();
    model.add_sheet("S").unwrap();
    model.evaluate();
    let bytes = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    assert!(xml.contains("<f>SUM(#REF!C4:C7)+#REF!$A$1</f>"), "{xml}");
    assert!(!xml.contains("*#REF!"), "{xml}");
}
