//! Numeric range groups of a pivot field (`<rangePr startNum endNum groupInterval>`, T-4235).
//!
//! One rule decides which group a value falls in, shared by the xlsx importer (which turns the
//! grouped field into its group items) and by GETPIVOTDATA (which picks a group by its lower
//! bound). The rule is Excel's own, probed with excel-graph (probes/t4228-pivot-range-groups;
//! docs/excel-semantics/web.md "Grouped pivot fields import as their group items").

use bitcode::{Decode, Encode};
use thiserror::Error as ThisError;

use super::{CacheField, CacheItem, MAX_ITEMS};
use crate::number_format::{to_excel_precision, to_excel_precision_str};

/// A field grouped by numbers from `start` to `end` in steps of `interval`. Its layout is the
/// `<start` group, the inner groups `[start + k * interval, start + (k + 1) * interval)` and the
/// `>end` group; [`NumberGroups::item`] says which cache item holds each.
#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub struct NumberGroups {
    /// `startNum`.
    pub start: f64,
    /// `endNum`.
    pub end: f64,
    /// `groupInterval`.
    pub interval: f64,
    /// Whether the cache items list the `<start` and `>end` groups first: the order Excel shows a
    /// grouping whose `<groupItems>` it relabeled ([`NumberGroups::relabeled`]).
    pub ends_first: bool,
}

impl NumberGroups {
    /// A grouping of `items_in_file` group items, or `None` for a range no grouping can have (an
    /// interval that is not positive and finite, an end below the start, too many groups).
    /// When the file's item count is not the layout's, Excel keeps the grouping, relabels it and
    /// lists the end groups first (probe pivot 3: six items for the five of 0 to 9 by 3). Only a
    /// list one item too long is probed; any other wrong length is UNVERIFIED (T-4249).
    #[must_use]
    pub fn new(start: f64, end: f64, interval: f64, items_in_file: usize) -> Option<NumberGroups> {
        let mut groups = NumberGroups {
            start,
            end,
            interval,
            ends_first: false,
        };
        let inner = groups.inner_count()?;
        groups.ends_first = items_in_file != inner + 2;
        Some(groups)
    }

    /// How many inner groups the range has: the steps from `start` that reach `end`, counted on
    /// the 15-digit quotient as values are (`0` to `1` by `0.1` has 10). At least one. Probed
    /// only for spans that are whole multiples of the interval; the count for another span (0
    /// to 10 by 3 as four groups, the last `9-12`) is UNVERIFIED (T-4249).
    #[must_use]
    pub fn inner_count(&self) -> Option<usize> {
        let span = self.end - self.start;
        if !(self.interval.is_finite() && self.interval > 0.0 && span.is_finite() && span >= 0.0) {
            return None;
        }
        let steps = to_excel_precision(span / self.interval, 15).ceil().max(1.0);
        // Two more items for the end groups; MAX_ITEMS bounds the labels a relabel allocates.
        (steps <= (MAX_ITEMS - 2) as f64).then_some(steps as usize)
    }

    /// The layout position of `value`: 0 for the `<start` group, 1 to `n` for the inner groups and
    /// `n + 1` for `>end`, where `n` is [`NumberGroups::inner_count`]. Below `start` is the
    /// first group and past `end` the last; `end` itself is the last inner group; any other value
    /// is in step `floor(q)` with q = (value - start) / interval rounded to 15 significant digits
    /// (0.3 by 0.1 is q 2.9999999999999996, step 3), and a step that rounds past the last inner
    /// group is `>end`, not clamped (0.9999999999999999 by 0.1 from 0 to 1 is in `>1`).
    #[must_use]
    pub fn layout_index(&self, value: f64) -> Option<usize> {
        let n = self.inner_count()?;
        if value.is_nan() {
            return None;
        }
        if value < self.start {
            return Some(0);
        }
        if value > self.end {
            return Some(n + 1);
        }
        if value == self.end {
            return Some(n);
        }
        let step = self.quotient(value).floor();
        Some(if step < n as f64 {
            step as usize + 1
        } else {
            n + 1
        })
    }

    /// The layout position of the group whose lower bound is `value`, as GETPIVOTDATA picks a
    /// number item: within the same 15-digit rounding (0.6000000000000001 by 0.1 is `0.6-0.7`),
    /// and `end` is the `>end` group. A number inside a group but not its lower bound, or below
    /// `start` (the `<start` group has none), is `None` (`#REF!`). UNVERIFIED (T-4249): a number
    /// past `end` (`#REF!` here) and one below `end` whose rounded q is the group count (`>end`
    /// here).
    #[must_use]
    pub fn lower_bound_index(&self, value: f64) -> Option<usize> {
        let n = self.inner_count()?;
        if value == self.end {
            return Some(n + 1);
        }
        if value.is_nan() || value < self.start {
            return None;
        }
        let q = self.quotient(value);
        if q.fract() != 0.0 || q > n as f64 {
            return None;
        }
        Some(q as usize + 1)
    }

    /// The cache item holding layout position `index` of a grouping of `n` inner groups.
    #[must_use]
    pub fn item(&self, index: usize, n: usize) -> usize {
        if !self.ends_first {
            return index;
        }
        match index {
            0 => 0,
            i if i > n => 1,
            i => i + 1,
        }
    }

    /// The labels Excel writes for the grouping, in cache-item order (`<0`, `>9`, `0-3`, `3-6`,
    /// `6-9` for 0 to 9 by 3 relabeled): each number as General shows it. Probed for whole
    /// numbers only (pivot 3); decimal labels are UNVERIFIED (T-4236).
    #[must_use]
    pub fn relabeled(&self) -> Option<Vec<String>> {
        let n = self.inner_count()?;
        let bound = |k: usize| to_excel_precision_str(self.start + k as f64 * self.interval);
        let mut labels = vec![String::new(); n + 2];
        for index in 0..n + 2 {
            let label = match index {
                0 => format!("<{}", to_excel_precision_str(self.start)),
                i if i > n => format!(">{}", to_excel_precision_str(self.end)),
                i => format!("{}-{}", bound(i - 1), bound(i)),
            };
            if let Some(slot) = labels.get_mut(self.item(index, n)) {
                *slot = label;
            }
        }
        Some(labels)
    }

    /// (value - start) / interval, rounded to 15 significant digits. The probe fixes the rounding
    /// as relative, not an absolute tolerance on q (by 0.001, 4e-13 below 123 is 123 and 8e-13
    /// below is not); that it rounds q rather than the value is a structural model the probe's
    /// points also allow (T-4235).
    fn quotient(&self, value: f64) -> f64 {
        to_excel_precision((value - self.start) / self.interval, 15)
    }
}

/// Why a field cannot be grouped by numbers.
#[derive(ThisError, Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum NumberGroupError {
    /// The interval is not positive and finite, the end is below the start, or the range has
    /// more groups than a cache can hold.
    #[error("a numeric range group with no valid range")]
    Range,
    /// A base value is text, a boolean or an error.
    #[error("a numeric range group over a value that is not a number")]
    NotANumber,
    /// A record points past the base field's items.
    #[error("a record outside the grouped field's items")]
    BadRecord,
}

/// Field `base` grouped by numbers from `start` to `end` in steps of `interval`, `labels` being
/// the `<groupItems>` labels: the grouped field's items are the labels (Excel's own, end groups
/// first, when their count is not the layout's: [`NumberGroups::new`]) and each record's item is
/// the group its value falls in ([`NumberGroups::layout_index`]). A blank value goes in the first
/// item, as the importer's other groups put it (web.md; UNVERIFIED, T-4236). The one grouping
/// step for numbers, shared by the xlsx importer (`apply_groups`) and `Txn::add_pivot_table`.
///
/// # Errors
/// [`NumberGroupError`] for a bad range, a value that is not a number, or a bad record.
pub fn group_by_numbers(
    base: &CacheField,
    [start, end, interval]: [f64; 3],
    labels: Vec<CacheItem>,
) -> Result<CacheField, NumberGroupError> {
    let groups =
        NumberGroups::new(start, end, interval, labels.len()).ok_or(NumberGroupError::Range)?;
    let n = groups.inner_count().ok_or(NumberGroupError::Range)?;
    let items = if groups.ends_first {
        let labels = groups.relabeled().ok_or(NumberGroupError::Range)?;
        labels.into_iter().map(CacheItem::Text).collect()
    } else {
        labels
    };
    let map = base
        .items
        .iter()
        .map(|item| match item {
            CacheItem::Missing => Ok(0),
            CacheItem::Number(v) | CacheItem::Date(v) => {
                let index = groups
                    .layout_index(*v)
                    .ok_or(NumberGroupError::NotANumber)?;
                // At most MAX_ITEMS groups (`inner_count`), so the item index fits a u32.
                u32::try_from(groups.item(index, n)).map_err(|_| NumberGroupError::Range)
            }
            _ => Err(NumberGroupError::NotANumber),
        })
        .collect::<Result<Vec<u32>, _>>()?;
    let records = base
        .records
        .iter()
        .map(|&i| map.get(i as usize).copied())
        .collect::<Option<Vec<u32>>>()
        .ok_or(NumberGroupError::BadRecord)?;
    Ok(CacheField {
        name: base.name.clone(),
        items,
        records,
        number_groups: Some(groups),
        date_groups: None,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    fn groups(start: f64, end: f64, interval: f64) -> NumberGroups {
        let n = NumberGroups::new(start, end, interval, 0)
            .unwrap()
            .inner_count()
            .unwrap();
        NumberGroups::new(start, end, interval, n + 2).unwrap()
    }

    /// Probe pivot 1 (N1 by 0.1 from 0 to 1): layout position per value; 0 is `<0`, 11 is `>1`.
    #[test]
    fn pivot_1_points_land_where_excel_puts_them() {
        let g = groups(0.0, 1.0, 0.1);
        assert_eq!(g.inner_count(), Some(10));
        for (value, index) in [
            (-0.5, 0),
            (0.0, 1),
            (0.3, 4),
            (0.299_999_999_999_999_93, 4),
            (0.299_999_999_999_999_9, 4),
            (0.299_999_999_999_999, 3),
            (0.299_999_999_999_99, 3),
            (0.299_999_999_999_9, 3),
            (0.299_999_999_999, 3),
            (0.299_999_999_9, 3),
            (0.6, 7),
            (0.7, 8),
            (0.9, 10),
            (0.999_999_999_999_999_9, 11),
            (0.999_999_999_999_999, 10),
            (0.999_999_999_999_99, 10),
            (1.0, 10),
            (1.000_000_000_000_000_2, 11),
            (1.5, 11),
        ] {
            assert_eq!(g.layout_index(value), Some(index), "{value}");
        }
    }

    /// Pivots 2, 4 and 5: from 1, by 3, and the ladder below 0.123 by 0.001.
    #[test]
    fn pivots_2_4_and_5_points_land_where_excel_puts_them() {
        let g = groups(1.0, 2.0, 0.1);
        for (value, index) in [
            (0.5, 0),
            (1.0, 1),
            (1.1, 2),
            (1.299_999_999_999_999_8, 4),
            (1.3, 4),
            (1.7, 8),
            (1.999_999_999_999_999_8, 11),
            (2.0, 10),
            (2.5, 11),
        ] {
            assert_eq!(g.layout_index(value), Some(index), "{value}");
        }
        let g = groups(0.0, 9.0, 3.0);
        for (value, index) in [
            (-1.0, 0),
            (0.0, 1),
            (2.5, 1),
            (2.999_999_999_999_999_6, 2),
            (3.0, 2),
            (8.999_999_999_999_998, 4),
            (9.0, 3),
            (9.5, 4),
        ] {
            assert_eq!(g.layout_index(value), Some(index), "{value}");
        }
        let g = groups(0.0, 1.0, 0.001);
        assert_eq!(g.inner_count(), Some(1000));
        for (value, index) in [
            (0.122_999_999_999_999_21, 123),
            (0.122_999_999_999_999_61, 124),
            (0.122_999_999_999_999_8, 124),
            (0.123, 124),
            (0.5, 501),
        ] {
            assert_eq!(g.layout_index(value), Some(index), "{value}");
        }
    }

    /// GETPIVOTDATA number items (probe Calc C30:C46, A35:A36).
    #[test]
    fn a_number_item_picks_the_group_of_that_lower_bound() {
        let g = groups(0.0, 1.0, 0.1);
        for (value, index) in [
            (0.0, Some(1)),
            (0.2, Some(3)),
            (0.3, Some(4)),
            (0.6, Some(7)),
            (0.600_000_000_000_000_1, Some(7)),
            (0.7, Some(8)),
            (0.700_000_000_000_000_1, Some(8)),
            (0.9, Some(10)),
            (1.0, Some(11)),
            (0.35, None),
            (-0.5, None),
        ] {
            assert_eq!(g.lower_bound_index(value), index, "{value}");
        }
        let g = groups(1.0, 2.0, 0.1);
        assert_eq!(g.lower_bound_index(1.3), Some(4));
        assert_eq!(g.lower_bound_index(1.300_000_000_000_000_3), Some(4));
        assert_eq!(g.lower_bound_index(1.1), Some(2));
        assert_eq!(g.lower_bound_index(1.0), Some(1));
        let g = groups(0.0, 9.0, 3.0);
        assert_eq!(g.lower_bound_index(3.0), Some(2));
        assert_eq!(g.lower_bound_index(4.0), None);
    }

    /// Pivot 3: six file items for a five-item layout; Excel lists `<0`, `>9`, `0-3`, `3-6`, `6-9`.
    #[test]
    fn a_wrong_item_count_relabels_with_the_ends_first() {
        let g = NumberGroups::new(0.0, 9.0, 3.0, 6).unwrap();
        assert!(g.ends_first);
        assert_eq!(
            g.relabeled().unwrap(),
            ["<0", ">9", "0-3", "3-6", "6-9"].map(String::from)
        );
        assert_eq!(g.item(0, 3), 0);
        assert_eq!(g.item(4, 3), 1);
        assert_eq!(g.item(1, 3), 2);
        assert!(!NumberGroups::new(0.0, 9.0, 3.0, 5).unwrap().ends_first);
    }

    #[test]
    fn a_range_no_grouping_can_have_is_refused() {
        for (start, end, interval) in [
            (0.0, 1.0, 0.0),
            (0.0, 1.0, -1.0),
            (1.0, 0.0, 0.1),
            (0.0, f64::INFINITY, 1.0),
            (0.0, 1.0, f64::NAN),
            (0.0, 1e300, 1e-300),
        ] {
            assert_eq!(NumberGroups::new(start, end, interval, 3), None);
        }
        assert_eq!(groups(5.0, 5.0, 1.0).inner_count(), Some(1));
    }

    #[test]
    fn a_field_groups_into_its_labels_and_records() {
        let base = CacheField {
            name: "N".into(),
            items: vec![
                CacheItem::Number(0.3),
                CacheItem::Missing,
                CacheItem::Number(1.5),
            ],
            records: vec![0, 2, 1, 0],
            ..CacheField::default()
        };
        let labels = |n: usize| (0..n).map(|i| CacheItem::Text(i.to_string())).collect();
        let grouped = group_by_numbers(&base, [0.0, 1.0, 0.1], labels(12)).unwrap();
        assert_eq!(grouped.records, [4, 11, 0, 4]);
        assert_eq!(grouped.items, labels(12));
        let relabeled = group_by_numbers(&base, [0.0, 1.0, 0.5], labels(3)).unwrap();
        assert_eq!(
            relabeled.items[..2],
            [CacheItem::Text("<0".into()), CacheItem::Text(">1".into())]
        );
        assert_eq!(relabeled.records, [2, 1, 0, 2]);
        let text = CacheField {
            items: vec![CacheItem::Text("a".into())],
            records: vec![0],
            ..base.clone()
        };
        assert_eq!(
            group_by_numbers(&text, [0.0, 1.0, 0.1], labels(12)),
            Err(NumberGroupError::NotANumber)
        );
        assert_eq!(
            group_by_numbers(&base, [0.0, 1.0, 0.0], labels(3)),
            Err(NumberGroupError::Range)
        );
    }
}
