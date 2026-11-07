//! Rewrites of the sheet prefixes in a parsed formula when a sheet is renamed or moved.

use super::wrong_reference::DELETED_SHEET;
use crate::expressions::parser::{for_each_chain_operand_mut, Node};
use crate::expressions::token::Error;
use crate::expressions::utils::rename_defined_name_sheet;
use crate::types::sheet_names_match;

/// Calls `f` on every node of `node` that names a sheet (a reference, a 3-D span, a wrong
/// reference or a defined name) and walks through every other node to reach them.
pub(super) fn for_each_sheet_reference_mut(node: &mut Node, f: &mut impl FnMut(&mut Node)) {
    match node {
        Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::ThisRowKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::SheetRefErrorKind { .. }
        | Node::DefinedNameKind(_) => f(node),

        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            for_each_sheet_reference_mut(left, f);
            for_each_sheet_reference_mut(right, f);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| for_each_sheet_reference_mut(operand, f));
        }
        Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
            for arg in args {
                for_each_sheet_reference_mut(arg, f);
            }
        }
        Node::UnaryKind { right: child, .. }
        | Node::ImplicitIntersection { child, .. }
        | Node::SpillRangeOperator { child }
        | Node::Group { child, .. }
        | Node::UnaryPlus { child }
        | Node::Spaced { child, .. }
        | Node::StructuredReferenceKind { lowered: child, .. }
        | Node::LambdaDefKind { body: child, .. } => for_each_sheet_reference_mut(child, f),
        Node::LambdaCallKind { lambda, args } => {
            for_each_sheet_reference_mut(lambda, f);
            for arg in args {
                for_each_sheet_reference_mut(arg, f);
            }
        }

        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::ArrayKind(_)
        | Node::NamedVariableKind { .. }
        | Node::EmptyArgKind => {}
    }
}

/// Writes `new_name` for the sheet at `sheet_index` (named `old_name` until now) everywhere `node`
/// names it. A `S!#REF!` holds no index, so it follows the rename by name: Excel prints
/// `T!#REF!` after S becomes T (probes/sheet-qualified-ref-error-text-rename F!A2).
pub(crate) fn rename_sheet_in_node(
    node: &mut Node,
    sheet_index: u32,
    old_name: &str,
    new_name: &str,
) {
    for_each_sheet_reference_mut(node, &mut |node| match node {
        Node::SheetSpanKind {
            first_sheet,
            last_sheet,
            first_index,
            last_index,
            ..
        } => {
            if *first_index == sheet_index {
                new_name.clone_into(first_sheet);
            }
            if *last_index == sheet_index {
                new_name.clone_into(last_sheet);
            }
        }
        Node::WrongReferenceKind { sheet_name, .. } => {
            if let Some(name) = sheet_name {
                if name.to_uppercase() == new_name.to_uppercase() {
                    *sheet_name = Some(name.to_owned());
                }
            }
        }
        Node::SheetRefErrorKind { sheet_name, .. } => {
            if sheet_names_match(sheet_name, old_name) {
                new_name.clone_into(sheet_name);
            }
        }
        Node::WrongRangeKind { sheet_name, .. } => {
            if sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }
        Node::DefinedNameKind((n, s, _)) => rename_defined_name_sheet(n, *s, sheet_index, new_name),
        _ => node.rename_reference_sheet(sheet_index, new_name),
    });
}

/// Points every reference of `node` written without a sheet prefix at the sheet `to`. A
/// D-function's computed criterion reads its unqualified references on the database's sheet, not
/// its own (probes/db-computed-criteria-cross-sheet, criteria.md "D* computed criteria and an
/// unknown header in the gen-m9 database sweep (T-2258)").
pub(crate) fn retarget_unqualified_references(node: &mut Node, to: u32) {
    for_each_sheet_reference_mut(node, &mut |node| {
        if let Node::ReferenceKind {
            sheet_name: None,
            sheet_index,
            ..
        }
        | Node::RangeKind {
            sheet_name: None,
            sheet_index,
            ..
        }
        | Node::ThisRowKind {
            sheet_name: None,
            sheet_index,
            ..
        } = node
        {
            *sheet_index = to;
        }
    });
}

/// Writes `sheet` as the prefix of every reference of `node` written without one, so the text
/// keeps reading that sheet from any other context: an API-added name's body read on the active
/// sheet is stored as the defined-name context would write it (T-4445).
pub(crate) fn qualify_unqualified_references(node: &mut Node, sheet: &str) {
    for_each_sheet_reference_mut(node, &mut |node| {
        if let Node::ReferenceKind { sheet_name, .. }
        | Node::RangeKind { sheet_name, .. }
        | Node::ThisRowKind { sheet_name, .. } = node
        {
            if sheet_name.is_none() {
                *sheet_name = Some(sheet.to_owned());
            }
        }
    });
}

/// Rewrites the 3-D spans of `node` for moving the sheet at `from` to `to`. Both indices and
/// `sheet_names` (in tab order) are from before the move.
///
/// Excel rewrites a span whose moved end lands past its other end to the sheets still between
/// the ends, and a span left with one sheet to a plain reference to it: with Calc, Jan, Feb, Mar,
/// moving Mar first turns `Jan:Mar!A1` into `Jan:Feb!A1`, and moving Feb first turns `Jan:Feb!A1`
/// into `Jan!A1` (docs/excel-semantics/references.md, "3-D references when sheets move or are
/// renamed"). Every other move keeps the ends, which then resolve by name in the new order.
pub(crate) fn move_sheet_in_node(node: &mut Node, from: u32, to: u32, sheet_names: &[String]) {
    // After the move, the last sheet lands before the first one when it goes to `low` or
    // earlier; the first sheet lands after the last one when it goes to `high` or later (the
    // last sheet is at `high - 1` once the first sheet is taken out).
    shrink_spans(node, sheet_names, |low, high| {
        if from == high && to <= low {
            Some((low, high - 1))
        } else if from == low && to >= high {
            Some((low + 1, high))
        } else {
            None
        }
    });
}

/// Rewrites the 3-D spans of `node` for deleting the sheet at `deleted`. The index and
/// `sheet_names` (in tab order) are from before the delete.
///
/// Excel moves an end that is deleted to the next sheet inward, and a span left with one sheet
/// becomes a plain reference: with Calc, Jan, Feb, Mar, deleting Mar turns `Jan:Mar!A1` into
/// `Jan:Feb!A1`, and deleting Feb turns `Jan:Feb!A1` into `Jan!A1`
/// (probes/sheet-ops/t4272-3d-span-delete-end, -delete-start, -delete-two-sheet-end).
/// Deleting a middle sheet keeps the ends.
///
/// A plain reference to the deleted sheet loses its sheet for good: it becomes a wrong reference
/// to [`DELETED_SHEET`], displayed and exported as `#REF!C4:C7`, so a sheet added later under the
/// old name does not revive it (probes/table-edits/t2612-delete-sheet-then-add, T-2612).
pub(crate) fn delete_sheet_in_node(node: &mut Node, deleted: u32, sheet_names: &[String]) {
    shrink_spans(node, sheet_names, |low, high| {
        if deleted == low {
            Some((low + 1, high))
        } else if deleted == high {
            Some((low, high - 1))
        } else {
            None
        }
    });
    for_each_sheet_reference_mut(node, &mut |node| lose_deleted_sheet(node, deleted));
}

/// [`delete_sheet_in_node`] for a defined name's formula, which is stored as A1 text and parsed
/// again: a reference to the deleted sheet becomes the bare `#REF!` error, as the lexer does not
/// read `#REF!$C$4` (Excel refuses that text, probes/t4628-deleted-sheet-reference-text, T-4628).
/// The name still evaluates to `#REF!` and `ISREF` of it is FALSE
/// (probes/table-edits/t2612-delete-sheet-then-add-names).
pub(crate) fn delete_sheet_in_name_node(node: &mut Node, deleted: u32, sheet_names: &[String]) {
    delete_sheet_in_node(node, deleted, sheet_names);
    for_each_sheet_reference_mut(node, &mut |node| {
        if let Node::WrongReferenceKind { sheet_name, .. }
        | Node::WrongRangeKind { sheet_name, .. } = node
        {
            if sheet_name.as_deref() == Some(DELETED_SHEET) {
                *node = Node::ErrorKind(Error::REF);
            }
        }
    });
}

/// `node` as a wrong reference to [`DELETED_SHEET`] when it is a reference or range on the sheet
/// at `deleted`. Only a sheet-qualified one can be: the deleted sheet's own formulas are not
/// rewritten.
fn lose_deleted_sheet(node: &mut Node, deleted: u32) {
    let lost = match node {
        Node::ReferenceKind {
            sheet_name: Some(_),
            sheet_index,
            absolute_row,
            absolute_column,
            row,
            column,
        } if *sheet_index == deleted => Node::WrongReferenceKind {
            sheet_name: Some(DELETED_SHEET.to_string()),
            absolute_row: *absolute_row,
            absolute_column: *absolute_column,
            row: *row,
            column: *column,
        },
        Node::RangeKind {
            sheet_name: Some(_),
            sheet_index,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
        } if *sheet_index == deleted => Node::WrongRangeKind {
            sheet_name: Some(DELETED_SHEET.to_string()),
            absolute_row1: *absolute_row1,
            absolute_column1: *absolute_column1,
            row1: *row1,
            column1: *column1,
            absolute_row2: *absolute_row2,
            absolute_column2: *absolute_column2,
            row2: *row2,
            column2: *column2,
        },
        _ => return,
    };
    *node = lost;
}

/// Gives every multi-sheet 3-D span of `node` the ends `ends(low, high)` returns for its ends in
/// tab order (`low < high`, indices into `sheet_names`), or leaves it when that is `None`.
fn shrink_spans(
    node: &mut Node,
    sheet_names: &[String],
    ends: impl Fn(u32, u32) -> Option<(u32, u32)>,
) {
    for_each_sheet_reference_mut(node, &mut |node| {
        let Node::SheetSpanKind {
            first_sheet,
            last_sheet,
            first_index,
            last_index,
            ..
        } = node
        else {
            return;
        };
        let (low, high) = if first_index <= last_index {
            (*first_index, *last_index)
        } else {
            (*last_index, *first_index)
        };
        if low == high {
            return;
        }
        let Some((low, high)) = ends(low, high) else {
            return;
        };
        let (Some(first), Some(last)) = (
            sheet_names.get(low as usize),
            sheet_names.get(high as usize),
        ) else {
            return;
        };
        first.clone_into(first_sheet);
        last.clone_into(last_sheet);
        *first_index = low;
        *last_index = high;
    });
}
