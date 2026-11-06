//! Scoring expectations against an evaluated workbook (moved out of runner.rs, T-4124).

use super::{Expectation, MISSING_SHEET};
use crate::accuracy::math::{Attached, MathTable};
use crate::cellref::Addr;
use crate::deviations::{self, Outcome};
use crate::engine::Evaluated;
use crate::runner::{compare, CellKind, CellResult, Target, Verdict};
use crate::value::Value;

/// Whether an excel-graph expectation of 0 hides `actual`. The Graph API reports a subnormal
/// cell value as 0.0, so its 0 cannot tell a true 0 from a kept subnormal: Excel keeps
/// `3E-308-2.5E-308` yet Graph reports 0 (probes/subnormal-function-results A69). Only cells
/// whose companion `(x)=0` is FALSE qualify (`from_graph`), so the evidence is Excel's.
/// Decision 0042.
pub(super) fn graph_hides_subnormal(e: &Expectation, actual: &Value) -> bool {
    e.from_graph
        && e.expected == Value::Number(0.0)
        && matches!(actual, Value::Number(a) if *a != 0.0 && a.abs() < f64::MIN_POSITIVE)
}

/// Scores every expectation against the evaluated workbook. `listed` marks every cell as listed
/// in the suite's `deviations` file, as a whole-case pin lists them (decision 0168), so a cell
/// that matches Excel reads as a stale line.
pub(super) fn score(
    evaluated: &dyn Evaluated,
    expectations: &[Expectation],
    listed: bool,
) -> Vec<CellResult> {
    expectations
        .iter()
        .map(|e| {
            let mut graph_subnormal = false;
            let (actual, mut verdict) = match evaluated.value(&e.sheet, e.addr) {
                Some(actual) if graph_hides_subnormal(e, &actual) => {
                    graph_subnormal = true;
                    (actual, Verdict::Pass)
                }
                Some(actual) => {
                    let verdict = compare(&e.expected, &actual);
                    (actual, verdict)
                }
                None => (Value::Error(MISSING_SHEET.into()), Verdict::TypeMismatch),
            };
            if let Some(pinned) = &e.pinned {
                if deviations::classify(pinned, &e.expected, &actual) == Outcome::Deviation {
                    verdict = Verdict::Deviation;
                }
            }
            CellResult {
                sheet: e.sheet.clone(),
                cell: e.addr.to_string(),
                formula: e.formula.clone(),
                anchor_formula: e.anchor_formula.clone(),
                expected: e.expected.clone(),
                actual,
                verdict,
                from_excel: e.from_excel,
                listed_deviation: listed || e.pinned.is_some(),
                graph_subnormal,
                kind: CellKind::Scored,
                math: None,
                target: Target::Excel,
            }
        })
        .collect()
}

/// Attaches each cell's math value from `table` (see [`CellResult::math`]); a value for another
/// formula, or for a cell with no formula of its own, is dropped.
pub(super) fn attach_math(cells: &mut [CellResult], table: &MathTable) {
    for cell in cells {
        let Some(addr) = Addr::parse(&cell.cell) else {
            continue;
        };
        if let Attached::Value(m) = table.attach(&cell.sheet, addr, cell.formula.as_deref()) {
            cell.math = Some(m.value.clone());
        }
    }
}
