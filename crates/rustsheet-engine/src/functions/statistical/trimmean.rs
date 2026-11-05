//! TRIMMEAN. The data argument is read and sorted once per call (`sorted_data.rs`); `percent`
//! lifts over arrays and ranges (probes/stat-argument-modes H549-H550, probes/stat-whole-lift
//! W35-W37).

use super::percentile::{num_error, value_at};
use super::sorted_data::{DataArg, DataOrder};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::LiftCost;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// TRIMMEAN's data, smallest first. Each element averages the values it keeps, so a call is
/// charged elements × numbers (`LiftCost::Cells`), not the sort alone.
const TRIMMEAN_DATA: DataArg = DataArg {
    // Read once in reference context, as PERCENTILE's data (T-2344): a name, INDEX, OFFSET, IF or
    // CHOOSE that picks a cell comes back as a range, so a lone cell holding nothing, text or a
    // boolean is data with no number, `#NUM!` (`TRIMMEAN(Data!B7,1.005)` with B7 TRUE, gen-m9
    // 9735c5ab72fa5ecf A264; `TRIMMEAN(D28!A6,3.14159)` with A6 blank, probes/t2475-stat-
    // subexpressions P!A353; the name, INDEX and IF forms, probes/t2883-stat-name-index-data
    // A2-A12, T-2883).
    read: |model, node, cell| match model
        .evaluate_picked_reference_arg(node, cell)
        .into_single_sheet_area(cell)
    {
        // An error value is that error, not `#VALUE!`: `TRIMMEAN(NA(),2)` is `#N/A`, the data's
        // error before the bad `percent` (probes/stat-test-lift P82, T-1982).
        error @ CalcResult::Error { .. } => Err(error),
        // A typed numeric-text or boolean literal is one number, unlike a cell holding one
        // (`TRIMMEAN("2.5",0)` is 2.5, `TRIMMEAN(TRUE,0)` is 1, gen-m9/4fc6bdaec0e5583d A199-A253,
        // A391-A439, T-3781); other text stays `#VALUE!` (`TRIMMEAN("abc",0)`, A259-A385).
        CalcResult::String(text) => match model.cast_number(&text) {
            Some(number) => Ok(vec![number]),
            None => model.numeric_flat_with_element_errors(CalcResult::String(text), cell),
        },
        CalcResult::Boolean(b) => Ok(vec![f64::from(u8::from(b))]),
        value => model.numeric_flat_with_element_errors(value, cell),
    },
    order: DataOrder::Ascending,
    cost: LiftCost::Cells,
};

impl Model {
    /// `TRIMMEAN(array, percent)`: the mean of the data after removing `floor(n*percent/2)`
    /// values from each end.
    pub(crate) fn fn_trimmean(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_with_sorted_data(
            Function::Trimmean,
            args,
            &TRIMMEAN_DATA,
            cell,
            &mut |model, data, values| {
                // percent's error comes before the no-number `#NUM!`, as PERCENTILE's k
                // (`TRIMMEAN(<text cells>,<#N/A>)` is `#N/A`, gen-m9/3e98b26cf6713e77 A247).
                let percent = match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                if data.is_empty() {
                    return num_error(cell);
                }
                if !(0.0..1.0).contains(&percent) {
                    return num_error(cell);
                }
                let sorted = data.sorted();
                let n = sorted.len();
                // Number of values to remove from each end (floored)
                let trim = (n as f64 * percent / 2.0).floor() as usize;
                match sorted.get(trim..n.saturating_sub(trim)) {
                    Some(trimmed) if !trimmed.is_empty() => {
                        CalcResult::Number(trimmed.iter().sum::<f64>() / trimmed.len() as f64)
                    }
                    _ => num_error(cell),
                }
            },
        )
    }
}
