//! A pivot definition's `<filters>` typed (T-4165): read from the element the table reader keeps
//! verbatim for export, as the x14 extensions are, so the round trip writes it back as it was.

use quick_xml::events::BytesStart;
use rustsheet_engine::pivot::{
    CacheItem, DateGroupItem, DynamicDate, FilterCondition, FilterCriteria, FilterOperator,
    PivotCache, PivotFilter, PivotFilterKind, MAX_ITEMS,
};

use super::super::package::ImportLimits;
use super::table::flag;
use super::{
    at, attribute, index_attribute, local_name, shown, skip, walk, Skip, Tag, MAX_AXIS_FIELDS,
};

/// Most `<filter>`s one pivot holds: four per field of a full row and column axis
/// (`MAX_AXIS_FIELDS` each), far past what Excel writes (one label, one value and one date filter
/// per field). Past it the pivot is skipped (DEVIATION, decision 0014: docs/excel-semantics/
/// pivots.md "Pivot filters past the import limits").
pub(super) const MAX_FILTERS: usize = 4 * 2 * MAX_AXIS_FIELDS;

/// Most `customFilter`s one filter holds: ECMA-376's `CT_CustomFilters` allows two.
pub(super) const MAX_CONDITIONS: usize = 2;

/// Most `filter` and `dateGroupItem` values one values list holds: a cache's items
/// ([`MAX_ITEMS`]), since a list names items. Past it the pivot is skipped (DEVIATION, as
/// [`MAX_FILTERS`]).
pub(super) const MAX_LIST_VALUES: usize = MAX_ITEMS;

/// The path of a filter's criterion element, below `<filters>`.
const COLUMN: [&str; 4] = ["filters", "filter", "autoFilter", "filterColumn"];

/// The `<filter>`s of a kept `<filters>` element `xml`, over `cache`. A filter whose `fld` names no
/// cache field skips the pivot: shown unfiltered it would show values Excel does not.
///
/// A date filter on a field the file caches as numbers (a number item and no date item) is
/// dropped, as Excel drops it on open and saves the pivot without it (docs/excel-semantics/
/// pivots.md "Pivot sorts and filters as Excel for the web renders them",
/// probes/t4130-pivot-date-filters `DLit`; that a field mixing numbers with other items drops it is
/// UNVERIFIED, T-4799).
pub(super) fn read_filters(
    xml: &str,
    cache: &PivotCache,
    limits: &ImportLimits,
) -> Result<Vec<PivotFilter>, Skip> {
    let mut filters: Vec<PivotFilter> = Vec::new();
    walk(xml.as_bytes(), |path, tag| {
        let Tag::Open(element) = tag else {
            return Ok(());
        };
        let name = local_name(element);
        if at(path, &["filters"]) && name == "filter" {
            if filters.len() >= MAX_FILTERS {
                return skip("too many pivot filters");
            }
            filters.push(read_filter(element, cache)?);
            return Ok(());
        }
        let Some(filter) = filters.last_mut() else {
            return Ok(());
        };
        if at(path, &COLUMN) {
            read_criterion(filter, element, &name)?;
        } else if at(
            path,
            &[
                "filters",
                "filter",
                "autoFilter",
                "filterColumn",
                "customFilters",
            ],
        ) && name == "customFilter"
        {
            let value = attribute(element, "val")?.unwrap_or_default();
            limits.check_text(&value)?;
            let operator = match attribute(element, "operator")?.as_deref() {
                None => FilterOperator::Equal,
                Some(operator) => FilterOperator::from_name(operator).map_or_else(
                    || skip(format!("unknown filter operator {:?}", shown(operator))),
                    Ok,
                )?,
            };
            if let FilterCriteria::Custom { conditions, .. } = &mut filter.criteria {
                if conditions.len() >= MAX_CONDITIONS {
                    return skip("a pivot filter with more than two customFilters");
                }
                conditions.push(FilterCondition { operator, value });
            }
        } else if at(
            path,
            &["filters", "filter", "autoFilter", "filterColumn", "filters"],
        ) {
            read_value(filter, element, &name, limits)?;
        } else if at(path, &["filters", "filter", "extLst", "ext"]) && name == "pivotFilter" {
            filter.whole_day = flag(element, "useWholeDay", false)?;
        }
        Ok(())
    })?;
    filters.retain(|filter| {
        !(filter.kind == PivotFilterKind::Date && numbers_only(cache, filter.field))
    });
    Ok(filters)
}

/// Whether the file caches `field` as numbers: a number item and no date item.
fn numbers_only(cache: &PivotCache, field: u32) -> bool {
    let items = cache.fields.get(field as usize).map(|f| f.items.as_slice());
    let items = items.unwrap_or_default();
    items.iter().any(|i| matches!(i, CacheItem::Number(_)))
        && !items.iter().any(|i| matches!(i, CacheItem::Date(_)))
}

/// A `<filter>`'s attributes; its criterion follows.
fn read_filter(element: &BytesStart, cache: &PivotCache) -> Result<PivotFilter, Skip> {
    let field = index_attribute(element, "fld")?;
    if field as usize >= cache.fields.len() {
        return skip(format!("filter field {field} is past the cache's fields"));
    }
    let number = |name: &str| -> Result<Option<i64>, Skip> {
        Ok(attribute(element, name)?.and_then(|v| v.parse::<i64>().ok()))
    };
    let kind = attribute(element, "type")?.map_or(PivotFilterKind::Unknown, |name| {
        PivotFilterKind::from_type_name(&name)
    });
    Ok(PivotFilter {
        field,
        kind,
        measure: number("iMeasureFld")?
            .and_then(|m| u32::try_from(m).ok())
            .unwrap_or(0),
        eval_order: number("evalOrder")?
            .and_then(|m| i32::try_from(m).ok())
            .unwrap_or(0),
        id: number("id")?
            .and_then(|m| u32::try_from(m).ok())
            .unwrap_or(0),
        criteria: FilterCriteria::Everything,
        whole_day: false,
    })
}

/// The criterion element of a `filterColumn`.
fn read_criterion(filter: &mut PivotFilter, element: &BytesStart, name: &str) -> Result<(), Skip> {
    filter.criteria = match name {
        "customFilters" => FilterCriteria::Custom {
            and: flag(element, "and", false)?,
            conditions: Vec::new(),
        },
        "top10" => FilterCriteria::Top10 {
            top: flag(element, "top", true)?,
            value: attribute(element, "val")?.unwrap_or_default(),
        },
        "dynamicFilter" => {
            let name = attribute(element, "type")?.unwrap_or_default();
            FilterCriteria::Dynamic(DynamicDate::from_name(&name))
        }
        "filters" => FilterCriteria::Values {
            values: Vec::new(),
            blank: flag(element, "blank", false)?,
            dates: Vec::new(),
        },
        // An icon (or a colour, which Excel for the web refuses) keeps no item: FIcon.
        "iconFilter" | "colorFilter" => FilterCriteria::Nothing,
        _ => return Ok(()),
    };
    Ok(())
}

/// A `filter` or `dateGroupItem` of a values list.
fn read_value(
    filter: &mut PivotFilter,
    element: &BytesStart,
    name: &str,
    limits: &ImportLimits,
) -> Result<(), Skip> {
    let FilterCriteria::Values { values, dates, .. } = &mut filter.criteria else {
        return Ok(());
    };
    if matches!(name, "filter" | "dateGroupItem")
        && values.len().saturating_add(dates.len()) >= MAX_LIST_VALUES
    {
        return skip("a pivot filter's values list is past the import limit");
    }
    match name {
        "filter" => {
            let value = attribute(element, "val")?.unwrap_or_default();
            limits.check_text(&value)?;
            values.push(value);
        }
        "dateGroupItem" => {
            let part = |name: &str| -> Result<u32, Skip> {
                Ok(attribute(element, name)?
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0))
            };
            let parts = match attribute(element, "dateTimeGrouping")?.as_deref() {
                Some("year") => 1,
                Some("month") => 2,
                Some("day") => 3,
                Some("hour") => 4,
                Some("minute") => 5,
                Some("second") => 6,
                other => {
                    return skip(format!(
                        "unknown dateTimeGrouping {:?}",
                        shown(other.unwrap_or_default())
                    ))
                }
            };
            dates.push(DateGroupItem {
                year: i32::try_from(part("year")?).unwrap_or(0),
                month: part("month")?,
                day: part("day")?,
                hour: part("hour")?,
                minute: part("minute")?,
                second: part("second")?,
                parts,
            });
        }
        _ => {}
    }
    Ok(())
}
