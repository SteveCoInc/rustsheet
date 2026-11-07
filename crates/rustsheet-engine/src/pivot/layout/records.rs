//! The records behind one value cell of a laid-out pivot, the drill-down of Excel's Show Details
//! (T-4184).
//!
//! A value cell reads the result node of its row line and of its column line
//! ([`AxisLine::node`]), so the records behind it are the records the filters keep
//! ([`PivotResult::records`]) whose item of each axis field above those nodes is the node's: a
//! leaf's records for an item cell, every record under the item for a subtotal, and every kept
//! record for the grand total. They are listed in cache record order. The order and the columns
//! of the sheet Excel's Show Details makes from them are unprobed (docs/excel-semantics/pivots.md
//! "Show Details", UNVERIFIED, T-4879).

use super::super::result::{charge, units, AxisTree, PivotComputeError, PivotResult};
use super::super::{PivotCache, PivotTable};
use super::lines::{AxisLine, PivotLayout};

/// The records behind the cell at 0-based (`row`, `column`) of `table`'s pivot over `cache`, laid
/// out as `layout` from `result`, in record order (see the module documentation); `None` when the
/// cell is no value cell: outside the value area, on a line that holds no values, or in a pivot
/// with no data field. A value cell no record reaches has none. One work unit is charged per kept
/// record and axis level tested.
///
/// # Errors
/// [`PivotComputeError::Malformed`] when `result` or `layout` is not over `table` and `cache`,
/// and [`PivotComputeError::Limit`] past `work`.
pub fn records_behind(
    table: &PivotTable,
    cache: &PivotCache,
    result: &PivotResult,
    layout: &PivotLayout,
    row: u32,
    column: u32,
    work: &mut u64,
) -> Result<Option<Vec<u32>>, PivotComputeError> {
    let (data_row, data_column) = layout.data_origin();
    let line = |lines: &[AxisLine], at: u32, first: u32| {
        let index = at.checked_sub(first)?;
        lines
            .get(index as usize)
            .filter(|l| l.values)
            .map(|l| l.node)
    };
    let row_node = line(&layout.rows, row, data_row);
    let column_node = line(&layout.columns, column, data_column);
    let (Some(row_node), Some(column_node)) = (row_node, column_node) else {
        return Ok(None);
    };
    if table.data_fields.is_empty() {
        return Ok(None);
    }
    // Per axis level above the two nodes: that level's field's records, and the node's item.
    let mut tests: Vec<(&[u32], u32)> = Vec::new();
    for (tree, node) in [(result.rows(), row_node), (result.columns(), column_node)] {
        push_tests(tree, node, cache, &mut tests)?;
    }
    let kept = result.records();
    charge(work, units(kept.len().saturating_mul(tests.len().max(1))))?;
    let mut records = Vec::new();
    'records: for &record in kept {
        for &(items, item) in &tests {
            let held = *items
                .get(record as usize)
                .ok_or(PivotComputeError::Malformed)?;
            if held != item {
                continue 'records;
            }
        }
        records.push(record);
    }
    Ok(Some(records))
}

/// Pushes onto `tests` one (field records, item) pair per level of `node` of `tree` and of each
/// of its ancestors but the root.
fn push_tests<'a>(
    tree: &AxisTree,
    node: u32,
    cache: &'a PivotCache,
    tests: &mut Vec<(&'a [u32], u32)>,
) -> Result<(), PivotComputeError> {
    for index in tree.ancestors(node) {
        let current = (tree.nodes().get(index as usize)).ok_or(PivotComputeError::Malformed)?;
        let Some(item) = current.item else {
            continue;
        };
        let field = (current.depth.checked_sub(1))
            .and_then(|depth| tree.fields().get(depth as usize))
            .and_then(|&field| cache.fields.get(field as usize))
            .ok_or(PivotComputeError::Malformed)?;
        tests.push((field.records.as_slice(), item));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
