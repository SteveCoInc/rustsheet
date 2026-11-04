//! Numeric range groups of a pivot field (`<rangePr startNum endNum groupInterval>`, T-4235).
//!
//! One rule decides which group a value falls in, shared by the xlsx importer (which turns the
//! grouped field into its group items) and by GETPIVOTDATA (which picks a group by its lower
//! bound). The rule is Excel's own, probed with excel-graph (probes/t4228-pivot-range-groups;
//! docs/excel-semantics/web.md "Grouped pivot fields import as their group items").

use bitcode::{Decode, Encode};
use thiserror::Error as ThisError;

use super::{CacheField, CacheItem, MAX_ITEMS};
use crate::number_format::to_excel_precision;

/// A field grouped by numbers from `start` to `end` in steps of `interval`. Its layout is the
/// `<start` group, the inner groups `[start + k * interval, start + (k + 1) * interval)` and the
/// `>end` group; [`NumberGroups::item`] says which cache item holds each.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct NumberGroups {
    /// `startNum`.
    pub start: f64,
    /// `endNum`.
    pub end: f64,
    /// `groupInterval`.
    pub interval: f64,
    /// The cache item of each layout position when Excel relabeled the file's `<groupItems>`
    /// ([`NumberGroups::new`]); empty when it kept them, where position k is item k.
    pub relabeled: Vec<u32>,
}

impl NumberGroups {
    /// A grouping whose file lists `file_items` as its `<groupItems>`, or `None` for a range no
    /// grouping can have (an interval that is not positive and finite, an end below the start,
    /// too many groups). When the file's item count is not the layout's, Excel keeps the grouping
    /// and relabels it with its own labels ([`NumberGroups::labels`]): the file's items that are
    /// one of those labels come first, in the file's order, then the other labels in layout
    /// order (probes/t4253-pivot-relabel-order; probe pivot 3 of t4228, whose `<0` and `>9`
    /// match, lists `<0`, `>9`, `0-3`, `3-6`, `6-9`). Only a list one item too long is probed;
    /// any other wrong length is UNVERIFIED (T-4249).
    #[must_use]
    pub fn new(
        start: f64,
        end: f64,
        interval: f64,
        file_items: &[CacheItem],
    ) -> Option<NumberGroups> {
        let mut groups = NumberGroups {
            start,
            end,
            interval,
            relabeled: Vec::new(),
        };
        let n = groups.inner_count()?;
        if file_items.len() == n + 2 {
            return Some(groups);
        }
        let labels = groups.labels(n);
        let mut order: Vec<usize> = Vec::with_capacity(labels.len());
        for item in file_items {
            let CacheItem::Text(text) = item else {
                continue;
            };
            if let Some(k) = labels.iter().position(|l| l == text) {
                if !order.contains(&k) {
                    order.push(k);
                }
            }
        }
        for k in 0..labels.len() {
            if !order.contains(&k) {
                order.push(k);
            }
        }
        let mut relabeled = vec![0; labels.len()];
        for (item, &k) in order.iter().enumerate() {
            // At most MAX_ITEMS labels (`inner_count`), so an item index fits a u32.
            let (Some(slot), Ok(item)) = (relabeled.get_mut(k), u32::try_from(item)) else {
                return None;
            };
            *slot = item;
        }
        groups.relabeled = relabeled;
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
    pub fn item(&self, index: usize) -> usize {
        self.relabeled
            .get(index)
            .map_or(index, |&item| item as usize)
    }

    /// The labels Excel writes for a grouping of `n` inner groups, in layout order (`<0`, `0-3`,
    /// `3-6`, `6-9`, `>9` for 0 to 9 by 3): `<start`, `bound(k)-bound(k+1)` and `>end`, each bound
    /// `start + k * interval` (probes/t4253-pivot-decimal-relabel).
    #[must_use]
    pub fn labels(&self, n: usize) -> Vec<String> {
        let bound = |k: usize| label_number(self.start + k as f64 * self.interval);
        let mut labels = Vec::with_capacity(n + 2);
        labels.push(format!("<{}", label_number(self.start)));
        labels.extend((0..n).map(|k| format!("{}-{}", bound(k), bound(k + 1))));
        labels.push(format!(">{}", label_number(self.end)));
        labels
    }

    /// The labels of a relabeled grouping in cache-item order ([`NumberGroups::new`]), or `None`
    /// when Excel kept the file's.
    #[must_use]
    pub fn relabeled_items(&self) -> Option<Vec<CacheItem>> {
        if self.relabeled.is_empty() {
            return None;
        }
        let n = self.inner_count()?;
        let mut items = vec![CacheItem::Missing; n + 2];
        for (k, label) in self.labels(n).into_iter().enumerate() {
            *items.get_mut(self.item(k))? = CacheItem::Text(label);
        }
        Some(items)
    }

    /// (value - start) / interval, rounded to 15 significant digits. The probe fixes the rounding
    /// as relative, not an absolute tolerance on q (by 0.001, 4e-13 below 123 is 123 and 8e-13
    /// below is not); that it rounds q rather than the value is a structural model the probe's
    /// points also allow (T-4235).
    fn quotient(&self, value: f64) -> f64 {
        to_excel_precision((value - self.start) / self.interval, 15)
    }
}

/// A group bound as a relabel writes it: rounded to 15 significant digits and printed in plain
/// decimal, never in General's scientific form (`0.00001-0.00002` by 1e-5, `0-0.333333333333333`
/// by 1/3, `49382715604.8-61728394506`; probes/t4253-pivot-decimal-relabel). A bound of 1e15 or
/// more, or below 1e-5, is UNVERIFIED (T-4253 Outcome).
fn label_number(value: f64) -> String {
    // `+ 0.0` turns -0 into 0; Rust's `Display` of an f64 is plain decimal (shortest round trip).
    format!("{}", to_excel_precision(value, 15) + 0.0)
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
/// the `<groupItems>` labels: the grouped field's items are the labels (Excel's own, in the order
/// [`NumberGroups::new`] gives, when their count is not the layout's) and each record's item is
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
    let groups = NumberGroups::new(start, end, interval, &labels).ok_or(NumberGroupError::Range)?;
    let items = if groups.relabeled.is_empty() {
        labels
    } else {
        groups.relabeled_items().ok_or(NumberGroupError::Range)?
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
                u32::try_from(groups.item(index)).map_err(|_| NumberGroupError::Range)
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
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    fn groups(start: f64, end: f64, interval: f64) -> NumberGroups {
        NumberGroups {
            start,
            end,
            interval,
            relabeled: Vec::new(),
        }
    }

    fn texts(labels: &[&str]) -> Vec<CacheItem> {
        labels
            .iter()
            .map(|l| CacheItem::Text((*l).to_owned()))
            .collect()
    }

    fn relabeled(g: &NumberGroups) -> Vec<String> {
        g.relabeled_items()
            .unwrap()
            .into_iter()
            .map(|item| match item {
                CacheItem::Text(t) => t,
                _ => String::new(),
            })
            .collect()
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

    /// Pivot 3: six file items for a five-item layout; Excel lists `<0`, `>9`, `0-3`, `3-6`, `6-9`:
    /// the file's `<0` and `>9` are Excel's labels too, so they come first.
    #[test]
    fn a_wrong_item_count_relabels_matching_items_first() {
        let file = texts(&["<0", "0-2", "3-5", "6-8", "9-11", ">9"]);
        let g = NumberGroups::new(0.0, 9.0, 3.0, &file).unwrap();
        assert_eq!(relabeled(&g), ["<0", ">9", "0-3", "3-6", "6-9"]);
        assert_eq!(g.item(0), 0);
        assert_eq!(g.item(4), 1);
        assert_eq!(g.item(1), 2);
        let kept = NumberGroups::new(0.0, 9.0, 3.0, &file[1..]).unwrap();
        assert!(kept.relabeled.is_empty());
        assert_eq!(kept.item(4), 4);
    }

    /// probes/t4253-pivot-relabel-order: by 0.1 from 0 to 1 with 13 file items, all wrong text
    /// but the named ones; the matching ones come first in the file's order.
    #[test]
    fn a_relabel_lists_the_file_s_matching_items_first_in_file_order() {
        let mut file = texts(&["w"; 13]);
        let own = groups(0.0, 1.0, 0.1).labels(10);
        let natural = own.clone();
        assert_eq!(
            relabeled(&NumberGroups::new(0.0, 1.0, 0.1, &file).unwrap()),
            natural
        );
        file[0] = CacheItem::Text(">1".into());
        file[12] = CacheItem::Text("<0".into());
        file[5] = CacheItem::Text("0.2-0.3".into());
        let g = NumberGroups::new(0.0, 1.0, 0.1, &file).unwrap();
        let first = [">1", "0.2-0.3", "<0"];
        let mut expected = first.to_vec();
        expected.extend(
            own.iter()
                .map(String::as_str)
                .filter(|l| !first.contains(l)),
        );
        assert_eq!(relabeled(&g), expected);
        // A record's layout position still finds its relabeled item.
        assert_eq!(g.item(11), 0);
        assert_eq!(g.item(3), 1);
        assert_eq!(g.item(0), 2);
        assert_eq!(g.item(1), 3);
    }

    /// probes/t4253-pivot-decimal-relabel: the bounds are 15-digit numbers in plain decimal.
    #[test]
    fn relabel_bounds_are_plain_decimals() {
        let cases: [(f64, f64, f64, usize, &str, &str); 8] = [
            (0.0, 1.0, 0.1, 10, "0.3-0.4", ">1"),
            (1.1, 2.1, 0.1, 10, "1.4-1.5", ">2.1"),
            (0.0, 0.01, 0.001, 10, "0.009-0.01", ">0.01"),
            (-1.0, 1.0, 0.25, 8, "-0.25-0", "<-1"),
            (0.0, 0.0001, 0.00001, 10, "0.00001-0.00002", ">0.0001"),
            (0.0, 1.5, 0.3, 5, "0.6-0.9", ">1.5"),
            (
                0.0,
                1.0,
                1.0 / 3.0,
                3,
                "0.333333333333333-0.666666666666667",
                ">1",
            ),
            (
                0.0,
                123_456_789_012.0,
                12_345_678_901.2,
                10,
                "49382715604.8-61728394506",
                ">123456789012",
            ),
        ];
        for (start, end, interval, n, inner, end_label) in cases {
            let g = groups(start, end, interval);
            assert_eq!(g.inner_count(), Some(n), "{start} {end} {interval}");
            let labels = g.labels(n);
            assert!(labels.iter().any(|l| l == inner), "{labels:?}");
            assert!(labels.iter().any(|l| l == end_label), "{labels:?}");
        }
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
            assert_eq!(NumberGroups::new(start, end, interval, &[]), None);
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
            number_groups: None,
        };
        let labels = |n: usize| (0..n).map(|i| CacheItem::Text(i.to_string())).collect();
        let grouped = group_by_numbers(&base, [0.0, 1.0, 0.1], labels(12)).unwrap();
        assert_eq!(grouped.records, [4, 11, 0, 4]);
        assert_eq!(grouped.items, labels(12));
        let relabeled = group_by_numbers(&base, [0.0, 1.0, 0.5], labels(3)).unwrap();
        // No file label is one of Excel's, so the relabel keeps the layout order.
        assert_eq!(
            relabeled.items,
            ["<0", "0-0.5", "0.5-1", ">1"].map(|l| CacheItem::Text(l.into()))
        );
        assert_eq!(relabeled.records, [1, 3, 0, 1]);
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
