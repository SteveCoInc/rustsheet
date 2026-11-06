//! The file-form (xlsx) printers: the text a formula is exported with.

use super::bound_variables::prefix_bound_variables;
use super::{stringify, Displacement, Node, Rewrite, NO_REWRITE};
use crate::expressions::parser::static_analysis::remove_redundant_implicit_intersection;
use crate::expressions::types::CellReferenceRC;
use crate::language::get_default_language;
use crate::locale::get_default_locale;

/// This is the mode used to export the formula to Excel
/// Internally the locale and language are always "en"
pub fn to_excel_string(node: &Node, context: &CellReferenceRC) -> String {
    excel_string(node, context, false, &NO_REWRITE)
}

/// [`to_excel_string`] displaced by a structural edit, as [`super::to_string_displaced_keeping`]
/// displaces the A1 text: the file-form text of a table's `calculatedColumnFormula` after the
/// edit (T-4274).
pub(crate) fn to_excel_string_displaced(
    node: &Node,
    context: &CellReferenceRC,
    displacement: Displacement,
) -> String {
    excel_string(node, context, false, &Rewrite::Displace(displacement))
}

/// Like [`to_excel_string`], for the formula of an array, CSE or dynamic.
///
/// No implicit intersection is added to the formula of an array when it is
/// read, by Excel or by the import, so none of its `@` is redundant: they are
/// all written, as `_xlfn.SINGLE`.
pub fn to_excel_array_formula_string(node: &Node, context: &CellReferenceRC) -> String {
    excel_string(node, context, true, &NO_REWRITE)
}

fn excel_string(
    node: &Node,
    context: &CellReferenceRC,
    is_array_formula: bool,
    rewrite: &Rewrite,
) -> String {
    let (locale, language) = (get_default_locale(), get_default_language());
    // The internal representation stores every implicit intersection as `@`,
    // without the `automatic` flag. Drop the operators that Excel would re-insert
    // automatically on import; the rest are kept as `_xlfn.SINGLE` while
    // stringifying. See `remove_redundant_implicit_intersection`.
    let mut node = node.clone();
    if !is_array_formula {
        remove_redundant_implicit_intersection(&mut node, true);
    }
    prefix_bound_variables(&mut node);
    stringify(&node, Some(context), rewrite, true, locale, language)
}
