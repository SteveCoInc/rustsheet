#![allow(clippy::unwrap_used, reason = "tests")]
//! Tests of `Workbook::load_pivot_parts` (edit/pivot/parts.rs).

use super::*;
use crate::{Input, Value, WorkbookOptions};

const NS: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// K/V over A1:B4 (east 1, west 2, east 3) and the parts a case builder writes for a pivot of
/// Sum of V by K anchored at D1, its cache `refreshOnLoad` or not.
fn book(refresh_on_load: bool) -> (Workbook, PivotParts) {
    let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
    let records = [
        ("K", Input::from("V")),
        ("east", 1.0.into()),
        ("west", 2.0.into()),
    ];
    wb.edit(|tx| {
        for (row, (key, value)) in records
            .into_iter()
            .chain([("east", 3.0.into())])
            .enumerate()
        {
            tx.set(format!("A{}", row + 1).as_str(), Input::from(key))?;
            tx.set(format!("B{}", row + 1).as_str(), value)?;
        }
        Ok(())
    })
    .unwrap();
    let refresh = if refresh_on_load {
        r#" refreshOnLoad="1""#
    } else {
        ""
    };
    let sheet = wb.sheet_id("Sheet1").unwrap();
    let parts = PivotParts {
        cache_definition: format!(
            r#"<pivotCacheDefinition xmlns="{NS}" xmlns:r="{REL}" r:id="rId1"{refresh} recordCount="3"><cacheSource type="worksheet"><worksheetSource ref="A1:B4" sheet="Sheet1"/></cacheSource><cacheFields count="2"><cacheField name="K" numFmtId="0"><sharedItems count="2"><s v="east"/><s v="west"/></sharedItems></cacheField><cacheField name="V" numFmtId="0"><sharedItems containsSemiMixedTypes="0" containsString="0" containsNumber="1" containsInteger="1" minValue="1" maxValue="3"/></cacheField></cacheFields></pivotCacheDefinition>"#
        ),
        cache_records: format!(
            r#"<pivotCacheRecords xmlns="{NS}" count="3"><r><x v="0"/><n v="1"/></r><r><x v="1"/><n v="2"/></r><r><x v="0"/><n v="3"/></r></pivotCacheRecords>"#
        ),
        cache_id: 1,
        tables: vec![(
            sheet,
            format!(
                r#"<pivotTableDefinition xmlns="{NS}" name="Pv" cacheId="1" dataCaption="Values" outline="1" outlineData="1"><location ref="D1:E2" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"/><pivotFields count="2"><pivotField axis="axisRow" showAll="0"><items count="3"><item x="0"/><item x="1"/><item t="default"/></items></pivotField><pivotField dataField="1" showAll="0"/></pivotFields><rowFields count="1"><field x="0"/></rowFields><dataFields count="1"><dataField name="Sum of V" fld="1" baseField="0" baseItem="0"/></dataFields></pivotTableDefinition>"#
            ),
        )],
    };
    (wb, parts)
}

fn value(wb: &Workbook, a1: &str) -> Value {
    wb.value(a1).unwrap()
}

#[test]
fn a_refresh_on_load_pivot_renders_at_the_next_recalculation() {
    let (mut wb, parts) = book(true);
    assert!(wb.load_pivot_parts(&[parts]).unwrap()[0].is_none());
    assert_eq!(value(&wb, "D1"), Value::Empty);
    wb.recalculate();
    for (a1, expected) in [
        ("D1", Value::from("Row Labels")),
        ("E1", Value::from("Sum of V")),
        ("D2", Value::from("east")),
        ("E2", Value::Number(4.0)),
        ("D3", Value::from("west")),
        ("E3", Value::Number(2.0)),
        ("D4", Value::from("Grand Total")),
        ("E4", Value::Number(6.0)),
    ] {
        assert_eq!(value(&wb, a1), expected, "{a1}");
    }
    assert_eq!(wb.pivot_places().len(), 1);
}

#[test]
fn a_pivot_saved_without_refresh_on_load_keeps_its_cells() {
    let (mut wb, parts) = book(false);
    assert!(wb.load_pivot_parts(&[parts]).unwrap()[0].is_none());
    wb.recalculate();
    assert_eq!(value(&wb, "D1"), Value::Empty);
    assert_eq!(wb.pivot_places().len(), 1);
}

#[test]
fn parts_the_importer_skips_add_nothing() {
    let (mut wb, mut parts) = book(true);
    parts.cache_id = 2;
    let skipped = wb.load_pivot_parts(&[parts]).unwrap();
    assert!(
        matches!(&skipped[..], [Some(PivotError::Parts(reason))] if reason.contains("cacheId")),
        "{skipped:?}"
    );
    wb.recalculate();
    assert!(wb.pivot_places().is_empty());
    assert_eq!(value(&wb, "D1"), Value::Empty);
}

#[test]
fn parts_past_the_pivot_limit_are_refused_whole() {
    let (mut wb, parts) = book(true);
    let many = vec![parts; MAX_PIVOTS + 1];
    let error = wb.load_pivot_parts(&many).unwrap_err();
    assert!(
        matches!(&error, EditError::Pivot(PivotError::Parts(reason)) if reason.contains("more than")),
        "{error}"
    );
    assert!(wb.pivot_places().is_empty());
}
