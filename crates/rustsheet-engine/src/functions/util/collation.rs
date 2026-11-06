//! Excel's text order for `<`, `>`, `=` between texts, sorting, approximate lookups and the
//! text criteria (docs/excel-semantics/coercion.md "Text comparison uses Excel's collation, not
//! code-point order"; probes/text-collation-order, probes/text-order-nbsp-space, excel-graph,
//! T-1469, T-0244).
//!
//! Excel compares texts in six passes, like a word sort, and ignores case at every pass:
//! 1. **Letters** (primary weights): the empty string, the space, CHAR(160), CHAR(9) to
//!    CHAR(13), punctuation ``! " # $ % & ( ) * , . / : ; ? @ [ \ ] ^ _ ` { | } ~``, the Latin-1
//!    marks, quotes and currency signs, `+ < = >`, the other Latin-1 symbols, digits (`¼ ½ ¾`
//!    between `0` and `1`, `¹ ² ³` with `1 2 3`), letters.
//!    An accented letter is its base letter (`"ü"` sorts with `"u"`, before `"x"`); `"æ"` is
//!    `"ae"`, `"ß"` is `"ss"`, `"ĳ"` is `"ij"` and `"œ"` is `"oe"`. Hyphens, apostrophes, the
//!    en and em dash and the control characters other than CHAR(9) to CHAR(13) are skipped
//!    (`"-1" > "0"`).
//! 2. **Accents**, letter by letter: the diacritic weights of Microsoft's Windows sorting weight
//!    table, a letter's own plus those of the combining marks after it, in one wrapping byte
//!    (`"a" < "ª" < "á" < "à" < "â" < "ä" < "ǎ" < "ă" < "ā" < "ã" < "å"`, `"ö" < "ø"`,
//!    `"d" < "đ" < "ð"`; [`MARK_WEIGHT`]). `þ` is not an accent: it is the letters `th`.
//! 3. **Widths**: the superscripts `ª º ¹ ² ³` after the plain letters and digits
//!    (`"1" < "¹"`) and a full-width form after its ASCII form (`"A" < "Ａ"`), before the kana pass
//!    even from a later position (`ぁ&"Ａ" > あ&"A"`; [`widths`], T-4087).
//! 4. **Kana**: a small kana before its letter, the long-vowel mark `ー` and the iteration marks
//!    `ゝ ゞ` after the letter they repeat, `々` before it (`"かぁ" < "かあ" < "かー"`,
//!    `"人々" < "人人"`; [`kana_weights`]).
//! 5. **Kana types**: half-width katakana, katakana, hiragana (`"ｱ" < "ア" < "あ"`;
//!    [`kana_types`]).
//! 6. **Skipped characters**: by position, a later one first, then control characters (by code
//!    point) before apostrophe, hyphen, en dash, em dash, the full-width apostrophe and hyphen
//!    after theirs (`"ab" < "a'b" < "a-b" < "a–b" <
//!    "a—b"`, `"a" < "a'" < "a-" < "'a" < "-a"`, `"1" < "'1" < "-1"`, `"" < CHAR(1) < "–"`).
//!
//! Kana of one syllable share a letter after Cyrillic, in syllabary order, voicing an accent;
//! the long-vowel and iteration marks weigh by the letter before them ([`resolve`];
//! probes/t3275-collation-width-kana, coercion.md "Full-width and half-width forms, kana, voicing
//! and the long-vowel mark").
//!
//! The non-ASCII characters placed here are those of probes/text-collation-unprobed (T-1590,
//! T-1595); where they were probed as single characters only, the module places them by the
//! smallest rule that fits (docs/excel-semantics/coercion.md "Collation equality in lookups and
//! criteria; unprobed characters"). Latin Extended-A and -B, Greek, Cyrillic and Latin Extended
//! Additional letters with a canonical decomposition are their base letter plus their marks, and a
//! combining mark (U+0300 to U+0362) adds its weight to the letter before it
//! (probes/t3114-collation-beyond-latin1, probes/t3154-collation-accents; coercion.md "Letters
//! beyond Latin-1 and combining marks"). Hebrew, Arabic, Devanagari, Thai, Lao, Hangul, the CJK
//! ideographs and astral characters follow Excel's block order ([`script_order`]); Hebrew points,
//! Arabic harakat and Thai tone marks are accents; Arabic-Indic, Devanagari, Thai, Lao and eight
//! other blocks' digits are their digit (T-3895, T-3917); arrows, symbols and dingbats sort after
//! the punctuation (probes/t3276-collation-other-scripts, T-3324). Every other character keeps
//! code-point order after the letters, as before T-0220.
//!
//! **Lone surrogates** (`cmp_text`): a lone surrogate unit weighs as the same unit of a pair does
//! ([`script_order`]) and an astral character weighs its two UTF-16 units, so lone units sort above
//! U+FFFD and a lone high half sorts before a pair it starts, a lone low half after it
//! (docs/excel-semantics/text.md "Lone surrogates through CODE, comparisons, ...").

use std::cell::Cell;
use std::cmp::{Ordering, Reverse};

mod accents;
mod case_fold;
mod compared;
mod kana;
mod lookup_fold;
mod scripts;

use super::collation_tables::{COMBINING_LETTERS, DECOMPOSED, MARK_WEIGHT};
use crate::text::{Char, Text};
use accents::{accents, cmp_accent_units};
pub(crate) use case_fold::{fold_case, fold_char, fold_exact_lookup, fold_final_sigma};
#[cfg(test)]
pub(crate) use compared::take_read_ticks;
use compared::{cmp_text_in, counted, out_of_ticks, CHAR_TICKS, FOLD_TICKS, KEY_TICKS};
pub(crate) use compared::{
    note_compared, restore_compared_ticks, restore_cut_short, set_compared_allowance, stoppable,
    take_compared_ticks, take_cut_short, BYTE_TICKS,
};
use kana::{kana, vowel};
pub(crate) use lookup_fold::fold_lookup;
use lookup_fold::{class_first, is_weightless};
use scripts::{script_element, script_order, symbol, unit, ABOVE_SCRIPTS, SYMBOLS};

/// Orders two texts by the module's six passes, the accents by their wrapped sums: a total
/// order, the one sorts, the sorted lookups and the collation key use. Excel's own `<`, `=`, `>`
/// are [`cmp_text_pairwise`].
#[cfg(test)]
pub(crate) fn cmp_text_excel(left: &str, right: &str) -> Ordering {
    cmp_str(left, right, AccentOrder::Total, &Cell::new(0))
}

/// How the accent pass compares two letters' accents.
#[derive(Clone, Copy)]
enum AccentOrder {
    /// By the wrapped sum of each letter's units: a total order.
    Total,
    /// Excel's comparison of two texts: the units both letters start with are skipped first
    /// ([`cmp_accent_units`]); not transitive.
    Pairwise,
}

/// [`cmp_text_excel`] or [`cmp_text_pairwise`] for well-formed text.
/// `read` counts the ticks of what the passes read ([`counted`]).
fn cmp_str(left: &str, right: &str, order: AccentOrder, read: &Cell<u64>) -> Ordering {
    if left.is_ascii() && right.is_ascii() {
        // ASCII has no accents, so the second pass always ties.
        return ascii_primaries(counted(left.bytes(), read, BYTE_TICKS))
            .cmp(ascii_primaries(counted(right.bytes(), read, BYTE_TICKS)))
            .then_with(|| {
                ascii_skipped(counted(left.bytes(), read, BYTE_TICKS)).cmp(ascii_skipped(counted(
                    right.bytes(),
                    read,
                    BYTE_TICKS,
                )))
            });
    }
    cmp_chars(
        counted(scalars(left), read, CHAR_TICKS),
        counted(scalars(right), read, CHAR_TICKS),
        order,
    )
}

/// Excel's `<`, `=`, `>` before the collation (coercion.md "Format characters, U+200B and the C1
/// controls (T-3662)"; probes/t3662-collation-ignorables, -prefix, -sweep): while both texts' units
/// are below U+007B and equal ignoring ASCII case, a text that runs out first is the smaller, whatever
/// the other has left. So `"ab"&U+00AD > "ab"` and `U+00AD > ""`, though the collation weighs
/// U+00AD nothing; but `"a~"&U+00AD = "a~"` and `"é"&U+00AD = "é"`. `None` once a unit is not
/// below U+007B or the texts differ (the collation decides), or when both run out together. Not
/// transitive (`"ab" < "ab"&U+00AD`, yet both equal U+00AD&`"ab"`), so the total order and the
/// [`CollationKey`] leave it out; it only tells apart texts with equal keys.
fn cmp_prefix(
    mut left: impl Iterator<Item = Char>,
    mut right: impl Iterator<Item = Char>,
) -> Option<Ordering> {
    let fast = |c: Char| match c {
        Char::Scalar(c) if c < '{' => Some(c.to_ascii_lowercase()),
        _ => None,
    };
    loop {
        match (left.next(), right.next()) {
            (None, None) => return None,
            (None, Some(_)) => return Some(Ordering::Less),
            (Some(_), None) => return Some(Ordering::Greater),
            (Some(l), Some(r)) => match (fast(l), fast(r)) {
                (Some(l), Some(r)) if l == r => {}
                _ => return None,
            },
        }
    }
}

/// The characters of well-formed text.
fn scalars(text: &str) -> impl Iterator<Item = Char> + Clone + '_ {
    text.chars().map(Char::Scalar)
}

/// [`cmp_text_excel`] for engine text, a lone surrogate kept as its own unit instead of the
/// lossy U+FFFD (module docs; probes/text-lone-surrogates A10-A24).
pub(crate) fn cmp_text(left: &Text, right: &Text) -> Ordering {
    cmp_text_in(left, right, AccentOrder::Total)
}

/// Orders two texts the way Excel's `<`, `=`, `>` and the text criteria do: as [`cmp_text`],
/// except that the accents of a letter are compared after the units both texts share at its
/// start ([`cmp_accent_units`]). Excel's rule, which is not transitive
/// (docs/excel-semantics/coercion.md "Shared leading units are skipped before the sum",
/// probes/t3181-collation-prefix-callers), so nothing may sort or hash with it: texts it finds
/// equal always have equal [`CollationKey`]s, so a key lookup followed by this test finds them.
pub(crate) fn cmp_text_pairwise(left: &Text, right: &Text) -> Ordering {
    cmp_text_in(left, right, AccentOrder::Pairwise)
}

/// What [`cmp_text`] compares: two texts are equal under it exactly when their keys are equal.
/// The exact-match lookup index of XMATCH and XLOOKUP hashes it (T-0448).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub(crate) struct CollationKey {
    primaries: Vec<u32>,
    accents: Vec<u8>,
    /// The positions and weights of the [`kana_weights`] other than [`NORMAL_KANA`].
    kana: Vec<(usize, u8)>,
    /// The positions and weights of the nonzero [`variants`].
    variants: Vec<(usize, u8)>,
    skipped: Vec<(usize, u8)>,
}

/// The [`CollationKey`] of `text`: its six passes (the widths and kana types in one variant
/// byte), lone surrogates kept as [`cmp_text`] keeps them.
///
/// Past the run's allowance, inside `compared::stoppable` (UNIQUE), it is the empty key without
/// reading the text ([`out_of_ticks`], T-4278): the run's value is then the work budget's error,
/// so the keys it was given never show.
pub(crate) fn collation_key(text: &Text) -> CollationKey {
    if out_of_ticks() {
        return CollationKey::default();
    }
    note_compared(text.len(), KEY_TICKS);
    // Each pass walks the elements, so they are made once (T-4279).
    let kept: Vec<Element> = elements(text.chars_lossless(), None).collect();
    let elements = || kept.iter().copied();
    let sparse = |weights: &mut dyn Iterator<Item = u8>, default: u8| {
        weights
            .enumerate()
            .filter(|&(_, weight)| weight != default)
            .collect()
    };
    let kana = sparse(&mut kana_weights(elements()), NORMAL_KANA);
    let variants = sparse(&mut variants(elements()), 0);
    CollationKey {
        primaries: primaries(elements()).collect(),
        accents: accents(elements()).collect(),
        kana,
        variants,
        skipped: skipped(elements())
            .map(|(Reverse(at), weight)| (at, weight))
            .collect(),
    }
}

/// The passes over two character sequences. The first pass reads the characters (and so counts
/// their ticks, [`counted`]) and keeps each text's elements; it decides at the first difference,
/// and a tie means both texts were read to the end, so the later passes walk the kept elements
/// instead of decomposing every character again (T-4279).
fn cmp_chars<L, R>(left: L, right: R, order: AccentOrder) -> Ordering
where
    L: Iterator<Item = Char> + Clone,
    R: Iterator<Item = Char> + Clone,
{
    let difference = match order {
        AccentOrder::Total => None,
        AccentOrder::Pairwise => Some(first_difference(left.clone(), right.clone())),
    };
    let (mut kept_left, mut kept_right) = (Vec::new(), Vec::new());
    let first = primaries(elements(left, difference).inspect(|&e| kept_left.push(e))).cmp(
        primaries(elements(right, difference).inspect(|&e| kept_right.push(e))),
    );
    if first.is_ne() {
        return first;
    }
    let (left, right) = (kept_left.iter().copied(), kept_right.iter().copied());
    match order {
        AccentOrder::Total => accents(left.clone()).cmp(accents(right.clone())),
        AccentOrder::Pairwise => cmp_accent_units(left.clone(), right.clone()),
    }
    .then_with(|| widths(left.clone()).cmp(widths(right.clone())))
    .then_with(|| kana_weights(left.clone()).cmp(kana_weights(right.clone())))
    .then_with(|| kana_types(left.clone()).cmp(kana_types(right.clone())))
    .then_with(|| skipped(left).cmp(skipped(right)))
}

/// The position of the first character at which two texts differ, ignoring case (the length of
/// the shorter when one starts with the other): where Excel's comparison of the two loses the
/// context of a long-vowel mark after a non-kana letter ([`resolve`]).
fn first_difference(left: impl Iterator<Item = Char>, right: impl Iterator<Item = Char>) -> usize {
    let folded = |c| match c {
        Char::Scalar(c) => Char::Scalar(fold(c)),
        Char::Lone(_) => c,
    };
    left.zip(right)
        .take_while(|&(l, r)| folded(l) == folded(r))
        .count()
}

/// The letter weight of each ASCII byte (0 for hyphen and apostrophe), as [`element`] gives it.
const ASCII_PRIMARY: [u32; 128] = ascii_primary_table();

const fn ascii_primary_table() -> [u32; 128] {
    let mut table = [0; 128];
    let mut byte = 0;
    while byte < 128 {
        table[byte] = match byte as u8 {
            b' ' => SPACE,
            b'\t'..=b'\r' => WHITESPACE_BASE + (byte as u32 - b'\t' as u32),
            b'0'..=b'9' => digit(byte as u32 - b'0' as u32),
            b'a'..=b'z' => LETTER_BASE + (byte as u32 - b'a' as u32),
            b'A'..=b'Z' => LETTER_BASE + (byte as u32 - b'A' as u32),
            // Skipped: hyphen, apostrophe and the other control characters.
            _ => 0,
        };
        byte += 1;
    }
    let mut index = 0;
    while index < PUNCTUATION.len() {
        let c = PUNCTUATION[index] as usize;
        if c < 128 {
            table[c] = PUNCTUATION_BASE + index as u32;
        }
        index += 1;
    }
    table
}

/// The primary weight of digit `value`, leaving room for `¼ ½ ¾` between `0` and `1`.
const fn digit(value: u32) -> u32 {
    DIGIT_BASE + 4 * value
}

/// The first pass over ASCII text, through [`ASCII_PRIMARY`].
fn ascii_primaries(bytes: impl Iterator<Item = u8>) -> impl Iterator<Item = u32> {
    bytes
        .map(|byte| ASCII_PRIMARY[usize::from(byte & 0x7F)])
        .filter(|&weight| weight != 0)
}

/// [`skipped`] of ASCII text, byte-wise: in ASCII every character is one element, a skipped
/// one exactly where [`ASCII_PRIMARY`] is 0 (the hyphen, the apostrophe and the control
/// characters other than CHAR(9) to CHAR(13), weighed as [`element`] weighs them), and none is
/// a mark or weightless (T-4279).
fn ascii_skipped(bytes: impl Iterator<Item = u8>) -> impl Iterator<Item = (Reverse<usize>, u8)> {
    bytes
        .enumerate()
        .filter(|&(_, byte)| ASCII_PRIMARY[usize::from(byte & 0x7F)] == 0)
        .map(|(at, byte)| {
            let weight = match byte {
                b'\'' => APOSTROPHE,
                b'-' => HYPHEN,
                _ => byte & 0x7F,
            };
            (Reverse(at), weight)
        })
}

/// Primary weights: the space, CHAR(160), CHAR(9) to CHAR(13), the Unicode spaces, punctuation,
/// digits, letters, and every other character after the letters by code point.
const SPACE: u32 = 0x40;
const NBSP: u32 = SPACE + 1;
/// CHAR(9), CHAR(10) and CHAR(13) sort above CHAR(160) (probes/text-collation-unprobed); CHAR(11)
/// and CHAR(12), unprobed, are kept with them in code-point order.
const WHITESPACE_BASE: u32 = NBSP + 1;
/// U+2000 to U+200B, after CHAR(9) and before `!`: U+2000 with U+2002, U+2001 with U+2003, then
/// U+2004 to U+200A and the zero-width space U+200B one by one (probes/t4039-collation-space-forms
/// Calc!B3 = 5, B5:B15 = 6, 8, 6, 8, 10 to 16, B18 = 17; probes/t3662-collation-ignorables-sweep
/// G1:G6). Their order against CHAR(10) to CHAR(13) is unprobed but for U+200B's (above both,
/// -sweep G2:G4).
const SPACE_FORMS: u32 = WHITESPACE_BASE + 5;
const PUNCTUATION_BASE: u32 = SPACE_FORMS + 10;
/// The symbols after [`PUNCTUATION`] ([`symbol`]).
const SYMBOL_BASE: u32 = PUNCTUATION_BASE + PUNCTUATION.len() as u32;
/// `•` sorts after every symbol of [`symbol`] and before the digits (`"⇿" < "•"`, `"☀" < "•" <
/// "0"`; probes/t3559-collation-unprobed-blocks ranks 22 to 29, G16, G18).
const BULLET: u32 = SYMBOL_BASE + SYMBOLS;
/// The C1 controls U+0080 to U+009F sort after `•` and before the digits, each by its code point
/// (probes/t4270-collation-c1-controls Calc!B1:B36, all 32; probes/t4039-collation-space-forms
/// Calc!B19:B23, J19:L23; probes/t3662-collation-ignorables-sweep K1:L16).
const C1_BASE: u32 = BULLET + 1;
const DIGIT_BASE: u32 = 0x800;
const _: () = assert!(C1_BASE + 0x20 <= DIGIT_BASE);
/// `∞` sorts between `"9"` and `"a"` (probes/t3276-collation-other-scripts rank A91 = 24).
const INFINITY: u32 = digit(9) + 4;
const LETTER_BASE: u32 = 0x840;
const OTHER_BASE: u32 = 0x900;

/// The primary weight of a character after the letters: by its place in [`script_order`], two
/// apart so that a weight fits between two neighbours ([`LONE_MARK`]).
fn other(c: char) -> u32 {
    OTHER_BASE + 2 * script_order(u32::from(c))
}

/// A long-vowel or iteration mark with no letter before it (`"ー"`, `"ゝ"`, `"ゞ"`, `"々"`, `"ーa"`):
/// all equal (probes/t3275-collation-width-kana Calc!B119 = 4, probes/t3275-collation-width-kana-pairs
/// AC25, AC37), and above every other character: the BMP and plane 2 ideographs, Extension A, the
/// compatibility ideographs, plane 3, Hangul, private use, U+FFFD and U+10FFFD
/// (probes/t4258-collation-lone-mark-astral C1:C90, C104:C105). What follows the mark does not
/// change that (C91:C103), so `"々ヲ" > "𠀀"` (probes/t3465-collation-holdout C313:D313).
const LONE_MARK: u32 = OTHER_BASE + 2 * ABOVE_SCRIPTS;

/// Punctuation and symbols in Excel's order: the ASCII ones from probes/text-collation-order
/// ranks 7-36, the others from probes/text-collation-unprobed, which ranks them against `!`, `~`,
/// `+` and `0` only. So the Latin-1 marks, quotes and currency signs go between `~` and `+`, and
/// the symbols after `+`, placed after `>` (their order against `< = >` is unprobed). The middle dot
/// `·` sits between `µ` and `…` and is neither skipped nor equal to `""`
/// (probes/t3896-collation-middle-dot Calc!A29 = 31, A54 = 30, A28 = 32). The CJK marks `、 。`
/// sort between `%` and `~`, `「` after `~`, and the currency signs `₩ € ₹` in that order before
/// `+` (probes/t3559-collation-unprobed-blocks ranks 7 to 15); their places against the unprobed
/// ASCII marks and the Latin-1 signs are UNVERIFIED. [`punctuation`] lists the marks above `€`.
const PUNCTUATION: [char; 64] = [
    '!', '"', '#', '$', '%', '&', '(', ')', '*', ',', '.', '/', ':', ';', '?', '@', '[', '\\', ']',
    '^', '_', '`', '{', '|', '}', '\u{3001}', '\u{3002}', '~', '\u{300c}', '¡', '¦', '¨', '¯', '´',
    '¸', '¿', '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}', '¢', '£', '¤', '¥', '\u{20a9}', '€',
    '\u{20b9}', '+', '<', '=', '>', '±', '«', '»', '×', '÷', '§', '©', '¬', '®', '°', 'µ',
    '\u{b7}', '…',
];

/// What one case-folded character contributes to each pass.
#[derive(Clone, Copy)]
enum Element {
    /// A skipped character (module docs): no letter weight, only a last-pass weight.
    Skipped(u8),
    /// One or two letter weights (two for `æ`, `ß`), the accent of the character, its kana-pass
    /// weight ([`kana_weights`]) and its variant-pass weight ([`variants`]).
    Weights {
        primary: [u32; 2],
        accent: u8,
        kana: u8,
        variant: u8,
    },
    /// A combining mark: no letter weight; it adds its diacritic weight to the letter before it.
    Mark(u8),
    /// A weightless character (T-3904): no weight in any pass, but it ends the letter before it,
    /// so a mark after it is a lone mark (`"a"&U+00AD&U+0301` is between `"a"` and `"á"`;
    /// probes/t4060-collation-open-choices Calc!B50:B63, T-4072).
    Weightless,
}

/// Diacritic (accent) weights, from Microsoft's Windows sorting weight table (the DW column; see
/// [`MARK_WEIGHT`]): a letter without an accent weighs 2, an accented letter 2 plus its marks'
/// weights. So `"a" < "á" < "à" < "â" < "ä" < "ǎ" < "ă" < "ā" < "ã" < "å" < "ą"` (2, 14, 15, 18,
/// 19, 20, 21, 23, 25, 26, 27; probes/t3154-collation-accents Calc!A1:A12). The letters with a
/// weight of their own (ordinal indicators, stroke, eth, dotless i) take theirs from the table too:
/// `"a" < "ª" < "á"`, `"ö" < "ø"`, `"d" < "đ" < "ð"` (probes/text-collation-unprobed).
const NO_ACCENT: u8 = 2;
/// `ª` and `º` (they also differ from `a`, `o` in the variant pass, [`variants`]).
const ORDINAL: u8 = 3;
const ACUTE: u8 = accent('\u{301}');
const GRAVE: u8 = accent('\u{300}');
const CIRCUMFLEX: u8 = accent('\u{302}');
const DIAERESIS: u8 = accent('\u{308}');
const TILDE: u8 = accent('\u{303}');
const RING: u8 = accent('\u{30a}');
const CEDILLA: u8 = accent('\u{327}');
/// `ı`, `đ`, `ł`, `ø`, `ð`: Windows table weights 4, 30, 31, 33, 104.
const DOTLESS: u8 = 4;
const D_STROKE: u8 = 30;
const L_STROKE: u8 = 31;
const O_STROKE: u8 = 33;
const ETH: u8 = 104;
/// The weight of the letters U+0363 to U+036F ([`COMBINING_LETTERS`]).
const COMBINING_LETTER: u8 = 3;

/// The accent of a letter carrying combining mark `mark` alone.
const fn accent(mark: char) -> u8 {
    NO_ACCENT + MARK_WEIGHT[mark as usize - 0x300]
}

/// The letter that `c` (U+0363 to U+036F) sorts as ([`COMBINING_LETTERS`]).
fn combining_letter(c: char) -> Option<char> {
    let index = usize::try_from(u32::from(c).checked_sub(0x363)?).ok()?;
    COMBINING_LETTERS.get(index).copied()
}

/// The diacritic weight combining mark `c` (U+0300 to U+0362) adds to the letter before it.
fn mark_weight(c: char) -> Option<u8> {
    let index = usize::try_from(u32::from(c).checked_sub(0x300)?).ok()?;
    MARK_WEIGHT.get(index).copied()
}

/// The variant pass weight of the superscripts `ª º ¹ ² ³`. The Windows table gives them
/// the case weight 14 (superscript and compatibility bits) where a plain letter or digit has 2;
/// Excel ignores the upper-case bit but keeps this one: `"¹" > "1"` with equal letters and
/// accents (probes/text-collation-longer), and `"ª"&UNICHAR(769)` (3 + 12 = 15) sorts after `"à"`
/// (15) (probes/t3154-collation-accent-weights Calc!A114, A10). `ς` has case weight 10 in the
/// table but equals `σ` in Excel (probes/t3114-collation-beyond-latin1), so it keeps weight 0.
const SUPERSCRIPT: u8 = 1;

/// The variant pass weight of a full-width form (U+FF01 to U+FF5E, the ideographic space U+3000):
/// it weighs as its ASCII form, just after it and never equal (`"0" < "０" < "1"`,
/// `"Ａ"="A"` FALSE), after the accents (`"Ａ" < "á"`), left to right (`"AＢ" < "Ａb"`)
/// (probes/t3275-collation-width-kana Calc!G1, G3, G6; pairs H6). Its order against
/// [`SUPERSCRIPT`] is unprobed.
const FULL_WIDTH: u8 = 2;
/// The accent weight of U+202F: it is the space at the first pass (`U+202F&"a" < " b"`,
/// probes/t4060-collation-open-choices Calc!B2, K2) and differs from it at the accent pass, letter
/// by letter (`U+202F&"a" > " á"`, `"a"&U+202F < "á"&" "`; probes/t4086-collation-later-passes
/// Calc!B1, B46:B48), so it sorts after U+3000, whose difference is its width, and before CHAR(160)
/// (probes/t4039-collation-space-forms Calc!B4, B16, B2 = 2, 3, 4; T-4087). Its value is 10: U+202F
/// equals a space with 88 U+030D (2 + 3 * 88 = 266, 10 in the wrapping byte), sorts above 9, 7 and
/// `" "&U+030D` (5), and below 11, 13 and every other space with one mark (14 and up)
/// (probes/t4153-narrow-nbsp-accent Calc!A1:E15, B30:D37; T-4153).
const NARROW_NBSP: u8 = 10;

/// The variant byte of a kana holds its kana type from here up; below it, a width or superscript
/// weight. Excel weighs the two in different passes: width and superscripts before the kana pass,
/// even from a later position (`"Ａ"&ぁ > "A"&あ`, `ぁ&"Ａ" > あ&"A"`, `"¹"&ぁ > "1"&あ`), the kana type
/// after it (`"ア"&ぁ < "あ"&あ`) (probes/t4086-collation-later-passes Calc!B30:B45; T-4087).
const KANA_TYPE: u8 = 0x40;

/// The kana types: half-width katakana < katakana < hiragana, left to right, after the kana pass
/// (`"ｱ" < "ア" < "あ"`, `"ｱあ" < "アあ" < "あア" < "ああ"`; probes/t3275-collation-width-kana
/// Calc!G12:G14, ranks A90:A95).
const HALF_WIDTH_KANA: u8 = KANA_TYPE;
const KATAKANA: u8 = KANA_TYPE + 1;
const HIRAGANA: u8 = KANA_TYPE + 2;

/// The kana pass weights, after the widths and before the kana types: a small kana before its
/// letter, the long-vowel mark `ー` and the kana iteration marks `ゝ ゞ` after the letter they
/// repeat, and `々` before it (`"かぁ" < "カア" < "かあ" < "カー" < "かー"`, `"かか" < "かゝ"`,
/// `"人々" < "人人"`; probes/t3275-collation-width-kana ranks A99, A101:A102, A120:A123, A130:A131,
/// pairs AB26, AC27). Every other letter weighs [`NORMAL_KANA`].
const SMALL_KANA: u8 = 1;
const NORMAL_KANA: u8 = 2;
const LONG_VOWEL: u8 = 3;
const KANA_REPEAT: u8 = 3;
const KANJI_REPEAT: u8 = 1;

/// The diacritic weights the voicing marks add (U+3099, U+309B, U+FF9E voiced; U+309A, U+309C,
/// U+FF9F semi-voiced), and that a voiced kana carries: an accent on its unvoiced letter
/// (`"か" < "が"`, `"は" < "ば" < "ぱ"`, `"があ" < "かい"`; `"か"&UNICHAR(12441)` = `"が"`;
/// probes/t3275-collation-width-kana Calc!G20:G24, B96:B97). The values follow the order; their
/// size against the Latin marks is unprobed.
const VOICED: u8 = 1;
const SEMI_VOICED: u8 = 2;

/// Last-pass weights of the skipped characters: control characters below the apostrophe (none
/// probed against it; `CHAR(1) < CHAR(31) < CHAR(127) < "–"`), then `' ＇ - － – —` (the
/// full-width forms just after theirs: `"'" < "＇" < "-" < "－"`, `"a＇b" < "a-b" < "a－b"`,
/// probes/t3275-collation-width-kana ranks A5, A6, A58:A62; their order against the dashes is
/// unprobed).
const APOSTROPHE: u8 = 0x80;
const FULL_WIDTH_APOSTROPHE: u8 = APOSTROPHE + 1;
const HYPHEN: u8 = APOSTROPHE + 2;
const FULL_WIDTH_HYPHEN: u8 = APOSTROPHE + 3;
const EN_DASH: u8 = APOSTROPHE + 4;
const EM_DASH: u8 = APOSTROPHE + 5;

fn letter(base: char) -> u32 {
    LETTER_BASE + (u32::from(base) - u32::from('a'))
}

fn accented(base: char, accent: u8) -> Element {
    Element::Weights {
        primary: [letter(base), 0],
        accent,
        kana: NORMAL_KANA,
        variant: 0,
    }
}

fn single(primary: u32) -> Element {
    Element::Weights {
        primary: [primary, 0],
        accent: NO_ACCENT,
        kana: NORMAL_KANA,
        variant: 0,
    }
}

/// A superscript (`ª º ¹ ² ³`): `primary` with `accent` and the [`SUPERSCRIPT`] variant weight.
fn superscript(primary: u32, accent: u8) -> Element {
    Element::Weights {
        primary: [primary, 0],
        accent,
        kana: NORMAL_KANA,
        variant: SUPERSCRIPT,
    }
}

/// A character that sorts as the two letters `first`, `second` (`æ` = `ae`).
fn expansion(first: char, second: char) -> Element {
    Element::Weights {
        primary: [letter(first), letter(second)],
        accent: NO_ACCENT,
        kana: NORMAL_KANA,
        variant: 0,
    }
}

/// The primary weight of a punctuation mark or symbol of [`PUNCTUATION`]. Every mark but `₹`,
/// `、`, `。` and `「` is at or below `€`, so text in other scripts skips the scan.
fn punctuation(c: char) -> Option<u32> {
    if c > '€' && !matches!(c, '\u{20b9}' | '\u{3001}' | '\u{3002}' | '\u{300c}') {
        return None;
    }
    PUNCTUATION
        .iter()
        .position(|&mark| mark == c)
        .and_then(|index| u32::try_from(index).ok())
        .map(|index| PUNCTUATION_BASE + index)
}

/// `c` with case removed: ASCII and Latin-1 letters directly, others through `to_lowercase`.
/// A member of one of Excel's wider classes ([`class_first`]: `ϵ` with `ε`, `ϑ` with `ϴ`, `ϳ`
/// with `Ͷ`, U+1FBE with `ι`, the Greek Extended psili+varia and dasia+oxia letters, ...) is its
/// class's first member, so all of them weigh alike and tie under `<`, as Excel's do
/// (probes/t4013-class-equality-order Calc!C1:E18; probes/t3584-match-fold-weightless Calc!B21:F21,
/// P7, P9; T-4013). A supplementary
/// character keeps its case: Excel's `=` keeps Deseret U+10400 apart from U+10428 (Calc!B25).
fn fold(c: char) -> char {
    match c {
        'A'..='Z' => c.to_ascii_lowercase(),
        'À'..='Þ' if c != '×' => char::from_u32(u32::from(c) + 0x20).unwrap_or(c),
        // Characters that are their own lowercase and in no class ([`class_first`]): the
        // lower-case Latin-1 letters and `÷`, the kana, CJK and Hangul blocks, and the astral
        // characters (a fast path, T-4279; `fold_fast_paths_are_identities` checks it).
        _ if c.is_ascii() || u32::from(c) > 0xFFFF => c,
        'ß'..='ÿ' | '\u{3040}'..='\u{9fff}' | '\u{ac00}'..='\u{d7a3}' => c,
        _ => {
            let lower = |c: char| fold_final_sigma(c.to_lowercase().next().unwrap_or(c));
            let folded = lower(c);
            class_first(c)
                .or_else(|| class_first(folded))
                .map_or(folded, lower)
        }
    }
}

fn element(c: Char) -> Element {
    let c = match c {
        Char::Scalar(c) => fold(c),
        Char::Lone(lone) => return single(unit(lone)),
    };
    if let Some(element) = script_element(c) {
        return element;
    }
    match c {
        '\'' => Element::Skipped(APOSTROPHE),
        '-' => Element::Skipped(HYPHEN),
        '\u{2013}' => Element::Skipped(EN_DASH),
        '\u{2014}' => Element::Skipped(EM_DASH),
        '\u{ff07}' => Element::Skipped(FULL_WIDTH_APOSTROPHE),
        '\u{ff0d}' => Element::Skipped(FULL_WIDTH_HYPHEN),
        '\u{3000}' => full_width(single(SPACE)),
        '\u{ff01}'..='\u{ff5e}' => char::from_u32(u32::from(c) - 0xFEE0).map_or_else(
            || single(other(c)),
            |ascii| full_width(element(Char::Scalar(ascii))),
        ),
        '\u{3099}' | '\u{309b}' | '\u{ff9e}' => Element::Mark(VOICED),
        '\u{309a}' | '\u{309c}' | '\u{ff9f}' => Element::Mark(SEMI_VOICED),
        // A control character other than CHAR(9) to CHAR(13): 0x00-0x1F, and DEL after them.
        _ if c.is_ascii() && ASCII_PRIMARY[c as usize] == 0 => {
            Element::Skipped(u8::try_from(c).unwrap_or(0x7F))
        }
        _ if c.is_ascii() => single(ASCII_PRIMARY[c as usize]),
        '\u{a0}' => single(NBSP),
        '\u{202f}' => Element::Weights {
            primary: [SPACE, 0],
            accent: NARROW_NBSP,
            kana: NORMAL_KANA,
            variant: 0,
        },
        // U+205F equals the space (probes/t4039-collation-space-forms Calc!C1, H17).
        '\u{205f}' => single(SPACE),
        '\u{2000}' | '\u{2002}' => single(SPACE_FORMS),
        '\u{2001}' | '\u{2003}' => single(SPACE_FORMS + 1),
        '\u{2004}'..='\u{200b}' => single(SPACE_FORMS + 2 + (u32::from(c) - 0x2004)),
        '\u{80}'..='\u{9f}' => single(C1_BASE + (u32::from(c) - 0x80)),
        '\u{363}'..='\u{36f}' => combining_letter(c)
            .map_or_else(|| single(other(c)), |base| accented(base, COMBINING_LETTER)),
        '\u{300}'..='\u{362}' => Element::Mark(mark_weight(c).unwrap_or(0)),
        '\u{221e}' => single(INFINITY),
        // `™` sorts after `"t"`, `"tm"` and `"tn"` and before `"z"` (probes/t3559-collation-unprobed-blocks
        // G1:G5, ranks 46 to 50), so it is not `tm` with a variant weight: placed as `t` and a weight
        // above every letter, before `"u"` (where it sits between `"tn"` and `"z"` is unprobed).
        '\u{2122}' => Element::Weights {
            primary: [letter('t'), letter('z') + 1],
            accent: NO_ACCENT,
            kana: NORMAL_KANA,
            variant: 0,
        },
        // `℃` and `№` sort after `"a"` and before `"tm"` (ranks 43 to 46): placed as the letters `c`
        // and `no` of their compatibility forms with the superscript variant (UNVERIFIED).
        '\u{2103}' => superscript(letter('c'), NO_ACCENT),
        '\u{2116}' => Element::Weights {
            primary: [letter('n'), letter('o')],
            accent: NO_ACCENT,
            kana: NORMAL_KANA,
            variant: SUPERSCRIPT,
        },
        '\u{2022}' => single(BULLET),
        '¼' => single(digit(0) + 1),
        '½' => single(digit(0) + 2),
        '¾' => single(digit(0) + 3),
        '¹' => superscript(digit(1), NO_ACCENT),
        '²' => superscript(digit(2), NO_ACCENT),
        '³' => superscript(digit(3), NO_ACCENT),
        'ª' => superscript(letter('a'), ORDINAL),
        'º' => superscript(letter('o'), ORDINAL),
        'á' => accented('a', ACUTE),
        'à' => accented('a', GRAVE),
        'â' => accented('a', CIRCUMFLEX),
        'ä' => accented('a', DIAERESIS),
        'å' => accented('a', RING),
        'ã' => accented('a', TILDE),
        'ç' => accented('c', CEDILLA),
        'đ' => accented('d', D_STROKE),
        'ð' => accented('d', ETH),
        'ı' => accented('i', DOTLESS),
        'ł' => accented('l', L_STROKE),
        // Excel: þ is the expansion `th` (probes/text-collation-longer Calc!A74-A76).
        'þ' => expansion('t', 'h'),
        'é' => accented('e', ACUTE),
        'è' => accented('e', GRAVE),
        'ê' => accented('e', CIRCUMFLEX),
        'ë' => accented('e', DIAERESIS),
        'í' => accented('i', ACUTE),
        'ì' => accented('i', GRAVE),
        'î' => accented('i', CIRCUMFLEX),
        'ï' => accented('i', DIAERESIS),
        'ñ' => accented('n', TILDE),
        'ó' => accented('o', ACUTE),
        'ò' => accented('o', GRAVE),
        'ô' => accented('o', CIRCUMFLEX),
        'ö' => accented('o', DIAERESIS),
        'õ' => accented('o', TILDE),
        'ø' => accented('o', O_STROKE),
        'ú' => accented('u', ACUTE),
        'ù' => accented('u', GRAVE),
        'û' => accented('u', CIRCUMFLEX),
        'ü' => accented('u', DIAERESIS),
        'ý' => accented('y', ACUTE),
        'ÿ' => accented('y', DIAERESIS),
        'æ' => expansion('a', 'e'),
        'ß' => expansion('s', 's'),
        'ĳ' => expansion('i', 'j'),
        // Between `ø` and `s`; whether it equals `oe` is unprobed, so it is kept with `æ`.
        'œ' => expansion('o', 'e'),
        _ if u32::from(c) > 0xFFFF => {
            let mut units = [0u16; 2];
            c.encode_utf16(&mut units);
            Element::Weights {
                primary: [unit(units[0]), unit(units[1])],
                accent: NO_ACCENT,
                kana: NORMAL_KANA,
                variant: 0,
            }
        }
        _ => kana(c).unwrap_or_else(|| {
            single(
                punctuation(c)
                    .or_else(|| symbol(c))
                    .unwrap_or_else(|| other(c)),
            )
        }),
    }
}

/// `c` case-folded and canonically decomposed through [`DECOMPOSED`]: one to three characters.
fn decompose(c: Char) -> impl Iterator<Item = Char> + Clone {
    let folded = match c {
        // Excel: `İ` is `i` + U+0307 (its canonical decomposition, lower-cased), not `i` or `I`:
        // `XMATCH("i"&UNICHAR(775),{"İ"})` is 1 and `XMATCH("I",{"İ"})` is #N/A
        // (probes/t3133-lookup-criteria-case-fold Calc!J1, J3, J33).
        Char::Scalar('\u{130}') => {
            return [Char::Scalar('i'), Char::Scalar('\u{307}'), c]
                .into_iter()
                .take(2)
        }
        Char::Scalar(c) => fold(c),
        // Every arm yields a `[Char; 3]` iterator so that the function has one return type.
        Char::Lone(_) => return [c, c, c].into_iter().take(1),
    };
    let entry = if ('\u{100}'..='\u{45f}').contains(&folded)
        || ('\u{1e00}'..='\u{1eff}').contains(&folded)
    {
        DECOMPOSED
            .binary_search_by_key(&folded, |&(letter, ..)| letter)
            .ok()
            .and_then(|index| DECOMPOSED.get(index))
    } else {
        None
    };
    match entry {
        Some(&(_, base, first, second)) => {
            let units = [
                Char::Scalar(base),
                Char::Scalar(first),
                Char::Scalar(second),
            ];
            units.into_iter().take(if second == '\0' { 2 } else { 3 })
        }
        None => [Char::Scalar(folded); 3].into_iter().take(1),
    }
}

/// The [`Element`] of each decomposed character of `text`, the long-vowel and iteration marks
/// resolved against the letter before them ([`resolve`]). `difference` is the position where a
/// pairwise comparison's two texts first differ ([`first_difference`]), `None` for the total order.
fn elements(
    text: impl Iterator<Item = Char> + Clone,
    difference: Option<usize>,
) -> impl Iterator<Item = Element> + Clone {
    text.enumerate()
        // Excel: a weightless character (T-3584's table) has no weight in any pass
        // ([`Element::Weightless`], before `decompose` could case-fold it into a weighted letter);
        // where it still counts is [`cmp_prefix`]'s (coercion.md "Format characters, U+200B and
        // the C1 controls (T-3662)", T-3904).
        .flat_map(|(at, c)| {
            let weightless = matches!(c, Char::Scalar(c) if is_weightless(c));
            decompose(c)
                .take(if weightless { 0 } else { 3 })
                .map(move |c| (at, Some(c)))
                .chain(weightless.then_some((at, None)))
        })
        .scan(None, move |previous, (at, c)| {
            let Some(c) = c else {
                return Some(Element::Weightless);
            };
            let element = resolve(c, *previous, difference == Some(at));
            if let Element::Weights { primary, .. } = element {
                *previous = Some(if primary[1] == 0 {
                    primary[0]
                } else {
                    primary[1]
                });
            }
            Some(element)
        })
}

/// The [`Element`] of `c`, a long-vowel or iteration mark weighed by `previous`, the last letter
/// weight before it (probes/t3275-collation-width-kana, probes/t3275-collation-width-kana-pairs;
/// docs/excel-semantics/coercion.md "Full-width and half-width forms, kana, voicing and the
/// long-vowel mark"):
/// - `ー` (and the half-width `ｰ`) after a kana is that kana's vowel, just after the written vowel
///   (`"かあ" < "かー" < "かい"`, `"きい" < "きー"`).
/// - `ゝ ヽ` repeat the letter before them and `ゞ ヾ` repeat it voiced, just after the written
///   repeat (`"かか" < "かゝ" < "かが" < "かゞ"`); `々` repeats it just before
///   (`"人" < "人々" < "人人"`).
/// - `ー` after any other letter is that letter again (`"aー"` sorts as `"aa"`: below `"Ａa"`),
///   except at `lost`, the first character where the two compared texts differ: there it has no
///   letter before it (`"aー" > "ab"`, `"aー" > "AＢ"`). So Excel's `<` is not transitive there
///   (pairs E8, F4, H8; the collation key and the sorts keep `"aa"`).
/// - With no letter before them (or lost) the marks are [`LONE_MARK`].
fn resolve(c: Char, previous: Option<u32>, lost: bool) -> Element {
    let Char::Scalar(mark) = c else {
        return element(c);
    };
    let repeat = |primary, accent, kana, variant| Element::Weights {
        primary: [primary, 0],
        accent,
        kana,
        variant,
    };
    let lone = single(LONE_MARK);
    match (mark, previous) {
        ('\u{30fc}' | '\u{ff70}', Some(previous)) => {
            let variant = if mark == '\u{30fc}' {
                KATAKANA
            } else {
                HALF_WIDTH_KANA
            };
            match vowel(previous) {
                Some(vowel) => repeat(vowel, NO_ACCENT, LONG_VOWEL, variant),
                None if lost => lone,
                None => single(previous),
            }
        }
        ('\u{309d}', Some(previous)) => repeat(previous, NO_ACCENT, KANA_REPEAT, HIRAGANA),
        ('\u{309e}', Some(previous)) => repeat(previous, NO_ACCENT + VOICED, KANA_REPEAT, HIRAGANA),
        ('\u{30fd}', Some(previous)) => repeat(previous, NO_ACCENT, KANA_REPEAT, KATAKANA),
        ('\u{30fe}', Some(previous)) => repeat(previous, NO_ACCENT + VOICED, KANA_REPEAT, KATAKANA),
        ('\u{3005}', Some(previous)) => repeat(previous, NO_ACCENT, KANJI_REPEAT, 0),
        (
            '\u{30fc}' | '\u{ff70}' | '\u{309d}' | '\u{309e}' | '\u{30fd}' | '\u{30fe}'
            | '\u{3005}',
            None,
        ) => lone,
        _ => element(c),
    }
}

/// `element` as a full-width form: the same weights, [`FULL_WIDTH`] in the variant pass.
fn full_width(element: Element) -> Element {
    match element {
        Element::Weights {
            primary,
            accent,
            kana,
            ..
        } => Element::Weights {
            primary,
            accent,
            kana,
            variant: FULL_WIDTH,
        },
        other => other,
    }
}

/// The first pass: letter weights, hyphens, apostrophes and combining marks skipped.
fn primaries(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u32> {
    elements
        .flat_map(|element| match element {
            Element::Skipped(_) | Element::Mark(_) | Element::Weightless => [0, 0],
            Element::Weights { primary, .. } => primary,
        })
        .filter(|&weight| weight != 0)
}

/// The kana pass, after the accents: one weight per letter weight of the first pass,
/// [`NORMAL_KANA`] but for small kana and the long-vowel and iteration marks.
fn kana_weights(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    letter_weights(elements, |kana, _| kana, NORMAL_KANA)
}

/// The variant byte as one weight per letter weight of the first pass: what the collation key
/// keeps (the [`widths`] and [`kana_types`] together).
fn variants(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    letter_weights(elements, |_, variant| variant, 0)
}

/// The width pass, after the accents and before the kana pass: one weight per letter weight of the
/// first pass, [`SUPERSCRIPT`] for `ª º ¹ ² ³`, [`FULL_WIDTH`] for the full-width forms, the other
/// scripts' digit variants, and 0 otherwise (a kana's type is [`kana_types`]').
fn widths(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    letter_weights(
        elements,
        |_, variant| if variant < KANA_TYPE { variant } else { 0 },
        0,
    )
}

/// The kana type pass, after the kana pass: one weight per letter weight of the first pass, a
/// kana's type ([`HALF_WIDTH_KANA`], [`KATAKANA`], [`HIRAGANA`]) and 0 otherwise.
fn kana_types(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    letter_weights(
        elements,
        |_, variant| {
            variant
                .checked_sub(KANA_TYPE)
                .map_or(0, |kana_type| kana_type + 1)
        },
        0,
    )
}

/// One weight per letter weight of the first pass: `pick` of the letter's kana and variant
/// weights, and `second` for the second letter of an expansion.
fn letter_weights(
    elements: impl Iterator<Item = Element>,
    pick: impl Fn(u8, u8) -> u8,
    second: u8,
) -> impl Iterator<Item = u8> {
    elements
        .flat_map(move |element| match element {
            Element::Weights {
                primary,
                kana,
                variant,
                ..
            } => [
                Some(pick(kana, variant)),
                (primary[1] != 0).then_some(second),
            ],
            Element::Skipped(_) | Element::Mark(_) | Element::Weightless => [None, None],
        })
        .flatten()
}

/// The last pass: each hyphen or apostrophe as (position, weight), a later position first.
/// Positions count letters and skipped characters, not combining marks, so a decomposed letter
/// keeps the position of its precomposed form. The marks right after a skipped character raise
/// its weight by theirs (`"a'b" < "a-b" < "a'"&UNICHAR(769)&"b" < "a-"&UNICHAR(769)&"b"`:
/// probes/t3304-collation-multi-letter-accents G4). That is the smallest fit for the probed
/// U+0301; other marks and runs of several are unprobed.
fn skipped(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = (Reverse<usize>, u8)> {
    let mut elements = elements.peekable();
    let mut position = 0;
    std::iter::from_fn(move || loop {
        match elements.next()? {
            Element::Skipped(mut weight) => {
                // Excel: a weightless character does not end a skipped character's marks
                // (`"a-"&U+00AD&U+0301 = "a-"&U+0301`; probes/t4086-collation-later-passes
                // Calc!B10:B22, B50:B52; T-4087).
                while let Some(element) = elements
                    .next_if(|element| matches!(element, Element::Mark(_) | Element::Weightless))
                {
                    if let Element::Mark(mark) = element {
                        weight = weight.wrapping_add(mark);
                    }
                }
                position += 1;
                return Some((Reverse(position - 1), weight));
            }
            Element::Weights { .. } => position += 1,
            Element::Mark(_) | Element::Weightless => {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{ascii_skipped, class_first, elements, fold, scalars, skipped};

    /// `fold`'s fast paths (T-4279) return what its general arm would: each character there is
    /// its own lowercase and in no class.
    #[test]
    fn fold_fast_paths_are_identities() {
        let ranges = [
            ('ß', 'ÿ'),
            ('\u{3040}', '\u{9fff}'),
            ('\u{ac00}', '\u{d7a3}'),
        ];
        for c in ranges.into_iter().flat_map(|(first, last)| first..=last) {
            assert_eq!(fold(c), c, "{c:?}");
            let mut lower = c.to_lowercase();
            assert_eq!((lower.next(), lower.next()), (Some(c), None), "{c:?}");
            assert_eq!(class_first(c), None, "{c:?}");
        }
    }

    /// The ASCII skipped pass (T-4279) is the general one over every ASCII character.
    #[test]
    fn ascii_skipped_pass_matches_the_general_one() {
        let all: String = (0u8..0x80).map(char::from).collect();
        for text in [all.as_str(), "a-b'c", "--''x\u{1}\u{7f}y", "", "'"] {
            let fast: Vec<_> = ascii_skipped(text.bytes()).collect();
            let general: Vec<_> = skipped(elements(scalars(text), None)).collect();
            assert_eq!(fast, general, "{text:?}");
        }
    }
}
