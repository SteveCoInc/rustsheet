//! The pivot lifecycle through the facade (SPEC-0032 "Who refreshes, and when", T-4151), on
//! crates/rustsheet-xlsx/tests/pivot_tables.xlsx (probes/getpivotdata-real-pivot built by the
//! case builder): two `refreshOnLoad` pivots over Data!A1:C7 whose saved cells are Excel's
//! layout of the source, so a refresh rewrites them unchanged. A saved cell is falsified
//! (P!B5, East/A, 14 in the file and the source) to tell a refresh from the saved cells.

mod support;

use std::error::Error;

use rustsheet::{EditError, LoadOptions, PivotError, Value, Workbook};
use support::rezip;

type Res = Result<(), Box<dyn Error>>;

const FILE: &[u8] = include_bytes!("../../rustsheet-xlsx/tests/pivot_tables.xlsx");

/// The file with P!B5 saved as 999, and `refreshOnLoad` on both caches or neither.
fn falsified(refresh_on_load: bool) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = rezip(
        FILE,
        "xl/worksheets/sheet2.xml",
        r#"<c r="B5"><v>14</v>"#,
        r#"<c r="B5"><v>999</v>"#,
    )?;
    if !refresh_on_load {
        for n in 1..=2 {
            let part = format!("xl/pivotCache/pivotCacheDefinition{n}.xml");
            bytes = rezip(&bytes, &part, r#" refreshOnLoad="1""#, "")?;
        }
    }
    Ok(bytes)
}

fn load(bytes: &[u8], recalc: bool) -> Result<Workbook, Box<dyn Error>> {
    let opts = LoadOptions {
        recalc,
        ..LoadOptions::default()
    };
    Ok(Workbook::from_xlsx_bytes(bytes, &opts)?)
}

#[test]
fn a_refresh_on_load_pivot_is_re_rendered_after_the_first_recalculation() -> Res {
    let wb = load(&falsified(true)?, true)?;
    assert_eq!(wb.value("P!B5")?, Value::Number(14.0));
    // The other cells are rewritten as the file saved them, the averages included.
    assert_eq!(wb.value("P!E7")?, Value::Number(49.0));
    assert_eq!(wb.value("P!B23")?, Value::Number(49.0 / 6.0));
    assert_eq!(wb.value("P!A4")?, Value::Text("Row Labels".into()));
    // A formula reading the pivot's cells sees the refresh: GETPIVOTDATA East/A.
    assert_eq!(wb.value("P!H5")?, Value::Number(14.0));
    // Loaded without a recalculation, the pivot waits for the first one.
    let mut wb = load(&falsified(true)?, false)?;
    assert_eq!(wb.value("P!B5")?, Value::Number(999.0));
    wb.recalculate();
    assert_eq!(wb.value("P!B5")?, Value::Number(14.0));
    Ok(())
}

#[test]
fn without_refresh_on_load_the_saved_cells_are_kept_until_a_refresh() -> Res {
    let mut wb = load(&falsified(false)?, true)?;
    assert_eq!(wb.value("P!B5")?, Value::Number(999.0));
    // A source edit does not refresh it (UNVERIFIED, T-4586).
    wb.set("Data!C2", 11.0)?;
    assert_eq!(wb.value("P!B5")?, Value::Number(999.0));
    let p = wb.sheet_id("P")?;
    wb.edit(|tx| tx.refresh_pivot_table(p, "PivotSum"))?;
    assert_eq!(wb.value("P!B5")?, Value::Number(15.0));
    assert_eq!(wb.value("P!E7")?, Value::Number(50.0));
    // Both pivots read the same source but keep their own caches: the other is not refreshed.
    assert_eq!(wb.value("P!B21")?, Value::Number(5.5));
    Ok(())
}

#[test]
fn a_refresh_rolls_back_with_its_edit_and_names_an_unknown_pivot() -> Res {
    let mut wb = load(&falsified(false)?, true)?;
    let p = wb.sheet_id("P")?;
    let failed = wb.edit(|tx| {
        tx.refresh_pivot_table(p, "PivotSum")?;
        tx.refresh_pivot_table(p, "Nope")
    });
    assert!(matches!(
        failed,
        Err(EditError::Pivot(PivotError::UnknownPivot(name))) if name == "Nope"
    ));
    assert_eq!(wb.value("P!B5")?, Value::Number(999.0));
    // The edit lists the cells the refresh rewrote.
    let ((), changes) = wb.edit(|tx| tx.refresh_pivot_table(p, "PivotSum"))?;
    assert!(changes
        .inputs
        .iter()
        .any(|c| c.after.as_deref() == Some("14") && c.before.as_deref() == Some("999")));
    Ok(())
}
