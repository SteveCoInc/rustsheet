//! Where the scripts after the letters sort, their marks and digits, and the symbols above `€`
//! (probes/t3276-collation-other-scripts, T-3324; docs/excel-semantics/coercion.md "Other scripts,
//! their marks and digits, symbols and astral characters").

use super::{
    digit, other, single, Element, NORMAL_KANA, NO_ACCENT, OTHER_BASE, SYMBOL_BASE, UNASSIGNED,
};

/// The groups of [`script_order`], each above the one before.
const SCRIPTS: u32 = 0x1_0000;
const SURROGATES: u32 = 0x2_0000;
const HANGUL: u32 = 0x3_0000;
pub(super) const IDEOGRAPHS: u32 = 0x4_0000;
const IDEOGRAPH_SURROGATES: u32 = 0x5_0000;

/// Where a BMP code point or a UTF-16 surrogate unit sorts among the characters after the letters.
/// Excel's block order is not code-point order. Lowest first: Latin, Greek, Cyrillic, Armenian,
/// Georgian, kana, Hebrew, Arabic, Devanagari, Bengali, Gurmukhi, Tamil, Telugu, Thai, Myanmar,
/// Mongolian, Tibetan, Lao, Khmer, Ethiopic, the astral characters of planes 1 and 3 (`𝐀`, `😀`,
/// `𰀀`), Hangul, the CJK ideographs, then plane 2 (`𠀀`, `𪜀`); inside each block, code-point order
/// with the exceptions of [`in_block`] (probes/t3276-collation-other-scripts G2:G8, G35, ranks
/// A18:A21, A44, A79, A95, A96; probes/t3559-collation-unprobed-blocks ranks A19:A30, A36:A155;
/// docs/excel-semantics/coercion.md "Other scripts, their marks and digits, symbols and astral
/// characters" and "Scripts and symbol blocks beyond T-3276"). The blocks not probed keep
/// code-point order inside the group they fall in (UNVERIFIED):
/// - below U+0590 and the Latin and Greek extensions U+1D00 to U+1FFF: code point, with Georgian
///   ([`GEORGIAN`]) after Armenian and the kana U+3040 to U+30FF just above them ([`super::kana`]
///   weighs them by code point);
/// - the rest of the BMP but Hangul syllables and ideographs: code point, above the kana, with
///   U+0E00 to U+18AF in [`BLOCKS`] order; the Thai prevowels `เ แ โ ใ ไ` just before the Thai
///   consonants (`"เก" < "โ" < "ไก" < "ก"`, t3559 ranks 101 to 108), the Lao prevowels in their
///   code-point place (t3276 G24, t3559 ranks 123 to 132);
/// - surrogate units of planes 1 and 3 and above (high units below U+D840 or from U+D880), then
///   Hangul syllables, the CJK ideographs U+3400 to U+9FFF, then plane 2's high units U+D840 to
///   U+D87F and every low unit. A lone surrogate is a unit like those of a pair, so it stays above
///   U+FFFD and a lone high half sorts before the pair it starts (probes/text-lone-surrogates
///   A18:A24).
pub(super) fn script_order(code: u32) -> u32 {
    match code {
        0x10a0..=0x10ff | 0x2d00..=0x2d2f => GEORGIAN + georgian(code),
        0..=0x58f | 0x1d00..=0x1fff | 0x3040..=0x30ff => code,
        // The Thai prevowels, on the unassigned U+0DFB to U+0DFF just below the Thai block.
        0xe40..=0xe44 => SCRIPTS + code - 0x45,
        0xe00..=0x18af => SCRIPTS + block_order(in_block(code)),
        0xac00..=0xd7a3 => HANGUL + code,
        0x3400..=0x9fff => IDEOGRAPHS + code,
        0xd800..=0xd83f | 0xd880..=0xdbff => SURROGATES + code,
        0xd840..=0xdfff => IDEOGRAPH_SURROGATES + code,
        _ => SCRIPTS + in_block(code),
    }
}

/// Georgian sorts after Armenian (`"ֆ" < "ა"`, t3559 ranks 55, 56), on the unassigned
/// U+0590-up of the group below the kana (Hebrew sorts in [`SCRIPTS`]).
const GEORGIAN: u32 = 0x590;

/// The Mkhedruli letters U+10D0 to U+10FF before the Asomtavruli capitals U+10A0 to U+10CF
/// (`"ა" < "ჰ" < "Ⴀ"`, t3559 ranks 56 to 58), which the case fold has made Nuskhuri U+2D00 to
/// U+2D2F; `Ⴀ` is not equal to `ა` (B54 = 1).
fn georgian(code: u32) -> u32 {
    match code {
        0x10d0..=0x10ff => code - 0x10d0,
        0x2d00..=0x2d2f => 0x30 + code - 0x2d00,
        _ => 0x30 + code - 0x10a0,
    }
}

/// The places inside a block where Excel's order is not code-point order: Gurmukhi `ਹ` U+0A39
/// before `ਕ` U+0A15 (`"ਅ" < "ਹ" < "ਕ"`, t3559 ranks 89 to 91) and Lao `ໄ` U+0EC4 before `ໃ`
/// U+0EC3 (ranks 131, 132). Only `ਹ` was probed of the Gurmukhi letters after `ਕ`, so the others
/// keep their order after it (UNVERIFIED).
fn in_block(code: u32) -> u32 {
    match code {
        0xa39 => 0xa15,
        0xa15..=0xa38 => code + 1,
        0xec3 => 0xec4,
        0xec4 => 0xec3,
        _ => code,
    }
}

/// U+0E00 to U+18AF in Excel's block order: Thai, Myanmar, Mongolian, Tibetan, Lao, Khmer,
/// Ethiopic (t3559 ranks 101 to 139), then the unprobed blocks of the span in code-point order
/// (Georgian sorts with [`GEORGIAN`]; UNVERIFIED for Hangul Jamo, Cherokee, Canadian syllabics,
/// Ogham, Runic and the Philippine scripts).
const BLOCKS: [(u32, u32); 10] = [
    (0xe00, 0xe7f),
    (0x1000, 0x109f),
    (0x1800, 0x18af),
    (0xf00, 0xfff),
    (0xe80, 0xeff),
    (0x1780, 0x17ff),
    (0x1200, 0x139f),
    (0x10a0, 0x10ff),
    (0x1100, 0x11ff),
    (0x13a0, 0x177f),
];

/// `code`'s place in U+0E00 to U+18AF reordered by [`BLOCKS`], which covers the span once, so
/// the result stays in it.
fn block_order(code: u32) -> u32 {
    let mut start = 0xe00;
    for (first, last) in BLOCKS {
        if (first..=last).contains(&code) {
            return start + code - first;
        }
        start += last - first + 1;
    }
    code
}

/// The primary weight of a UTF-16 surrogate unit, of a pair or lone ([`script_order`]). An astral
/// character weighs its two units, so a lone high surrogate sorts before a pair starting with it
/// and a lone low one after it (`H<"😀"`, `"😀"<L`, `H<L` TRUE, `L<H` FALSE;
/// probes/text-lone-surrogates A18-A22). A lone surrogate is above U+FFFD (A23, A24 FALSE).
pub(super) fn unit(unit: u16) -> u32 {
    OTHER_BASE + 2 * script_order(u32::from(unit))
}

/// The symbols after [`super::PUNCTUATION`] and before `•` ([`super::BULLET`]), lowest first: the
/// math operators `≠ ∑ ∫ ≤` (not in code-point order), the geometric shapes (`■ ●`), the arrows (`←
/// ⇿`), box drawing and block elements (`─ █`), the miscellaneous symbols and dingbats (`☀ ✓`); a
/// block's characters by code point (`">" < "←" < "☺" < "♥" < "✓" < "0"`,
/// probes/t3276-collation-other-scripts ranks A6, A90, A94, A92, A93, A8, G29:G31;
/// probes/t3559-collation-unprobed-blocks ranks 15 to 28, G16, G18). Only those math operators are
/// probed, so the others keep [`super::other`]; the unprobed characters of the four blocks are
/// placed by code point (UNVERIFIED).
pub(super) fn symbol(c: char) -> Option<u32> {
    const MATH: [char; 4] = ['\u{2260}', '\u{2211}', '\u{222b}', '\u{2264}'];
    const GROUPS: [(u32, u32); 4] = [
        (0x25a0, 0x25ff),
        (0x2190, 0x21ff),
        (0x2500, 0x259f),
        (0x2600, 0x27bf),
    ];
    let code = u32::from(c);
    if let Some(index) = MATH.iter().position(|&m| m == c) {
        return u32::try_from(index).ok().map(|index| SYMBOL_BASE + index);
    }
    let mut start = SYMBOL_BASE + 4;
    for (first, last) in GROUPS {
        if (first..=last).contains(&code) {
            return Some(start + code - first);
        }
        start += last - first + 1;
    }
    None
}

/// The number of weights [`symbol`] takes: [`super::BULLET`] sorts just after them.
pub(super) const SYMBOLS: u32 = 4 + 0x60 + 0x70 + 0xa0 + 0x1c0;

/// The variant pass weights of the other scripts' digits: after the ASCII digit and its
/// [`super::FULL_WIDTH`] form, in code-point order of their blocks: Arabic-Indic, Extended
/// Arabic-Indic, Devanagari, Bengali, Gurmukhi, Tamil, Telugu, Thai, Lao, Tibetan, Myanmar, Khmer
/// (probes/t3276-collation-other-scripts G25:G27; probes/t3895-collation-script-digits A4:A6,
/// A129:A131, A151:A178; probes/t3559-collation-unprobed-blocks ranks Calc!A11, A136:A145).
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

/// The element of a case-folded character of another script whose weight is not its block's
/// code-point place ([`script_order`]): the digits, the marks that are accents, the stand-in for
/// characters without a weight, and the forms equal to another letter. `None` for every other
/// character.
pub(super) fn script_element(c: char) -> Option<Element> {
    Some(match c {
        // Other-script digits are their digit, after it in the variant pass (`"3" < "٣" < "३" <
        // "3a"`; probes/t3276-collation-other-scripts ranks A10, A81:A86, G25:G27).
        '\u{660}'..='\u{669}' => digit_variant(u32::from(c) - 0x660, ARABIC_INDIC_DIGIT),
        '\u{6f0}'..='\u{6f9}' => digit_variant(u32::from(c) - 0x6f0, EXTENDED_ARABIC_INDIC_DIGIT),
        '\u{966}'..='\u{96f}' => digit_variant(u32::from(c) - 0x966, DEVANAGARI_DIGIT),
        // The Bengali, Gurmukhi, Tamil, Telugu, Thai, Lao, Tibetan, Myanmar and Khmer digits too,
        // below every letter (`"2z" < "๓" < "3a"`, `"๓" < "a"`; probes/t3895-collation-script-digits
        // A1:A150, probes/t3559-collation-unprobed-blocks ranks Calc!A11, A136:A145).
        '\u{9e6}'..='\u{9ef}' => digit_variant(u32::from(c) - 0x9e6, BENGALI_DIGIT),
        '\u{a66}'..='\u{a6f}' => digit_variant(u32::from(c) - 0xa66, GURMUKHI_DIGIT),
        '\u{be6}'..='\u{bef}' => digit_variant(u32::from(c) - 0xbe6, TAMIL_DIGIT),
        '\u{c66}'..='\u{c6f}' => digit_variant(u32::from(c) - 0xc66, TELUGU_DIGIT),
        '\u{e50}'..='\u{e59}' => digit_variant(u32::from(c) - 0xe50, THAI_DIGIT),
        '\u{ed0}'..='\u{ed9}' => digit_variant(u32::from(c) - 0xed0, LAO_DIGIT),
        '\u{f20}'..='\u{f29}' => digit_variant(u32::from(c) - 0xf20, TIBETAN_DIGIT),
        '\u{1040}'..='\u{1049}' => digit_variant(u32::from(c) - 0x1040, MYANMAR_DIGIT),
        '\u{17e0}'..='\u{17e9}' => digit_variant(u32::from(c) - 0x17e0, KHMER_DIGIT),
        // U+0E83, unassigned in the Lao block, is one of T-3584's weightless characters
        // (docs/research/t3584/weightless-bmp.txt). Skipping it matches Excel alone (above `""`,
        // below `" "`) and trailing (`"a"&U+0E83 > "a"`; probes/t3895-collation-script-digits
        // A179:A184, A187), but not leading: Excel finds U+0E83&"a" equal to "a" (A185). T-3904
        // owns the weightless set and replaces this one-point stand-in.
        '\u{e83}' => Element::Skipped(UNASSIGNED),
        // `➿` U+27BF, `ᡸ` U+1878 and `₿` U+20BF, also weightless characters of T-3584, are equal to
        // each other, above `""` and below `"!"` (probes/t3559-collation-unprobed-blocks ranks A35,
        // A62, A126 = 1, B35 = 3, G17), as the same stand-in gives. T-3904 replaces it with the
        // weightless rule for the whole set.
        '\u{27bf}' | '\u{1878}' | '\u{20bf}' => Element::Skipped(UNASSIGNED),
        // Hebrew `ץ` equals `צ` (G14 = 2, B97 = 2); the other final letters keep their code-point
        // place for ordering and `=` (UNVERIFIED, T-3904), although lookup and criteria folds do
        // equate them (T-3995, lookup_fold.rs).
        '\u{5e5}' => single(other('\u{5e6}')),
        // Arabic `ﻻ` (the lam-alef ligature's isolated form) is `لا` (G13 = 2, B109 = 2).
        '\u{fefb}' => Element::Weights {
            primary: [other('\u{644}'), other('\u{627}')],
            accent: NO_ACCENT,
            kana: NORMAL_KANA,
            variant: 0,
        },
        // Hebrew points and cantillation, Arabic harakat and the Thai tone marks are accents on
        // their letter, weighed by code point (`"ב" < "ב"&UNICHAR(1463) < "ב"&UNICHAR(1468) <
        // "בא"`, `"ب" < "ب"&UNICHAR(1614) < "ب"&UNICHAR(1617) < "با"`, `"ก" < "ก"&UNICHAR(3656) <
        // "กก"`; probes/t3276-collation-other-scripts G13:G16, G19, G20). Only U+05B7, U+05BC,
        // U+064E, U+0651 and U+0E48 are probed.
        '\u{591}'..='\u{5bd}'
        | '\u{5bf}'
        | '\u{5c1}'
        | '\u{5c2}'
        | '\u{5c4}'
        | '\u{5c5}'
        | '\u{5c7}' => script_mark(c, 0x590),
        '\u{610}'..='\u{61a}' | '\u{64b}'..='\u{65f}' | '\u{670}' => script_mark(c, 0x600),
        '\u{e48}'..='\u{e4b}' => script_mark(c, 0xe00),
        _ => return None,
    })
}
