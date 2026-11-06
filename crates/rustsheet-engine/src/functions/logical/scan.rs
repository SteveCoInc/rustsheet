use crate::cast::calc_result_to_element;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
    variable_stack::Binding,
};

impl Model {
    /// `=SCAN([initial_value], array, lambda)`
    ///
    /// Like REDUCE but returns an array of all intermediate accumulated values
    /// rather than just the final one. The output has the same shape as `array`.
    /// With 2 arguments the first element seeds the accumulator (not included in output).
    pub(crate) fn fn_scan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (data, mut caller, has_initial) = match self.fold_operands(args, cell) {
            Ok(operands) => operands,
            Err(error) => return error,
        };

        if data.rows() == 0 || data.columns() == 0 {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        let num_rows = data.rows();
        let num_cols = data.columns();

        // The accumulator is a binding, so that the first call takes `initial_value` as a reference
        // where it is one (`Model::lambda_initial_value`); each later call takes the step's value.
        let mut accumulator = if has_initial {
            self.lambda_initial_value(&args[0], cell)
        } else {
            Binding::Value(data.at(0, 0))
        };

        let mut result = vec![vec![ArrayNode::Empty; num_cols]; num_rows];

        // When no initial value, the first element seeds the accumulator and its
        // slot in the output is that seed value itself (not the result of calling lambda).
        let mut start_col = 0;
        if !has_initial {
            let seed = match self.lambda_result_value(data.at(0, 0), cell) {
                Ok(value) => value,
                Err(error) => return error,
            };
            result[0][0] = calc_result_to_element(seed);
            start_col = 1;
        }

        for (i, result_row) in result.iter_mut().enumerate() {
            let col_start = if i == 0 { start_col } else { 0 };
            for (j, slot) in result_row.iter_mut().enumerate().skip(col_start) {
                let step = self.call_lambda_caller_bound(
                    &mut caller,
                    [accumulator, Binding::Value(data.at(i, j))],
                    cell,
                );
                let value = match self.lambda_result_value(step.clone(), cell) {
                    Ok(value) => value,
                    Err(error) => return error,
                };
                accumulator = Binding::Value(step);
                *slot = calc_result_to_element(value);
            }
        }

        CalcResult::Array(result)
    }
}
