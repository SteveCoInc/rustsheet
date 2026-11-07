//! SPEC-0034 D1/D2, T-4559: a workbook saved unchanged since its load is its source, byte for
//! byte; any change is patched or regenerated (T-4560).

mod support;

use std::error::Error;

use support::rezip;

use rustsheet::{LoadOptions, SaveOptions, Value, Workbook, WorkbookOptions};

type Res = Result<(), Box<dyn Error>>;

/// A1 = 2, A2 = `=A1*3` with its cached value 6, as the export writes them: recalculating
/// reproduces every cached value.
fn source() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    wb.edit(|t| {
        t.set("Sheet1!A1", 2.0)?;
        t.set("Sheet1!A2", "=A1*3")
    })?;
    Ok(wb.to_xlsx_bytes(SaveOptions::default())?)
}

fn load(bytes: &[u8], recalc: bool) -> Result<Workbook, Box<dyn Error>> {
    let opts = LoadOptions {
        recalc,
        ..LoadOptions::default()
    };
    Ok(Workbook::from_xlsx_bytes(bytes, &opts)?)
}

#[test]
fn an_unchanged_workbook_saves_as_its_source() -> Res {
    let bytes = source()?;
    for recalc in [false, true] {
        let saved = load(&bytes, recalc)?.to_xlsx_bytes(SaveOptions::default())?;
        assert!(
            saved == bytes,
            "recalc {recalc}: the save is not the source"
        );
    }
    Ok(())
}

#[test]
fn an_edited_workbook_is_not_written_back_verbatim() -> Res {
    let bytes = source()?;
    let mut wb = load(&bytes, true)?;
    wb.edit(|t| t.set("Sheet1!A1", 5.0))?;
    let saved = wb.to_xlsx_bytes(SaveOptions::default())?;
    assert_ne!(saved, bytes);
    assert_eq!(load(&saved, true)?.value("Sheet1!A2")?, Value::Number(15.0));
    Ok(())
}

#[test]
fn a_changed_cached_value_is_not_written_back_verbatim() -> Res {
    // A stale cache: A2's stored value is 0, and recalculating makes it 6.
    let stale = rezip(
        &source()?,
        "xl/worksheets/sheet1.xml",
        "<v>6</v>",
        "<v>0</v>",
    )?;
    assert_ne!(
        load(&stale, true)?.to_xlsx_bytes(SaveOptions::default())?,
        stale
    );
    assert_eq!(
        load(&stale, false)?.to_xlsx_bytes(SaveOptions::default())?,
        stale
    );
    Ok(())
}
