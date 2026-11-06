//! The criterion of a table autoFilter's `<filterColumn>` (ECMA-376-1:2016 section 18.3.2), read
//! into checked, bounded data so the export writes back the filter Excel read: values (with
//! their date groups), custom, top 10, dynamic, color and icon filters (T-2838, T-4004). A
//! criterion whose enumeration or number is outside its schema type is not kept (`None`), and
//! the caller falls back to [`FilterCriterion::Other`].

use rustsheet_engine::types::{CustomFilter, DateGroupItem, FilterCriterion};

use super::iso_date::iso8601_to_serial;
use super::shared_strings::decode_xlsx_escapes;
use super::util::{get_bool, get_bool_false};
use super::xml::XmlNode;

/// `ST_DynamicFilterType`.
const DYNAMIC_FILTER_TYPES: &[&str] = &[
    "null",
    "aboveAverage",
    "belowAverage",
    "tomorrow",
    "today",
    "yesterday",
    "nextWeek",
    "thisWeek",
    "lastWeek",
    "nextMonth",
    "thisMonth",
    "lastMonth",
    "nextQuarter",
    "thisQuarter",
    "lastQuarter",
    "nextYear",
    "thisYear",
    "lastYear",
    "yearToDate",
    "Q1",
    "Q2",
    "Q3",
    "Q4",
    "M1",
    "M2",
    "M3",
    "M4",
    "M5",
    "M6",
    "M7",
    "M8",
    "M9",
    "M10",
    "M11",
    "M12",
];

/// `ST_IconSetType`.
const ICON_SET_TYPES: &[&str] = &[
    "3Arrows",
    "3ArrowsGray",
    "3Flags",
    "3TrafficLights1",
    "3TrafficLights2",
    "3Signs",
    "3Symbols",
    "3Symbols2",
    "4Arrows",
    "4ArrowsGray",
    "4RedToBlack",
    "4Rating",
    "4TrafficLights",
    "5Arrows",
    "5ArrowsGray",
    "5Rating",
    "5Quarters",
];

/// `ST_CalendarType`.
const CALENDAR_TYPES: &[&str] = &[
    "none",
    "gregorian",
    "gregorianUs",
    "japan",
    "taiwan",
    "korea",
    "hijri",
    "thai",
    "hebrew",
    "gregorianMeFrench",
    "gregorianArabic",
    "gregorianXlitEnglish",
    "gregorianXlitFrench",
];

/// `ST_DateTimeGrouping`.
const DATE_TIME_GROUPINGS: &[&str] = &["year", "month", "day", "hour", "minute", "second"];

/// The criterion of a `<filterColumn>` whose first element is `node`, or `None` when it is not
/// one of the kinds this module reads or does not check out.
pub(super) fn criterion(node: &XmlNode) -> Option<FilterCriterion> {
    match node.tag_name() {
        "filters" => values_filter(node),
        "customFilters" => Some(FilterCriterion::Custom {
            and: get_bool_false(node, "and"),
            conditions: node
                .children()
                .filter(|n| n.has_tag_name("customFilter"))
                .map(|n| CustomFilter {
                    operator: n.attribute("operator").map(str::to_string),
                    val: decode_xlsx_escapes(n.attribute("val").unwrap_or_default()),
                })
                .collect(),
        }),
        "top10" => Some(FilterCriterion::Top10 {
            top: get_bool(node, "top"),
            percent: get_bool_false(node, "percent"),
            val: double(node.attribute("val")?)?,
            filter_val: optional(node.attribute("filterVal"), double).ok()?,
        }),
        "dynamicFilter" => Some(FilterCriterion::Dynamic {
            kind: enumeration(node.attribute("type")?, DYNAMIC_FILTER_TYPES)?,
            val: optional(node.attribute("val"), double).ok()?,
            max_val: optional(node.attribute("maxVal"), double).ok()?,
            val_iso: optional(node.attribute("valIso"), date_time).ok()?,
            max_val_iso: optional(node.attribute("maxValIso"), date_time).ok()?,
        }),
        // Excel (excel-graph, 2026-11-06, T-4404): a `<colorFilter>` with no `dxfId` opens without
        // repair but is no criterion; SUBTOTAL calls its hidden rows manual hides. See
        // docs/excel-semantics/xlsx-storage.md, "Table parts in the export".
        "colorFilter" => Some(FilterCriterion::Color {
            dxf_id: Some(node.attribute("dxfId")?.parse().ok()?),
            cell_color: get_bool(node, "cellColor"),
        }),
        "iconFilter" => Some(FilterCriterion::Icon {
            icon_set: enumeration(node.attribute("iconSet")?, ICON_SET_TYPES)?,
            icon_id: optional(node.attribute("iconId"), |v| v.parse().ok()).ok()?,
        }),
        _ => None,
    }
}

/// A `<filters>` whose children are `<filter>` and `<dateGroupItem>` elements only. Values are
/// stored decoded from the file's `_xHHHH_` escapes, as cell text is; a `<filter>` without `val`
/// is dropped.
fn values_filter(filters: &XmlNode) -> Option<FilterCriterion> {
    let mut values = Vec::new();
    let mut date_groups = Vec::new();
    for child in filters.children() {
        match child.tag_name() {
            "filter" => {
                if let Some(val) = child.attribute("val") {
                    values.push(decode_xlsx_escapes(val));
                }
            }
            "dateGroupItem" => date_groups.push(date_group_item(child)?),
            _ => return None,
        }
    }
    Some(FilterCriterion::Values {
        values,
        blank: get_bool_false(filters, "blank"),
        calendar_type: optional(filters.attribute("calendarType"), |v| {
            enumeration(v, CALENDAR_TYPES)
        })
        .ok()?,
        date_groups,
    })
}

/// A `<dateGroupItem>`: `year` and `dateTimeGrouping` are required, the finer fields are
/// `xsd:unsignedShort`.
fn date_group_item(node: &XmlNode) -> Option<DateGroupItem> {
    let field = |name: &str| optional(node.attribute(name), |v| v.parse::<u16>().ok());
    Some(DateGroupItem {
        grouping: enumeration(node.attribute("dateTimeGrouping")?, DATE_TIME_GROUPINGS)?,
        year: node.attribute("year")?.parse().ok()?,
        month: field("month").ok()?,
        day: field("day").ok()?,
        hour: field("hour").ok()?,
        minute: field("minute").ok()?,
        second: field("second").ok()?,
    })
}

/// `Ok(None)` for an absent attribute, `Ok(Some(_))` for one `read` accepts, and `Err(())` for
/// one it refuses.
fn optional<T>(value: Option<&str>, read: impl Fn(&str) -> Option<T>) -> Result<Option<T>, ()> {
    match value {
        None => Ok(None),
        Some(value) => read(value).map(Some).ok_or(()),
    }
}

/// `value` when it is one of `allowed` (enumerations are case-sensitive).
fn enumeration(value: &str, allowed: &[&str]) -> Option<String> {
    allowed.contains(&value).then(|| value.to_string())
}

/// `value` when it is a finite `xsd:double`, kept as the file wrote it.
fn double(value: &str) -> Option<String> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|_| value.trim().to_string())
}

/// `value` when it is an ISO 8601 date-time Excel can read.
fn date_time(value: &str) -> Option<String> {
    iso8601_to_serial(value).map(|_| value.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::criterion;
    use crate::import::xml::XmlNode;
    use rustsheet_engine::types::{DateGroupItem, FilterCriterion};

    fn read(xml: &str) -> Option<FilterCriterion> {
        criterion(&XmlNode::parse_str(xml).unwrap())
    }

    #[test]
    fn top10_reads_its_defaults_and_refuses_a_non_finite_val() {
        assert_eq!(
            read(r#"<top10 val="2" filterVal="5"/>"#),
            Some(FilterCriterion::Top10 {
                top: true,
                percent: false,
                val: "2".to_string(),
                filter_val: Some("5".to_string()),
            })
        );
        assert_eq!(read(r#"<top10 val="NaN"/>"#), None);
        assert_eq!(read(r#"<top10 val="1e400"/>"#), None);
        assert_eq!(read(r"<top10/>"), None);
    }

    #[test]
    fn dynamic_color_and_icon_filters_check_their_enumerations() {
        assert_eq!(
            read(r#"<dynamicFilter type="aboveAverage" val="3.5"/>"#),
            Some(FilterCriterion::Dynamic {
                kind: "aboveAverage".to_string(),
                val: Some("3.5".to_string()),
                max_val: None,
                val_iso: None,
                max_val_iso: None,
            })
        );
        assert_eq!(read(r#"<dynamicFilter type="sometime"/>"#), None);
        assert_eq!(
            read(r#"<dynamicFilter type="today" valIso="2024-x"/>"#),
            None
        );
        assert_eq!(
            read(r#"<colorFilter dxfId="0" cellColor="0"/>"#),
            Some(FilterCriterion::Color {
                dxf_id: Some(0),
                cell_color: false,
            })
        );
        assert_eq!(read(r#"<colorFilter dxfId="-1"/>"#), None);
        assert_eq!(read(r"<colorFilter/>"), None);
        assert_eq!(read(r#"<colorFilter cellColor="0"/>"#), None);
        assert_eq!(
            read(r#"<iconFilter iconSet="3Arrows" iconId="2"/>"#),
            Some(FilterCriterion::Icon {
                icon_set: "3Arrows".to_string(),
                icon_id: Some(2),
            })
        );
        assert_eq!(read(r#"<iconFilter iconSet="3Stars"/>"#), None);
    }

    #[test]
    fn date_groups_are_kept_with_their_values() {
        let xml = r#"<filters blank="1" calendarType="gregorian"><filter val="a"/><dateGroupItem year="2024" month="2" dateTimeGrouping="month"/></filters>"#;
        assert_eq!(
            read(xml),
            Some(FilterCriterion::Values {
                values: vec!["a".to_string()],
                blank: true,
                calendar_type: Some("gregorian".to_string()),
                date_groups: vec![DateGroupItem {
                    grouping: "month".to_string(),
                    year: 2024,
                    month: Some(2),
                    day: None,
                    hour: None,
                    minute: None,
                    second: None,
                }],
            })
        );
        let bad_year =
            r#"<filters><dateGroupItem year="70000" dateTimeGrouping="year"/></filters>"#;
        assert_eq!(read(bad_year), None);
        let unknown = r#"<filters><dateGroupItem year="2024" dateTimeGrouping="week"/></filters>"#;
        assert_eq!(read(unknown), None);
        assert_eq!(read(r#"<filters><other/></filters>"#), None);
    }
}
