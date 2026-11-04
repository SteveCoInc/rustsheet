//! A defined name whose body is one literal or one cell, which a legacy formula reads as that body
//! (docs/excel-semantics/references.md "A defined name takes the `@` by its body", T-3856).

use super::Node;
use crate::expressions::lexer::{Lexer, LexerMode};
use crate::expressions::token::TokenType;
use crate::language::get_default_language;
use crate::locale::get_default_locale;

/// The longest body worth lexing: a text literal of Excel's 255 characters with every quote
/// doubled, or a quoted 31-character sheet name and a cell. A longer body is never one token, so
/// the lexer does not copy it (static analysis asks once per use of the name).
const MAX_SCALAR_BODY: usize = 520;

/// A defined name's body that a legacy formula reads as a plain value, as if written in its place.
pub(super) enum ScalarBody {
    /// A number literal, which counts as a literal INDEX index (`literal_index` in `if_arm.rs`).
    Number(f64),
    /// A text or boolean literal, or a reference to one cell.
    Other,
}

/// What `node`'s body is when `node` is a defined name whose body is one number, text or boolean
/// literal or one cell reference, of any sheet: Excel shows no `@` on such a name at the root, under
/// ABS or as an IF arm (`=nCell`, `=ABS(nNum)`, `=IF(TRUE,nTxt)`), and reads it as an INDEX index
/// as it reads the literal or the cell (`=INDEX(Z1:Z6,nNum)`, but `=@INDEX(Z1:Z6,nCell)` and
/// `=@INDEX(Z1:Z6,nTxt)`), while a one-element array constant `{3}`, a range, the one-cell range
/// `$Z$3:$Z$3` and a computed `1+2` take the `@` (probes/t3856-legacy-at-name-body, excel-graph,
/// T-3856). A negative number, an error literal or a parenthesized body is unprobed and stays a
/// name of unknown shape, and so is a body that is another name. Bodies are stored in English (`Model::parse_defined_names`).
pub(super) fn scalar_body(node: &Node) -> Option<ScalarBody> {
    match single_token_body(node)? {
        TokenType::Number(n) => Some(ScalarBody::Number(n)),
        TokenType::String(_) | TokenType::Boolean(_) | TokenType::Reference { .. } => {
            Some(ScalarBody::Other)
        }
        _ => None,
    }
}

/// Whether `node` is a defined name whose body is one cell or one range reference (`Sheet1!$B$1:$B$6`),
/// so an INDEX over it with literal indexes picks a cell, whatever an array body would give.
pub(super) fn is_reference_body(node: &Node) -> bool {
    matches!(
        single_token_body(node),
        Some(TokenType::Reference { .. } | TokenType::Range { .. })
    )
}

/// The one token of `node`'s body when `node` is a defined name whose body is one token.
fn single_token_body(node: &Node) -> Option<TokenType> {
    let Node::DefinedNameKind((_, _, formula)) = node else {
        return None;
    };
    let body = formula.strip_prefix('=').unwrap_or(formula).trim();
    if body.len() > MAX_SCALAR_BODY {
        return None;
    }
    let mut lexer = Lexer::new(
        body,
        LexerMode::A1,
        get_default_locale(),
        get_default_language(),
    );
    let first = lexer.next_token();
    matches!(lexer.next_token(), TokenType::EOF).then_some(first)
}
