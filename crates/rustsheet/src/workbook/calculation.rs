//! Recalculation and the calculation settings of a [`Workbook`].

use rustsheet_engine::types::Iteration;

use super::Workbook;

impl Workbook {
    /// Recalculates every formula. The first recalculation of a loaded workbook then refreshes
    /// each pivot whose cache is `refreshOnLoad` and recalculates again, so formulas that read
    /// pivot cells or GETPIVOTDATA see the refreshed pivots (SPEC-0032 "Who refreshes, and when";
    /// pivots.md, probes/t4128-pivot-spike R). Other pivots keep the cells the file saved.
    pub fn recalculate(&mut self) {
        self.model.evaluate();
        if std::mem::take(&mut self.pivots_due) {
            self.refresh_pivots_on_load();
        }
    }

    /// The iterative-calculation settings (`calcPr@iterate`), `None` when iteration is off.
    #[must_use]
    pub fn iteration(&self) -> Option<Iteration> {
        self.model.workbook.settings.iteration
    }

    /// Turns iterative calculation on with `iteration`'s count and delta, or off with `None`.
    /// It applies from the next recalculation (docs/excel-semantics/recalc.md).
    pub fn set_iteration(&mut self, iteration: Option<Iteration>) {
        self.model.workbook.settings.iteration = iteration;
    }
}
