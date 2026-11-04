//! The corners of a plain range when a delete takes its first or last rows or columns.

use super::DisplaceData;

/// The displacements of the two corners of a range spanning `span` (rows or columns) when a
/// delete of `-delta` of them from `at` takes its first or last, but not all of them: the range
/// shrinks to what is left, where displacing the deleted corner alone would give `#REF!`.
/// `shift_to(from, to)` is the displacement that moves the corner on `from` to `to`.
///
/// Rows: plain-range-delete-row-first and -last (excel-graph, T-2338). Columns:
/// table-column-range-delete-start and -end (excel-graph 2026-11-01, T-3978), where deleting
/// column C or D turns `=SUM(S!C4:D7)` into `=SUM(S!C4:C7)`.
fn shrunk_corners(
    span: (i32, i32),
    at: i32,
    delta: i32,
    shift_to: impl Fn(i32, i32) -> DisplaceData,
) -> Option<(DisplaceData, DisplaceData)> {
    let deleted = at..=at - delta - 1;
    if deleted.contains(&span.0) && !deleted.contains(&span.1) {
        Some((shift_to(span.0, at), shift_to(span.1, span.1 + delta)))
    } else if !deleted.contains(&span.0) && deleted.contains(&span.1) {
        Some((DisplaceData::None, shift_to(span.1, at - 1)))
    } else {
        None
    }
}

/// [`shrunk_corners`] for the row or column delete `data` on sheet `sheet_index`, a range spanning
/// `rows` x `columns`: `None` when `data` is no such delete, else the corners it gives (`None`
/// inside when the range is untouched or wholly deleted, whose corners displace as usual).
pub(super) fn delete_corners(
    data: &DisplaceData,
    sheet_index: u32,
    rows: (i32, i32),
    columns: (i32, i32),
) -> Option<Option<(DisplaceData, DisplaceData)>> {
    match *data {
        DisplaceData::Row { sheet, row, delta } if sheet == sheet_index && delta < 0 => {
            Some(shrunk_corners(rows, row, delta, |from, to| {
                DisplaceData::Row {
                    sheet,
                    row: from.min(to),
                    delta: to - from,
                }
            }))
        }
        DisplaceData::Column {
            sheet,
            column,
            delta,
        } if sheet == sheet_index && delta < 0 => {
            Some(shrunk_corners(columns, column, delta, |from, to| {
                DisplaceData::Column {
                    sheet,
                    column: from.min(to),
                    delta: to - from,
                }
            }))
        }
        _ => None,
    }
}
