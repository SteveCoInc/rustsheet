//! Calculated cache fields (`cacheField@formula`, T-4174): what loads, and what skips the cache.

use super::*;

/// `parts` with calculated fields `extra` after the cache's fields and a data field over the
/// first of them.
fn calculated_parts(extra: &str) -> Vec<(String, String)> {
    let body = TABLE_BODY
        .replace(
            r#"<pivotField dataField="1"/></pivotFields>"#,
            r#"<pivotField dataField="1"/><pivotField dataField="1" dragToRow="0"/></pivotFields>"#,
        )
        .replace(r#"fld="2""#, r#"fld="3""#)
        .replace("Sum of Sales", "Sum of Double");
    let parts = with(
        parts(),
        DEFINITION_PART,
        definition(&format!("{FIELDS}{extra}")),
    );
    with(parts, TABLE_PART, table(&body))
}

#[test]
fn a_calculated_field_loads_with_its_formula_and_no_records() {
    let (caches, pivots) = load(&calculated_parts(
        r#"<cacheField name="Double" numFmtId="0" formula="Sales*2+'Region'" databaseField="0"/>"#,
    ));
    let field = &caches[0].fields[3];
    assert_eq!(field.name, "Double");
    assert_eq!(field.formula.as_deref(), Some("Sales*2+'Region'"));
    assert!(field.records.is_empty() && field.items.is_empty());
    assert_eq!(pivots[0].data_fields[0].field, 3);
    assert_eq!(pivots[0].fields.len(), 4);
}

#[test]
fn calculated_fields_excel_for_the_web_refuses_skip_the_cache() {
    // pivots.md "Pivot flags Excel for the web refuses" (T-4505), and the engine's own refusals.
    for formula in [
        "NOW()",
        "TODAY()",
        "RAND()",
        "INDIRECT(&quot;A1&quot;)",
        "SUM({1,2})+Sales",
        "Data!C2*2",
        "A1*2",
        "Foo*2",
        "OFFSET(Sales,0,0)",
    ] {
        let field = format!(r#"<cacheField name="Double" formula="{formula}" databaseField="0"/>"#);
        assert_no_cache(&calculated_parts(&field));
    }
    // Unprobed shapes: a calculated database field, a field after one, one with children.
    let double = r#"<cacheField name="Double" formula="Sales*2" databaseField="0"/>"#;
    assert_no_cache(&calculated_parts(
        r#"<cacheField name="Double" formula="Sales*2"/>"#,
    ));
    assert_no_cache(&calculated_parts(&format!(
        r#"{double}<cacheField name="More"><sharedItems/></cacheField>"#
    )));
    assert_no_cache(&calculated_parts(
        r#"<cacheField name="Double" formula="Sales*2" databaseField="0"><sharedItems><s v="x"/></sharedItems></cacheField>"#,
    ));
}
