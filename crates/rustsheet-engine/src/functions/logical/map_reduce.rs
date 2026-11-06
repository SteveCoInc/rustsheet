use super::bycol_byrow::LambdaGrid;
use super::lambda::LambdaCaller;
use crate::cast::calc_result_to_element;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    limits::check_array_cells,
    model::Model,
    variable_stack::Binding,
};

impl Model {
    /// `=MAP(array1, [array2, ...], lambda)`
    ///
    /// Applies the lambda element-wise across one or more arrays; smaller arrays pad with `#N/A`.
    /// The lambda receives one scalar value from each array per call.
    pub(crate) fn fn_map(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let n_arrays = args.len() - 1;

        let function = self.evaluate_node_in_context(&args[n_arrays], cell);
        let mut arrays: Vec<LambdaGrid> = Vec::with_capacity(n_arrays);
        for (index, arg) in args[..n_arrays].iter().enumerate() {
            let data = match self.lambda_cells(arg, cell, index > 0, true) {
                Ok(d) => d,
                Err(e) => return e,
            };
            arrays.push(data);
        }

        // The arrays are read before the function is checked: an error array wins over a function
        // that cannot be called (docs/excel-semantics/arrays.md "LAMBDA helpers refuse a function
        // they cannot call", probes/t3753-lambda-helper-error-array-order).
        let mut caller = match self.helper_caller(&function, n_arrays, cell) {
            Ok(caller) => caller,
            Err(error) => return error,
        };

        if arrays
            .iter()
            .any(|arr| arr.rows() == 0 || arr.columns() == 0)
        {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        // Arrays of different sizes pad to the largest rows and columns: an element missing from
        // any array is #N/A and the LAMBDA is not called for it, a scalar or a row next to a column
        // does not broadcast (probes/map-unequal-sizes, excel-graph 2026-10-17; arrays.md "MAP
        // over arrays of different sizes").
        let num_rows = arrays.iter().map(LambdaGrid::rows).max().unwrap_or(0);
        let num_cols = arrays.iter().map(LambdaGrid::columns).max().unwrap_or(0);
        // The padded shape is known before allocating it (DEVIATION, arrays.md "Array results
        // above 1,048,576 cells").
        if let Some((error, message)) = check_array_cells(num_rows, num_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let mut result = vec![vec![ArrayNode::Error(Error::NA); num_cols]; num_rows];
        // One argument buffer for every element.
        let mut values: Vec<CalcResult> = Vec::with_capacity(arrays.len());

        for (i, result_row) in result.iter_mut().enumerate() {
            for (j, slot) in result_row.iter_mut().enumerate() {
                if arrays
                    .iter()
                    .any(|arr| i >= arr.rows() || j >= arr.columns())
                {
                    continue;
                }
                values.clear();
                values.extend(arrays.iter().map(|arr| arr.at(i, j)));
                let cell_result = self.call_lambda_caller(&mut caller, values.drain(..), cell);
                let cell_result = match self.lambda_result_value(cell_result, cell) {
                    Ok(value) => value,
                    Err(error) => return error,
                };
                *slot = calc_result_to_element(cell_result);
            }
        }

        CalcResult::Array(result)
    }

    /// The array and the function of SCAN and REDUCE (`[initial_value], array, lambda`), and
    /// whether an initial value was given. The array is read before the function is checked: an
    /// error array wins over a function that cannot be called (docs/excel-semantics/arrays.md
    /// "LAMBDA helpers refuse a function they cannot call", probes/t3753-lambda-helper-error-array-order).
    pub(super) fn fold_operands(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(LambdaGrid, LambdaCaller, bool), CalcResult> {
        if args.len() < 2 || args.len() > 3 {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let (array_idx, lambda_idx, has_initial) = if args.len() == 3 {
            (1, 2, true)
        } else {
            (0, 1, false)
        };
        let function = self.evaluate_node_in_context(&args[lambda_idx], cell);
        let data = self.lambda_cells(&args[array_idx], cell, false, false)?;
        let caller = self.helper_caller(&function, 2, cell)?;
        Ok((data, caller, has_initial))
    }

    /// `=REDUCE([initial_value], array, lambda)`
    ///
    /// Accumulates a result by applying the lambda to each element of `array`.
    /// The lambda takes `(accumulator, current_value)` and returns the new accumulator.
    /// With 2 arguments the first element of the array seeds the accumulator.
    pub(crate) fn fn_reduce(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (data, caller, has_initial) = match self.fold_operands(args, cell) {
            Ok(operands) => operands,
            Err(error) => return error,
        };

        let columns = data.columns();
        let count = data.rows() * columns;

        if count == 0 {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        // A reference the LAMBDA returns stays one, as the next step's accumulator and as the
        // result: `REDUCE(A1,A2:A3,LAMBDA(a,v,IF(ISREF(a),v,"no")))` is A3 (3), and ISREF of a
        // REDUCE that ends on a reference is TRUE (probes/t4478-reduce-scan-reference-accumulator
        // Calc!E2:E3, H2, E8, E11, E20:H21). SCAN reads it instead (`fn_scan`, Calc!H13).
        let mut caller = caller.keeping_trimrange_references().keeping_references();
        // The first call takes `initial_value` bound as a LAMBDA argument
        // (`Model::lambda_initial_value`).
        let mut accumulator = if has_initial {
            let initial = self.lambda_initial_value(&args[0], cell);
            self.call_lambda_caller_bound(
                &mut caller,
                [initial, (Binding::Value(data.at(0, 0)), false)],
                cell,
            )
        } else {
            data.at(0, 0)
        };
        for index in 1..count {
            accumulator = self.call_lambda_caller(
                &mut caller,
                [accumulator, data.at(index / columns, index % columns)],
                cell,
            );
        }

        accumulator
    }

    /// SCAN's and REDUCE's `initial_value` as the LAMBDA's first accumulator: bound as a LAMBDA
    /// call's argument is ([`Model::bind_argument`]), so a cell, and an IF, CHOOSE, INDEX,
    /// XLOOKUP, OFFSET or INDIRECT that picks one, a LET variable bound to any of them and a union
    /// reach the LAMBDA as references, and IFERROR as a value: `REDUCE(IF(TRUE,A2),A3,
    /// LAMBDA(a,v,COUNT(a)))` with A2 the text "1" is 0, `REDUCE((A1,A3),A1,LAMBDA(a,v,PRODUCT(a)))`
    /// with A1 TRUE and A3 2.5 is 2.5 and ISREF(a) is TRUE (probes/t4475-scan-reduce-reference-initial
    /// D2:I9; gen-m7/2c5ea078f25443a2 A1767, T-2307). A 3-D span is `#VALUE!`, not a reference
    /// (D10:I10: COUNT(a) 0, PRODUCT(a) `#VALUE!`, ISREF(a) FALSE). An empty `initial_value`
    /// is read as before, not as an omitted parameter (T-2670). docs/excel-semantics/arrays.md
    /// "SCAN and REDUCE keep errors in the LAMBDA" (probes/t4478-reduce-scan-reference-accumulator
    /// Calc!G17:H19 agree).
    pub(super) fn lambda_initial_value(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> (Binding, bool) {
        match self.bind_argument(node, cell) {
            (Binding::Omitted, _) => (
                Binding::Value(self.evaluate_node_in_context(node, cell)),
                false,
            ),
            (span @ Binding::Span(_), marked) => (Binding::Value(span.value(cell)), marked),
            bound => bound,
        }
    }
}
