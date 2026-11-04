//! The `_xlpm.` form of LAMBDA/LET bound names: Excel's, on export, and the R1C1 storage form's.

use super::super::{for_each_chain_operand_mut, Node};
use crate::expressions::types::CellReferenceRC;
use crate::expressions::utils::{reads_as_r1c1_cell, spelled_like_a1_cell};
use crate::functions::logical::same_variable;
use crate::functions::Function;
use crate::language::get_default_language;

/// Where [`walk_names`] meets a name.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum NameSite {
    /// A name read as a value: a LET or LAMBDA name, or a name that only resolves at evaluation.
    Use,
    /// A name called like a function, `f(2)`: a bound LAMBDA or a name that is no function.
    Callee,
    /// The name a LET pair declares (it is bound from its own value onwards).
    Declaration,
    /// A LAMBDA parameter (bound in the LAMBDA's body).
    Parameter,
}

/// Calls `visit` with every name of `node`, where it is and the spelling of the innermost
/// enclosing LAMBDA or LET declaration that binds it (`None` when none does), `bound` holding the
/// names already in scope. Matching is case-insensitive, mirroring evaluation (see
/// `same_variable`). A LAMBDA parameter is visited as [`NameSite::Parameter`], declared by itself.
pub(crate) fn walk_names<F>(node: &mut Node, bound: &mut Vec<String>, visit: &mut F)
where
    F: FnMut(NameSite, &mut String, Option<&str>),
{
    // A run of signs, unary pluses, whitespace and parentheses is peeled in a loop, so it costs
    // no frame: 1,022 spaced signs are about 2,000 wrappers, which overflowed a 2 MiB unoptimized
    // stack at one frame each when a formula was set (T-3691, as T-3253's `Clone` and `PartialEq`).
    let mut node = node;
    loop {
        let current = node;
        node = match current {
            Node::UnaryKind { right: child, .. }
            | Node::ImplicitIntersection { child, .. }
            | Node::SpillRangeOperator { child }
            | Node::Group { child, .. }
            | Node::UnaryPlus { child }
            | Node::Spaced { child, .. } => child,
            other => {
                node = other;
                break;
            }
        };
    }
    // A use the parser has not settled yet may still carry the `_xlpm.` prefix it was written
    // with; the declaration binding it has none (`call_names::settle_lambda_prefixes`, T-4099).
    let declared = |bound: &[String], name: &str| {
        let name = name.strip_prefix("_xlpm.").unwrap_or(name);
        bound.iter().rev().find(|n| same_variable(n, name)).cloned()
    };
    match node {
        Node::NamedVariableKind { name, .. } => {
            let d = declared(bound, name);
            visit(NameSite::Use, name, d.as_deref());
        }
        Node::NamedFunctionKind { name, args, .. } => {
            let d = declared(bound, name);
            visit(NameSite::Callee, name, d.as_deref());
            for arg in args {
                walk_names(arg, bound, visit);
            }
        }
        Node::LambdaDefKind { parameters, body } => {
            let depth = bound.len();
            for parameter in parameters.iter_mut() {
                bound.push(parameter.name.clone());
                let d = parameter.name.clone();
                visit(NameSite::Parameter, &mut parameter.name, Some(&d));
            }
            walk_names(body, bound, visit);
            bound.truncate(depth);
        }
        Node::FunctionKind {
            kind: Function::Let,
            args,
        } if args.len() >= 3 && args.len() % 2 == 1 => {
            // LET(name1, value1, [name2, value2, ...], body): each name is in
            // scope from its own value expression onwards.
            let depth = bound.len();
            let pair_count = (args.len() - 1) / 2;
            for i in 0..pair_count {
                // The name may be written with whitespace around it, `LET( x, 1, x)` (T-2948).
                if let Node::NamedVariableKind { name, .. } = args[2 * i].unwrap_spaced_mut() {
                    bound.push(name.clone());
                    let d = name.clone();
                    visit(NameSite::Declaration, name, Some(&d));
                }
                walk_names(&mut args[2 * i + 1], bound, visit);
            }
            walk_names(&mut args[2 * pair_count], bound, visit);
            bound.truncate(depth);
        }
        Node::FunctionKind { args, .. } => {
            for arg in args {
                walk_names(arg, bound, visit);
            }
        }
        Node::LambdaCallKind { lambda, args } => {
            walk_names(lambda, bound, visit);
            for arg in args {
                walk_names(arg, bound, visit);
            }
        }
        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            walk_names(left, bound, visit);
            walk_names(right, bound, visit);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| walk_names(operand, bound, visit));
        }
        Node::UnaryKind { right, .. } => walk_names(right, bound, visit),
        Node::ImplicitIntersection { child, .. }
        | Node::SpillRangeOperator { child }
        | Node::Group { child, .. }
        | Node::UnaryPlus { child }
        | Node::Spaced { child, .. } => walk_names(child, bound, visit),
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::StructuredReferenceKind { .. }
        | Node::ThisRowKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::ArrayKind(_)
        | Node::DefinedNameKind(_)
        | Node::ErrorKind(_)
        | Node::SheetRefErrorKind { .. }
        | Node::ParseErrorKind { .. }
        | Node::EmptyArgKind => {}
    }
}

/// Excel stores LAMBDA parameters and LET variables with an `_xlpm.` prefix,
/// both at the declaration and at every use site:
/// `LET(x,1,x*2)` is written as `_xlfn.LET(_xlpm.x,1,_xlpm.x*2)`.
/// Internally IronCalc keeps the bare names, so before exporting we walk the
/// tree tracking which names are bound by an enclosing LAMBDA/LET and rename
/// those occurrences, a bound lambda used as a function (`LET(f,LAMBDA(a,a*a),f(2))`)
/// included. Unbound names (e.g. a defined name that only resolves at
/// evaluation time) are left alone.
pub(super) fn prefix_bound_variables(node: &mut Node) {
    // A LAMBDA parameter is printed with its own prefix, `_xlpm.` or `_xlop.` (`stringify`).
    walk_names(node, &mut Vec::new(), &mut |site, name, declared| {
        if declared.is_some() && site != NameSite::Parameter {
            *name = format!("_xlpm.{name}");
        }
    });
}

/// The name inside `'f1'`, the quoted spelling `quote_unbound_cell_shaped_names` (`call_names`) gives an
/// unbound name spelled like an A1 cell.
fn unquoted_cell_shaped_name(name: &str) -> Option<&str> {
    name.strip_prefix('\'')
        .and_then(|name| name.strip_suffix('\''))
        .filter(|inner| spelled_like_a1_cell(inner))
}

/// Gives every LET name and LAMBDA parameter spelled like an A1 cell (`f1`, `a1`), at its
/// declaration and at every use, the `_xlpm.` prefix the parser drops, and an unbound one (the
/// quoted `'f1'`) two of them, `_xlpm._xlpm.f1`: the A1 text of a formula typed in R1C1 mode,
/// where such a name is no reference. One prefix would not do for the unbound one: typed in A1
/// mode, an unbound `_xlpm.f1` is the name written with its prefix, which FORMULATEXT shows
/// (probes/t4099-file-xlpm-unbound-names A10, T-4099), so the parser reads the doubled prefix in
/// A1 text as the quoted name instead (`Parser::written_lambda_name`). Excel binds a bound one (probes/t3897-r1c1-let-cell-names, T-3897), which the
/// typed A1 entry then binds too instead of refusing it as it refuses `f1` typed in A1 mode
/// (decision 0103); an unbound one is Excel's undefined name (`#NAME?`, A8:A10, A13), which the
/// typed A1 entry reads back quoted (`quote_unbound_cell_shaped_names` (`call_names`), T-4040) where bare A1
/// text would read the cell, and Excel refuses `='f1'` typed in either mode
/// (probes/t4040-r1c1-unbound-cell-names A1:A6).
fn prefix_a1_cell_shaped_variables(node: &mut Node) {
    walk_names(node, &mut Vec::new(), &mut |_, name, declared| {
        if let Some(inner) = unquoted_cell_shaped_name(name) {
            *name = format!("_xlpm._xlpm.{inner}");
        } else if declared.is_some() && spelled_like_a1_cell(name) {
            *name = format!("_xlpm.{name}");
        }
    });
}

/// [`super::to_english_string`] of a formula typed in R1C1 mode, as the A1 text the typed A1 entry
/// reads the same: a name spelled like an A1 cell, bound or not, keeps its `_xlpm.` prefix
/// ([`prefix_a1_cell_shaped_variables`], T-3897, T-4040).
pub(crate) fn to_english_string_typed_from_r1c1(node: &Node, context: &CellReferenceRC) -> String {
    let mut node = node.clone();
    prefix_a1_cell_shaped_variables(&mut node);
    super::to_english_string(&node, context)
}

/// Whether a stored (R1C1) formula would read `name` back as a cell, or as another name: an R1C1
/// cell (`R1C1`, T-1658), or a name spelled like an A1 cell (`f1`, T-2040), which the R1C1 lexer
/// reads as a name, but quoted when no LET or LAMBDA binds it
/// (`quote_unbound_cell_shaped_names` (`call_names`)).
fn reads_back_as_cell(name: &str) -> bool {
    reads_as_r1c1_cell(name) || spelled_like_a1_cell(name)
}

/// The text of the quoted name `'f1'` (`quote_unbound_cell_shaped_names` (`call_names`)), when `name` is one:
/// quoted in A1 output, as Excel's FORMULATEXT shows it, and `_xlpm.f1` in the R1C1 storage form
/// (no `context`) and in a file (`export_to_excel`). The parser reads the storage form back
/// quoted; a file's `_xlpm.f1` is Excel's undefined name too (`#NAME?`), though one written with
/// its prefix, which FORMULATEXT shows (probes/t4040-r1c1-unbound-cell-names A14:C16, T-4099).
fn quoted_name_text(
    name: &str,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> Option<String> {
    let inner = unquoted_cell_shaped_name(name)?;
    Some(if context.is_none() || export_to_excel {
        format!("_xlpm.{inner}")
    } else {
        name.to_string()
    })
}

/// A LET or LAMBDA name (or any unresolved name). In the R1C1 storage form (no `context`), one
/// the stored formula would read back as a cell gets the `_xlpm.` prefix the parser drops, as on
/// xlsx import: Excel takes `LET(R1C1,5,R1C1*2)` (10, probes/r1c1-let-name-r1c1, T-1658) and,
/// stored as `_xlpm.f1`, `LET(f1,1,f1+1)` (2, probes/let-cell-shaped-name-stored, T-2040). An
/// unbound name written with its prefix, `_xlpm.abc`, keeps it, so it is stored with two,
/// `_xlpm._xlpm.abc`, which the parser reads back as one (`Parser::written_lambda_name`, T-4099).
pub(super) fn variable_text(
    name: &str,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> String {
    if let Some(text) = quoted_name_text(name, context, export_to_excel) {
        return text;
    }
    // No `context` is the R1C1 storage form (`to_rc_format`); a context is A1 output.
    if context.is_none() && (reads_back_as_cell(name) || name.starts_with("_xlpm.")) {
        format!("_xlpm.{name}")
    } else {
        name.to_string()
    }
}

/// A name called like a function that is none. In the R1C1 storage form (no `context`), one the
/// stored formula would read back as something else gets the `_xlpm.` prefix the parser drops:
/// a cell, as for [`variable_text`], or a function or boolean of the English the storage is read
/// in (a Spanish `=TRUE()` calls no function, and must not share the stored text of `TRUE()`).
pub(super) fn callee_text(
    name: &str,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> String {
    if let Some(text) = quoted_name_text(name, context, export_to_excel) {
        return text;
    }
    let reads_as_function = || {
        let english = get_default_language();
        english.functions.lookup_stored(name).is_some()
            || name.eq_ignore_ascii_case(&english.booleans.r#true)
            || name.eq_ignore_ascii_case(&english.booleans.r#false)
    };
    if context.is_none()
        && (reads_back_as_cell(name) || name.starts_with("_xlpm.") || reads_as_function())
    {
        format!("_xlpm.{name}")
    } else {
        name.to_string()
    }
}

/// A plain name called with arguments (an unknown identifier used as a function, a
/// `LambdaCallKind`): lower case, to tell it from a real function call in the R1C1 form and to
/// match the display of an invalid function; the quoted `'f1'` keeps its text (`='f1'(2)`,
/// [`quoted_name_text`], T-4040).
pub(super) fn called_variable_text(
    name: &str,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> String {
    quoted_name_text(name, context, export_to_excel).unwrap_or_else(|| name.to_lowercase())
}
