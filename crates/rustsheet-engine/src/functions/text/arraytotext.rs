// DOLLAR, FIXED, NUMBERVALUE, PROPER, REPLACE, ARRAYTOTEXT

use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
    number_format::number_to_text,
};

impl Model {
    /// ARRAYTOTEXT(array, [format])
    /// format=0 (default, concise): values joined with "," (columns) and ";" (rows)
    /// format=1 (strict): same but wrapped in {} and strings are quoted
    ///
    /// The array is read whole; `format` lifts over a range or an array literal and spills to its
    /// shape, one text per element (docs/excel-semantics/text.md "ARRAYTOTEXT: a range format
    /// argument", gen-m5/af89ea54bae02c71 Calc!A31:B32, A529:B530,
    /// probes/arraytotext-array-literal-format A1:B1; T-3884).
    pub(crate) fn fn_arraytotext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !Function::Arraytotext
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // Excel: an error value in the first argument is the result before the format is looked at,
        // so `ARRAYTOTEXT(SINH(134217727),2)` is `#NUM!` and `ARRAYTOTEXT(NA(),1/0)` is `#N/A`; an
        // error inside an array does not count (`ARRAYTOTEXT({1,#N/A},2)` is `#VALUE!`)
        // (docs/excel-semantics/text.md#arraytotext-error-before-format,
        // probes/arraytotext-error-vs-format).
        let first = self.evaluate_node_in_context(&args[0], cell);
        let from_range = matches!(first, CalcResult::Range { .. });
        // `Err` is what every format element gives instead of text: the first argument's error,
        // or `#VALUE!` for a blank, read before the element's own format, so a range format spills
        // that error to its shape (gen-m5/af89ea54bae02c71 Calc!A463:B464, A466:B467, A511:B512,
        // A514:B515).
        let source: Result<Vec<Vec<ArrayNode>>, CalcResult> = match first {
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                // Excel: a 1x1 range holding an empty cell is as much an empty cell as a bare reference;
                // only a range, not an array (GROUPBY hands a one-blank group over as an array, which
                // gives "": probes/groupby-blank-values).
                Ok(array) if matches!(array.as_slice(), [row] if matches!(row.as_slice(), [ArrayNode::Empty])) => {
                    Err(empty_cell_error(cell))
                }
                Ok(array) => Ok(array),
                Err(error) => Err(error),
            },
            CalcResult::Array(a) => Ok(a),
            // Excel: a scalar is a one-element array, so strict form braces it
            // (docs/excel-semantics/text.md#arraytotext-number-text, probes/arraytotext-number-text).
            CalcResult::Number(n) => Ok(vec![vec![ArrayNode::Number(n)]]),
            CalcResult::String(s) => Ok(vec![vec![ArrayNode::String(s)]]),
            CalcResult::Boolean(b) => Ok(vec![vec![ArrayNode::Boolean(b)]]),
            // Excel: a single empty cell (also a 1x1 range, INDEX/IF result or an omitted argument) is
            // #VALUE!, even over an error format (`ARRAYTOTEXT(Data!H1,#DIV/0!)`, gen-m5/af89ea54bae02c71
            // Calc!A499), while an empty cell inside a larger range is empty text
            // (docs/excel-semantics/text.md#arraytotext-empty-cell, probes/arraytotext-empty-ref).
            CalcResult::EmptyCell | CalcResult::EmptyArg => Err(empty_cell_error(cell)),
            CalcResult::Lambda(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Lambda not supported".to_string(),
            )),
        };
        let mut texts: [Option<CalcResult>; 2] = [None, None];
        self.lift_call(Function::Arraytotext, args, cell, &mut |model, values| {
            let array = match &source {
                Ok(array) => array,
                Err(error) => return error.clone(),
            };
            let format = match values.get(1) {
                Some(format) => match model.calc_result_to_number(format) {
                    Ok(f) => f.floor() as i32,
                    Err(error) => return CalcResult::new_error(error, cell, String::new()),
                },
                None => 0,
            };
            if format != 0 && format != 1 {
                // A one-cell range holding an error is that error too (`ARRAYTOTEXT(B1:B1,2)` with
                // B1 `#N/A` is `#N/A`; probes/arraytotext-error-vs-format A14).
                if let (true, [row]) = (from_range, array.as_slice()) {
                    if let [ArrayNode::Error(error)] = row.as_slice() {
                        return CalcResult::new_error(
                            error.clone(),
                            cell,
                            "ARRAYTOTEXT of an error".to_string(),
                        );
                    }
                }
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "format must be 0 or 1".to_string(),
                );
            }
            // An element reads only as 0 or 1, so at most two texts are built.
            texts[format as usize]
                .get_or_insert_with(|| arraytotext_render(array, format == 1, cell))
                .clone()
        })
    }
}

/// The text of `array` for ARRAYTOTEXT: concise (`strict` false) or braced and quoted.
fn arraytotext_render(
    array: &[Vec<ArrayNode>],
    strict: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    // A nested GROUPBY or PIVOTBY cell makes the whole call `#CALC!` (SPEC-0018 rule 4,
    // probes/t2592-nested-groupby-cells A10).
    if array
        .iter()
        .flatten()
        .any(|v| matches!(v, ArrayNode::Nested(_)))
    {
        return CalcResult::new_error(
            Error::CALC,
            cell,
            "ARRAYTOTEXT of a nested array".to_string(),
        );
    }
    // Excel: a scalar is a one-element array, so strict form braces it, as a larger array.
    let result = if strict {
        // Strict: {col1,col2;row2col1,row2col2}
        let rows: Vec<String> = array
            .iter()
            .map(|row| {
                row.iter()
                    .map(|v| format_array_value(v, true))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect();
        format!("{{{}}}", rows.join(";"))
    } else {
        // Concise: flatten all elements row-by-row, join with ", "
        let elements: Vec<String> = array
            .iter()
            .flat_map(|row| row.iter().map(|v| format_array_value(v, false)))
            .collect();
        elements.join(", ")
    };
    CalcResult::String(result.into())
}

fn empty_cell_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "ARRAYTOTEXT of an empty cell".to_string(),
    )
}

fn format_array_value(node: &ArrayNode, strict: bool) -> String {
    match node {
        // Excel: General text, 15 significant digits, the text `&` and VALUETOTEXT give
        // (docs/excel-semantics/text.md#arraytotext-number-text).
        ArrayNode::Number(n) => number_to_text(*n),
        ArrayNode::Boolean(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        ArrayNode::String(s) => {
            if strict {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                (**s).to_owned()
            }
        }
        ArrayNode::Error(e) => format!("{e}"),
        ArrayNode::Empty => String::new(),
        // Excel: a LAMBDA element is the letter lambda (probes/lambda-arrays B28).
        ArrayNode::Lambda(_) => "\u{3bb}".to_string(),
        // Never reached: an array holding one is `#CALC!` (`fn_arraytotext`).
        ArrayNode::Nested(_) => format!("{}", Error::CALC),
    }
}
