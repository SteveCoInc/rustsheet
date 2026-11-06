//! Which sheet an API-added name's unqualified references bind to (T-4353, T-4445).

use super::*;

#[test]
fn an_unqualified_name_reference_binds_to_the_first_sheet() {
    let mut wb = book();
    wb.edit(|tx| {
        let s2 = tx.add_sheet("S2")?;
        tx.set("B1", 1.0)?;
        tx.set("B2", 10.0)?;
        tx.set("S2!B1", 7.0)?;
        tx.set("S2!B2", 70.0)?;
        tx.define_sheet_name(s2, "Rate", "=$B$1")?;
        tx.define_sheet_name(s2, "Span", "=$B$1:$B$2")?;
        tx.define_name("W", "=$B$1")?;
        tx.set("S2!A1", Input::Formula("=S2!Rate".into()))?;
        tx.set("S2!A2", Input::Formula("=SUM(Span)".into()))?;
        tx.set("S2!A3", Input::Formula("=W".into()))?;
        Ok(())
    })
    .unwrap();
    assert_eq!(wb.value("S2!A1").unwrap(), Value::Number(1.0));
    assert_eq!(wb.value("S2!A2").unwrap(), Value::Number(11.0));
    assert_eq!(wb.value("S2!A3").unwrap(), Value::Number(1.0));
}

#[test]
fn an_unqualified_name_reference_binds_to_the_active_sheet() {
    // probes/t4394-api-name-active-s2 and -last (T-4445): Graph `names/add` reads `$B$1` on the
    // active sheet, for a workbook name and for one scoped to any sheet.
    let mut wb = book();
    let (last, _) = wb
        .edit(|tx| {
            tx.add_sheet("S2")?;
            let last = tx.add_sheet("Last")?;
            tx.set("B1", 1.0)?;
            tx.set("S2!B1", 7.0)?;
            tx.set("S2!B2", 70.0)?;
            tx.set("Last!B1", 9.0)?;
            tx.set("Last!B2", 90.0)?;
            Ok(last)
        })
        .unwrap();
    assert_eq!(wb.active_sheet(), wb.sheet_id("Sheet1").unwrap());
    let s2 = wb.sheet_id("S2").unwrap();
    wb.set_active_sheet(s2).unwrap();
    assert_eq!(wb.active_sheet(), s2);
    wb.edit(|tx| {
        tx.define_name("W", "=$B$1")?;
        tx.define_sheet_name(last, "Span", "=$B$1:$B$2")?;
        tx.set("A1", Input::Formula("=W".into()))?;
        tx.set("Last!A1", Input::Formula("=SUM(Span)".into()))?;
        Ok(())
    })
    .unwrap();
    assert_eq!(wb.value("A1").unwrap(), Value::Number(7.0));
    assert_eq!(wb.value("Last!A1").unwrap(), Value::Number(77.0));
    // A name added after the active sheet changes binds to the new one; earlier names keep theirs.
    wb.set_active_sheet(last).unwrap();
    wb.edit(|tx| {
        tx.define_name("V", "=$B$1")?;
        tx.set("A2", Input::Formula("=V+W".into()))?;
        Ok(())
    })
    .unwrap();
    assert_eq!(wb.value("A2").unwrap(), Value::Number(16.0));
    assert!(wb.set_active_sheet(SheetId::new(999)).is_err());
}
