// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_precision_loss)]
// LINT-DEBT-END
use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::variance::squared_deviations;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    // DEVSQ(number1, [number2], ...)
    pub(crate) fn fn_devsq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values = Vec::new();
        if let Err(error) = self.for_each_value(args, cell, |value| values.push(value)) {
            return error;
        }

        if values.is_empty() {
            // Excel: DEVSQ/AVEDEV of no numbers is #NUM!, not #DIV/0!
            // (docs/excel-semantics/coercion.md, poi EverythingTests!H392).
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "DEVSQ with no numeric data".to_string(),
            );
        }

        // Excel's DEVSQ is the two-pass sum around the double mean, never the one-pass
        // Σx² - (Σx)²/n: `DEVSQ(0.1,0.2,0.3)` = 0.019999999999999997, `DEVSQ(1E15,1E15+1)` = 0.5
        // and `DEVSQ({100000000.1,100000000.2,100000000.3})` = 0.020000000596046563, where the
        // one-pass form gives 0.019999999999999976, 0 and -4 (probes/stat-order-floor-batch,
        // gen-m9; docs/excel-semantics/statistics.md "DEVSQ is a two-pass sum (T-1749)").
        let sum = values.iter().fold(0.0, |total, &x| total + x);
        let mean = flush_subnormal(sum / values.len() as f64);
        let result = squared_deviations(&values, mean);

        CalcResult::finite_number_or_num(result, cell)
    }
}
