//! The layout lines of a pivot: Excel's `rowItems`/`colItems` (`<i t r i><x v/></i>`) and the
//! `<location>` they give, for compact, outline and tabular form (SPEC-0032 stage 7, T-4147).
//!
//! Each axis is the result's [`AxisTree`] walked in display order, with the Values field (`x="-2"`
//! of `rowFields`/`colFields`) as one more level whose items are the data fields. A line's tuple is
//! one `x` per level down to its own: a pivot item's position in the field's `<items>`, or a data
//! field's index for the Values level; `r` counts the leading `x`s it shares with the line before
//! (never its own last one), and `x` lists the rest. Evidence: every probe's sidecar `row_items` and
//! `col_items` (docs/excel-semantics/pivots.md "Layout lines"):
//! - every tuple of one item per level is a `t="data"` line (probes/t4123-pivot-capture,
//!   t4138-pivot-geometry), its `x` the item's position in the field's `<items>`, not its cache
//!   index (the Excel-saved pivot of crates/rustsheet-xlsx/tests/example.xlsx lists items out of
//!   cache order);
//! - on rows, an item of a field in outline form (`pivotField@outline`, the default; compact form
//!   is outline form with `compact`) has a line of its own above its children, which carries its
//!   default subtotal when `subtotalTop` is on (t4138 G2, G3, G6; t4583 R1, where the outer field
//!   left at the default is outline form inside a tabular pivot; t4609 L3, where `compact="0"`
//!   alone keeps it). An item of a tabular field (`outline="0"`, whatever its `compact`) shares its
//!   first child's line, and its subtotal lines follow its children (t4138 G4, t4123 T, t4583 R2 to
//!   R6, t4609 L4);
//! - on columns, every item shares its first child's line and its subtotal lines follow its
//!   children (t4138 G5);
//! - a field has subtotal lines only when a field other than Values is inside it: a field whose
//!   only inner level is Values has none (t4138 G6, t4135 `A_sum`). The lines are one `default`
//!   line, or one line per custom subtotal function in the field's order (t4137 K5 `sum`, `count`;
//!   t4168 F1 all eleven), after the children in compact form too (t4168 F2; plain outline form,
//!   `compact` off, is the engine's reading: UNVERIFIED, T-4699);
//! - the grand total is one `t="grand"` line `<x/>` per data field when Values is on the axis
//!   (`i` the data field), one otherwise, and none on an axis with no field but Values (t4138 G6,
//!   G9; t4135 `A_sum`, `EC`). `rowGrandTotals` gives the row axis's line and `colGrandTotals` the
//!   column axis's (only the "on" case is probed: UNVERIFIED, T-4649);
//! - an axis with no level at all has one empty line `<i/>` (t4138 G1, G7, G9; t4137 K4);
//! - a row field with `insertBlankRow` has a `t="blank"` line after each item, below its
//!   subtotals, with the item's tuple and no values (probes/t4168-pivot-layout-options BR
//!   tabular, BR2 compact);
//! - a collapsed item (`item@sd="0"`) has one line, its own tuple, with its values and no
//!   children or subtotal lines, in compact and tabular form (t4168 SD, SD2). On columns, and on
//!   a field with Values inside it (one line, no data field chosen, so the first data field's
//!   values), it is the engine's reading (UNVERIFIED, T-4699).
//!
//! The engine's own readings where no probe shows Excel's (pivots.md "Layout lines", UNVERIFIED):
//! subtotals at the bottom (`subtotalTop="0"`), `defaultSubtotal="0"` with no custom subtotal,
//! and grand totals off (T-4649); the Values field outer or in the middle of an axis, and its own
//! line with `outlineData` (T-4650). Filters, sorting, groups and calculated items reshape the
//! trees first (T-4165, T-4171, T-4175).
//!
//! The geometry (pivots.md "Pivot geometry: location and header offsets per layout"):
//! `firstHeaderRow` 1, `firstDataRow` 1 plus the column levels, `firstDataCol` the row label
//! columns (a row field in compact form, `compact` and `outline` both on, shares its column with
//! the next level: t4583 R1; either off gives the next level its own, t4609 L3 and L4; Values with
//! `compactData` and `outlineData`), 1 with no row level but a column field, 0 with neither; the
//! location is the lines plus those offsets from the anchor. Classic layout (`gridDropZones`)
//! with no column level adds one row and puts `firstHeaderRow` on the first data row (t4129 GZ,
//! probes/t4745-pivot-classic-subtotal-caption G1; compact and outline form are the engine's
//! reading, UNVERIFIED, T-4785). Page fields fill `pageWrap` per
//! column (`rowPageCount` rows), or per row with `pageOverThenDown` (t4137 K4, t4138 G8; the other
//! orders are T-4650's).
//!
//! Every line is charged to the work budget as it is made, and a layout past the sheet or past
//! [`MAX_PIVOT_CELLS`] cells has no result (pivots.md "A pivot result is bounded in work and
//! nodes", DEVIATION). The walk keeps its own stack, so an untrusted axis depth never recurses.
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::struct_field_names)]
// LINT-DEBT-END

use crate::constants::{LAST_COLUMN, LAST_ROW};

use super::super::result::{charge, units, AxisTree, PivotComputeError, PivotResult};
use super::super::{
    AxisField, CellArea, ItemType, PivotCache, PivotField, PivotGeometry, PivotOptions, PivotTable,
};

/// Most cells a pivot's layout covers: its location's rows times columns, plus two per page
/// field (SPEC-0032 "Untrusted-input limits"). Checked as the lines are made, before any cell is.
pub const MAX_PIVOT_CELLS: usize = 4_000_000;

/// One line of `rowItems` or `colItems`: an `<i>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisLine {
    /// `t`: an item's line ([`ItemType::Data`]), a subtotal ([`ItemType::Default`] or
    /// [`ItemType::Function`]) or the grand total ([`ItemType::Grand`]).
    pub kind: ItemType,
    /// `r`: how many leading `x`s of the line before it repeats.
    pub repeat: u32,
    /// `i`: the data field of the line (0 unless Values is on its axis).
    pub data_field: u32,
    /// The `<x v>`s after the repeated ones: pivot item positions in a field's `<items>`, a data
    /// field index at the Values level; `[0]` for a grand total, empty for an axis's only `<i/>`.
    pub items: Vec<u32>,
    /// The result node the line's cells read ([`AxisTree`]; the root for a grand total).
    pub node: u32,
    /// Whether the line's cells hold values: false for an item's own line whose subtotal is
    /// elsewhere or off.
    pub values: bool,
}

/// A pivot's layout lines and the location they give (SPEC-0032 stage 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PivotLayout {
    /// `rowItems`, top to bottom.
    pub rows: Vec<AxisLine>,
    /// `colItems`, left to right.
    pub columns: Vec<AxisLine>,
    /// `<location ref>`: from the definition's top-left cell, as many rows and columns as the
    /// offsets and lines take.
    pub location: CellArea,
    /// The rest of `<location>`.
    pub geometry: PivotGeometry,
}

/// One level of an axis: a field of the result's tree, or the Values field.
#[derive(Debug, Clone, Copy)]
pub(super) enum Level<'a> {
    Values,
    Field {
        /// The field's index among the tree's fields.
        depth: usize,
        /// Its definition, when the pivot has one for it.
        field: Option<&'a PivotField>,
    },
}

/// The levels of `axis` (whose fields are the result tree's, in order), the Values field left out
/// when there is no data field.
pub(super) fn levels<'a>(table: &'a PivotTable, axis: &[AxisField]) -> Vec<Level<'a>> {
    let mut depth = 0;
    axis.iter()
        .filter_map(|level| match level {
            AxisField::Values => (!table.data_fields.is_empty()).then_some(Level::Values),
            AxisField::Field(index) => {
                let level = Level::Field {
                    depth,
                    field: table.fields.get(*index as usize),
                };
                depth += 1;
                Some(level)
            }
        })
        .collect()
}

/// Per cache item of a field, its position in the field's `<items>`: the first data item naming
/// it. A cache item the field does not list (Excel's files list every one) takes the positions
/// after the listed items, in cache order, the order the result shows them in.
fn positions(field: Option<&PivotField>, item_count: usize) -> Vec<u32> {
    let listed = field.map_or(&[][..], |f| f.items.as_slice());
    let mut at = vec![u32::MAX; item_count];
    for (position, item) in (0u32..).zip(listed) {
        let named = item.index.filter(|_| item.kind == ItemType::Data);
        if let Some(slot) = named.and_then(|x| at.get_mut(x as usize)) {
            if *slot == u32::MAX {
                *slot = position;
            }
        }
    }
    let mut next = u32::try_from(listed.len()).unwrap_or(u32::MAX);
    for slot in at.iter_mut().filter(|slot| **slot == u32::MAX) {
        *slot = next;
        next = next.saturating_add(1);
    }
    at
}

/// The step of the walk over an axis.
enum Step {
    /// Emit a node's lines and queue its children.
    Enter(Visit),
    /// Emit a node's subtotal (or, at the root, grand-total) lines after its children.
    Close(Visit),
}

/// A node of the walk: a result node at `level` levels deep, with the data field the Values
/// level above it chose and its tuple of `x`s.
struct Visit {
    node: u32,
    level: usize,
    data: Option<u32>,
    tuple: Vec<u32>,
}

/// One axis's lines, as the walk makes them.
struct Lines<'w> {
    lines: Vec<AxisLine>,
    previous: Vec<u32>,
    /// Most lines the axis may have before the layout passes [`MAX_PIVOT_CELLS`].
    cap: usize,
    work: &'w mut u64,
}

impl Lines<'_> {
    fn push(
        &mut self,
        kind: ItemType,
        visit: &Visit,
        data: Option<u32>,
        values: bool,
    ) -> Result<(), PivotComputeError> {
        charge(self.work, units(visit.tuple.len()).saturating_add(1))?;
        if self.lines.len() >= self.cap {
            return Err(PivotComputeError::Limit);
        }
        let (tuple, grand) = (&visit.tuple, kind == ItemType::Grand);
        let shared = self
            .previous
            .iter()
            .zip(tuple)
            .take_while(|(a, b)| a == b)
            .count();
        let repeat = if grand {
            0
        } else {
            shared.min(tuple.len().saturating_sub(1))
        };
        self.lines.push(AxisLine {
            kind,
            repeat: u32::try_from(repeat).map_err(|_| PivotComputeError::Limit)?,
            data_field: data.unwrap_or(0),
            items: if grand {
                vec![0]
            } else {
                tuple.get(repeat..).unwrap_or_default().to_vec()
            },
            node: visit.node,
            values,
        });
        self.previous.clone_from(tuple);
        Ok(())
    }
}

/// One axis of a pivot, with what its lines depend on.
struct AxisPlan<'a> {
    tree: &'a AxisTree,
    levels: Vec<Level<'a>>,
    /// Per tree field, its [`positions`].
    positions: Vec<Vec<u32>>,
    data_fields: u32,
    rows: bool,
    grand: bool,
}

impl<'a> AxisPlan<'a> {
    /// The row axis (`rows`) or the column axis of `table`, whose result tree is `tree`.
    fn new(
        table: &'a PivotTable,
        cache: &PivotCache,
        tree: &'a AxisTree,
        rows: bool,
        work: &mut u64,
    ) -> Result<Self, PivotComputeError> {
        let (axis, grand) = if rows {
            (&table.row_fields, table.options.row_grand_totals)
        } else {
            (&table.column_fields, table.options.column_grand_totals)
        };
        let mut at = Vec::with_capacity(tree.fields().len());
        for &field in tree.fields() {
            let item_count = cache
                .fields
                .get(field as usize)
                .ok_or(PivotComputeError::Malformed)?
                .items
                .len();
            let pivot_field = table.fields.get(field as usize);
            let listed = pivot_field.map_or(0, |f| f.items.len());
            charge(work, units(item_count.saturating_add(listed)))?;
            at.push(positions(pivot_field, item_count));
        }
        Ok(AxisPlan {
            tree,
            levels: levels(table, axis),
            positions: at,
            data_fields: u32::try_from(table.data_fields.len())
                .map_err(|_| PivotComputeError::Limit)?,
            rows,
            grand,
        })
    }

    /// Whether an item at `level` has a line of its own above its children: a row field in
    /// outline form (compact form included), or the Values field with `outlineData`.
    fn own_line(&self, level: usize, options: &PivotOptions) -> bool {
        self.rows
            && match self.levels.get(level) {
                Some(Level::Field { field, .. }) => field.is_none_or(|f| f.outline),
                Some(Level::Values) => options.outline_data,
                None => false,
            }
    }

    /// Whether the levels inside `level` hold a field, so that `level`'s items have subtotals.
    fn has_subtotals(&self, level: usize) -> bool {
        matches!(self.levels.get(level), Some(Level::Field { .. }))
            && self
                .levels
                .iter()
                .skip(level + 1)
                .any(|l| matches!(l, Level::Field { .. }))
    }

    fn values_inside(&self, level: usize) -> bool {
        self.levels
            .iter()
            .skip(level + 1)
            .any(|l| matches!(l, Level::Values))
    }

    /// The field's subtotal lines: `default`, or its custom functions when it has no default
    /// subtotal (a field with both is read as the default alone: UNVERIFIED, T-4649).
    fn subtotals(&self, level: usize) -> Vec<ItemType> {
        match self.levels.get(level) {
            Some(Level::Field { field, .. }) if self.has_subtotals(level) => match field {
                Some(f) if !f.default_subtotal => f
                    .custom_subtotals
                    .iter()
                    .map(|a| ItemType::Function(*a))
                    .collect(),
                _ => vec![ItemType::Default],
            },
            _ => Vec::new(),
        }
    }

    /// Whether `level`'s default subtotal sits on its items' own lines (`subtotalTop`).
    fn subtotal_on_top(&self, level: usize, options: &PivotOptions) -> bool {
        let top = match self.levels.get(level) {
            Some(Level::Field { field, .. }) => field.is_none_or(|f| f.subtotal_top),
            _ => false,
        };
        top && self.own_line(level, options)
            && !self.values_inside(level)
            && self.subtotals(level) == [ItemType::Default]
    }

    /// The axis's lines in display order, at most `cap` of them.
    fn lines(
        &self,
        options: &PivotOptions,
        cap: usize,
        work: &mut u64,
    ) -> Result<Vec<AxisLine>, PivotComputeError> {
        let mut out = Lines {
            lines: Vec::new(),
            previous: Vec::new(),
            cap,
            work,
        };
        let mut stack = vec![Step::Enter(Visit {
            node: 0,
            level: 0,
            data: None,
            tuple: Vec::new(),
        })];
        while let Some(step) = stack.pop() {
            match step {
                Step::Enter(visit) => {
                    let level = visit.level;
                    let Some(&inner) = self.levels.get(level) else {
                        out.push(ItemType::Data, &visit, visit.data, true)?;
                        continue;
                    };
                    if let Some(own) = level.checked_sub(1) {
                        if self.collapsed(own, &visit) {
                            out.push(ItemType::Data, &visit, visit.data, true)?;
                            self.blank_line(own, &visit, &mut out)?;
                            continue;
                        }
                        if self.own_line(own, options) {
                            let values = self.subtotal_on_top(own, options);
                            out.push(ItemType::Data, &visit, visit.data, values)?;
                        }
                    }
                    let children = self.children(&visit, inner, out.work)?;
                    stack.push(Step::Close(visit));
                    stack.extend(children.into_iter().rev().map(Step::Enter));
                }
                Step::Close(visit) => self.close(&visit, options, &mut out)?,
            }
        }
        Ok(out.lines)
    }

    /// The children of `visit` at its inner level, in display order.
    fn children(
        &self,
        visit: &Visit,
        inner: Level<'_>,
        work: &mut u64,
    ) -> Result<Vec<Visit>, PivotComputeError> {
        let child = |node, data, x| {
            let mut tuple = Vec::with_capacity(visit.tuple.len() + 1);
            tuple.extend_from_slice(&visit.tuple);
            tuple.push(x);
            Visit {
                node,
                level: visit.level + 1,
                data,
                tuple,
            }
        };
        // Charged before any child is made: the tree counts them without allocating.
        let count = match inner {
            Level::Values => self.data_fields as usize,
            Level::Field { .. } => self.tree.children(visit.node).count(),
        };
        charge(work, units(count.saturating_mul(visit.tuple.len() + 1)))?;
        let children: Vec<Visit> = match inner {
            Level::Values => (0..self.data_fields)
                .map(|d| child(visit.node, Some(d), d))
                .collect(),
            Level::Field { depth, .. } => {
                let at = self.positions.get(depth);
                self.tree
                    .children(visit.node)
                    .map(|node| {
                        let item = self.tree.nodes().get(node as usize).and_then(|n| n.item);
                        let x = item.and_then(|i| at?.get(i as usize).copied());
                        x.map(|x| child(node, visit.data, x))
                            .ok_or(PivotComputeError::Malformed)
                    })
                    .collect::<Result<_, _>>()?
            }
        };
        Ok(children)
    }

    /// The lines after `visit`'s children: its subtotals, or the grand total at the root.
    fn close(
        &self,
        visit: &Visit,
        options: &PivotOptions,
        out: &mut Lines<'_>,
    ) -> Result<(), PivotComputeError> {
        let values_on_axis = self.levels.iter().any(|l| matches!(l, Level::Values));
        let per_data = |values: bool| -> Vec<Option<u32>> {
            if values {
                (0..self.data_fields).map(Some).collect()
            } else {
                vec![visit.data]
            }
        };
        let Some(own) = visit.level.checked_sub(1) else {
            if self.grand && !self.tree.fields().is_empty() {
                for data in per_data(values_on_axis) {
                    out.push(ItemType::Grand, visit, data, true)?;
                }
            }
            return Ok(());
        };
        if !self.subtotal_on_top(own, options) {
            for kind in self.subtotals(own) {
                for data in per_data(self.values_inside(own)) {
                    out.push(kind, visit, data, true)?;
                }
            }
        }
        self.blank_line(own, visit, out)
    }

    /// The `t="blank"` line after an item of a row field with `insertBlankRow`, below its
    /// subtotals, the item's own tuple and no values (probes/t4168-pivot-layout-options BR,
    /// BR2). Only an outer field is probed: on the innermost row field, whose items have no
    /// children, the engine adds none (UNVERIFIED, T-4699).
    fn blank_line(
        &self,
        level: usize,
        visit: &Visit,
        out: &mut Lines<'_>,
    ) -> Result<(), PivotComputeError> {
        let blank = match self.levels.get(level) {
            Some(Level::Field { field: Some(f), .. }) => f.insert_blank_row,
            _ => false,
        };
        if blank && self.rows && self.has_subtotals(level) {
            out.push(ItemType::Blank, visit, visit.data, false)?;
        }
        Ok(())
    }

    /// Whether `visit`'s item, at field level `level`, is collapsed (`item@sd="0"`): its line
    /// stands alone, with the item's values and none of its children or subtotal lines
    /// (probes/t4168-pivot-layout-options SD compact, SD2 tabular).
    fn collapsed(&self, level: usize, visit: &Visit) -> bool {
        let Some(Level::Field {
            depth,
            field: Some(field),
        }) = self.levels.get(level)
        else {
            return false;
        };
        let item = self
            .tree
            .nodes()
            .get(visit.node as usize)
            .and_then(|n| n.item);
        let position = item.and_then(|i| self.positions.get(*depth)?.get(i as usize));
        position
            .and_then(|&p| field.items.get(p as usize))
            .is_some_and(|i| i.kind == ItemType::Data && !i.expanded)
    }
}

/// The layout of `table`'s `result` over `cache` (from [`compute`](super::super::compute) of the
/// same two), spending at most `work` units (see the module documentation). It is a separate
/// call, not part of `compute`, so that a pivot too large to lay out still has its values for
/// GETPIVOTDATA.
///
/// # Errors
/// [`PivotComputeError::Limit`] past `work` (`work` is then 0) or [`MAX_PIVOT_CELLS`],
/// [`PivotComputeError::PastSheet`] past the sheet's last row or column, and
/// [`PivotComputeError::Malformed`] when `result` is not over `cache`.
pub fn lay_out(
    table: &PivotTable,
    cache: &PivotCache,
    result: &PivotResult,
    work: &mut u64,
) -> Result<PivotLayout, PivotComputeError> {
    let (rows, columns) = (result.rows(), result.columns());
    let options = &table.options;
    let row_plan = AxisPlan::new(table, cache, rows, true, work)?;
    let column_plan = AxisPlan::new(table, cache, columns, false, work)?;
    let geometry = geometry(table, &row_plan, &column_plan);
    // The rows stop once they alone pass the cap at the narrowest width, and the columns once
    // they pass it at the rows' height, so the cap fails before both axes are built.
    let cells = MAX_PIVOT_CELLS.saturating_sub(table.page_fields.len().saturating_mul(2));
    let cap = |offset: u32, span: usize| (cells / span.max(1)).saturating_sub(offset as usize);
    let row_cap = cap(geometry.first_data_row, geometry.first_data_column as usize);
    let row_lines = row_plan.lines(options, row_cap.saturating_add(1), work)?;
    let height = (geometry.first_data_row as usize).saturating_add(row_lines.len());
    let column_cap = cap(geometry.first_data_column, height);
    let column_lines = column_plan.lines(options, column_cap.saturating_add(1), work)?;
    let location = location(table, &geometry, [row_lines.len(), column_lines.len()])?;
    Ok(PivotLayout {
        rows: row_lines,
        columns: column_lines,
        location,
        geometry,
    })
}

/// The `<location>` offsets and page counts of a layout over `rows` and `columns`.
fn geometry(table: &PivotTable, rows: &AxisPlan<'_>, columns: &AxisPlan<'_>) -> PivotGeometry {
    let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let options = &table.options;
    let label_columns = match label_columns(&rows.levels, options).last() {
        Some(&last) => last as usize + 1,
        None => usize::from(!columns.tree.fields().is_empty()),
    };
    let pages = table.page_fields.len();
    let wrap = match options.page_wrap as usize {
        0 => pages,
        wrap => wrap.min(pages),
    };
    let across = if wrap == 0 { 0 } else { pages.div_ceil(wrap) };
    let (row_pages, column_pages) = if options.page_over_then_down {
        (across, wrap)
    } else {
        (wrap, across)
    };
    // Classic layout (`gridDropZones`) keeps a column-header row with no column level, and its
    // `firstHeaderRow` is then the first data row (t4129 GZ, t4745 G1: 2 and 2); with a column
    // level its geometry is the plain one (t4745 GC, GV).
    let classic = options.grid_drop_zones && columns.levels.is_empty();
    let first_data_row = count(columns.levels.len())
        .saturating_add(1)
        .saturating_add(u32::from(classic));
    PivotGeometry {
        first_header_row: if classic { first_data_row } else { 1 },
        first_data_row,
        first_data_column: count(label_columns),
        row_page_count: count(row_pages),
        column_page_count: count(column_pages),
    }
}

/// Per row level, its label column counted from the location's left: a row level in compact
/// form (`compact` and `outline` both on; the Values field with `compactData` and
/// `outlineData`) shares its column with the next level (probes/t4583-pivot-row-labels R1), and
/// any other level puts the next one in a column of its own: `compact="0"` alone or `outline="0"`
/// alone is enough (probes/t4609-pivot-row-labels-2 L3, L4).
pub(super) fn label_columns(levels: &[Level<'_>], options: &PivotOptions) -> Vec<u32> {
    let shares_column = |level: &Level<'_>| match level {
        Level::Field { field, .. } => field.is_none_or(|f| f.compact && f.outline),
        Level::Values => options.compact_data && options.outline_data,
    };
    let mut column = 0u32;
    levels
        .iter()
        .map(|level| {
            let at = column;
            if !shares_column(level) {
                column = column.saturating_add(1);
            }
            at
        })
        .collect()
}

/// The location of `geometry` with `lines` row and column lines from the definition's anchor;
/// within the sheet and [`MAX_PIVOT_CELLS`].
fn location(
    table: &PivotTable,
    geometry: &PivotGeometry,
    [row_lines, column_lines]: [usize; 2],
) -> Result<CellArea, PivotComputeError> {
    let extent = |offset: u32, lines: usize| {
        u32::try_from(lines)
            .ok()
            .and_then(|lines| lines.checked_add(offset))
            .ok_or(PivotComputeError::Limit)
    };
    let height = extent(geometry.first_data_row, row_lines)?.max(1);
    let width = extent(geometry.first_data_column, column_lines)?.max(1);
    let cells = (height as usize)
        .saturating_mul(width as usize)
        .saturating_add(table.page_fields.len().saturating_mul(2));
    if cells > MAX_PIVOT_CELLS {
        return Err(PivotComputeError::Limit);
    }
    let (top, left) = (table.location.top, table.location.left);
    let bottom = top.checked_add(height - 1);
    let right = left.checked_add(width - 1);
    match (bottom, right) {
        (Some(bottom), Some(right))
            if i64::from(bottom) < i64::from(LAST_ROW)
                && i64::from(right) < i64::from(LAST_COLUMN) =>
        {
            Ok(CellArea {
                top,
                left,
                bottom,
                right,
            })
        }
        _ => Err(PivotComputeError::PastSheet),
    }
}

#[cfg(test)]
mod tests;
