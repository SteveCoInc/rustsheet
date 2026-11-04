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
/// (`parse_excel_number`, T-3932), so `4910E-9` (the nearest double to 4.91E-06) prints as `4.91e-6`,
/// which would read one ulp higher: such a number prints all 15 digits instead
/// (`4.91000000000000e-6`), which read to the nearest double.
fn literal_text(number: f64) -> String {
    let short = to_excel_precision_str(number);
    if !number.is_finite() || parse_excel_number(&short) == Some(number) {
        return short;
    }
    let full = format!("{number:.14e}");
    if parse_excel_number(&full) == Some(number) {
        full
    } else {
        short
    }
}
