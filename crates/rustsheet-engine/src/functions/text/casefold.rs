//! The case folds of TEXTSPLIT, TEXTBEFORE and TEXTAFTER, built on SEARCH's one-to-one
//! [`fold_char`] (util/collation.rs, shared with lookups and criteria). Every fold maps one char
//! to exactly one char, so char positions in the folded text are char positions in the original.

use super::string_format::sigma_is_final;
use crate::functions::util::collation::fold_char;

/// The folding `match_mode` 1 of TEXTSPLIT, TEXTBEFORE and TEXTAFTER share (sigma is folded by
/// [`fold_sigma_by_position`]): U+0130 matches `i`, unlike SEARCH; the Kelvin sign does not match
/// `k` and the Angstrom sign (U+212B) matches neither `å` nor `Å`, again unlike SEARCH; `ß` and
/// `ẞ` do not match `ss` (probes/textsplit-casefold-length A2, A6, A7,
/// probes/textbefore-casefold-length A1-A12, probes/t4473-textsplit-fold-gaps A27:A38;
/// docs/excel-semantics/text.md). Elsewhere [`fold_char`] already gives Excel's result: `ſ`
/// matches neither `s` nor `S`, `µ` (U+00B5) neither `μ` nor `Μ`, and each titlecase digraph
/// (`ǅ`, `ǈ`, `ǲ`) matches its upper and lower forms (t4473-textsplit-fold-gaps A39:A74, A81:A104).
fn fold_match_char(c: char) -> char {
    match c {
        '\u{130}' => 'i',
        '\u{212A}' | '\u{212B}' => c,
        _ => fold_char(c),
    }
}

/// Folds `text` char by char with `fold`, except that sigma folds by position, with LOWER's
/// final-sigma rule ([`sigma_is_final`], the 30-character list verified in
/// probes/t3820-lower-sigma-next-char C1:C50): a `Σ` at the end of the text or before one of
/// those characters is `ς`, any other `Σ` is `σ`, and `ς` stays `ς`, so `ς` matches only `ς` and
/// a final `Σ`. TEXTBEFORE and TEXTAFTER (probes/textbefore-casefold-length A17-A25,
/// probes/textbefore-sigma-contexts: a lone `Σ` is `ς` even against a text `Σ` that is `σ`) and
/// TEXTSPLIT (probes/t4473-textsplit-fold-gaps A14:A23 and A75:A80, not SEARCH's `ς` = `σ`)
/// both fold sigma this way.
fn fold_sigma_by_position(text: &str, fold: impl Fn(char) -> char) -> String {
    let mut folded = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        folded.push(match c {
            'Σ' if !sigma_is_final(chars.peek().copied()) => 'σ',
            'Σ' | 'ς' => 'ς',
            _ => fold(c),
        });
    }
    folded
}

/// TEXTSPLIT's `match_mode` 1 folding: [`fold_match_char`] with sigma by position.
pub(super) fn fold_for_split(text: &str) -> String {
    fold_sigma_by_position(text, fold_match_char)
}

/// TEXTBEFORE and TEXTAFTER's `match_mode` 1 folding: [`fold_match_char`] with sigma by
/// position, and `ẞ` does not fold to `ß`.
pub(super) fn fold_for_before_after(text: &str) -> String {
    fold_sigma_by_position(text, |c| match c {
        // The delimiter `ß` does not match `ẞ` in the text (A9), although `ẞ` lowercases to `ß`.
        '\u{1E9E}' => c,
        _ => fold_match_char(c),
    })
}

/// Byte offset of every char of `s`, plus `s.len()` for the end.
pub(super) fn char_offsets(s: &str) -> Vec<usize> {
    s.char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(s.len()))
        .collect()
}
