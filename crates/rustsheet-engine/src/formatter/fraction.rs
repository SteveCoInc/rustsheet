//! Fraction sections with a fixed denominator (`# ?/4`, `# ??/16`, `?/100`).
//!
//! Rules from Excel's displayed text (docs/excel-semantics/number-formats.md "Fraction formats";
//! eval/suites/gen-m10 `F!` cells, excel-graph):
//! - the numerator is the fraction part times the denominator, rounded half away from zero at 15
//!   significant digits and never reduced (`# ?/4` on 1.5 is "1 2/4", `?/100` on 12.345 is
//!   "1235/100"); with no integer part the whole value goes into the numerator ("150/100");
//! - a numerator that rounds up to the denominator carries into the integer (`# ?/2` on 1.75 is
//!   "2    "), and a zero numerator blanks the fraction to spaces of its width, the integer then
//!   printing even under `#` (`# ?/4` on 1E-5 is "0    ");
//! - a zero integer part prints as its placeholders do with no digit (`#` nothing, `?` a space,
//!   `0` a zero: `# ?/4` on 0.5 is " 2/4");
//! - the minus leads the whole text, also when the value rounds to zero (`# ?/2` on -1/7 is
//!   "-0    "), except with no integer part, where a zero numerator takes no minus (`?/100` on
//!   -0.001 is "0/100", probes/fraction-format-features F9, F13);
//! - a comma between integer placeholders groups the integer digits (`#,##0 ?/4` on 1234.5 is
//!   "1,234 2/4", probes/fraction-format-features I10);
//! - with no integer part a numerator past Excel's bound cannot be shown ([`format_fraction`]
//!   returns `None`);
//! - a `,` right after the denominator scales the value up ([`fraction_scale`]), and a fixed
//!   denominator past 32768 or followed by a digit placeholder is replaced by a picked one
//!   ([`Choice`], [`LARGE_FIXED_MAX_DENOMINATOR`]).

use super::parser::{NumberPart, TextToken};
use crate::decimal_round::{round_decimal, RoundMode};
use crate::number_format::{group_thousands, to_precision};

/// Where a fraction sits among a number section's `tokens`: the digit placeholders before
/// `numerator_start` are the integer part, those from it up to `numerator_end` the numerator.
/// Literals may sit on either side of the `/` (`#\:#=/=#`).
pub struct Fraction {
    /// Index of the first numerator placeholder in `tokens`.
    pub numerator_start: usize,
    /// Index just past the last numerator placeholder in `tokens`.
    pub numerator_end: usize,
    /// Index in `tokens` where the denominator prints, after the literals that follow the `/`;
    /// the denominator is not a token.
    pub denominator_at: usize,
    /// Fixed digits or placeholder kinds; see [`Denominator`].
    pub denominator: Denominator,
}

/// The denominator of a fraction section.
pub enum Denominator {
    /// Digits as written (`16` in `# ??/16`).
    Fixed(String),
    /// The kinds of its digit placeholders (`??` in `# ??/??`): Excel picks the fraction.
    Placeholders(Vec<char>),
}

/// The text of `value` under the number section `part`, which holds `fraction`, or `None` when
/// Excel cannot show it: with no integer part, a fixed denominator's numerator past 32767 and a
/// placeholder denominator's value from 2147483647 on, either sign (`?/100` on 327.67 is
/// "32767/100" and on 327.68 255 `#`, `TEXT` `#VALUE!`; `?/?` on 2147483646.9 is "2147483647/1"
/// and on 2147483647 255 `#`; probes/fraction-numerator-overflow, -overflow2). A whole part takes
/// the rest, so `# ?/100` and `# ?/?` show 1E15.
pub(super) fn format_fraction(
    value: f64,
    part: &NumberPart,
    fraction: &Fraction,
    group_separator: &str,
) -> Option<String> {
    let negative = value < 0.0;
    let mut value = value.abs() * 100.0_f64.powi(part.percent);
    if part.comma > 0 {
        // The scaled value is cut to 15 significant digits: `?/?,,` on 1E-251 is "0/1", where the
        // double product (0.10000000000000002) would pick 1/9 (probes/fraction-overflow-scaled A65).
        value = to_precision(value * fraction_scale(part.comma), 15);
    }
    // A scaled value past f64's range would print past 255 characters, which Excel cannot show.
    if !value.is_finite() {
        return None;
    }
    let layout = Layout::new(&part.tokens, fraction);
    let choice = Choice::of(
        &fraction.denominator,
        layout.trailing_end > layout.denominator_at,
    );
    let shown = choose(value, layout.has_integer, &choice)?;
    let plan = plan(
        part,
        fraction,
        &layout,
        &choice,
        &shown,
        negative,
        group_separator,
    );
    Some(emit(part, &layout, plan))
}

/// How a fraction's denominator is chosen.
enum Choice {
    /// A fixed denominator up to 32768 with no digit placeholder after it, used as written.
    AsWritten(f64),
    /// The last convergent with a denominator up to the bound: for placeholders, 10^n - 1 for
    /// n of them; for any other fixed denominator, [`LARGE_FIXED_MAX_DENOMINATOR`].
    Picked(f64),
}

impl Choice {
    /// The choice for `denominator`, in a section with digit placeholders after it or not
    /// (`trailing_digits`). Excel picks the fraction for a fixed denominator past 32768 and for
    /// one with a placeholder anywhere after it (`# ?/4|0` on 0.0625 is " 1/4|60", 1/16;
    /// `# ?/4"x"` on 0.5 " 2/4x"; probes/t3711-fixed-denominator-then-placeholder,
    /// t4043-fixed-denominator-slots).
    fn of(denominator: &Denominator, trailing_digits: bool) -> Self {
        match denominator {
            // The parser only builds a fixed denominator from ASCII digits starting with 1-9.
            Denominator::Fixed(digits) => match digits.parse::<f64>() {
                Ok(d) if d <= MAX_FIXED_DENOMINATOR && !trailing_digits => Choice::AsWritten(d),
                _ => Choice::Picked(LARGE_FIXED_MAX_DENOMINATOR),
            },
            // Past 15 digits 10^n - 1 is no longer exact in a double; no such bound can bind.
            Denominator::Placeholders(kinds) => {
                let digits = i32::try_from(kinds.len()).unwrap_or(i32::MAX).min(15);
                Choice::Picked(10.0_f64.powi(digits) - 1.0)
            }
        }
    }
}

/// The integer, numerator and denominator a value shows.
struct Shown {
    integer: f64,
    numerator: f64,
    denominator: f64,
}

/// The fraction of `value >= 0` under `choice`, or `None` when it cannot be shown (see
/// [`format_fraction`]): with an integer part the fraction is of the fraction part, and a numerator
/// that rounds up to the denominator carries into the integer.
fn choose(value: f64, has_integer: bool, choice: &Choice) -> Option<Shown> {
    let mut integer = if has_integer { value.trunc() } else { 0.0 };
    let (mut numerator, denominator) = match *choice {
        Choice::AsWritten(denominator) => (
            round_numerator((value - integer) * denominator),
            denominator,
        ),
        Choice::Picked(max_denominator) => last_convergent(value - integer, max_denominator),
    };
    let overflows = match choice {
        Choice::AsWritten(_) => numerator > MAX_FIXED_NUMERATOR,
        Choice::Picked(_) => value >= MAX_PLACEHOLDER_VALUE,
    };
    if !has_integer && overflows {
        return None;
    }
    if has_integer && numerator >= denominator {
        integer += 1.0;
        numerator = 0.0;
    }
    Some(Shown {
        integer,
        numerator,
        denominator,
    })
}

/// Which part of a fraction section a token belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Zone {
    /// Up to the last integer placeholder (or the numerator, with no integer part).
    Integer,
    /// The literals between the integer part and the numerator.
    Separator,
    /// The numerator's placeholders.
    Numerator,
    /// The `/` and the literals around it; the denominator prints at the zone's end.
    Denominator,
    /// Everything after the denominator.
    Trailing,
}

/// Where the zones of a fraction section's tokens start and end.
struct Layout {
    has_integer: bool,
    /// Just past the last integer placeholder.
    integer_end: usize,
    numerator_start: usize,
    numerator_end: usize,
    denominator_at: usize,
    /// Just past the last placeholder after the denominator: a blank fraction blanks up to there.
    trailing_end: usize,
}

impl Layout {
    fn new(tokens: &[TextToken], fraction: &Fraction) -> Self {
        let integer_end = digits_end(&tokens[..fraction.numerator_start]);
        Layout {
            has_integer: integer_end.is_some(),
            integer_end: integer_end.unwrap_or(0),
            numerator_start: fraction.numerator_start,
            numerator_end: fraction.numerator_end,
            denominator_at: fraction.denominator_at,
            trailing_end: digits_end(tokens)
                .filter(|end| *end > fraction.denominator_at)
                .unwrap_or(fraction.denominator_at),
        }
    }

    fn zone(&self, index: usize) -> Zone {
        if index >= self.denominator_at {
            Zone::Trailing
        } else if index >= self.numerator_end {
            Zone::Denominator
        } else if index >= self.numerator_start {
            Zone::Numerator
        } else if self.has_integer && index >= self.integer_end {
            Zone::Separator
        } else {
            Zone::Integer
        }
    }

    /// Whether a blank fraction blanks the token at `index` with it.
    fn blanks(&self, index: usize) -> bool {
        match self.zone(index) {
            Zone::Numerator | Zone::Denominator => true,
            Zone::Trailing => index < self.trailing_end,
            Zone::Integer | Zone::Separator => false,
        }
    }
}

/// How the chosen denominator prints: the kinds of its slots, its text, and for a denominator
/// with `?` slots the digits it hands to the placeholders after it ([`trailing_fill`]).
struct DenominatorPlan {
    kinds: Vec<char>,
    text: String,
    spill: Option<Spill>,
}

/// The digits of a picked denominator past its slots, and how many more digits than slots it has
/// (negative when it has fewer: a picked 2 under a written `16`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Spill {
    digits: String,
    excess: isize,
}

impl DenominatorPlan {
    /// The plan for `denominator` chosen by `choice` as `digits`, under a numerator whose last
    /// placeholder is `numerator_kind`. A fixed denominator used as written prints as written. A
    /// picked fixed one prints the code's own digits cut to the picked denominator's length,
    /// as placeholders that pad with spaces under a `?` numerator and with nothing otherwise
    /// (512/625 under `?/40000` is "512/400  ", `0/40000` and `#/40000` on 0.5 "1/4"); its further
    /// digits print after them under a `#` or `0` numerator (`# 0/4|0` on 0.3 is "3/40|0", on
    /// 0.0625 "1/46|0") and go to the placeholders after the denominator under a `?` one, or
    /// nowhere (4095937/5000000 under `?/40000` is "4095937/40000";
    /// probes/t4043-fixed-denominator-slots). Placeholders fill from the left.
    fn new(
        denominator: &Denominator,
        choice: &Choice,
        digits: &str,
        numerator_kind: Option<char>,
    ) -> Self {
        let (kinds, written) = match (denominator, choice) {
            (Denominator::Fixed(written), Choice::AsWritten(_)) => {
                return DenominatorPlan {
                    kinds: Vec::new(),
                    text: written.clone(),
                    spill: None,
                };
            }
            (Denominator::Fixed(written), Choice::Picked(_)) => {
                let kind = if numerator_kind == Some('?') {
                    '?'
                } else {
                    '#'
                };
                (vec![kind; written.len()], Some(written))
            }
            (Denominator::Placeholders(kinds), _) => (kinds.clone(), None),
        };
        let spill = Spill {
            digits: digits.chars().skip(kinds.len()).collect(),
            excess: digits.len().cast_signed() - kinds.len().cast_signed(),
        };
        let mut text = match written {
            Some(written) => fill_left(
                &written.chars().take(digits.len()).collect::<String>(),
                &kinds,
            ),
            None => fill_left(digits, &kinds),
        };
        let spill = if kinds.contains(&'?') {
            Some(spill)
        } else {
            text.push_str(&spill.digits);
            None
        };
        DenominatorPlan { kinds, text, spill }
    }
}

/// Everything a fraction section prints, prepared before its tokens are walked ([`emit`]).
struct Plan {
    minus: bool,
    integer: Vec<String>,
    numerator: Vec<String>,
    denominator: String,
    trailing: Vec<String>,
    /// A zero numerator blanks the fraction ([`Layout::blanks`]), to spaces or to nothing.
    blank: Option<bool>,
    separator: Separator,
}

/// The [`Plan`] of `shown` under `part`.
fn plan(
    part: &NumberPart,
    fraction: &Fraction,
    layout: &Layout,
    choice: &Choice,
    shown: &Shown,
    negative: bool,
    group_separator: &str,
) -> Plan {
    let tokens = &part.tokens;
    let Shown {
        integer,
        numerator,
        denominator,
    } = *shown;
    let integer_kinds = placeholder_kinds(&tokens[..layout.numerator_start]);
    let numerator_kinds = placeholder_kinds(&tokens[layout.numerator_start..layout.numerator_end]);
    let denominator = DenominatorPlan::new(
        &fraction.denominator,
        choice,
        &format!("{denominator}"),
        numerator_kinds.last().copied(),
    );
    // A zero numerator blanks the fraction unless a `0` placeholder prints it (`# 00/16` on 0 is
    // "00/16"; `#\:0=/=#` on 1 "1:0=/=1", poi/NumberFormatTests A258).
    let blank = layout.has_integer && numerator == 0.0 && !numerator_kinds.contains(&'0');
    let last_integer_kind = integer_kinds.last().copied();
    // A blank fraction prints as spaces of its width, or as nothing under a `#` numerator with
    // neither a `?` integer nor a `?` in the denominator (`#\:#=/=?` and `?\:#=/=#` on 1 are
    // "1      ", `#\:#=/=#`, `#\:#=/=0` and `0\:#=/=#` "1"; poi/NumberFormatTests A253, A261,
    // A252, A254, A270).
    let blank_spaces = numerator_kinds.first() != Some(&'#')
        || denominator.kinds.contains(&'?')
        || last_integer_kind == Some('?');
    let integer_digits = match (integer == 0.0, numerator == 0.0) {
        // A blank fraction leaves the integer printing even under `#`; a printed zero numerator
        // drops a zero integer under `#` and keeps it under `?` (`#\:0=/=#` on 0 is "0=/=1",
        // `?\:0=/=#` "0:0=/=1"; poi/NumberFormatTests A204, A213).
        (true, true) if blank || last_integer_kind != Some('#') => "0".to_owned(),
        (true, _) => String::new(),
        // Past 15 significant digits Excel prints zeros, as for any number format.
        (false, _) => {
            let digits = format!("{}", to_precision(integer, 15));
            if part.use_thousands {
                group_thousands(&digits, group_separator)
            } else {
                digits
            }
        }
    };
    let numerator_digits = if numerator == 0.0 {
        "0".to_owned()
    } else {
        format!("{numerator}")
    };
    let blank = blank.then_some(blank_spaces);
    // A numeral byte maps every digit the fraction prints (`[$-1B000409]# ?/?` on 12.5 is
    // "一二 一/二", `# ?/16` "一二 八/一六"; probes/t3532-numeral-digit-fields C14,
    // t3532-numeral-digit-sections A8).
    let numerals = |digits: Vec<String>| -> Vec<String> {
        digits.iter().map(|d| part.numerals.digits(d)).collect()
    };
    Plan {
        minus: negative && (layout.has_integer || numerator != 0.0),
        separator: separator_kind(
            blank,
            !integer_digits.is_empty(),
            last_integer_kind,
            numerator_kinds.first().copied(),
        ),
        integer: numerals(fill(&integer_digits, &integer_kinds)),
        numerator: numerals(fill(&numerator_digits, &numerator_kinds)),
        trailing: numerals(trailing_fill(
            &placeholder_kinds(&tokens[layout.denominator_at..]),
            denominator.spill.as_ref(),
        )),
        denominator: part.numerals.digits(&denominator.text),
        blank,
    }
}

/// The text of `plan`, walking `part`'s tokens zone by zone. The arms are disjoint: a blank
/// fraction takes the zones it blanks whole (never the integer or separator), the separator is
/// printed, spaced or dropped whole, and every other placeholder takes the next digits of its
/// zone.
fn emit(part: &NumberPart, layout: &Layout, plan: Plan) -> String {
    let tokens = &part.tokens;
    let mut text = String::new();
    if plan.minus {
        text.push('-');
    }
    if let Some(c) = part.currency {
        text.push(c);
    }
    let push_denominator = |text: &mut String| match plan.blank {
        None => text.push_str(&plan.denominator),
        Some(true) => text.extend(plan.denominator.chars().map(|_| ' ')),
        Some(false) => {}
    };
    let mut integer = plan.integer.iter();
    let mut numerator = plan.numerator.iter();
    let mut trailing = plan.trailing.iter();
    for (index, token) in tokens.iter().enumerate() {
        if index == layout.denominator_at {
            push_denominator(&mut text);
        }
        let zone = layout.zone(index);
        match (zone, token) {
            // Every placeholder and literal of a blank fraction takes its width in spaces, and so
            // do those after the denominator up to its last placeholder (`# ?/?|0 "12"` on 12 is
            // "12       12"; probes/t3711-placeholders-after-fraction B1:B20).
            _ if plan.blank.is_some() && layout.blanks(index) => {
                if plan.blank == Some(true) {
                    push_spaces(&mut text, token);
                }
            }
            (Zone::Separator, _) if plan.separator == Separator::Dropped => {}
            (Zone::Separator, _) if plan.separator == Separator::Spaces => {
                push_spaces(&mut text, token);
            }
            (Zone::Integer, TextToken::Digit(_)) => text.extend(integer.next().map(String::as_str)),
            (Zone::Numerator, TextToken::Digit(_)) => {
                text.extend(numerator.next().map(String::as_str));
            }
            (Zone::Trailing, TextToken::Digit(_)) => {
                text.extend(trailing.next().map(String::as_str));
            }
            // A `.` after the denominator prints (`# ?/?|0.0` on 12.5 is "12 1/2|0.0").
            (Zone::Trailing, TextToken::Period) => text.push('.'),
            (_, TextToken::Literal(c)) => text.push(*c),
            (_, TextToken::Text(t)) => text.push_str(t),
            (_, TextToken::Ghost(_)) => text.push(' '),
            // Excel: a `*x` fill adds nothing, `TEXT(1.5,"# ?/?*x")` is "1 1/2"
            // (docs/excel-semantics/number-formats.md "Fill characters"; T-2480).
            _ => {}
        }
    }
    if layout.denominator_at == tokens.len() {
        push_denominator(&mut text);
    }
    text
}

/// What each placeholder after the denominator prints, for their `kinds`; they get no digit of
/// the value (number-formats.md "Placeholders after a fraction"). After a denominator without `?`
/// slots (`spill` `None`: one used as written, or a picked fixed one under a `#` or `0`
/// numerator) `0` and `#` print a zero and `?` a space (`# 0/4|0` on 0.3 is "3/40|0",
/// `# 0/16|?` on 0.01 "1/160| ").
///
/// After a denominator with `?` slots, `spill` holds its digits past the slots and how many more
/// digits than slots it has (negative when it has fewer):
/// - all `?`: the spilled digits, or one `0` per missing digit, fill them from the left and a
///   placeholder with none prints a space (`# ?/4|?` on 0.0625 is " 1/4|6", `# ?/16|?` on 0.5
///   " 1/1 |0", `# ?/100|??` on 0.3 " 3/10 |0 ", `# ??/??|?` on 0.5 "  1/2 |0", on 0.3 "  3/10| ");
/// - otherwise the spilled digits print before the first one, then `0` prints a zero, `#` a zero
///   unless digits are missing, and `?` a zero unless the slots fit exactly, where it prints a
///   space until a placeholder before it printed a zero (`# ?/4|0` on 0.0625 is " 1/4|60",
///   `# ?/16|#` on 0.5 " 1/1 |", `# ?/?|?0` " 1/2| 0", `# ?/?|0?` " 1/2|00";
///   probes/t4043-fixed-denominator-slots, -cut, t4043-placeholder-denominator-slots).
fn trailing_fill(kinds: &[char], spill: Option<&Spill>) -> Vec<String> {
    let Some(Spill { digits, excess }) = spill else {
        return kinds
            .iter()
            .map(|kind| if *kind == '?' { " " } else { "0" }.to_owned())
            .collect();
    };
    let excess = *excess;
    if kinds.iter().all(|kind| *kind == '?') {
        let digits: Vec<char> = if excess < 0 {
            vec!['0'; excess.unsigned_abs()]
        } else {
            digits.chars().collect()
        };
        return (0..kinds.len())
            .map(|i| digits.get(i).copied().unwrap_or(' ').to_string())
            .collect();
    }
    let mut zero_before = false;
    let mut slots: Vec<String> = kinds
        .iter()
        .map(|kind| {
            let zero = match kind {
                '#' => excess >= 0,
                '?' => excess != 0 || zero_before,
                _ => true,
            };
            zero_before |= zero;
            match (zero, *kind) {
                (true, _) => "0",
                (false, '?') => " ",
                (false, _) => "",
            }
            .to_owned()
        })
        .collect();
    if let Some(first) = slots.first_mut() {
        first.insert_str(0, digits);
    }
    slots
}

/// The index just past the last digit placeholder of `tokens`, if any.
fn digits_end(tokens: &[TextToken]) -> Option<usize> {
    tokens
        .iter()
        .rposition(|t| matches!(t, TextToken::Digit(_)))
        .map(|i| i + 1)
}

/// How the separator (the literals between the integer part and the numerator) prints, given
/// whether the fraction is blank and blanks to spaces (`blank`), whether the integer prints digits,
/// and the kinds of the last integer and the first numerator placeholders. A blank fraction takes
/// its separator with it. Where the integer prints no digit the separator prints under a `0`
/// integer, turns into spaces under a `?` integer or before a `?` numerator, and is dropped
/// otherwise (`# #/#` on 0.5 is "1/2", gen-m10 039accab K8; `#\:?=/=#` on 0.75 " 3=/=4",
/// `?\:#=/=#` "  3=/=4", `0\:#=/=#` "0:3=/=4", poi/NumberFormatTests A228, A234, A243).
fn separator_kind(
    blank: Option<bool>,
    integer_prints: bool,
    last_integer_kind: Option<char>,
    first_numerator_kind: Option<char>,
) -> Separator {
    match blank {
        Some(true) => Separator::Spaces,
        Some(false) => Separator::Dropped,
        None if integer_prints || last_integer_kind == Some('0') => Separator::Printed,
        None if last_integer_kind == Some('?') || first_numerator_kind == Some('?') => {
            Separator::Spaces
        }
        None => Separator::Dropped,
    }
}

/// How the literals between a fraction's integer part and its numerator print.
#[derive(PartialEq)]
enum Separator {
    Printed,
    Spaces,
    Dropped,
}

/// Pushes one space per column `token` would take.
fn push_spaces(text: &mut String, token: &TextToken) {
    let width = match token {
        TextToken::Text(t) => t.chars().count(),
        _ => 1,
    };
    text.extend(std::iter::repeat_n(' ', width));
}

/// The last continued-fraction convergent `(p, q)` of `x >= 0` whose denominator is at most
/// `max_denominator`, which is Excel's choice for a placeholder denominator (not the closest
/// fraction: `# ??/??` on `PI()` is "3  1/7 "; number-formats.md "Fraction formats").
///
/// Excel also stops where a partial quotient times the last numerator passes 2^31 - 1, although
/// the numerator itself (that product plus the one before) may pass it: `?/?` on 2147483646.6 is
/// "4294967293/2" (1 x 2147483647, then 2 x 4294967293 stops). When that stop comes before any
/// fractional term, the value is rounded to an integer: 1073741824.5 is "1073741825/1" (2 x
/// 1073741824 stops), 1073741823.5 "2147483647/2" (probes/fraction-placeholder-past-2-30;
/// number-formats.md "Placeholder fractions past 2^30").
fn last_convergent(x: f64, max_denominator: f64) -> (f64, f64) {
    // Cell values are finite; this only keeps a NaN out of the loop.
    if !x.is_finite() {
        return (0.0, 1.0);
    }
    let (mut p0, mut q0, mut p1, mut q1) = (0.0, 1.0, 1.0, 0.0);
    let mut rest = x;
    // A double's continued fraction ends within ~45 terms; the bound only guards the loop.
    for _ in 0..64 {
        let a = rest.floor();
        if a * p1 > MAX_PLACEHOLDER_VALUE {
            if q1 <= 1.0 {
                return (x.round(), 1.0);
            }
            break;
        }
        let (p, q) = (a * p1 + p0, a * q1 + q0);
        if q > max_denominator {
            break;
        }
        (p0, q0, p1, q1) = (p1, q1, p, q);
        let fract = rest - a;
        if fract < CONVERGENT_EPSILON {
            break;
        }
        rest = 1.0 / fract;
    }
    if q1 == 0.0 {
        (x.round(), 1.0)
    } else {
        (p1, q1)
    }
}

/// The factor `n` scaling commas right after a fraction's denominator apply:
/// 10^((-3n) mod 256), as if Excel kept the exponent -3n in an unsigned byte. `?/?,` on 1E-250 is
/// "1000/1" (10^253), `?/?,,` "1/1" (10^250), `?/?,,,` on 1E-238 "1000000000/1" (10^247); a
/// nonzero value that scales past the bounds is 255 `#` and `#VALUE!` from TEXT (`?/?,` on 0.5;
/// probes/fraction-overflow-scaled, number-formats.md "Scaling commas in fraction formats").
fn fraction_scale(commas: i32) -> f64 {
    10.0_f64.powi(comma_exponent(commas, false))
}

/// The exponent of ten that `commas` scaling commas apply: -3n, which Excel keeps in a byte,
/// unsigned in a fraction section ([`fraction_scale`]) and signed elsewhere (`signed`), where 252
/// commas on 5 print "5000000000000" (-756 wraps to 12; probes/t2842-dollar-run-contexts K1:K3,
/// number-formats.md "Many scaling commas wrap").
pub(super) fn comma_exponent(commas: i32, signed: bool) -> i32 {
    let exponent = commas.saturating_mul(-3).rem_euclid(256);
    if signed && exponent > 127 {
        exponent - 256
    } else {
        exponent
    }
}

/// The largest fixed denominator Excel uses as written (2^15); past it Excel picks the
/// denominator as for placeholders, up to [`LARGE_FIXED_MAX_DENOMINATOR`] (`?/32768` on 0.5 is
/// "16384/32768", `?/32769` "1/3    "; probes/fraction-large-fixed-denominator).
const MAX_FIXED_DENOMINATOR: f64 = 32768.0;

/// The largest denominator Excel picks for a fixed denominator it does not use as written
/// (10^7 - 1), whatever its digit count: `?/40000` on 1/9999999 is "1/40000" and on 1E-7
/// "0/4    ", and `?/32769` alike (probes/fraction-large-fixed-denominator); `# ?/4|?` on `PI()` is
/// "3 244252/4|7", 3 + 244252/1725033 (probes/t4043-fixed-denominator-slots).
const LARGE_FIXED_MAX_DENOMINATOR: f64 = 9_999_999.0;

/// The largest numerator a fixed denominator shows with no integer part (2^15 - 1).
const MAX_FIXED_NUMERATOR: f64 = 32767.0;

/// The magnitude from which a placeholder denominator with no integer part cannot show a value
/// (2^31 - 1).
const MAX_PLACEHOLDER_VALUE: f64 = 2_147_483_647.0;

/// A remainder below this ends the continued fraction.
const CONVERGENT_EPSILON: f64 = 1e-12;

/// `x` rounded half away from zero to an integer at 15 significant digits, as ROUND does.
fn round_numerator(x: f64) -> f64 {
    round_decimal(x, 0.0, RoundMode::HalfAwayFromZero).unwrap_or(x)
}

/// The kinds (`#`, `?` or `0`) of the digit placeholders among `tokens`, in order.
fn placeholder_kinds(tokens: &[TextToken]) -> Vec<char> {
    tokens
        .iter()
        .filter_map(|t| match t {
            TextToken::Digit(d) => Some(d.kind),
            _ => None,
        })
        .collect()
}

/// What each placeholder of `kinds` prints for `digits`, right-aligned: the first one takes any
/// digits beyond the placeholder count, and one with no digit prints nothing (`#`), a space (`?`)
/// or a zero (`0`).
fn fill(digits: &str, kinds: &[char]) -> Vec<String> {
    let digits: Vec<char> = digits.chars().collect();
    let pad = kinds.len().saturating_sub(digits.len());
    let extra = digits.len().saturating_sub(kinds.len());
    kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            if i < pad {
                match kind {
                    '?' => " ".to_owned(),
                    '0' => "0".to_owned(),
                    _ => String::new(),
                }
            } else if i == 0 {
                digits[..=extra].iter().collect()
            } else {
                digits[i - pad + extra].to_string()
            }
        })
        .collect()
}

/// Like [`fill`] for a denominator: each placeholder past `digits` prints nothing (`#`), a space
/// after the digits (`?`) or a zero before them (`0`): `# ???/???` on 1.5 is "1   1/2  ", `?/00`
/// on 1.5 "3/02" (number-formats.md "Fraction formats", probes/fraction-format-features B13) and
/// `#\:? ?0#/000` on 23.75 "2:3  03/004" (poi/NumberFormatTests A160).
fn fill_left(digits: &str, kinds: &[char]) -> String {
    let mut zeros = String::new();
    let mut spaces = String::new();
    for kind in kinds.iter().skip(digits.chars().count()) {
        match kind {
            '?' => spaces.push(' '),
            '0' => zeros.push('0'),
            _ => {}
        }
    }
    format!("{zeros}{digits}{spaces}")
}

#[cfg(test)]
mod comma_tests {
    use super::comma_exponent;

    #[test]
    fn comma_exponents_wrap_in_a_byte() {
        assert_eq!(comma_exponent(1, true), -3);
        assert_eq!(comma_exponent(42, true), -126);
        assert_eq!(comma_exponent(252, true), 12);
        assert_eq!(comma_exponent(254, true), 6);
        assert_eq!(comma_exponent(1, false), 253);
    }
}

#[cfg(test)]
mod trailing_tests {
    use super::{trailing_fill, Choice, Denominator, DenominatorPlan, Spill};

    /// One row per probed shape: the placeholders after the denominator, the picked
    /// denominator's spill (digits past its slots, excess), and what they print
    /// (probes/t4043-fixed-denominator-slots, -cut, t4043-placeholder-denominator-slots).
    #[test]
    fn trailing_placeholders() {
        let spill = |digits: &str, excess| {
            Some(Spill {
                digits: digits.to_owned(),
                excess,
            })
        };
        let rows: [(&str, Option<Spill>, &str); 13] = [
            // No `?` slots: `0`/`#` print a zero, `?` a space (`# 0/16|?` on 0.01).
            ("0", None, "0"),
            ("?", None, " "),
            // All `?`: spilled digits from the left (`# ?/4|?` on 0.0625, on PI()).
            ("?", spill("6", 1), "6"),
            ("??", spill("725033", 6), "72"),
            // ... or a zero per missing digit (`# ?/16|?` on 0.5, `# ?/100|??` on 0.5 and 0.3).
            ("?", spill("", -1), "0"),
            ("??", spill("", -2), "00"),
            ("??", spill("", -1), "0 "),
            ("??", spill("", 0), "  "),
            // A `0` or `#` among them: the spill first, then a zero each (`# ?/4|0` on 0.0625).
            ("0", spill("6", 1), "60"),
            ("?0", spill("6", 1), "600"),
            // Exact fit: `?` is a space until a zero printed (`# ?/?|?0`, `|0?` on 0.5).
            ("?0", spill("", 0), " 0"),
            ("0?", spill("", 0), "00"),
            // Missing digits: `#` prints nothing (`# ?/16|#` on 0.5).
            ("#", spill("", -1), ""),
        ];
        for (kinds, spill, expected) in rows {
            let kinds: Vec<char> = kinds.chars().collect();
            assert_eq!(
                trailing_fill(&kinds, spill.as_ref()).concat(),
                expected,
                "{kinds:?} {spill:?}"
            );
        }
    }

    #[test]
    fn denominator_plans() {
        let fixed = |digits: &str| Denominator::Fixed(digits.to_owned());
        let picked = Choice::Picked(9_999_999.0);
        // `# ?/4|0` on 0.0625: "4" shows, "6" spills to the trailing placeholders.
        let plan = DenominatorPlan::new(&fixed("4"), &picked, "16", Some('?'));
        assert_eq!(
            (plan.text.as_str(), plan.spill.map(|s| s.digits)),
            ("4", Some("6".into()))
        );
        // `# 0/4|0` on 0.0625: the extra digit prints in the denominator itself.
        let plan = DenominatorPlan::new(&fixed("4"), &picked, "16", Some('0'));
        assert_eq!((plan.text.as_str(), plan.spill), ("46", None));
        // `?/40000` on 0.5: cut to the picked length and padded under a `?` numerator.
        let plan = DenominatorPlan::new(&fixed("40000"), &picked, "2", Some('?'));
        assert_eq!(plan.text, "4    ");
        // As written: printed as is.
        let plan = DenominatorPlan::new(&fixed("16"), &Choice::AsWritten(16.0), "16", Some('?'));
        assert_eq!((plan.text.as_str(), plan.spill), ("16", None));
    }
}
