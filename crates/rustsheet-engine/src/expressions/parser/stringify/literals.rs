//! How literals print in formula text: numbers in a locale, array-constant elements and the
//! argument separator. Split out of `stringify.rs` to keep it under its size ratchet (T-0499).

use crate::expressions::parser::ArrayNode;
use crate::language::Language;
use crate::locale::Locale;
use crate::number_format::{parse_excel_number, to_excel_precision_str};

/// The separator of function arguments, and of union operands, in `locale`: `,`, or `;`
/// where the decimal separator is not `.`.
pub(crate) fn argument_separator(locale: &Locale) -> &'static str {
    if locale.numbers.symbols.decimal == "." {
        ","
    } else {
        ";"
    }
}

/// `number` as a formula prints it, with the decimal separator of `locale`.
pub(super) fn format_number_locale(number: f64, locale: &Locale) -> String {
    let s = literal_text(number);
    let decimal = &locale.numbers.symbols.decimal;
    if decimal == "." {
        s
    } else {
        s.replace('.', decimal)
    }
}

/// The printed element `node` of an array constant.
pub(super) fn to_string_array_node(
    node: &ArrayNode,
    locale: &Locale,
    language: &Language,
) -> String {
    match node {
        ArrayNode::Boolean(true) => language.booleans.r#true.to_uppercase(),
        ArrayNode::Boolean(false) => language.booleans.r#false.to_uppercase(),
        ArrayNode::Number(number) => format_number_locale(*number, locale),
        ArrayNode::String(value) => format!("\"{value}\""),
        ArrayNode::Error(kind) => format!("{kind}"),
        ArrayNode::Empty => "0".to_string(),
        ArrayNode::Lambda(_) => "#CALC!".to_string(), // never parsed: what a cell shows
        ArrayNode::Nested(_) => "#VALUE!".to_string(), // never parsed: what a cell shows
    }
}

/// `number`'s 15-digit text, which formulas are reparsed from, written so that it reads back as
/// `number`. Short number text reads through Excel's x87 conversion of its digits as written
/// (`parse_excel_number`, T-3932), so the shortest text of a literal can read one ulp away from
/// it: `4910E-9` (the nearest double to 4.91E-06) would print as `4.91e-6`, which reads one ulp
/// higher, and `14550E-15` (one ulp above the nearest double) as `1.455e-11`, which reads as the
/// nearest (probes-numeric/t3658-round-scaling Hits!C18 and C59). Such a number prints its digits
/// with trailing zeros added (`4910e-9`, `14550e-15`: the digits as written times 10^-n), or
/// failing that all 15 digits (`4.91000000000000e-6`), which read to the nearest double. A double
/// none of these reads back to keeps the shortest text, as before T-3932.
fn literal_text(number: f64) -> String {
    let short = to_excel_precision_str(number);
    if !number.is_finite() || parse_excel_number(&short) == Some(number) {
        return short;
    }
    let full = format!("{number:.14e}");
    let found = padded_digit_texts(&full)
        .chain([full.clone()])
        .find(|text| parse_excel_number(text) == Some(number));
    found.unwrap_or(short)
}

/// The 15-digit scientific text `full` (`1.45500000000000e-11`) as its significant digits with
/// 0, 1, … trailing zeros, up to the six digits the x87 read covers, over a matching exponent:
/// `1455e-14`, `14550e-15`, `145500e-16`.
fn padded_digit_texts(full: &str) -> impl Iterator<Item = String> + '_ {
    let (mantissa, exponent) = full.split_once('e').unwrap_or((full, "0"));
    let exponent: i64 = exponent.parse().unwrap_or(0);
    let sign = if mantissa.starts_with('-') { "-" } else { "" };
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = digits.trim_end_matches('0').to_string();
    // `digits` is at most 15 characters, so the conversion holds.
    let length = i64::try_from(digits.len()).unwrap_or(15);
    (0..=(6 - length).max(-1)).map(move |pad| {
        let zeros = "0".repeat(usize::try_from(pad).unwrap_or(0));
        format!("{sign}{digits}{zeros}e{}", exponent - (length - 1) - pad)
    })
}
