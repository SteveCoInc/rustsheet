//! CHOOSECOLS and CHOOSEROWS: the columns or rows of an array picked by a list of indexes.
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_possible_truncation)]
// LINT-DEBT-END

use crate::functions::spill_functions::one_cell_error_ref_as_array;
use crate::functions::util::array_node_to_calc_result;
use crate::limits::check_array_cells;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// One index argument: whether it is an array or multi-cell range, and its elements or its error.
type IndexArg = (bool, Result<Vec<Vec<ArrayNode>>, CalcResult>);

impl Model {
    // ── CHOOSECOLS ────────────────────────────────────────────────────────────

    /// `=CHOOSECOLS(array, col_num1, [col_num2], ...)`
    ///
    /// Returns the specified columns from an array or range.
    /// Column numbers are 1-based; negative numbers select from the right.
    pub(crate) fn fn_choosecols(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }

        // Excel: arrays.md "A one-cell error reference in the other array functions (T-3667)". A lone
        // blank cell stays blank (`CHOOSECOLS(Z1,1)&"x"` is "x"; probes/t4448-lone-blank-reshapers
        // A12:D12, T-4448).
        let data = self.eval_array_arg_keep_blank(&args[0], cell);
        let data = match one_cell_error_ref_as_array(&args[0], data, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }

        let col_indices = match self.chosen_indices(&args[1..], data[0].len(), data.len(), cell) {
            Ok(indices) => indices,
            Err(error) => return error,
        };

        let result = data
            .iter()
            .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
            .collect();

        CalcResult::Array(result)
    }

    // ── CHOOSEROWS ────────────────────────────────────────────────────────────

    /// `=CHOOSEROWS(array, row_num1, [row_num2], ...)`
    ///
    /// Returns the specified rows from an array or range.
    /// Row numbers are 1-based; negative numbers select from the bottom.
    pub(crate) fn fn_chooserows(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }

        // A lone blank cell stays blank (`CHOOSEROWS(Z1,1)&"x"` is "x";
        // probes/t4448-lone-blank-reshapers A11:D11, A13:D13, T-4448).
        let data = self.eval_to_array_keep_blank(&args[0], cell);
        let data = match one_cell_error_ref_as_array(&args[0], data, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() {
            return CalcResult::Array(vec![]);
        }

        let width = data[0].len();
        match self.chosen_indices(&args[1..], data.len(), width, cell) {
            Ok(indices) => CalcResult::Array(indices.iter().map(|&r| data[r].clone()).collect()),
            Err(error) => error,
        }
    }

    /// The 0-based indexes CHOOSECOLS or CHOOSEROWS picks out of `count` columns or rows, each
    /// picked one `other` cells long. Every argument is a list read whole, not lifted: a range or
    /// an array of indexes adds its elements in row-major order, whatever its orientation
    /// (`CHOOSECOLS(A1:B3,A1:A2)` and `CHOOSECOLS(A1:B3,{1,2})` are both 3x2,
    /// `CHOOSEROWS(A1:B3,A1:A2)` is 2x2; probes/lookup-argument-modes F7:G10,
    /// docs/excel-semantics/lookup.md "Lookup and reference function argument modes"). An index
    /// is 1-based, negative from the end; 0 or past `count` is `#VALUE!`, and an error element is
    /// itself. An error in any argument beats that `#VALUE!`, so the scan goes on past a bad index
    /// (docs/excel-semantics/lookup.md "CHOOSEROWS and CHOOSECOLS: an error argument beats an
    /// out-of-range index"; which problem wins: the entry "which problem wins in the index list").
    /// The result is bounded before it is built (CLAUDE.md directive 5).
    fn chosen_indices(
        &mut self,
        args: &[Node],
        count: usize,
        other: usize,
        cell: CellReferenceIndex,
    ) -> Result<Vec<usize>, CalcResult> {
        let count = i64::try_from(count).unwrap_or(i64::MAX);
        // Excel scans every single-value argument first, in argument order, and only then the array and
        // multi-cell range arguments, in argument order (docs/excel-semantics/lookup.md "CHOOSEROWS
        // and CHOOSECOLS: which problem wins in the index list", T-4718). So `CHOOSECOLS(r,{"x"},NA())`
        // is `#N/A` and `CHOOSECOLS(r,"x",NA())` is `#VALUE!`.
        let mut lists: Vec<IndexArg> = Vec::with_capacity(args.len());
        // Elements read so far, in range or not: the scan goes on past a bad index, so the bound counts
        // what is read, not only what is picked.
        let mut scanned = 0usize;
        for arg in args {
            let result = self.evaluate_node_in_context(arg, cell);
            let multi = match &result {
                CalcResult::Array(_) => true,
                CalcResult::Range { left, right } => left != right,
                _ => false,
            };
            let list = self.result_to_array(result, cell);
            if let Ok(list) = &list {
                let added = list.len().saturating_mul(list.first().map_or(0, Vec::len));
                scanned = scanned.saturating_add(added);
                if let Some((error, message)) = check_array_cells(scanned, other) {
                    return Err(CalcResult::new_error(error, cell, message));
                }
            }
            lists.push((multi, list));
        }
        let mut per_arg: Vec<Vec<usize>> = vec![Vec::new(); lists.len()];
        let mut out_of_range = false;
        for pass_multi in [false, true] {
            for (slot, (multi, list)) in lists.iter().enumerate() {
                if *multi != pass_multi {
                    continue;
                }
                let list = match list {
                    Ok(list) => list,
                    Err(error) => return Err(error.clone()),
                };
                // A 2-D index is `#VALUE!` whatever it holds, errors included (probes/t4718 A21:A26).
                if list.len() > 1 && list.first().is_some_and(|row| row.len() > 1) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Index must be a single row or column".to_string(),
                    ));
                }
                for node in list.iter().flatten() {
                    let n =
                        self.cast_number_ref(&array_node_to_calc_result(node, cell), cell)? as i64;
                    if n == 0 || n.unsigned_abs() > count.unsigned_abs() {
                        out_of_range = true;
                        continue;
                    }
                    let index = if n > 0 { n - 1 } else { count + n };
                    per_arg[slot].push(usize::try_from(index).unwrap_or(0));
                }
            }
        }
        if out_of_range {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Index out of range".to_string(),
            ));
        }
        let indices: Vec<usize> = per_arg.into_iter().flatten().collect();
        Ok(indices)
    }
}
