//! How close a function's scored cells come to Excel: the buckets of the parity page
//! (`tools/src/rustsheet_tools/progress_report/parity.py`, T-2041), counted by the report (T-2036).
//!
//! The buckets follow decision 0083's order of parity: a cell is *visibly* right when the 15
//! significant digits Excel shows are right (`digits15` and better), and visibly off otherwise
//! (`visible_numeric`, `visible_other`; T-0061). The ulp buckets take precedence: a number within
//! 4 ulp is never visibly off, even where 4 ulp exceed half a unit in the 15th digit (decision
//! 0088).
//!
//! Decision 0267 moves the bar to 13 digits for the incomplete-beta family
//! ([`THIRTEEN_DIGIT_FAMILY`]): a number those functions alone compute that is past half a unit in
//! the 15th digit but within half a unit in the 13th is `residue13`, Phase 1b residue that every
//! report counts and shows but none counts as visibly off. There Excel itself is rarely correctly
//! rounded (about 1% of 2,026 sampled cells), so those digits measure Excel's rounding noise.
//!
//! Decision 0188 R3 has the engine follow the 50-digit math where Excel is more than G = 1e-4 off
//! it: a number the run marks [`Target::Math`] (R1(c)'s [`crate::r1c::R1c::R3Pass`] test,
//! [`crate::r1c::r3_math`]) is `r3_math`, math over Excel by design, counted and shown every run
//! but not visibly off (decision 0310, T-4520).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::formula_calls::functions_in;
use crate::runner::{CellResult, Target};
use crate::value::{ulp_distance, Value, Verdict};

/// Scored cells per closeness bucket. `unsupported` cells are not scored and count in none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closeness {
    /// Bit-exact, or an accepted deviation's pinned value (decision 0036).
    pub pass: usize,
    /// Numbers 1 ulp apart.
    pub ulp1: usize,
    /// Numbers 2 to 4 ulp apart.
    pub ulp4: usize,
    /// Numbers more than 4 ulp apart but within half a unit in the 15th significant digit of
    /// the expected value ([`within_fifteen_digits`]). They may still display differently when
    /// they straddle a rounding boundary of that digit.
    pub digits15: usize,
    /// Numbers of the incomplete-beta family ([`THIRTEEN_DIGIT_FAMILY`]) further apart than half
    /// a unit in the 15th significant digit but within half a unit in the 13th: Phase 1b residue,
    /// not visibly off (decision 0267). `default` keeps older report.json files readable.
    #[serde(default)]
    pub residue13: usize,
    /// Numbers where the engine returns the 50-digit math instead of Excel's value by design
    /// (decision 0188 R3, [`Target::Math`]): not visibly off (decision 0310). `default`
    /// keeps older report.json files readable.
    #[serde(default)]
    pub r3_math: usize,
    /// Numbers further apart than half a unit in the 15th significant digit, including 0
    /// against nonzero and non-finite values, and neither `residue13` nor `r3_math`.
    pub visible_numeric: usize,
    /// A wrong type, error, text or boolean.
    pub visible_other: usize,
}

/// Which bucket a scored cell falls in; `None` for an `unsupported` cell.
fn bucket(cell: &CellResult) -> Option<Bucket> {
    Some(match cell.verdict {
        Verdict::Pass | Verdict::Deviation => Bucket::Pass,
        Verdict::Unsupported => return None,
        Verdict::NumericUlp => match (&cell.expected, &cell.actual) {
            (Value::Number(e), Value::Number(a)) if ulp_distance(*e, *a) <= 1 => Bucket::Ulp1,
            _ => Bucket::Ulp4,
        },
        Verdict::NumericClose | Verdict::NumericMismatch => match (&cell.expected, &cell.actual) {
            (Value::Number(e), Value::Number(a)) if within_fifteen_digits(*e, *a) => {
                Bucket::Digits15
            }
            // The run marks numbers only, where R1(c) passes an R3 cell (`r1c::mark_r3_math`).
            (Value::Number(_), Value::Number(_)) if cell.target == Target::Math => Bucket::R3Math,
            (Value::Number(e), Value::Number(a))
                if within_significant_digits(*e, *a, 13) && in_thirteen_digit_family(cell) =>
            {
                Bucket::Residue13
            }
            _ => Bucket::VisibleNumeric,
        },
        Verdict::TextMismatch
        | Verdict::BoolMismatch
        | Verdict::ErrorMismatch
        | Verdict::TypeMismatch => Bucket::VisibleOther,
    })
}

/// A scored cell's closeness bucket, serialized under [`Closeness`]'s field names: the names
/// report.json gives each listed cell ([`serialize_cells`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Bucket {
    Pass,
    Ulp1,
    Ulp4,
    Digits15,
    Residue13,
    R3Math,
    VisibleNumeric,
    VisibleOther,
}

impl Closeness {
    /// Counts `cell` in its bucket; an `unsupported` cell is skipped.
    pub fn add(&mut self, cell: &CellResult) {
        let Some(b) = bucket(cell) else { return };
        *match b {
            Bucket::Pass => &mut self.pass,
            Bucket::Ulp1 => &mut self.ulp1,
            Bucket::Ulp4 => &mut self.ulp4,
            Bucket::Digits15 => &mut self.digits15,
            Bucket::Residue13 => &mut self.residue13,
            Bucket::R3Math => &mut self.r3_math,
            Bucket::VisibleNumeric => &mut self.visible_numeric,
            Bucket::VisibleOther => &mut self.visible_other,
        } += 1;
    }

    /// The scored cells, every bucket summed.
    #[must_use]
    pub fn scored(&self) -> usize {
        self.pass
            + self.ulp1
            + self.ulp4
            + self.digits15
            + self.residue13
            + self.r3_math
            + self.visible()
    }

    /// The visibly-off cells: `visible_numeric` and `visible_other` (decision 0083). `residue13`
    /// (decision 0267) and `r3_math` (decision 0310) are not among them.
    #[must_use]
    pub fn visible(&self) -> usize {
        self.visible_numeric + self.visible_other
    }
}

/// One listed cell as report.json writes it: the cell, its closeness bucket (none for an
/// `unsupported` cell) and whether its formula is a [`THIRTEEN_DIGIT_FAMILY`] formula, so
/// `scripts/family_net.py` reads both rather than re-deriving them (decision 0267 item 6, T-4159).
#[derive(Serialize)]
struct ListedCell<'a> {
    #[serde(flatten)]
    cell: &'a CellResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    closeness: Option<Bucket>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    thirteen_digit_family: bool,
}

/// Serializes a case's cells with each one's bucket and family flag ([`ListedCell`]). Reading
/// them back ignores both, so a report.json from before them still loads.
///
/// # Errors
/// When the serializer fails.
pub fn serialize_cells<S: serde::Serializer>(
    cells: &[CellResult],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(cells.iter().map(|cell| ListedCell {
        cell,
        closeness: bucket(cell),
        thirteen_digit_family: in_thirteen_digit_family(cell),
    }))
}

/// Visibly-off cells blamed on one function ([`blamed`]), split as the buckets split them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visible {
    /// Cells in the `visible_numeric` bucket.
    pub numeric: usize,
    /// Cells in the `visible_other` bucket.
    pub other: usize,
}

impl Visible {
    /// `cell`'s contribution: one cell in its visible bucket, or nothing.
    #[must_use]
    pub fn of(cell: &CellResult) -> Visible {
        match bucket(cell) {
            Some(Bucket::VisibleNumeric) => Visible {
                numeric: 1,
                other: 0,
            },
            Some(Bucket::VisibleOther) => Visible {
                numeric: 0,
                other: 1,
            },
            _ => Visible::default(),
        }
    }

    /// Adds `other`'s counts to these.
    pub fn merge(&mut self, other: Visible) {
        self.numeric += other.numeric;
        self.other += other.other;
    }

    /// Both kinds summed.
    #[must_use]
    pub fn total(&self) -> usize {
        self.numeric + self.other
    }

    /// Whether `cell` is visibly off (decision 0083): the one test every report section uses,
    /// with the incomplete-beta family at 13 digits (decision 0267) and R3 math-over-Excel cells
    /// left out (decision 0310).
    #[must_use]
    pub fn is_off(cell: &CellResult) -> bool {
        Visible::of(cell).total() > 0
    }

    /// Whether `cell` is off at the 15-digit bar for every function, `residue13` and `r3_math`
    /// included: Gate A's envelope (decision 0188 R2), which decision 0267 leaves at 15 digits and
    /// which drops Excel's side of an R3 point its own way, so decision 0310 does not reach it.
    /// The same buckets as [`Visible::is_off`], so there is still one 15-digit rule.
    #[must_use]
    pub fn is_off_at_fifteen_digits(cell: &CellResult) -> bool {
        matches!(
            bucket(cell),
            Some(
                Bucket::Residue13 | Bucket::R3Math | Bucket::VisibleNumeric | Bucket::VisibleOther
            )
        )
    }

    /// Whether `cell` is in the `r3_math` bucket: a number the run marked
    /// [`Target::Math`], past half a unit in the 15th digit of Excel's value (decision
    /// 0310). R1(c) still classifies it, as [`crate::r1c::R1c::R3Pass`].
    #[must_use]
    pub fn is_r3_math(cell: &CellResult) -> bool {
        bucket(cell) == Some(Bucket::R3Math)
    }

    /// Whether `cell` is a known failure (neither bit-exact nor a pinned deviation) that is
    /// visibly off: the cells the probe rows list and decision 0188 R1(c) classifies.
    #[must_use]
    pub fn is_failing_off(cell: &CellResult) -> bool {
        cell.verdict.is_failure() && Self::is_off(cell)
    }
}

/// Whether `actual` is within half a unit in the 15th significant digit of `expected`: the
/// digits Excel displays are the same, or differ only by rounding at that digit (decision 0083).
/// [`within_significant_digits`] at 15.
#[must_use]
pub fn within_fifteen_digits(expected: f64, actual: f64) -> bool {
    within_significant_digits(expected, actual, 15)
}

/// Whether `actual` is within half a unit in the `digits`-th significant digit of `expected`.
///
/// The boundary itself counts as within. An expected 0 has no such digit, so only an exact 0
/// (either sign) is within. For a subnormal `expected` (and the smallest normals) half the unit
/// underflows to 0 or a few subnormal steps, so in practice only an equal value is within.
/// Non-finite values are never within. The half unit is the double nearest `5e(E-digits)` for the
/// decimal exponent `E` of `expected`, so a gap within half an ulp of it may land on either side;
/// the unit tests pin the exactly representable boundary.
#[must_use]
pub fn within_significant_digits(expected: f64, actual: f64, digits: i32) -> bool {
    if !expected.is_finite() || !actual.is_finite() {
        return false;
    }
    let gap = (actual - expected).abs();
    if expected == 0.0 {
        return gap == 0.0;
    }
    // `{:e}` prints the shortest round-trip digits, whose exponent is the true decimal exponent
    // of `expected` (a double never rounds up to the next power of ten in that form).
    let exponent = format!("{:e}", expected.abs())
        .split_once('e')
        .and_then(|(_, e)| e.parse::<i32>().ok());
    let Some(exponent) = exponent else {
        return false;
    };
    let half_unit = format!("5e{}", exponent - digits)
        .parse::<f64>()
        .unwrap_or(0.0);
    gap <= half_unit
}

/// The functions decision 0267 rules on, whose numbers count at 13 digits: their cdfs run on
/// TOMS 708 `bratio` (docs/excel-semantics/statistics.md, "Incomplete beta is TOMS 708
/// `bratio`"), and the ruling's evidence covers their cdfs, densities and pmfs alike: there
/// Excel's 14th and 15th digits are its rounding noise (statistics.md, "Incomplete-beta family:
/// Excel's 14th and 15th digits are noise").
pub const THIRTEEN_DIGIT_FAMILY: &[&str] = &[
    "BETA.DIST",
    "BETADIST",
    "BINOM.DIST",
    "BINOMDIST",
    "F.DIST",
    "F.DIST.RT",
    "FDIST",
    "NEGBINOM.DIST",
    "NEGBINOMDIST",
];

/// Whether every function `cell`'s formula is blamed on ([`blamed`]; its array anchor's formula
/// for a member cell) is in [`THIRTEEN_DIGIT_FAMILY`]. A cell with no formula, or one that also
/// calls a function outside the family (`BETA.INV(BETA.DIST(..))`), keeps the 15-digit bar.
fn in_thirteen_digit_family(cell: &CellResult) -> bool {
    let Some(formula) = cell.formula.as_deref().or(cell.anchor_formula.as_deref()) else {
        return false;
    };
    let called = functions_in(formula);
    let blamed = blamed(&called);
    !blamed.is_empty()
        && blamed
            .iter()
            .all(|f| THIRTEEN_DIGIT_FAMILY.contains(&f.as_str()))
}

/// Functions that only route or combine values (control flow, plain sums, reference picks,
/// dates built from parts). A visibly-off cell is blamed on the other functions it calls, so
/// `IF(x, GAMMA(y))` counts against GAMMA, not IF (decision 0083's attribution).
pub const GLUE: &[&str] = &[
    "AND", "CHOOSE", "COLUMN", "DATE", "FALSE", "IF", "IFERROR", "IFNA", "IFS", "INDEX", "LET",
    "NOT", "OR", "ROW", "SUM", "SWITCH", "TIME", "TRUE",
];

/// The functions of `called` that a visibly-off cell is blamed on: every function not in
/// [`GLUE`], or all of `called` when the formula calls only glue.
#[must_use]
pub fn blamed(called: &BTreeSet<String>) -> Vec<&String> {
    let specialised: Vec<&String> = called
        .iter()
        .filter(|f| !GLUE.contains(&f.as_str()))
        .collect();
    if specialised.is_empty() {
        called.iter().collect()
    } else {
        specialised
    }
}

/// The function a cell is attributed to under decision 0118: the one function of `called` not in
/// [`GLUE`], or, when the formula calls only glue, its one glue function. `None` when the formula
/// calls two or more distinct non-glue functions, or only glue and two or more distinct ones (or
/// nothing). `called` is the distinct-call set [`blamed`] takes, so repeated calls of one function
/// count once: `GAMMA(1)+GAMMA(2)` is GAMMA's. Unlike [`blamed`], `IF(AND(..),..)` is nobody's.
#[must_use]
pub fn sole_call(called: &BTreeSet<String>) -> Option<&String> {
    let mut specialised = called.iter().filter(|f| !GLUE.contains(&f.as_str()));
    match (specialised.next(), specialised.next()) {
        (Some(f), None) => Some(f),
        (None, _) if called.len() == 1 => called.first(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(expected: Value, actual: Value, verdict: Verdict) -> CellResult {
        CellResult {
            sheet: "S".into(),
            cell: "A1".into(),
            formula: None,
            anchor_formula: None,
            expected,
            actual,
            verdict,
            from_excel: true,
            listed_deviation: false,
            graph_subnormal: false,
            kind: crate::runner::CellKind::Scored,
            math: None,
            target: crate::runner::Target::Excel,
        }
    }

    #[test]
    fn cells_fall_in_their_bucket_and_unsupported_in_none() {
        let one = 1.0_f64;
        let up = |n: u64| f64::from_bits(one.to_bits() + n);
        let n = Value::Number;
        let mut c = Closeness::default();
        for x in [
            cell(n(one), n(one), Verdict::Pass),
            cell(n(one), n(9.0), Verdict::Deviation),
            cell(n(one), n(up(1)), Verdict::NumericUlp),
            cell(n(one), n(up(4)), Verdict::NumericUlp),
            cell(n(one), n(up(5)), Verdict::NumericClose),
            cell(n(one), n(one + 1e-12), Verdict::NumericClose),
            cell(n(one), n(2.0), Verdict::NumericMismatch),
            cell(Value::Text("a".into()), n(one), Verdict::TypeMismatch),
            cell(n(one), Value::Error("#NAME?".into()), Verdict::Unsupported),
        ] {
            c.add(&x);
        }
        let expected = Closeness {
            pass: 2,
            ulp1: 1,
            ulp4: 1,
            digits15: 1,
            residue13: 0,
            r3_math: 0,
            visible_numeric: 2,
            visible_other: 1,
        };
        assert_eq!(c, expected);
        assert_eq!(c.scored(), 8);
        assert_eq!(c.visible(), 3);
    }

    fn with_formula(formula: &str, expected: f64, actual: f64) -> CellResult {
        CellResult {
            formula: Some(formula.into()),
            ..cell(
                Value::Number(expected),
                Value::Number(actual),
                Verdict::NumericClose,
            )
        }
    }

    #[test]
    fn the_incomplete_beta_family_counts_at_thirteen_digits() {
        // 1e15 + 6 is past half a unit in the 15th digit (5) and within half a unit in the 13th (500).
        let (e, off15, off13) = (1e15, 1e15 + 6.0, 1e15 + 501.0);
        let residue = |c: &CellResult| bucket(c) == Some(Bucket::Residue13);
        for f in [
            "=BETA.DIST(A1,2,3,TRUE)",
            "=_xlfn.F.DIST.RT(2,3,4)",
            "=IF(A1,BINOM.DIST(1,2,0.5,FALSE))",
        ] {
            assert!(residue(&with_formula(f, e, off15)), "{f}");
            assert!(!Visible::is_off(&with_formula(f, e, off15)), "{f}");
            // Past the 13th digit it is visibly off again.
            assert!(Visible::is_off(&with_formula(f, e, off13)), "{f}");
        }
        // Outside the family, or mixed with a function outside it, the 15-digit bar holds.
        for f in [
            "=T.DIST(1,2,TRUE)",
            "=BETA.INV(BETA.DIST(0.5,2,3,TRUE),2,3)",
            "=A1*2",
        ] {
            assert!(Visible::is_off(&with_formula(f, e, off15)), "{f}");
        }
        // An array member is judged by its anchor's formula; a cell with no formula keeps 15.
        let member = CellResult {
            anchor_formula: Some("=FDIST(A1:A3,2,3)".into()),
            ..cell(
                Value::Number(e),
                Value::Number(off15),
                Verdict::NumericClose,
            )
        };
        assert!(residue(&member));
        assert!(Visible::is_off(&cell(
            Value::Number(e),
            Value::Number(off15),
            Verdict::NumericClose
        )));
        // A wrong type is never residue.
        let typed = CellResult {
            formula: Some("=BETA.DIST(1,2,3,TRUE)".into()),
            ..cell(
                Value::Number(1.0),
                Value::Error("#NUM!".into()),
                Verdict::TypeMismatch,
            )
        };
        assert!(Visible::is_off(&typed));
        // Gate A's envelope keeps the 15-digit bar for the family too.
        assert!(Visible::is_off_at_fifteen_digits(&with_formula(
            "=BETA.DIST(1,2,3,TRUE)",
            e,
            off15
        )));
        assert!(!Visible::is_off_at_fifteen_digits(&with_formula(
            "=BETA.DIST(1,2,3,TRUE)",
            e,
            e + 4.0
        )));
        assert!(Visible::is_off_at_fifteen_digits(&typed));
        let mut c = Closeness::default();
        c.add(&with_formula("=BETA.DIST(1,2,3,TRUE)", e, off15));
        assert_eq!((c.residue13, c.visible(), c.scored()), (1, 0, 1));
    }

    #[test]
    fn a_number_the_run_marks_r3_math_is_its_own_bucket() {
        // Decision 0310: the mark (set by `r1c::mark_r3_math`) moves a number past the 15-digit
        // bar out of `visible_numeric`; within 15 digits it stays `digits15`.
        let marked = |c: CellResult| CellResult {
            target: Target::Math,
            ..c
        };
        let off = marked(cell(
            Value::Number(1.0),
            Value::Number(1.001),
            Verdict::NumericMismatch,
        ));
        assert_eq!(bucket(&off), Some(Bucket::R3Math));
        assert!(!Visible::is_off(&off));
        assert_eq!(Visible::of(&off), Visible::default());
        assert!(Visible::is_off_at_fifteen_digits(&off));
        let close = marked(cell(
            Value::Number(1.0),
            Value::Number(1.0 + 1e-15),
            Verdict::NumericClose,
        ));
        assert_eq!(bucket(&close), Some(Bucket::Digits15));
        // A wrong type keeps its bucket whatever the mark says.
        let typed = marked(cell(
            Value::Number(1.0),
            Value::Error("#NUM!".into()),
            Verdict::TypeMismatch,
        ));
        assert_eq!(bucket(&typed), Some(Bucket::VisibleOther));
        let mut c = Closeness::default();
        c.add(&off);
        assert_eq!((c.r3_math, c.visible(), c.scored()), (1, 0, 1));
        // A report.json from before the bucket reads it as 0.
        let old: Closeness = serde_json::from_str(
            r#"{"pass":1,"ulp1":0,"ulp4":0,"digits15":0,"visible_numeric":2,"visible_other":0}"#,
        )
        .unwrap();
        assert_eq!((old.r3_math, old.residue13, old.visible()), (0, 0, 2));
    }

    #[test]
    fn listed_cells_carry_their_bucket_and_family_flag() {
        let (e, off15) = (1e15, 1e15 + 6.0);
        let unsupported = cell(
            Value::Number(e),
            Value::Error("#NAME?".into()),
            Verdict::Unsupported,
        );
        let cells = vec![
            with_formula("=BETA.DIST(A1,2,3,TRUE)", e, off15),
            with_formula("=T.DIST(1,2,TRUE)", e, off15),
            unsupported,
        ];
        let mut json = serde_json::Serializer::new(Vec::new());
        serialize_cells(&cells, &mut json).unwrap();
        let out: serde_json::Value = serde_json::from_slice(&json.into_inner()).unwrap();
        let field = |i: usize, k: &str| out[i].get(k).cloned();
        assert_eq!(field(0, "closeness"), Some("residue13".into()));
        assert_eq!(field(0, "thirteen_digit_family"), Some(true.into()));
        assert_eq!(field(1, "closeness"), Some("visible_numeric".into()));
        assert_eq!(field(1, "thirteen_digit_family"), None);
        // An unsupported cell has no bucket; the cell's own fields are all still there.
        assert_eq!(field(2, "closeness"), None);
        assert_eq!(field(2, "verdict"), Some("unsupported".into()));
        // They read back as plain cells.
        let back: Vec<CellResult> = serde_json::from_value(out).unwrap();
        assert_eq!(back.len(), 3);
        assert_eq!(back[0].formula, cells[0].formula);
    }

    #[test]
    fn half_a_unit_in_the_thirteenth_digit_is_the_boundary() {
        assert!(within_significant_digits(1e15, 1e15 + 500.0, 13));
        assert!(!within_significant_digits(1e15, 1e15 + 502.0, 13));
        assert!(within_significant_digits(1.0, 1.0 + 4e-13, 13));
        assert!(!within_significant_digits(1.0, 1.0 + 6e-13, 13));
        assert!(!within_significant_digits(0.0, 1e-300, 13));
    }

    #[test]
    fn half_a_unit_in_the_fifteenth_digit_is_the_boundary() {
        // 1e15 has 16 digits, so its 15th-digit unit is 10 and half of it, 5, is exact.
        assert!(within_fifteen_digits(1e15, 1e15 + 5.0));
        assert!(within_fifteen_digits(1e15, 1e15 - 5.0));
        assert!(!within_fifteen_digits(1e15, 1e15 + 6.0));
        assert!(!within_fifteen_digits(-1e15, -1e15 - 6.0));
        // 9.99e15 still has exponent 15; 1e16 moves the unit to 100.
        assert!(within_fifteen_digits(9.99e15, 9.99e15 + 4.0));
        assert!(within_fifteen_digits(1e16, 1e16 + 50.0));
        assert!(!within_fifteen_digits(1e16, 1e16 + 52.0));
        // Fractions: 1 shows 14 decimals, so 5e-15 is half a unit and 2e-14 is not.
        assert!(within_fifteen_digits(1.0, 1.0 + 4e-15));
        assert!(!within_fifteen_digits(1.0, 1.0 + 2e-14));
        assert!(within_fifteen_digits(0.001_234, 0.001_234 + 4e-18));
        assert!(!within_fifteen_digits(0.001_234, 0.001_234 + 2e-17));
    }

    #[test]
    fn shared_vectors_agree_with_the_fit_lab_port() {
        // The same table is read by tools/tests/test_fitlab_holdout.py (T-3453).
        let table = include_str!("../tests/fixtures/within_fifteen_digits.tsv");
        let mut rows = 0;
        for line in table
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
        {
            let f: Vec<&str> = line.split('\t').collect();
            let (e, a): (f64, f64) = (f[0].parse().unwrap(), f[1].parse().unwrap());
            assert_eq!(within_fifteen_digits(e, a), f[2] == "1", "{line}");
            rows += 1;
        }
        assert_eq!(rows, 16);
    }

    #[test]
    fn zero_subnormal_and_non_finite_expectations() {
        assert!(within_fifteen_digits(0.0, -0.0));
        assert!(!within_fifteen_digits(0.0, 1e-300));
        assert!(!within_fifteen_digits(1e-300, 0.0));
        let tiny = f64::from_bits(1000);
        assert!(within_fifteen_digits(tiny, tiny));
        assert!(!within_fifteen_digits(tiny, f64::from_bits(1010)));
        // The smallest normal still has a representable half unit.
        let normal = f64::MIN_POSITIVE;
        assert!(within_fifteen_digits(normal, normal * (1.0 + 1e-15)));
        assert!(!within_fifteen_digits(f64::NAN, f64::NAN));
        assert!(!within_fifteen_digits(1.0, f64::INFINITY));
    }

    #[test]
    fn visibly_off_cells_blame_the_specialised_functions() {
        let set = |names: &[&str]| -> BTreeSet<String> {
            names.iter().map(|s| (*s).to_string()).collect()
        };
        let gamma_in_if = set(&["IF", "GAMMA", "SUM"]);
        assert_eq!(blamed(&gamma_in_if), vec!["GAMMA"]);
        let glue_only = set(&["DATE", "IF"]);
        assert_eq!(blamed(&glue_only), vec!["DATE", "IF"]);
        assert!(blamed(&BTreeSet::new()).is_empty());
        let n = Value::Number;
        let mut v = Visible::default();
        for x in [
            cell(n(1.0), n(2.0), Verdict::NumericMismatch),
            cell(n(1.0), n(1.0 + 4e-15), Verdict::NumericClose),
            cell(Value::Bool(true), Value::Bool(false), Verdict::BoolMismatch),
            cell(n(1.0), n(1.0), Verdict::Pass),
        ] {
            v.merge(Visible::of(&x));
        }
        assert_eq!(
            v,
            Visible {
                numeric: 1,
                other: 1
            }
        );
        assert_eq!(v.total(), 2);
    }

    #[test]
    fn a_cell_is_attributed_to_its_sole_non_glue_call() {
        use crate::formula_calls::functions_in;
        let sole = |formula: &str| sole_call(&functions_in(formula)).cloned();
        let name = |f: &str| Some(f.to_string());
        assert_eq!(sole("IF(A1,GAMMA(B1))"), name("GAMMA"));
        assert_eq!(sole("GAMMA(1)+GAMMA(2)"), name("GAMMA"));
        assert_eq!(sole("SUM(A1:A3)"), name("SUM"));
        assert_eq!(sole("IF(A1,1,2)"), name("IF"));
        assert_eq!(sole("GAMMA(BETA.DIST(0.5,2,3,TRUE))"), None);
        assert_eq!(sole("A1+1"), None);
        // Decision 0118: a glue-only formula with two distinct calls is nobody's, though
        // decision 0088's `blamed` still blames both.
        let if_and = functions_in("IF(AND(A1,B1),1,2)");
        assert_eq!(sole_call(&if_and), None);
        assert_eq!(blamed(&if_and), vec!["AND", "IF"]);
    }
}
