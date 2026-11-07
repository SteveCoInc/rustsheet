//! Group fields Excel adds to a cache (`databaseField="0"`, T-4171): the levels of a date field
//! grouped by several (`Quarters`, `Years`) and a manual (discrete) group field (`Product2`).
//!
//! Such a field has no source column: each record's group is derived from its base field's value
//! (`<fieldGroup base>`). The importer derives it once from the saved records; a refresh reads only
//! the source columns, so it derives each added field again from its base field's new values
//! ([`derive_added`]), before the base is grouped in place, as the importer does. The rules are the
//! M6 ones, not a second implementation (directive 3): a date level by [`regroup`] (that is,
//! [`group_by_dates`](super::group_by_dates)), a manual group by [`group_by_discrete`].
//! Evidence: probes/t4170-pivot-manual-groups and probes/t4170-pivot-date-levels (pivots.md
//! "Manual groups and date levels as Excel for the web renders them").

use std::collections::{HashMap, HashSet};

use bitcode::{Decode, Encode};

use super::refresh::{regroup, RangeGrouping};
use super::{CacheField, CacheItem, ItemKey, PivotCache, PivotCacheError, MAX_ITEMS};
use crate::functions::util::collation::fold_case;

/// How a field Excel added (`databaseField="0"`) derives from its base field.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AddedGroup {
    /// The cache index of the field whose values it groups (`<fieldGroup base>`).
    pub base: u32,
    /// For a manual group (`<discretePr>`), per item of the base field, its group item; `None`
    /// for a date level, whose grouping is the field's [`CacheField::date_groups`] (or
    /// [`CacheField::number_groups`]).
    pub discrete: Option<Vec<u32>>,
}

/// Field `base` grouped by hand: `groups` holds, per item of `base`, its group item among
/// `labels`, and each record takes its base item's group. An ungrouped item is a group item of its
/// own, under its own label, as Excel saves it (probes/t4170-pivot-manual-groups MC!A12 F).
///
/// # Errors
/// [`PivotCacheError::Discrete`] when a base item has no group item in `labels`, and
/// [`PivotCacheError::BadRecord`] for a record past `base`'s items.
pub fn group_by_discrete(
    base: &CacheField,
    groups: &[u32],
    labels: Vec<CacheItem>,
) -> Result<CacheField, PivotCacheError> {
    let map = groups
        .get(..base.items.len())
        .filter(|map| map.iter().all(|&k| (k as usize) < labels.len()))
        .ok_or(PivotCacheError::Discrete)?;
    let records = (base.records.iter())
        .map(|&i| map.get(i as usize).copied())
        .collect::<Option<Vec<u32>>>()
        .ok_or(PivotCacheError::BadRecord)?;
    Ok(CacheField {
        name: base.name.clone(),
        items: labels,
        records,
        ..CacheField::default()
    })
}

/// The added group fields of `old`, derived again from `fresh`, its source read again with every
/// field still ungrouped, in `old`'s order. `values_left` is the record-value budget `fresh` was
/// read within. An added field whose base field the source no longer has is dropped, as a gone
/// source field is (`field_map`).
///
/// A manual group keeps each old base item's group: a refresh keeps the base field's old items in
/// their slots (`read_source`). A base item new to the source becomes a group item of its own,
/// after the old ones, as an ungrouped item is saved (UNVERIFIED that Excel's refresh does this,
/// probe pending: T-4802).
///
/// # Errors
/// [`PivotCacheError::Group`] when a date level no longer fits its grouping, or the refresh would
/// drop it ([`regroup`] over a blank: UNVERIFIED what Excel does then, T-4802), when a new base
/// item's label equals a group's, or when the source gained a column of an added field's name
/// (UNVERIFIED, T-4802); the cache limits.
pub(super) fn derive_added(
    old: &PivotCache,
    fresh: &PivotCache,
    values_left: usize,
) -> Result<Vec<CacheField>, PivotCacheError> {
    let added = old.added_group_count();
    if added == 0 {
        return Ok(Vec::new());
    }
    let total = fresh.fields.len().saturating_add(added);
    if (fresh.record_count as usize).saturating_mul(total) > values_left {
        return Err(PivotCacheError::TooManyRecordValues);
    }
    let mut by_name = HashMap::with_capacity(fresh.fields.len());
    for (index, field) in (0u32..).zip(&fresh.fields) {
        by_name.entry(fold_case(&field.name)).or_insert(index);
    }
    let mut items: usize = fresh.fields.iter().map(|f| f.items.len()).sum();
    let mut out = Vec::with_capacity(added);
    for field in &old.fields {
        let Some(group) = &field.added_group else {
            continue;
        };
        if by_name.contains_key(&fold_case(&field.name)) {
            return Err(PivotCacheError::Group);
        }
        let base_name = old
            .fields
            .get(group.base as usize)
            .map(|b| fold_case(&b.name));
        let Some(&base_index) = base_name.and_then(|name| by_name.get(&name)) else {
            continue;
        };
        let base = fresh
            .fields
            .get(base_index as usize)
            .ok_or(PivotCacheError::BadRecord)?;
        let (mut derived, discrete) = match &group.discrete {
            Some(map) => {
                let (map, labels) = extend_discrete(base, map, &field.items, &mut items)?;
                (group_by_discrete(base, &map, labels)?, Some(map))
            }
            None => (derive_level(base, field)?, None),
        };
        items = items.saturating_add(derived.items.len());
        if items > MAX_ITEMS {
            return Err(PivotCacheError::TooManyItems);
        }
        derived.name.clone_from(&field.name);
        derived.added_group = Some(AddedGroup {
            base: base_index,
            discrete,
        });
        out.push(derived);
    }
    Ok(out)
}

/// A date level (or a range) `field` built again over `base`'s new values: the M6 rule, which
/// takes the level's labels back unchanged ([`regroup`]).
fn derive_level(base: &CacheField, field: &CacheField) -> Result<CacheField, PivotCacheError> {
    let grouping = RangeGrouping::of(field).ok_or(PivotCacheError::Group)?;
    let derived = regroup(base, &grouping, &field.items).map_err(|_| PivotCacheError::Group)?;
    // `regroup` leaves a field ungrouped where a refresh drops its grouping (a blank); an added
    // level has no values of its own to fall back to.
    if derived.number_groups.is_none() && derived.date_groups.is_none() {
        return Err(PivotCacheError::Group);
    }
    Ok(derived)
}

/// `map` and `labels` extended over the base items new to the source: each a group item of its
/// own, labelled with the item. `items` counts the new labels against [`MAX_ITEMS`].
fn extend_discrete(
    base: &CacheField,
    map: &[u32],
    labels: &[CacheItem],
    items: &mut usize,
) -> Result<(Vec<u32>, Vec<CacheItem>), PivotCacheError> {
    let mut map = map.to_vec();
    let mut labels = labels.to_vec();
    let new = base.items.get(map.len()..).unwrap_or_default();
    if new.is_empty() {
        return Ok((map, labels));
    }
    *items = items.saturating_add(new.len());
    if *items > MAX_ITEMS {
        return Err(PivotCacheError::TooManyItems);
    }
    let mut taken: HashSet<ItemKey> = labels.iter().map(ItemKey::identity).collect();
    for item in new {
        if !taken.insert(ItemKey::identity(item)) {
            return Err(PivotCacheError::Group);
        }
        // At most MAX_ITEMS labels, so the index fits a u32.
        map.push(u32::try_from(labels.len()).map_err(|_| PivotCacheError::TooManyItems)?);
        labels.push(item.clone());
    }
    Ok((map, labels))
}

#[cfg(test)]
mod tests;
