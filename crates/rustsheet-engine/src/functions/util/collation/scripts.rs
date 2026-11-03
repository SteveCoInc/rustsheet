//! Where the scripts after the letters sort, their marks and digits, and the symbols above `€`
//! (probes/t3276-collation-other-scripts, T-3324; docs/excel-semantics/coercion.md "Other scripts,
//! their marks and digits, symbols and astral characters").

use super::{digit, Element, NORMAL_KANA, NO_ACCENT, OTHER_BASE, SYMBOL_BASE};

/// The groups of [`script_order`], each above the one before.
const SCRIPTS: u32 = 0x1_0000;
const SURROGATES: u32 = 0x2_0000;
const HANGUL: u32 = 0x3_0000;
pub(super) const IDEOGRAPHS: u32 = 0x4_0000;
const IDEOGRAPH_SURROGATES: u32 = 0x5_0000;

/// Where a BMP code point or a UTF-16 surrogate unit sorts among the characters after the letters.
/// Excel's block order is not code-point order: Latin, Greek and Cyrillic, then kana, Hebrew,
/// Arabic, Devanagari, Thai, Lao, the astral `𝐀` and `😀`, Hangul, the CJK ideographs, then the
/// astral ideograph `𠀀`; inside each block, code-point order (probes/t3276-collation-other-scripts
/// G2:G8, G35, ranks A18:A21, A44, A79, A95, A96; docs/excel-semantics/coercion.md "Other scripts,
/// their marks and digits, symbols and astral characters"). The blocks not probed keep code-point
/// order inside the group they fall in (UNVERIFIED):
/// - below U+0590 and the Latin and Greek extensions U+1D00 to U+1FFF: code point, with the kana
///   U+3040 to U+30FF just above them ([`super::kana`] weighs them by code point);
/// - the rest of the BMP but Hangul syllables and ideographs: code point, above the kana; the Thai
///   prevowels `เ แ โ ใ ไ` just before the Thai consonants (`"เก" < "ก"`, ranks A72, A74, A39;
///   only `เ` and `แ` probed), the Lao prevowel `ເ` in its code-point place (G24);
/// - surrogate units below U+D840 (the astral planes before the ideographs), then Hangul
///   syllables, the CJK ideographs U+3400 to U+9FFF, then surrogate units from U+D840. A lone
///   surrogate is a unit like those of a pair, so it stays above U+FFFD and a lone high half sorts
///   before the pair it starts (probes/text-lone-surrogates A18:A24).
pub(super) fn script_order(code: u32) -> u32 {
    match code {
        0..=0x58f | 0x1d00..=0x1fff | 0x3040..=0x30ff => code,
        // The Thai prevowels, on the unassigned U+0DFB to U+0DFF just below the Thai block.
        0xe40..=0xe44 => SCRIPTS + code - 0x45,
        0xac00..=0xd7a3 => HANGUL + code,
        0x3400..=0x9fff => IDEOGRAPHS + code,
        0xd800..=0xd83f => SURROGATES + code,
        0xd840..=0xdfff => IDEOGRAPH_SURROGATES + code,
        _ => SCRIPTS + code,
    }
}

/// The primary weight of a UTF-16 surrogate unit, of a pair or lone ([`script_order`]). An astral
/// character weighs its two units, so a lone high surrogate sorts before a pair starting with it
/// and a lone low one after it (`H<"😀"`, `"😀"<L`, `H<L` TRUE, `L<H` FALSE;
/// probes/text-lone-surrogates A18-A22). A lone surrogate is above U+FFFD (A23, A24 FALSE).
pub(super) fn unit(unit: u16) -> u32 {
    OTHER_BASE + 2 * script_order(u32::from(unit))
}

/// The primary weight of an arrow (U+2190 to U+21FF), a miscellaneous symbol or a dingbat (U+2600
/// to U+27BF): after [`super::PUNCTUATION`], before the digits, by code point (`">" < "←" < "☺" < "♥" <
/// "✓" < "0"`; probes/t3276-collation-other-scripts ranks A6, A90, A94, A92, A93, A8, G29:G31).
/// Only those four are probed; the other symbol blocks keep [`super::other`].
pub(super) fn symbol(c: char) -> Option<u32> {
    let code = u32::from(c);
    matches!(code, 0x2190..=0x21ff | 0x2600..=0x27bf).then(|| SYMBOL_BASE + code - 0x2190)
}

/// The variant pass weights of the other scripts' digits: after the ASCII digit and its
/// [`super::FULL_WIDTH`] form, in code-point order of their blocks: Arabic-Indic, Extended
/// Arabic-Indic, Devanagari, Bengali, Gurmukhi, Tamil, Telugu, Thai, Lao, Tibetan, Myanmar, Khmer
/// (probes/t3276-collation-other-scripts G25:G27; probes/t3895-collation-script-digits A4:A6,
/// A129:A131, A151:A178; probes/t3559-collation-unprobed-blocks ranks A30:A41).
pub(super) const ARABIC_INDIC_DIGIT: u8 = 3;
pub(super) const EXTENDED_ARABIC_INDIC_DIGIT: u8 = 4;
pub(super) const DEVANAGARI_DIGIT: u8 = 5;
pub(super) const BENGALI_DIGIT: u8 = 6;
pub(super) const GURMUKHI_DIGIT: u8 = 7;
pub(super) const TAMIL_DIGIT: u8 = 8;
pub(super) const TELUGU_DIGIT: u8 = 9;
pub(super) const THAI_DIGIT: u8 = 10;
pub(super) const LAO_DIGIT: u8 = 11;
pub(super) const TIBETAN_DIGIT: u8 = 12;
pub(super) const MYANMAR_DIGIT: u8 = 13;
pub(super) const KHMER_DIGIT: u8 = 14;

/// A digit of another script: digit `value` with `variant` in the variant pass.
pub(super) fn digit_variant(value: u32, variant: u8) -> Element {
    Element::Weights {
        primary: [digit(value), 0],
        accent: NO_ACCENT,
        kana: NORMAL_KANA,
        variant,
    }
}

/// A combining mark of another script, weighing its code point less `block`.
pub(super) fn script_mark(c: char, block: u32) -> Element {
    Element::Mark(u8::try_from(u32::from(c) - block).unwrap_or(0))
}
