//! The per-pivot index GETPIVOTDATA answers from (SPEC-0015 "Untrusted-input limits", T-2083).
//!
//! Built lazily, on a pivot's first GETPIVOTDATA call, in one pass over the cache's records: each
//! distinct tuple of axis-item indices keeps its records in cache order, so a call collects the
//! records of the tuples its pairs pick and folds them in cache order (one rule however the
//! records are grouped; SPEC-0015 unknown 11 leaves the summation order to T-2085). A call's
//! answer is memoized per (data field, items). The index and the memo live in the evaluation
//! state, keyed by the pivot's position in `Workbook::pivot_tables`, not on the pivot itself, so
//! `Workbook`'s `PartialEq` and `Clone` see no derived state. Both are dropped at each
//! `Model::evaluate` and whenever pivots are dropped: pivots and caches change only between
//! evaluations, so an index never outlives the pivot it was built from, and the memo holds at
//! most one entry per call made in one evaluation.

use std::collections::{HashMap, HashSet};

use crate::expressions::token::Error;
use crate::model::Model;
use crate::pivot::layout::{aggregate, record_numbers};
use crate::pivot::{AxisField, CacheField, CacheItem, PivotCache, PivotTable, BLANK_ITEM};

/// The indices of every pivot used in the current evaluation.
#[derive(Default)]
pub(crate) struct PivotIndexes {
    /// By position in `Workbook::pivot_tables`: code that removes or reorders pivots between
    /// evaluations must call [`PivotIndexes::clear`] (`Model::displace_pivots`,
    /// `move_pivots`, `take_pivots_on` and `restore_pivots` do).
    pivots: HashMap<usize, PivotIndex>,
    /// The `Evaluation::evaluations` count the indices were built in: a new evaluation drops
    /// them (`Model::pivot_axis`).
    evaluation: u64,
    /// How many indices were built since the model was made: a test counts the passes over the
    /// records.
    builds: u64,
}

impl PivotIndexes {
    /// Drops every index and memo.
    pub(crate) fn clear(&mut self) {
        self.pivots.clear();
    }

    /// How many indices were built so far.
    #[cfg(test)]
    pub(crate) fn builds(&self) -> u64 {
        self.builds
    }
}

/// What a call asks one pivot: the data field, and per axis field (in [`PivotIndex::axis`]
/// order) the item it is restricted to, `None` for every item.
#[derive(Hash, PartialEq, Eq, Clone)]
pub(crate) struct PivotQuery {
    pub(crate) data_field: usize,
    pub(crate) items: Box<[Option<u32>]>,
}

/// How an item name matches a field's items, ignoring case.
#[derive(Clone, Copy)]
enum ItemMatch {
    One(u32),
    /// Two text items differ only in case (`east`, `East`): Excel's pivot holds one item for both;
    /// what GETPIVOTDATA does when the cache keeps two is unprobed, so it is `#REF!`.
    Several,
}

struct PivotIndex {
    /// The cache fields on the row axis, then the column axis.
    axis: Vec<u32>,
    /// Per distinct tuple of axis-item indices, its records in cache order.
    tuples: HashMap<Box<[u32]>, Vec<u32>>,
    /// Per axis position, the items some tuple holds a record of: `shown_group` asks it in
    /// constant time instead of scanning `tuples` (T-4252).
    occupied: Vec<HashSet<u32>>,
    /// Per axis position, its text items by lowercased name, built on first use.
    item_names: Vec<Option<HashMap<String, ItemMatch>>>,
    memo: HashMap<PivotQuery, Result<f64, Error>>,
}

/// A pivot GETPIVOTDATA can answer from: one data field, no page field and no hidden item.
/// Every other shape is one of SPEC-0015's unknowns (1 and 4), `#REF!` until T-2085 probes it.
/// Several fields on an axis (unknown 10) are indexed too, but only their grand total is
/// answered ([`nested_axis`]).
fn supported(pivot: &PivotTable) -> Option<Vec<u32>> {
    if pivot.data_fields.len() != 1
        || pivot.fields.iter().any(|f| {
            f.axis == Some(crate::pivot::Axis::Page) || f.hidden.iter().any(|word| *word != 0)
        })
    {
        return None;
    }
    pivot
        .row_fields
        .iter()
        .chain(&pivot.column_fields)
        .map(|field| match field {
            AxisField::Field(index) => Some(*index),
            AxisField::Values => None,
        })
        .collect()
}

/// Whether `pivot` has several fields on its row or its column axis. GETPIVOTDATA answers such a
/// pivot's grand total (calipers `formula_stress_test` `Database!B16`: two row and two column
/// fields, `GETPIVOTDATA("Sum of Qux",B32)` is 57); any pair is SPEC-0015 unknown 10, `#REF!`.
pub(crate) fn nested_axis(pivot: &PivotTable) -> bool {
    pivot.row_fields.len() > 1 || pivot.column_fields.len() > 1
}

/// One pass over the records: `None` when an axis field or a record's index is not in the cache.
fn build(cache: &PivotCache, axis: Vec<u32>) -> Option<PivotIndex> {
    let columns = axis
        .iter()
        .map(|&f| cache.fields.get(f as usize).map(|f| &f.records))
        .collect::<Option<Vec<_>>>()?;
    let mut tuples: HashMap<Box<[u32]>, Vec<u32>> = HashMap::new();
    let mut occupied = vec![HashSet::new(); axis.len()];
    let mut key = Vec::with_capacity(axis.len());
    for record in 0..cache.record_count {
        key.clear();
        for column in &columns {
            key.push(*column.get(record as usize)?);
        }
        match tuples.get_mut(key.as_slice()) {
            Some(records) => records.push(record),
            None => {
                for (items, item) in occupied.iter_mut().zip(&key) {
                    items.insert(*item);
                }
                tuples.insert(key.as_slice().into(), vec![record]);
            }
        }
    }
    Some(PivotIndex {
        item_names: vec![None; axis.len()],
        axis,
        tuples,
        occupied,
        memo: HashMap::new(),
    })
}

impl Model {
    /// The axis fields of pivot `pivot` (row axis first), building its index on first use; `None`
    /// when the pivot is not one GETPIVOTDATA answers from ([`supported`]) or its cache is
    /// malformed, both `#REF!`.
    pub(crate) fn pivot_axis(&mut self, pivot: usize) -> Option<&[u32]> {
        let Model {
            workbook,
            evaluation,
            ..
        } = self;
        let indexes = &mut evaluation.pivot_indexes;
        if indexes.evaluation != evaluation.evaluations {
            indexes.pivots.clear();
            indexes.evaluation = evaluation.evaluations;
        }
        if !indexes.pivots.contains_key(&pivot) {
            let table = workbook.pivot_tables.get(pivot)?;
            let cache = workbook.pivot_caches.get(table.cache as usize)?;
            let index = build(cache, supported(table)?)?;
            indexes.builds += 1;
            indexes.pivots.insert(pivot, index);
        }
        indexes
            .pivots
            .get(&pivot)
            .map(|index| index.axis.as_slice())
    }

    /// The item of the field at axis position `position` of pivot `pivot` whose text equals
    /// `name` ignoring case; numbers, dates and booleans never match (SPEC-0015 unknown 3), and
    /// a blank only as `"(blank)"` on an ungrouped field ([`Model::pivot_blank_text`]). Call
    /// [`Model::pivot_axis`] first.
    pub(crate) fn pivot_item(&mut self, pivot: usize, position: usize, name: &str) -> Option<u32> {
        let Model {
            workbook,
            evaluation,
            ..
        } = self;
        let index = evaluation.pivot_indexes.pivots.get_mut(&pivot)?;
        let field = *index.axis.get(position)?;
        let names = index.item_names.get_mut(position)?;
        if names.is_none() {
            let table = workbook.pivot_tables.get(pivot)?;
            let items = &workbook
                .pivot_caches
                .get(table.cache as usize)?
                .fields
                .get(field as usize)?
                .items;
            let mut map = HashMap::new();
            for (item, i) in items.iter().zip(0u32..) {
                if let CacheItem::Text(text) = item {
                    map.entry(text.to_lowercase())
                        .and_modify(|m| *m = ItemMatch::Several)
                        .or_insert(ItemMatch::One(i));
                }
            }
            *names = Some(map);
        }
        let item = match names.as_ref()?.get(&name.to_lowercase()) {
            Some(ItemMatch::One(item)) => *item,
            Some(ItemMatch::Several) => return None,
            None => return self.pivot_blank_text(pivot, position, name),
        };
        self.shown_group(pivot, position, item)
    }

    /// The group of a numeric range field at axis position `position` of pivot `pivot` that the
    /// number `value` names ([`crate::pivot::NumberGroups::lower_bound_index`]:
    /// probes/t4228-pivot-range-groups Calc A35, C30:C46; probes/t4249-pivot-range-group-labels
    /// Calc P20:U31), or of a date-grouped field whose calendar value is `value`'s integer
    /// part ([`crate::pivot::DateGroups::number_item`]: probes/t4236-pivot-date-groups-no-blank),
    /// or the date item equal to it on an ungrouped field (probes/t4236-pivot-date-groups);
    /// `None` otherwise, where a number is SPEC-0015 unknown 3. Call
    /// [`Model::pivot_axis`] first.
    pub(crate) fn pivot_number_item(
        &self,
        pivot: usize,
        position: usize,
        value: f64,
    ) -> Option<u32> {
        let field = self.pivot_axis_field(pivot, position)?;
        let item = if let Some(groups) = &field.number_groups {
            groups.item(groups.lower_bound_index(value)?)
        } else if let Some(groups) = field.date_groups {
            groups.number_item(value)?
        } else {
            // An ungrouped date field (a date grouping over a blank that Excel's refresh
            // dropped): a serial equal to a second-rounded date item picks it, `DATE(2019,1,1)`
            // and `DATE(2018,1,1)` (probes/t4236-pivot-date-groups Calc B19, N11); `2018`, which
            // no item equals, is `#REF!` (N8). A serial within the second but not equal to the
            // item is unprobed (`#REF!` here).
            field
                .items
                .iter()
                .position(|item| matches!(item, CacheItem::Date(date) if *date == value))?
        };
        self.shown_group(pivot, position, u32::try_from(item).ok()?)
    }

    /// `item`, unless it holds no record and its field's `showAll` is off: Excel does not show
    /// such an item and GETPIVOTDATA on it is `#REF!` (a numeric or date group, probe Calc B3, R1,
    /// probes/t4236-pivot-date-groups-no-blank, t4311-pivot-showall-grouped H; an unused shared
    /// item of an ungrouped row or column field, probes/t4248-getpivotdata-recordless-item,
    /// t4311-pivot-showall-column). With `showAll` on it is shown, so 0: an empty group, a date
    /// group's `<start` and `>end` included (t4311-pivot-showall-grouped G, M), or an unused item
    /// (web.md).
    fn shown_group(&self, pivot: usize, position: usize, item: u32) -> Option<u32> {
        if self.pivot_axis_shows_all(pivot, position)? {
            return Some(item);
        }
        let index = self.evaluation.pivot_indexes.pivots.get(&pivot)?;
        index
            .occupied
            .get(position)?
            .contains(&item)
            .then_some(item)
    }

    /// The `showAll` of the pivot field at axis position `position` of pivot `pivot`.
    fn pivot_axis_shows_all(&self, pivot: usize, position: usize) -> Option<bool> {
        let field = self.pivot_axis_index(pivot, position)?;
        let table = self.workbook.pivot_tables.get(pivot)?;
        Some(table.fields.get(field as usize)?.show_all)
    }

    /// The cache field index at axis position `position` of pivot `pivot`.
    fn pivot_axis_index(&self, pivot: usize, position: usize) -> Option<u32> {
        let index = self.evaluation.pivot_indexes.pivots.get(&pivot)?;
        index.axis.get(position).copied()
    }

    /// The cache field at axis position `position` of pivot `pivot`.
    fn pivot_axis_field(&self, pivot: usize, position: usize) -> Option<&CacheField> {
        let field = self.pivot_axis_index(pivot, position)?;
        let table = self.workbook.pivot_tables.get(pivot)?;
        self.workbook
            .pivot_caches
            .get(table.cache as usize)?
            .fields
            .get(field as usize)
    }

    /// The blank item (`<m/>`, shown as `(blank)`) of the field at axis position `position` of
    /// pivot `pivot`, which an omitted item argument picks (calipers `pv23_041` `Liability Summary`
    /// G5:G7, `GETPIVOTDATA(...,"Remediation Status",)`); `None` when the field has none. Call
    /// [`Model::pivot_axis`] first.
    pub(crate) fn pivot_blank_item(&self, pivot: usize, position: usize) -> Option<u32> {
        let items = &self.pivot_axis_field(pivot, position)?.items;
        let mut blanks = (0u32..)
            .zip(items)
            .filter(|(_, item)| matches!(item, CacheItem::Missing))
            .map(|(i, _)| i);
        let blank = blanks.next()?;
        blanks.next().is_none().then_some(blank)
    }

    /// The blank item of the ungrouped field at axis position `position` of pivot `pivot` when
    /// `name` is `"(blank)"`, its row label: GETPIVOTDATA takes it as it takes an omitted item
    /// (probes/t4236-pivot-date-groups Calc B15, F34, J11, N12, Z15). On a numeric range or
    /// date-grouped field `"(blank)"` is `#REF!` (probes/t4228-pivot-range-groups,
    /// t4236-pivot-date-groups-no-blank). Matched ignoring case like any item; another case than
    /// `(blank)`'s own, and a field that also holds the text `(blank)` (that item wins here), are
    /// unprobed. Call [`Model::pivot_axis`] first.
    fn pivot_blank_text(&self, pivot: usize, position: usize, name: &str) -> Option<u32> {
        let field = self.pivot_axis_field(pivot, position)?;
        if field.number_groups.is_some()
            || field.date_groups.is_some()
            || !name.eq_ignore_ascii_case(BLANK_ITEM)
        {
            return None;
        }
        self.pivot_blank_item(pivot, position)
    }

    /// The answer of pivot `pivot` to `query`, memoized. Every record the query picks must hold
    /// a number in the data field: an error, blank, text, boolean or date there is `#REF!`
    /// (SPEC-0015 unknown 6). Call [`Model::pivot_axis`] first.
    pub(crate) fn pivot_answer(&mut self, pivot: usize, query: &PivotQuery) -> Result<f64, Error> {
        let Model {
            workbook,
            evaluation,
            ..
        } = self;
        let index = evaluation
            .pivot_indexes
            .pivots
            .get_mut(&pivot)
            .ok_or(Error::REF)?;
        if let Some(answer) = index.memo.get(query) {
            return answer.clone();
        }
        let answer = (|| {
            let table = workbook.pivot_tables.get(pivot).ok_or(Error::REF)?;
            let data = table.data_fields.get(query.data_field).ok_or(Error::REF)?;
            let field = workbook
                .pivot_caches
                .get(table.cache as usize)
                .and_then(|cache| cache.fields.get(data.field as usize))
                .ok_or(Error::REF)?;
            let mut records: Vec<u32> = Vec::new();
            for (tuple, tuple_records) in &index.tuples {
                if tuple
                    .iter()
                    .zip(query.items.iter())
                    .all(|(item, wanted)| wanted.is_none_or(|w| w == *item))
                {
                    records.extend_from_slice(tuple_records);
                }
            }
            records.sort_unstable();
            aggregate(data.function, &record_numbers(field, &records)?)
        })();
        index.memo.insert(query.clone(), answer.clone());
        answer
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;
    use crate::pivot::CacheField;

    /// One field of three groups, the middle one holding no record, over four records.
    fn cache() -> PivotCache {
        PivotCache {
            source: None,
            fields: vec![CacheField {
                name: "Group".to_string(),
                items: vec![CacheItem::Missing; 3],
                records: vec![0, 2, 2, 0],
                number_groups: None,
                date_groups: None,
            }],
            record_count: 4,
        }
    }

    #[test]
    fn the_occupied_set_holds_the_items_with_a_record_and_not_an_empty_group() {
        let index = build(&cache(), vec![0]).unwrap();
        let occupied = index.occupied.first().unwrap();
        assert!(occupied.contains(&0), "a filled group");
        assert!(occupied.contains(&2), "a filled group");
        assert!(!occupied.contains(&1), "an empty group");
        assert_eq!(index.tuples.len(), 2, "one tuple per distinct item");
    }
}
