//! XIRR's and XNPV's cash flows and dates as read, in Excel's checking order (T-3312): split from
//! `financial/mod.rs`, which reads the arguments and runs the lifted kernels.

use crate::{
    calc_result::CalcResult,
    constants::{MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    /// XIRR's values and dates, read whole and checked pair by pair: a text or boolean value or
    /// date is `#VALUE!` and a negative date or one past the last serial `#NUM!`, whichever comes
    /// first (`{-2.5,39448,"x"}` and `{39448,-2.5,"x"}` dates are `#NUM!`, `{39448,"x",-2.5}`
    /// `#VALUE!`, probes/t3312-xirr-guess-dates-weibull A21-A24, A34; a negative first date
    /// before a third text value is `#NUM!`, A38, but one text value cell before 16 dates holding
    /// a negative one `#VALUE!`, gen-m9/7c20351562a2d71d A40; T-3312). Both come before the
    /// flows' signs (A39). An argument that is no range or array keeps the plain reading.
    pub(super) fn xirr_flows(
        &mut self,
        values: CalcResult,
        dates: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Flows, CalcResult> {
        let values = self.xirr_marked_numbers(values, cell)?;
        let error = |error: Error| CalcResult::new_error(error, cell, String::new());
        let dates = match self.xirr_marked_numbers(dates, cell) {
            Ok(dates) => dates,
            // A text value is read before an error in the dates' cells.
            Err(_) if values.iter().any(|value| value.is_nan()) => return Err(error(Error::VALUE)),
            Err(dates_error) => return Err(dates_error),
        };
        for index in 0..values.len().max(dates.len()) {
            if values.get(index).is_some_and(|value| value.is_nan()) {
                return Err(error(Error::VALUE));
            }
            match dates.get(index) {
                Some(date) if date.is_nan() => return Err(error(Error::VALUE)),
                // Truncated first, as `xirr_dates` reads them: 2958465.5 is a valid date.
                Some(&date)
                    if date.floor() < 0.0 || date.floor() > MAXIMUM_DATE_SERIAL_NUMBER as f64 =>
                {
                    return Err(error(Error::NUM))
                }
                _ => {}
            }
        }
        let dates = xirr_dates(&values, &dates);
        Ok((values, dates))
    }

    /// XIRR's numbers of a range or array with a text or boolean element as NaN, for
    /// [`Self::xirr_flows`]; any other argument is read plainly, refusing text with `#VALUE!`.
    fn xirr_marked_numbers(
        &mut self,
        arg: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        if matches!(arg, CalcResult::Range { .. } | CalcResult::Array(_)) {
            self.array_of_numbers_from(
                arg,
                &cell,
                false,
                true, // numeric text in a range cell reads as its number (xirr-back-dated-stop A31)
                || Ok(Some(0.0)),
                || Ok(Some(f64::NAN)),
            )
        } else {
            self.xirr_numbers(arg, &cell)
        }
    }
}

/// XNPV's and XIRR's dates truncated to whole days, with the first: `#NUM!` when there are none
/// or when the values and dates differ in count. The errors carry no message: the kernels run
/// lifted, where an element's message is dropped (SPEC-0011).
fn whole_day_dates(values: &[f64], dates: &[f64]) -> Result<(Vec<f64>, f64), Error> {
    let dates: Vec<f64> = dates.iter().map(|date| date.floor()).collect();
    match dates.first() {
        Some(&first_date) if values.len() == dates.len() => Ok((dates, first_date)),
        _ => Err(Error::NUM),
    }
}

/// XIRR's and XNPV's cash flows as read, with their dates truncated and checked (or the dates' error).
type Flows = (Vec<f64>, Result<Vec<f64>, Error>);

/// XNPV's values and dates as read, in Excel's order: a text or boolean first date (read as NaN)
/// is `#VALUE!` before an error or non-number value, which comes before a non-number later date
/// (`XNPV(0.1,{-100,#N/A,60},<TRUE,39448,39508>)` is `#VALUE!`, `<39448,TRUE,39508>` `#N/A`;
/// probes/t3312-xirr-xnpv-inner-calls A33-A37; T-3312).
pub(super) fn xnpv_flows(
    values: Result<Vec<f64>, CalcResult>,
    dates: Result<Vec<f64>, CalcResult>,
    cell: CellReferenceIndex,
) -> Result<Flows, CalcResult> {
    if let Ok(dates) = &dates {
        if dates.first().is_some_and(|date| date.is_nan()) {
            return Err(CalcResult::new_error(Error::VALUE, cell, String::new()));
        }
    }
    let values = values?;
    let dates = dates?;
    let dates = xnpv_dates(&values, &dates);
    Ok((values, dates))
}

/// XNPV's dates truncated to whole days, with the scalar call's checks in its order: the counts,
/// then the dates (the rate is checked before them, per element).
fn xnpv_dates(values: &[f64], dates: &[f64]) -> Result<Vec<f64>, Error> {
    let (dates, first_date) = whole_day_dates(values, dates)?;
    for date in &dates {
        // A date that is no valid serial, one before the first and a text or boolean date after
        // the first (NaN here) are `#NUM!`, not the `#VALUE!` Excel's documentation names:
        // `XNPV(0.1,{-100;30;40;50},{7;TRUE;8;9})` (probes/t3312-xirr-xnpv-inner-calls A10-A16,
        // ranges and arrays).
        if date.is_nan()
            || *date < MINIMUM_DATE_SERIAL_NUMBER as f64
            || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64
            || *date < first_date
        {
            return Err(Error::NUM);
        }
    }
    Ok(dates)
}

/// XIRR's dates truncated to whole days, with the scalar call's checks in its order: the counts,
/// then the dates (the solver's own checks follow, per element).
fn xirr_dates(values: &[f64], dates: &[f64]) -> Result<Vec<f64>, Error> {
    let (dates, first_date) = whole_day_dates(values, dates)?;
    // Excel reads an empty value or date cell as 0 and accepts date 0 and dates before the
    // first one, but is #NUM! for a negative date, a date past 9999-12-31, or when no date is
    // later than the first: {-1000,1100} on {0,45365} is 1.1^(365/45365) - 1,
    // {-1000,1100,-1000} on {45000,45365,44000} is -0.2467, and {-1000,1100} on
    // {45365,45000} is #NUM! although it has a root (financial.md, "XIRR reads empty cells as
    // 0 and accepts earlier dates"; probes/xirr-mismatched-empties, xirr-early-dates; T-1124).
    // A zero first value skips the later-date check: `{0,-1000,1100}` on 46000, 45000, 45365 is
    // 2.98e-9, and so are equal dates (probes/xirr-span-zero-first A28-A30, xirr-zero-first-bracket
    // A8-A13; gen-m9/e168be7f6d3f84c2 A125; T-2329).
    let first_value_is_zero = values.first() == Some(&0.0);
    if dates
        .iter()
        .any(|date| *date < 0.0 || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64)
        || (!first_value_is_zero && !dates.iter().any(|date| *date > first_date))
    {
        return Err(Error::NUM);
    }
    Ok(dates)
}
