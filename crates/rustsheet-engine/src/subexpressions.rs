//! Every subexpression of a formula, printed so that it computes the same value in any cell, for
//! the harness's `subexpr-case` localization probe (T-3333). The harness reuses the engine's
//! parser and printer instead of a second one (CLAUDE.md directive 3).

use std::collections::HashMap;

use crate::expressions::parser::stringify::to_english_string;
use crate::expressions::parser::{new_parser_english, Node};
use crate::expressions::types::CellReferenceRC;
use crate::functions::Function;

/// What a subexpression is, so the harness knows whether a cell of its own can check it.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubexpressionKind {
    /// A constant (`2`, `-2`, `"x"`, `TRUE`, `#N/A`, `{1,2}`, an omitted argument): nothing to
    /// compute.
    Literal,
    /// A reference or range on its own (`Data!$A$1`, `$B$2:$C$3`): an input, not a computation.
    Reference,
    /// Reads a LET or LAMBDA name it does not bind, so it cannot leave its scope.
    Scoped,
    /// Depends on the cell it sits in (`ROW()`, `COLUMN()`, `SHEET()` without an argument,
    /// `INDIRECT`, `CELL`), so it may give another value in another cell.
    Positional,
    /// Anything else: a call or an operator that a cell of its own computes as the original did.
    Computed,
}

/// One node of a formula's tree.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct Subexpression {
    /// The node's en-US text, without `=`, every reference absolute and sheet-qualified, so it
    /// reads the same cells from any cell of any sheet.
    pub text: String,
    /// What it is.
    pub kind: SubexpressionKind,
    /// The indexes (into the returned list) of the subexpressions directly under it.
    pub children: Vec<usize>,
}

/// Testing hook, not public API (no stability promise): every subexpression of `formula`, parsed
/// as written in `sheet` at 1-based (`row`, `column`) of a workbook with `sheets`, children before
/// their parent, so the whole formula is last. Parentheses, whitespace and an automatic implicit
/// intersection are not nodes of their own: their child stands in for them. A formula that does
/// not parse is one `Computed` node of its own text, as the parser keeps it.
#[doc(hidden)]
#[must_use]
pub fn subexpressions(
    formula: &str,
    sheets: &[String],
    sheet: &str,
    row: i32,
    column: i32,
) -> Vec<Subexpression> {
    let mut parser = new_parser_english(sheets.to_vec(), Vec::new(), HashMap::new());
    let context = CellReferenceRC {
        sheet: sheet.to_string(),
        row,
        column,
    };
    let mut tree = parser.parse(formula.trim_start_matches('='), &context);
    absolutize(&mut tree, &context);

    // Pre-order with each node's nearest kept ancestor; reversed, every child comes before its
    // parent. An explicit stack keeps the walk off the call stack: a formula's tree is untrusted.
    let mut order: Vec<(&Node, Option<usize>)> = Vec::new();
    let mut pending: Vec<(&Node, Option<usize>)> = vec![(&tree, None)];
    let mut children = Vec::new();
    while let Some((node, parent)) = pending.pop() {
        let kept_parent = if is_transparent(node) {
            parent
        } else {
            order.push((node, parent));
            Some(order.len() - 1)
        };
        children.clear();
        node.push_children(&mut children);
        // Pushed in reverse, so the first argument is visited (and kept) first.
        pending.extend(children.iter().rev().map(|child| (*child, kept_parent)));
    }

    let count = order.len();
    // Pre-order index `i` is output index `count - 1 - i`.
    let mut out: Vec<Subexpression> = order
        .iter()
        .rev()
        .map(|(node, _)| Subexpression {
            text: to_english_string(node, &context),
            kind: kind(node),
            children: Vec::new(),
        })
        .collect();
    for (i, (_, parent)) in order.iter().enumerate() {
        if let Some(parent) = parent {
            out[count - 1 - parent].children.push(count - 1 - i);
        }
    }
    // A node is as scoped or positional as anything under it (children come first).
    for i in 0..count {
        let inherited = out[i]
            .children
            .iter()
            .map(|&c| out[c].kind)
            .find(|k| matches!(k, SubexpressionKind::Scoped | SubexpressionKind::Positional));
        if let Some(inherited) = inherited {
            if out[i].kind == SubexpressionKind::Computed {
                out[i].kind = inherited;
            }
        }
    }
    out
}

/// Whether `node` only wraps its child: parentheses, whitespace, an automatic `@`.
fn is_transparent(node: &Node) -> bool {
    matches!(
        node,
        Node::Group { .. }
            | Node::Spaced { .. }
            | Node::ImplicitIntersection {
                automatic: true,
                ..
            }
    )
}

/// What `node` is on its own (before what is under it is taken into account).
fn kind(node: &Node) -> SubexpressionKind {
    match node {
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::EmptyArgKind
        | Node::ArrayKind(_) => SubexpressionKind::Literal,
        Node::UnaryKind { right, .. } if matches!(right.as_ref(), Node::NumberKind(_)) => {
            SubexpressionKind::Literal
        }
        Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::OpRangeKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::DefinedNameKind(_) => SubexpressionKind::Reference,
        Node::NamedVariableKind { .. } => SubexpressionKind::Scoped,
        Node::FunctionKind { kind, args } => match kind {
            Function::Row | Function::Column | Function::Sheet if args.is_empty() => {
                SubexpressionKind::Positional
            }
            Function::Indirect | Function::Cell => SubexpressionKind::Positional,
            _ => SubexpressionKind::Computed,
        },
        // A whole LET or LAMBDA binds its names, but reading them under it is what `Scoped`
        // propagates up: the LET's own value is checked as the formula is, when it is the root.
        _ => SubexpressionKind::Computed,
    }
}

/// Makes every reference in `tree` absolute and names its sheet, as resolved from `context`, so
/// its text reads the same cells from anywhere.
fn absolutize(tree: &mut Node, context: &CellReferenceRC) {
    let mut pending = vec![tree];
    while let Some(node) = pending.pop() {
        match node {
            Node::ReferenceKind {
                sheet_name,
                absolute_row,
                absolute_column,
                row,
                column,
                ..
            }
            | Node::WrongReferenceKind {
                sheet_name,
                absolute_row,
                absolute_column,
                row,
                column,
            } => {
                anchor(absolute_row, row, context.row);
                anchor(absolute_column, column, context.column);
                sheet_name.get_or_insert_with(|| context.sheet.clone());
            }
            Node::RangeKind {
                sheet_name,
                absolute_row1,
                absolute_column1,
                row1,
                column1,
                absolute_row2,
                absolute_column2,
                row2,
                column2,
                ..
            }
            | Node::WrongRangeKind {
                sheet_name,
                absolute_row1,
                absolute_column1,
                row1,
                column1,
                absolute_row2,
                absolute_column2,
                row2,
                column2,
            } => {
                anchor(absolute_row1, row1, context.row);
                anchor(absolute_column1, column1, context.column);
                anchor(absolute_row2, row2, context.row);
                anchor(absolute_column2, column2, context.column);
                sheet_name.get_or_insert_with(|| context.sheet.clone());
            }
            // A 3-D reference names its sheets itself; only its rows and columns are anchored.
            Node::SheetSpanKind { reference, .. } => anchor_span(reference, context),
            other => other.push_children_mut(&mut pending),
        }
    }
}

/// The rows and columns of a 3-D reference's cell or range, anchored at `context`.
fn anchor_span(reference: &mut Node, context: &CellReferenceRC) {
    match reference {
        Node::ReferenceKind {
            absolute_row,
            absolute_column,
            row,
            column,
            ..
        } => {
            anchor(absolute_row, row, context.row);
            anchor(absolute_column, column, context.column);
        }
        Node::RangeKind {
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
            ..
        } => {
            anchor(absolute_row1, row1, context.row);
            anchor(absolute_column1, column1, context.column);
            anchor(absolute_row2, row2, context.row);
            anchor(absolute_column2, column2, context.column);
        }
        _ => {}
    }
}

/// A relative coordinate is stored as an offset from the formula's cell (A1 mode): resolve it.
fn anchor(absolute: &mut bool, coordinate: &mut i32, origin: i32) {
    if !*absolute {
        *coordinate = coordinate.saturating_add(origin);
        *absolute = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nodes(formula: &str) -> Vec<Subexpression> {
        let sheets = vec!["Calc".to_string(), "Data".to_string()];
        subexpressions(formula, &sheets, "Calc", 5, 3)
    }

    fn texts(formula: &str) -> Vec<(String, SubexpressionKind)> {
        nodes(formula)
            .into_iter()
            .map(|n| (n.text, n.kind))
            .collect()
    }

    #[test]
    fn children_come_first_and_references_keep_their_targets() {
        use SubexpressionKind::{Computed, Literal, Reference};
        let got = nodes("=ROUND(SQRT(D6)+Data!A1,2)*$B$2");
        let shown: Vec<(&str, SubexpressionKind)> =
            got.iter().map(|n| (n.text.as_str(), n.kind)).collect();
        // D6 written in C5 is the same cell from anywhere once absolute: Calc!$D$6.
        assert_eq!(
            shown,
            [
                ("Calc!$B$2", Reference),
                ("2", Literal),
                ("Data!$A$1", Reference),
                ("Calc!$D$6", Reference),
                ("SQRT(Calc!$D$6)", Computed),
                ("SQRT(Calc!$D$6)+Data!$A$1", Computed),
                ("ROUND(SQRT(Calc!$D$6)+Data!$A$1,2)", Computed),
                ("ROUND(SQRT(Calc!$D$6)+Data!$A$1,2)*Calc!$B$2", Computed),
            ]
        );
        let root = got.len() - 1;
        let names = |i: usize| -> Vec<&str> {
            got[i]
                .children
                .iter()
                .map(|&c| got[c].text.as_str())
                .collect()
        };
        assert_eq!(
            names(root),
            ["ROUND(SQRT(Calc!$D$6)+Data!$A$1,2)", "Calc!$B$2"]
        );
        assert_eq!(names(5), ["SQRT(Calc!$D$6)", "Data!$A$1"]);
    }

    #[test]
    fn relative_ranges_and_parentheses() {
        let got = texts("=SUM((A1:B2))+1");
        assert_eq!(got[0].0, "1");
        assert!(got.iter().any(|(t, _)| t == "Calc!$A$1:$B$2"), "{got:?}");
        assert_eq!(
            got.last().map(|g| g.0.as_str()),
            Some("SUM((Calc!$A$1:$B$2))+1")
        );
        // The parentheses are not a node of their own.
        assert!(!got.iter().any(|(t, _)| t.starts_with('(')), "{got:?}");
    }

    #[test]
    fn let_and_lambda_names_and_positional_calls_are_marked() {
        use SubexpressionKind::{Computed, Positional, Scoped};
        let got = texts("=LET(x,SQRT(4),x+1)+ROW()*ABS(ROW())+LAMBDA(y,y*2)(3)");
        let of = |t: &str| got.iter().find(|g| g.0 == t).map(|g| g.1);
        assert_eq!(of("SQRT(4)"), Some(Computed));
        assert_eq!(of("x+1"), Some(Scoped));
        assert_eq!(of("y*2"), Some(Scoped));
        assert_eq!(of("ROW()"), Some(Positional));
        assert_eq!(of("ABS(ROW())"), Some(Positional));
        assert_eq!(of("INDIRECT(\"A1\")"), None);
    }

    #[test]
    fn a_formula_that_does_not_parse_never_panics() {
        for formula in ["=SUM(", "=)", "=", "=1+", "=A1:", "=#REF!+1"] {
            let got = nodes(formula);
            assert!(!got.is_empty(), "{formula}");
        }
    }
}
