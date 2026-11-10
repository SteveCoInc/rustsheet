//! The per-pivot state GETPIVOTDATA answers from (SPEC-0015 "Untrusted-input limits", T-2083;
//! T-4154).
//!
//! GETPIVOTDATA answers the value the pivot shows (docs/excel-semantics/web.md "GETPIVOTDATA on
//! other pivot shapes (T-2085)"), so it reads the pivot's [`PivotResult`] (SPEC-0032), the same
//! values the grid writer renders: page fields and hidden items filter its records, its axis
//! trees hold exactly the item tuples the pivot shows, and its totals follow the pivot's
//! summation order and its calculated fields. The result is computed lazily, on a pivot's first
//! GETPIVOTDATA call in an evaluation, within [`MAX_PIVOT_WORK`] and [`MAX_PIVOT_NODES`]
//! (`pivot::result`), so an untrusted pivot is bounded (CLAUDE.md directive 5).
//!
//! The results live in the evaluation state, keyed by the pivot's position in
//! `Workbook::pivot_tables`, not on the pivot itself, so `Workbook`'s `PartialEq` and `Clone` see
//! no derived state. They are dropped at each `Model::evaluate` and whenever pivots are dropped:
//! pivots and caches change only between evaluations, so a result never outlives the pivot it
//! was computed from.
//!
//! [`MAX_PIVOT_NODES`]: crate::pivot::MAX_PIVOT_NODES

use std::collections::HashMap;

use crate::model::Model;
use crate::pivot::{
    compute_on, item_label, AxisField, CacheField, CacheItem, PivotResult, PivotTable,
    MAX_PIVOT_WORK,
};

/// The pivot results of the current evaluation.
#[derive(Default)]
pub(crate) struct PivotIndexes {
    /// By position in `Workbook::pivot_tables`: code that removes or reorders pivots between
    /// evaluations must call [`PivotIndexes::clear`] (`Model::displace_pivots`,
    /// `move_pivots`, `take_pivots_on` and `restore_pivots` do). `None` records a pivot
    /// GETPIVOTDATA does not answer from, so it is not recomputed per call.
    pivots: HashMap<usize, Option<PivotIndex>>,
    /// The `Evaluation::evaluations` count the results were computed in: a new evaluation drops
    /// them (`Model::pivot_index`).
    evaluation: u64,
    /// How many results were computed since the model was made: a test counts the passes over
    /// the records.
    builds: u64,
}

impl PivotIndexes {
    /// Drops every result.
    pub(crate) fn clear(&mut self) {
        self.pivots.clear();
    }

    /// Drops the result of the pivot at position `pivot`, whose definition or cache changed in
    /// place (the grid writer, T-4151).
    pub(crate) fn forget(&mut self, pivot: usize) {
        self.pivots.remove(&pivot);
    }

    /// How many results were computed so far.
    #[cfg(test)]
    pub(crate) fn builds(&self) -> u64 {
        self.builds
    }
}

/// How an item name matches a field's text items, ignoring case.
#[derive(Clone, Copy)]
enum ItemMatch {
    One(u32),
    /// Two text items differ only in case (`east`, `East`): Excel's pivot holds one item for both;
    /// what GETPIVOTDATA does when the cache keeps two is unprobed, so it is `#REF!`.
    Several,
}

/// One pivot's result, and the lookups GETPIVOTDATA makes against it.
pub(crate) struct PivotIndex {
    pub(crate) result: PivotResult,
    /// Per cache field, its text items by lowercased name, built on first use.
    item_names: HashMap<u32, HashMap<String, ItemMatch>>,
}

/// Whether GETPIVOTDATA answers from `pivot`: one data field, or several with the Values field on
/// an axis (web.md "GETPIVOTDATA on other pivot shapes (T-2085)", DF), each shown as itself, and
/// no filter, `AutoShow`, slicer or timeline. Page fields and hidden items are read from the
/// result (LY A1:A12). What GETPIVOTDATA answers on a filtered pivot, slicers and timelines
/// included, is unprobed (UNVERIFIED, T-4799), as is a field shown as a calculation (UNVERIFIED,
/// T-4750): both stay SPEC-0015's `#REF!`.
fn supported(pivot: &PivotTable) -> bool {
    let several = pivot.data_fields.len() > 1;
    !(pivot.data_fields.is_empty()
        || several != values_on_axis(pivot)
        || pivot
            .data_fields
            .iter()
            .any(|d| d.show_data_as != crate::pivot::ShowDataAs::Normal)
        || !pivot.filters.is_empty()
        || pivot
            .fields
            .iter()
            .any(|f| f.slicer.is_some() || f.timeline.is_some() || f.auto_show.is_some()))
}

/// Whether the Values field (`x="-2"`) is on `pivot`'s row or column axis, as it is when the
/// pivot shows several data fields.
pub(crate) fn values_on_axis(pivot: &PivotTable) -> bool {
    pivot
        .row_fields
        .iter()
        .chain(&pivot.column_fields)
        .any(|field| matches!(field, AxisField::Values))
}

impl Model {
    /// The result of pivot `pivot`, computing it on first use in this evaluation; `None` when the
    /// pivot is not one GETPIVOTDATA answers from ([`supported`]), its cache is malformed, or it
    /// passes the work or node limits, all `#REF!`.
    pub(crate) fn pivot_index(&mut self, pivot: usize) -> Option<&mut PivotIndex> {
        let today = self.pivot_today();
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
            let index = (|| {
                let table = workbook.pivot_tables.get(pivot)?;
                if !supported(table) {
                    return None;
                }
                let cache = workbook.pivot_caches.get(table.cache as usize)?;
                let mut work = MAX_PIVOT_WORK;
                let result = compute_on(table, cache, today, &mut work).ok()?;
                Some(PivotIndex {
                    result,
                    item_names: HashMap::new(),
                })
            })();
            indexes.builds += 1;
            indexes.pivots.insert(pivot, index);
        }
        indexes.pivots.get_mut(&pivot)?.as_mut()
    }

    /// The cache field `field` of pivot `pivot`'s cache, and its pivot field.
    fn pivot_cache_field(
        &self,
        pivot: usize,
        field: u32,
    ) -> Option<(&CacheField, Option<&crate::pivot::PivotField>)> {
        let table = self.workbook.pivot_tables.get(pivot)?;
        let cache = self.workbook.pivot_caches.get(table.cache as usize)?;
        Some((
            cache.fields.get(field as usize)?,
            table.fields.get(field as usize),
        ))
    }

    /// The item of cache field `field` of pivot `pivot` that the text `name` names, ignoring
    /// case: a text item by its text; a number, date or boolean item by the text its label shows
    /// ([`item_label`]: `"1"`, `"2.5"`, `"1/1/2024"`, `"TRUE"`, `"true"`; IT Q!A2, A4, A9, B3,
    /// C2, C5), so ISO text and a date's serial as text are `#REF!` (B4, B6); and the blank item
    /// as `"(blank)"` on an ungrouped field ([`Model::pivot_blank_text`]). Call
    /// [`Model::pivot_index`] first.
    pub(crate) fn pivot_item(&mut self, pivot: usize, field: u32, name: &str) -> Option<u32> {
        let lower = name.to_lowercase();
        let found = {
            let Model {
                workbook,
                evaluation,
                ..
            } = self;
            let index = evaluation.pivot_indexes.pivots.get_mut(&pivot)?.as_mut()?;
            let table = workbook.pivot_tables.get(pivot)?;
            let cache_field = workbook
                .pivot_caches
                .get(table.cache as usize)?
                .fields
                .get(field as usize)?;
            let names = index.item_names.entry(field).or_insert_with(|| {
                let num_fmts = &workbook.styles.num_fmts;
                let pivot_field = table.fields.get(field as usize);
                let mut map = HashMap::new();
                for (item, i) in cache_field.items.iter().zip(0u32..) {
                    // An error source cell is an error item (`CacheItem::Error`, as
                    // `pivot::read_source` reads it), so a `"#N/A"` text argument does not name
                    // it; whether Excel's GETPIVOTDATA matches it is UNVERIFIED (probe pending:
                    // T-4640). The blank item is named only as `"(blank)"`, below.
                    let label = match item {
                        CacheItem::Text(text) => text.to_lowercase(),
                        CacheItem::Number(_) | CacheItem::Date(_) | CacheItem::Boolean(_) => {
                            item_label(pivot_field, cache_field, item, num_fmts).to_lowercase()
                        }
                        CacheItem::Error(_) | CacheItem::Missing => continue,
                    };
                    map.entry(label)
                        .and_modify(|m| *m = ItemMatch::Several)
                        .or_insert(ItemMatch::One(i));
                }
                map
            });
            names.get(&lower).copied()
        };
        match found {
            Some(ItemMatch::One(item)) => Some(item),
            Some(ItemMatch::Several) => None,
            None => self.pivot_blank_text(pivot, field, name),
        }
    }

    /// The item of cache field `field` of pivot `pivot` that the number `value` names: on a
    /// numeric range field the group whose lower bound it is
    /// ([`crate::pivot::NumberGroups::lower_bound_index`]: probes/t4228-pivot-range-groups Calc
    /// A35, C30:C46; probes/t4249-pivot-range-group-labels Calc P20:U31), on a date-grouped field
    /// the group of its calendar value ([`crate::pivot::DateGroups::number_item`]:
    /// probes/t4236-pivot-date-groups-no-blank), and otherwise the number or date item equal to it
    /// (IT Q!A1, A3, A5, A6, A8 on a number field; B1, B2, B5 on a date field, where 45292.5 is
    /// `#REF!`, B7; probes/t4236-pivot-date-groups Calc B19, N11). A boolean or text item never
    /// equals a number (IT C4).
    pub(crate) fn pivot_number_item(&self, pivot: usize, field: u32, value: f64) -> Option<u32> {
        let (field, _) = self.pivot_cache_field(pivot, field)?;
        let item = if let Some(groups) = &field.number_groups {
            groups.item(groups.lower_bound_index(value)?)
        } else if let Some(groups) = field.date_groups {
            groups.number_item(value)?
        } else {
            // A date item first: a refreshed cache can keep the source's serials as unused
            // number items beside the date items its records index (IT D!, `When`), and only the
            // date items are shown.
            let equal = |n: &f64| n.total_cmp(&value).is_eq();
            field
                .items
                .iter()
                .position(|item| matches!(item, CacheItem::Date(n) if equal(n)))
                .or_else(|| {
                    field
                        .items
                        .iter()
                        .position(|item| matches!(item, CacheItem::Number(n) if equal(n)))
                })?
        };
        u32::try_from(item).ok()
    }

    /// The boolean item `value` of cache field `field` of pivot `pivot` (IT Q!C1, C3; `TRUE` on a
    /// number field is `#REF!`, A7).
    pub(crate) fn pivot_boolean_item(&self, pivot: usize, field: u32, value: bool) -> Option<u32> {
        let (field, _) = self.pivot_cache_field(pivot, field)?;
        let item = field
            .items
            .iter()
            .position(|item| matches!(item, CacheItem::Boolean(b) if *b == value))?;
        u32::try_from(item).ok()
    }

    /// The blank item (`<m/>`, shown as `(blank)`) of cache field `field` of pivot `pivot`, which
    /// an omitted item argument picks (calipers `pv23_041` `Liability Summary` G5:G7,
    /// `GETPIVOTDATA(...,"Remediation Status",)`); `None` when the field has none.
    pub(crate) fn pivot_blank_item(&self, pivot: usize, field: u32) -> Option<u32> {
        let (field, _) = self.pivot_cache_field(pivot, field)?;
        let mut blanks = (0u32..)
            .zip(&field.items)
            .filter(|(_, item)| matches!(item, CacheItem::Missing))
            .map(|(i, _)| i);
        let blank = blanks.next()?;
        blanks.next().is_none().then_some(blank)
    }

    /// The blank item of the ungrouped cache field `field` of pivot `pivot` when `name` is
    /// `"(blank)"`, its row label: GETPIVOTDATA takes it as it takes an omitted item
    /// (probes/t4236-pivot-date-groups Calc B15, F34, J11, N12, Z15). On a numeric range or
    /// date-grouped field `"(blank)"` is `#REF!` (probes/t4228-pivot-range-groups,
    /// t4236-pivot-date-groups-no-blank). Matched ignoring case like any item; another case than
    /// `(blank)`'s own, and a field that also holds the text `(blank)` (that item wins here), are
    /// unprobed.
    fn pivot_blank_text(&self, pivot: usize, field: u32, name: &str) -> Option<u32> {
        let (cache_field, _) = self.pivot_cache_field(pivot, field)?;
        if cache_field.number_groups.is_some()
            || cache_field.date_groups.is_some()
            || !name.eq_ignore_ascii_case(crate::pivot::BLANK_ITEM)
        {
            return None;
        }
        self.pivot_blank_item(pivot, field)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    /// A pivot of rows Group and the count of Group: one `supported` answers from.
    fn plain_pivot() -> PivotTable {
        let mut table = PivotTable::new(
            "P".into(),
            1,
            crate::pivot::CellArea {
                top: 0,
                left: 0,
                bottom: 3,
                right: 1,
            },
            0,
        );
        table.fields = vec![crate::pivot::PivotField::new(
            Some(crate::pivot::Axis::Row),
            3,
        )];
        table.row_fields = vec![AxisField::Field(0)];
        table.data_fields = vec![crate::pivot::DataField {
            caption: "Count of Group".into(),
            field: 0,
            function: crate::pivot::Aggregate::Count,
            show_data_as: crate::pivot::ShowDataAs::Normal,
            base_field: 0,
            base_item: 0,
            number_format: None,
        }];
        table
    }

    /// A slicer or a timeline filters the pivot's records (T-4188); what GETPIVOTDATA answers on
    /// such a pivot is unprobed, so it is refused, as one with a filter is.
    #[test]
    fn a_pivot_driven_by_a_slicer_or_a_timeline_is_not_answered() {
        assert!(supported(&plain_pivot()));
        let mut sliced = plain_pivot();
        sliced.fields[0].slicer = Some(vec![0]);
        assert!(!supported(&sliced), "a slicer selecting every item");
        let mut timed = plain_pivot();
        timed.fields[0].timeline = crate::pivot::DateSpan::from_serials(1.0, 2.0);
        assert!(timed.fields[0].timeline.is_some());
        assert!(!supported(&timed));
    }

    /// Hidden items and page fields are read from the pivot's result (LY A1:A12), so they no
    /// longer refuse the pivot.
    #[test]
    fn a_pivot_with_a_hidden_item_is_answered() {
        let mut hidden = plain_pivot();
        hidden.fields[0].hidden = vec![0b10];
        assert!(supported(&hidden));
    }
}
