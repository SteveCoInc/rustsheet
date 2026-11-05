//! The case folds of SEARCH, the exact-match lookups and the criteria (one character to one),
//! beside the collation's own `fold`.

/// The final sigma `ς` as `σ`, every other character as is: Excel's `Σ = σ = ς` in comparisons
/// and in exact MATCH (probes/t3114-collation-beyond-latin1 Calc!B168, B192, B216, D20:E20, H10).
/// The collation key and [`fold_case`] both fold through it.
pub(crate) fn fold_final_sigma(c: char) -> char {
    if c == 'ς' {
        'σ'
    } else {
        c
    }
}

/// Lowercases `text` one character at a time (see [`fold_char`]), never changing its length:
/// the case fold of SEARCH, of exact-match lookup keys (MATCH, VLOOKUP and HLOOKUP with an exact
/// match, XMATCH in wildcard mode and, inferred, XLOOKUP) and of wildcard and D-function prefix
/// criteria.
/// Excel: `SEARCH("x","İx")` is 2 and `SEARCH("i","İx")` is #VALUE!, whereas Rust's
/// `to_lowercase` turns U+0130 into "i" + U+0307 (probes/text-search-casefold-length A1, A2, A8,
/// A12, A13); `MATCH("i",{"İ"},0)` is #N/A and `COUNTIF(r,"?")` counts `İ`
/// (probes/t3133-lookup-criteria-case-fold; docs/excel-semantics/criteria.md "Wildcard escapes,
/// case and trailing tilde").
pub(crate) fn fold_case(text: &str) -> String {
    text.chars().map(fold_char).collect()
}

/// The two long-vowel and iteration mark equalities of exact MATCH, VLOOKUP and HLOOKUP with a
/// literal lookup value, applied to case-folded UTF-16 `units` ([`fold_case`]): a `ー` after a
/// letter or digit that is neither kana nor an ideograph is that character again, and a text
/// starting with `ゝ`, `ゞ`, `ヽ`, `ヾ` or `々` starts with `ー`. Excel: `MATCH("aー",r,0)` finds
/// `"aa"` and `MATCH` of each lone mark finds `"ー"`, though `=`, COUNTIF and XMATCH find `"aー"`
/// equal only to itself (probes/t3275-collation-width-kana Calc!B136:E136, C127:C129). One unit
/// to one; wildcard patterns and criteria keep [`fold_case`] (how the marks match there is
/// unprobed).
pub(crate) fn fold_exact_lookup(units: &[u16]) -> Vec<u16> {
    let mut previous: Option<char> = None;
    let mut folded = Vec::with_capacity(units.len());
    for decoded in char::decode_utf16(units.iter().copied()) {
        let c = decoded.clone().ok();
        let c = match (c, previous) {
            (Some('\u{30fc}'), Some(letter))
                if letter.is_alphanumeric()
                    && !('\u{3005}'..='\u{9fff}').contains(&letter)
                    && !('\u{ff66}'..='\u{ff9f}').contains(&letter) =>
            {
                Some(letter)
            }
            (Some('\u{309d}' | '\u{309e}' | '\u{30fd}' | '\u{30fe}' | '\u{3005}'), None)
                if folded.is_empty() =>
            {
                Some('\u{30fc}')
            }
            _ => c,
        };
        match (c, decoded) {
            (Some(c), _) => {
                let mut buffer = [0u16; 2];
                folded.extend_from_slice(c.encode_utf16(&mut buffer));
            }
            (None, Err(lone)) => folded.push(lone.unpaired_surrogate()),
            (None, Ok(_)) => {}
        }
        previous = c;
    }
    folded
}

/// [`fold_case`] for one character: always exactly one character out. A character whose full
/// lowercase is one character takes it (the Kelvin sign is `k`, `ẞ` is `ß`, `ǅ` and `Ǆ` are `ǆ`,
/// the Ångström sign is `å`, the Ohm sign is `ω`); one whose lowercase is longer (`İ`) stays as
/// it is; final sigma matches sigma in either case. `ſ`/`s`, `µ`/`μ` and `ϐ`/`β` stay apart: each
/// is its own lowercase (SEARCH: probes/text-search-casefold-length A14-A24; lookups and criteria:
/// probes/t3133-lookup-criteria-case-fold).
pub(crate) fn fold_char(c: char) -> char {
    let c = fold_final_sigma(c);
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        _ => c,
    }
}
