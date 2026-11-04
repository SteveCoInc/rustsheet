//! The accent pass of the collation (the module docs' pass 2): one diacritic weight per letter
//! weight of the first pass, in a wrapping byte, and Excel's pairwise variant that skips the
//! units two letters share first.

use std::cmp::Ordering;
use std::iter::Peekable;

use super::{Element, NO_ACCENT};

/// One unit of the accent pass: a letter's own diacritic weight (`letter`), or the weight of a
/// combining mark after it.
#[derive(Clone, Copy)]
struct AccentUnit {
    weight: u8,
    letter: bool,
}

/// The accent units of `text`, letter by letter: one letter unit per letter weight of the first
/// pass (an expansion such as `æ` is two, its accent then an unaccented one), each followed by
/// the units of the combining marks after it. Marks with no letter before them follow an
/// unaccented letter unit of their own, as do marks right after a weightless character.
fn accent_units(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = AccentUnit> {
    let letter = |weight| AccentUnit {
        weight,
        letter: true,
    };
    let mut started = false;
    // A mark right after a hyphen or apostrophe belongs to that skipped character and adds
    // nothing here (it is weighed by [`skipped`]; probes/t3304-collation-multi-letter-accents G4).
    let mut after_skipped = false;
    elements
        .flat_map(move |element| match element {
            Element::Skipped(_) => {
                after_skipped = true;
                [None, None]
            }
            // Excel: a weightless character ends the letter before it, so a mark after it is a
            // lone mark (probes/t4060-collation-open-choices Calc!B50:B63, T-4072).
            Element::Weightless => {
                started = false;
                after_skipped = false;
                [None, None]
            }
            Element::Mark(_) if after_skipped => [None, None],
            Element::Mark(weight) => {
                let mark = Some(AccentUnit {
                    weight,
                    letter: false,
                });
                if std::mem::replace(&mut started, true) {
                    [mark, None]
                } else {
                    [Some(letter(NO_ACCENT)), mark]
                }
            }
            Element::Weights {
                primary, accent, ..
            } => {
                started = true;
                after_skipped = false;
                let second = (primary[1] != 0).then(|| letter(NO_ACCENT));
                [Some(letter(accent)), second]
            }
        })
        .flatten()
}

/// The next unit of the letter whose units `units` is reading, if any is left.
fn next_mark(units: &mut Peekable<impl Iterator<Item = AccentUnit>>) -> Option<u8> {
    units.next_if(|unit| !unit.letter).map(|unit| unit.weight)
}

/// `first` plus the units left of the letter `units` is reading, in a wrapping byte.
fn letter_sum(first: u8, units: &mut Peekable<impl Iterator<Item = AccentUnit>>) -> u8 {
    std::iter::from_fn(|| next_mark(units)).fold(first, u8::wrapping_add)
}

/// The second pass: one accent weight per letter weight of the first pass, the sum of its
/// [`accent_units`] in one wrapping byte, like the Windows sort key's one-byte diacritic weight.
/// So the order of the marks does not matter:
/// `"a"&UNICHAR(769)&UNICHAR(780)` = `"a"&UNICHAR(780)&UNICHAR(769)` = `"á"&UNICHAR(780)` =
/// `"ǎ"&UNICHAR(769)` (2 + 12 + 18), and three U+0323 (2 + 3 * 86 = 260, so 4) sort between `"a"`
/// and `"á"` (probes/t3154-collation-accent-weights, probes/t3154-collation-accent-pairs).
pub(super) fn accents(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    let mut units = accent_units(elements).peekable();
    std::iter::from_fn(move || {
        let first = units.next()?.weight;
        Some(letter_sum(first, &mut units))
    })
}

/// Excel's accent pass between two texts (coercion.md "Shared leading units are skipped before
/// the sum", probes/t3201-collation-long-mark-runs, probes/t3201-collation-wrapped-run-pairs):
/// letter by letter, drop the leading units both letters share with equal weights; a letter
/// with no units left is less (so a text is below any text it is a proper prefix of,
/// probes/t3154-collation-accent-pairs Calc!S19, T18); otherwise the sums of the units left, each
/// in a wrapping byte, decide. Texts equal here have equal [`accents`].
pub(super) fn cmp_accent_units(
    left: impl Iterator<Item = Element>,
    right: impl Iterator<Item = Element>,
) -> Ordering {
    let (mut left, mut right) = (
        accent_units(left).peekable(),
        accent_units(right).peekable(),
    );
    loop {
        let (mut l, mut r) = match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(l), Some(r)) => (Some(l.weight), Some(r.weight)),
        };
        while l.is_some() && l == r {
            (l, r) = (next_mark(&mut left), next_mark(&mut right));
        }
        let order = match (l, r) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(l), Some(r)) => letter_sum(l, &mut left).cmp(&letter_sum(r, &mut right)),
        };
        if order.is_ne() {
            return order;
        }
    }
}
