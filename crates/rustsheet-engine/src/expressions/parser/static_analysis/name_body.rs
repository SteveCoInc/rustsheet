//! A defined name whose body is one literal or one cell, which a legacy formula reads as that body
//! (docs/excel-semantics/references.md "A defined name takes the `@` by its body", T-3856).

use super::Node;
use crate::expressions::lexer::{Lexer, LexerMode};
use crate::expressions::token::{OpSum, TokenType};
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

/// What `node`'s body is when `node` is a defined name whose body is one number (negative too),
/// text, boolean or error literal or one cell reference, of any sheet: Excel shows no `@` on such a
/// name at the root, under ABS or as an IF arm (`=nCell`, `=ABS(nNum)`, `=IF(TRUE,nTxt)`, `=nNeg`
/// with nNeg = `-3`, `=ABS(nErr)` with nErr = `#N/A`), and reads it as an INDEX index as it reads
/// the literal or the cell (`=INDEX(Z1:Z6,nNum)`, but `=@INDEX(Z1:Z6,nCell)`,
/// `=@INDEX(Z1:Z6,nTxt)`, `=@INDEX(Z1:Z6,nNeg)` and `=@INDEX(Z1:Z6,nErr)`), while a one-element
/// array constant `{3}`, a range, the one-cell range `$Z$3:$Z$3`, a computed `1+2`, a
/// parenthesized `(3)` and a body that is another name take the `@`
/// (probes/t3856-legacy-at-name-body, T-3856; probes/t4273-legacy-at-name-body-more rows 31-35,
/// T-4273; excel-graph). Spaces around the body are trimmed (`= 3 `, row 34). Bodies are stored in
/// English (`Model::parse_defined_names`).
pub(super) fn scalar_body(node: &Node) -> Option<ScalarBody> {
    let tokens = body_tokens(node, MAX_SCALAR_BODY, 2)?;
    match tokens.as_slice() {
        [TokenType::Number(n)] => Some(ScalarBody::Number(*n)),
        [TokenType::Addition(OpSum::Minus), TokenType::Number(n)] => Some(ScalarBody::Number(-n)),
        [TokenType::String(_)
        | TokenType::Boolean(_)
        | TokenType::Error(_)
        | TokenType::Reference { .. }] => Some(ScalarBody::Other),
        _ => None,
    }
}

/// Whether `node` is a defined name whose body is one cell or one range reference (`Sheet1!$B$1:$B$6`),
/// so an INDEX over it with literal indexes picks a cell, whatever an array body would give.
pub(super) fn is_reference_body(node: &Node) -> bool {
    matches!(
        body_tokens(node, MAX_SCALAR_BODY, 1).as_deref(),
        Some([TokenType::Reference { .. } | TokenType::Range { .. }])
    )
}

/// The longest array-constant body looked at: Excel's 8,192-character formula limit. A longer
/// body is not lexed, and its name stays of unknown shape.
const MAX_ARRAY_BODY: usize = 8192;

/// Whether `node` is a defined name whose body is one array constant (`={1,2;3,4}`), so an INDEX
/// over it with a literal row and column index picks one element: `=INDEX(n2d,1,1)` shows no `@`,
/// under ABS, `+0` or as an IF arm, while `=@INDEX(n2d,1)`, a row, keeps it
/// (probes/t4273-legacy-at-name-body-more row 45, excel-graph, T-4273).
pub(super) fn is_array_body(node: &Node) -> bool {
    body_tokens(node, MAX_ARRAY_BODY, MAX_ARRAY_BODY).is_some_and(|tokens| {
        matches!(
            tokens.as_slice(),
            [TokenType::LeftBrace, inner @ .., TokenType::RightBrace]
                if !inner
                    .iter()
                    .any(|token| matches!(token, TokenType::LeftBrace | TokenType::RightBrace))
        )
    })
}

/// The tokens of `node`'s body when `node` is a defined name whose body, trimmed, is at most
/// `max_len` bytes and lexes to at most `max_tokens` tokens.
fn body_tokens(node: &Node, max_len: usize, max_tokens: usize) -> Option<Vec<TokenType>> {
    let Node::DefinedNameKind((_, _, formula)) = node else {
        return None;
    };
    let body = formula.strip_prefix('=').unwrap_or(formula).trim();
    if body.len() > max_len {
        return None;
    }
    let mut lexer = Lexer::new(
        body,
        LexerMode::A1,
        get_default_locale(),
        get_default_language(),
    );
    let mut tokens = Vec::new();
    loop {
        match lexer.next_token() {
            TokenType::EOF => return Some(tokens),
            TokenType::Illegal(_) => return None,
            _ if tokens.len() == max_tokens => return None,
            token => tokens.push(token),
        }
    }
}
