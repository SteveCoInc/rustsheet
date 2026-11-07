use crate::{
    calc_result::{CalcResult, Range},
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

mod accrint;
mod amortization;
mod annuity;
mod cumulative;
mod dates;
pub(crate) use amortization::{amortization_values_valid, asset_life_refused};
pub(crate) use dates::{day_count_basis, days_30_360_us_ymd};
mod discount;
mod financial_bonds;
mod financial_depreciation;
pub(crate) use financial_depreciation::MAX_DEPRECIATION_PERIODS;
mod financial_util;
mod flows;
mod growth;
mod mirr;
mod rate_conversions;

use financial_util::{compute_irr, compute_xirr, compute_xnpv, excel_npv};
use flows::xnpv_flows;

use crate::functions::lift::{num_error, number_or_error, number_result, LiftCost};
use crate::functions::Function;

/// The cash flows (or FVSCHEDULE's rates) a lifted NPV, IRR, MIRR, XNPV, XIRR or FVSCHEDULE
/// element walks, at least 1: what the call is charged per element (`LiftCost::Cells`).
fn flow_count(flows: Result<&[f64], &CalcResult>) -> u64 {
    let count = flows.map_or(1, |flows| flows.len().max(1));
    u64::try_from(count).unwrap_or(u64::MAX)
}

/// A computation's error as the cell's error, with no message: the kernels run lifted, where an
/// element's message is dropped (`calc_result_to_array_node`), so building one only allocates
/// (SPEC-0011, T-1823).
fn formula_error(cell: CellReferenceIndex) -> impl Fn(Error) -> CalcResult {
    move |error| CalcResult::new_error(error, cell, String::new())
}

/// Which XNPV argument [`Model::xnpv_numbers`] reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum XnpvArgument {
    Values,
    Dates,
}

// See:
// https://github.com/apache/openoffice/blob/c014b5f2b55cff8d4b0c952d5c16d62ecde09ca1/main/scaddins/source/analysis/financial.cxx

impl Model {
    fn get_array_of_numbers_generic(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        accept_number_node: bool,
        handle_empty_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
        handle_non_number_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
    ) -> Result<Vec<f64>, CalcResult> {
        let result = self.evaluate_node_in_context(arg, *cell);
        self.array_of_numbers_from(
            result,
            cell,
            accept_number_node,
            false,
            handle_empty_cell,
            handle_non_number_cell,
        )
    }

    fn get_array_of_numbers(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            true,        // accept_number_node
            || Ok(None), // Ignore empty cells
            || Ok(None), // Ignore non-number cells
        )
    }

    /// XNPV's values or dates. A non-number cell refuses the values with `#NUM!`. A text or
    /// boolean date in a range or array is read as NaN, which [`xnpv_flows`] turns into
    /// `#VALUE!` in first place and `#NUM!` later; one that is no range or array is `#VALUE!`. A
    /// values argument that is one empty cell is a single cash flow of 0, `XNPV(1,A2,B11)` is 0
    /// (probes/xnpv-empty-values-cell C1-C4, T-2599); empty cells in a larger range stay refused.
    fn xnpv_numbers(
        &mut self,
        result: CalcResult,
        cell: &CellReferenceIndex,
        kind: XnpvArgument,
    ) -> Result<Vec<f64>, CalcResult> {
        let refuse = |error: Error| {
            move || {
                Err(CalcResult::new_error(
                    error.clone(),
                    *cell,
                    "Expected number".to_string(),
                ))
            }
        };
        if self.empty_cell(&result) {
            // One empty cell, `A2` or `A2:A2`, is a 0 cash flow but no date: `XNPV(1,A2:A2,B11)`
            // is 0 and `XNPV(1,B11,A2)` is `#NUM!` (probes/xnpv-empty-cell-forms C1-C5).
            return match kind {
                XnpvArgument::Values => Ok(vec![0.0]),
                XnpvArgument::Dates => Err(CalcResult::new_error(
                    Error::NUM,
                    *cell,
                    "Expected number".to_string(),
                )),
            };
        }
        // `None` marks a text or boolean element as NaN instead of refusing it.
        let non_number = match kind {
            XnpvArgument::Values => Some(Error::NUM),
            XnpvArgument::Dates
                if matches!(result, CalcResult::Range { .. } | CalcResult::Array(_)) =>
            {
                None
            }
            XnpvArgument::Dates => Some(Error::VALUE),
        };
        self.array_of_numbers_from(
            result,
            cell,
            true, // accept_number_node
            false,
            refuse(Error::NUM),
            move || match &non_number {
                Some(error) => refuse(error.clone())(),
                None => Ok(Some(f64::NAN)),
            },
        )
    }

    fn xirr_numbers(
        &mut self,
        arg: CalcResult,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.array_of_numbers_from(
            arg,
            cell,
            false,            // Do not accept a single number node
            true,             // numeric text reads as its number (probes/xirr-back-dated-stop A31)
            || Ok(Some(0.0)), // Treat empty cells as zero
            || {
                Err(CalcResult::new_error(
                    Error::VALUE,
                    *cell,
                    "Expected number".to_string(),
                ))
            },
        )
    }

    /// XIRR's size checks (T-2428): one value and one date is `#N/A`
    /// (`XIRR(Data!A9:A9,Data!E9:E9)`, gen-m9/30e20ba3302fea8b A110; also with a FALSE date,
    /// bc04c1f478c9896c A24), a single date that is text or a boolean `#VALUE!`
    /// (`XIRR({-100,#N/A,60},<FALSE>)`, 1cd3096e3e87c965 A48), then values and dates of
    /// different counts `#NUM!`, whatever the cells hold (`XIRR({-100,#N/A,60},<5 dates>)`,
    /// b72d952c5f52cb4d A1). An argument that is no range, array or scalar is left to the
    /// number reading.
    fn xirr_size_error(
        &mut self,
        values: &CalcResult,
        dates: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let value_count = self.flow_element_count(values)?;
        let date_count = self.flow_element_count(dates)?;
        let error = if value_count == 1 && date_count == 1 {
            // A non-numeric text or boolean cell keeps the old `#VALUE!` (gen-m9/097af1ca33638670
            // A255, a91bad4e84f49739 A203, bc04c1f478c9896c A318; `XIRR(5,"x")`,
            // probes/t3312-xirr-xnpv-inner-calls A45), but an empty cell is a number here:
            // `XIRR(<empty>,39448)` and `XIRR(5,<empty>)` are `#N/A` (A20-A25, A40, A43, A44;
            // T-3312), and so is a numeric or date text cell: `XIRR(<-100>,<"39508">)` and
            // `XIRR(<"1/1/2025">,<39448>)` are `#N/A` (probes/t3816-xirr-element-reading D13,
            // D14, D20, D24, D25, D30; T-3816).
            if self.non_numeric_text_or_boolean_cell(values)
                || self.non_numeric_text_or_boolean_cell(dates)
            {
                return None;
            }
            Error::NA
        } else if date_count == 1 && self.non_numeric_text_or_boolean_cell(dates) {
            // gen-m9/1cd3096e3e87c965 A48. An empty or numeric-text date cell is left to the
            // count (`XIRR(Data!B6:C11,<empty>,0.5)` and `XIRR(Data!A6:B12,<"1">)` are `#NUM!`,
            // 89f0aee953a55a45 A330, c0681a8b150ce301 A266; T-2611).
            Error::VALUE
        } else if value_count == 1 && self.non_numeric_text_or_boolean_cell(values) {
            // One text or boolean value cell against several dates is read first, `#VALUE!`
            // (gen-m9/6376e27be7a05871 A67), but one empty value cell is the count's `#NUM!`
            // (`XIRR(Data!D10:D10,Data!B7:E8)`, 1dce4774f5c0e736 A91; T-2611); a larger values
            // range of another count is `#NUM!` whatever it holds (0a957119475760c5 A47,
            // bbdfc06cae0e0d71 A44).
            return None;
        } else if value_count != date_count {
            Error::NUM
        } else {
            return None;
        };
        Some(CalcResult::new_error(error, cell, String::new()))
    }

    /// XNPV's checks before it reads a number (T-2428), after the rate's, in Excel's order:
    ///
    /// 1. One values cell that is non-numeric text or a boolean is `#VALUE!`, before an error
    ///    dates argument (`XNPV(-1,<FALSE>,<#N/A>)`, gen-m9/3aa168259335be54 A334). Numeric text
    ///    is not refused: `XNPV(0.1,<"1.5">,Data!C5:C6)` is the count's `#NUM!` (81bcdc0bec0c8e6b
    ///    A37, through DB).
    /// 2. An error dates argument is that error, before the values' cells
    ///    (`XNPV(0.05,Data!B1:E2,<#N/A>)`, 5c6a23c4f6597f1c A111).
    /// 3. One dates cell that is non-numeric text or a boolean is `#VALUE!`
    ///    (d0854366830dcdee A274).
    /// 4. Values and dates of different counts are `#NUM!` whatever they hold
    ///    (`XNPV(0.1,{-100,#N/A,60},<5 dates>)`, bc04c1f478c9896c A313;
    ///    `XNPV(0.2,0.3,Data!C9:D12)`, 1cd3096e3e87c965 A217).
    ///
    /// An empty cell is left to the reading: one empty values cell is a 0 cash flow (T-2599).
    fn xnpv_size_error(
        &mut self,
        values: &CalcResult,
        dates: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        if self.non_numeric_text_or_boolean_cell(values) {
            return Some(CalcResult::new_error(Error::VALUE, cell, String::new()));
        }
        // An error values argument is read before any dates check: `XNPV(0.1,<#N/A>,<TRUE>)` is
        // `#N/A` (probes/t3815-xnpv-element-reading D36), and it comes before the rate too.
        if matches!(values, CalcResult::Error { .. }) {
            return None;
        }
        if matches!(dates, CalcResult::Error { .. }) {
            return Some(dates.clone());
        }
        if self.non_numeric_text_or_boolean_cell(dates) {
            return Some(CalcResult::new_error(Error::VALUE, cell, String::new()));
        }
        let value_count = self.flow_element_count(values)?;
        let date_count = self.flow_element_count(dates)?;
        (value_count != date_count).then(|| CalcResult::new_error(Error::NUM, cell, String::new()))
    }

    /// The number of elements XIRR and XNPV read from an evaluated argument: a range's cells (a
    /// whole column or row trimmed to the used area, as [`Self::array_of_numbers_from`]), an
    /// array's elements, 1 for a scalar; `None` for an error or anything else.
    fn flow_element_count(&self, arg: &CalcResult) -> Option<usize> {
        match arg {
            CalcResult::Range { left, right } if left.sheet == right.sheet => {
                let (row2, column2) = self.flow_range_end(left, right)?;
                let rows = usize::try_from(row2 - left.row + 1).ok()?;
                let columns = usize::try_from(column2 - left.column + 1).ok()?;
                Some(rows.saturating_mul(columns))
            }
            CalcResult::Array(array) => Some(
                array
                    .len()
                    .saturating_mul(array.first().map_or(0, Vec::len)),
            ),
            CalcResult::Number(_)
            | CalcResult::String(_)
            | CalcResult::Boolean(_)
            | CalcResult::EmptyCell => Some(1),
            _ => None,
        }
    }

    /// A range's last row and column with a whole column or row trimmed to the used area, as
    /// [`Self::array_of_numbers_from`] and the XIRR and XNPV size checks read it; `None` for a
    /// missing sheet.
    fn flow_range_end(
        &self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
    ) -> Option<(i32, i32)> {
        let (used_row, used_column) = self.workbook.worksheet(left.sheet).ok()?.used_area_end();
        let row2 = if left.row == 1 && right.row == LAST_ROW {
            used_row
        } else {
            right.row
        };
        let column2 = if left.column == 1 && right.column == LAST_COLUMN {
            used_column
        } else {
            right.column
        };
        Some((row2, column2))
    }

    /// Whether a one-cell range or scalar is text or a boolean.
    ///
    /// An array element is not a cell: `XIRR(D2,BYROW(D9,LAMBDA(x,ISBLANK(x))))` is `#N/A`
    /// (gen-m9/bc04c1f478c9896c A24).
    fn text_or_boolean_cell(&mut self, arg: &CalcResult) -> bool {
        matches!(
            *self.single_cell_value(arg),
            CalcResult::String(_) | CalcResult::Boolean(_)
        )
    }

    /// Whether XNPV is `#VALUE!` for the first cell of a multi-cell values range (T-2603,
    /// probes/t2603-xnpv-no-number-values). That cell is empty, a boolean or text, numeric text
    /// too (probes/t3815-xnpv-element-reading A30, T-3815).
    /// Only the first cell counts: `XNPV(1,A4:A5,..)` with A4 empty and A5 = 5 is `#VALUE!`
    /// (C3, also with later text or booleans, C18-C20), but `XNPV(1,A5:A6,..)` with A6 empty is
    /// `#NUM!` (C4).
    ///
    /// The dates come first when their first cell is a number and a later date cell is a boolean
    /// or non-numeric text: those calls stay `#NUM!` (gen-m9/3aa168259335be54 A236, text;
    /// 76b07b57099ccc8d A263, FALSE; b6a02893a8d6b151 A114, 'abc'). An empty date or one before
    /// the first does not block (100, 0 and an empty cell, C7), nor do dates whose first cell is
    /// no number (C5, C14). A date past the last serial or a negative one only came with a
    /// boolean or text, so both are unprobed and do not block. Numeric text, arrays and dates
    /// that are no range are unprobed and keep the older reading.
    fn xnpv_first_value_refused(
        &mut self,
        values: &CalcResult,
        dates: &CalcResult,
        cell: CellReferenceIndex,
    ) -> bool {
        if let CalcResult::Array(array) = values {
            // An array's first element that is text or a boolean is refused too:
            // `XNPV(0.1,{"-100";30;40;50},..)` and `{TRUE;..}` are `#VALUE!`
            // (probes/t3815-xnpv-element-reading B30, B31). The element walk has already
            // settled any date that would come first.
            let elements = array
                .len()
                .saturating_mul(array.first().map_or(0, Vec::len));
            return elements > 1
                && matches!(
                    array.first().and_then(|row| row.first()),
                    Some(ArrayNode::String(_) | ArrayNode::Boolean(_))
                );
        }
        let CalcResult::Range { left, right } = values else {
            return false;
        };
        if left == right || left.sheet != right.sheet {
            return false;
        }
        let refused = |model: &mut Self, position: CellReferenceIndex| {
            let value = CalcResult::Range {
                left: position,
                right: position,
            };
            // Numeric text is refused too: `XNPV(0.1,<"-100",30,40,50>,..)` is `#VALUE!`
            // (probes/t3815-xnpv-element-reading A30).
            model.empty_cell(&value) || model.text_or_boolean_cell(&value)
        };
        if !refused(self, *left) {
            return false;
        }
        let CalcResult::Range {
            left: date_left,
            right: date_right,
        } = dates
        else {
            return false;
        };
        if date_left.sheet != date_right.sheet {
            return false;
        }
        if refused(self, *date_left) {
            return true;
        }
        // Values and dates of different counts are the size check's `#NUM!` (C2, C12), so the
        // dates are not scanned.
        if self.flow_element_count(values) != self.flow_element_count(dates) {
            return false;
        }
        let Some((row2, column2)) = self.flow_range_end(date_left, date_right) else {
            return false;
        };
        let range = Range {
            left: *date_left,
            right: CellReferenceIndex {
                sheet: date_left.sheet,
                row: row2,
                column: column2,
            },
        };
        // Only the stored cells are read: an empty date blocks nothing, so two far-apart cells
        // cost two reads, not the area between them (directive 5).
        let mut blocked = false;
        let walked = self.for_each_existing_cell(&range, cell, &mut |model, position| {
            let value = CalcResult::Range {
                left: position,
                right: position,
            };
            if model.non_numeric_text_or_boolean_cell(&value) {
                blocked = true;
                // Stops the walk: the first blocking date settles it.
                return Err(CalcResult::EmptyCell);
            }
            Ok(())
        });
        !blocked && walked.is_ok()
    }

    /// Whether a one-cell range or scalar is empty.
    fn empty_cell(&mut self, arg: &CalcResult) -> bool {
        matches!(*self.single_cell_value(arg), CalcResult::EmptyCell)
    }

    /// Whether a one-cell range or scalar is a boolean or text that does not read as a number.
    fn non_numeric_text_or_boolean_cell(&mut self, arg: &CalcResult) -> bool {
        match &*self.single_cell_value(arg) {
            CalcResult::String(text) => self.cast_number(text).is_none(),
            CalcResult::Boolean(_) => true,
            _ => false,
        }
    }

    /// `NPV(rate, value1, [value2], ...)`: `Sum[value[i]/(1+rate)^i, {i, 1, n}]`. The rate lifts
    /// over arrays and ranges; the values are read once, whole, and handed to every element, whose
    /// rate is checked first: a rate error wins over an error in the values, the scalar order kept
    /// from before T-1821. The values are evaluated before the rate, also when the rate is an
    /// error, since the call's charge needs their count; Excel's order is unprobed (financial.md
    /// "NPV, IRR and MIRR error order", T-2103).
    pub(crate) fn fn_npv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let [_, flows @ ..] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        if flows.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let flows = self
            .npv_cash_flows(flows, cell)
            .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Npv,
            args,
            LiftCost::Cells(flow_count(flows.as_deref())),
            cell,
            &mut |model, values| {
                let rate = match values.first() {
                    Some(rate) => model.cast_number_ref(rate, cell),
                    None => Err(CalcResult::new_args_number_error(cell)),
                };
                match (rate, &flows) {
                    (Err(error), _) => error,
                    (Ok(_), Err(error)) => error.clone(),
                    (Ok(rate), Ok(flows)) => number_result(excel_npv(rate, flows), cell),
                }
            },
        )
    }

    /// The cash flows of NPV, read from its value arguments in order.
    fn npv_cash_flows(
        &mut self,
        flows: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        // Excel: an error inside a range or an array is the result only when no direct argument
        // errs, whatever their order: `NPV(0.1,{1,#DIV/0!},"x")` is #VALUE!, `NPV(0.1,Q1:Q2,NA())`
        // over a #DIV/0! cell #N/A, and the first range or array error wins among them
        // (probes/npv-power-edges A36-A51, npv-power-edges-2 A31-A40). docs/excel-semantics/
        // financial.md "NPV's error order".
        let mut deferred = None;
        for arg in flows {
            // Excel: an argument read as a reference skips the text and booleans of its cells,
            // however the reference is written: `B1`, `(B1)`, `@B1:B1`, `IF(TRUE,B1)`,
            // `INDEX(B1:B2,1)`, `CHOOSE(1,B1)`, `OFFSET(B1,0,0)`, `INDIRECT("B1")`, `LET`, a
            // name for a cell, `Sheet1:Sheet1!B1`; a union is its areas as written (ironcalc-calc
            // FINANCIAL/NPV.xlsx K5, L5; probes/npv-fvschedule-typed A7, A10, A11;
            // probes/npv-reference-forms). Any other value is typed: it is coerced
            // (`NPV(0.1,-10000,"2750",TRUE,4250)` is -3914.6233180793665) and text that is no
            // number is #VALUE! (probes/fin-argument-modes H368-H371; probes/npv-fvschedule-typed
            // A5, A6, A8, A9; probes/npv-fvschedule-edges A1, A5).
            // docs/excel-semantics/financial.md "NPV and FVSCHEDULE arguments".
            match self
                .evaluate_reference_arg(arg, cell)
                .into_ranges_or_value()
            {
                Ok(areas) => {
                    if deferred.is_none() {
                        if let Err(error) =
                            self.for_each_number_in_areas(&areas, cell, |v| values.push(v))
                        {
                            deferred = Some(error);
                        }
                    }
                }
                Err(CalcResult::Number(value)) => values.push(value),
                Err(direct @ (CalcResult::String(_) | CalcResult::Boolean(_))) => {
                    values.push(self.cast_number_ref(&direct, cell)?);
                }
                // A written empty argument is a cash flow of 0 (probes/npv-fvschedule-edges A2).
                Err(CalcResult::EmptyArg) => values.push(0.0),
                // An array is read whole: its numbers are the cash flows, its text and booleans
                // are skipped and its first error is deferred like a range's (H368;
                // probes/npv-reference-forms A7, A8, A13).
                Err(array @ CalcResult::Array(_)) => {
                    if deferred.is_none() {
                        match self.array_of_numbers_from(
                            array,
                            &cell,
                            true,
                            false,
                            || Ok(None),
                            || Ok(None),
                        ) {
                            Ok(numbers) => values.extend(numbers),
                            Err(error) => deferred = Some(error),
                        }
                    }
                }
                Err(error @ CalcResult::Error { .. }) => return Err(error),
                Err(_) => {}
            }
        }
        deferred.map_or(Ok(values), Err)
    }

    /// `IRR(values, [guess])`: the internal rate of return of cash flows at regular intervals.
    /// The guess lifts over arrays and ranges; the values are read once, whole, and an error in
    /// them is every element's (T-1821). Each element may run the solver's every pass over the
    /// flows, so the call is charged their count per element, weighted by the table's
    /// `IRR_WEIGHT`.
    pub(crate) fn fn_irr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (Some(values), 1..=2) = (args.first(), args.len()) else {
            return CalcResult::new_args_number_error(cell);
        };
        // Excel: an error inside an array or a referenced range is #VALUE!, whatever its kind,
        // so the original error is dropped. An error argument that is not a single cell
        // passes through (docs/excel-semantics/financial.md, "IRR over an error element";
        // probes/fin-irr-error-array). That error comes before the guess's; the #VALUE! of an
        // error element, or of a number straight from `CELL("row"|"col",..)` or ERROR.TYPE
        // (where any other scalar flow is `#NUM!`), comes after it: `IRR(1/0,NA())` is
        // `#DIV/0!`, and so are `IRR({1,"a"}+0,1/0)` and `IRR(CELL("row",D2),1/0)`
        // (financial.md "IRR's error order", probes/t2728-irr-guess-ref-mark, T-2728).
        // The mark follows LET, IF, CHOOSE, IFERROR, a LAMBDA call and INDEX
        // (`IRR(IF(TRUE,CELL("row",D2)))` is `#VALUE!`, probes/t2728-irr-guess-ref-mark A53:A61, T-2766).
        let (evaluated, marked) = self.evaluate_marked(values, cell);
        let flows_error_first = matches!(evaluated, CalcResult::Error { .. })
            && !matches!(*values.ungrouped_at(cell.row), Node::ReferenceKind { .. });
        let flows = if flows_error_first {
            Err(evaluated)
        } else if marked && matches!(evaluated, CalcResult::Number(_)) {
            Err(CalcResult::new_error(Error::VALUE, cell, String::new()))
        } else {
            self.array_of_numbers_from(evaluated, &cell, true, false, || Ok(None), || Ok(None))
                .map_err(|_| CalcResult::new_error(Error::VALUE, cell, String::new()))
        }
        .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Irr,
            args,
            LiftCost::Cells(flow_count(flows.as_deref())),
            cell,
            &mut |model, values| {
                if let (true, Err(error)) = (flows_error_first, &flows) {
                    return error.clone();
                }
                let guess = match model.cast_number_or(values.get(1), 0.1, cell) {
                    Ok(guess) => guess,
                    Err(error) => return error,
                };
                let flows = match &flows {
                    Ok(flows) => flows,
                    Err(error) => return error.clone(),
                };
                number_result(compute_irr(flows, guess).map_err(|error| error.0), cell)
            },
        )
    }

    /// `XNPV(rate, values, dates)`: the rate lifts over arrays (a multi-cell range is refused);
    /// the values and dates are read once, whole, and handed to every element, whose rate is
    /// checked first, the scalar order (T-1843).
    pub(crate) fn fn_xnpv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // All three are required (docs/excel-semantics/syntax.md, "Minimum argument counts").
        let [_, values_node, dates_node] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        let values = self.evaluate_node_in_context(values_node, cell);
        let values = self.xnpv_one_cell(values, values_node);
        let dates = self.evaluate_node_in_context(dates_node, cell);
        let dates = self.xnpv_one_cell(dates, dates_node);
        let size_error = self.xnpv_size_error(&values, &dates, cell);
        let walk_error = self.xnpv_walk_error(&values, &dates, cell);
        let first_value_refused = self.xnpv_first_value_refused(&values, &dates, cell);
        // The dates are truncated and checked once, not per element: each element is charged
        // only the values' count.
        // An error values argument, unlike an error among its elements, comes before a rate at or
        // below 0 and a non-number first date (`XNPV(-1+2^-52,<#VALUE!>,<4 dates>)` is `#VALUE!`,
        // gen-m9/7d2979fba2c95d64 A84; c0681a8b150ce301 A112; T-3312).
        let values_error = matches!(values, CalcResult::Error { .. });
        let values = self.xnpv_numbers(values, &cell, XnpvArgument::Values);
        let dates = self.xnpv_numbers(dates, &cell, XnpvArgument::Dates);
        let flows = if values_error {
            values.map(|_| (Vec::new(), Ok(Vec::new())))
        } else {
            xnpv_flows(values, dates, cell)
        }
        .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Xnpv,
            args,
            LiftCost::Cells(flow_count(
                flows.as_ref().map(|(values, _)| values.as_slice()),
            )),
            cell,
            &mut |model, values| {
                // A boolean rate, a cell or an array element, is `#VALUE!` (`XNPV(<TRUE>,..)`,
                // gen-m9/1dce4774f5c0e736 A22; `XNPV(SORT(<FALSE>),..)`, 5c6a23c4f6597f1c A258).
                let rate = match values.first() {
                    Some(rate) => model.cast_number_no_bools_ref(rate, cell),
                    None => Err(CalcResult::new_args_number_error(cell)),
                };
                if let (Ok(_), Some(size_error)) = (&rate, &size_error) {
                    return size_error.clone();
                }
                // A values range whose first cell is no number is `#VALUE!`, after the counts and
                // a rate at or below 0 (`XNPV(1,A2:A3,B11)` and `XNPV(-1,A2:A3,B11:B12)` with A2:A3
                // empty are `#NUM!`; probes/t2603-xnpv-no-number-values C2, C13). The dates are
                // not read first: all-empty dates are `#VALUE!` too (C5).
                // The element walk's error comes after a rate at or below 0 and before the first
                // value's `#VALUE!` (T-3815).
                if let (Ok(rate), Some(walk_error)) = (&rate, &walk_error) {
                    return if *rate <= 0.0 {
                        CalcResult::new_error(Error::NUM, cell, String::new())
                    } else {
                        walk_error.clone()
                    };
                }
                if let (Ok(rate), true) = (&rate, first_value_refused) {
                    let error = if *rate <= 0.0 {
                        Error::NUM
                    } else {
                        Error::VALUE
                    };
                    return CalcResult::new_error(error, cell, String::new());
                }
                match (rate, &flows) {
                    (Err(error), _) => error,
                    // A rate at or below 0 is `#NUM!` before the flows' cells: an error value or
                    // a non-number first date (`XNPV(0,{-100,#N/A,60},<TRUE,0,1.005>)`,
                    // probes/t3312-xirr-xnpv-inner-calls A32; `XNPV(-1,..)` A17; T-3312).
                    (Ok(rate), _) if rate <= 0.0 && !values_error => {
                        CalcResult::new_error(Error::NUM, cell, String::new())
                    }
                    (Ok(_), Err(error)) => error.clone(),
                    (Ok(rate), Ok((values, dates))) => {
                        let xnpv = dates.as_ref().map_err(Clone::clone).and_then(|dates| {
                            compute_xnpv(rate, values, dates).map_err(|error| error.0)
                        });
                        number_result(xnpv, cell)
                    }
                }
            },
        )
    }

    /// `XIRR(values, dates, [guess])`: the guess lifts over arrays (a multi-cell range is
    /// refused); the values and dates are read once, whole, and an error in them is every
    /// element's: an error values argument before the guess's error, which comes before an error
    /// dates argument and the values' cells (T-1843, T-2611). Each element may run the bracket
    /// walks and the bisection over the flows, so the call is charged their count per element,
    /// weighted by the table's `XIRR_WEIGHT`.
    pub(crate) fn fn_xirr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let ([values, dates] | [values, dates, _]) = args else {
            return CalcResult::new_args_number_error(cell);
        };
        let values = self.evaluate_node_in_context(values, cell);
        let dates = self.evaluate_node_in_context(dates, cell);
        // Excel checks the sizes before it reads a number, and only an error dates or guess
        // comes first (financial.md "XIRR and XNPV under the gen-m9 statistical cells" and
        // "Financial functions in the gen-m9 financial sweep (T-2257)", T-2428).
        let size_error = if matches!(dates, CalcResult::Error { .. }) {
            // One values cell that is a boolean or non-numeric text is read before an error dates
            // argument: `XIRR(<TRUE>,<#DIV/0!>)` is `#VALUE!` (probes/t3816-xirr-element-reading
            // D39, T-3816), but a larger values range is not (`XIRR(<" x " in range>,<#N/A>)` is
            // `#N/A`, gen-m9/67a4c03afd63f988 A241).
            (self.flow_element_count(&values) == Some(1)
                && self.non_numeric_text_or_boolean_cell(&values))
            .then(|| CalcResult::new_error(Error::VALUE, cell, String::new()))
        } else {
            self.xirr_size_error(&values, &dates, cell)
        };
        // The dates are truncated and checked once, not per element, as XNPV's. An error dates
        // argument comes before the values' cells are read (T-2611: `XIRR(<" x " in range>,
        // <#N/A>)` is `#N/A`, gen-m9/67a4c03afd63f988 A241), but after an error values argument
        // (79d87fc527927626 A228 is the values' `#VALUE!` beside a `#DIV/0!` dates).
        let values_error =
            matches!(values, CalcResult::Error { .. }).then(|| values.clone().without_message());
        let flows = if values_error.is_none() && matches!(dates, CalcResult::Error { .. }) {
            Err(dates.clone())
        } else {
            self.xirr_flows(values, dates, cell)
        }
        .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Xirr,
            args,
            LiftCost::Cells(flow_count(
                flows.as_ref().map(|(values, _)| values.as_slice()),
            )),
            cell,
            &mut |model, arguments| {
                // The guess comes first: an error guess is that error, and a text guess that is
                // no number or a boolean guess (literal or cell, TRUE or FALSE) `#VALUE!`, before
                // an error values or dates argument, the size checks and the cells
                // (`XIRR(NA(),{2 dates},1/0)` is `#DIV/0!`, `XIRR(1/0,NA(),"x")` and
                // `XIRR({-100,110},{2 dates},TRUE)` `#VALUE!`; probes/t3312-xirr-xnpv-inner-calls
                // A51-A57, t3312-xirr-guess-dates-weibull A41-A43; T-3312). Then an error values
                // argument, then an error dates argument (T-2611: `XIRR(1E+21,<#VALUE!>,<#NUM!>)`
                // is `#NUM!`, gen-m9/1dce4774f5c0e736 A189).
                // Excel: a written empty guess (`XIRR(v,d,)`) is the default 0.1, but an empty
                // cell is guess 0; IRR and RATE read both as 0 (financial.md "XIRR (and IRR): an
                // empty guess argument is the default 0.1 for XIRR, guess 0 for IRR";
                // probes/xirr-irr-empty-guess).
                let guess = arguments
                    .get(2)
                    .filter(|guess| !matches!(guess, CalcResult::EmptyArg));
                let guess = match model.cast_number_no_bools_or(guess, 0.1, cell) {
                    Ok(guess) => guess,
                    Err(error) => return error,
                };
                if let Some(error) = &values_error {
                    return error.clone();
                }
                // A size error stands beside any guess that reads as a number, numeric text
                // included (`XIRR(<-100,110,"x">,{2 dates},"0.1")` is `#NUM!`, t3312-xirr-xnpv-
                // inner-calls A63; probes/xirr-size-mismatch-empty-guess G21-G36). A guess at or
                // below -1 turns the one-value `#N/A` into `#NUM!` (`XIRR(5,39448,-1)`,
                // t3312-xirr-guess-dates-weibull A1, A2, A7, A17), but not a single text or boolean
                // date's `#VALUE!` (A11, A12), which has no size error here.
                if let Some(size_error) = &size_error {
                    let one_text_date = matches!(
                        size_error,
                        CalcResult::Error {
                            error: Error::VALUE,
                            ..
                        }
                    );
                    if guess <= -1.0 && !one_text_date {
                        return CalcResult::new_error(Error::NUM, cell, String::new());
                    }
                    return size_error.clone();
                }
                let (values, dates) = match &flows {
                    Ok(flows) => flows,
                    Err(error) => return error.clone(),
                };
                let xirr = dates
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|dates| compute_xirr(values, dates, guess).map_err(|error| error.0));
                number_result(xirr, cell)
            },
        )
    }

    fn get_yearfrac(
        &mut self,
        start: i64,
        end: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let (days, year) = self.year_parts(start, end, basis, cell)?;
        // Signed: basis 0 counts 1900-02-28 to the fictitious 1900-02-29 as -1 day, so
        // `ACCRINTM(59,60,0.1,1000,0)` is negative (probes/discount-serial-zero).
        Ok(days / year)
    }

    /// YEARFRAC's `(day count, year length)` from `start` to `end` (`Model::year_fraction_parts`),
    /// `#NUM!` for a basis outside 0-4 or a date outside the calendar.
    fn year_parts(
        &self,
        start: i64,
        end: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        day_count_basis(basis)
            .and_then(|basis| self.year_fraction_parts(start as f64, end as f64, basis))
            .ok_or_else(|| num_error(cell))
    }

    /// Day-count parts shared by PRICEMAT and YIELDMAT, as Excel's `getMatFactors`: one year
    /// length `b`, the year of issue to settlement (so basis 1 reads the year length of that
    /// span), and the issue-to-maturity and issue-to-settlement day counts, with
    /// settlement-to-maturity their difference (financial.md "PRICEMAT and YIELDMAT count the
    /// settlement-to-maturity days").
    ///
    /// Returns `(b, dim, a, dsm)` in days.
    fn get_mat_factors(
        &self,
        issue: i64,
        settlement: i64,
        maturity: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64, f64, f64), CalcResult> {
        let (a, b) = self.year_parts(issue, settlement, basis, cell)?;
        let (dim, _) = self.year_parts(issue, maturity, basis, cell)?;
        Ok((b, dim, a, dim - a))
    }

    /// `FVSCHEDULE(principal, schedule)`: the principal lifts over arrays (a multi-cell range is
    /// refused); the schedule is read once, whole, and handed to every element, whose principal
    /// is checked first, the scalar order (T-1843).
    pub(crate) fn fn_fvschedule(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let [_, schedule] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        // Excel: text in the schedule, in a range or an array literal, is #VALUE!
        // (probes/fin-argument-modes H382-H384).
        // Typed text that reads as a number is a one-rate schedule: `FVSCHEDULE(1000,"0.1")`,
        // `("0.1")` and `T("0.1")` are 1100, `"x"` and `TRUE` are #VALUE!
        // (probes/npv-fvschedule-typed A12, A13; probes/npv-fvschedule-edges A3, A4, A6).
        let not_a_number = || CalcResult::new_error(Error::VALUE, cell, String::new());
        let schedule = match self.evaluate_node_in_context(schedule, cell) {
            CalcResult::String(text) => self
                .cast_number(&text)
                .map(|rate| vec![rate])
                .ok_or_else(not_a_number),
            // One empty cell is an empty schedule, the principal unchanged: `FVSCHEDULE(6527,D1)`
            // is 6527 with D1 empty, also through `IF(..,B3,B9)` (gen-m9/30e20ba3302fea8b A250,
            // 6376e27be7a05871 A140; T-2428).
            CalcResult::EmptyCell => Ok(Vec::new()),
            result => self.array_of_numbers_from(
                result,
                &cell,
                true,
                false,
                || Ok(None),
                || Err(not_a_number()),
            ),
        }
        .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Fvschedule,
            args,
            LiftCost::Cells(flow_count(schedule.as_deref())),
            cell,
            &mut |model, values| {
                // A boolean principal is #VALUE!, though a text number is converted
                // (probes/fin-fvschedule A18, A9).
                let principal = match values.first() {
                    Some(principal) => model.cast_number_no_bools_ref(principal, cell),
                    None => Err(CalcResult::new_args_number_error(cell)),
                };
                match (principal, &schedule) {
                    (Err(error), _) => error,
                    (Ok(_), Err(error)) => error.clone(),
                    (Ok(principal), Ok(schedule)) => {
                        let result = schedule
                            .iter()
                            .fold(principal, |result, rate| result * (1.0 + rate));
                        if result.is_infinite() {
                            return CalcResult::new_error(Error::DIV, cell, String::new());
                        }
                        CalcResult::Number(result)
                    }
                }
            },
        )
    }
}
