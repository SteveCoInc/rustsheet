//! Grouped cache fields as their `<fieldGroup>` (T-4734): the shapes Excel saves, which the import
//! reads back (`import/pivots/groups.rs` `apply_groups`).
//!
//! - A field grouped in place by a range (`number_groups`, `date_groups`) writes its own values
//!   ([`CacheField::ungrouped`]) as its shared items and record values, then
//!   `<fieldGroup [par] base>` with its `<rangePr>` and its group items as `<groupItems>`
//!   (probes/t4128-pivot-spike G; t4170-pivot-date-levels's Date).
//! - A date level Excel added (`Quarters`, `Years`: an [`AddedGroup`] without a list, beside a
//!   base grouped in place by dates) is a `databaseField="0"` field with `<fieldGroup base>`, its
//!   `<rangePr>` and `<groupItems>`; the base's `par` names the last of them.
//! - A manual group Excel added (`Product2`: an [`AddedGroup`] with a list, over a base that is not
//!   grouped in place) is a `databaseField="0"` field with `<fieldGroup base>`, `<discretePr>` (per
//!   base shared item, its group item) and `<groupItems>`; the base has only `<fieldGroup par>`
//!   (probes/t4170-pivot-manual-groups).
//!
//! The shapes are the case builder's (`tools/src/rustsheet_tools/pivot_groups.py`, read off
//! Excel-authored caches; golden parts `tools/tests/pivot_parts_golden/t4170-*`), which Excel for
//! the web opened and rendered (pivots.md "Manual groups and date levels as Excel for the web
//! renders them"). `autoStart` and `autoEnd` are written off, as the builder writes them: the model
//! keeps no such flag, and with them off Excel keeps the bounds the engine groups by (the engine's
//! refresh keeps them too).
//!
//! A grouping is written only when the import derives the field back from what is written: the
//! check groups the base values again by the import's own rule (`group_by_numbers`,
//! `group_by_dates`, `group_by_discrete`) and requires the field it holds. Any other shape (a
//! numeric level Excel added, a manual group beside a range grouping, two manual groups of one
//! field, a date group in a 1904 workbook, which the import refuses) drops the cache with a log
//! line, as before.
//!
//! The work is linear in the cache (directive 5): the fields Excel added are indexed by base
//! once ([`Added`]), and each grouped field's own values are copied once, to be grouped again.

use std::fmt::Write as _;

use rustsheet_engine::pivot::{
    group_by_dates, group_by_discrete, group_by_numbers, AddedGroup, CacheField, CacheItem,
    DateGroups, PivotCache,
};
use rustsheet_engine::types::DateSystem;

use super::cache::{date_text, write_item, Unwritable};

/// The values a database field's shared items and records hold: its own values before a range
/// grouping ([`CacheField::ungrouped`]), else its items and records.
pub(super) fn own_values(field: &CacheField) -> (&[CacheItem], &[u32]) {
    match &field.ungrouped {
        Some(values) => (&values.items, &values.records),
        None => (&field.items, &field.records),
    }
}

/// How many of `cache`'s fields are database fields, which hold record values: those before
/// the first field Excel added or calculated field (T-4174).
pub(super) fn database_fields(cache: &PivotCache) -> usize {
    (cache.fields.iter())
        .take_while(|f| f.added_group.is_none() && f.formula.is_none())
        .count()
}

/// The fields Excel added to a cache, by the database field they group, indexed once per cache
/// so that no step scans the fields per field.
pub(super) struct Added {
    /// Per database field, the indices of the added fields whose base it is, in cache order;
    /// empty when the cache has no added field.
    by_base: Vec<Vec<usize>>,
    /// How many added fields have a database field as their base.
    indexed: usize,
}

impl Added {
    /// The added fields of `cache`, whose first `database` fields are its database fields.
    pub(super) fn of(cache: &PivotCache, database: usize) -> Added {
        let mut added = Added {
            by_base: Vec::new(),
            indexed: 0,
        };
        if cache.added_group_count() == 0 {
            return added;
        }
        added.by_base = vec![Vec::new(); database];
        for (index, field) in cache.fields.iter().enumerate() {
            let base = field.added_group.as_ref().map(|g| g.base as usize);
            if let Some(slot) = base.and_then(|b| added.by_base.get_mut(b)) {
                slot.push(index);
                added.indexed += 1;
            }
        }
        added
    }

    /// The added fields grouping database field `index`.
    fn of_base(&self, index: usize) -> &[usize] {
        self.by_base.get(index).map_or(&[], Vec::as_slice)
    }

    /// The `par` of database field `index`: the last field Excel added to group it.
    pub(super) fn par(&self, index: usize) -> Option<usize> {
        self.of_base(index).last().copied()
    }
}

/// Checks that every grouping of `cache` has a shape the export writes and the import reads
/// back as it is (the module docs).
pub(super) fn check(cache: &PivotCache, date_system: DateSystem) -> Result<(), Unwritable> {
    let database = database_fields(cache);
    if (cache.fields.iter().skip(database)).any(|f| f.added_group.is_none()) {
        return Err(Unwritable(
            "a database field after a field Excel added".into(),
        ));
    }
    if date_system != DateSystem::Excel1900 && cache.fields.iter().any(|f| f.date_groups.is_some())
    {
        // The import refuses it: a 1904 cache's date groups are unprobed.
        return Err(Unwritable("a date group in a 1904 workbook".into()));
    }
    let added = Added::of(cache, database);
    if added.indexed != cache.added_group_count() {
        return Err(Unwritable(
            "an added field whose base is not a database field".into(),
        ));
    }
    for (index, field) in cache.fields.iter().enumerate().take(database) {
        check_database_field(cache, field, added.of_base(index))?;
    }
    Ok(())
}

/// Checks database field `field`, grouped in place or not, and the fields Excel added to group it
/// (`levels`): each must be what its base values group to.
fn check_database_field(
    cache: &PivotCache,
    field: &CacheField,
    levels: &[usize],
) -> Result<(), Unwritable> {
    let groups = || {
        (levels.iter())
            .filter_map(|&k| cache.fields.get(k))
            .filter_map(|f| f.added_group.as_ref().map(|g| (f, g)))
    };
    let in_place = field.number_groups.is_some() || field.date_groups.is_some();
    let fits = if field.date_groups.is_some() {
        // Date levels beside the finest one, grouped in place.
        groups().all(|(_, g)| g.discrete.is_none())
    } else if in_place {
        // Excel adds no field beside a numeric range group.
        levels.is_empty()
    } else {
        // At most one manual group, beside a field that is not grouped.
        levels.len() <= 1 && groups().all(|(_, g)| g.discrete.is_some())
    };
    if !fits {
        return Err(Unwritable(format!(
            "field {:?} has added group fields Excel does not save beside it",
            shown(&field.name)
        )));
    }
    let owned;
    let base = match (&field.ungrouped, in_place) {
        (None, false) => field,
        (Some(own), true) => {
            // The one copy of the field's own values, which every grouping of it reads.
            owned = CacheField {
                name: field.name.clone(),
                items: own.items.clone(),
                records: own.records.clone(),
                ..CacheField::default()
            };
            let regrouped = range_grouped(&owned, field)?;
            same_grouping(&regrouped, field)?;
            &owned
        }
        // Kept values without a grouping are not written, so they would not come back.
        (Some(_), false) => {
            return Err(Unwritable(format!(
                "field {:?} keeps values of a grouping it does not have",
                shown(&field.name)
            )));
        }
        (None, true) => {
            return Err(Unwritable(format!(
                "range-grouped field {:?} without its own values",
                shown(&field.name)
            )));
        }
    };
    for (level, group) in groups() {
        // Its values are its base's: kept ones of its own would not come back.
        if level.ungrouped.is_some() {
            return Err(Unwritable(format!(
                "added field {:?} keeps values of its own",
                shown(&level.name)
            )));
        }
        let mut derived = added_field(base, level, group)?;
        derived.added_group = Some(group.clone());
        same_grouping(&derived, level)?;
    }
    Ok(())
}

/// Added field `field` derived again from `base`, its base field's own values.
fn added_field(
    base: &CacheField,
    field: &CacheField,
    group: &AddedGroup,
) -> Result<CacheField, Unwritable> {
    match &group.discrete {
        Some(map) => {
            if field.number_groups.is_some() || field.date_groups.is_some() {
                return Err(Unwritable(format!(
                    "manual group field {:?} with a range grouping",
                    shown(&field.name)
                )));
            }
            // The import keeps a list only as long as the base's items.
            if map.len() != base.items.len() {
                return Err(Unwritable(format!(
                    "manual group field {:?} does not list each base item once",
                    shown(&field.name)
                )));
            }
            group_by_discrete(base, map, field.items.clone()).map_err(|e| Unwritable(e.to_string()))
        }
        // A date level, beside a base grouped in place by dates ([`check_database_field`]).
        None if field.date_groups.is_some() => range_grouped(base, field),
        None => Err(Unwritable(format!(
            "added field {:?} that is not a date level",
            shown(&field.name)
        ))),
    }
}

/// Checks that `derived`, a field grouped again by the import's rule, is `field` but for its name
/// (the group functions name it after the base) and its kept own values (`derived` was grouped
/// from them).
fn same_grouping(derived: &CacheField, field: &CacheField) -> Result<(), Unwritable> {
    let same = derived.items == field.items
        && derived.records == field.records
        && derived.number_groups == field.number_groups
        && derived.date_groups == field.date_groups
        && derived.added_group == field.added_group;
    if same {
        Ok(())
    } else {
        Err(Unwritable(format!(
            "grouped field {:?} is not what its base values group to",
            shown(&field.name)
        )))
    }
}

/// `base` grouped by `field`'s range grouping and group items, as the import groups it
/// (`group_field`).
fn range_grouped(base: &CacheField, field: &CacheField) -> Result<CacheField, Unwritable> {
    let unwritable = |e: &dyn std::fmt::Display| Unwritable(e.to_string());
    match (&field.number_groups, field.date_groups) {
        (Some(g), None) => group_by_numbers(base, [g.start, g.end, g.interval], &field.items)
            .map_err(|e| unwritable(&e)),
        (None, Some(g)) => {
            let (by, start, end) = g.bounds();
            let groups =
                DateGroups::new(by, start, end, field.items.len()).map_err(|e| unwritable(&e))?;
            group_by_dates(base, groups, field.items.clone()).map_err(|e| unwritable(&e))
        }
        _ => Err(Unwritable(format!(
            "field {:?} grouped by both numbers and dates",
            shown(&field.name)
        ))),
    }
}

/// Writes the `<sharedItems>` attributes a date group needs beside [`SharedItemsFlags`]'s, for
/// a field grouped in place by dates whose own values are all dates: `containsNonDate="0"`
/// (the schema's default is on) and `minDate`/`maxDate`. Excel saved all three on the date field
/// of probes/t4170-pivot-date-levels (its sidecar,
/// eval/suites/probes/cases/t4170-pivot-date-levels.oracle.json, `cache_fields[0].shared_flags`,
/// with `maxDate` a day past the last date); the case builder writes the same shape. Without them
/// Excel for the web's refresh drops the date grouping and lists each date (T-4734 oracle run,
/// pivots.md "Excel for the web opens engine-exported grouped pivot caches"). `maxDate` is written
/// as the last date: whether Excel's day past it matters is UNVERIFIED (T-4654).
///
/// [`SharedItemsFlags`]: rustsheet_engine::pivot::SharedItemsFlags
pub(super) fn write_date_flags(
    xml: &mut String,
    field: &CacheField,
    date_system: DateSystem,
) -> Result<(), Unwritable> {
    let (Some(_), None) = (field.date_groups, &field.added_group) else {
        return Ok(());
    };
    let (items, _) = own_values(field);
    let mut range: Option<(f64, f64)> = None;
    for item in items {
        let CacheItem::Date(d) = item else {
            return Ok(());
        };
        let (low, high) = range.unwrap_or((*d, *d));
        range = Some((low.min(*d), high.max(*d)));
    }
    if let Some((low, high)) = range {
        let _ = write!(
            xml,
            r#" containsNonDate="0" minDate="{}" maxDate="{}""#,
            date_text(low, date_system)?,
            date_text(high, date_system)?
        );
    }
    Ok(())
}

/// Writes `field`'s `<fieldGroup>`, if it has one, at `index` of a cache, `par` being its
/// [`Added::par`] for a database field. [`check`] must have passed.
pub(super) fn write_field_group(
    xml: &mut String,
    index: usize,
    field: &CacheField,
    par: Option<usize>,
    date_system: DateSystem,
) -> Result<(), Unwritable> {
    let par = if field.added_group.is_none() {
        par
    } else {
        None
    };
    let base = match &field.added_group {
        Some(group) => Some(group.base as usize),
        None if field.number_groups.is_some() || field.date_groups.is_some() => Some(index),
        None => None,
    };
    let Some(base) = base else {
        // A field with only a manual group beside it: `<fieldGroup par/>` names it.
        if let Some(par) = par {
            let _ = write!(xml, r#"<fieldGroup par="{par}"/>"#);
        }
        return Ok(());
    };
    xml.push_str("<fieldGroup");
    if let Some(par) = par {
        let _ = write!(xml, r#" par="{par}""#);
    }
    let _ = write!(xml, r#" base="{base}">"#);
    if let Some(g) = &field.number_groups {
        let _ = write!(
            xml,
            r#"<rangePr autoStart="0" autoEnd="0" startNum="{}" endNum="{}" groupInterval="{}"/>"#,
            g.start, g.end, g.interval
        );
    } else if let Some(g) = field.date_groups {
        let (by, start, end) = g.bounds();
        let _ = write!(
            xml,
            r#"<rangePr autoStart="0" autoEnd="0" groupBy="{}" startDate="{}" endDate="{}"/>"#,
            by.name(),
            date_text(start, date_system)?,
            date_text(end, date_system)?
        );
    } else if let Some(map) = field.added_group.as_ref().and_then(|g| g.discrete.as_ref()) {
        let _ = write!(xml, r#"<discretePr count="{}">"#, map.len());
        for k in map {
            let _ = write!(xml, r#"<x v="{k}"/>"#);
        }
        xml.push_str("</discretePr>");
    }
    let _ = write!(xml, r#"<groupItems count="{}">"#, field.items.len());
    for item in &field.items {
        write_item(xml, item, date_system)?;
    }
    xml.push_str("</groupItems></fieldGroup>");
    Ok(())
}

/// A name from the file, cut short for a log line.
fn shown(name: &str) -> String {
    name.chars().take(200).collect()
}
