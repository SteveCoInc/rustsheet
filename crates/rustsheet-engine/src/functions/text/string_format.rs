// DOLLAR, FIXED, NUMBERVALUE, PROPER, REPLACE, ARRAYTOTEXT

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_text,
    cast::number_text_too_long,
    cast::MAX_TEXT_UNITS,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{lift::number_or_error, Function},
    model::Model,
    number_format::{cut_15_digits, group_thousands, increment_digits, parse_excel_number},
};

use super::proper_letters::is_proper_letter;
use crate::functions::util::near_integer::{near_integer_count, near_integer_trunc};

/// Formats a non-negative f64 with the given decimal places and optional thousands separator.
/// Returns the formatted string without sign. The value is cut to 15 significant digits and then
/// rounded half away from zero at `decimals`, in decimal as ROUND does.
fn format_abs(abs_value: f64, decimals: i32, use_thousands: bool) -> String {
    let (mut int, mut frac) = cut_15_digits(abs_value, false);
    if decimals >= 0 {
        let keep = usize::try_from(decimals).unwrap_or(0);
        let round_up = frac.get(keep).is_some_and(|&d| d >= 5);
        frac.resize(keep, 0);
        if round_up {
            let mut all = int;
            all.extend(frac);
            increment_digits(&mut all);
            let split = all.len() - keep;
            frac = all.split_off(split);
            int = all;
        }
    } else {
        // Negative decimals round the integer part; a place past its length gives 0.
        let places = usize::try_from(decimals.unsigned_abs()).unwrap_or(usize::MAX);
        if places >= int.len() {
            let round_up = places == int.len() && int[0] >= 5;
            int = if round_up { vec![1] } else { vec![0] };
            if round_up {
                int.resize(places + 1, 0);
            }
        } else {
            let split = int.len() - places;
            let round_up = int[split] >= 5;
            int[split..].fill(0);
            if round_up {
                let mut head = int[..split].to_vec();
                increment_digits(&mut head);
                head.resize(head.len() + places, 0);
                int = head;
            }
        }
        frac.clear();
    }
    let int_text: String = int.iter().map(|&d| char::from(b'0' + d)).collect();
    let int_text = if use_thousands {
        group_thousands(&int_text, ",")
    } else {
        int_text
    };
    if frac.is_empty() {
        int_text
    } else {
        let frac_text: String = frac.iter().map(|&d| char::from(b'0' + d)).collect();
        format!("{int_text}.{frac_text}")
    }
}

/// Longest text DOLLAR and FIXED return: a longer result is `#VALUE!`, whatever the value's
/// magnitude (`FIXED(1E+200,2,TRUE)` prints, `FIXED(1E+188,4)` at 256 characters does not).
/// DOLLAR counts one more character for a value without parentheses, as if the `_)` padding of
/// its positive format section were part of the text (docs/excel-semantics/text.md "FIXED and
/// DOLLAR return at most 255 characters", probes/fixed-dollar-length-bound).
const MAX_FORMATTED_LEN: usize = 255;

/// `text` as DOLLAR or FIXED's result, or `#VALUE!` past [`MAX_FORMATTED_LEN`] with `padding`
/// extra characters counted. The text is ASCII, so bytes are characters.
fn formatted_result(
    text: String,
    padding: usize,
    cell: CellReferenceIndex,
) -> Result<CalcResult, CalcResult> {
    if text.len() + padding > MAX_FORMATTED_LEN {
        return Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "result longer than 255 characters".to_string(),
        ));
    }
    Ok(CalcResult::String(text.into()))
}

impl Model {
    /// The `decimals` argument of DOLLAR and FIXED (default 2): truncated toward zero, except that
    /// a magnitude just below an integer counts as that integer ([`near_integer_trunc`]), and
    /// `#VALUE!` from 128 up (docs/excel-semantics/text.md#dollar-and-fixed-decimals).
    fn decimals_arg(
        &self,
        arg: Option<&CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<i32, CalcResult> {
        let Some(value) = arg else { return Ok(2) };
        let d = near_integer_trunc(self.cast_number_ref(value, cell)?);
        if d >= 128.0 {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "decimals too large".to_string(),
            ));
        }
        // The clamp only keeps the cast in range: every large negative gives "0" (FIXED(1,-400)).
        Ok(d.max(-1000.0) as i32)
    }

    /// DOLLAR(number, [decimals]) — Formats a number as a dollar currency string.
    /// Negative numbers use parentheses: ($1,234.57). Both arguments lift over an array and a
    /// range (probes/t2198-m5-lift-argument-kinds rows 20, 21 and 33).
    pub(crate) fn fn_dollar(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dollar, args, cell, &mut |model, values| {
            model.dollar_of(values, cell).unwrap_or_else(|e| e)
        })
    }

    /// One DOLLAR element.
    fn dollar_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        let Some(number) = values.first() else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let value = self.cast_number_ref(number, cell)?;
        let decimals = self.decimals_arg(values.get(1), cell)?;
        let formatted = format_abs(value.abs(), decimals, true);
        // A negative value that rounds to zero at negative `decimals` has no parentheses
        // (docs/excel-semantics/text.md "FIXED, DOLLAR and BAHTTEXT cut to 15 significant digits
        // first": `DOLLAR(-0.5,-1)` is "$0"), while `DOLLAR(-0.004)` keeps "($0.00)".
        let rounds_to_zero = !formatted.bytes().any(|b| matches!(b, b'1'..=b'9'));
        if value < 0.0 && !(decimals < 0 && rounds_to_zero) {
            formatted_result(format!("(${formatted})"), 0, cell)
        } else {
            formatted_result(format!("${formatted}"), 1, cell)
        }
    }

    /// FIXED(number, [decimals], [`no_commas`]) — Formats a number as text with fixed decimal places.
    /// Every argument lifts over an array and a range (probes/t2198-m5-lift-argument-kinds rows
    /// 22-24 and 34).
    pub(crate) fn fn_fixed(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Fixed, args, cell, &mut |model, values| {
            model.fixed_of(values, cell).unwrap_or_else(|e| e)
        })
    }

    /// One FIXED element.
    fn fixed_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        let Some(number) = values.first() else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let value = self.cast_number_ref(number, cell)?;
        let decimals = self.decimals_arg(values.get(1), cell)?;
        let no_commas = match values.get(2) {
            Some(flag) => Self::cast_bool_ref(flag, cell)?,
            None => false,
        };
        let formatted = format_abs(value.abs(), decimals, !no_commas);
        // Excel drops the sign when the value rounds to zero: look at the rounded text, not the
        // value (docs/excel-semantics/text.md "FIXED drops the sign of a value that rounds to
        // zero"; gen-m6/f94b5b8236742618 A24, probes/dollar-fixed-unichar-edges A49, A50).
        let rounds_to_zero = !formatted.bytes().any(|b| matches!(b, b'1'..=b'9'));
        let result = if value < 0.0 && !rounds_to_zero {
            format!("-{formatted}")
        } else {
            formatted
        };
        formatted_result(result, 0, cell)
    }

    /// NUMBERVALUE(text, [`decimal_separator`], [`group_separator`])
    /// Converts text to a number using the given separators. Every argument lifts over an array
    /// and a range (probes/t2198-m5-lift-argument-kinds rows 25-27 and 35).
    pub(crate) fn fn_numbervalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Numbervalue, args, cell, &mut |model, values| {
            number_or_error(model.numbervalue_of(values, cell))
        })
    }

    /// One NUMBERVALUE element.
    fn numbervalue_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let Some(text) = values.first() else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let text = Self::cast_string_ref(text, cell)?;
        let separator = |i: usize, default: &str| -> Result<String, CalcResult> {
            values.get(i).map_or_else(
                || Ok(default.to_string()),
                |v| Self::cast_string_ref(v, cell),
            )
        };
        let decimal_sep = separator(1, ".")?;
        // The omitted group separator is "." when the decimal separator starts with ",", else ",":
        // `NUMBERVALUE("1.5",",")` is 15, `("1.000,5",",")` 1000.5, `("1,5",";")` 15
        // (probes/numbervalue-default-group A1-A14; docs/excel-semantics/text.md, T-2441).
        let default_group = if decimal_sep.starts_with(',') {
            "."
        } else {
            ","
        };
        let group_sep = separator(2, default_group)?;
        // A separator over 255 characters is #VALUE!, though only its first character is used
        // (probes/numbervalue-separator-edges A1, A2, A10, A11, A17-A19; T-2379).
        if number_text_too_long(&decimal_sep) || number_text_too_long(&group_sep) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Separator too long".to_string(),
            ));
        }

        // Only the first character of each separator counts, and an empty one is #VALUE!
        // (docs/excel-semantics/text.md "NUMBERVALUE reads text as VALUE does"; gen-m5/5704666097cbc5b2
        // Calc!A261, gen-m5/a6fec430ac350c1a Calc!A32), a written-empty argument too, not the
        // default an omitted one takes (probes/t4874-numbervalue-empty-separator, T-4874).
        let (Some(decimal_sep), Some(group_sep)) =
            (decimal_sep.chars().next(), group_sep.chars().next())
        else {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid separators".to_string(),
            ));
        };
        if decimal_sep == group_sep {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid separators".to_string(),
            ));
        }

        // Excel counts the text with its spaces: 255 characters parse, 256 are #VALUE!
        // (probes/text-number-long-paths A13, A14).
        if number_text_too_long(&text) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Text too long".to_string(),
            ));
        }
        let text = numbervalue_text(&text, decimal_sep, group_sep);
        if text.is_empty() {
            return Ok(0.0);
        }
        // The decimal separator is read before any "%": with the decimal separator "%",
        // `NUMBERVALUE("%5","%")` is 0.5, `("5%","%")` 5 and `("%5%","%")` #VALUE!
        // (probes/t3647-numbervalue-whitespace-decimal A13-A16).
        let cleaned = text.replace(decimal_sep, ".");

        // NBSP and U+3000 are not trimmed, where the shared reader below would trim them, but
        // only those left after the decimal separator is replaced: as the decimal separator they
        // are read at the ends too (probes/t3746-numbervalue-nbsp-separator A3, A5, A9, A11):
        // `NUMBERVALUE(UNICHAR(160)&"1.5")`, `("1.5"&UNICHAR(160))` and `(UNICHAR(12288)&"1.5")`
        // are #VALUE! (probes/t3647-numbervalue-whitespace-decimal A25-A27).
        let untrimmed = |c: char| matches!(c, '\u{a0}' | '\u{3000}');
        if cleaned.starts_with(untrimmed) || cleaned.ends_with(untrimmed) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Cannot parse number".to_string(),
            ));
        }

        // A trailing run of "%" divides by 100 each; text of "%" signs alone is 0: `"%"` and
        // `"%%"` (probes/t3647-numbervalue-percent-and-controls A10, A11).
        let mut pct_count = 0u32;
        let mut rest = cleaned.as_str();
        while let Some(shorter) = rest.strip_suffix('%') {
            rest = shorter;
            pct_count += 1;
        }
        if rest.is_empty() {
            return Ok(0.0);
        }
        // One leading "%" counts too, also after a sign or "(", but not with currency anywhere:
        // `"%1.5"`, `"-%1.5"`, `"+%1.5"`, `"%-1.5"` and `"(%1.5)"` read; `"%%1.5"`, `"%$1.5"`,
        // `"$%1.5"`, `"%1.5$"` and `"%€1.5"` are #VALUE! (probes/t3647-numbervalue-percent-and-controls
        // A1-A8, probes/t3647-numbervalue-whitespace-decimal A17-A21). Excel's VALUE reads
        // `"%1.5"` too (T-3811).
        let leading = strip_leading_percent(rest);
        if let Some(stripped) = &leading {
            if stripped.contains(['$', '€']) {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Percent with currency".to_string(),
                ));
            }
            pct_count += 1;
        }
        let cleaned = leading.as_deref().unwrap_or(rest);
        // The shared reader now reads one leading "%" itself (VALUE, T-3811), so a second one
        // left here must stay #VALUE!: `"%%1.5"` (probes/t3647-numbervalue-percent-and-controls A1).
        if cleaned.contains('%') {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Cannot parse number".to_string(),
            ));
        }

        // Reject if more than one '.' after replacement
        if cleaned.matches('.').count() > 1 {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Multiple decimal separators".to_string(),
            ));
        }

        // Then the text reads as VALUE reads it: currency, parentheses, dates and clock text past
        // 24 hours too (docs/excel-semantics/text.md "NUMBERVALUE reads text as VALUE does, with
        // its own separators"; gen-m5/033e65b92653f425 A161, ae9ed30812c4d62c A285,
        // 6b831f5e2a20bc34 A227).
        let number = parse_excel_number(cleaned)
            .or_else(|| self.parse_number_text(cleaned, &["$", "€"], &|| self.current_year()));
        match number {
            Some(mut v) => {
                for _ in 0..pct_count {
                    v /= 100.0;
                }
                Ok(v)
            }
            None => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Cannot parse number".to_string(),
            )),
        }
    }

    /// PROPER(text) — Converts text to title case (first letter of each word uppercase).
    pub(crate) fn fn_proper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.try_apply_text_unary_text(args, cell, |text| {
            super::common::reject_over_limit(text)?;
            Ok(text.map_runs(proper_case))
        })
    }

    /// `REPLACE(old_text`, `start_num`, `num_chars`, `new_text`)
    /// Replaces `num_chars` characters starting at `start_num` (1-indexed) with `new_text`.
    /// If any of the arguments is an array it works element by element.
    pub(crate) fn fn_replace(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Replace, args, cell, &mut |model, values| {
            let [old_text, start_num, num_chars, new_text] = values else {
                return Err(Error::ERROR);
            };
            replace_element(
                old_text,
                model.calc_result_to_number(start_num),
                model.calc_result_to_number(num_chars),
                new_text,
            )
        })
    }
}

/// Computes a single REPLACE result from one element of each argument.
fn replace_element(
    old_text: &CalcResult,
    start_num: Result<f64, Error>,
    num_chars: Result<f64, Error>,
    new_text: &CalcResult,
) -> Result<ArrayNode, Error> {
    // Every argument is coerced before the range checks: an error argument beats a start or length
    // failure (docs/excel-semantics/arrays.md, probes/error-arg-vs-range-check, T-1056).
    let old_text = calc_result_to_text(old_text)?;
    let start_num = start_num?;
    let num_chars = num_chars?;
    let new_text = calc_result_to_text(new_text)?;
    super::common::reject_over_limit(&old_text)?;
    // Excel: both numbers count as whole like LEFT's (near an integer they round up, so
    // `REPLACE(t,1-2E-7,1,"z")` starts at 1) and either one above 32,767 is #VALUE!, even
    // past the end of a short text (probes/text-count-argument-bounds J, K, U, F22, F24).
    let start_num = near_integer_count(start_num);
    if num_chars < 0.0 || !(1.0..=MAX_TEXT_UNITS as f64).contains(&start_num) {
        return Err(Error::VALUE);
    }
    let num_chars = near_integer_count(num_chars);
    if num_chars > MAX_TEXT_UNITS as f64 {
        return Err(Error::VALUE);
    }

    // Excel: positions count UTF-16 code units, so `REPLACE("a😀b",2,2,"x")` is "axb"
    // (probes/text-surrogate-positions A27-A31).
    let len = old_text.len_utf16();
    // start_num is 1-indexed; clamp to valid range
    let start = (start_num as usize).saturating_sub(1).min(len);
    let end = start.saturating_add(num_chars as usize).min(len);
    // The lone half of a pair either bound cuts is kept (probes/text-surrogate-positions A29).
    let mut result = old_text.slice_utf16(0, start);
    result.push(&new_text);
    result.push(&old_text.slice_utf16(end, len));
    Ok(ArrayNode::String(result))
}

/// Characters Excel's UPPER leaves unchanged although Rust maps them (text.md "UPPER and LOWER use an old
/// per-character case table", probes/t3298-case-mapping-sweep). Sorted, inclusive code point ranges.
const UPPER_UNMAPPED: &[(u32, u32)] = &[
    (0x00B5, 0x00B5),
    (0x017F, 0x017F),
    (0x019B, 0x019B),
    (0x023F, 0x0240),
    (0x0250, 0x0252),
    (0x025C, 0x025C),
    (0x0261, 0x0261),
    (0x0264, 0x0266),
    (0x026A, 0x026A),
    (0x026C, 0x026C),
    (0x0271, 0x0271),
    (0x0282, 0x0282),
    (0x0287, 0x0287),
    (0x029D, 0x029E),
    (0x0345, 0x0345),
    (0x0371, 0x0373),
    (0x0377, 0x0377),
    (0x037B, 0x037D),
    (0x03D7, 0x03D9),
    (0x03F2, 0x03F5),
    (0x03F8, 0x03F8),
    (0x03FB, 0x03FB),
    (0x0515, 0x052F),
    (0x10D0, 0x10FA),
    (0x10FD, 0x10FF),
    (0x13F8, 0x13FD),
    (0x1C80, 0x1C8A),
    (0x1D79, 0x1D79),
    (0x1D8E, 0x1D8E),
    (0x1E9B, 0x1E9B),
    (0x1EFB, 0x1F07),
    (0x1F10, 0x1F15),
    (0x1F20, 0x1F27),
    (0x1F30, 0x1F37),
    (0x1F40, 0x1F45),
    (0x1F51, 0x1F51),
    (0x1F53, 0x1F53),
    (0x1F55, 0x1F55),
    (0x1F57, 0x1F57),
    (0x1F60, 0x1F67),
    (0x1F70, 0x1F7D),
    (0x1FB0, 0x1FB1),
    (0x1FBE, 0x1FBE),
    (0x1FD0, 0x1FD1),
    (0x1FE0, 0x1FE1),
    (0x1FE5, 0x1FE5),
    (0x2170, 0x217F),
    (0x2C30, 0x2C5F),
    (0x2C73, 0x2C73),
    (0x2C81, 0x2CE3),
    (0x2CEC, 0x2CEE),
    (0x2CF3, 0x2CF3),
    (0x2D00, 0x2D27),
    (0x2D2D, 0x2D2D),
    (0xA641, 0xA66D),
    (0xA681, 0xA69B),
    (0xA723, 0xA72F),
    (0xA733, 0xA76F),
    (0xA77A, 0xA77C),
    (0xA77F, 0xA787),
    (0xA78C, 0xA78C),
    (0xA791, 0xA794),
    (0xA797, 0xA7A9),
    (0xA7B5, 0xA7C3),
    (0xA7C8, 0xA7CA),
    (0xA7CD, 0xA7DB),
    (0xA7F6, 0xA7F6),
    (0xAB53, 0xAB53),
    (0xAB70, 0xABBF),
];

/// Characters Excel's LOWER leaves unchanged although Rust maps them (same evidence). Sorted, inclusive ranges.
const LOWER_UNMAPPED: &[(u32, u32)] = &[
    (0x0370, 0x0372),
    (0x0376, 0x0376),
    (0x037F, 0x037F),
    (0x03CF, 0x03CF),
    (0x03D8, 0x03D8),
    (0x03F4, 0x03F4),
    (0x03F7, 0x03FA),
    (0x03FD, 0x03FF),
    (0x0514, 0x052E),
    (0x10C7, 0x10C7),
    (0x10CD, 0x10CD),
    (0x13A0, 0x13F5),
    (0x1C89, 0x1C89),
    (0x1C90, 0x1CBA),
    (0x1CBD, 0x1CBF),
    (0x1E9E, 0x1E9E),
    (0x1EFA, 0x1EFE),
    (0x1F08, 0x1F0F),
    (0x1F18, 0x1F1D),
    (0x1F28, 0x1F2F),
    (0x1F38, 0x1F3F),
    (0x1F48, 0x1F4D),
    (0x1F59, 0x1F5F),
    (0x1F68, 0x1F6F),
    (0x1F88, 0x1F8F),
    (0x1F98, 0x1F9F),
    (0x1FA8, 0x1FAF),
    (0x1FB8, 0x1FBC),
    (0x1FC8, 0x1FCC),
    (0x1FD8, 0x1FDB),
    (0x1FE8, 0x1FEC),
    (0x1FF8, 0x1FFC),
    (0x2126, 0x2126),
    (0x212A, 0x212B),
    (0x2160, 0x216F),
    (0x2C00, 0x2C2F),
    (0x2C6D, 0x2C72),
    (0x2C7E, 0x2CE2),
    (0x2CEB, 0x2CED),
    (0x2CF2, 0x2CF2),
    (0xA640, 0xA66C),
    (0xA680, 0xA69A),
    (0xA722, 0xA72E),
    (0xA732, 0xA76E),
    (0xA779, 0xA786),
    (0xA78B, 0xA78D),
    (0xA790, 0xA792),
    (0xA796, 0xA7DC),
    (0xA7F5, 0xA7F5),
];

/// Whether `c` lies in one of the sorted inclusive `ranges`.
fn in_ranges(ranges: &[(u32, u32)], c: char) -> bool {
    let cp = c as u32;
    let i = ranges.partition_point(|&(_, hi)| hi < cp);
    ranges.get(i).is_some_and(|&(lo, _)| lo <= cp)
}

/// Excel's per-character case mapping for PROPER, UPPER and LOWER: the simple (one-to-one) mapping only, so ß, ŉ,
/// ǰ and the ﬁ ligatures keep their form where Rust would expand them ("SS", "ʼN"); İ folds to
/// ASCII I (upper) and i (lower), and ΐ, ΰ upper-case to Ϊ, Ϋ (probes/clean-proper-chars
/// D4, D14:D18, D23:D24). UPPER and LOWER use the same mapping (probes/case-mapping-expanding
/// A1:C13).
fn simple_case(c: char, upper: bool) -> char {
    match (c, upper) {
        ('İ', true) => 'I',
        ('İ', false) => 'i',
        ('ΐ', true) => 'Ϊ',
        ('ΰ', true) => 'Ϋ',
        // Excel drops the tonos in UPPER (Rust keeps it: ά to Ά).
        ('ά', true) => 'Α',
        ('έ', true) => 'Ε',
        ('ή', true) => 'Η',
        ('ί', true) => 'Ι',
        ('ό', true) => 'Ο',
        ('ύ', true) => 'Υ',
        ('ώ', true) => 'Ω',
        // Asomtavruli lowers to Mkhedruli (Rust: Nuskhuri).
        ('\u{10A0}'..='\u{10C5}', false) => char::from_u32(c as u32 + 0x30).unwrap_or(c),
        // No supplementary letter changes case in Excel; surrogate pairs are never mapped.
        _ if c as u32 > 0xFFFF => c,
        _ if in_ranges(
            if upper {
                UPPER_UNMAPPED
            } else {
                LOWER_UNMAPPED
            },
            c,
        ) =>
        {
            c
        }
        _ => {
            let mut mapped: Vec<char> = if upper {
                c.to_uppercase().collect()
            } else {
                c.to_lowercase().collect()
            };
            match (mapped.pop(), mapped.is_empty()) {
                (Some(one), true) => one,
                _ => c,
            }
        }
    }
}

/// Whether a capital sigma followed by `next` lowers to the final ς: only at the end of the text or
/// before one of 30 ASCII characters (TAB, LF, CR, space and ``! " # $ % & ' + , - . / : ; < = > ?
/// @ [ \ ] _ { | }``); every other character, letters, marks, U+00A0, U+2018, emoji included, gives σ
/// (probes/t3820-lower-sigma-next-char, Calc!A1:C50 and the whole-BMP sweep E1:F256). The rule is
/// the same in PROPER, LOWER and TEXTBEFORE/TEXTAFTER's case-insensitive mode.
pub(super) fn sigma_is_final(next: Option<char>) -> bool {
    next.is_none_or(|n| {
        matches!(
            n,
            '\t' | '\n'
                | '\r'
                | ' '
                | '!'
                | '"'
                | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '+'
                | ','
                | '-'
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '='
                | '>'
                | '?'
                | '@'
                | '['
                | '\\'
                | ']'
                | '_'
                | '{'
                | '|'
                | '}'
        )
    })
}

/// Excel's LOWER: the simple mapping per character, with the final-sigma rule.
pub(super) fn lower_case(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if c == 'Σ' && sigma_is_final(chars.get(i + 1).copied()) {
                'ς'
            } else {
                simple_case(c, false)
            }
        })
        .collect()
}

/// Excel's UPPER: the simple mapping per character (ß, ŉ, ǰ, ligatures stay as they are). Unlike
/// PROPER, UPPER leaves İ alone (probes/case-mapping-expanding B12).
pub(super) fn upper_case(s: &str) -> String {
    s.chars()
        .map(|c| if c == 'İ' { c } else { simple_case(c, true) })
        .collect()
}

/// PROPER's case for a letter that starts a word. It is UPPER's table with two exceptions:
/// the Greek tonos vowels keep their tonos (`PROPER("άλφα")` is Άλφα, while `UPPER("ά")` is Α),
/// and Georgian Asomtavruli U+10A0..=U+10C5 lowers to Mkhedruli as LOWER does (`PROPER("Ⴀბ")`
/// is აბ) (docs/excel-semantics/text.md "PROPER and REGEXREPLACE case forcing";
/// probes/t3777-proper-case-table A9:A10, C1:C7, C12, D15:D16, E1).
fn proper_initial(c: char) -> char {
    match c {
        'ά' => 'Ά',
        'έ' => 'Έ',
        'ή' => 'Ή',
        'ί' => 'Ί',
        'ό' => 'Ό',
        'ύ' => 'Ύ',
        'ώ' => 'Ώ',
        '\u{10A0}'..='\u{10C5}' => simple_case(c, false),
        _ => simple_case(c, true),
    }
}

/// A word start is a letter after a non-letter, digits included ("2Nd"). Letters are Excel's own
/// table (`is_proper_letter`): many Unicode letters and marks are not letters to PROPER, and every
/// non-letter is lowered, so a word-initial Ⓐ becomes ⓐ (probes/t3657-proper-word-starts B21:D21,
/// probes/t3657-proper-letter-sweep). A capital sigma ending a word (see `sigma_is_final`) lowers to
/// the final ς: "ΑΣ.ΑΣ-Σ ΑΣ1" is "Ας.Ας-Σ Ασ1" (probes/clean-proper-chars D7, D16, D25).
fn proper_case(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for (i, &c) in chars.iter().enumerate() {
        let letter = is_proper_letter(c);
        if letter && capitalize_next {
            result.push(proper_initial(c));
        } else if c == 'Σ' && sigma_is_final(chars.get(i + 1).copied()) {
            result.push('ς');
        } else {
            result.push(simple_case(c, false));
        }
        capitalize_next = !letter;
    }
    result
}

/// NUMBERVALUE's text once its whitespace and group separators are gone. Every space, tab, line
/// feed and carriage return goes first, wherever it is and even when it is a separator, so
/// `NUMBERVALUE("1 5"," ",",")` is 15; other control characters and spaces (NBSP, U+3000, U+202F)
/// stay: "1"&CHAR(9)&"234.5" is 1234.5 and "1"&CHAR(160)&"234.5" #VALUE!. Then the group
/// character goes only before the first decimal separator: after it, it stays, so a digit reads
/// as a digit and anything else is #VALUE! (`NUMBERVALUE("1.5,2")` is #VALUE!, `("17582","7","8")`
/// 1.582, `("1,2.3")` 12.3). (docs/excel-semantics/text.md "NUMBERVALUE removes group separators
/// only before the decimal separator"; probes/t3647-numbervalue-separator-placement rows 3, 22-27,
/// 37, 39-41, probes/t3647-numbervalue-percent-and-controls A17-A29,
/// probes/t3647-numbervalue-whitespace-decimal A1-A12.)
fn numbervalue_text(text: &str, decimal_sep: char, group_sep: char) -> String {
    let text: String = text
        .chars()
        .filter(|c| !matches!(c, ' ' | '\t' | '\n' | '\r'))
        .collect();
    let (head, tail) = text
        .find(decimal_sep)
        .map_or((text.as_str(), ""), |i| text.split_at(i));
    head.chars()
        .filter(|&c| c != group_sep)
        .chain(tail.chars())
        .collect()
}

/// The text without its one leading "%", which may follow a leading sign or "(", or `None` when
/// it has none (probes/t3647-numbervalue-percent-and-controls A1-A8).
fn strip_leading_percent(text: &str) -> Option<String> {
    if let Some(rest) = text.strip_prefix('%') {
        return Some(rest.to_string());
    }
    let mut chars = text.chars();
    let first = chars.next().filter(|c| matches!(c, '-' | '+' | '('))?;
    let rest = chars.as_str().strip_prefix('%')?;
    Some(format!("{first}{rest}"))
}

#[cfg(test)]
mod tests {
    use super::{format_abs, lower_case, sigma_is_final};

    // Expected strings come from docs/excel-semantics/text.md "FIXED, DOLLAR and BAHTTEXT cut to
    // 15 significant digits first" (excel-graph, gen-m5) and from ROUND's decimal rules.
    #[test]
    fn cuts_to_15_digits_before_rounding() {
        assert_eq!(format_abs(1.005, 2, true), "1.01");
        assert_eq!(
            format_abs(9_007_199_254_740_993.0, 0, true),
            "9,007,199,254,740,990"
        );
        assert_eq!(
            format_abs(999_999_999_999_998.1, 2, true),
            "999,999,999,999,998.00"
        );
        assert_eq!(
            format_abs(4_999_999_999_999_995.0, 2, true),
            "4,999,999,999,999,990.00"
        );
    }

    // Classes from docs/excel-semantics/text.md (T-3820, probes/t3820-lower-sigma-next-char).
    #[test]
    fn sigma_is_final_only_before_the_listed_separators() {
        assert!(sigma_is_final(None));
        for c in ['\t', ' ', '-', '\'', '}', '_'] {
            assert!(sigma_is_final(Some(c)), "{c:?}");
        }
        for c in [
            '(', '\u{a0}', '\u{301}', '\u{2018}', '\u{2019}', '😀', 'a', '1',
        ] {
            assert!(!sigma_is_final(Some(c)), "{c:?}");
        }
        assert_eq!(lower_case("Σ(Σ"), "σ(ς");
    }

    #[test]
    fn carries_into_a_new_leading_digit() {
        assert_eq!(format_abs(999.9996, 3, true), "1,000.000");
        assert_eq!(format_abs(0.999, 2, false), "1.00");
        assert_eq!(format_abs(0.0, 2, true), "0.00");
        assert_eq!(format_abs(0.004, 2, true), "0.00");
    }

    #[test]
    fn negative_decimals_round_the_integer_part() {
        assert_eq!(format_abs(5.0, -1, true), "10");
        assert_eq!(format_abs(4.0, -1, true), "0");
        assert_eq!(format_abs(500.0, -3, true), "1,000");
        assert_eq!(format_abs(15.0, -5, true), "0");
        assert_eq!(format_abs(1234.0, -2, false), "1200");
        assert_eq!(format_abs(1.0, -1000, true), "0");
    }

    #[test]
    fn largest_doubles_stay_bounded() {
        let text = format_abs(f64::MAX, 127, false);
        assert_eq!(text.len(), 309 + 1 + 127);
        assert!(text.starts_with("17976931348623200000"));
    }
}

#[cfg(test)]
mod case_table_tests {
    use super::{lower_case, upper_case};

    // text.md "UPPER and LOWER use an old per-character case table" (probes/t3298-case-mapping-sweep).
    #[test]
    fn upper_leaves_old_table_gaps() {
        for s in [
            "µ", "ſ", "ა", "ⰰ", "ⲁ", "ꭰ", "ⅰ", "ἀ", "𐐨", "ԯ", "\u{250}", "\u{29D}", "\u{371}",
            "\u{3F2}", "\u{3FB}", "\u{A641}", "\u{1C80}", "\u{13F8}", "\u{A723}", "\u{A7B5}",
            "\u{A796}",
        ] {
            assert_eq!(upper_case(s), s);
        }
        assert_eq!(upper_case("abc"), "ABC");
    }

    #[test]
    fn upper_drops_greek_tonos() {
        assert_eq!(upper_case("άέήίόύώ"), "ΑΕΗΙΟΥΩ");
        assert_eq!(upper_case("ΐΰ"), "ΪΫ");
    }

    #[test]
    fn lower_leaves_old_table_gaps() {
        for s in [
            "\u{1E9E}", "\u{2126}", "\u{212A}", "\u{212B}", "Ⅰ", "Ⰰ", "Ꭰ", "𐐀", "Ͱ",
        ] {
            assert_eq!(lower_case(s), s);
        }
    }

    #[test]
    fn lower_georgian_to_mkhedruli() {
        assert_eq!(lower_case("\u{10A0}\u{10C5}"), "\u{10D0}\u{10F5}");
    }
}
