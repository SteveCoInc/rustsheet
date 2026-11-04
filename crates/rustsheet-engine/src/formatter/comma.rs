//! What a `,` in a number section does, as a function of the field it stands in and its
//! neighbours (docs/excel-semantics/number-formats.md "A comma between digit placeholders turns
//! on thousands grouping" and "Scaling commas in fraction formats").

/// The part of a number section a `,` stands in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Field {
    /// Before any decimal point, exponent or fraction.
    Integer,
    /// After the decimal point.
    Decimal,
    /// After a fraction's denominator (and after any scaling commas right behind it).
    AfterFraction,
    /// Directly after a fraction's denominator, or in the run of commas there.
    AfterDenominator,
    /// After the `E` of a scientific code with no fraction.
    Exponent,
}

/// What stands before the `,`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Prev {
    /// A digit placeholder (the token right before).
    Digit,
    /// The decimal point.
    Point,
    /// A `%` (the token right before).
    Percent,
    /// A literal, quoted text, a `_x` space or a `*x` fill.
    Literal,
    /// Nothing, or anything else (a date token, `@`, `General`).
    Other,
}

/// What follows the `,`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Next {
    /// A digit placeholder right after it.
    Digit,
    /// More commas, then a digit placeholder.
    CommasThenDigit,
    /// Anything else.
    Other,
}

/// What a `,` does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum CommaRole {
    /// Turns on thousands grouping of the integer part.
    Group,
    /// Scales the value (by 1/1000 per comma, by `fraction::comma_exponent` after a fraction).
    Scale,
    /// Prints nothing and does nothing, with the run of commas it starts.
    Skip,
    /// Prints as a `,`.
    Literal,
}

/// The role of a `,` in `field` between `prev` and `next`, once the section has `seen_digit` (a
/// digit placeholder before the decimal point):
/// - **integer part**: between two placeholders it groups (`#,##0`), after one and before
///   anything else it scales (`0,`, `#,,`), before any placeholder it prints (`,0`);
/// - **decimal part**: after a literal it prints, also last (`0.00|,0` on 1234567.89 is
///   "1234567.89|,0", `0.0|,` "1234567.9|,"); a run after a placeholder or the point vanishes
///   before a placeholder (`0.,0` is "1234567.9", `0.0,,0` "1234567.89") and scales otherwise
///   (`0.00|0,` is "1234.56|8", `0.,` "1235."; probes/t3598-decimal-part-commas,
///   t3666-review-followups A7:A12);
/// - **after a fraction**: a run between placeholders vanishes (`# ?/?|#,##0` on 1234.25 is
///   "1234 1/4|0000"), after anything but a placeholder it prints (`?/? ,` on 1E-250 "0/1 ,");
/// - **directly after the denominator** it scales (`?/?,` on 1E-250 is "1000/1");
/// - **after a `%`** it prints (`0%,` on 0.5 is "50%,", `?/?%,` "0/1%,";
///   probes/fraction-overflow-scaled);
/// - **in the exponent** the integer part's reading holds (unprobed; the scaling is dropped), and
///   after a fraction an `E` changes nothing (unprobed).
pub(super) fn comma_role(field: Field, prev: Prev, next: Next, seen_digit: bool) -> CommaRole {
    let digit_follows = next != Next::Other;
    match (field, prev) {
        (Field::AfterDenominator, _) => CommaRole::Scale,
        (_, Prev::Percent)
        | (Field::AfterFraction, Prev::Point | Prev::Literal | Prev::Other)
        | (Field::Decimal, Prev::Literal) => CommaRole::Literal,
        (Field::AfterFraction, Prev::Digit) | (Field::Decimal, Prev::Digit | Prev::Point)
            if digit_follows =>
        {
            CommaRole::Skip
        }
        (Field::Integer | Field::Exponent, Prev::Digit) if next == Next::Digit => CommaRole::Group,
        _ if seen_digit => CommaRole::Scale,
        _ => CommaRole::Literal,
    }
}

#[cfg(test)]
mod tests {
    use super::{comma_role, CommaRole, Field, Next, Prev};

    #[test]
    fn comma_roles_by_field() {
        use CommaRole::{Group, Literal, Scale, Skip};
        use Field::{AfterDenominator, AfterFraction, Decimal, Exponent, Integer};
        use Next::{CommasThenDigit, Digit};
        let fields = [Integer, Decimal, AfterFraction, AfterDenominator, Exponent];
        let prevs = [
            Prev::Digit,
            Prev::Point,
            Prev::Percent,
            Prev::Literal,
            Prev::Other,
        ];
        let nexts = [Digit, CommasThenDigit, Next::Other];
        // (field, prev, next, seen_digit) -> role, for every combination.
        let expected = |field, prev, next, seen: bool| -> CommaRole {
            let scale_or_literal = if seen { Scale } else { Literal };
            match (field, prev, next) {
                // `?/?,`, `# ?/4,,`
                (AfterDenominator, _, _) => Scale,
                // `0%,`, `?/?%,`
                (_, Prev::Percent, _) => Literal,
                // `# ?/?|#,##0`, `# ?/?|0,,0`
                (AfterFraction, Prev::Digit, Digit | CommasThenDigit) => Skip,
                // `# ?/?|0,`
                (AfterFraction, Prev::Digit, Next::Other) => scale_or_literal,
                // `?/? ,`, `# ?/?|.,0`
                (AfterFraction, _, _) => Literal,
                // `0.00|,0`, `0.0|,`
                (Decimal, Prev::Literal, _) => Literal,
                // `0.0,0`, `0.0,,0`, `0.,0`
                (Decimal, Prev::Digit | Prev::Point, Digit | CommasThenDigit) => Skip,
                // `0.00|0,`, `0.,`
                (Decimal, _, _) => scale_or_literal,
                // `#,##0`, `0.0E+0,0`
                (Integer | Exponent, Prev::Digit, Digit) => Group,
                // `0,`, `#,,##0` (the first of two scales), `,0` (literal)
                (Integer | Exponent, _, _) => scale_or_literal,
            }
        };
        for field in fields {
            for prev in prevs {
                for next in nexts {
                    for seen in [false, true] {
                        assert_eq!(
                            comma_role(field, prev, next, seen),
                            expected(field, prev, next, seen),
                            "{field:?} {prev:?} {next:?} seen_digit={seen}"
                        );
                    }
                }
            }
        }
    }
}
