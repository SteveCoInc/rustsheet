//! Show values as (SPEC-0032 stage 6, T-4163): a post-pass over a pivot's computed values that
//! replaces each value of a data field with a `showDataAs` or x14 `pivotShowAs` mode by what
//! the mode shows. Every (row node, column node) is computed, leaves, subtotals and grand totals
//! alike, from the plain values of stage 3 ([`PivotResult::plain_as`]); an intersection no
//! record reaches is a value too, since several modes show it (0, `#NULL!`, a running total).
//!
//! The rules are docs/excel-semantics/pivots.md "Show-values-as modes as Excel for the web renders
//! them" (probes/t4129-pivot-show-as, t4129-pivot-show-as-x14) and "Show-values-as edge cases
//! (T-4161)" (probes/t4161-pivot-show-as-base-items, -running-rank, -zeros, -edges, -running-zero,
//! -nested-parent). "Empty" below is an empty cell, `v` the cell's plain value:
//! - **percentOfRow, percentOfCol, percentOfTotal** divide `v` (empty is 0) by the row's, the
//!   column's or the pivot's grand total; a denominator of 0 is `#DIV/0!`. **index** is
//!   `v` × grand total / row total / column total, and `#DIV/0!` when the row or column total is
//!   0. The order of its operations matches every t4129 and t4161 index cell, but only t4129
//!   IX!B5 tells the orders apart, by 1 ulp, below the web-versus-desktop floor (UNVERIFIED,
//!   T-4754).
//! - **percentOfParentRow** and **percentOfParentCol** divide `v` (empty is 0) by the cell of the
//!   node's parent on that axis, the grand total line by itself; **percentOfParent** by the cell
//!   of the node's ancestor on the base field, every node above the base field empty. A parent
//!   of 0 or empty leaves the cell empty.
//! - **difference, percent, percentDiff** compare `v` with the base cell: the node with the base
//!   field's item replaced by the base item ((previous) and (next) are the sibling before and
//!   after it, the first or last item its own base), the other axis's node kept. A node above the
//!   base field is empty. A base item its parent lacks is `#N/A` on every cell below that parent;
//!   a base item that exists over a missing deeper item leaves the cell empty. The base item's own
//!   cells are empty for difference and percentDiff, and `v`/`v` for percent. Otherwise
//!   difference is `v` − base (empty is 0); percent and percentDiff are `#NULL!` for an empty
//!   `v`, empty for an empty base, `#DIV/0!` for a base of 0, and `v`/base or (`v` − base)/base.
//! - **runTotal, percentOfRunningTotal, rankAscending, rankDescending** walk a *run*: the nodes
//!   that share everything but the base field's item, under one parent of the base field, in
//!   display order (Products within a Region; under the outer Region, the same Product, or the
//!   subtotal rows, across Regions). The running total adds the run's values so far (empty is 0).
//!   percentOfRunningTotal divides it by the run's total (its last running total: for a subtotal
//!   row under the outer field, the Grand Total row); over a total of 0 a cell whose own value is
//!   0 is empty and one with a nonzero value `#DIV/0!`. The ranks are dense ranks of the run's present
//!   values, empty cells unranked and empty.
//!
//! The engine's own readings, where no probe shows Excel's (pivots.md, UNVERIFIED, T-4750): a base
//! field on no axis makes every cell `#N/A`; a `baseItem` that names no item of its field makes
//! every cell below the base field's parents `#N/A` (Excel for the web refused the one probed
//! file, `baseItem="9"`); a base item that exists over a missing deeper item leaves a difference
//! cell empty, as it leaves percentDiff's (t4161 QRN); an empty value over a percentOfRunningTotal
//! total of 0 is empty; an error value, base or denominator is the cell's error (the value's
//! first), and a running total past an error stays that error; a rank leaves errors out of the
//! run; a cell whose denominator line has no record at all (a `showAll` item) is empty; a result
//! that is not finite is `#NUM!`, and -0 is shown as 0; `baseItem` indexes the base field's
//! `<items>`, as ECMA-376 reads it (the probes' fields list their items in cache order, so it is
//! also the cache item); and a custom subtotal line of another function than the data field's
//! shows its plain value.
//!
//! The pass charges one work unit per cell of each shown data field, the cells of a run again for
//! percentOfRunningTotal and a sort per run and column for the ranks, and counts every shown
//! value against [`super::MAX_PIVOT_NODES`] as it is stored (CLAUDE.md directive 5).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::comparison_chain, clippy::match_same_arms)]
// LINT-DEBT-END

use std::collections::HashMap;

use crate::expressions::token::Error;

use super::result::{charge, count_nodes, sort_units, units, AxisTree, PivotComputeError};
use super::{ItemType, PivotResult, PivotTable, ShowDataAs};

/// `baseItem` (previous): the item before (MS-OI29500 §18.10.1.22).
const PREVIOUS: u32 = 1_048_828;
/// `baseItem` (next): the item after.
const NEXT: u32 = 1_048_829;

/// A cell: `None` is empty.
type Cell = Option<Result<f64, Error>>;

/// One data field's shown values by (row node, column node); a cell not in it is empty.
pub(super) type Shown = HashMap<(u32, u32), Result<f64, Error>>;

/// Per data field of `table`, its shown values, or `None` for a field shown as itself.
///
/// # Errors
/// [`PivotComputeError::Limit`] past `work` or the result's node count.
pub(super) fn show_values_as(
    result: &PivotResult,
    table: &PivotTable,
    work: &mut u64,
    nodes: &mut usize,
) -> Result<Vec<Option<Shown>>, PivotComputeError> {
    let mut all = Vec::with_capacity(table.data_fields.len());
    for (data_field, field) in table.data_fields.iter().enumerate() {
        let mode = field.show_data_as;
        if mode == ShowDataAs::Normal {
            all.push(None);
            continue;
        }
        let cells = units(result.rows().nodes().len())
            .saturating_mul(units(result.columns().nodes().len()));
        charge(work, cells)?;
        let mut pass = Pass {
            result,
            data_field,
            on_rows: true,
            shown: Shown::new(),
            nodes,
        };
        match mode {
            ShowDataAs::Normal => {}
            ShowDataAs::PercentOfRow
            | ShowDataAs::PercentOfColumn
            | ShowDataAs::PercentOfTotal
            | ShowDataAs::Index => pass.totals(mode)?,
            ShowDataAs::PercentOfParentRow => pass.parents()?,
            ShowDataAs::PercentOfParentColumn => {
                pass.on_rows = false;
                pass.parents()?;
            }
            _ => match base_axis(result, field.base_field) {
                None => pass.no_base()?,
                Some((on_rows, depth)) => {
                    pass.on_rows = on_rows;
                    match mode {
                        ShowDataAs::Difference | ShowDataAs::Percent | ShowDataAs::PercentDiff => {
                            let item = base_item(table, field.base_field, field.base_item);
                            pass.compare(mode, depth, item, work)?;
                        }
                        ShowDataAs::PercentOfParent => pass.of_parent(depth)?,
                        _ => pass.runs(mode, depth, work)?,
                    }
                }
            },
        }
        all.push(Some(pass.shown));
    }
    Ok(all)
}

/// The axis of `base_field` (whether it is the row axis) and the depth of its nodes.
fn base_axis(result: &PivotResult, base_field: i32) -> Option<(bool, u32)> {
    let field = u32::try_from(base_field).ok()?;
    let depth = |tree: &AxisTree| {
        let at = tree.fields().iter().position(|&f| f == field)?;
        u32::try_from(at + 1).ok()
    };
    depth(result.rows())
        .map(|d| (true, d))
        .or_else(|| depth(result.columns()).map(|d| (false, d)))
}

/// The base item `base_item` of `base_field`: (previous), (next), or the cache item of the
/// field's `<items>` entry (the cache item itself when the field lists none); `None` for an
/// index that names no data item, whose cells are `#N/A` (UNVERIFIED, T-4750: Excel for the web
/// refused the one probed file, `baseItem="9"` on a two-item field; pivots.md "Pivot flags Excel
/// for the web refuses").
fn base_item(table: &PivotTable, base_field: i32, base_item: u32) -> BaseItem {
    match base_item {
        PREVIOUS => BaseItem::Previous,
        NEXT => BaseItem::Next,
        index => {
            let field = usize::try_from(base_field)
                .ok()
                .and_then(|f| table.fields.get(f));
            let item = match field {
                Some(f) if !f.items.is_empty() => f
                    .items
                    .get(index as usize)
                    .filter(|i| i.kind == ItemType::Data)
                    .and_then(|i| i.index),
                _ => Some(index),
            };
            item.map_or(BaseItem::None, BaseItem::Item)
        }
    }
}

/// What a comparison mode compares with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BaseItem {
    Previous,
    Next,
    Item(u32),
    /// An index that names no item.
    None,
}

/// A node's base node for difference, percent and percentDiff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Base {
    /// The node is above the base field: its cells are empty.
    Above,
    /// The node is the base item's own (or below it).
    Own,
    /// The base node.
    Node(u32),
    /// The base item exists, but not the node's deeper items under it: the cells are empty.
    Absent,
    /// The node's parent of the base field lacks the base item: `#N/A`.
    Missing,
}

/// One data field's pass: `node` is a node of the axis the mode walks (the base field's, the
/// row axis for the total modes), `other` a node of the other axis.
struct Pass<'a> {
    result: &'a PivotResult,
    data_field: usize,
    /// Whether the axis the mode walks is the row axis.
    on_rows: bool,
    shown: Shown,
    nodes: &'a mut usize,
}

impl Pass<'_> {
    fn axis(&self) -> &AxisTree {
        if self.on_rows {
            self.result.rows()
        } else {
            self.result.columns()
        }
    }

    fn other_len(&self) -> u32 {
        let other = if self.on_rows {
            self.result.columns()
        } else {
            self.result.rows()
        };
        u32::try_from(other.nodes().len()).unwrap_or(u32::MAX)
    }

    fn axis_len(&self) -> u32 {
        u32::try_from(self.axis().nodes().len()).unwrap_or(u32::MAX)
    }

    fn at(&self, node: u32, other: u32) -> (u32, u32) {
        if self.on_rows {
            (node, other)
        } else {
            (other, node)
        }
    }

    /// The plain value of (`node`, `other`).
    fn get(&self, node: u32, other: u32) -> Cell {
        let (row, column) = self.at(node, other);
        let function = *self.result.functions().get(self.data_field)?;
        self.result.plain_as(row, column, self.data_field, function)
    }

    /// Stores the shown value of (`node`, `other`), counting it as a node.
    fn put(&mut self, node: u32, other: u32, cell: Cell) -> Result<(), PivotComputeError> {
        if let Some(value) = cell {
            count_nodes(self.nodes, 1)?;
            let at = self.at(node, other);
            self.shown.insert(at, value.and_then(finite));
        }
        Ok(())
    }

    /// percentOfRow, percentOfCol, percentOfTotal and index, on the row axis.
    fn totals(&mut self, mode: ShowDataAs) -> Result<(), PivotComputeError> {
        let grand = self.get(0, 0);
        for row in 0..self.axis_len() {
            let row_total = self.get(row, 0);
            for column in 0..self.other_len() {
                let value = self.get(row, column);
                let shown = match mode {
                    ShowDataAs::PercentOfRow => ratio(value, row_total.clone()),
                    ShowDataAs::PercentOfColumn => ratio(value, self.get(0, column)),
                    ShowDataAs::PercentOfTotal => ratio(value, grand.clone()),
                    _ => index(value, row_total.clone(), self.get(0, column), grand.clone()),
                };
                self.put(row, column, shown)?;
            }
        }
        Ok(())
    }

    /// percentOfParentRow or percentOfParentCol: each node over its parent, the root over itself.
    fn parents(&mut self) -> Result<(), PivotComputeError> {
        for node in 0..self.axis_len() {
            let parent = self.parent(node).unwrap_or(node);
            for other in 0..self.other_len() {
                let shown = share(self.get(node, other), self.get(parent, other));
                self.put(node, other, shown)?;
            }
        }
        Ok(())
    }

    /// percentOfParent: each node at or below the base field's `depth` over its ancestor there.
    fn of_parent(&mut self, depth: u32) -> Result<(), PivotComputeError> {
        for node in 0..self.axis_len() {
            let mut anchor = node;
            while self.depth(anchor) > depth {
                match self.parent(anchor) {
                    Some(parent) => anchor = parent,
                    None => break,
                }
            }
            if self.depth(anchor) != depth {
                continue;
            }
            for other in 0..self.other_len() {
                let shown = share(self.get(node, other), self.get(anchor, other));
                self.put(node, other, shown)?;
            }
        }
        Ok(())
    }

    /// A base field on no axis: every cell is `#N/A` (UNVERIFIED, T-4750).
    fn no_base(&mut self) -> Result<(), PivotComputeError> {
        for node in 0..self.axis_len() {
            for other in 0..self.other_len() {
                self.put(node, other, Some(Err(Error::NA)))?;
            }
        }
        Ok(())
    }

    /// difference, percent and percentDiff against `item` of the base field at `depth`.
    fn compare(
        &mut self,
        mode: ShowDataAs,
        depth: u32,
        item: BaseItem,
        work: &mut u64,
    ) -> Result<(), PivotComputeError> {
        let bases = bases(self.axis(), depth, item, work)?;
        for (node, base) in (0u32..).zip(bases) {
            for other in 0..self.other_len() {
                let value = self.get(node, other);
                let shown = match base {
                    // An absent base node is empty even where the value is empty (t4161 QRN
                    // East/C); under difference too (UNVERIFIED, T-4750).
                    Base::Above | Base::Absent => None,
                    Base::Missing => Some(Err(Error::NA)),
                    Base::Own => match mode {
                        ShowDataAs::Percent => value.map(|v| v.and_then(|v| divide(v, v))),
                        _ => None,
                    },
                    Base::Node(base) => compared(mode, value, self.get(base, other)),
                };
                self.put(node, other, shown)?;
            }
        }
        Ok(())
    }

    /// runTotal, percentOfRunningTotal and the ranks over the runs of the base field at `depth`.
    fn runs(
        &mut self,
        mode: ShowDataAs,
        depth: u32,
        work: &mut u64,
    ) -> Result<(), PivotComputeError> {
        let runs = runs(self.axis(), depth, work)?;
        let mut values: Vec<Cell> = Vec::new();
        for run in &runs {
            for other in 0..self.other_len() {
                values.clear();
                values.extend(run.iter().map(|&node| self.get(node, other)));
                let shown = match mode {
                    ShowDataAs::RunTotal => running(&values).into_iter().map(Some).collect(),
                    ShowDataAs::PercentOfRunningTotal => {
                        charge(work, units(run.len()))?;
                        of_running_total(&values)
                    }
                    _ => {
                        charge(work, sort_units(run.len()))?;
                        ranks(&values, mode == ShowDataAs::RankDescending)
                    }
                };
                for (&node, cell) in run.iter().zip(shown) {
                    self.put(node, other, cell)?;
                }
            }
        }
        Ok(())
    }

    fn parent(&self, node: u32) -> Option<u32> {
        self.axis().nodes().get(node as usize)?.parent
    }

    fn depth(&self, node: u32) -> u32 {
        self.axis()
            .nodes()
            .get(node as usize)
            .map_or(0, |n| n.depth)
    }
}

/// A shown number as a cell holds it: -0 as 0 (no probe shows a -0; t4161 E PPC East/2023 is 0
/// over -1 and shows 0), and a result that is not finite `#NUM!` (both UNVERIFIED, T-4750).
fn finite(n: f64) -> Result<f64, Error> {
    if n.is_finite() {
        Ok(n + 0.0)
    } else {
        Err(Error::NUM)
    }
}

/// `n / d`, `#DIV/0!` for a `d` of 0.
fn divide(n: f64, d: f64) -> Result<f64, Error> {
    if d == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(n / d)
    }
}

/// A value that counts an empty cell as 0.
fn or_zero(cell: Cell) -> Result<f64, Error> {
    cell.unwrap_or(Ok(0.0))
}

/// `value` over the total `total`: empty when the total's line has no record (UNVERIFIED,
/// T-4750), `#DIV/0!` over 0.
fn ratio(value: Cell, total: Cell) -> Cell {
    let total = total?;
    Some(or_zero(value).and_then(|v| divide(v, total?)))
}

/// index: `value` × grand total / row total / column total, left to right (UNVERIFIED, T-4754).
fn index(value: Cell, row: Cell, column: Cell, grand: Cell) -> Cell {
    let (row, column, grand) = (row?, column?, grand?);
    Some((|| {
        let (v, row, column, grand) = (or_zero(value)?, row?, column?, grand?);
        if row == 0.0 || column == 0.0 {
            return Err(Error::DIV);
        }
        Ok(v * grand / row / column)
    })())
}

/// `value` over its parent's `parent`: empty when the parent is empty or 0.
fn share(value: Cell, parent: Cell) -> Cell {
    let parent = parent?;
    // The value's error first, as every mode reads its errors (UNVERIFIED, T-4750).
    let value = match or_zero(value) {
        Ok(v) => v,
        Err(error) => return Some(Err(error)),
    };
    let parent = match parent {
        Ok(p) => p,
        Err(error) => return Some(Err(error)),
    };
    if parent == 0.0 {
        return None;
    }
    Some(Ok(value / parent))
}

/// difference, percent or percentDiff of `value` against a base cell that is not its own.
fn compared(mode: ShowDataAs, value: Cell, base: Cell) -> Cell {
    if mode == ShowDataAs::Difference {
        return Some(or_zero(value).and_then(|v| Ok(v - or_zero(base)?)));
    }
    let value = match value {
        None => return Some(Err(Error::NULL)),
        Some(Err(error)) => return Some(Err(error)),
        Some(Ok(v)) => v,
    };
    let base = match base? {
        Ok(b) => b,
        Err(error) => return Some(Err(error)),
    };
    Some(if mode == ShowDataAs::Percent {
        divide(value, base)
    } else {
        divide(value - base, base)
    })
}

/// The running totals of a run's `values`, an empty one adding 0; past an error, that error.
fn running(values: &[Cell]) -> Vec<Result<f64, Error>> {
    let mut total: Result<f64, Error> = Ok(0.0);
    values
        .iter()
        .map(|value| {
            total = total.clone().and_then(|t| Ok(t + or_zero(value.clone())?));
            total.clone()
        })
        .collect()
}

/// percentOfRunningTotal of a run's `values`: each running total over the last.
fn of_running_total(values: &[Cell]) -> Vec<Cell> {
    let totals = running(values);
    let last = totals.last().cloned().unwrap_or(Ok(0.0));
    values
        .iter()
        .zip(totals)
        .map(|(value, so_far)| match (&last, value) {
            (Err(error), _) => Some(Err(error.clone())),
            (Ok(last), _) if *last != 0.0 => Some(so_far.map(|s| s / last)),
            (Ok(_), None) => None,
            (Ok(_), Some(Ok(v))) if *v == 0.0 => None,
            (Ok(_), Some(Ok(_))) => Some(Err(Error::DIV)),
            (Ok(_), Some(Err(error))) => Some(Err(error.clone())),
        })
        .collect()
}

/// The dense ranks of a run's present values, smallest first or, `descending`, largest first.
fn ranks(values: &[Cell], descending: bool) -> Vec<Cell> {
    // `+ 0.0` so that -0 and 0 tie.
    let mut distinct: Vec<f64> = values
        .iter()
        .filter_map(|v| v.clone().and_then(Result::ok))
        .map(|v| v + 0.0)
        .collect();
    distinct.sort_unstable_by(f64::total_cmp);
    distinct.dedup();
    let count = distinct.len();
    values
        .iter()
        .map(|value| {
            let value = value.clone()?;
            Some(value.map(|v| {
                let below = distinct.partition_point(|&d| d < v + 0.0);
                let rank = if descending { count - below } else { below + 1 };
                f64::from(u32::try_from(rank).unwrap_or(u32::MAX))
            }))
        })
        .collect()
}

/// Per node of `tree`, its [`Base`] against `item` of the field at `depth`.
fn bases(
    tree: &AxisTree,
    depth: u32,
    item: BaseItem,
    work: &mut u64,
) -> Result<Vec<Base>, PivotComputeError> {
    let nodes = tree.nodes();
    charge(work, units(nodes.len()).saturating_mul(3))?;
    let mut child: HashMap<(u32, u32), u32> = HashMap::new();
    let (mut previous, mut next) = (vec![None; nodes.len()], vec![None; nodes.len()]);
    for (index, node) in (0u32..).zip(nodes) {
        if let (true, Some(parent), Some(item)) = (node.depth >= depth, node.parent, node.item) {
            child.insert((parent, item), index);
        }
        if node.depth + 1 == depth {
            let mut last: Option<u32> = None;
            for sibling in tree.children(index) {
                if let Some(slot) = previous.get_mut(sibling as usize) {
                    *slot = last;
                }
                if let Some(slot) = last.and_then(|l| next.get_mut(l as usize)) {
                    *slot = Some(sibling);
                }
                last = Some(sibling);
            }
        }
    }
    let mut bases = vec![Base::Above; nodes.len()];
    for (index, node) in (0u32..).zip(nodes) {
        let (Some(parent), Some(own)) = (node.parent, node.item) else {
            continue;
        };
        let base = if node.depth == depth {
            let sibling = match item {
                BaseItem::Previous => previous.get(index as usize).copied().flatten(),
                BaseItem::Next => next.get(index as usize).copied().flatten(),
                BaseItem::Item(item) => child.get(&(parent, item)).copied(),
                BaseItem::None => None,
            };
            match (sibling, item) {
                (Some(base), _) if base != index => Base::Node(base),
                (None, BaseItem::Item(_) | BaseItem::None) => Base::Missing,
                _ => Base::Own,
            }
        } else if node.depth > depth {
            match bases.get(parent as usize).copied().unwrap_or(Base::Above) {
                Base::Node(above) => child
                    .get(&(above, own))
                    .map_or(Base::Absent, |&b| Base::Node(b)),
                other => other,
            }
        } else {
            Base::Above
        };
        if let Some(slot) = bases.get_mut(index as usize) {
            *slot = base;
        }
    }
    Ok(bases)
}

/// The runs of the field at `depth`: the nodes at or below it that share their parent of the
/// field and their items below it, each in display order.
fn runs(tree: &AxisTree, depth: u32, work: &mut u64) -> Result<Vec<Vec<u32>>, PivotComputeError> {
    let nodes = tree.nodes();
    charge(work, units(nodes.len()).saturating_mul(2))?;
    // A run of the field's own nodes by their parent; a deeper run by its parent's run and item.
    let mut top: HashMap<u32, usize> = HashMap::new();
    let mut deeper: HashMap<(usize, u32), usize> = HashMap::new();
    let mut run_of: Vec<Option<usize>> = vec![None; nodes.len()];
    let mut runs: Vec<Vec<u32>> = Vec::new();
    for (index, node) in (0u32..).zip(nodes) {
        let (Some(parent), Some(item)) = (node.parent, node.item) else {
            continue;
        };
        let next = runs.len();
        let run = if node.depth == depth {
            *top.entry(parent).or_insert(next)
        } else if node.depth > depth {
            let Some(above) = run_of.get(parent as usize).copied().flatten() else {
                continue;
            };
            *deeper.entry((above, item)).or_insert(next)
        } else {
            continue;
        };
        if run == next {
            runs.push(Vec::new());
        }
        if let Some(members) = runs.get_mut(run) {
            members.push(index);
        }
        if let Some(slot) = run_of.get_mut(index as usize) {
            *slot = Some(run);
        }
    }
    Ok(runs)
}

#[cfg(test)]
mod tests;
