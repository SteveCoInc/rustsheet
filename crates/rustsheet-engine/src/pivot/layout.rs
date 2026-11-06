//! The compact-form layout of a pivot with one row field and one data field, and the aggregation
//! it and GETPIVOTDATA share (T-4235).
//!
//! A grouped spec pivot has no rendered cells (the case builder cannot know Excel's groups), so the
//! engine lays it out as Excel does on load. The layout is the case builder's
//! (`tools/src/rustsheet_tools/pivot.py` `cells`, SPEC-0015 "Spec cases in the eval adapter"):
//! `Row Labels` and the data field's caption, one row per shown item, `Grand Total`. Oracle reads
//! of Excel's own layout hold it in step (probes/t4228-pivot-range-groups,
//! probes/t4235-pivot-range-rounding).

use crate::expressions::token::Error;
use crate::functions::statistical::variance::{excel_variance, VarianceKind};

use super::{Aggregate, CacheField, CacheItem, PivotCache};

/// The caption above a compact-form pivot's row labels.
pub const ROW_LABELS: &str = "Row Labels";
/// The label of a blank item's row, and the text GETPIVOTDATA takes for that item on an
/// ungrouped field (probes/t4236-pivot-date-groups).
pub const BLANK_ITEM: &str = "(blank)";
/// The label of a compact-form pivot's grand-total row.
pub const GRAND_TOTAL: &str = "Grand Total";

/// One cell of a laid-out pivot.
#[derive(Debug, PartialEq, Clone)]
pub enum LayoutCell {
    /// A caption or an item label.
    Text(String),
    /// An item's or the grand total's value.
    Number(f64),
    /// The value of a shown item with no record (`showAll="1"`): an empty cell
    /// (probes/t4311-pivot-showall-column-and-groups Calc J2, J4, J7).
    Blank,
    /// A value with no number: the aggregate's error.
    Error(Error),
}

/// How many columns wide a [`row_layout`] row is: the label and the value.
pub const LAYOUT_COLUMNS: usize = 2;

/// The most rows [`row_layout`] gives over `record_count` records: the captions, at most one item
/// per record, and the grand total; with `showAll` on, every one of the row field's `items`
/// instead. [`LAYOUT_COLUMNS`] wide.
#[must_use]
pub fn max_layout_rows(record_count: u32, show_all_items: Option<u32>) -> u32 {
    show_all_items.unwrap_or(record_count).saturating_add(2)
}

/// The area a layout of `rows` rows covers from the cell `(top, left)`, as `(bottom, right)`: the
/// one definition of a grouped pivot's footprint, shared by the facade's layout and the eval
/// harness's input set, which passes [`max_layout_rows`]. Saturates, and `rows` of 0 covers the
/// top row.
#[must_use]
pub fn layout_extent(top: u32, left: u32, rows: u32) -> (u32, u32) {
    let columns = u32::try_from(LAYOUT_COLUMNS).unwrap_or(u32::MAX);
    (
        top.saturating_add(rows.saturating_sub(1)),
        left.saturating_add(columns - 1),
    )
}

/// The rows of a compact-form pivot over `cache` with row field `row` and one data field captioned
/// `caption` summarizing field `data` by `function`, two cells each: the captions, per item of
/// `row` that holds a record (in item order) its label and value (a date or number as its value,
/// the blank as `(blank)`), then the grand total. An item
/// with no record is not shown with `show_all` off (`showAll="0"`, the builder's and Excel's
/// default for a new pivot; probes/t4228-pivot-range-groups), and is shown with a blank value with
/// it on, a group below `start` or past `end` too (probes/t4311-pivot-showall-column-and-groups
/// Calc I1:J10). `None` when a field is not in the cache, a record is malformed or a shown item is
/// not text, a number, a date or the blank.
#[must_use]
pub fn row_layout(
    cache: &PivotCache,
    [row, data]: [u32; 2],
    function: Aggregate,
    caption: &str,
    show_all: bool,
) -> Option<Vec<[LayoutCell; LAYOUT_COLUMNS]>> {
    let row_field = cache.fields.get(row as usize)?;
    let data_field = cache.fields.get(data as usize)?;
    let mut by_item: Vec<Vec<u32>> = vec![Vec::new(); row_field.items.len()];
    for record in 0..cache.record_count {
        let item = *row_field.records.get(record as usize)?;
        by_item.get_mut(item as usize)?.push(record);
    }
    let value = |records: &[u32]| match record_numbers(data_field, records)
        .and_then(|values| aggregate(function, &values))
    {
        Ok(n) => LayoutCell::Number(n),
        Err(error) => LayoutCell::Error(error),
    };
    let mut rows = vec![[
        LayoutCell::Text(ROW_LABELS.to_string()),
        LayoutCell::Text(caption.to_string()),
    ]];
    for (item, records) in row_field.items.iter().zip(&by_item) {
        if records.is_empty() && !show_all {
            continue;
        }
        // A date is its serial, as Excel writes the label cell of an ungrouped date field
        // (probes/t4236-pivot-date-groups reads `43496.9999884259`), and a blank is `(blank)`.
        let label = match item {
            CacheItem::Text(label) => LayoutCell::Text(label.clone()),
            CacheItem::Date(serial) => LayoutCell::Number(*serial),
            // An ungrouped number (a numeric grouping Excel's refresh dropped,
            // probes/t4287-pivot-group-whole-end pivot S) is its value too.
            CacheItem::Number(value) => LayoutCell::Number(*value),
            CacheItem::Missing => LayoutCell::Text(BLANK_ITEM.to_string()),
            _ => return None,
        };
        rows.push([
            label,
            if records.is_empty() {
                LayoutCell::Blank
            } else {
                value(records)
            },
        ]);
    }
    let all: Vec<u32> = (0..cache.record_count).collect();
    rows.push([LayoutCell::Text(GRAND_TOTAL.to_string()), value(&all)]);
    Some(rows)
}

/// The data field `field`'s numbers in `records` (cache order). Every record must hold a number
/// there: an error, blank, text, boolean or date is `#REF!` (SPEC-0015 unknown 6).
pub(crate) fn record_numbers(field: &CacheField, records: &[u32]) -> Result<Vec<f64>, Error> {
    records
        .iter()
        .map(|&record| {
            match field
                .records
                .get(record as usize)
                .and_then(|&i| field.items.get(i as usize))
            {
                Some(CacheItem::Number(n)) => Ok(*n),
                _ => Err(Error::REF),
            }
        })
        .collect()
}

/// The data field's aggregate over `values` (cache order). A Sum, Count, Average, Max or Min over
/// nothing is 0 (probes/getpivotdata-real-pivot H13, probes/getpivotdata-empty-intersection);
/// any other aggregate with no defined value is `#REF!` (SPEC-0015 unknown 2), and a non-finite
/// result `#NUM!` (unknown 16). Var and `StdDev` reuse the worksheet functions' algorithm
/// (`excel_variance`); that the pivot computes them alike is UNVERIFIED (unknown 15).
pub(crate) fn aggregate(function: Aggregate, values: &[f64]) -> Result<f64, Error> {
    let n = values.len();
    let minimum = match function {
        Aggregate::Sum
        | Aggregate::Count
        | Aggregate::Average
        | Aggregate::Max
        | Aggregate::Min => 0,
        Aggregate::StdDev | Aggregate::Var => 2,
        _ => 1,
    };
    if n < minimum {
        return Err(Error::REF);
    }
    if n == 0 {
        return Ok(0.0);
    }
    let sum = || values.iter().fold(0.0, |acc, x| acc + x);
    let variance = |kind| excel_variance(values, kind);
    let value = match function {
        Aggregate::Sum => sum(),
        Aggregate::Count | Aggregate::CountNums => n as f64,
        Aggregate::Average => sum() / n as f64,
        Aggregate::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        Aggregate::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
        Aggregate::Product => values.iter().fold(1.0, |acc, x| acc * x),
        Aggregate::Var => variance(VarianceKind::Sample),
        Aggregate::VarP => variance(VarianceKind::Population),
        Aggregate::StdDev => variance(VarianceKind::Sample).sqrt(),
        Aggregate::StdDevP => variance(VarianceKind::Population).sqrt(),
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::NUM)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_extent_is_the_rows_below_and_the_columns_right_of_the_top_left_cell() {
        assert_eq!(layout_extent(4, 3, 5), (8, 4));
        assert_eq!(
            layout_extent(4, 3, 0),
            (4, 4),
            "no rows still covers the top row"
        );
        assert_eq!(layout_extent(u32::MAX, u32::MAX, 9), (u32::MAX, u32::MAX));
    }
}
