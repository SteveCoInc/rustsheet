//! Numeric range groups of a pivot field (`<rangePr startNum endNum groupInterval>`, T-4235).
//!
//! One rule decides which group a value falls in, shared by the xlsx importer (which turns the
//! grouped field into its group items) and by GETPIVOTDATA (which picks a group by its lower
//! bound). The rule is Excel's own, probed with excel-graph (probes/t4228-pivot-range-groups,
//! t4253-*, t4249-*; docs/excel-semantics/web.md "Grouped pivot fields import as their group
//! items").

use std::collections::HashMap;

use bitcode::{Decode, Encode};
use thiserror::Error as ThisError;

use super::{CacheField, CacheItem, MAX_ITEMS};
use crate::functions::util::collation::fold_case;
use crate::number_format::{number_to_text, to_excel_precision};

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
    /// Whether the labels are whole-number ones ([`NumberGroups::labels`]): the base data,
    /// `start`, `end` and `interval` are all whole numbers.
    pub whole: bool,
    /// The cache item of each layout position ([`NumberGroups::new`]); empty when position k is
    /// item k.
    pub order: Vec<u32>,
}

impl NumberGroups {
    /// A grouping whose file lists `file_items` as its `<groupItems>` over base data that are all
    /// whole numbers when `whole_data`, or `None` for a range no grouping can have (an interval
    /// that is not positive and finite, an end below the start, too many groups). Whatever the
    /// file lists, Excel writes its own labels ([`NumberGroups::labels`]): the file's items that
    /// are one of those labels keep the file's order and come first, then Excel's other labels
    /// in layout order. A list of the layout's length in wrong text shows the layout order, and
    /// one of Excel's labels permuted keeps the permutation (probes/t4249-pivot-range-group-labels
    /// pivots L and M); a list too short (A, C), two too long (B), a `>10` the layout writes as
    /// `>12` (D) and pivot 3 of t4228, whose `<0` and `>9` match and come first, all follow it
    /// (probes/t4249-pivot-range-group-edges, t4253-pivot-relabel-order).
    #[must_use]
    pub fn new(
        start: f64,
        end: f64,
        interval: f64,
        whole_data: bool,
        file_items: &[CacheItem],
    ) -> Option<NumberGroups> {
        let mut groups = NumberGroups {
            start,
            end,
            interval,
            // A whole `end` too: whole data 0 to 9.5 by 3 reads `0-3` ... `9-12`
            // (probes/t4287-pivot-group-whole-end).
            whole: whole_data
                && start.fract() == 0.0
                && end.fract() == 0.0
                && interval.fract() == 0.0,
            order: Vec::new(),
        };
        let n = groups.inner_count()?;
        let labels = groups.labels(n);
        // A map and a seen-set keep this linear: both counts come from the file and reach
        // MAX_ITEMS. The match ignores case (`>3e+20` matches `>3E+20`) but not spacing (`6 - 9`,
        // ` 3-5` and `3-5 ` match nothing; probes/t4287-pivot-group-label-match). Excel's labels
        // are ASCII, so how a non-ASCII text folds is moot.
        let position: HashMap<String, usize> = labels
            .iter()
            .enumerate()
            .map(|(k, label)| (fold_case(label), k))
            .collect();
        // A label the file lists twice takes its last place: `6-9`, `3-5`, `6-9` shows `3-5`
        // before `6-9`. A non-text item (`<n v="3"/>`) matches nothing and is skipped
        // (probes/t4316-pivot-group-label-twice, t4316-pivot-group-number-item).
        let matching: Vec<usize> = file_items
            .iter()
            .filter_map(|item| match item {
                CacheItem::Text(text) => position.get(&fold_case(text)).copied(),
                _ => None,
            })
            .collect();
        let mut seen = vec![false; labels.len()];
        let mut layout: Vec<usize> = Vec::with_capacity(labels.len());
        let mut last_first: Vec<usize> = Vec::with_capacity(matching.len());
        for &k in matching.iter().rev() {
            if let Some(slot) = seen.get_mut(k) {
                if !*slot {
                    *slot = true;
                    last_first.push(k);
                }
            }
        }
        seen.fill(false);
        for k in last_first.into_iter().rev().chain(0..labels.len()) {
            if let Some(slot) = seen.get_mut(k) {
                if !*slot {
                    *slot = true;
                    layout.push(k);
                }
            }
        }
        if layout.iter().enumerate().all(|(item, &k)| item == k) {
            return Some(groups);
        }
        let mut order = vec![0; labels.len()];
        for (item, &k) in layout.iter().enumerate() {
            // At most MAX_ITEMS labels (`inner_count`), so an item index fits a u32.
            let (Some(slot), Ok(item)) = (order.get_mut(k), u32::try_from(item)) else {
                return None;
            };
            *slot = item;
        }
        groups.order = order;
        Some(groups)
    }

    /// How many inner groups the range has: the steps from `start` that reach `end`, counted on
    /// the 15-digit quotient as values are (`0` to `1` by `0.1` has 10), a partial last step
    /// counting as one (0 to 10 by 3 has four, the last `9-12`: probes/t4249-pivot-range-group-edges
    /// pivot D). At least one.
    #[must_use]
    pub fn inner_count(&self) -> Option<usize> {
        let span = self.end - self.start;
        if !(self.interval.is_finite() && self.interval > 0.0 && span.is_finite() && span >= 0.0) {
            return None;
        }
        let steps = to_excel_precision(span / self.interval, 15).ceil().max(1.0);
        // Two more items for the end groups; MAX_ITEMS bounds the labels a grouping allocates.
        (steps <= (MAX_ITEMS - 2) as f64).then_some(steps as usize)
    }

    /// The layout position of `value`: 0 for the `<start` group, 1 to `n` for the inner groups and
    /// `n + 1` for `>end`, where `n` is [`NumberGroups::inner_count`]. Below `start` is the
    /// first group and past `end` the last, even inside a partial last step (11 by 3 from 0 to 10
    /// is in `>12`, probe pivot D); `end` itself is the last inner group; any other value
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

    /// The layout position GETPIVOTDATA picks for a number item. With q the 15-digit quotient
    /// and k = q truncated toward zero, the number matches when q is not above k: a positive q
    /// must be whole (0.6000000000000001 by 0.1 is `0.6-0.7`; 0.35, and 4 by 3, are `#REF!`),
    /// while a negative one truncates up (by 3 from 0, -1 and -1.5 are `0-3`, -3 and -4.5 are
    /// `<0` and -6 is `#REF!`: probes/t4249-pivot-range-group-labels Calc P20:Q31). k = -1 is the
    /// `<start` group and k = n the `>end` one (its label's bound: 12 for 0 to 10 by 3, where 10
    /// and 11 are `#REF!`, edges Calc P40:Q50); anything else is `None` (`#REF!`).
    #[must_use]
    pub fn lower_bound_index(&self, value: f64) -> Option<usize> {
        let n = self.inner_count()?;
        let q = self.quotient(value);
        let k = q.trunc();
        if q.is_nan() || q > k || k < -1.0 || k > n as f64 {
            return None;
        }
        // -1 <= k <= n <= MAX_ITEMS, so k + 1 is a small whole number.
        Some((k + 1.0) as usize)
    }

    /// The cache item holding layout position `index`.
    #[must_use]
    pub fn item(&self, index: usize) -> usize {
        self.order.get(index).map_or(index, |&item| item as usize)
    }

    /// The labels Excel writes for a grouping of `n` inner groups, in layout order: `<start`, the
    /// inner groups, and `>` followed by the last inner group's upper bound (`>12` for 0 to 10 by
    /// 3, `>10` for 0 to 9 by 2.5). An inner group of a [`NumberGroups::whole`] grouping reads
    /// `lo-(lo + interval - 1)`, the last one ending at `end` when that is larger (`0-2`, `3-5`,
    /// `6-9` for 0 to 9 by 3; `9-11` for 0 to 10); any other reads `lo-(lo + interval)` (`0-3`,
    /// `7.5-10`; probes t4249-pivot-range-group-edges and -labels). Each bound is
    /// `start + k * interval` as text coercion writes it (15 digits, plain decimal: `0.00001`, not
    /// General's `1E-05`; probes/t4253-pivot-decimal-relabel), which holds past it too: 1e15 and
    /// 1e-15 plain, `1E+20`, and a 15-digit tie (probes/t4287-pivot-group-bound-text). The
    /// 20-character plain limit also keeps a label short for a file-controlled range.
    #[must_use]
    pub fn labels(&self, n: usize) -> Vec<String> {
        let bound = |k: usize| self.start + k as f64 * self.interval;
        let upper = |k: usize| {
            if !self.whole {
                bound(k + 1)
            } else if k + 1 == n {
                (bound(k + 1) - 1.0).max(self.end)
            } else {
                bound(k + 1) - 1.0
            }
        };
        let mut labels = Vec::with_capacity(n + 2);
        labels.push(format!("<{}", number_to_text(self.start)));
        labels.extend(
            (0..n).map(|k| format!("{}-{}", number_to_text(bound(k)), number_to_text(upper(k)))),
        );
        labels.push(format!(">{}", number_to_text(bound(n))));
        labels
    }

    /// The labels in cache-item order ([`NumberGroups::new`]).
    #[must_use]
    pub fn items(&self) -> Option<Vec<CacheItem>> {
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
/// the `<groupItems>` labels: the grouped field's items are Excel's own labels, in the order
/// [`NumberGroups::new`] gives, and each record's item is the group its value falls in
/// ([`NumberGroups::layout_index`]); a caller grouping `base` in place keeps its values
/// with [`keep_ungrouped`](super::keep_ungrouped) (T-4734). A blank value goes in the `<start`
/// item, as the importer's other groups put it (web.md; UNVERIFIED, T-4236). The one grouping
/// step for numbers, shared by the xlsx importer (`apply_groups`) and `Txn::add_pivot_table`.
///
/// # Errors
/// [`NumberGroupError`] for a bad range, a value that is not a number, or a bad record.
pub fn group_by_numbers(
    base: &CacheField,
    [start, end, interval]: [f64; 3],
    labels: &[CacheItem],
) -> Result<CacheField, NumberGroupError> {
    let mut whole = true;
    for item in &base.items {
        match item {
            CacheItem::Missing => {}
            CacheItem::Number(v) | CacheItem::Date(v) => whole &= v.fract() == 0.0,
            _ => return Err(NumberGroupError::NotANumber),
        }
    }
    let groups =
        NumberGroups::new(start, end, interval, whole, labels).ok_or(NumberGroupError::Range)?;
    let items = groups.items().ok_or(NumberGroupError::Range)?;
    let map = base
        .items
        .iter()
        .map(|item| {
            let index = match item {
                CacheItem::Number(v) | CacheItem::Date(v) => groups
                    .layout_index(*v)
                    .ok_or(NumberGroupError::NotANumber)?,
                // A blank, the only other item the loop above lets through.
                _ => 0,
            };
            // At most MAX_ITEMS groups (`inner_count`), so the item index fits a u32.
            u32::try_from(groups.item(index)).map_err(|_| NumberGroupError::Range)
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
        added_group: None,
        formula: None,
        ungrouped: None,
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
            whole: false,
            order: Vec::new(),
        }
    }

    fn texts(labels: &[&str]) -> Vec<CacheItem> {
        labels
            .iter()
            .map(|l| CacheItem::Text((*l).to_owned()))
            .collect()
    }

    fn relabeled(g: &NumberGroups) -> Vec<String> {
        g.items()
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

    /// GETPIVOTDATA number items (t4228 probe Calc C30:C46, A35:A36; t4249 edges N40:Q50 and
    /// labels L20:Q27).
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
        for (value, index) in [
            (3.0, Some(2)),
            (4.0, None),
            (9.0, Some(4)),
            (8.999_999_999_999_998, Some(4)),
            (8.999_999_999_999_9, None),
            (9.5, None),
            (12.0, None),
            (-1.0, Some(1)),
            (-1e-6, Some(1)),
            (-1.5, Some(1)),
            (-2.999_999_999_999_999_6, Some(0)),
            (-3.0, Some(0)),
            (-4.5, Some(0)),
            (-6.0, None),
            (f64::NAN, None),
        ] {
            assert_eq!(g.lower_bound_index(value), index, "{value}");
        }
        // A partial last step: `>12` is 12, and `end` (10) is no lower bound.
        let g = groups(0.0, 10.0, 3.0);
        for (value, index) in [
            (9.0, Some(4)),
            (9.000_000_000_000_002, Some(4)),
            (10.0, None),
            (11.0, None),
            (12.0, Some(5)),
            (15.0, None),
            (9.999_999_999_999_998, None),
        ] {
            assert_eq!(g.lower_bound_index(value), index, "{value}");
        }
    }

    /// Excel's own labels: whole-number data (t4249 pivots A, G, K) and the rest (D, H, I, J).
    #[test]
    fn labels_follow_the_data_kind_and_the_last_step() {
        let labels = |s, e, i, whole: bool| {
            let g = NumberGroups::new(s, e, i, whole, &[]).unwrap();
            g.labels(g.inner_count().unwrap())
        };
        assert_eq!(
            labels(0.0, 9.0, 3.0, true),
            ["<0", "0-2", "3-5", "6-9", ">9"]
        );
        assert_eq!(
            labels(0.0, 10.0, 3.0, true),
            ["<0", "0-2", "3-5", "6-8", "9-11", ">12"]
        );
        assert_eq!(
            labels(0.0, 9.0, 3.0, false),
            ["<0", "0-3", "3-6", "6-9", ">9"]
        );
        assert_eq!(
            labels(0.0, 10.0, 3.0, false),
            ["<0", "0-3", "3-6", "6-9", "9-12", ">12"]
        );
        assert_eq!(
            labels(0.0, 9.0, 2.5, true),
            ["<0", "0-2.5", "2.5-5", "5-7.5", "7.5-10", ">10"]
        );
        assert_eq!(
            labels(0.5, 9.5, 3.0, true),
            ["<0.5", "0.5-3.5", "3.5-6.5", "6.5-9.5", ">9.5"]
        );
        let g = groups(0.0, 10.0, 3.0);
        assert_eq!(g.inner_count(), Some(4));
        for (value, index) in [
            (8.999_999_999_999_998, 4),
            (10.0, 4),
            (10.000_000_000_000_002, 5),
            (11.0, 5),
        ] {
            assert_eq!(g.layout_index(value), Some(index), "{value}");
        }
    }

    /// probes/t4316-pivot-group-label-twice and t4316-pivot-group-number-item: a label listed
    /// twice takes its last place, and a non-text item is skipped.
    #[test]
    fn a_repeated_label_takes_its_last_place_and_a_number_is_skipped() {
        let base = CacheField {
            name: "N".into(),
            items: [-1.0, 1.0, 4.0, 7.0, 10.0].map(CacheItem::Number).to_vec(),
            records: (0..5).collect(),
            ..CacheField::default()
        };
        let layout = ["<0", "0-2", "3-5", "6-9", ">9"];
        let num = |v: f64| CacheItem::Number(v);
        let text = |t: &str| CacheItem::Text(t.to_owned());
        for (file, shown) in [
            (
                texts(&["6-9", "3-5", "6-9", "w3", "w4"]),
                ["3-5", "6-9", "<0", "0-2", ">9"],
            ),
            (texts(&["<0", "<0", "0-2", "3-5", "6-9", ">9"]), layout),
            (texts(&["x", "x", "x", "x", "x"]), layout),
            (
                vec![text("<0"), num(0.0), text("3-5"), text("6-9"), text(">9")],
                ["<0", "3-5", "6-9", ">9", "0-2"],
            ),
            (
                vec![num(3.0), text("6-9"), text("3-5"), text("w3"), text("w4")],
                ["6-9", "3-5", "<0", "0-2", ">9"],
            ),
            ([-1.0, 0.0, 3.0, 6.0, 9.0].map(num).to_vec(), layout),
        ] {
            let grouped = group_by_numbers(&base, [0.0, 9.0, 3.0], &file).unwrap();
            assert_eq!(grouped.items, texts(&shown), "{file:?}");
        }
    }

    /// t4249 pivots A, B, C and E: the file's labels that Excel writes keep their place, and
    /// Excel's others follow in layout order.
    #[test]
    fn excel_keeps_the_file_place_of_its_own_labels() {
        let base = CacheField {
            name: "N".into(),
            items: [-1.0, 0.0, 2.0, 3.0, 5.0, 6.0, 8.0, 9.0, 10.0]
                .map(CacheItem::Number)
                .to_vec(),
            records: (0..9).collect(),
            ..CacheField::default()
        };
        for (file, shown) in [
            (
                &["<0", "0-3", "3-6", ">9"][..],
                ["<0", ">9", "0-2", "3-5", "6-9"],
            ),
            (&["<0", "0-3", ">9"][..], ["<0", ">9", "0-2", "3-5", "6-9"]),
            (
                &["<0", "0-2", "3-5", "6-8", "9-11", "12-14", ">9"][..],
                ["<0", "0-2", "3-5", ">9", "6-9"],
            ),
            (
                &["<0", "0-2", "3-5", "6-9", ">9"][..],
                ["<0", "0-2", "3-5", "6-9", ">9"],
            ),
        ] {
            let grouped = group_by_numbers(&base, [0.0, 9.0, 3.0], &texts(file)).unwrap();
            assert_eq!(grouped.items, texts(&shown), "{file:?}");
            let by_record: Vec<&CacheItem> = grouped
                .records
                .iter()
                .map(|&r| &grouped.items[r as usize])
                .collect();
            let want = texts(&["<0", "0-2", "0-2", "3-5", "3-5", "6-9", "6-9", "6-9", ">9"]);
            assert_eq!(by_record, want.iter().collect::<Vec<_>>(), "{file:?}");
        }
        let identity = group_by_numbers(
            &base,
            [0.0, 9.0, 3.0],
            &texts(&["<0", "0-2", "3-5", "6-9", ">9"]),
        );
        assert!(identity.unwrap().number_groups.unwrap().order.is_empty());
    }

    /// Pivot 3: six file items for a five-item layout; Excel lists `<0`, `>9`, `0-3`, `3-6`, `6-9`:
    /// the file's `<0` and `>9` are Excel's labels too, so they come first.
    #[test]
    fn a_wrong_item_count_relabels_matching_items_first() {
        let file = texts(&["<0", "0-2", "3-5", "6-8", "9-11", ">9"]);
        let g = NumberGroups::new(0.0, 9.0, 3.0, false, &file).unwrap();
        assert_eq!(relabeled(&g), ["<0", ">9", "0-3", "3-6", "6-9"]);
        assert_eq!(g.item(0), 0);
        assert_eq!(g.item(4), 1);
        assert_eq!(g.item(1), 2);
        // The layout's length in Excel's own labels keeps them; in another text (t4249 labels
        // pivot L) it shows Excel's labels in layout order; Excel's labels permuted (pivot M) keep
        // the file's order.
        let own = texts(&["<0", "0-3", "3-6", "6-9", ">9"]);
        let kept = NumberGroups::new(0.0, 9.0, 3.0, false, &own).unwrap();
        assert!(kept.order.is_empty());
        assert_eq!(kept.item(4), 4);
        let wrong = NumberGroups::new(0.0, 9.0, 3.0, false, &texts(&["a"; 5])).unwrap();
        assert!(wrong.order.is_empty());
        let permuted = texts(&["<0", "3-6", "0-3", ">9", "6-9"]);
        let g = NumberGroups::new(0.0, 9.0, 3.0, false, &permuted).unwrap();
        assert_eq!(g.items().unwrap(), permuted);
    }

    /// probes/t4253-pivot-relabel-order: by 0.1 from 0 to 1 with 13 file items, all wrong text
    /// but the named ones; the matching ones come first in the file's order.
    #[test]
    fn a_relabel_lists_the_file_s_matching_items_first_in_file_order() {
        let mut file = texts(&["w"; 13]);
        let own = groups(0.0, 1.0, 0.1).labels(10);
        let natural = own.clone();
        assert_eq!(
            relabeled(&NumberGroups::new(0.0, 1.0, 0.1, false, &file).unwrap()),
            natural
        );
        file[0] = CacheItem::Text(">1".into());
        file[12] = CacheItem::Text("<0".into());
        file[5] = CacheItem::Text("0.2-0.3".into());
        let g = NumberGroups::new(0.0, 1.0, 0.1, false, &file).unwrap();
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

    /// A file-controlled range of nearly `MAX_ITEMS` groups with a wrong item count relabels in
    /// linear time, and a huge bound keeps a short label.
    #[test]
    fn a_large_relabel_is_linear_and_its_labels_short() {
        let n = MAX_ITEMS - 2;
        let mut file = texts(&["w"; 4]);
        file.push(CacheItem::Text(format!(">{n}")));
        let g = NumberGroups::new(0.0, n as f64, 1.0, false, &file).unwrap();
        assert_eq!(g.item(n + 1), 0);
        assert_eq!(g.item(0), 1);
        let huge = groups(0.0, 1e300, 1e299).labels(10);
        assert!(huge.iter().all(|l| l.len() < 40), "{huge:?}");
    }

    /// probes/t4287-pivot-group-label-match: a file item matches one of Excel's labels ignoring
    /// case (pivot R's `>3e+20`) but not spacing (pivot P's `6 - 9` and ` 3-5`, Q's `3-5 `).
    #[test]
    fn a_file_label_matches_ignoring_case_but_not_spacing() {
        let file = texts(&["6 - 9", "<0", " 3-5", "0-2", ">9"]);
        let g = NumberGroups::new(0.0, 9.0, 3.0, true, &file).unwrap();
        assert_eq!(relabeled(&g), ["<0", "0-2", ">9", "3-5", "6-9"]);
        let file = texts(&[">9", "6-9", "3-5 ", "0-2", "<0"]);
        let g = NumberGroups::new(0.0, 9.0, 3.0, true, &file).unwrap();
        assert_eq!(relabeled(&g), [">9", "6-9", "0-2", "<0", "3-5"]);
        let file = texts(&[">3e+20", "2e+20-3e+20", "1e+20-2e+20", "0-1e+20", "<0"]);
        let g = NumberGroups::new(0.0, 3e20, 1e20, false, &file).unwrap();
        assert_eq!(
            relabeled(&g),
            [">3E+20", "2E+20-3E+20", "1E+20-2E+20", "0-1E+20", "<0"]
        );
    }

    /// probes/t4287-pivot-group-whole-end: whole data with an `end` that is not whole gets the
    /// `lo-(lo + interval)` labels.
    #[test]
    fn a_fractional_end_turns_whole_labels_off() {
        let labels = |e| {
            let g = NumberGroups::new(0.0, e, 3.0, true, &[]).unwrap();
            g.labels(g.inner_count().unwrap())
        };
        assert_eq!(labels(9.5), ["<0", "0-3", "3-6", "6-9", "9-12", ">12"]);
        assert_eq!(labels(8.5), ["<0", "0-3", "3-6", "6-9", ">9"]);
        assert_eq!(labels(7.5), ["<0", "0-3", "3-6", "6-9", ">9"]);
    }

    /// probes/t4287-pivot-group-bound-text: 1e15 and 1e-15 are plain, 1e20 is `1E+20`, and a
    /// 15-digit tie (100000000000000.5) writes its lower neighbour.
    #[test]
    fn large_tiny_and_tie_bounds_write_as_coercion_does() {
        let cases: [(f64, f64, f64, &str); 5] = [
            (0.0, 4e15, 1e15, "1000000000000000-2000000000000000"),
            (0.0, 3e20, 1e20, "1E+20-2E+20"),
            (0.0, 4e-6, 1e-6, "0.000001-0.000002"),
            (0.0, 4e-15, 1e-15, "0.000000000000001-0.000000000000002"),
            (
                100_000_000_000_000.5,
                100_000_000_000_003.5,
                1.0,
                "100000000000000-100000000000001",
            ),
        ];
        for (start, end, interval, inner) in cases {
            let g = groups(start, end, interval);
            let labels = g.labels(g.inner_count().unwrap());
            assert!(labels.iter().any(|l| l == inner), "{labels:?}");
        }
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
            assert_eq!(NumberGroups::new(start, end, interval, false, &[]), None);
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
        let labels = |n: usize| -> Vec<CacheItem> {
            (0..n).map(|i| CacheItem::Text(i.to_string())).collect()
        };
        // Placeholder labels give way to Excel's own: 0.3 is in `0.3-0.4`, 1.5 past the end.
        let grouped = group_by_numbers(&base, [0.0, 1.0, 0.1], &labels(12)).unwrap();
        assert_eq!(grouped.records, [4, 11, 0, 4]);
        assert_eq!(
            grouped.items.get(4),
            Some(&CacheItem::Text("0.3-0.4".into()))
        );
        // A blank follows `<0` to its cache item when the file lists that label later.
        let moved = group_by_numbers(&base, [0.0, 1.0, 0.5], &texts(&[">1", "<0"])).unwrap();
        assert_eq!(moved.items, texts(&[">1", "<0", "0-0.5", "0.5-1"]));
        assert_eq!(moved.records, [2, 0, 1, 2]);
        let relabeled = group_by_numbers(&base, [0.0, 1.0, 0.5], &labels(3)).unwrap();
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
            group_by_numbers(&text, [0.0, 1.0, 0.1], &labels(12)),
            Err(NumberGroupError::NotANumber)
        );
        assert_eq!(
            group_by_numbers(&base, [0.0, 1.0, 0.0], &labels(3)),
            Err(NumberGroupError::Range)
        );
    }
}
