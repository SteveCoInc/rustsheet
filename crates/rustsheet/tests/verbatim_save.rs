//! SPEC-0034 D1/D2, T-4559: a workbook saved unchanged since its load is its source, byte for
//! byte; any change regenerates.

use rustsheet::{LoadOptions, SaveOptions, Workbook};

/// A1 = 2, A2 = `=A1*3` with its cached value 6, as the export writes them: recalculating
/// reproduces every cached value.
fn source() -> Vec<u8> {
    let mut wb = Workbook::new(&rustsheet::WorkbookOptions::default()).expect("new");
    wb.edit(|t| {
        t.set("Sheet1!A1", 2.0)?;
        t.set("Sheet1!A2", "=A1*3")
    })
    .expect("edit");
    wb.to_xlsx_bytes(SaveOptions::default()).expect("save")
}

#[test]
fn an_unchanged_workbook_saves_as_its_source() {
    let bytes = source();
    for recalc in [false, true] {
        let opts = LoadOptions {
            recalc,
            ..LoadOptions::default()
        };
        let wb = Workbook::from_xlsx_bytes(&bytes, &opts).expect("load");
        assert_eq!(
            wb.to_xlsx_bytes(SaveOptions::default()).expect("save"),
            bytes,
            "recalc {recalc}"
        );
    }
}

#[test]
fn an_edited_workbook_regenerates() {
    let bytes = source();
    let mut wb = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).expect("load");
    wb.edit(|t| t.set("Sheet1!A1", 5.0)).expect("edit");
    let saved = wb.to_xlsx_bytes(SaveOptions::default()).expect("save");
    assert_ne!(saved, bytes);
    let back = Workbook::from_xlsx_bytes(&saved, &LoadOptions::default()).expect("reload");
    assert_eq!(
        back.value("Sheet1!A2").expect("A2"),
        rustsheet::Value::Number(15.0)
    );
}

#[test]
fn a_changed_cached_value_regenerates() {
    // A stale cache: A2's stored value is 0, and recalculating makes it 6.
    let bytes = source();
    let stale = rezip(&bytes, "xl/worksheets/sheet1.xml", "<v>6</v>", "<v>0</v>");
    let wb = Workbook::from_xlsx_bytes(&stale, &LoadOptions::default()).expect("load");
    assert_ne!(
        wb.to_xlsx_bytes(SaveOptions::default()).expect("save"),
        stale
    );
    let unrecalculated = Workbook::from_xlsx_bytes(
        &stale,
        &LoadOptions {
            recalc: false,
            ..LoadOptions::default()
        },
    )
    .expect("load");
    assert_eq!(
        unrecalculated
            .to_xlsx_bytes(SaveOptions::default())
            .expect("save"),
        stale
    );
}

/// `bytes` with one text replacement in `part`.
fn rezip(bytes: &[u8], part: &str, from: &str, to: &str) -> Vec<u8> {
    use std::io::{Cursor, Read as _, Write as _};
    let mut src = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut f = src.by_index(i).expect("entry");
        let name = f.name().expect("name").to_string();
        if name.ends_with('/') {
            continue;
        }
        let mut data = Vec::new();
        f.read_to_end(&mut data).expect("read");
        if name == part {
            let text = String::from_utf8(data).expect("utf-8");
            assert!(text.contains(from), "{from}");
            data = text.replacen(from, to, 1).into_bytes();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}
