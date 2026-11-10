//! Decision 0188 R1(c), as decision 0355 item 4 amends it: which visibly-off cells a milestone's
//! probe row and ironcalc-calc row count, and which are Phase 1b evidence instead (T-3427,
//! T-5092).
//!
//! A row counts a visibly-off cell (decision 0083, [`Visible::is_off`]) unless:
//! - it is a probe-row cell (suite `probes`) that an open M14 task alone owns (R7, decision 0143's
//!   generated list, [`Phase1Exceptions::desktop_owns`]): it leaves the row ([`R1c::M14Owned`]).
//!   R7 names the probe rows only, so an M14-owned ironcalc-calc cell still counts;
//! - `math-over-excel.tsv` marks it, Excel is more than G = 1e-4 off its math value (R3's evidence,
//!   checked per cell) and the engine returns the math within ε: it passes ([`R1c::R3Pass`]);
//! - it is inside decision 0355's envelope ([`cell_envelope`]: within 12 significant digits of
//!   Excel, or within ε of its 50-digit math value): it is Phase 1b evidence ([`R1c::Phase1b`]),
//!   measured and reported every run but not counted.
//!
//! Everything else counts ([`R1c::Counts`]): a wrong type, error, text or boolean
//! ([`CellEnvelope::WrongType`], at any number of digits), and a number outside the envelope
//! ([`CellEnvelope::Outside`]). [`R1cCells::miss`] says which of the two a counted cell is, for
//! the rows' tier bars (decision 0355 item 4).
//!
//! Before decision 0355 a number within ε of the math also counted when Excel's own value was
//! more than ε off the math, since the gap might be a rule Excel applies. Decision 0355 item 2
//! makes a value within ε of the math good whatever Excel says, so that clause is gone
//! (SPEC-0037 design 2).
//!
//! The R3 test ([`r3_math`]) is also what the closeness buckets read: the run marks each such cell
//! once ([`mark_r3_math`], [`Target::Math`]), so it is `r3_math` rather than visibly off in
//! every count (decision 0310). [`classify`] still classifies it, as [`R1c::R3Pass`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::accuracy::envelope::{cell_envelope, envelope, excel_off_math, CellEnvelope};
use crate::accuracy::lists::R3Marks;
use crate::closeness::Visible;
use crate::phase1_exceptions::Phase1Exceptions;
use crate::probe_milestones;
use crate::runner::{cell_id, CaseResult, CellResult, Target};
use crate::value::Value;

/// How R1(c) reads one visibly-off cell (see the module doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum R1c {
    /// The row counts it.
    Counts,
    /// Inside decision 0355's envelope ([`cell_envelope`]) but not bit-exact: Phase 1b evidence.
    Phase1b,
    /// An R3 cell where the engine returns the math: passing.
    R3Pass,
    /// A probe-row cell only open M14 tasks own (R7): it leaves the row.
    M14Owned,
}

/// Whether cell `c` of case `case` is a decision 0188 R3 cell where the engine returns the math:
/// two numbers, `math-over-excel.tsv` marks the cell, Excel is more than G = 1e-4 off its math
/// value (R3's evidence, checked per cell: below G the mark is ignored and Excel's value stays
/// the target) and the engine's value is inside Gate A's envelope with Excel's side dropped,
/// within ε of the math. The one test behind [`R1c::R3Pass`] and [`Target::Math`]. R3
/// never applies to an error, a text or a wrong type.
#[must_use]
pub fn r3_math(case: &str, c: &CellResult, r3: &R3Marks) -> bool {
    r3_marked(case, c, r3)
        && c.math
            .as_ref()
            .is_some_and(|math| envelope(Some(c), &c.actual, Some(math), true).is_inside())
}

/// Whether cell `c` of case `case` is one decision 0188 R3 holds to the math, passing or not: two
/// numbers with a math value, `math-over-excel.tsv` marks the cell, and Excel is more than G =
/// 1e-4 off the math (R3's evidence, checked per cell). [`r3_math`] adds that the engine is
/// within ε of the math.
#[must_use]
pub fn r3_marked(case: &str, c: &CellResult, r3: &R3Marks) -> bool {
    let (Value::Number(_), Value::Number(_), Some(math)) = (&c.expected, &c.actual, &c.math) else {
        return false;
    };
    r3.contains_key(&cell_id(case, &c.key())) && excel_off_math(&c.expected, math)
}

/// Sets [`CellResult::target`] on every cell of `results`: [`Target::Math`] by [`r3_math`],
/// [`Target::MathMissed`] for the other cells [`r3_marked`] holds to the math (T-5097), else
/// [`Target::Excel`]. Once per run, before the report counts the closeness buckets (decision 0310)
/// and before anything calls `cell_envelope`.
pub fn mark_r3_math(results: &mut [CaseResult], r3: &R3Marks) {
    for r in results {
        for c in &mut r.cells {
            c.target = if r3_math(&r.id, c, r3) {
                Target::Math
            } else if r3_marked(&r.id, c, r3) {
                Target::MathMissed
            } else {
                Target::Excel
            };
        }
    }
}

/// Classifies cell `c` of case `case`, a visibly-off cell or an R3 cell where the engine returns
/// the math (`None` for any other cell).
#[must_use]
pub fn classify(
    case: &str,
    c: &CellResult,
    r3: &R3Marks,
    exceptions: &Phase1Exceptions,
) -> Option<R1c> {
    let r3_pass = r3_math(case, c, r3);
    if !(Visible::is_failing_off(c) || c.verdict.is_failure() && r3_pass) {
        return None;
    }
    let key = c.key();
    // R7 extends decision 0143 to the milestone probe rows only: an M14-owned ironcalc-calc cell
    // still counts in the calc rows (decision 0146 item 3 leaves those to exit exceptions).
    let probe_row = case.split('/').next() == Some(probe_milestones::SUITE);
    if probe_row && exceptions.desktop_owns(case, &key) {
        return Some(R1c::M14Owned);
    }
    if r3_pass {
        return Some(R1c::R3Pass);
    }
    Some(match cell_envelope(c) {
        CellEnvelope::Inside => R1c::Phase1b,
        CellEnvelope::Outside | CellEnvelope::WrongType => R1c::Counts,
    })
}

/// Every visibly-off cell of a run with its [`R1c`] class, by `case!Sheet!A1`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct R1cCells {
    cells: BTreeMap<String, R1c>,
    /// The [`R1c::Counts`] cells' envelope class, `Outside` or `WrongType`.
    misses: BTreeMap<String, CellEnvelope>,
}

impl R1cCells {
    /// Classifies every visibly-off cell of `results`.
    #[must_use]
    pub fn build(results: &[CaseResult], r3: &R3Marks, exceptions: &Phase1Exceptions) -> Self {
        let mut cells = BTreeMap::new();
        let mut misses = BTreeMap::new();
        for r in results {
            for c in &r.cells {
                if let Some(class) = classify(&r.id, c, r3, exceptions) {
                    let id = cell_id(&r.id, &c.key());
                    if class == R1c::Counts {
                        misses.insert(id.clone(), cell_envelope(c));
                    }
                    cells.insert(id, class);
                }
            }
        }
        Self { cells, misses }
    }

    /// Classes given directly, by `case!Sheet!A1`, for tests of the readers.
    #[cfg(test)]
    pub(crate) fn from_classes<'a>(cells: impl IntoIterator<Item = (&'a str, R1c)>) -> Self {
        Self {
            cells: cells
                .into_iter()
                .map(|(id, c)| (id.to_string(), c))
                .collect(),
            misses: BTreeMap::new(),
        }
    }

    /// These classes with counted cell `id` an [`CellEnvelope::Outside`] number, for tests of the
    /// tier bars (a counted cell reads as a wrong type otherwise).
    #[cfg(test)]
    pub(crate) fn with_outside(mut self, id: &str) -> Self {
        self.cells.insert(id.to_string(), R1c::Counts);
        self.misses.insert(id.to_string(), CellEnvelope::Outside);
        self
    }

    /// The class of a visibly-off cell by its `case!Sheet!A1`; [`R1c::Counts`] for a cell the run
    /// did not classify, so nothing reads as met early.
    #[must_use]
    pub fn class(&self, id: &str) -> R1c {
        self.cells.get(id).copied().unwrap_or(R1c::Counts)
    }

    /// The cells R1(c) leaves out of the count, with their class: what the report writes as
    /// `r1c_left_out` for `scripts/exit_owners.py` (T-3428). Counted cells are the rest.
    #[must_use]
    pub fn left_out(&self) -> BTreeMap<String, R1c> {
        self.cells
            .iter()
            .filter(|(_, c)| **c != R1c::Counts)
            .map(|(id, c)| (id.clone(), *c))
            .collect()
    }

    /// Whether R1(c) counts the visibly-off cell `id`.
    #[must_use]
    pub fn counts(&self, id: &str) -> bool {
        self.class(id) == R1c::Counts
    }

    /// How a visibly-off cell `id` misses decision 0355's envelope when R1(c) counts it:
    /// [`CellEnvelope::Outside`] or [`CellEnvelope::WrongType`]; `None` for a cell R1(c) leaves
    /// out. A cell the run did not classify reads as a wrong type, the stricter bar, so nothing
    /// reads as met early.
    #[must_use]
    pub fn miss(&self, id: &str) -> Option<CellEnvelope> {
        self.counts(id).then(|| {
            self.misses
                .get(id)
                .copied()
                .unwrap_or(CellEnvelope::WrongType)
        })
    }

    /// The Phase 1b evidence of the run: per suite, the visibly-off numeric cells inside decision
    /// 0355's envelope that the probe rows leave out.
    #[must_use]
    pub fn phase1b(&self) -> Phase1bEvidence {
        let mut out = Phase1bEvidence::default();
        for (id, class) in &self.cells {
            let suite = id.split('/').next().unwrap_or_default();
            if !EVIDENCE_SUITES.contains(&suite) {
                continue;
            }
            let row = out.suites.entry(suite.to_string()).or_default();
            match class {
                R1c::Counts => row.counts += 1,
                R1c::Phase1b => row.phase1b += 1,
                R1c::R3Pass => row.r3_pass += 1,
                R1c::M14Owned => row.m14_owned += 1,
            }
        }
        out
    }
}

/// The suites whose visibly-off numeric cells R1(c) reads as Phase 1b evidence.
pub const EVIDENCE_SUITES: [&str; 3] = ["probes", "probes-numeric", "ironcalc-calc"];

/// One suite's visibly-off cells by [`R1c`] class.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct R1cCounts {
    /// Cells R1(c) counts ([`R1c::Counts`]).
    pub counts: usize,
    /// Phase 1b evidence ([`R1c::Phase1b`]).
    pub phase1b: usize,
    /// R3 cells where the engine returns the math ([`R1c::R3Pass`]).
    pub r3_pass: usize,
    /// Cells only open M14 tasks own ([`R1c::M14Owned`]).
    pub m14_owned: usize,
}

/// R1(c)'s classes per [`EVIDENCE_SUITES`] suite the run scored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase1bEvidence {
    /// Suite name → its visibly-off cells by class.
    pub suites: BTreeMap<String, R1cCounts>,
}

impl Phase1bEvidence {
    /// The report line: per suite, the counted cells, the Phase 1b evidence, the R3 passes and
    /// the M14-owned cells; nothing when no evidence suite ran.
    #[must_use]
    pub fn line(&self) -> Option<String> {
        if self.suites.is_empty() {
            return None;
        }
        let parts: Vec<String> = self
            .suites
            .iter()
            .map(|(suite, c)| {
                format!(
                    "{suite} {} counted, {} Phase 1b evidence, {} R3 passing, {} M14-owned",
                    c.counts, c.phase1b, c.r3_pass, c.m14_owned
                )
            })
            .collect();
        Some(format!(
            "Visibly-off cells under decision 0188 R1(c): {}. Phase 1b evidence is a number inside decision 0355's envelope: within 12 significant digits of Excel, or within 1e-9 of its 50-digit math value. R3 passing cells are the `r3_math` bucket, not visibly off (decision 0310).",
            parts.join("; ")
        ))
    }
}

#[cfg(test)]
mod tests;
