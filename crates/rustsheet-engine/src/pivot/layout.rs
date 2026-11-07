//! A pivot's layout: in [`lines`], its layout lines and location (SPEC-0032 stage 7, T-4147),
//! in [`render`], the cells they give (T-4149), in [`records_behind`], the records behind one
//! value cell (T-4184), and in [`chart_data`], what a pivot chart plots (T-4186); and the
//! aggregation GETPIVOTDATA reads until it moves onto the `PivotResult` (T-4154).
//!
//! There is one layout path. A grouped spec pivot (T-4235), which has no rendered cells because
//! the case builder cannot know Excel's groups, is computed, laid out and rendered by the same
//! stages as any other pivot (the facade's `lay_out_grouped`); oracle reads of Excel's own
//! layout hold it in step (probes/t4228-pivot-range-groups, probes/t4235-pivot-range-rounding,
//! probes/t4311-pivot-showall-column-and-groups).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_precision_loss)]
// LINT-DEBT-END

mod chart;
mod lines;
mod records;
mod render;

pub use chart::{chart_data, ChartCategory, ChartSeries, PivotChartData, TOTAL_SERIES};
pub use lines::{lay_out, AxisLine, PivotLayout, MAX_PIVOT_CELLS};
pub use records::records_behind;
pub(super) use render::item_text;
pub use render::{
    render, PivotCell, RenderedCell, ALL_ITEMS, BLANK_ITEM, COLUMN_LABELS, GRAND_TOTAL,
    MAX_PIVOT_TEXT_BYTES, MULTIPLE_ITEMS, ROW_LABELS, VALUES,
};

use crate::expressions::token::Error;
use crate::functions::statistical::variance::{excel_variance, VarianceKind};

use super::{Aggregate, CacheField, CacheItem};

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

/// The data field's aggregate over `values` (cache order). Every aggregate over nothing is 0, as
/// the pivot's blank intersection cell is (probes/getpivotdata-real-pivot H13,
/// probes/getpivotdata-empty-intersection, probes/t2085-getpivotdata-data-fields Q!B6:J6); Var and
/// `StdDev` over one value are `#DIV/0!` (DF Q!G5, I5), and a non-finite result is `#NUM!` (DF
/// Q!K1:L3). Var and `StdDev` reuse the worksheet functions' algorithm (`excel_variance`); DF
/// Q!G1:J4 match it (unknown 15). The pivot result (`pivot::result`, T-4142) uses the textbook
/// one-pass formula instead (probes/t4616-pivot-total-order-3), so until T-4154 moves
/// GETPIVOTDATA onto the `PivotResult` the two may differ in the last bits for Var and `StdDev`.
pub(crate) fn aggregate(function: Aggregate, values: &[f64]) -> Result<f64, Error> {
    let n = values.len();
    if n == 0 {
        return Ok(0.0);
    }
    if n == 1 && matches!(function, Aggregate::StdDev | Aggregate::Var) {
        return Err(Error::DIV);
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

    /// probes/t2085-getpivotdata-data-fields Q!B6:J6, G5:J5, K1:L3 (T-4804).
    #[test]
    fn aggregates_over_none_one_and_an_overflow() {
        let every = [
            Aggregate::Sum,
            Aggregate::Count,
            Aggregate::CountNums,
            Aggregate::Average,
            Aggregate::Max,
            Aggregate::Min,
            Aggregate::Product,
            Aggregate::StdDev,
            Aggregate::StdDevP,
            Aggregate::Var,
            Aggregate::VarP,
        ];
        for function in every {
            assert_eq!(aggregate(function, &[]), Ok(0.0), "{function:?} over none");
        }
        assert_eq!(aggregate(Aggregate::StdDev, &[2.0]), Err(Error::DIV));
        assert_eq!(aggregate(Aggregate::Var, &[2.0]), Err(Error::DIV));
        assert_eq!(aggregate(Aggregate::StdDevP, &[2.0]), Ok(0.0));
        assert_eq!(aggregate(Aggregate::VarP, &[2.0]), Ok(0.0));
        assert_eq!(aggregate(Aggregate::Sum, &[1e308, 1e308]), Err(Error::NUM));
        assert_eq!(
            aggregate(Aggregate::Product, &[1e200, 1e200]),
            Err(Error::NUM)
        );
        assert_eq!(aggregate(Aggregate::Sum, &[1e200, 1e200]), Ok(2e200));
    }
}
