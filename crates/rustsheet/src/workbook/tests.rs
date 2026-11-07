use std::io::Cursor;

use super::*;
use crate::value::{ErrorKind, XlError};
use crate::Input;

#[test]
fn text_utf16_is_some_only_for_text_holding_a_lone_surrogate() {
    use rustsheet_engine::types::{Cell, FormulaValue};
    use rustsheet_engine::Text;
    let mut wb = Workbook::new(&WorkbookOptions::default()).expect("new");
    wb.set("A1", Input::Formula("=\"a\"".into())).expect("a1");
    wb.set("A2", Input::Formula("=1".into())).expect("a2");
    // No engine function produces a lone surrogate yet (T-1579): plant the cached value.
    let lone = Text::from_utf16(vec![0x61, 0xDE00, 0x62]);
    let sheet = wb.model.workbook.worksheet_mut(0).expect("sheet");
    sheet.sheet_data.set_cell(
        3,
        1,
        Cell::CellFormula {
            f: 0,
            s: 0,
            v: FormulaValue::Text(lone),
        },
    );
    assert_eq!(wb.text_utf16("A1").unwrap(), None);
    assert_eq!(wb.text_utf16("A2").unwrap(), None);
    assert_eq!(wb.text_utf16("A4").unwrap(), None);
    assert_eq!(
        wb.text_utf16("A3").unwrap().as_deref(),
        Some(&[0x61, 0xDE00, 0x62][..])
    );
    assert_eq!(wb.value("A3").unwrap(), Value::from("a\u{FFFD}b"));
}

#[test]
fn date_system_is_chosen_at_new_and_survives_a_round_trip() {
    let default = Workbook::new(&WorkbookOptions::default()).expect("new");
    assert_eq!(default.date_system(), DateSystem::Excel1900);
    let opts = WorkbookOptions {
        date_system: DateSystem::Excel1904,
        ..WorkbookOptions::default()
    };
    let wb = Workbook::new(&opts).expect("new");
    assert_eq!(wb.date_system(), DateSystem::Excel1904);
    let bytes = wb.to_xlsx_bytes(SaveOptions::default()).expect("save");
    let loaded = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).expect("load");
    assert_eq!(loaded.date_system(), DateSystem::Excel1904);
}

#[test]
fn options_name_is_the_file_name_cell_reports_at_new_and_at_load() {
    let named = WorkbookOptions {
        name: Some("Budget".into()),
        ..WorkbookOptions::default()
    };
    let mut wb = Workbook::new(&named).expect("new");
    wb.edit(|tx| tx.add_sheet("Data").map(|_| ()))
        .expect("sheet");
    wb.set("A1", Input::Formula("=CELL(\"address\",Data!B2)".into()))
        .expect("a1");
    assert_eq!(
        wb.value("A1").unwrap(),
        Value::Text("[Budget.xlsx]Data!$B$2".into())
    );

    let bytes = wb.to_xlsx_bytes(SaveOptions::default()).expect("save");
    let unnamed = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).expect("load");
    assert!(matches!(unnamed.value("A1").unwrap(), Value::Error(_)));
    let loaded = LoadOptions {
        workbook: WorkbookOptions {
            name: Some("Other".into()),
            ..WorkbookOptions::default()
        },
        ..LoadOptions::default()
    };
    let renamed = Workbook::from_xlsx_bytes(&bytes, &loaded).expect("load");
    assert_eq!(
        renamed.value("A1").unwrap(),
        Value::Text("[Other.xlsx]Data!$B$2".into())
    );
}

#[test]
fn options_clock_applies_at_new_and_at_the_load_time_recalc() {
    let opts = WorkbookOptions {
        clock: Clock::fixed_unix_millis(0).expect("clock"),
        ..WorkbookOptions::default()
    };
    let mut wb = Workbook::new(&opts).expect("new");
    wb.set("A1", Input::Formula("=NOW()".into())).expect("a1");
    wb.set("A2", Input::Formula("=TODAY()".into())).expect("a2");
    assert_eq!(wb.value("A1").unwrap(), Value::Number(25569.0));
    assert_eq!(wb.value("A2").unwrap(), Value::Number(25569.0));

    let bytes = wb.to_xlsx_bytes(SaveOptions::default()).expect("save");
    let next_day = LoadOptions {
        workbook: WorkbookOptions {
            clock: Clock::fixed_unix_millis(86_400_000).expect("clock"),
            ..WorkbookOptions::default()
        },
        ..LoadOptions::default()
    };
    let loaded = Workbook::from_xlsx_bytes(&bytes, &next_day).expect("load");
    assert_eq!(loaded.value("A1").unwrap(), Value::Number(25570.0));
    assert_eq!(loaded.value("A2").unwrap(), Value::Number(25570.0));
}

/// A workbook built through the engine and round-tripped through xlsx.
fn sample() -> Workbook {
    let mut model = Model::new_empty("w", "en", "UTC", "en").expect("model");
    model.update_cell_with_number(0, 1, 1, 2.5).expect("a1");
    model.update_cell_with_text(0, 2, 1, "hi").expect("a2");
    model
        .update_cell_with_formula(0, 1, 2, "=A1*2".into())
        .expect("b1");
    model
        .update_cell_with_formula(0, 2, 2, "=1/0".into())
        .expect("b2");
    model.add_sheet("My Sheet").expect("sheet");
    model.update_cell_with_bool(1, 3, 3, true).expect("c3");
    model.evaluate();
    let bytes = rustsheet_xlsx::export::save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .expect("save")
        .into_inner();
    Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).expect("load")
}

#[test]
fn reads_values_formulas_and_errors() {
    let wb = sample();
    assert_eq!(wb.value("A1").unwrap(), Value::Number(2.5));
    assert_eq!(wb.value("Sheet1!a2").unwrap(), Value::Text("hi".into()));
    assert_eq!(wb.value("B1").unwrap(), Value::Number(5.0));
    assert_eq!(
        wb.value("B2").unwrap(),
        Value::Error(XlError::with_detail(ErrorKind::Div0, "Divide by 0"))
    );
    assert_eq!(wb.value("'my sheet'!C3").unwrap(), Value::Bool(true));
    assert_eq!(wb.value("Z9").unwrap(), Value::Empty);
    let info = wb.cell("B1").unwrap();
    assert_eq!(info.formula.as_deref(), Some("=A1*2"));
    assert_eq!(info.number_format, None);
    assert_eq!(wb.formatted("A1").unwrap(), "2.5");
}

#[test]
fn values_returns_row_major_array() {
    let wb = sample();
    let a = wb.values("A1:B2").unwrap();
    assert_eq!((a.rows(), a.cols()), (2, 2));
    assert_eq!(a.get(0, 1), Some(&Value::Number(5.0)));
    assert_eq!(a.get(1, 0), Some(&Value::Text("hi".into())));
}

#[test]
fn oversized_reads_fail_before_allocating() {
    let wb = sample();
    assert!(matches!(wb.values("A:XFD"), Err(RefError::TooLarge)));
    assert!(matches!(wb.values("1:1048576"), Err(RefError::TooLarge)));
}

#[test]
fn sheets_and_used_range() {
    let wb = sample();
    let sheets = wb.sheets();
    assert_eq!(sheets.len(), 2);
    assert_eq!(sheets[1].name, "My Sheet");
    assert_eq!(wb.sheet_id("MY SHEET").unwrap(), sheets[1].id);
    assert!(matches!(
        wb.sheet_id("nope"),
        Err(RefError::UnknownSheet(_))
    ));
    let used = wb.used_range(sheets[0].id).unwrap().unwrap();
    assert_eq!((used.top, used.left, used.bottom, used.right), (0, 0, 1, 1));
    let used = wb.used_range(sheets[1].id).unwrap().unwrap();
    assert_eq!((used.top, used.left), (2, 2));
    let empty = Workbook::new(&WorkbookOptions::default()).unwrap();
    let id = empty.sheets()[0].id;
    assert_eq!(empty.used_range(id).unwrap(), None);
}

#[test]
fn stored_cells_list_spill_members_sparsely() {
    let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
    let id = wb.sheets()[0].id;
    wb.set("B2", Input::Formula("=SEQUENCE(1,2)".into()))
        .unwrap();
    wb.set("XFD1048576", Input::Formula("=1".into())).unwrap();
    wb.recalculate();
    let cells: Vec<(u32, u32)> = wb
        .stored_cells(id)
        .unwrap()
        .into_iter()
        .map(|c| (c.row, c.col))
        .collect();
    assert_eq!(cells, vec![(1, 1), (1, 2), (1_048_575, 16_383)]);
}

#[test]
fn xlsx_round_trip_keeps_values() {
    let wb = sample();
    let bytes = wb.to_xlsx_bytes(SaveOptions::default()).unwrap();
    let again = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).unwrap();
    assert_eq!(again.value("B1").unwrap(), Value::Number(5.0));
}

#[test]
fn rejects_garbage_and_zip_limits() {
    assert!(matches!(
        Workbook::from_xlsx_bytes(b"not a zip", &LoadOptions::default()),
        Err(LoadError::Format(_))
    ));
    let bytes = sample().to_xlsx_bytes(SaveOptions::default()).unwrap();
    let few = LoadOptions {
        max_zip_entries: 1,
        ..LoadOptions::default()
    };
    assert!(matches!(
        Workbook::from_xlsx_bytes(&bytes, &few),
        Err(LoadError::Options(_))
    ));
    let small = LoadOptions {
        max_uncompressed_bytes: 10,
        ..LoadOptions::default()
    };
    assert!(matches!(
        Workbook::from_xlsx_bytes(&bytes, &small),
        Err(LoadError::Options(_))
    ));
}

/// A valid workbook whose `xl/sharedStrings.xml` (a part the importer reads) is replaced by
/// `len` zero bytes; deflate keeps the archive tiny.
fn workbook_with_bomb_part(len: usize) -> Vec<u8> {
    use std::io::Write;
    let good = sample().to_xlsx_bytes(SaveOptions::default()).unwrap();
    let mut src = zip::ZipArchive::new(Cursor::new(good)).unwrap();
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut entry = src.by_index(i).unwrap();
        let name = entry.name().unwrap().into_owned();
        zip.start_file(&name, zip::write::SimpleFileOptions::default())
            .unwrap();
        if name == "xl/sharedStrings.xml" {
            zip.write_all(&vec![0u8; len]).unwrap();
        } else {
            std::io::copy(&mut entry, &mut zip).unwrap();
        }
    }
    zip.finish().unwrap().into_inner()
}

#[test]
fn zip_bomb_is_rejected_by_actual_inflated_size() {
    let opts = LoadOptions {
        max_uncompressed_bytes: 1 << 20,
        ..LoadOptions::default()
    };
    let bomb = workbook_with_bomb_part(20 << 20);
    assert!(bomb.len() < 100_000, "test zip must be tiny");
    assert!(matches!(
        Workbook::from_xlsx_bytes(&bomb, &opts),
        Err(LoadError::Options(_))
    ));
    // Understate the declared size in the central directory (offset 24 of its header).
    let mut lying = bomb;
    let mut at = 0;
    while let Some(p) = lying[at..].windows(4).position(|w| w == b"PK\x01\x02") {
        at += p;
        let name = &lying[at + 46..at + 46 + 20];
        if name == b"xl/sharedStrings.xml" {
            lying[at + 24..at + 28].copy_from_slice(&10u32.to_le_bytes());
        }
        at += 4;
    }
    // zip 9 stops inflating an entry at its declared size and reports the overrun itself
    // ("File is larger than its declared uncompressed size"), so the lie is a format error
    // before the inflated-bytes budget is reached (T-1668).
    assert!(matches!(
        Workbook::from_xlsx_bytes(&lying, &opts),
        Err(LoadError::Format(_))
    ));
}

#[cfg(feature = "metadata_clock")]
#[test]
fn pin_clock_reads_the_metadata_now_row() {
    let mut model = Model::new_empty("w", "en", "UTC", "en").expect("model");
    model.add_sheet("METADATA").expect("sheet");
    model.update_cell_with_text(1, 1, 1, "NOW").expect("label");
    model
        .update_cell_with_number(1, 1, 2, 45000.5)
        .expect("serial");
    let bytes = rustsheet_xlsx::export::save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .expect("save")
        .into_inner();
    let mut pinned = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).unwrap();
    assert_eq!(pinned.model.clock(), Clock::system());
    assert!(pinned.pin_clock_from_metadata());
    // (45000.5 - 25569) days after the Unix epoch.
    assert_eq!(pinned.model.clock().fixed(), Some(1_678_881_600_000));
    let mut unpinned = sample();
    assert!(!unpinned.pin_clock_from_metadata());
    assert_eq!(unpinned.model.clock(), Clock::system());
}

#[cfg(feature = "harness_row_insert")]
#[test]
fn insert_rows_shifts_cells_and_rewrites_references() {
    // sample(): A1 2.5, B1 =A1*2.
    let mut wb = sample();
    let sheet = wb.sheet_id("Sheet1").unwrap();
    wb.insert_rows(sheet, 0, 1).unwrap();
    wb.recalculate();
    let moved = wb.cell("B2").unwrap();
    assert_eq!(moved.formula.as_deref(), Some("=A2*2"));
    assert_eq!(moved.value, Value::Number(5.0));
    assert_eq!(wb.value("A1").unwrap(), Value::Empty);
    for (row, count) in [(u32::MAX, 1), (0, 0), (0, u32::MAX)] {
        assert!(matches!(
            wb.insert_rows(sheet, row, count),
            Err(crate::error::EditError::Ref(RefError::OutOfBounds))
        ));
    }
}

#[cfg(feature = "harness_row_insert")]
#[test]
fn harness_deletes_and_column_edits_shift_cells_and_reject_bad_spans() {
    // sample(): A1 2.5, B1 =A1*2.
    let mut wb = sample();
    let sheet = wb.sheet_id("Sheet1").unwrap();
    wb.insert_columns(sheet, 0, 1).unwrap();
    wb.recalculate();
    assert_eq!(wb.cell("C1").unwrap().formula.as_deref(), Some("=B1*2"));
    wb.delete_columns(sheet, 0, 1).unwrap();
    wb.recalculate();
    assert_eq!(wb.cell("B1").unwrap().formula.as_deref(), Some("=A1*2"));
    wb.insert_rows(sheet, 0, 1).unwrap();
    wb.delete_rows(sheet, 0, 1).unwrap();
    wb.recalculate();
    assert_eq!(wb.value("B1").unwrap(), Value::Number(5.0));
    for (start, count) in [(u32::MAX, 1), (0, 0), (0, u32::MAX)] {
        assert!(matches!(
            wb.delete_rows(sheet, start, count),
            Err(crate::error::EditError::Ref(RefError::OutOfBounds))
        ));
        assert!(matches!(
            wb.insert_columns(sheet, start, count),
            Err(crate::error::EditError::Ref(RefError::OutOfBounds))
        ));
        assert!(matches!(
            wb.delete_columns(sheet, start, count),
            Err(crate::error::EditError::Ref(RefError::OutOfBounds))
        ));
    }
    assert!(matches!(
        wb.delete_table("Nope"),
        Err(crate::error::EditError::Engine(_))
    ));
}

#[test]
fn unknown_locale_is_an_options_error() {
    let opts = WorkbookOptions {
        locale: "xx".into(),
        ..WorkbookOptions::default()
    };
    assert!(matches!(Workbook::new(&opts), Err(LoadError::Options(_))));
}

/// The part `name` of the package `bytes`.
fn package_part(bytes: &[u8], name: &str) -> Vec<u8> {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
    let mut part = Vec::new();
    archive
        .by_name(name)
        .expect("part")
        .read_to_end(&mut part)
        .expect("read");
    part
}

/// A `refreshOnLoad` pivot that nothing edited is refreshed at the load, lays out to the cells it
/// already shows, and so saves as read: its cache definition, its records and its table, cacheId
/// and saved `rowItems`/`colItems` included, are the source's bytes (T-4825, SPEC-0034). Before,
/// the refresh dropped the saved lines, the save regenerated the pivot parts, and
/// `roundtrip-poi/54436.xlsx` lost them.
#[test]
fn an_unedited_refresh_on_load_pivot_saves_its_parts_as_read() {
    let source = without_formula_cells(
        &std::fs::read("../rustsheet-xlsx/tests/pivot_tables.xlsx").expect("fixture"),
        "xl/worksheets/sheet2.xml",
    );
    assert!(String::from_utf8_lossy(&package_part(
        &source,
        "xl/pivotCache/pivotCacheDefinition1.xml"
    ))
    .contains(r#"refreshOnLoad="1""#));
    let loaded = Workbook::from_xlsx_bytes(&source, &LoadOptions::default()).expect("load");
    let saved = loaded.to_xlsx_bytes(SaveOptions::default()).expect("save");
    for name in [
        "xl/pivotCache/pivotCacheDefinition1.xml",
        "xl/pivotCache/pivotCacheRecords1.xml",
        "xl/pivotTables/pivotTable1.xml",
        "xl/pivotCache/pivotCacheDefinition2.xml",
        "xl/pivotCache/pivotCacheRecords2.xml",
        "xl/pivotTables/pivotTable2.xml",
        "xl/workbook.xml",
    ] {
        assert_eq!(
            package_part(&saved, name),
            package_part(&source, name),
            "{name}"
        );
    }
}

/// `bytes` with the formula cells of its part `sheet` taken out. The fixture's GETPIVOTDATA cells
/// cache a placeholder 0, so the load's recalculation changes them, and the save then regenerates
/// for them, whatever the pivots do.
fn without_formula_cells(bytes: &[u8], sheet: &str) -> Vec<u8> {
    use std::io::{Read, Write};
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().expect("name").to_string();
        let mut text = String::new();
        entry.read_to_string(&mut text).expect("text part");
        if name == sheet {
            while let Some(start) = text.find(r#" cm="1"><f"#) {
                let open = text[..start].rfind("<c ").expect("cell start");
                let close = start + text[start..].find("</c>").expect("cell end") + "</c>".len();
                text.replace_range(open..close, "");
            }
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .expect("start");
        out.write_all(text.as_bytes()).expect("write");
    }
    out.finish().expect("finish").into_inner()
}
