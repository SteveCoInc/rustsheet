//! What a sheet-scoped name added after load does to the formulas of its own sheet (T-4049).
//!
//! Excel binds a cell's name when the formula is entered or loaded, and a sheet-scoped add moves
//! only the bindings that pointed at a *defined* workbook name: probes/t4049-sheet-api-name-plain
//! and -over-book (excel-graph 2026-11-05; docs/excel-semantics/syntax.md "A sheet-scoped name
//! added after load"). The engine keeps a formula's binding in its stored text, so the add
//! rewrites that text before every formula is re-parsed.

use super::bound_variables::{walk_names, NameSite};
use super::sheet_edits::for_each_sheet_reference_mut;
use crate::expressions::parser::Node;
use crate::expressions::utils::{bare_defined_name, quote_name, OWN_BOOK_NAME_PREFIX};

/// Pins every use of the unresolved name `name` in `node` to the workbook name it was bound to,
/// spelled `[0]!name` (the parser reads it back as that, shown as `name`): a sheet-scoped `name`
/// added afterwards leaves the cell `#NAME?` (probes/t4049-sheet-api-name-plain S!A1:A3).
/// A use some LET or LAMBDA binds is a variable, not the name, and is left alone.
pub(crate) fn pin_unresolved_name(node: &mut Node, name: &str) {
    walk_names(node, &mut Vec::new(), &mut |site, used, declared| {
        if site == NameSite::Use && declared.is_none() && used.to_lowercase() == name.to_lowercase()
        {
            *used = format!("{OWN_BOOK_NAME_PREFIX}{used}");
        }
    });
}

/// Points every sheet-qualified read of the workbook name `name` in `node` (`S!N` falling back to
/// the workbook's `N`, stored `[0]!N`) at the name scoped to sheet `scope`, written
/// `<sheet_name>!N`: on that sheet Excel moves it to the sheet-scoped name it just added
/// (probes/t4049-sheet-api-name-over-book S!A3, FORMULATEXT `=S!N`).
pub(crate) fn rebind_workbook_name_to_sheet(
    node: &mut Node,
    name: &str,
    scope: u32,
    sheet_name: &str,
) {
    for_each_sheet_reference_mut(node, &mut |node| {
        if let Node::DefinedNameKind((text, text_scope, _)) = node {
            let bare = bare_defined_name(text);
            let qualified = bare.len() < text.len();
            if text_scope.is_none()
                && qualified
                && (!text.starts_with('[') || text.starts_with("[0]"))
                && bare.to_lowercase() == name.to_lowercase()
            {
                *text = format!("{}!{bare}", quote_name(sheet_name));
                *text_scope = Some(scope);
            }
        }
    });
}
