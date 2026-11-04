//! The `_xlpm.` form of LAMBDA/LET bound names: Excel's, on export, and the R1C1 storage form's.

use super::super::{for_each_chain_operand_mut, Node};
use crate::expressions::types::CellReferenceRC;
use crate::expressions::utils::{parse_reference_a1, reads_as_r1c1_cell};
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
    let declared =
        |bound: &[String], name: &str| bound.iter().rev().find(|n| same_variable(n, name)).cloned();
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

/// Gives every LET name and LAMBDA parameter spelled like an A1 cell (`f1`, `a1`), at its
/// declaration and at every use, the `_xlpm.` prefix the parser drops: the A1 text of a formula
/// typed in R1C1 mode, where such a name is no reference and Excel binds it
/// (probes/t3897-r1c1-let-cell-names, T-3897), so that the typed A1 entry binds it too instead of
/// refusing it as it refuses `f1` typed in A1 mode (decision 0103). Returns whether every name
/// spelled like an A1 cell is bound: an unbound one (`=f1`) is an undefined name in Excel
/// (`#NAME?`, A8:A10, A13), which bare A1 text would read as the cell instead.
fn prefix_a1_cell_shaped_variables(node: &mut Node) -> bool {
    let mut all_bound = true;
    walk_names(node, &mut Vec::new(), &mut |_, name, declared| {
        if parse_reference_a1(&name.to_ascii_uppercase()).is_some() {
            if declared.is_some() {
                *name = format!("_xlpm.{name}");
            } else {
                all_bound = false;
            }
        }
    });
    all_bound
}

/// [`super::to_english_string`] of a formula typed in R1C1 mode, as the A1 text the typed A1 entry
/// reads the same: a LET name or LAMBDA parameter spelled like an A1 cell keeps its `_xlpm.`
/// prefix ([`prefix_a1_cell_shaped_variables`], T-3897). `None` when such a name
/// is unbound: Excel reads it as an undefined name, which no A1 text here spells yet
/// (TODO(T-4040)).
pub(crate) fn to_english_string_typed_from_r1c1(
    node: &Node,
    context: &CellReferenceRC,
) -> Option<String> {
    let mut node = node.clone();
    prefix_a1_cell_shaped_variables(&mut node).then(|| super::to_english_string(&node, context))
}

/// Whether a stored (R1C1) formula would read `name` back as a cell, or refuse it: an R1C1 cell
/// (`R1C1`, T-1658), or a name spelled like an A1 cell (`f1`, T-2040), which the R1C1 lexer
/// refuses as an identifier.
fn reads_back_as_cell(name: &str) -> bool {
    reads_as_r1c1_cell(name) || parse_reference_a1(&name.to_ascii_uppercase()).is_some()
}

/// A LET or LAMBDA name (or any unresolved name). In the R1C1 storage form (no `context`), one
/// the stored formula would read back as a cell gets the `_xlpm.` prefix the parser drops, as on
/// xlsx import: Excel takes `LET(R1C1,5,R1C1*2)` (10, probes/r1c1-let-name-r1c1, T-1658) and,
/// stored as `_xlpm.f1`, `LET(f1,1,f1+1)` (2, probes/let-cell-shaped-name-stored, T-2040).
pub(super) fn variable_text(name: &str, context: Option<&CellReferenceRC>) -> String {
    // No `context` is the R1C1 storage form (`to_rc_format`); a context is A1 output.
    if context.is_none() && reads_back_as_cell(name) {
        format!("_xlpm.{name}")
    } else {
        name.to_string()
    }
}

/// A name called like a function that is none. In the R1C1 storage form (no `context`), one the
/// stored formula would read back as something else gets the `_xlpm.` prefix the parser drops:
/// a cell, as for [`variable_text`], or a function or boolean of the English the storage is read
/// in (a Spanish `=TRUE()` calls no function, and must not share the stored text of `TRUE()`).
pub(super) fn callee_text(name: &str, context: Option<&CellReferenceRC>) -> String {
    let reads_as_function = || {
        let english = get_default_language();
        english.functions.lookup_stored(name).is_some()
            || name.eq_ignore_ascii_case(&english.booleans.r#true)
            || name.eq_ignore_ascii_case(&english.booleans.r#false)
    };
    if context.is_none() && (reads_back_as_cell(name) || reads_as_function()) {
        format!("_xlpm.{name}")
    } else {
        name.to_string()
    }
}
