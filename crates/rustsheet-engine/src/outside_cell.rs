//! Formulas evaluated outside a cell: the rules and thresholds of conditional formats, and a
//! pivot cache's defined-name source (T-4506). Each one is a formula evaluation of its own, with
//! its own area budget (`area_budget.rs`), as a cell's formula is (`run_cell_formula`).

use crate::{
    calc_result::CalcResult,
    evaluation::OrderState,
    expressions::{
        parser::Node,
        types::{CellReferenceIndex, CellReferenceRC},
    },
    model::Model,
};

impl Model {
    /// Evaluates `node` in the context of `cell` as a formula of its own, outside any cell's
    /// formula: with a fresh area budget, so that one conditional format evaluated over many
    /// cells does not spend one budget for all of them.
    pub(crate) fn evaluate_outside_cell(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.outside_cell(cell, |model| model.evaluate_node_in_context(node, cell))
    }

    /// [`Model::evaluate_outside_cell`] in reference context: the one area `node` stands for, as
    /// a `CalcResult::Range`, or the error a function taking one area gives (several areas are
    /// `#VALUE!`). A pivot cache's defined-name source is read so (`pivot_source_area`, T-4506).
    pub(crate) fn evaluate_reference_outside_cell(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.outside_cell(cell, |model| {
            model
                .evaluate_reference_arg(node, cell)
                .into_single_area(cell)
        })
    }

    /// Runs `f`, one formula's evaluation in `cell`, with an area budget of its own.
    fn outside_cell(
        &mut self,
        cell: CellReferenceIndex,
        f: impl FnOnce(&mut Self) -> CalcResult,
    ) -> CalcResult {
        // The calculation-order state lives for one formula here (SPEC-0017 "State").
        self.evaluation.order_state = OrderState::default();
        let result = self.with_area_budget(cell, f);
        self.evaluation.order_state = OrderState::default();
        result
    }

    /// Evaluates a formula string on a sheet, returning the numeric result.
    /// Assumes the workbook has already been evaluated (cell values are up-to-date).
    /// Returns `None` if the formula is invalid or does not produce a number.
    pub(crate) fn evaluate_formula(&mut self, formula: &str, sheet: u32) -> Option<f64> {
        let body = formula.trim().strip_prefix('=').unwrap_or(formula.trim());
        if body.is_empty() {
            return None;
        }
        let sheet_name = self.workbook.worksheets.get(sheet as usize)?.get_name();
        let context_rc = CellReferenceRC {
            sheet: sheet_name,
            row: 1,
            column: 1,
        };
        let mut node = self.parse_internal_formula(body, &context_rc);
        // Evaluation reads no whitespace (see `ParsedFormula`).
        node.strip_spaces();
        let context_index = CellReferenceIndex {
            sheet,
            row: 1,
            column: 1,
        };
        match self.evaluate_outside_cell(&node, context_index) {
            CalcResult::Number(n) => Some(n),
            _ => None,
        }
    }
}
