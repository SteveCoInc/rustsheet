//! Printing a reference whose sheet the parser did not find (`WrongReferenceKind`,
//! `WrongRangeKind`): a deleted sheet, or an external one.

use super::{open_range_flags, stringify_reference, DisplaceData};
use crate::expressions::parser::{Node, Reference};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceRC;
use crate::expressions::utils::quote_name;

/// The sheet printed in place of a missing one: none, after a `#REF!` prefix.
const NO_SHEET: &Option<String> = &None;

/// The sheet name a reference to a deleted sheet is stored under once the delete rewrote it
/// (`delete_sheet_in_node`, T-2612). `*` is not allowed in a sheet name, so no sheet added later
/// can take it, and the reference stays `#REF!` as in Excel
/// (probes/table-edits/t2612-delete-sheet-then-add). Only the stored R1C1 text holds it; the
/// display and the export print `#REF!` in its place.
pub(crate) const DELETED_SHEET: &str = "*#REF!";

/// A reference to a sheet that does not exist (a deleted one) is displayed as `#REF!` before the
/// cell or range, as Excel prints `=SUM(#REF!C4:C7)` for `S!C4:C7` once S is deleted (T-2340,
/// probes/table-edits/table-delete-sheet); Excel refuses that text typed in, and so does the lexer
/// (T-4628). A sheet delete rewrites its references to [`DELETED_SHEET`], which the export writes
/// as the bare `#REF!` ([`is_exported_deleted_sheet`]); other missing sheets (a loaded `Nope!A1`) keep their name in
/// the stored text and the export. An external reference (`[1]Uses!A1`) is not a missing sheet.
fn is_missing_sheet(
    sheet_name: Option<&str>,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> bool {
    match sheet_name {
        Some(DELETED_SHEET) => context.is_some(),
        Some(name) => !name.starts_with('[') && context.is_some() && !export_to_excel,
        None => false,
    }
}

/// Whether the export writes a reference to a deleted sheet as the bare `#REF!`: Excel for the web
/// refuses a workbook that stores `SUM(#REF!C4:C7)`, `#REF!$A$1` or `#REF!A:A`, and loads
/// `SUM(#REF!)` (probes/t4628-deleted-sheet-reference-text, hand-built workbooks, excel-graph,
/// 2026-11-07, T-4628). The value is `#REF!` either way; the range is lost from the file.
fn is_exported_deleted_sheet(sheet_name: Option<&str>, export_to_excel: bool) -> bool {
    export_to_excel && sheet_name == Some(DELETED_SHEET)
}

/// `#REF!` written for a deleted reference, after the sheet prefix Excel keeps (`S!#REF!`) when
/// there is one (references.md "A sheet-qualified #REF! is #REF!", T-2855).
pub(super) fn ref_error_text(sheet_name: Option<&str>) -> String {
    match sheet_name {
        Some(name) => format!("{}!{}", quote_name(name), Error::REF),
        None => Error::REF.to_string(),
    }
}

/// The text of a written `S!#REF!`: `#REF!#REF!` when S does not exist (Excel, once S is deleted;
/// probes/sheet-qualified-ref-error-text-delete-sheet F!A2), as for any reference to a deleted
/// sheet ([`is_missing_sheet`]); otherwise `S!#REF!`.
fn sheet_ref_error_text(
    sheet_name: &str,
    missing: bool,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> String {
    if missing && is_missing_sheet(Some(sheet_name), context, export_to_excel) {
        format!("{}{}", Error::REF, Error::REF)
    } else {
        ref_error_text(Some(sheet_name))
    }
}

/// The text of a `WrongReferenceKind` or `WrongRangeKind` node and of a written `S!#REF!`
/// (empty for any other node).
pub(super) fn text(
    node: &Node,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> String {
    let corner = |sheet_name, row, column, absolute_row, absolute_column, full_row, full_column| {
        stringify_reference(
            context,
            &DisplaceData::None,
            &Reference {
                sheet_name,
                sheet_index: 0, // HACK
                row,
                column,
                absolute_row,
                absolute_column,
            },
            full_row,
            full_column,
        )
    };
    let missing = |sheet_name: &Option<String>| {
        is_missing_sheet(sheet_name.as_deref(), context, export_to_excel)
    };
    match node {
        Node::SheetRefErrorKind {
            sheet_name,
            missing,
        } => sheet_ref_error_text(sheet_name, *missing, context, export_to_excel),
        Node::WrongReferenceKind {
            sheet_name,
            column,
            row,
            absolute_row,
            absolute_column,
        } => {
            if is_exported_deleted_sheet(sheet_name.as_deref(), export_to_excel) {
                return Error::REF.to_string();
            }
            let (prefix, sheet_name) = if missing(sheet_name) {
                ("#REF!", NO_SHEET)
            } else {
                ("", sheet_name)
            };
            let text = corner(
                sheet_name,
                *row,
                *column,
                *absolute_row,
                *absolute_column,
                false,
                false,
            );
            format!("{prefix}{text}")
        }
        Node::WrongRangeKind {
            sheet_name,
            row1,
            column1,
            absolute_row1,
            absolute_column1,
            row2,
            column2,
            absolute_row2,
            absolute_column2,
        } => {
            if is_exported_deleted_sheet(sheet_name.as_deref(), export_to_excel) {
                return Error::REF.to_string();
            }
            let (full_row, full_column) = open_range_flags(
                (*absolute_row1, *row1, *absolute_row2, *row2),
                (*absolute_column1, *column1, *absolute_column2, *column2),
            );
            let (prefix, sheet_name) = if missing(sheet_name) {
                ("#REF!", NO_SHEET)
            } else {
                ("", sheet_name)
            };
            let s1 = corner(
                sheet_name,
                *row1,
                *column1,
                *absolute_row1,
                *absolute_column1,
                full_row,
                full_column,
            );
            let s2 = corner(
                &None,
                *row2,
                *column2,
                *absolute_row2,
                *absolute_column2,
                full_row,
                full_column,
            );
            format!("{prefix}{s1}:{s2}")
        }
        _ => String::new(),
    }
}
