//! XIRR's and XNPV's cash flows and dates as read, in Excel's checking order (T-3312): split from
//! `financial/mod.rs`, which reads the arguments and runs the lifted kernels.

use crate::{
    calc_result::{CalcResult, Range},
    constants::{MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
};

/// One element of XNPV's values or dates as [`Model::xnpv_walk_error`] reads it.
enum FlowElement {
    Number(f64),
    Error(Error),
    Empty,
    /// Text (numeric or not), a boolean or anything else that is no number.
    Other,
}

impl Model {
    /// XIRR's values and dates, read whole and checked pair by pair: a text or boolean value or
    /// date is `#VALUE!` and a negative date or one past the last serial `#NUM!`, whichever comes
    /// first (`{-2.5,39448,"x"}` and `{39448,-2.5,"x"}` dates are `#NUM!`, `{39448,"x",-2.5}`
    /// `#VALUE!`, probes/t3312-xirr-guess-dates-weibull A21-A24, A34; a negative first date
    /// before a third text value is `#NUM!`, A38, but one text value cell before 16 dates holding
    /// a negative one `#VALUE!`, gen-m9/7c20351562a2d71d A40; T-3312). Both come before the
    /// flows' signs (A39). Values and dates that are both ranges or arrays are walked first, with
    /// their error elements, by [`Self::xirr_walk_error`]. An argument that is no range or array
    /// keeps the plain reading.
    pub(super) fn xirr_flows(
        &mut self,
        values: CalcResult,
        dates: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Flows, CalcResult> {
        if let Some(error) = self.xirr_walk_error(&values, &dates, cell) {
            return Err(error);
        }
        let values = self.xirr_marked_numbers(values, cell)?;
        let error = |error: Error| CalcResult::new_error(error, cell, String::new());
        let dates = match self.xirr_marked_numbers(dates, cell) {
            Ok(dates) => dates,
            // A text value is read before an error in the dates' cells.
            Err(_) if values.iter().any(|value| value.is_nan()) => return Err(error(Error::VALUE)),
            Err(dates_error) => return Err(dates_error),
        };
        // The date before the value at each index, as `xirr_walk_error`.
        for index in 0..values.len().max(dates.len()) {
            if let Some(date_error) = dates.get(index).and_then(|&date| xirr_date_error(date)) {
                return Err(error(date_error));
            }
            if values.get(index).is_some_and(|value| value.is_nan()) {
                return Err(error(Error::VALUE));
            }
        }
        let dates = xirr_dates(&values, &dates);
        Ok((values, dates))
    }

    /// The error of XIRR's element walk over values and dates that are both ranges or arrays
    /// (T-3816, probes/t3816-xirr-element-reading), or `None`. Excel takes each index in turn,
    /// the date before the value, and stops at the first that refuses:
    ///
    /// - an error date or value is that error, so a later error loses to an earlier one in the
    ///   other argument (`XIRR(<-100,30,#N/A,50>,<39448,#DIV/0!,39600,39700>)` is `#DIV/0!`,
    ///   with the errors at 2 and 3 swapped `#N/A`; rows 35, 38), in a range, an array constant
    ///   and a computed array alike;
    /// - a date that is non-numeric text or a boolean is `#VALUE!` and one that is negative or
    ///   past the last serial, truncated, `#NUM!`, before a value at its index (`<-100,TRUE,..>`
    ///   against `<39448,-1,..>` is `#NUM!`, row 40; an error value later `#VALUE!` or `#NUM!`,
    ///   rows 36, 37);
    /// - a value that is non-numeric text or a boolean is `#VALUE!` (row 39, before a later
    ///   error date).
    ///
    /// Numeric and date text read as numbers, in a range and in an array alike (rows 13, 14, 20,
    /// 24, 25, 30); an empty element reads as 0 and passes. Only the stored elements can stop
    /// the walk, so it costs them, not the area (directive 5).
    fn xirr_walk_error(
        &mut self,
        values: &CalcResult,
        dates: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let (_, values) = self.flow_elements(values, cell, true)?;
        let (_, dates) = self.flow_elements(dates, cell, true)?;
        let mut indexes: Vec<usize> = values.iter().chain(&dates).map(|(i, _)| *i).collect();
        indexes.sort_unstable();
        indexes.dedup();
        let error = |error: Error| Some(CalcResult::new_error(error, cell, String::new()));
        for index in indexes {
            match element_at(&dates, index) {
                FlowElement::Error(date_error) => return error(date_error.clone()),
                FlowElement::Other => return error(Error::VALUE),
                FlowElement::Number(date) => {
                    if let Some(date_error) = xirr_date_error(*date) {
                        return error(date_error);
                    }
                }
                FlowElement::Empty => {}
            }
            match element_at(&values, index) {
                FlowElement::Error(value_error) => return error(value_error.clone()),
                FlowElement::Other => return error(Error::VALUE),
                FlowElement::Number(_) | FlowElement::Empty => {}
            }
        }
        None
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

/// Whether `node` hands XNPV one cell as a reference (a cell, a defined name, or `INDEX` of a
/// reference), where a literal or computed text is another argument kind.
fn returns_one_cell_reference(node: &Node) -> bool {
    match node {
        Node::ReferenceKind { .. } | Node::DefinedNameKind(_) => true,
        Node::FunctionKind {
            kind: Function::Index,
            args,
        } => args.first().is_some_and(|arg| {
            matches!(
                arg.ungrouped(),
                Node::ReferenceKind { .. } | Node::RangeKind { .. }
            )
        }),
        _ => false,
    }
}

impl Model {
    /// The error of XNPV's element walk (T-3815, probes/t3815-xnpv-element-reading), or `None`.
    /// Over values and dates of the same count, both ranges or arrays of more than one element,
    /// with a number as the first value, Excel takes each index in turn, the value before its
    /// date:
    ///
    /// - an error value or date is that error (`XNPV(0.1,<-100,30,#N/A,50>,<39448,#DIV/0!,..>)`
    ///   is `#DIV/0!`, with the errors swapped `#N/A`; rows 35, 38; `{-100,#N/A,60}` against
    ///   `<39448,TRUE,39508>` is `#N/A`, probes/t3312-xirr-xnpv-inner-calls A37);
    /// - a later value that is no number (text, numeric text too, a boolean, an empty cell) is
    ///   `#NUM!` (rows 24-28, 39);
    /// - a date that is text or a boolean is `#NUM!`, a first one that is text, a boolean or empty
    ///   `#VALUE!` (rows 13-17, 20, 21, 36); a first date below the first serial or past the last
    ///   one, truncated, and a later one that is negative or past the last serial as given
    ///   (2958465.5 is `#NUM!`, row 9) are `#NUM!` (rows 4-8, 37).
    ///
    /// A later empty date, a later 0 and one before the first are `#NUM!` at their index too, so
    /// before a later error value (probes/t4375-xnpv-refused-first-and-one-cell A29-A31). When
    /// the first value is no number the walk does not run and that `#VALUE!` stands: later
    /// values, errors and bad dates are left to it (t2603 C18-C20; t4375 rows 1-28).
    pub(super) fn xnpv_walk_error(
        &mut self,
        values: &CalcResult,
        dates: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let (count, values) = self.flow_elements(values, cell, false)?;
        let (date_count, dates) = self.flow_elements(dates, cell, false)?;
        if count < 2 || count != date_count {
            return None;
        }
        // Behind a first value that is no number the first value's `#VALUE!` (T-2603) beats every
        // later error and bad date (probes/t4375-xnpv-refused-first-and-one-cell rows 1-28).
        if !matches!(element_at(&values, 0), FlowElement::Number(_)) {
            return None;
        }
        // Only the stored elements and two more indexes can stop the walk: the first, and the
        // first empty value after it. Every other index is an
        // empty value and an empty later date, which pass. So the walk visits those, in order,
        // and costs the stored cells, not the area (directive 5).
        let mut indexes: Vec<usize> = values.iter().chain(&dates).map(|(i, _)| *i).collect();
        indexes.push(0);
        if let Some(gap) =
            (1..count).find(|&index| matches!(element_at(&values, index), FlowElement::Empty))
        {
            indexes.push(gap);
        }
        if let Some(gap) =
            (1..count).find(|&index| matches!(element_at(&dates, index), FlowElement::Empty))
        {
            indexes.push(gap);
        }
        indexes.sort_unstable();
        indexes.dedup();
        let error = |error: Error| Some(CalcResult::new_error(error, cell, String::new()));
        // A later date below the first (0 and an empty cell too) is `#NUM!` at its own index, so
        // before a later error value (probes/t4375-xnpv-refused-first-and-one-cell A29-A31).
        let first_day = match element_at(&dates, 0) {
            FlowElement::Number(date) => date.floor(),
            _ => f64::NEG_INFINITY,
        };
        for index in indexes {
            match element_at(&values, index) {
                FlowElement::Error(value_error) => return error(value_error.clone()),
                FlowElement::Empty | FlowElement::Other if index > 0 => return error(Error::NUM),
                _ => {}
            }
            match element_at(&dates, index) {
                FlowElement::Error(date_error) => return error(date_error.clone()),
                FlowElement::Other | FlowElement::Empty if index == 0 => {
                    return error(Error::VALUE)
                }
                FlowElement::Other | FlowElement::Empty => return error(Error::NUM),
                // The first date is checked truncated (as `xnpv_dates`). A later one only when it is
                // negative or past the last serial as given: a later 0, like one before the first, is
                // left to `xnpv_dates` (dates 100 and 0 keep the first value's `#VALUE!`,
                // probes/t2603-xnpv-no-number-values C1).
                FlowElement::Number(date)
                    if (index == 0
                        && (date.floor() < MINIMUM_DATE_SERIAL_NUMBER as f64
                            || date.floor() > MAXIMUM_DATE_SERIAL_NUMBER as f64))
                        || (index > 0
                            && (date.floor() < first_day
                                || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64)) =>
                {
                    return error(Error::NUM)
                }
                FlowElement::Number(_) => {}
            }
        }
        None
    }

    /// XNPV's values or dates argument with a one-cell range holding an error or numeric text
    /// read as that cell's value: the error is the argument's (`XNPV(0.1,<#N/A>,<39448>)` is
    /// `#N/A`, before an error or boolean date, probes/t3815-xnpv-element-reading D32-D38; an error
    /// date `#N/A`, D19, D22), and numeric or date text is its number (`XNPV(0.1,<"30">,<39448>)`
    /// is 30, `<"1/1/2025">` 45658, a date `<"39508">` too; D13, D14, D20, D24, D25, D30), where a
    /// range or array element of numeric text is `#NUM!`. Anything else is left as it is.
    ///
    /// A one-cell reference already evaluates to its value, so text is read this way only when
    /// `node` is a cell reference, a defined name or an `INDEX` of a reference (rows 38, 39, 42,
    /// 43 of probes/t4375-xnpv-refused-first-and-one-cell; `OFFSET` and a one-cell range keep a
    /// range): a literal text argument (`XNPV(0.1,"30",39448)`) is unprobed
    /// and keeps its earlier reading.
    pub(super) fn xnpv_one_cell(&mut self, arg: CalcResult, node: &Node) -> CalcResult {
        let value = match &arg {
            CalcResult::Range { left, right } if left == right => self.evaluate_cell(*left),
            CalcResult::String(_) if returns_one_cell_reference(node.ungrouped()) => arg.clone(),
            _ => return arg,
        };
        match value {
            error @ CalcResult::Error { .. } => error,
            CalcResult::String(text) => match self.cast_number(&text) {
                Some(number) => CalcResult::Number(number),
                None => arg,
            },
            _ => arg,
        }
    }

    /// The count of elements of a range (trimmed to the used area, as [`Self::flow_range_end`])
    /// or an array, and the elements that are not empty with their row-major index, sorted;
    /// `None` for anything else. With `text_as_number` (XIRR's reading) numeric and date text is
    /// its number; otherwise (XNPV's) it is [`FlowElement::Other`]. A range is read through its
    /// stored cells only, so a sparse range costs its stored cells, not its area (directive 5).
    fn flow_elements(
        &mut self,
        arg: &CalcResult,
        cell: CellReferenceIndex,
        text_as_number: bool,
    ) -> Option<(usize, Vec<(usize, FlowElement)>)> {
        match arg {
            CalcResult::Range { left, right } if left.sheet == right.sheet => {
                let (row2, column2) = self.flow_range_end(left, right)?;
                let rows = usize::try_from(row2 - left.row + 1).ok()?;
                let width = usize::try_from(column2 - left.column + 1).ok()?;
                let range = Range {
                    left: *left,
                    right: CellReferenceIndex {
                        sheet: left.sheet,
                        row: row2,
                        column: column2,
                    },
                };
                let mut elements = Vec::new();
                self.for_each_existing_cell(&range, cell, &mut |model, position| {
                    let element = match model.evaluate_cell(position) {
                        CalcResult::Number(value) => FlowElement::Number(value),
                        CalcResult::Error { error, .. } => FlowElement::Error(error),
                        CalcResult::EmptyCell => return Ok(()),
                        CalcResult::String(text) if text_as_number => model
                            .cast_number(&text)
                            .map_or(FlowElement::Other, FlowElement::Number),
                        _ => FlowElement::Other,
                    };
                    // In the range by the walk's contract, so both offsets are non-negative.
                    let row = usize::try_from(position.row - left.row).unwrap_or(0);
                    let column = usize::try_from(position.column - left.column).unwrap_or(0);
                    elements.push((row * width + column, element));
                    Ok(())
                })
                .ok()?;
                elements.sort_unstable_by_key(|(index, _)| *index);
                Some((rows.saturating_mul(width), elements))
            }
            CalcResult::Array(array) => {
                let width = array.first().map_or(0, Vec::len);
                let elements = array
                    .iter()
                    .flatten()
                    .enumerate()
                    .filter_map(|(index, node)| {
                        let element = match node {
                            ArrayNode::Number(value) => FlowElement::Number(*value),
                            ArrayNode::Error(error) => FlowElement::Error(error.clone()),
                            ArrayNode::Empty => return None,
                            ArrayNode::String(text) if text_as_number => self
                                .cast_number(text)
                                .map_or(FlowElement::Other, FlowElement::Number),
                            _ => FlowElement::Other,
                        };
                        Some((index, element))
                    })
                    .collect();
                Some((array.len().saturating_mul(width), elements))
            }
            _ => None,
        }
    }
}

/// The element at `index` of [`Model::flow_elements`]'s sorted non-empty elements: empty when
/// it is not there.
fn element_at(elements: &[(usize, FlowElement)], index: usize) -> &FlowElement {
    elements
        .binary_search_by_key(&index, |(at, _)| *at)
        .ok()
        .and_then(|found| elements.get(found))
        .map_or(&FlowElement::Empty, |(_, element)| element)
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
    // The last serial bounds a later date as given, before it is truncated: 2958465.5 is `#NUM!`
    // there (probes/t3815-xnpv-element-reading A9-C9, T-3815), but a first date of 2958465.5 is
    // read truncated (`XNPV(0.1,30,2958465.5)` is 30, D9). XIRR truncates every date first.
    if dates
        .iter()
        .skip(1)
        .any(|date| *date > MAXIMUM_DATE_SERIAL_NUMBER as f64)
    {
        return Err(Error::NUM);
    }
    let (dates, first_date) = whole_day_dates(values, dates)?;
    for date in &dates {
        // A date that is no valid serial, one before the first and a text or boolean date after
        // the first (NaN here) are `#NUM!`: `XNPV(0.1,{-100;30;40;50},{7;TRUE;8;9})`
        // (probes/t3312-xirr-xnpv-inner-calls A10-A16, ranges and arrays). Numeric text, date
        // text, FALSE and an empty cell as a later date are `#NUM!` too, in a range, an array
        // constant and a computed array alike (probes/t3815-xnpv-element-reading rows 13-18).
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

/// The `#NUM!` of an XIRR date that is negative or past the last serial once truncated
/// (2958465.5 is a valid date there, as `xirr_dates` reads it), or `#VALUE!` for a text or
/// boolean date read as NaN; `None` for a valid date.
fn xirr_date_error(date: f64) -> Option<Error> {
    if date.is_nan() {
        Some(Error::VALUE)
    } else if date.floor() < 0.0 || date.floor() > MAXIMUM_DATE_SERIAL_NUMBER as f64 {
        Some(Error::NUM)
    } else {
        None
    }
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
