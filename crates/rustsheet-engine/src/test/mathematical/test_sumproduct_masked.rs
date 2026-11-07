// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_precision_loss)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

//! SUMPRODUCT's `block*mask*...` argument read from the cached block (T-3230) gives what the
//! operators' arrays give: values, errors in masked-out cells, mask errors and shapes that are no
//! masks.

use crate::cell::CellValue;
use crate::expressions::types::CellReferenceIndex;
use crate::test::util::new_empty_model;
use crate::Model;

/// Sheet2: keys in A2:A41 (clean) and P2:P41 (an error at row 30), headers in B1:G1 (clean) and
/// I1:N1 (an error at L1), a block of numbers, booleans, blanks and numeric text in B2:G41, and
/// the same with a text, `#DIV/0!` and `#NUM!` in I2:N41. Sheet3!B2:G41 holds formulas that are
/// not evaluated yet when Sheet1's first call reads them.
fn fill(model: &mut Model) {
    model.new_sheet();
    model.new_sheet();
    for (index, column) in ["B", "C", "D", "E", "F", "G"].iter().enumerate() {
        model.set_a1(&format!("Sheet2!{column}1"), &format!("h{}", index % 3));
    }
    for (index, column) in ["I", "J", "K", "L", "M", "N"].iter().enumerate() {
        let header = if *column == "L" {
            "=NA()".to_string()
        } else {
            format!("h{}", index % 3)
        };
        model.set_a1(&format!("Sheet2!{column}1"), &header);
    }
    for row in 2..=41 {
        model.set_a1(&format!("Sheet2!A{row}"), &format!("k{}", row % 5));
        let key = if row == 30 {
            "=1/0".to_string()
        } else {
            format!("k{}", row % 5)
        };
        model.set_a1(&format!("Sheet2!P{row}"), &key);
        for (index, (clean, dirty)) in ["B", "C", "D", "E", "F", "G"]
            .iter()
            .zip(["I", "J", "K", "L", "M", "N"])
            .enumerate()
        {
            let value = match (row + index) % 11 {
                3 => String::new(),
                5 => "TRUE".to_string(),
                7 => "'7".to_string(),
                _ => format!("{}", (row as f64 * (index as f64 + 1.0) - 50.0) / 8.0),
            };
            model.set_a1(&format!("Sheet2!{clean}{row}"), &value);
            let dirty_value = match (row, dirty) {
                (10, "J") => "abc".to_string(),
                (35, "M") => "=1/0".to_string(),
                (5, "N") => "=SQRT(-1)".to_string(),
                _ => value.clone(),
            };
            model.set_a1(&format!("Sheet2!{dirty}{row}"), &dirty_value);
            model.set_a1(
                &format!("Sheet3!{clean}{row}"),
                &format!("=Sheet2!{clean}{row}*2"),
            );
        }
    }
}

/// `fill`, and errors placed to meet at one position: Sheet2!Q2:V41 holds numbers but `#NUM!` at
/// Q30 (the row of P30's `#DIV/0!`), X2:AC41 numbers but `#NUM!` at Y10 (between row 30's first
/// cell and P30's error, counting row by row), Q1:V1 headers but `#N/A` at Q1, and O2:O41 keys but
/// `#DIV/0!` at O2.
fn fill_meeting_errors(model: &mut Model) {
    fill(model);
    let columns = ["Q", "R", "S", "T", "U", "V"];
    for (index, column) in columns.iter().enumerate() {
        let header = if index == 0 {
            "=NA()".to_string()
        } else {
            format!("h{}", index % 3)
        };
        model.set_a1(&format!("Sheet2!{column}1"), &header);
    }
    for row in 2..=41 {
        let key = if row == 2 {
            "=1/0".to_string()
        } else {
            format!("k{}", row % 5)
        };
        model.set_a1(&format!("Sheet2!O{row}"), &key);
        for (index, (q, x)) in columns
            .iter()
            .zip(["X", "Y", "Z", "AA", "AB", "AC"])
            .enumerate()
        {
            let number = format!("{}", row * 10 + index);
            let error = "=SQRT(-1)".to_string();
            let q_value = if (row, *q) == (30, "Q") {
                &error
            } else {
                &number
            };
            let x_value = if (row, x) == (10, "Y") {
                &error
            } else {
                &number
            };
            model.set_a1(&format!("Sheet2!{q}{row}"), q_value);
            model.set_a1(&format!("Sheet2!{x}{row}"), x_value);
        }
    }
}

/// Errors of the block and of masks that meet at one position: the block's error wins there,
/// then the masks' in operator order; a row mask's error is first met at the start of its row.
#[test]
fn errors_meeting_at_one_position() {
    let clean_h = "(Sheet2!$B$1:$G$1=\"h1\")";
    let error_h = "(Sheet2!$Q$1:$V$1=\"h1\")";
    let p_key = "(Sheet2!$P$2:$P$41=\"k2\")";
    let o_key = "(Sheet2!$O$2:$O$41=\"k2\")";
    // Evaluates the blocks' formulas, so that every call below reads them cached.
    let mut formulas = vec!["=COUNT(Sheet2!$Q$2:$V$41,Sheet2!$X$2:$AC$41)".to_string()];
    for block in ["Sheet2!$Q$2:$V$41", "Sheet2!$X$2:$AC$41"] {
        for (h, k) in [(clean_h, p_key), (error_h, o_key), (error_h, p_key)] {
            formulas.push(format!("=SUMPRODUCT({block}*{h}*{k})"));
            formulas.push(format!("=SUMPRODUCT({block}*{k}*{h})"));
        }
    }
    let masked = assert_alike(&fill_meeting_errors, &formulas);
    assert_eq!(masked, formulas.len() - 1);
}

/// Each formula's text, and whether it is marked circular, with the pass caches on and off, and
/// how many arguments the cached run read as masked blocks.
fn cached_and_walked(
    fill: &dyn Fn(&mut Model),
    formulas: &[String],
) -> (Vec<(String, String)>, usize) {
    cached_and_walked_at(fill, formulas, "A")
}

/// [`cached_and_walked`] with the formulas in Sheet1's `column`.
fn cached_and_walked_at(
    fill: &dyn Fn(&mut Model),
    formulas: &[String],
    column: &str,
) -> (Vec<(String, String)>, usize) {
    let run = |no_pass_caches: bool| {
        let mut model = new_empty_model();
        model.evaluation.no_pass_caches = no_pass_caches;
        fill(&mut model);
        for (index, formula) in formulas.iter().enumerate() {
            model.set_a1(&format!("{column}{}", index + 1), formula);
        }
        model.evaluate();
        let texts: Vec<String> = (1..=formulas.len())
            .map(|row| {
                let at = CellReferenceIndex {
                    sheet: 0,
                    row: i32::try_from(row).unwrap(),
                    column: crate::expressions::utils::column_to_number(column).unwrap(),
                };
                let circular = model
                    .evaluation
                    .circular
                    .contains(&crate::evaluation::key(at));
                format!(
                    "{} circular={circular}",
                    model.get_text_a1(&format!("{column}{row}"))
                )
            })
            .collect();
        (texts, model.evaluation.ifs_cache.masked)
    };
    let (cached, masked) = run(false);
    let (walked, _) = run(true);
    (cached.into_iter().zip(walked).collect(), masked)
}

fn assert_alike(fill: &dyn Fn(&mut Model), formulas: &[String]) -> usize {
    let (results, masked) = cached_and_walked(fill, formulas);
    for ((cached, walked), formula) in results.iter().zip(formulas) {
        assert_eq!(cached, walked, "{formula}");
    }
    masked
}

/// Every block, header row, key column, criterion and operand order: errors and text anywhere in
/// the block, errors in either mask, and their positions against each other.
#[test]
fn masked_blocks_match_the_operators() {
    let mut formulas = Vec::new();
    for block in [
        "Sheet2!$B$2:$G$41",
        "Sheet2!$I$2:$N$41",
        "Sheet3!$B$2:$G$41",
    ] {
        for header in ["Sheet2!$B$1:$G$1", "Sheet2!$I$1:$N$1"] {
            for key in ["Sheet2!$A$2:$A$41", "Sheet2!$P$2:$P$41"] {
                for x in ["\"h0\"", "\"h2\"", "\"zz\""] {
                    for y in ["\"k0\"", "\"k3\"", "\"none\""] {
                        let h = format!("({header}={x})");
                        let k = format!("({key}={y})");
                        for argument in [
                            format!("{block}*{h}*{k}"),
                            format!("{block}*{k}*{h}"),
                            format!("{block}*{h}"),
                            format!("({block}*{k})"),
                            format!("{block}*{k}*{h}*{k}"),
                        ] {
                            formulas.push(format!("=SUMPRODUCT({argument})"));
                        }
                    }
                }
            }
        }
    }
    let masked = assert_alike(&fill, &formulas);
    assert!(masked >= 400, "the masked path ran {masked} times");
}

/// Operands that are no masks (a mismatched width, a one-cell range, a number, a mask already
/// multiplied out, a text comparison result), other arguments beside a masked one, and a block too
/// small to cache: the operators run as before. A 2-D range of the block's shape is a weight
/// (T-3725), with errors and text in it.
#[test]
fn operands_that_are_no_masks() {
    let h = "(Sheet2!$B$1:$G$1=\"h1\")";
    let k = "(Sheet2!$A$2:$A$41=\"k2\")";
    let block = "Sheet2!$B$2:$G$41";
    let formulas: Vec<String> = [
        format!("=SUMPRODUCT({block}*(Sheet2!$B$1:$F$1=\"h1\")*{k})"),
        format!("=SUMPRODUCT({block}*{k}*(Sheet2!$A$2:$A$40=\"k2\"))"),
        format!("=SUMPRODUCT({block}*{h}*Sheet2!$I$2:$N$41)"),
        format!("=SUMPRODUCT({block}*{h}*Sheet2!$B$2)"),
        format!("=SUMPRODUCT({block}*{h}*2)"),
        format!("=SUMPRODUCT({block}*({h}*1)*{k})"),
        format!("=SUMPRODUCT({block}*{h}*(Sheet2!$A$2:$A$41&\"\"))"),
        format!("=SUMPRODUCT({block}*{h}*{{1,2,3,4,5,6}})"),
        format!("=SUMPRODUCT({block}*{h}*{k},{block})"),
        format!("=SUMPRODUCT({block},{block}*{h}*{k})"),
        format!("=SUMPRODUCT({block}*{h}*{k},Sheet2!$B$2:$G$40)"),
        format!("=SUMPRODUCT(Sheet2!$I$2:$N$41*{h},{block}*{k})"),
        format!("=IFERROR(SUMPRODUCT(Sheet2!$I$2:$N$41*{h}*{k}),\"\")"),
        "=SUMPRODUCT(Sheet2!$B$2:$C$9*(Sheet2!$B$1:$C$1=\"h1\"))".to_string(),
        format!("=SUMPRODUCT({h}*{block}*{k})"),
        format!("=SUMPRODUCT({block}/{h}*{k})"),
    ]
    .into_iter()
    .collect();
    let masked = assert_alike(&fill, &formulas);
    assert!(masked >= 6, "the masked path ran {masked} times");
}

/// A block cell that reads the call back: the cell is circular, so the block is never cached and
/// the walk reports the cycle as before.
#[test]
fn a_cycle_through_the_block() {
    let with_cycle = |model: &mut Model| {
        fill(model);
        model.set_a1("Sheet2!D20", "=Sheet1!A1");
    };
    let formulas: Vec<String> = (0..3)
        .map(|_| {
            "=SUMPRODUCT(Sheet2!$B$2:$G$41*(Sheet2!$B$1:$G$1=\"h1\")*(Sheet2!$A$2:$A$41=\"k0\"))"
                .to_string()
        })
        .collect();
    assert_alike(&with_cycle, &formulas);
}

/// One value worked by hand: columns D and G (header h2) at the rows keyed k0 (5, 10, ..., 40),
/// each cell as `fill` writes it (a blank 0, TRUE 1, the text "7" 7).
#[test]
fn a_masked_sum_by_hand() {
    let mut model = new_empty_model();
    fill(&mut model);
    model.set_a1(
        "A1",
        "=SUMPRODUCT(Sheet2!$B$2:$G$41*(Sheet2!$B$1:$G$1=\"h2\")*(Sheet2!$A$2:$A$41=\"k0\"))",
    );
    model.evaluate();
    let expected: f64 = (5..=40)
        .step_by(5)
        .flat_map(|row: i32| [2, 5].map(|index| (row, index)))
        .map(|(row, index)| match (row + index) % 11 {
            3 => 0.0,
            5 => 1.0,
            7 => 7.0,
            _ => f64::from(row * (index + 1) - 50) / 8.0,
        })
        .sum();
    assert_eq!(model.get_text_a1("A1"), format!("{expected}"));
    assert_eq!(model.evaluation.ifs_cache.masked, 1);
}

/// A block past the operators' array limit (2^20 cells) is refused by them (`#ERROR!`), not read
/// as a masked block.
#[test]
fn a_block_past_the_array_limit() {
    let fill = |model: &mut Model| {
        model.set_a1("A1", "h");
        model.set_a1("B1", "k");
        model.set_a1("A600000", "3");
    };
    let formulas = [
        "=SUMPRODUCT(Sheet1!$A$1:$B$600000*(Sheet1!$A$1:$B$1=\"h\")*(Sheet1!$A$1:$A$600000=3))"
            .to_string(),
    ];
    let (results, masked) = cached_and_walked_at(&fill, &formulas, "D");
    assert_eq!(results[0].0, results[0].1);
    assert!(results[0].0.starts_with("#ERROR!"), "{}", results[0].0);
    assert_eq!(masked, 0);
}

/// Sheet2 for weights (T-3725), every range large enough to cache: a block B2:BM81 (64 columns,
/// 80 rows) of numbers, blanks, booleans and numeric text, headers B1:BM1, keys A2:A81, and
/// weights: a clean column BO2:BO81 and row B83:BM83, a column BP2:BP81 with a text, `TRUE`, a
/// blank, `#DIV/0!` at BP50 and a text that is no number at BP70, a row B84:BM84 with `#N/A` at
/// BJ84, a column BQ2:BQ81 of 1E+300 (with the block's 1E+10 at I40, a product overflows),
/// and Sheet3!BO2:BO81, formulas not evaluated when Sheet1's first call reads them.
fn fill_weights(model: &mut Model) {
    use crate::expressions::utils::number_to_column;
    model.new_sheet();
    model.new_sheet();
    let column = |index: i32| number_to_column(index + 2).unwrap();
    for index in 0..64 {
        let c = column(index);
        model.set_a1(&format!("Sheet2!{c}1"), &format!("h{}", index % 3));
        model.set_a1(
            &format!("Sheet2!{c}83"),
            &format!("{}", f64::from(index % 7) - 2.5),
        );
        let dirty = if index == 60 {
            "=NA()".to_string()
        } else {
            format!("{}", index % 5)
        };
        model.set_a1(&format!("Sheet2!{c}84"), &dirty);
    }
    for row in 2..=81 {
        model.set_a1(&format!("Sheet2!A{row}"), &format!("k{}", row % 5));
        for index in 0..64 {
            let value = match (row + index) % 13 {
                3 => String::new(),
                5 => "TRUE".to_string(),
                7 => "'7".to_string(),
                _ if (row, index) == (40, 7) => "1E+10".to_string(),
                _ => format!("{}", f64::from(row * (index + 1) - 50) / 8.0),
            };
            model.set_a1(&format!("Sheet2!{}{row}", column(index)), &value);
        }
        let weight = f64::from(row % 9) * 0.375 - 1.25;
        model.set_a1(&format!("Sheet2!BO{row}"), &format!("{weight}"));
        let dirty = match row {
            10 => "'2.5".to_string(),
            20 => "TRUE".to_string(),
            30 => String::new(),
            50 => "=1/0".to_string(),
            70 => "abc".to_string(),
            _ => format!("{weight}"),
        };
        model.set_a1(&format!("Sheet2!BP{row}"), &dirty);
        model.set_a1(&format!("Sheet2!BQ{row}"), "1E+300");
        model.set_a1(&format!("Sheet3!BO{row}"), &format!("=Sheet2!BO{row}*2"));
    }
}

/// A weight anywhere after the block, of each shape, beside masks and other weights, with errors
/// and text in the weight and the block, gives what the operators give; one whose products could
/// overflow, one not cached yet, one cell, and a mismatched shape go to the operators.
#[test]
fn weights_match_the_operators() {
    let block = "Sheet2!$B$2:$BM$81";
    let h = "(Sheet2!$B$1:$BM$1=\"h1\")";
    let k = "(Sheet2!$A$2:$A$81=\"k2\")";
    let column = "Sheet2!$BO$2:$BO$81";
    let row = "Sheet2!$B$83:$BM$83";
    let dirty_column = "Sheet2!$BP$2:$BP$81";
    let dirty_row = "Sheet2!$B$84:$BM$84";
    // Evaluates Sheet2's formulas first; Sheet3's stay unevaluated until a walk reads them.
    let mut formulas = vec![format!("=COUNT({block},{dirty_column},{dirty_row})")];
    for weight in [column, row, dirty_column, dirty_row, block] {
        for argument in [
            format!("{block}*{h}*{weight}"),
            format!("{block}*{weight}*{h}"),
            format!("{block}*{weight}"),
            format!("{block}*{k}*{weight}*{h}"),
            format!("{block}*{h}*{weight}*{k}*{column}"),
        ] {
            formulas.push(format!("=SUMPRODUCT({argument})"));
        }
    }
    // The row weight's `#N/A` (row 2, column 61) comes before the column weight's errors (rows
    // 50 and 70, column 1 of the block, row by row).
    for (first, second) in [(dirty_column, dirty_row), (dirty_row, dirty_column)] {
        formulas.push(format!("=SUMPRODUCT({block}*{first}*{second})"));
        formulas.push(format!("=SUMPRODUCT({block}*{k}*{first}*{h}*{second})"));
    }
    let weighted = formulas.len() - 1;
    for refused in [
        format!("{block}*{h}*Sheet2!$BQ$2:$BQ$81"),
        format!("{block}*{h}*Sheet3!$BO$2:$BO$81"),
        format!("{block}*{h}*Sheet2!$BO$2"),
        format!("{block}*{h}*Sheet2!$BO$2:$BO$80"),
        format!("{block}*{h}*Sheet2!$B$83:$BL$83"),
        format!("{block}*{h}*Sheet2!$BO$2:$BP$81"),
    ] {
        formulas.push(format!("=SUMPRODUCT({refused})"));
    }
    let (results, masked) = cached_and_walked(&fill_weights, &formulas);
    for ((cached, walked), formula) in results.iter().zip(&formulas) {
        assert_eq!(cached, walked, "{formula}");
    }
    assert_eq!(
        masked, weighted,
        "the weighted calls did not all read masked blocks"
    );
    let overflow = &results[weighted + 1].0;
    assert!(overflow.starts_with("#NUM!"), "{overflow}");
}

/// Weighted values worked by hand: the block's columns with header h1 at the rows keyed k2,
/// times the clean weight column; and one cell, C2, picked by a row and a column weight, whose
/// product with its weight rounds twice, as `*` rounds it (probes-numeric/double-rounding-ops A1,
/// `power_tests.rs`), a different double from the plain product.
#[test]
#[expect(
    clippy::float_cmp,
    reason = "the two products must differ in their last bit"
)]
fn a_weighted_sum_by_hand() {
    let mut model = new_empty_model();
    fill_weights(&mut model);
    let a = 1.0 + 57_546_701_686_154.0 * f64::EPSILON;
    let b = 1.0 + 791_744_674_158_799.0 * f64::EPSILON;
    assert_ne!(a * b, crate::arithmetic::extended_product(a, b));
    model.set_a1("Sheet2!C2", "=1+57546701686154*2^-52");
    for row in 2..=81 {
        model.set_a1(&format!("Sheet2!BS{row}"), if row == 2 { "1" } else { "0" });
    }
    for index in 0..64 {
        let c = crate::expressions::utils::number_to_column(index + 2).unwrap();
        let weight = if index == 1 {
            "=1+791744674158799*2^-52"
        } else {
            "0"
        };
        model.set_a1(&format!("Sheet2!{c}85"), weight);
    }
    // Evaluates C2 and C85 first, so that the calls below read the blocks cached.
    model.set_a1("A3", "=COUNT(Sheet2!$B$2:$BM$81,Sheet2!$B$85:$BM$85)");
    model.set_a1(
        "A4",
        "=SUMPRODUCT(Sheet2!$B$2:$BM$81*Sheet2!$B$85:$BM$85*Sheet2!$BS$2:$BS$81)",
    );
    model.set_a1(
        "A5",
        "=SUMPRODUCT(Sheet2!$B$2:$BM$81*(Sheet2!$B$1:$BM$1=\"h1\")*(Sheet2!$A$2:$A$81=\"k2\")\
         *Sheet2!$BO$2:$BO$81)",
    );
    model.evaluate();
    let mut total = 0.0;
    for row in (2..=81).filter(|row| row % 5 == 2) {
        for index in (0..64).filter(|index| index % 3 == 1) {
            let value = match (row + index) % 13 {
                _ if (row, index) == (2, 1) => a,
                3 => 0.0,
                5 => 1.0,
                7 => 7.0,
                _ if (row, index) == (40, 7) => 1E+10,
                _ => f64::from(row * (index + 1) - 50) / 8.0,
            };
            let weight = f64::from(row % 9) * 0.375 - 1.25;
            total = crate::arithmetic::extended_sum(
                total,
                crate::arithmetic::extended_product(value, weight),
            );
        }
    }
    let number = |cell: &str| model.get_cell_value_by_ref(cell);
    assert_eq!(number("Sheet1!A5"), Ok(CellValue::Number(total)));
    let product = crate::arithmetic::extended_product(a, b);
    assert_eq!(number("Sheet1!A4"), Ok(CellValue::Number(product)));
    assert_eq!(model.evaluation.ifs_cache.masked, 2);
}

/// `*` flushes a product below the smallest normal double to 0 (`times`; numbers.md "Subnormal
/// results become 0"), so a masked block flushes each product too: a cell times a weight that
/// underflows stays 0 when a second weight, or an earlier SUMPRODUCT argument, would grow it back
/// to a normal number. Sheet2!A1:A64 holds 1 and 1E-200 and a kept subnormal formula result,
/// B1:B64 1E-110, C1:C64 and D1:D64 1E+100, and E1:E64 a mask key.
#[test]
fn underflowing_products_flush_as_the_operators_do() {
    let fill = |model: &mut Model| {
        model.new_sheet();
        for row in 1..=64 {
            let value = match row % 4 {
                0 => "1E-200",
                // A kept subnormal, built by subtraction (numbers.md).
                1 => "=3E-308-2.5E-308",
                _ => "1",
            };
            model.set_a1(&format!("Sheet2!A{row}"), value);
            model.set_a1(&format!("Sheet2!B{row}"), "1E-110");
            model.set_a1(&format!("Sheet2!C{row}"), "1E+100");
            model.set_a1(&format!("Sheet2!D{row}"), "1E+100");
            model.set_a1(&format!("Sheet2!E{row}"), &format!("k{}", (row % 4).min(2)));
        }
    };
    let block = "Sheet2!$A$1:$A$64";
    // k0 picks the 1E-200 rows, k1 the kept subnormals: only their products are summed.
    let mask = "(Sheet2!$E$1:$E$64=\"k0\")";
    let subnormals = "(Sheet2!$E$1:$E$64=\"k1\")";
    let tiny = "Sheet2!$B$1:$B$64";
    let big = "Sheet2!$C$1:$C$64";
    let formulas: Vec<String> = vec![
        format!("=COUNT({block})"),
        format!("=SUMPRODUCT({block}*{mask}*{tiny}*{big})"),
        format!("=SUMPRODUCT({block}*{tiny}*{mask}*{big})"),
        format!("=SUMPRODUCT(Sheet2!$D$1:$D$64,{block}*{mask}*{tiny})"),
        format!("=SUMPRODUCT(Sheet2!$D$1:$D$64,{block}*{subnormals})"),
        format!("=SUMPRODUCT({block}*{subnormals}*{big})"),
    ];
    let (results, masked) = cached_and_walked(&fill, &formulas);
    for ((cached, walked), formula) in results.iter().zip(&formulas) {
        assert_eq!(cached, walked, "{formula}");
    }
    assert_eq!(
        masked,
        formulas.len() - 1,
        "every SUMPRODUCT read a masked block"
    );
}

/// Masked-off positions are skipped only when no product can overflow (T-3732): a grid argument
/// before a masked one whose product is past the double range at a masked-off position is
/// `#NUM!` there (inf times the mask's 0; numbers.md "Overflow in `+ - * /` is #NUM! where it
/// happens"), as the operators give it. With the masked argument first, the masked-off products
/// are 0 before any large factor and stay finite. Sheet4!B2:G41 holds 1 where the mask keeps a
/// cell and 1E+200 elsewhere.
#[test]
fn an_overflow_at_a_masked_off_position_is_not_skipped() {
    let with_big = |model: &mut Model| {
        fill(model);
        model.new_sheet();
        for (index, column) in ["B", "C", "D", "E", "F", "G"].iter().enumerate() {
            for row in 2..=41 {
                let kept = index % 3 == 1 && row % 5 == 2;
                let value = if kept { "1" } else { "1E+200" };
                model.set_a1(&format!("Sheet4!{column}{row}"), value);
            }
        }
    };
    let masked = "Sheet2!$B$2:$G$41*(Sheet2!$B$1:$G$1=\"h1\")*(Sheet2!$A$2:$A$41=\"k2\")";
    let big = "Sheet4!$B$2:$G$41";
    let formulas = vec![
        format!("=SUMPRODUCT({big},{big},{masked})"),
        format!("=SUMPRODUCT({big},{masked})"),
        format!("=SUMPRODUCT({masked},{big},{big})"),
    ];
    let (results, masked_count) = cached_and_walked(&with_big, &formulas);
    for ((cached, walked), formula) in results.iter().zip(&formulas) {
        assert_eq!(cached, walked, "{formula}");
    }
    assert!(results[0].0.starts_with("#NUM!"), "{}", results[0].0);
    assert_eq!(masked_count, formulas.len());
}
