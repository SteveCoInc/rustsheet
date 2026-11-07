//! A pivot cache as its `pivotCacheDefinition` and `pivotCacheRecords` parts (T-4176).
//!
//! The model keeps a cache's items as one list per field (the file's `<sharedItems>`, then the
//! inline record values interned at import) and every record as indices into it, so the export
//! writes every item as a shared item and every record value as `<x v>`: the import reads the
//! items back in the same order and the records to the same indices. The `<sharedItems>` flags
//! are derived from the items ([`SharedItemsFlags`]), the ones Excel writes and opens
//! (docs/excel-semantics/pivots.md "Shared-item flags Excel accepts for mixed fields"). The root's
//! version attributes are the case builder's, which every filled pivot probe opened with
//! (`tools/src/rustsheet_tools/pivot_definition.py`). A calculated field is written as the
//! builder and Excel write it, `<cacheField name numFmtId="0" formula databaseField="0"/>` after
//! the other fields, its formula as the file had it, with no record values (T-4174).

use std::fmt::Write as _;

use chrono::{Duration, NaiveDate};
use rustsheet_engine::pivot::{
    check_calculated_fields, has_case_variant_texts, CacheItem, PivotCache, SharedItemsFlags,
    MAX_ITEMS, MAX_RECORDS, MAX_RECORD_VALUES,
};
use rustsheet_engine::types::DateSystem;

use super::super::escape::escape_xml;
use super::super::xml_constants::XML_DECLARATION;
use super::groups;
use super::{MAIN_NS, REL_NS};

const SECONDS_PER_DAY: f64 = 86_400.0;
/// The last day Excel's calendar holds, 9999-12-31, as a 1900 serial.
const LAST_SERIAL: f64 = 2_958_465.0;

/// Why a cache is not exported; its pivots are dropped with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Unwritable(pub(super) String);

/// Checks that `cache` can be written: a worksheet source (a range or a name), groupings of the
/// shapes [`groups::check`] accepts (T-4734), no two text items of a field that differ only in
/// case (Excel for the web calls such a file corrupt: docs/excel-semantics/pivots.md
/// "Case-variant text items make Excel for the web call the file corrupt"), finite numbers,
/// dates inside Excel's calendar, and every record an index into its field's items; calculated
/// fields only after the others, and formulas the import would read back
/// ([`check_calculated_fields`]). A cache does not hold both: the import skips one with
/// calculated and grouped fields, so the export does not write one.
///
/// A cache's fields are public, so one built through the API or decoded need not respect the
/// import limits: it is held to them here, before anything is written (directive 5), with
/// `values_left` the record values the workbook's earlier exported caches left of
/// [`MAX_RECORD_VALUES`]. On success, returns this cache's record values.
pub(super) fn check(
    cache: &PivotCache,
    date_system: DateSystem,
    values_left: usize,
) -> Result<usize, Unwritable> {
    if cache.source.is_none() && cache.source_name.is_none() {
        return Err(Unwritable("a cache without a worksheet source".into()));
    }
    let calculated = (cache.fields.iter()).any(|f| f.formula.is_some());
    if calculated && (cache.fields.iter()).any(|f| f.is_grouped() || f.ungrouped.is_some()) {
        return Err(Unwritable(
            "calculated fields beside grouped fields, a cache the import skips".into(),
        ));
    }
    // The fields before the first calculated field or field Excel added.
    let database = groups::database_fields(cache);
    if calculated {
        if (cache.fields.iter().skip(database)).any(|f| f.formula.is_none()) {
            return Err(Unwritable("a cache field after a calculated field".into()));
        }
        check_calculated_fields(&cache.fields).map_err(|error| Unwritable(error.to_string()))?;
    }
    // Every field but a calculated one counts its records, the added group fields too, as the
    // import counts them.
    let valued = (cache.fields.iter())
        .filter(|f| f.formula.is_none())
        .count();
    let records = cache.record_count as usize;
    let values = records.saturating_mul(valued);
    if records > MAX_RECORDS || values > values_left.min(MAX_RECORD_VALUES) {
        return Err(Unwritable(format!(
            "{records} records of {valued} fields, past the cache limits"
        )));
    }
    // A field grouped in place holds its own values beside its group items.
    let items = (cache.fields.iter()).fold(0usize, |n, f| {
        let own = f.ungrouped.as_ref().map_or(0, |u| u.items.len());
        n.saturating_add(f.items.len()).saturating_add(own)
    });
    if items > MAX_ITEMS {
        return Err(Unwritable(format!("{items} items, past the cache limits")));
    }
    for (index, field) in cache.fields.iter().enumerate() {
        for item in &field.items {
            check_item(item, date_system)?;
        }
        // A field Excel added holds no values of its own: [`groups::check`] derived it again
        // from its base.
        if index >= database {
            continue;
        }
        let (own_items, own_records) = groups::own_values(field);
        if has_case_variant_texts(own_items) {
            return Err(Unwritable(format!(
                "field {:?} holds texts that differ only in case",
                field.name.chars().take(200).collect::<String>()
            )));
        }
        for item in own_items {
            check_item(item, date_system)?;
        }
        let bad_record = own_records.len() != records
            || (own_records.iter()).any(|&index| index as usize >= own_items.len());
        if bad_record {
            return Err(Unwritable(format!(
                "field {:?} has records that are not indices into its items",
                field.name.chars().take(200).collect::<String>()
            )));
        }
    }
    // Last, once the limits bound the values it groups again; a cache with calculated fields
    // has no grouping (above).
    if !calculated {
        groups::check(cache, date_system)?;
    }
    Ok(values)
}

/// Checks that `item` has a faithful text: a finite number, a date inside the calendar.
fn check_item(item: &CacheItem, date_system: DateSystem) -> Result<(), Unwritable> {
    match item {
        CacheItem::Number(n) if !n.is_finite() => {
            Err(Unwritable("a cache number that is not finite".into()))
        }
        CacheItem::Date(serial) => date_text(*serial, date_system).map(|_| ()),
        _ => Ok(()),
    }
}

/// The `pivotCacheDefinition` part of `cache`, whose records part is its relationship `rId1`.
/// [`check`] must have passed.
pub(super) fn definition_xml(
    cache: &PivotCache,
    date_system: DateSystem,
) -> Result<String, Unwritable> {
    let mut xml = String::new();
    let refresh = if cache.refresh_on_load {
        r#" refreshOnLoad="1""#
    } else {
        ""
    };
    let _ = write!(
        xml,
        r#"{XML_DECLARATION}
<pivotCacheDefinition xmlns="{MAIN_NS}" xmlns:r="{REL_NS}" r:id="rId1"{refresh} createdVersion="6" refreshedVersion="6" minRefreshableVersion="3" recordCount="{}"><cacheSource type="worksheet"><worksheetSource"#,
        cache.record_count
    );
    if let Some(source) = &cache.source {
        let _ = write!(
            xml,
            r#" ref="{}" sheet="{}""#,
            escape_xml(&source.range),
            escape_xml(&source.sheet_name)
        );
    }
    if let Some(name) = &cache.source_name {
        let _ = write!(xml, r#" name="{}""#, escape_xml(name));
    }
    let _ = write!(
        xml,
        r#"/></cacheSource><cacheFields count="{}">"#,
        cache.fields.len()
    );
    let database = groups::database_fields(cache);
    let added = groups::Added::of(cache, database);
    for (index, field) in cache.fields.iter().enumerate() {
        if let Some(formula) = &field.formula {
            let _ = write!(
                xml,
                r#"<cacheField name="{}" numFmtId="0" formula="{}" databaseField="0"/>"#,
                escape_xml(&field.name),
                escape_xml(formula)
            );
            continue;
        }
        let _ = write!(
            xml,
            r#"<cacheField name="{}" numFmtId="0""#,
            escape_xml(&field.name)
        );
        if index >= database {
            // A field Excel added: no shared items or record values, only its group.
            xml.push_str(r#" databaseField="0">"#);
        } else {
            // A field grouped in place lists its own values; its group items follow in
            // `<fieldGroup>`.
            let (own, _) = groups::own_values(field);
            xml.push_str("><sharedItems");
            write_flags(&mut xml, &SharedItemsFlags::of_items(own));
            groups::write_date_flags(&mut xml, field, date_system)?;
            if own.is_empty() {
                xml.push_str("/>");
            } else {
                let _ = write!(xml, r#" count="{}">"#, own.len());
                for item in own {
                    write_item(&mut xml, item, date_system)?;
                }
                xml.push_str("</sharedItems>");
            }
        }
        groups::write_field_group(&mut xml, index, field, added.par(index), date_system)?;
        xml.push_str("</cacheField>");
    }
    xml.push_str("</cacheFields></pivotCacheDefinition>");
    Ok(xml)
}

/// The `pivotCacheRecords` part of `cache`: each record's values as shared-item indices, over the
/// database fields only (a field grouped in place by its own values; a field Excel added has
/// none).
/// [`check`] must have passed, which bounds the records; the text grows as written rather than
/// being reserved up front.
pub(super) fn records_xml(cache: &PivotCache) -> String {
    let records = cache.record_count as usize;
    let database = groups::database_fields(cache);
    let mut xml = String::new();
    let _ = write!(
        xml,
        r#"{XML_DECLARATION}
<pivotCacheRecords xmlns="{MAIN_NS}" xmlns:r="{REL_NS}" count="{records}">"#
    );
    for record in 0..records {
        xml.push_str("<r>");
        for field in cache.fields.iter().take(database) {
            // Every database field holds `record_count` records ([`check`]).
            let index = groups::own_values(field)
                .1
                .get(record)
                .copied()
                .unwrap_or_default();
            let _ = write!(xml, r#"<x v="{index}"/>"#);
        }
        xml.push_str("</r>");
    }
    xml.push_str("</pivotCacheRecords>");
    xml
}

/// The `<sharedItems>` attributes of `flags`, each written only when it is not the schema's
/// default.
fn write_flags(xml: &mut String, flags: &SharedItemsFlags) {
    if !flags.contains_semi_mixed_types {
        xml.push_str(r#" containsSemiMixedTypes="0""#);
    }
    if !flags.contains_string {
        xml.push_str(r#" containsString="0""#);
    }
    if flags.contains_mixed_types {
        xml.push_str(r#" containsMixedTypes="1""#);
    }
    if flags.contains_number {
        xml.push_str(r#" containsNumber="1""#);
    }
    if flags.contains_integer {
        xml.push_str(r#" containsInteger="1""#);
    }
    if let Some((low, high)) = flags.number_range {
        let _ = write!(xml, r#" minValue="{low}" maxValue="{high}""#);
    }
    if flags.contains_blank {
        xml.push_str(r#" containsBlank="1""#);
    }
    if flags.contains_date {
        xml.push_str(r#" containsDate="1""#);
    }
}

/// One cache item. A number is written as Rust's shortest text that reads back as it.
pub(super) fn write_item(
    xml: &mut String,
    item: &CacheItem,
    date_system: DateSystem,
) -> Result<(), Unwritable> {
    match item {
        CacheItem::Missing => xml.push_str("<m/>"),
        CacheItem::Number(n) => {
            let _ = write!(xml, r#"<n v="{n}"/>"#);
        }
        CacheItem::Text(text) => {
            let _ = write!(xml, r#"<s v="{}"/>"#, escape_xml(text));
        }
        CacheItem::Boolean(b) => {
            let _ = write!(xml, r#"<b v="{}"/>"#, u8::from(*b));
        }
        CacheItem::Error(error) => {
            let _ = write!(xml, r#"<e v="{}"/>"#, escape_xml(&error.to_string()));
        }
        CacheItem::Date(serial) => {
            let _ = write!(xml, r#"<d v="{}"/>"#, date_text(*serial, date_system)?);
        }
    }
    Ok(())
}

/// A cache date's ISO 8601 text, to the second as Excel writes it (`2023-03-15T00:00:00`). The
/// import reads a date below serial 61 on the 1899-12-30 epoch and later ones on the cell
/// calendar, which agree from 1900-03-01 on (`pivot_cache_date_serial`, T-3182), so in a 1900
/// workbook every serial is days from 1899-12-30; in a 1904 workbook it is days from 1904-01-01.
pub(super) fn date_text(serial: f64, date_system: DateSystem) -> Result<String, Unwritable> {
    let (epoch, last) = match date_system {
        DateSystem::Excel1900 => (NaiveDate::from_ymd_opt(1899, 12, 30), LAST_SERIAL),
        DateSystem::Excel1904 => (NaiveDate::from_ymd_opt(1904, 1, 1), LAST_SERIAL - 1462.0),
    };
    let unwritable = || Unwritable(format!("a cache date {serial} outside the calendar"));
    if !(0.0..last + 1.0).contains(&serial) {
        return Err(unwritable());
    }
    let seconds = (serial * SECONDS_PER_DAY).round();
    // In range, so a whole number of seconds below 2^38: exact as an i64.
    let seconds = seconds as i64;
    let moment = epoch
        .and_then(|day| day.and_hms_opt(0, 0, 0))
        .and_then(|midnight| midnight.checked_add_signed(Duration::seconds(seconds)))
        .ok_or_else(unwritable)?;
    Ok(moment.format("%Y-%m-%dT%H:%M:%S").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_written_on_the_epoch_the_import_reads() {
        let text = |serial| date_text(serial, DateSystem::Excel1900).unwrap();
        assert_eq!(text(45000.0), "2023-03-15T00:00:00");
        assert_eq!(text(45000.5), "2023-03-15T12:00:00");
        assert_eq!(text(0.0), "1899-12-30T00:00:00");
        assert_eq!(text(60.0), "1900-02-28T00:00:00");
        assert_eq!(text(61.0), "1900-03-01T00:00:00");
        assert_eq!(text(443.0 / 86_400.0), "1899-12-30T00:07:23");
        assert_eq!(
            date_text(43830.0, DateSystem::Excel1904).unwrap(),
            "2024-01-01T00:00:00"
        );
        assert!(date_text(-1.0, DateSystem::Excel1900).is_err());
        assert!(date_text(f64::NAN, DateSystem::Excel1900).is_err());
    }
}
