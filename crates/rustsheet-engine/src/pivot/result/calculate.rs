//! The values of a pivot's calculated data fields, set once its accumulators are full
//! (`pivot::calculated`, T-4174).

use super::super::calculated::Plan;
use super::{charge, units, Aggregate, PivotComputeError, PivotResult};

/// Sets the value of every calculated data field in every cell: its formula over the cell's
/// operand Sums, each read as a Sum data field over that field would show it ([`Plan`]).
pub(super) fn calculate(
    result: &mut PivotResult,
    plan: &mut Plan,
    work: &mut u64,
) -> Result<(), PivotComputeError> {
    // The operand slots follow the data fields' (`PivotResult::slots`).
    let first_operand = result.functions.len();
    let operand_count = plan.operands.len();
    let mut calculated = vec![None; result.accumulators.len()];
    charge(
        work,
        units(
            result
                .cells
                .len()
                .saturating_mul(operand_count.saturating_add(1)),
        ),
    )?;
    for (&(row, column), &base) in &result.cells {
        let base = base as usize;
        let operands = (first_operand..first_operand + operand_count)
            .map(|slot| {
                result
                    .slot_as(row, column, base, slot, Aggregate::Sum)
                    .unwrap_or(Ok(0.0))
            })
            .collect();
        let values = plan.values(operands, work)?;
        for (data_field, step) in plan.shows.iter().enumerate() {
            if let (Some(step), Some(slot)) = (step, calculated.get_mut(base + data_field)) {
                *slot = values.get(*step).cloned();
            }
        }
    }
    result.calculated = calculated;
    // An intersection no record reaches shows the formula over sums of 0: probes/
    // t4505-pivot-calc-fields LC B7:C7 (Product C has no East record) show 0 = 0×0. That it is
    // the formula's value, not a plain 0, is UNVERIFIED (T-4897): no probe has a formula whose
    // value over zeros is not 0.
    let empty = plan.values(vec![Ok(0.0); operand_count], work)?;
    result.calculated_empty = (plan.shows.iter())
        .map(|step| step.and_then(|step| empty.get(step).cloned()))
        .collect();
    Ok(())
}
