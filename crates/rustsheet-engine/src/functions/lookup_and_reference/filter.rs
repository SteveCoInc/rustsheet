use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::spill_functions::reject_empty_value,
    model::Model,
};

/// Whether an include element keeps its row or column, or the error it makes FILTER return. A
/// number keeps it unless 0, a blank does not, text is a boolean only when it spells TRUE or FALSE
/// in any case (`"true"` keeps it, `" TRUE"` and `"1"` are `#VALUE!`), an error is itself
/// (docs/excel-semantics/arrays.md "FILTER: the type of each include element and of the result";
/// probes/t3654-filter-result-types P20:P42).
fn include_keeps(node: &ArrayNode, cell: CellReferenceIndex) -> Result<bool, CalcResult> {
    match node {
        ArrayNode::Boolean(b) => Ok(*b),
        ArrayNode::Number(n) => Ok(*n != 0.0),
        ArrayNode::Empty => Ok(false),
        ArrayNode::String(text) => Model::cast_bool_ref(&CalcResult::String(text.clone()), cell)
            .map_err(|_| {
                CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "include array must contain numbers or booleans".to_string(),
                )
            }),
        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "include array must contain numbers or booleans".to_string(),
        )),
        ArrayNode::Error(e) => Err(CalcResult::new_error(
            e.clone(),
            cell,
            "include array contains an error".to_string(),
        )),
    }
}

/// Whether `include`, a mask for an array of `rows` x `cols`, filters its rows (`Some(true)`) or
/// its columns (`Some(false)`): a column or a row as long as the rows, else a row as long as the
/// columns; `None` when it is neither. A column as tall as the array is wide is `None`, not a
/// column filter: Excel gives `#VALUE!` for `FILTER(A1:B3,{TRUE;FALSE})`, `{1;0}`, `{1;#N/A}` and
/// `FILTER(A1:C2,{1;0;1})` (probes/t4517-filter-transposed-include A10:C12, E10:F12; T-4517).
fn include_axis(include: &[Vec<ArrayNode>], rows: usize, cols: usize) -> Option<bool> {
    let height = include.len();
    let width = include.first().map_or(0, Vec::len);
    if (height == rows && width == 1) || (height == 1 && width == rows) {
        Some(true)
    } else if height == 1 && width == cols {
        Some(false)
    } else {
        None
    }
}

/// FILTER's `#VALUE!` for an include of the wrong shape.
fn include_shape_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "include array dimensions must match array rows or columns".to_string(),
    )
}

impl Model {
    /// FILTER's array (`keep_blank`, a lone blank cell kept blank) or include, read as a
    /// reference where it is one, so an error read through a one-cell reference is a 1x1 array
    /// holding it whether the reference is written, a LET variable, or picked by INDEX or IF:
    /// `FILTER(A1:C3,LET(x,G9,x))`, `FILTER(A1:C3,INDEX(G1:G9,9))` and
    /// `FILTER(A1:C3,IF(TRUE,G9))` are `#VALUE!` for the include's shape, `FILTER(LET(x,G9,x),G2)`
    /// `#CALC!` with G2 0 and `FILTER(INDEX(G1:G9,9),TRUE)` a 1x1 `#DIV/0!`, G9 being `#DIV/0!`
    /// (probes/t4477-filter-include-error-shapes J51:L59, J67:L71; `FILTER(Data!G9,Data!G2)`
    /// `#CALC!`, gen-m7/68e7f10771a7118a A10798, T-3506). A literal error is still the error
    /// (`FILTER(#DIV/0!,Data!C1:C12<>0.3)`, gen-m7/ee81c689b5fde629 A977).
    // Excel: docs/excel-semantics/arrays.md "A one-cell error reference is a 1x1 array in TOCOL,
    // TOROW and FILTER (T-3497)".
    fn filter_arg_array(
        &mut self,
        arg: &Node,
        keep_blank: bool,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let result = self
            .evaluate_picked_reference_arg(arg, cell)
            .into_single_area(cell);
        let result = reject_empty_value(arg, result, cell)?;
        if keep_blank {
            self.result_to_array_keep_blank(result, cell)
        } else {
            self.result_to_array(result, cell)
        }
    }

    /// `=FILTER(array, include, [if_empty])`
    ///
    /// Returns only the rows of array where the corresponding include value is truthy.
    ///   * include  – a column vector of booleans/numbers with the same height as array
    ///   * `if_empty` – value returned when no rows pass the filter (default: #CALC! error)
    pub(crate) fn fn_filter(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        // A lone blank cell stays a blank element, as in a range: `ISBLANK(FILTER(A2,TRUE))` is
        // TRUE and `FILTER(A2,TRUE)&"|"` is "|" (probes/t3654-filter-result-types Q63:Q66).
        let data = match self.filter_arg_array(&args[0], true, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        let include = match self.filter_arg_array(&args[1], false, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        // The include's shape is checked before its elements: a wrongly shaped include holding
        // an error is `#VALUE!` (`FILTER(A1:A3,{1;#N/A})`, `FILTER(A1:A3,G8:G9)` with G9
        // `#DIV/0!`), and so is a one-cell error reference that does not fit
        // (`FILTER(Data!A1:D12,Data!G9,Data!G3)`, gen-m7/d7a127bd9334c2b4 A2380, T-3506), while
        // one that fits a one-row or one-column array is its error (`FILTER(A1:A3,G9)` and
        // `FILTER(A1:C1,G9)` `#DIV/0!`; probes/t4477-filter-include-error-shapes J11:L47).
        let Some(filter_rows) = include_axis(&include, num_rows, num_cols) else {
            return include_shape_error(cell);
        };

        // Every element is checked, the excluded ones too: an error anywhere is returned
        // (`FILTER(C1:C3,{0;#N/A;0})` is #N/A; probes/t3654-filter-result-types P35).
        let mut keep: Vec<Vec<bool>> = Vec::with_capacity(include.len());
        for row in &include {
            let mut kept = Vec::with_capacity(row.len());
            for node in row {
                match include_keeps(node, cell) {
                    Ok(k) => kept.push(k),
                    Err(error) => return error,
                }
            }
            keep.push(kept);
        }

        let include_flat: Vec<bool> = keep.into_iter().flatten().collect();

        let filtered: Vec<Vec<ArrayNode>> = if filter_rows {
            data.into_iter()
                .enumerate()
                .filter_map(|(i, row)| if include_flat[i] { Some(row) } else { None })
                .collect()
        } else {
            let col_indices: Vec<usize> = include_flat
                .iter()
                .enumerate()
                .filter_map(|(i, &keep)| if keep { Some(i) } else { None })
                .collect();
            data.iter()
                .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
                .collect()
        };

        let is_empty_result =
            filtered.is_empty() || filtered.first().is_none_or(std::vec::Vec::is_empty);

        if is_empty_result {
            if args.len() >= 3 {
                let empty_val = self.evaluate_node_in_context(&args[2], cell);
                match empty_val {
                    CalcResult::EmptyArg => CalcResult::new_error(
                        Error::CALC,
                        cell,
                        "No data returned by FILTER".to_string(),
                    ),
                    // A range is its values: FILTER gives values, never a reference, found or not
                    // (Excel-cached: upstream ISREF.xlsx Sheet1!D11, D14, D17, D20; probes/isref-names-filter-if-empty).
                    // A blank one-cell reference is 0 (`ROWS` 1, `&"|"` "0|"), unlike a blank
                    // inside a larger range, which stays blank (probes/t3654-filter-result-types
                    // P46:X46, P55:X55).
                    CalcResult::EmptyCell => CalcResult::Number(0.0),
                    CalcResult::Range { left, right } => {
                        match self.evaluate_range(left, right, cell) {
                            Ok(array) => CalcResult::Array(array),
                            Err(error) => error,
                        }
                    }
                    // An array if_empty spills as is (Excel: probes/filter-if-empty-array).
                    CalcResult::Array(a) => CalcResult::Array(a),
                    // A scalar stays a scalar, not a 1x1 array: `MAX(FILTER(.., .., ""))` with no
                    // match is #VALUE! because MAX gets the text "" as a value (upstream
                    // MAX_MIN.xlsx H39, H40, H43).
                    v => v,
                }
            } else {
                CalcResult::new_error(Error::CALC, cell, "No data returned by FILTER".to_string())
            }
        } else {
            CalcResult::Array(filtered)
        }
    }
}
