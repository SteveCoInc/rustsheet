//! The accumulator of one pivot result cell, and Excel's rules for the eleven aggregates over it
//! (SPEC-0032 "`PivotResult`" stage 3, T-4142).
//!
//! A cell is fed its records' values in the result's accumulation order ([`super::result`]), and
//! keeps what every aggregate needs as running folds, so a cell costs the same whatever its
//! record count: the sum, the sum of squares and the product (plain left-to-right folds), the
//! extremes, the two counts and the first error. The rules are docs/excel-semantics/pivots.md's
//! "Aggregates over text, blanks, booleans and errors", "Summation order of Sum, Average and
//! Product totals" and "A Count total over an error field is the error when the pivot mixes
//! aggregates" (probes/t4135-pivot-aggregates,
//! t4589-pivot-total-order, t4614-pivot-total-order-2, t4616-pivot-total-order-3).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![cfg_attr(test, expect(clippy::unreadable_literal))]
// LINT-DEBT-END

use crate::expressions::token::Error;

use super::{Aggregate, CacheItem};

/// The sum of `cells` in order from 0, or the first error among them: how a Sum total adds its
/// children and a grand-grand total its row totals.
pub(crate) fn sum_in_order(
    cells: impl IntoIterator<Item = Result<f64, Error>>,
) -> Result<f64, Error> {
    cells
        .into_iter()
        .try_fold(0.0, |acc, cell| cell.map(|n| acc + n))
}

/// The order-dependent folds of an [`Accumulator`]: what a partial cell adds to a total.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Folds {
    sum: f64,
    squares: f64,
    product: f64,
}

/// What the records of one (row node, column node, data field) hold so far.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Accumulator {
    /// Values that are not blank: numbers, text, booleans and errors (Count).
    values: u32,
    /// Numbers (`CountNums`, and the divisor of Average and the variances).
    numbers: u32,
    sum: f64,
    squares: f64,
    product: f64,
    max: f64,
    min: f64,
    /// The first error fed, which poisons every aggregate but the counts. Of two different
    /// errors the first in accumulation order wins (UNVERIFIED, T-4633).
    error: Option<Error>,
    /// The sum of the child cells' Sums, for a total of a pivot whose data fields are all Sum
    /// ([`Accumulator::set_children_sum`]).
    children: Result<f64, Error>,
    /// The [`CutCount`] values of the inner-item cells this cell covers, added up: what a custom
    /// Count subtotal shows under a Sum data field ([`Accumulator::cut_count`]).
    cut_values: u32,
    /// The same for the numbers: a custom `CountNums` subtotal.
    cut_numbers: u32,
}

/// The counts of one inner-item cell (a row leaf by a column leaf) that stop at its first error
/// in record order: `values` counts the error and nothing after it, `numbers` the numbers before
/// it. Under a Sum data field a custom `countA` or `count` subtotal adds these up over the
/// inner-item cells it covers (pivots.md "Which function the countA and count custom subtotals
/// compute": probes/t4622-pivot-count-subtotals CA, CN, B1, B4, B5 C6:C7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct CutCount {
    values: u32,
    numbers: u32,
    stopped: bool,
}

impl CutCount {
    /// Feeds one record's value, in record order; nothing counts after the first error.
    pub(crate) fn push(&mut self, item: &CacheItem) {
        if self.stopped {
            return;
        }
        match item {
            CacheItem::Missing => {}
            CacheItem::Number(_) | CacheItem::Date(_) => {
                self.values = self.values.saturating_add(1);
                self.numbers = self.numbers.saturating_add(1);
            }
            CacheItem::Text(_) | CacheItem::Boolean(_) => {
                self.values = self.values.saturating_add(1);
            }
            CacheItem::Error(_) => {
                self.values = self.values.saturating_add(1);
                self.stopped = true;
            }
        }
    }
}

impl Accumulator {
    /// An empty accumulator.
    pub(crate) fn new() -> Self {
        Accumulator {
            values: 0,
            numbers: 0,
            sum: 0.0,
            squares: 0.0,
            product: 1.0,
            max: f64::NEG_INFINITY,
            min: f64::INFINITY,
            error: None,
            children: Ok(0.0),
            cut_values: 0,
            cut_numbers: 0,
        }
    }

    /// Adds the counts of one inner-item cell this cell covers.
    pub(crate) fn add_cut(&mut self, part: CutCount) {
        self.cut_values = self.cut_values.saturating_add(part.values);
        self.cut_numbers = self.cut_numbers.saturating_add(part.numbers);
    }

    /// The cell's custom Count (`countA`) or `CountNums` (`count`) subtotal under a Sum data
    /// field: the inner-item cells' counts, each ending at its first error, added up; never an
    /// error (t4622 CA, CN, B1, B4, B5; with no error it is the plain count, B2). Any other
    /// `function` is its [`Accumulator::finish`] as a total.
    pub(crate) fn cut_count(&self, function: Aggregate, counts_mixed: bool) -> Result<f64, Error> {
        match function {
            Aggregate::Count => Ok(f64::from(self.cut_values)),
            Aggregate::CountNums => Ok(f64::from(self.cut_numbers)),
            _ => self.finish(function, true, counts_mixed, None),
        }
    }

    /// Feeds one record's value. A boolean is not a number (`TRUE` is not 1) and text is
    /// skipped by all but Count (t4135 `A_sum!I6:K6`); a date is its serial number (UNVERIFIED:
    /// no probe sums a date field).
    pub(crate) fn push(&mut self, item: &CacheItem) {
        match item {
            CacheItem::Missing => {}
            CacheItem::Number(n) | CacheItem::Date(n) => {
                let n = *n;
                self.values = self.values.saturating_add(1);
                self.numbers = self.numbers.saturating_add(1);
                self.sum += n;
                self.squares += n * n;
                self.product *= n;
                self.max = self.max.max(n);
                self.min = self.min.min(n);
            }
            CacheItem::Text(_) | CacheItem::Boolean(_) => {
                self.values = self.values.saturating_add(1);
            }
            CacheItem::Error(error) => {
                self.values = self.values.saturating_add(1);
                if self.error.is_none() {
                    self.error = Some(error.clone());
                }
            }
        }
    }

    /// Whether every record fed was blank: the cell holds no value at all.
    pub(crate) fn is_blank(&self) -> bool {
        self.values == 0
    }

    /// The order-dependent folds (sum, sum of squares, product) of this cell.
    pub(crate) fn folds(&self) -> Folds {
        Folds {
            sum: self.sum,
            squares: self.squares,
            product: self.product,
        }
    }

    /// Empties the order-dependent folds, so that [`Accumulator::add_folds`] can rebuild them from
    /// partial cells; the counts, the extremes and the first error do not depend on order and
    /// stay.
    pub(crate) fn clear_folds(&mut self) {
        self.sum = 0.0;
        self.squares = 0.0;
        self.product = 1.0;
    }

    /// Folds one partial cell's sums and product into this cell's: how a cell that is a total on
    /// both axes combines the cells of the leaf rows below it (pivots.md "Summation order of Sum,
    /// Average and Product totals": t4634 T2, T3, V1, V6, X6, X7; Varp, `StdDevp` and Product
    /// by analogy, UNVERIFIED, T-4678).
    pub(crate) fn add_folds(&mut self, part: Folds) {
        self.sum += part.sum;
        self.squares += part.squares;
        self.product *= part.product;
    }

    /// The record sum, or the first error: the value of a cell under Sum when its records are
    /// accumulated.
    pub(crate) fn record_sum(&self) -> Result<f64, Error> {
        match &self.error {
            Some(error) => Err(error.clone()),
            None => Ok(self.sum),
        }
    }

    /// The Sum this cell shows in a pivot whose data fields are all Sum: its record sum for an
    /// item cell, the sum of its children for a total.
    pub(crate) fn children_sum(&self) -> Result<f64, Error> {
        self.children.clone()
    }

    /// Sets the sum of the child cells' Sums, `cells` in display order: the first error among
    /// them, or their sum from 0.
    pub(crate) fn set_children_sum(&mut self, cells: impl IntoIterator<Item = Result<f64, Error>>) {
        self.children = sum_in_order(cells);
    }

    /// The cell's value under `function`.
    /// - `total`: the cell is a subtotal or a grand total on either axis.
    /// - `counts_mixed`: the pivot has a data field that is neither Count nor `CountNums`: Count
    ///   beside Sum or Average, beside all eleven (t4135 EC), and beside only Max, Min, Product,
    ///   Var, Varp or `StdDev` (t4634 C1 to C7).
    /// - `sum`: for a total, the Sum it shows when it is not the record sum: the sum of its
    ///   child cells in a pivot whose data fields are all Sum ([`super::result`]).
    pub(crate) fn finish(
        &self,
        function: Aggregate,
        total: bool,
        counts_mixed: bool,
        sum: Option<Result<f64, Error>>,
    ) -> Result<f64, Error> {
        // Count and CountNums never show an error in an item's cell; a total (a subtotal of an
        // outer field too) that covers an error is the error only when the pivot has another
        // aggregate beside them; Count beside CountNums is not mixed (t4616 I1, I2, M1 to M3;
        // t4634 C1 to C7; t4614 C1 to C4; t4589 E1 to E4; t4135 A_count R8, EC).
        if let Aggregate::Count | Aggregate::CountNums = function {
            if let (true, true, Some(error)) = (total, counts_mixed, &self.error) {
                return Err(error.clone());
            }
            let count = if function == Aggregate::Count {
                self.values
            } else {
                self.numbers
            };
            return Ok(f64::from(count));
        }
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        let n = f64::from(self.numbers);
        let record_sum = Ok(self.sum);
        // A cell whose records hold no number (only text, blanks or booleans) has not been
        // probed: these follow the worksheet functions (SUM, AVERAGE, MAX, MIN and PRODUCT of no
        // number; UNVERIFIED, T-4633).
        let value = match function {
            Aggregate::Sum => sum.unwrap_or(record_sum)?,
            Aggregate::Average if self.numbers == 0 => return Err(Error::DIV),
            Aggregate::Average => sum.unwrap_or(record_sum)? / n,
            Aggregate::Max | Aggregate::Min | Aggregate::Product if self.numbers == 0 => 0.0,
            Aggregate::Max => self.max,
            Aggregate::Min => self.min,
            Aggregate::Product => self.product,
            // Over one number StdDev and Var are #DIV/0! and StdDevp and Varp 0 (t4135
            // A_stdDev!B6:G6, A_stdDevp).
            Aggregate::StdDev | Aggregate::Var if self.numbers < 2 => return Err(Error::DIV),
            Aggregate::StdDevP | Aggregate::VarP if self.numbers < 1 => return Err(Error::DIV),
            // Var is the textbook one-pass formula over the accumulated sums, never the
            // worksheet functions' two-pass switch (`excel_variance`): t4616 V1 to V4 refute it,
            // and the n·Σx² form, for Var and StdDev. Varp is that n·Σx² form,
            // (n·Σx² − (Σx)²)/(n·n), not divided by n twice (t4634 V2 E7, E8, C11; V7 E8:E10;
            // X8 G9 refute that; on t4135 the two agree): of the candidates tried
            // (textbook /n, Σx²/n − mean², two-pass, Welford, (Σx² − Σx·mean)/n) it is the only
            // family that fits t4135 A_stdDevp!N6 and M8 (and t4589 S1 G5:H7). Its row and
            // column totals take Var's order (t4634 V2, V4, V5, V7, X8); its grand-grand order is
            // Var's by analogy (UNVERIFIED, T-4678).
            Aggregate::Var => self.sample_variance(),
            Aggregate::VarP => self.population_variance(),
            Aggregate::StdDev => self.sample_variance().sqrt(),
            Aggregate::StdDevP => self.population_variance().sqrt(),
            Aggregate::Count | Aggregate::CountNums => n,
        };
        if value.is_finite() {
            Ok(value)
        } else {
            // numbers.md "Overflow".
            Err(Error::NUM)
        }
    }

    /// `(Σx² − (Σx)²/n) / (n − 1)`.
    fn sample_variance(&self) -> f64 {
        let n = f64::from(self.numbers);
        (self.squares - self.sum * self.sum / n) / (n - 1.0)
    }

    /// `(n·Σx² − (Σx)²) / n²`.
    fn population_variance(&self) -> f64 {
        let n = f64::from(self.numbers);
        (n * self.squares - self.sum * self.sum) / (n * n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fed(items: &[CacheItem]) -> Accumulator {
        let mut acc = Accumulator::new();
        for item in items {
            acc.push(item);
        }
        acc
    }

    #[test]
    fn an_error_poisons_all_but_the_counts_and_a_mixed_total_count_shows_it() {
        let acc = fed(&[CacheItem::Number(4.0), CacheItem::Error(Error::NA)]);
        assert_eq!(
            acc.finish(Aggregate::Sum, false, true, None),
            Err(Error::NA)
        );
        assert_eq!(acc.finish(Aggregate::Count, false, true, None), Ok(2.0));
        assert_eq!(acc.finish(Aggregate::CountNums, false, true, None), Ok(1.0));
        assert_eq!(acc.finish(Aggregate::Count, true, false, None), Ok(2.0));
        assert_eq!(
            acc.finish(Aggregate::Count, true, true, None),
            Err(Error::NA)
        );
    }

    #[test]
    fn a_cut_count_ends_at_the_first_error_of_each_inner_cell() {
        // t4622 B1 East: A holds 10, `abc`; B a blank, `#N/A`, 5.
        let cells: [&[CacheItem]; 2] = [
            &[CacheItem::Number(10.0), CacheItem::Text("abc".to_string())],
            &[
                CacheItem::Missing,
                CacheItem::Error(Error::NA),
                CacheItem::Number(5.0),
            ],
        ];
        let mut total = Accumulator::new();
        for cell in cells {
            let mut cut = CutCount::default();
            for item in cell {
                cut.push(item);
                total.push(item);
            }
            total.add_cut(cut);
        }
        assert_eq!(total.cut_count(Aggregate::Count, true), Ok(3.0));
        assert_eq!(total.cut_count(Aggregate::CountNums, true), Ok(1.0));
        assert_eq!(total.cut_count(Aggregate::Sum, true), Err(Error::NA));
    }

    #[test]
    fn a_cell_with_no_number_follows_the_worksheet_functions() {
        let acc = fed(&[CacheItem::Text("abc".to_string()), CacheItem::Boolean(true)]);
        assert_eq!(acc.finish(Aggregate::Sum, false, false, None), Ok(0.0));
        assert_eq!(
            acc.finish(Aggregate::Average, false, false, None),
            Err(Error::DIV)
        );
        assert_eq!(acc.finish(Aggregate::Max, false, false, None), Ok(0.0));
        assert_eq!(acc.finish(Aggregate::Product, false, false, None), Ok(0.0));
        assert_eq!(
            acc.finish(Aggregate::VarP, false, false, None),
            Err(Error::DIV)
        );
        assert_eq!(acc.finish(Aggregate::Count, false, false, None), Ok(2.0));
    }

    #[test]
    fn the_children_sum_is_the_first_error_or_the_sum_in_order() {
        let mut acc = Accumulator::new();
        acc.set_children_sum([Ok(0.3), Ok(0.30000000000000004)]);
        assert_eq!(acc.children_sum(), Ok(0.6000000000000001));
        acc.set_children_sum([Ok(1.0), Err(Error::NA), Err(Error::DIV)]);
        assert_eq!(acc.children_sum(), Err(Error::NA));
    }

    #[test]
    fn an_overflowing_sum_is_num() {
        let acc = fed(&[CacheItem::Number(f64::MAX), CacheItem::Number(f64::MAX)]);
        assert_eq!(
            acc.finish(Aggregate::Sum, false, false, None),
            Err(Error::NUM)
        );
    }
}
