// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::float_cmp)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::clock::Clock;
use crate::expressions::types::{Area, CellReferenceIndex};
use crate::model::Model;
use crate::types::{Cell, FormulaValue};

/// The instant unit tests read as "now": 2022-11-08T11:13:28.578Z (8 November 2022 12:13 Berlin
/// time), the one the former thread-local mock clock defaulted to (SPEC-0008).
pub const TEST_CLOCK_MS: i64 = 1_667_906_008_578;

/// A fixed clock at `ms` since the Unix epoch.
pub fn fixed_clock(ms: i64) -> Clock {
    Clock::fixed_unix_millis(ms).unwrap()
}

/// The ulps between two doubles of the same sign (their bit patterns' distance; for two negative
/// doubles the sign bits cancel in the difference).
pub fn ulps(a: f64, b: f64) -> u64 {
    a.to_bits().abs_diff(b.to_bits())
}

/// Whether `actual` is within half a unit in the 15th significant digit of `expected`, the
/// digits Excel shows (decision 0083's "not visibly off"; the eval harness's
/// `closeness::within_fifteen_digits`, which the engine crate cannot depend on).
pub fn within_fifteen_digits(expected: f64, actual: f64) -> bool {
    if expected == 0.0 || !expected.is_finite() || !actual.is_finite() {
        return expected == actual;
    }
    // The decimal exponent from the shortest round-trip digits, as the harness takes it: log10
    // can round across a power of ten.
    let exponent: i32 = format!("{:e}", expected.abs())
        .split_once('e')
        .and_then(|(_, e)| e.parse().ok())
        .unwrap_or(0);
    (actual - expected).abs() <= 0.5 * 10f64.powi(exponent - 14)
}

/// The whole-column references `C:C,D:D,…`, `count` of them from column C, as one argument list.
pub fn whole_column_arms(count: i32) -> String {
    (0..count)
        .filter_map(|i| crate::expressions::utils::number_to_column(3 + i))
        .map(|name| format!("{name}:{name}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// An empty en-US model in UTC on the fixed test clock.
pub fn new_empty_model() -> Model {
    Model::new_empty_with_clock("model", "en", "UTC", "en", fixed_clock(TEST_CLOCK_MS)).unwrap()
}

/// `text` typed in R1C1 mode into `row`, `column` of sheet `sheet` of `model`
/// ([`Model::set_typed_r1c1_user_input`]): the cell's formula as A1 text, `None` when Excel
/// refuses it (the cell keeps its content).
pub fn typed_r1c1(
    model: &mut Model,
    sheet: u32,
    text: &str,
    row: i32,
    column: i32,
) -> Option<String> {
    model
        .set_typed_r1c1_user_input(sheet, row, column, text.to_string())
        .ok()?;
    model.get_cell_formula(sheet, row, column).ok().flatten()
}

/// A cell's value: a number as Rust's shortest round-trip text (so equal text is equal bits), a
/// boolean as `TRUE`/`FALSE`, anything else (an error) as the cell's text.
pub fn round_trip_value(model: &Model, cell: &str) -> String {
    match model.get_cell_value_by_ref(&format!("Sheet1!{cell}")) {
        Ok(CellValue::Number(number)) => format!("{number:?}"),
        Ok(CellValue::Boolean(true)) => "TRUE".to_string(),
        Ok(CellValue::Boolean(false)) => "FALSE".to_string(),
        _ => model.get_text_a1(cell),
    }
}

/// Sets `inputs` (cell, content) and `cells` (cell, formula, expected) on an empty model,
/// evaluates it and checks each of `cells` with [`round_trip_value`]: an expected number is
/// compared bit for bit, anything else as text.
pub fn check_cells(inputs: &[(&str, &str)], cells: &[(&str, &str, &str)]) {
    check_cells_with_numbers(&[], inputs, cells);
}

/// [`check_cells`] with `numbers` (cell, value) stored as exact doubles first: typed input keeps
/// 15 significant digits, as Excel's does, so a probe's 17-digit input goes in this way.
pub fn check_cells_with_numbers(
    numbers: &[(&str, f64)],
    inputs: &[(&str, &str)],
    cells: &[(&str, &str, &str)],
) {
    let mut model = new_empty_model();
    for (cell, number) in numbers {
        let reference = model.parse_a1(cell);
        model
            .update_cell_with_number(reference.sheet, reference.row, reference.column, *number)
            .unwrap();
    }
    for (cell, input) in inputs {
        model.set_a1(cell, input);
    }
    for (cell, formula, _) in cells {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        let expected = expected
            .parse::<f64>()
            .map_or_else(|_| (*expected).to_string(), |number| format!("{number:?}"));
        assert_eq!(
            round_trip_value(&model, cell),
            expected,
            "{cell}: {formula}"
        );
    }
}

/// A union of `leaves` copies of `leaf`, nested as a balanced tree so that its depth, about
/// three levels per halving, stays far under the parser's `MAX_FORMULA_DEPTH`.
pub fn balanced_union(leaf: &str, leaves: usize) -> String {
    if leaves <= 1 {
        return leaf.to_string();
    }
    let half = leaves / 2;
    format!(
        "({},{})",
        balanced_union(leaf, half),
        balanced_union(leaf, leaves - half)
    )
}

impl Model {
    pub fn parse_a1(&self, cell: &str) -> CellReferenceIndex {
        if cell.contains('!') {
            self.parse_reference(cell).unwrap()
        } else {
            let sheet_name = self.get_worksheets_properties()[0].name.clone();
            self.parse_reference(&format!("{sheet_name}!{cell}"))
                .unwrap()
        }
    }
    pub fn set_a1(&mut self, cell: &str, value: &str) {
        let cell_reference = self.parse_a1(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        self.set_user_input(cell_reference.sheet, row, column, value.to_string())
            .unwrap();
    }
    pub fn has_formula_a1(&self, cell: &str) -> bool {
        self.get_formula_opt_a1(cell).is_some()
    }
    pub fn get_formula_a1(&self, cell: &str) -> String {
        self.get_formula_opt_a1(cell).unwrap_or_default()
    }
    fn get_formula_opt_a1(&self, cell: &str) -> Option<String> {
        let cell_reference = self.parse_a1(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        self.get_cell_formula(cell_reference.sheet, row, column)
            .unwrap()
    }
    pub fn get_text_by_index(&self, sheet: u32, row: i32, column: i32) -> String {
        self.get_formatted_cell_value(sheet, row, column).unwrap()
    }
    pub fn get_text_a1(&self, cell: &str) -> String {
        let CellReferenceIndex { sheet, row, column } = self.parse_a1(cell);
        self.get_text_by_index(sheet, row, column)
    }
    /// Whether `cell`, or the anchor of the array it is part of, is on a loop or reads one in
    /// the last evaluation: it then holds 0 (recalc.md "A circular reference without
    /// iteration is 0").
    pub fn is_circular_a1(&self, cell: &str) -> bool {
        let CellReferenceIndex { sheet, row, column } = self.parse_a1(cell);
        let (row_a, column_a) = match self.get_cell_a1(cell) {
            Cell::SpillCell { a, .. } => *a,
            _ => (row, column),
        };
        self.get_text_by_index(sheet, row, column) == "0"
            && self.evaluation.circular.contains(&(sheet, row_a, column_a))
    }
    /// The message of the error in formula cell `cell`, which tells the engine's caps apart
    /// from a plain error of the same kind.
    pub fn get_error_message_a1(&self, cell: &str) -> String {
        match self.get_cell_a1(cell) {
            Cell::CellFormula {
                v: FormulaValue::Error { m, .. },
                ..
            }
            | Cell::ArrayFormula {
                v: FormulaValue::Error { m, .. },
                ..
            } => m.clone(),
            other => format!("not an error: {other:?}"),
        }
    }
    pub fn get_cell_a1(&self, cell: &str) -> &Cell {
        let cell_reference = self.parse_a1(cell);
        let worksheet = self.workbook.worksheet(cell_reference.sheet).unwrap();
        worksheet
            .cell(cell_reference.row, cell_reference.column)
            .unwrap()
    }

    pub fn clear_contents_at(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let area = Area {
            sheet,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.range_clear_contents(&area)
    }

    pub fn clear_all_at(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let area = Area {
            sheet,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.range_clear_all(&area)
    }
}

pub use crate::test_support::Rng;
