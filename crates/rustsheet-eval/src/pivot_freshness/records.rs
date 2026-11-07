//! The freshness test of T-4906: do a pivot's saved cache records reproduce the values Excel
//! saved for it?
//!
//! Each value cell of Excel's own saved layout is found from the definition's `<rowItems>` and
//! `<colItems>` (one line per body row and per data column, at `firstDataRow` and `firstDataCol`
//! inside `<location>`), and its value is read from the engine's result tree over the cache
//! records (`rustsheet_engine::pivot::compute_on`: the record filter, groups and aggregation,
//! never the layout or the renderer the render score tests). The pivot is fresh when every one
//! equals Excel's saved cell. Labels and headers are left out: they are what the renderer must
//! reproduce, so they cannot also decide whether it is scored.
//!
//! A line names its items as `<x v>` indices into its pivot field's `<items>`, after `r` items
//! repeated from the line above; a `t` line is a subtotal (`default` or a custom function), the
//! grand total or a blank line. An item line short of the axis's last field (a compact or outline
//! parent row) shows its subtotal only when its field's flags put the automatic subtotal there
//! (`defaultSubtotal` and `subtotalTop` on, no custom subtotal, the Values field not below it on the axis), so
//! only where they do not is a cell Excel saved empty left uncompared.

use std::collections::HashMap;

use rustsheet_engine::pivot::{
    compute_on, Aggregate, AxisField, AxisTree, ItemType, PivotCache, PivotResult, PivotTable,
    MAX_PIVOT_WORK,
};

use super::SavedPivot;
use crate::cellref::Addr;
use crate::value::{compare, Value};
use crate::xlsx_snapshot::PivotLine;

/// What the records test found for one pivot: cells compared, those differing, and the first
/// difference.
#[derive(Debug, Default, PartialEq)]
pub(super) struct RecordCheck {
    pub(super) compared: usize,
    pub(super) differing: usize,
    pub(super) first: Option<String>,
}

/// How one saved line reads a value from the result tree.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Read {
    /// An item line: its node (`None` when no record reaches its items), whether it is the
    /// axis's last field (a leaf), and for a parent line whether its field's flags put its
    /// subtotal on it.
    Items {
        node: Option<u32>,
        leaf: bool,
        total_on_line: bool,
    },
    /// A subtotal line of the node, by the data field's own function or by a custom one.
    Subtotal {
        node: Option<u32>,
        function: Option<Aggregate>,
    },
    /// A blank line, which holds no value.
    Blank,
}

/// One decoded line: how it reads, and the data field it names when the axis holds the Values
/// field.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Line {
    read: Read,
    data_field: Option<usize>,
}

/// The `t` of a line of `kind`: `None` for an item's line.
pub(super) fn line_type(kind: ItemType) -> Option<&'static str> {
    Some(match kind {
        ItemType::Data => return None,
        ItemType::Default => "default",
        ItemType::Grand => "grand",
        ItemType::Blank => "blank",
        ItemType::Function(f) => f.item_type_name(),
    })
}

/// An axis tree's children by (parent node, cache item), so each line is found in one lookup
/// per field rather than by scanning siblings (directive 5: a saved area holds up to 100,000
/// lines).
fn child_map(tree: &AxisTree) -> HashMap<(u32, u32), u32> {
    let mut map = HashMap::new();
    for (index, node) in (0u32..).zip(tree.nodes()) {
        if let (Some(parent), Some(item)) = (node.parent, node.item) {
            map.entry((parent, item)).or_insert(index);
        }
    }
    map
}

/// Decodes the saved `lines` of the axis `fields` against `tree`.
///
/// # Errors
/// When a line repeats more items than the line above holds, lists more than the axis's fields,
/// names an item its pivot field does not have, or has a `t` that is not `ST_ItemType`'s.
fn decode(
    table: &PivotTable,
    fields: &[AxisField],
    tree: &AxisTree,
    lines: &[PivotLine],
) -> Result<Vec<Line>, String> {
    let children = child_map(tree);
    let leaf_depth = fields.iter().filter(|f| **f != AxisField::Values).count();
    let values_at = fields.iter().position(|f| *f == AxisField::Values);
    let mut previous: Vec<u32> = Vec::new();
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        let repeated = previous
            .get(..line.r as usize)
            .ok_or_else(|| format!("a line repeats {} items of {}", line.r, previous.len()))?;
        let mut tuple = repeated.to_vec();
        tuple.extend_from_slice(&line.x);
        if tuple.len() > fields.len().max(1) {
            return Err(format!("a line lists {} items", tuple.len()));
        }
        let kind = line.t.as_deref().unwrap_or("data");
        let line_field = usize::try_from(line.i).unwrap_or(usize::MAX);
        // The Values field's place in a line holds a data field's index, not an item.
        let data_field = values_at.map(|at| match (kind, tuple.get(at)) {
            ("grand" | "blank", _) | (_, None) => line_field,
            (_, Some(&v)) => v as usize,
        });
        let mut node = Some(0u32);
        let mut depth = 0;
        let mut last_field = None;
        if kind != "grand" && kind != "blank" {
            for (&field, &x) in fields.iter().zip(&tuple) {
                let AxisField::Field(f) = field else { continue };
                let item = (table.fields.get(f as usize))
                    .and_then(|pf| pf.items.get(x as usize))
                    .filter(|item| item.kind == ItemType::Data)
                    .and_then(|item| item.index)
                    .ok_or_else(|| format!("item {x} of field {f} is not a data item"))?;
                node = node.and_then(|n| children.get(&(n, item)).copied());
                depth += 1;
                last_field = table.fields.get(f as usize);
            }
        }
        let read = match kind {
            "data" => Read::Items {
                node,
                leaf: depth == leaf_depth,
                // A Values field inside the line spreads its data fields over the lines below.
                total_on_line: last_field.is_some_and(|pf| {
                    pf.default_subtotal && pf.subtotal_top && pf.custom_subtotals.is_empty()
                }) && values_at.is_none_or(|at| at < tuple.len()),
            },
            "default" => Read::Subtotal {
                node,
                function: None,
            },
            "grand" => Read::Subtotal {
                node: Some(0),
                function: None,
            },
            "blank" => Read::Blank,
            other => Read::Subtotal {
                node,
                function: Some(
                    Aggregate::from_item_type_name(other)
                        .ok_or_else(|| format!("line type {other:?}"))?,
                ),
            },
        };
        out.push(Line { read, data_field });
        previous = tuple;
    }
    Ok(out)
}

/// The lines of an axis Excel saved none for: one, the grand total, when the axis has no field
/// (the snapshot does not tell an absent `<rowItems>` from an empty one).
fn lines_or_total(lines: &[PivotLine], fields: &[AxisField]) -> Result<Vec<PivotLine>, String> {
    match (lines.is_empty(), fields.is_empty()) {
        (false, _) => Ok(lines.to_vec()),
        (true, true) => Ok(vec![PivotLine {
            t: None,
            r: 0,
            i: 0,
            x: Vec::new(),
        }]),
        (true, false) => Err("Excel saved no layout lines for an axis with fields".into()),
    }
}

/// The value the result gives for the cell of `row` and `column`, as Excel shows it with the
/// pivot's missing and error captions; `None` for a blank line's cell.
fn read_value(table: &PivotTable, result: &PivotResult, row: Line, column: Line) -> Option<Value> {
    let data_field = row.data_field.or(column.data_field).unwrap_or(0);
    let (r, rf) = match row.read {
        Read::Blank => return None,
        Read::Items { node, .. } => (node, None),
        Read::Subtotal { node, function } => (node, function),
    };
    let (c, cf) = match column.read {
        Read::Blank => return None,
        Read::Items { node, .. } => (node, None),
        Read::Subtotal { node, function } => (node, function),
    };
    let value = match (r, c, rf.or(cf)) {
        (Some(r), Some(c), Some(function)) => result.value_as(r, c, data_field, function),
        (Some(r), Some(c), None) => result.value(r, c, data_field),
        _ => None,
    };
    let options = &table.options;
    Some(match value {
        Some(Ok(n)) => Value::Number(n),
        Some(Err(e)) => match (&options.error_caption, options.show_error) {
            (Some(caption), true) => Value::Text(caption.clone()),
            _ => Value::Error(e.to_string()),
        },
        // An empty `missingCaption` reads as a blank cell, as the corpus's Excel-saved cells show
        // (UNVERIFIED: revisit when T-4967's probe lands).
        None => match (&options.missing_caption, options.show_missing) {
            (Some(caption), true) if !caption.is_empty() => Value::Text(caption.clone()),
            _ => Value::Empty,
        },
    })
}

/// Whether a line is an item line short of its axis's last field whose field's flags leave its
/// own cells blank (its subtotal off, at the bottom or on custom lines).
fn blank_parent(line: Line) -> bool {
    matches!(
        line.read,
        Read::Items {
            leaf: false,
            total_on_line: false,
            ..
        }
    )
}

/// Checks `saved`'s value cells against `table`'s result over `cache` on the day `today`.
///
/// # Errors
/// Why the pivot cannot be checked: it has no data field, its cache holds no records, its
/// result passes the engine's limits, or Excel's lines do not fit its location.
pub(super) fn check(
    saved: &SavedPivot,
    table: &PivotTable,
    cache: &PivotCache,
    today: Option<f64>,
) -> Result<RecordCheck, String> {
    if table.data_fields.is_empty() {
        return Err("no data field, so no value cell to check".into());
    }
    if cache.record_count == 0 {
        return Err("the cache was saved without records".into());
    }
    let mut work = MAX_PIVOT_WORK;
    let result = compute_on(table, cache, today, &mut work).map_err(|e| format!("result: {e}"))?;
    let row_lines = lines_or_total(&saved.row_items, &table.row_fields)?;
    let col_lines = lines_or_total(&saved.col_items, &table.column_fields)?;
    let rows = decode(table, &table.row_fields, result.rows(), &row_lines)?;
    let columns = decode(table, &table.column_fields, result.columns(), &col_lines)?;
    let (first, last) = saved.area.clone()?;
    let top = saved.anchor.row.saturating_add(saved.first_data_row);
    let left = saved.anchor.col.saturating_add(saved.first_data_col);
    let fits = |start: u32, count: usize, end: u32| {
        u32::try_from(count).is_ok_and(|n| start.saturating_add(n) <= end.saturating_add(1))
    };
    if !fits(top, rows.len(), last.row) || !fits(left, columns.len(), last.col) || top < first.row {
        return Err("Excel's layout lines do not fit its location".into());
    }
    let mut out = RecordCheck::default();
    for (&row, at_row) in rows.iter().zip(top..) {
        for (&column, at_col) in columns.iter().zip(left..) {
            let Some(records) = read_value(table, &result, row, column) else {
                continue;
            };
            let at = Addr {
                row: at_row,
                col: at_col,
            };
            let excel = saved.cells.get(&at).unwrap_or(&Value::Empty);
            if *excel == Value::Empty && (blank_parent(row) || blank_parent(column)) {
                continue;
            }
            out.compared += 1;
            if !compare(excel, &records).is_pass() {
                out.differing += 1;
                out.first
                    .get_or_insert_with(|| format!("{at}: Excel {excel}, records {records}"));
            }
        }
    }
    Ok(out)
}
