//! Grouped pivot cache fields (`<fieldGroup>`, T-4225).
//!
//! A grouped field shows group items (`<groupItems>`: `Qtr1`, `Jan`, `0-9`, ...) instead of its own
//! values. Excel stores no record values for a grouped field: each record's group is derived from
//! the base field's value, by a range (`<rangePr>`: numbers in steps, or dates by seconds, minutes,
//! hours, days, months, quarters or years) or by a list (`<discretePr>`: one group index per base
//! shared item). A grouping field Excel adds (`databaseField="0"`, `Quarters`, `Years`) derives
//! from its `base` field; a base field grouped in place (`par` set, `base` itself) shows its groups
//! in place of its values. The importer turns both into ordinary cache fields whose items are the
//! group items and whose records are each record's group, so the engine model and GETPIVOTDATA
//! need nothing new: a pivot field's `<item x>` indexes the group items, as Excel writes it.
//! Numbers and dates follow Excel's probed rules, [`group_by_numbers`] (T-4235) and
//! [`group_by_dates`] (T-4236), which also keep the grouping for GETPIVOTDATA's number items
//! (docs/excel-semantics/web.md "Grouped pivot fields import as their group items").

use rustsheet_engine::pivot::{
    group_by_dates, group_by_numbers, CacheField, CacheItem, DateGroupBy, DateGroups, PivotCache,
    PivotCacheError, MAX_ITEMS, MAX_RECORD_VALUES,
};

use super::{skip, Skip};

/// A `<rangePr>`: numbers from `start` to `end` in steps of `interval` (`by` is `None`, ECMA-376's
/// `range`), or dates from `start` to `end` grouped by `by`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RangeGroup {
    pub(super) by: Option<DateGroupBy>,
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) interval: f64,
}

/// One `<fieldGroup>` as read from the definition.
#[derive(Debug, Default)]
pub(super) struct FieldGroup {
    /// The cache field (in `<cacheFields>` order) the group belongs to.
    pub(super) field: u32,
    /// The database field its values come from.
    pub(super) base: u32,
    pub(super) range: Option<RangeGroup>,
    /// `<discretePr>`: per base shared item, its group index; `None` without one.
    pub(super) discrete: Option<Vec<u32>>,
    /// `<groupItems>`.
    pub(super) items: Vec<CacheItem>,
}

/// Turns the grouped fields of `cache` (its database fields only, as the records part holds them)
/// into group-item fields: each group replaces its field's items and records, and each name of
/// `grouping_fields` (the `databaseField="0"` fields, in order after the database fields) becomes
/// a field of its own. `values_left` is the workbook's record-value budget.
pub(super) fn apply_groups(
    cache: &mut PivotCache,
    groups: &[FieldGroup],
    grouping_fields: Vec<String>,
    values_left: usize,
) -> Result<(), Skip> {
    let database_fields = cache.fields.len();
    let total = database_fields + grouping_fields.len();
    if (cache.record_count as usize).saturating_mul(total) > values_left.min(MAX_RECORD_VALUES) {
        return Err(PivotCacheError::TooManyRecordValues.into());
    }
    let mut grouped = vec![false; total];
    for group in groups {
        let Some(seen) = grouped.get_mut(group.field as usize) else {
            return skip("a fieldGroup past the cache's fields");
        };
        if *seen {
            return skip("a field grouped twice");
        }
        *seen = true;
        if group.base as usize >= database_fields {
            return skip("a fieldGroup whose base is not a database field");
        }
    }
    if grouped.iter().skip(database_fields).any(|g| !g) {
        return skip("a grouping field without a fieldGroup");
    }
    // Every group reads its base's own values, so all are derived before any field changes.
    let mut derived = Vec::with_capacity(groups.len());
    let mut items = cache.fields.iter().map(|f| f.items.len()).sum::<usize>();
    for group in groups {
        let Some(base) = cache.fields.get(group.base as usize) else {
            return skip("a fieldGroup whose base is not a database field");
        };
        let field = group_field(base, group)?;
        items = items.saturating_add(field.items.len());
        if items > MAX_ITEMS {
            return Err(PivotCacheError::TooManyItems.into());
        }
        derived.push((group.field as usize, field));
    }
    derived.sort_by_key(|(index, _)| *index);
    let mut names = grouping_fields.into_iter();
    for (index, field) in derived {
        if let Some(existing) = cache.fields.get_mut(index) {
            existing.items = field.items;
            existing.records = field.records;
            existing.number_groups = field.number_groups;
            existing.date_groups = field.date_groups;
        } else {
            // Sorted, and every grouping field has exactly one group, so this is the next one.
            let Some(name) = names.next() else {
                return skip("a grouping field without a name");
            };
            cache.fields.push(CacheField { name, ..field });
        }
    }
    Ok(())
}

/// The group items of `group` and, per record of `base`, its group.
fn group_field(base: &CacheField, group: &FieldGroup) -> Result<CacheField, Skip> {
    let count = group.items.len();
    if count == 0 {
        return skip("a fieldGroup without groupItems");
    }
    // A range: Excel's own rule for numbers, which relabels a `<groupItems>` list of the wrong
    // length rather than refusing it (`group_by_numbers`; probes/t4228-pivot-range-groups pivot
    // 3), and for dates (`group_by_dates`, T-4236).
    match (&group.range, &group.discrete) {
        (Some(range), None) => {
            return Ok(match range.by {
                None => group_by_numbers(
                    base,
                    [range.start, range.end, range.interval],
                    group.items.clone(),
                )?,
                Some(by) => {
                    let groups = DateGroups::new(by, range.start, range.end, count)?;
                    group_by_dates(base, groups, group.items.clone())?
                }
            });
        }
        (_, None) => return skip("a fieldGroup without rangePr or discretePr"),
        (_, Some(_)) => {}
    }
    let mut map = Vec::with_capacity(base.items.len());
    for index in 0..base.items.len() {
        let target = match group.discrete.as_ref().and_then(|d| d.get(index)) {
            Some(&k) if (k as usize) < count => k,
            _ => return skip("a base item without a valid discrete group"),
        };
        map.push(target);
    }
    let records = base
        .records
        .iter()
        .map(|&i| map.get(i as usize).copied())
        .collect::<Option<Vec<u32>>>()
        .ok_or(PivotCacheError::BadRecord)?;
    Ok(CacheField {
        name: String::new(),
        items: group.items.clone(),
        records,
        ..CacheField::default()
    })
}
