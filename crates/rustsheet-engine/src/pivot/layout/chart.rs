//! The data a pivot chart plots, derived from a laid-out pivot (T-4186; plan.md §7, item 1).
//!
//! [`chart_data`] is a pure function of the definition, the cache, the [`PivotResult`], the
//! [`PivotLayout`] and the workbook's number formats. It reads the layout lines, so it plots
//! what the pivot shows:
//! - **Categories** are the plotted row lines, top to bottom, each labelled by its tuple, outer
//!   level first: nested row fields give multi-level categories, and Values on rows is one more
//!   level, labelled by the data fields' captions.
//! - **Series** are the plotted column lines, left to right: a column tuple times a data field
//!   when Values is on columns, a column tuple alone otherwise. A series with no label is named
//!   [`TOTAL_SERIES`].
//! - A line is plotted when it is an item's line ([`ItemType::Data`]) that holds values and has
//!   no line of its descendants after it: a leaf tuple or a collapsed item (`sd="0"`). So the
//!   grand-total lines are left out, and so are subtotal lines, an outline item's own line above
//!   its children (with or without `subtotalTop`) and `insertBlankRow` lines.
//! - **Values** are [`PivotResult::value`]: `None` where no record reaches.
//! - **Labels** are an item's text; a number or date item is its value in its pivot field's
//!   `numFmtId`, formatted here (`number_format::format_number`, en-US).
//!
//! The evidence and what is UNVERIFIED are in docs/excel-semantics/pivots.md "A `PivotChart` plots
//! the pivot's item lines". In short, two Excel-saved corpus charts show leaf categories, Values
//! on rows as the inner level, `Total`, a collapsed column item as one series, no point for an
//! empty cell, formatted number items and no grand totals; leaving out subtotal, `subtotalTop`,
//! outline-item and blank lines, show-as values and filtered or hidden items are the engine's
//! reading (UNVERIFIED, T-4938). Writing the chart part's caches is the export's (T-4187).
//!
//! The work is charged before it is done: one unit per line, per tuple level and per point; and
//! the labels' formatted text counts against [`MAX_PIVOT_TEXT_BYTES`], so the data is bounded by
//! the layout it reads (CLAUDE.md directive 5).

use crate::expressions::token::Error;
use crate::number_format::{format_number, get_num_fmt};
use crate::types::NumFmt;

use super::super::result::{charge, units, AxisTree, PivotComputeError, PivotResult};
use super::super::{CacheItem, ItemType, PivotCache, PivotTable};
use super::lines::{levels, AxisLine, Level, PivotLayout};
use super::render::{item_text, MAX_PIVOT_TEXT_BYTES};

/// The name Excel's chart cache gives the one series of a pivot with no column field and no
/// Values on columns (pivots.md "A `PivotChart` plots the pivot's item lines").
pub const TOTAL_SERIES: &str = "Total";

/// The locale number labels are formatted in: en-US, the parity target.
const LABEL_LOCALE: &str = "en";

/// What a pivot chart plots: its categories and its series ([`chart_data`]).
#[derive(Debug, Clone, PartialEq)]
pub struct PivotChartData {
    /// The categories, in the row lines' order.
    pub categories: Vec<ChartCategory>,
    /// The series, in the column lines' order; each has one value per category.
    pub series: Vec<ChartSeries>,
}

/// One category of a pivot chart: a plotted row line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartCategory {
    /// The labels of the line's tuple, outer level first: an item's text (a number in its
    /// field's number format), or a data field's caption at the Values level.
    pub labels: Vec<String>,
    /// The index of the row line in [`PivotLayout::rows`].
    pub line: u32,
}

/// One series of a pivot chart: a plotted column line.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartSeries {
    /// The labels of the line's tuple, outer level first, as [`ChartCategory::labels`].
    pub labels: Vec<String>,
    /// The index of the column line in [`PivotLayout::columns`].
    pub line: u32,
    /// Per category, the value of the cell where the category's row line meets this line:
    /// `None` where no record reaches, the aggregate's error as `Err`.
    pub values: Vec<Option<Result<f64, Error>>>,
}

impl ChartSeries {
    /// The series' name as Excel's chart cache writes it: [`TOTAL_SERIES`] with no label, the
    /// label with one. With several it is `None`: no Excel file shows it (UNVERIFIED, T-4938).
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self.labels.as_slice() {
            [] => Some(TOTAL_SERIES),
            [label] => Some(label),
            _ => None,
        }
    }
}

impl PivotChartData {
    /// The category labels as a `c:multiLvlStrCache` lists them: one level per label level,
    /// innermost first, each the `(point, label)`s it lists. A level lists a label at the first
    /// point of each run of categories that agree on every level down to it. A category with
    /// fewer levels than the deepest lists none at the levels it lacks (UNVERIFIED, T-4938).
    #[must_use]
    pub fn category_levels(&self) -> Vec<Vec<(u32, &str)>> {
        let depth = self.categories.iter().map(|c| c.labels.len()).max();
        let mut out: Vec<Vec<(u32, &str)>> = vec![Vec::new(); depth.unwrap_or(0)];
        let mut previous: &[String] = &[];
        for (point, category) in (0u32..).zip(&self.categories) {
            let labels = category.labels.as_slice();
            // The levels from the first that differs from the previous category start a run.
            let same = (labels.iter().zip(previous))
                .take_while(|(a, b)| a == b)
                .count();
            let first = if point == 0 { 0 } else { same };
            for (level, label) in labels.iter().enumerate().skip(first) {
                let slot = out
                    .len()
                    .checked_sub(level + 1)
                    .and_then(|i| out.get_mut(i));
                if let Some(slot) = slot {
                    slot.push((point, label.as_str()));
                }
            }
            previous = labels;
        }
        out
    }
}

/// One plotted line: its index, its result node, its data field and its labels.
struct PlottedLine {
    index: u32,
    node: u32,
    data_field: u32,
    labels: Vec<String>,
}

/// One axis, with what its labels are made from.
struct Plotted<'a> {
    tree: &'a AxisTree,
    levels: Vec<Level<'a>>,
    table: &'a PivotTable,
    cache: &'a PivotCache,
    num_fmts: &'a [NumFmt],
}

impl Plotted<'_> {
    /// Per tree depth (0 for the first axis field), the cache item of `node`'s path at it: one
    /// walk up the tree, whose length is the tuple's.
    fn path(&self, node: u32, work: &mut u64) -> Result<Vec<Option<u32>>, PivotComputeError> {
        let nodes = self.tree.nodes();
        let depth = nodes
            .get(node as usize)
            .ok_or(PivotComputeError::Malformed)?
            .depth;
        charge(work, u64::from(depth).saturating_add(1))?;
        let mut path = vec![None; depth as usize];
        for at in self.tree.ancestors(node) {
            let at = nodes.get(at as usize).ok_or(PivotComputeError::Malformed)?;
            if let Some(slot) = (at.depth.checked_sub(1)).and_then(|d| path.get_mut(d as usize)) {
                *slot = at.item;
            }
        }
        Ok(path)
    }

    /// The label at `level` of a line whose tuple's `x` there is `x` and whose path is `path`.
    fn label(&self, path: &[Option<u32>], level: usize, x: u32) -> Option<String> {
        match self.levels.get(level)? {
            Level::Values => Some(self.table.data_fields.get(x as usize)?.caption.clone()),
            Level::Field { depth, field } => {
                let item = (*path.get(*depth)?)?;
                let cache_field = self
                    .cache
                    .fields
                    .get(*self.tree.fields().get(*depth)? as usize)?;
                Some(match cache_field.items.get(item as usize)? {
                    CacheItem::Number(value) | CacheItem::Date(value) => {
                        let id = field.and_then(|f| f.number_format).unwrap_or(0);
                        let code = get_num_fmt(i32::try_from(id).unwrap_or(0), self.num_fmts);
                        format_number(*value, &code, LABEL_LOCALE).text
                    }
                    other => item_text(other),
                })
            }
        }
    }

    /// The plotted lines of `lines`, in order (see the module documentation).
    fn lines(
        &self,
        lines: &[AxisLine],
        work: &mut u64,
        text_left: &mut usize,
    ) -> Result<Vec<PlottedLine>, PivotComputeError> {
        let mut out = Vec::new();
        let mut tuple: Vec<u32> = Vec::new();
        for (index, line) in (0u32..).zip(lines) {
            charge(work, units(line.items.len()).saturating_add(1))?;
            if line.kind == ItemType::Grand {
                tuple.clear();
                continue;
            }
            tuple.truncate(line.repeat as usize);
            tuple.extend_from_slice(&line.items);
            // A line followed by one of its descendants (which repeats all of its tuple) is an
            // item's line above its children, not a point.
            let next = lines.get(index as usize + 1);
            let parent =
                next.is_some_and(|n| n.kind != ItemType::Grand && n.repeat as usize >= tuple.len());
            if line.kind != ItemType::Data || !line.values || parent {
                continue;
            }
            let path = self.path(line.node, work)?;
            let mut labels = Vec::with_capacity(tuple.len());
            for (level, &x) in tuple.iter().enumerate() {
                let label = self
                    .label(&path, level, x)
                    .ok_or(PivotComputeError::Malformed)?;
                *text_left =
                    (text_left.checked_sub(label.len())).ok_or(PivotComputeError::Limit)?;
                labels.push(label);
            }
            out.push(PlottedLine {
                index,
                node: line.node,
                data_field: line.data_field,
                labels,
            });
        }
        Ok(out)
    }
}

/// What a pivot chart bound to `table`'s pivot over `cache` plots, from its `result`
/// ([`compute`](super::super::compute)) and `layout` ([`lay_out`](super::lay_out)), its number
/// labels formatted by the workbook's `num_fmts` (see the module documentation), spending at
/// most `work` units.
///
/// # Errors
/// [`PivotComputeError::Malformed`] when `result` or `layout` is not over `table` and `cache`,
/// and [`PivotComputeError::Limit`] past `work` or when the labels pass
/// [`MAX_PIVOT_TEXT_BYTES`].
pub fn chart_data(
    table: &PivotTable,
    cache: &PivotCache,
    result: &PivotResult,
    layout: &PivotLayout,
    num_fmts: &[NumFmt],
    work: &mut u64,
) -> Result<PivotChartData, PivotComputeError> {
    let axis = |tree, fields| Plotted {
        tree,
        levels: levels(table, fields),
        table,
        cache,
        num_fmts,
    };
    let rows = axis(result.rows(), &table.row_fields);
    let columns = axis(result.columns(), &table.column_fields);
    let mut text_left = MAX_PIVOT_TEXT_BYTES;
    let mut row_lines = rows.lines(&layout.rows, work, &mut text_left)?;
    let column_lines = columns.lines(&layout.columns, work, &mut text_left)?;
    let has_values =
        |plotted: &Plotted<'_>| (plotted.levels.iter()).any(|l| matches!(l, Level::Values));
    let (values_on_rows, values_on_columns) = (has_values(&rows), has_values(&columns));
    let mut series = Vec::new();
    if !table.data_fields.is_empty() {
        for column in column_lines {
            charge(work, units(row_lines.len()).saturating_add(1))?;
            let values = (row_lines.iter())
                .map(|row| {
                    let data = if values_on_rows {
                        row.data_field
                    } else if values_on_columns {
                        column.data_field
                    } else {
                        0
                    };
                    result.value(row.node, column.node, data as usize)
                })
                .collect();
            series.push(ChartSeries {
                labels: column.labels,
                line: column.index,
                values,
            });
        }
    }
    let categories = (row_lines.iter_mut())
        .map(|row| ChartCategory {
            labels: std::mem::take(&mut row.labels),
            line: row.index,
        })
        .collect();
    Ok(PivotChartData { categories, series })
}

#[cfg(test)]
mod tests;
