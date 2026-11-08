//! The usage-tier gates of decision 0355 item 3 (SPEC-0037 design 3, T-5092), which replace
//! decision 0188's Gate A and Gate B as Phase 1's exit measure: report.json `usage_gates` and the
//! report.md "Usage tiers" section.
//!
//! Every in-scope catalog function is foundational or expert (`catalog::Usage`). Its cells are
//! classified by the one per-cell envelope test, [`cell_envelope`]: a wrong type, error, text or
//! boolean, a number outside the envelope, or inside. The bars:
//!
//! | | type, error and text right | inside the envelope |
//! |---|---|---|
//! | foundational | 100% pooled | ≥ 99.99% pooled |
//! | expert | ≥ 99.9% pooled, ≥ 99% per function | ≥ 99.9% pooled, ≥ 99% per function |
//!
//! The cells counted are the Excel cells of `probes`, `probes-numeric` and Gate B's suites
//! ([`gate_b::counts`]), attributed as decision 0118 says (a function's sole-call cells, or every
//! cell that calls it below the evidence floor), so a pooled count sums the functions' counts.
//! Left out: the decision 0134 exceptions ([`Phase1Exceptions::contains`]), the decision 0036
//! deviations, the pivot suites and every cell that calls GETPIVOTDATA (Phase 2, decision 0354).
//! The accuracy tables' points join the tiers in `scripts/phase1_exit.py`, which reads
//! eval/runs/accuracy.json: the report does not run the accuracy check.
//!
//! [`TierTallies`] carries the same bars to the milestone rows (decision 0355 item 4), pooled
//! within a row, with each cell placed in a tier by [`cell_usage`]. A report, never a CI gate.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::accuracy::envelope::{cell_envelope, CellEnvelope};
use crate::attribution::Attribution;
use crate::catalog::{Catalog, Usage};
use crate::closeness::sole_call;
use crate::formula_calls::functions_in;
use crate::gate_b;
use crate::milestone_exit::{meets_rate, RATE_DEN};
use crate::phase1_exceptions::Phase1Exceptions;
use crate::runner::CaseResult;
use crate::value::Verdict;

/// The suites counted besides Gate B's (decision 0355 item 3).
const PROBE_SUITES: [&str; 2] = ["probes", "probes-numeric"];
/// Pivot suites, left out (Phase 2, decision 0354); `pivot-corpus-*` too ([`PIVOT_CORPUS`]).
const PIVOT_SUITES: [&str; 2] = ["gen-pivot", "pivot-export"];
/// The prefix of the fetched pivot corpus suites.
const PIVOT_CORPUS: &str = "pivot-corpus-";
/// A cell that calls it is left out (Phase 2, decision 0354).
pub const LEFT_OUT_FUNCTION: &str = "GETPIVOTDATA";

/// Whether the tier gates count the cells of `suite`.
#[must_use]
pub fn counts(suite: &str) -> bool {
    let pivot = PIVOT_SUITES.contains(&suite) || suite.starts_with(PIVOT_CORPUS);
    !pivot && (PROBE_SUITES.contains(&suite) || gate_b::counts(suite))
}

/// A pair of bars, as shares over [`RATE_DEN`]: the type, error and text share and the envelope
/// share a count of cells must reach. A bar of 0 is no bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bars {
    /// Cells right on type, error and text, over [`RATE_DEN`].
    pub type_num: usize,
    /// Cells inside the envelope, over [`RATE_DEN`].
    pub envelope_num: usize,
}

impl Bars {
    /// The pooled bars of `usage`'s cells (decision 0355 item 3).
    #[must_use]
    pub fn pooled(usage: Usage) -> Bars {
        match usage {
            Usage::Foundational => Bars {
                type_num: RATE_DEN,
                envelope_num: 9_999,
            },
            Usage::Expert => Bars {
                type_num: 9_990,
                envelope_num: 9_990,
            },
        }
    }

    /// The bars of one function of `usage`. The foundational envelope bar is pooled only (0);
    /// its type bar of 100% follows from the pooled 100%.
    #[must_use]
    pub fn per_function(usage: Usage) -> Bars {
        match usage {
            Usage::Foundational => Bars {
                type_num: RATE_DEN,
                envelope_num: 0,
            },
            Usage::Expert => Bars {
                type_num: 9_900,
                envelope_num: 9_900,
            },
        }
    }
}

/// [`Bars`] as report.json writes them: `{"type": 1.0, "envelope": 0.9999}`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Bar {
    /// The type, error and text share.
    #[serde(rename = "type")]
    pub type_share: f64,
    /// The envelope share.
    pub envelope: f64,
}

impl From<Bars> for Bar {
    fn from(b: Bars) -> Bar {
        let share = |num: usize| ratio(num, RATE_DEN);
        Bar {
            type_share: share(b.type_num),
            envelope: share(b.envelope_num),
        }
    }
}

/// `num / den` as a double; both stay far below 2^52, so the casts are exact.
#[allow(clippy::cast_precision_loss)]
fn ratio(num: usize, den: usize) -> f64 {
    num as f64 / den as f64
}

/// Cells and their misses against the envelope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    /// Cells counted.
    pub cells: usize,
    /// Of those, a wrong type, error, text or boolean ([`CellEnvelope::WrongType`]).
    pub wrong_type: usize,
    /// Of those, a number outside the envelope ([`CellEnvelope::Outside`]).
    pub outside: usize,
}

impl Tally {
    /// Counts one cell of class `class`.
    pub fn add(&mut self, class: CellEnvelope) {
        self.cells += 1;
        self.miss(class);
    }

    /// Counts a miss of class `class` on a cell already counted in `cells`.
    pub fn miss(&mut self, class: CellEnvelope) {
        match class {
            CellEnvelope::Inside => {}
            CellEnvelope::Outside => self.outside += 1,
            CellEnvelope::WrongType => self.wrong_type += 1,
        }
    }

    /// Adds `other`'s counts.
    pub fn merge(&mut self, other: &Tally) {
        self.cells += other.cells;
        self.wrong_type += other.wrong_type;
        self.outside += other.outside;
    }

    /// Both kinds of miss.
    #[must_use]
    pub fn misses(&self) -> usize {
        self.wrong_type + self.outside
    }

    /// `(cells − wrong_type) / cells`; 1 without cells.
    #[must_use]
    pub fn type_rate(&self) -> f64 {
        self.rate(self.wrong_type)
    }

    /// `(cells − outside) / cells`; 1 without cells.
    #[must_use]
    pub fn envelope_rate(&self) -> f64 {
        self.rate(self.outside)
    }

    fn rate(&self, misses: usize) -> f64 {
        if self.cells == 0 {
            1.0
        } else {
            ratio(self.cells.saturating_sub(misses), self.cells)
        }
    }

    /// Whether the type share meets `bars`; true without cells.
    #[must_use]
    pub fn meets_type(&self, bars: Bars) -> bool {
        self.meets_share(self.wrong_type, bars.type_num)
    }

    /// Whether the envelope share meets `bars`; true without cells.
    #[must_use]
    pub fn meets_envelope(&self, bars: Bars) -> bool {
        self.meets_share(self.outside, bars.envelope_num)
    }

    /// Whether both shares meet `bars`; true without cells.
    #[must_use]
    pub fn meets(&self, bars: Bars) -> bool {
        self.meets_type(bars) && self.meets_envelope(bars)
    }

    /// No cell and no miss meets; a miss without cells (a report that kept the misses but not
    /// the cells) never does.
    fn meets_share(&self, misses: usize, num: usize) -> bool {
        (self.cells == 0 && misses == 0)
            || meets_rate(self.cells.saturating_sub(misses), self.cells, num)
    }
}

/// The tier of one cell that calls `called` (the functions its formula calls), for the
/// milestone rows: the tier of its sole non-glue call ([`sole_call`]) when the catalog knows it;
/// else foundational if it calls a foundational function or no catalog function at all (an
/// operator, a spill extent), else expert. `usage_of` reads a name's catalog tier. Where a cell
/// has no one function, the stricter tier holds it, so no foundational miss hides in a row's
/// expert count.
#[must_use]
pub fn cell_usage(called: &BTreeSet<String>, usage_of: impl Fn(&str) -> Option<Usage>) -> Usage {
    if let Some(u) = sole_call(called).and_then(|f| usage_of(f)) {
        return u;
    }
    let mut tiers = called.iter().filter_map(|f| usage_of(f)).peekable();
    if tiers.peek().is_none() {
        return Usage::Foundational;
    }
    if tiers.any(|u| u == Usage::Foundational) {
        Usage::Foundational
    } else {
        Usage::Expert
    }
}

/// A milestone row's cells by tier (decision 0355 item 4): met when each tier meets its pooled
/// bars within the row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierTallies {
    /// The row's foundational cells.
    pub foundational: Tally,
    /// The row's expert cells.
    pub expert: Tally,
}

impl TierTallies {
    /// The tally of `usage`.
    pub fn tally_mut(&mut self, usage: Usage) -> &mut Tally {
        match usage {
            Usage::Foundational => &mut self.foundational,
            Usage::Expert => &mut self.expert,
        }
    }

    /// Whether both tiers meet their pooled bars ([`Bars::pooled`]); a tier without cells does.
    #[must_use]
    pub fn meets_bars(&self) -> bool {
        self.foundational.meets(Bars::pooled(Usage::Foundational))
            && self.expert.meets(Bars::pooled(Usage::Expert))
    }

    /// `F <wrong type>/<outside> of <cells>, E …` for the report table.
    #[must_use]
    pub fn text(&self) -> String {
        let one = |t: &Tally| format!("{}/{} of {}", t.wrong_type, t.outside, t.cells);
        format!("F {}, E {}", one(&self.foundational), one(&self.expert))
    }
}

/// One in-scope function against its tier's per-function bars.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FunctionGate {
    /// Upper-case Excel name.
    pub name: String,
    /// Its tier.
    pub usage: Usage,
    /// The cells decision 0118 counts it by.
    pub cells: usize,
    /// Of those, a wrong type, error, text or boolean.
    pub wrong_type: usize,
    /// Of those, a number outside the envelope.
    pub outside: usize,
    /// `(cells − wrong_type) / cells`; 1 without cells.
    pub type_rate: f64,
    /// `(cells − outside) / cells`; 1 without cells.
    pub envelope_rate: f64,
    /// Both rates meet [`Bars::per_function`]; true with no counted cell (coverage and the
    /// evidence floor gate that elsewhere).
    pub meets: bool,
}

impl FunctionGate {
    fn tally(&self) -> Tally {
        Tally {
            cells: self.cells,
            wrong_type: self.wrong_type,
            outside: self.outside,
        }
    }
}

/// One tier's gate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TierGate {
    /// The tier.
    pub usage: Usage,
    /// In-scope catalog functions of the tier.
    pub functions: usize,
    /// Their counted cells, summed.
    pub cells: usize,
    /// Of those, a wrong type, error, text or boolean.
    pub wrong_type: usize,
    /// Of those, a number outside the envelope.
    pub outside: usize,
    /// `(cells − wrong_type) / cells`; 1 without cells.
    pub type_rate: f64,
    /// `(cells − outside) / cells`; 1 without cells.
    pub envelope_rate: f64,
    /// [`Bars::pooled`].
    pub pooled_bar: Bar,
    /// [`Bars::per_function`]; 0 is no bar.
    pub function_bar: Bar,
    /// The functions keeping the tier from its bars, worst first (most misses): each below the
    /// per-function bars, and, while a pooled bar is unmet, each with a miss of that kind.
    pub failing_functions: Vec<String>,
    /// Counted cells, both pooled bars met and no function below the per-function bars.
    pub met: bool,
}

/// report.json's `usage_gates`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageGates {
    /// The counted suites the run scored.
    pub suites: Vec<String>,
    /// Foundational, then expert.
    pub tiers: Vec<TierGate>,
    /// Every in-scope catalog function, those failing first, then most misses first.
    pub functions: Vec<FunctionGate>,
    /// Every tier is met.
    pub met: bool,
}

/// The tier gates over `results` (every cell, not only the failing ones the report keeps);
/// `exceptions` lists the decision 0134 cells left out.
#[must_use]
pub fn summarize(
    catalog: &Catalog,
    results: &[CaseResult],
    exceptions: &Phase1Exceptions,
) -> UsageGates {
    // Per function, its sole-call cells and every cell that calls it (decision 0118).
    let mut sole: BTreeMap<String, Tally> = BTreeMap::new();
    let mut all: BTreeMap<String, Tally> = BTreeMap::new();
    let mut suites = BTreeSet::new();
    for r in results.iter().filter(|r| counts(&r.suite)) {
        suites.insert(r.suite.clone());
        for c in &r.cells {
            // Excel evidence only, as decision 0118 counts; a by-design value says nothing.
            if !c.from_excel
                || c.verdict == Verdict::Deviation
                || exceptions.contains(&r.id, &c.key())
            {
                continue;
            }
            let called = c.formula.as_deref().map(functions_in).unwrap_or_default();
            if called.contains(LEFT_OUT_FUNCTION) {
                continue;
            }
            let class = cell_envelope(c);
            let only = sole_call(&called);
            for f in &called {
                all.entry(f.clone()).or_default().add(class);
                if only == Some(f) {
                    sole.entry(f.clone()).or_default().add(class);
                }
            }
        }
    }
    let mut functions: Vec<FunctionGate> = catalog
        .in_scope_functions()
        .map(|f| {
            let attributed = sole.get(&f.name).copied().unwrap_or_default();
            let t = match Attribution::for_attributed(attributed.cells) {
                Attribution::Sole => attributed,
                Attribution::All => all.get(&f.name).copied().unwrap_or_default(),
            };
            FunctionGate {
                name: f.name.clone(),
                usage: f.usage,
                cells: t.cells,
                wrong_type: t.wrong_type,
                outside: t.outside,
                type_rate: t.type_rate(),
                envelope_rate: t.envelope_rate(),
                meets: t.meets(Bars::per_function(f.usage)),
            }
        })
        .collect();
    functions.sort_by(|a, b| {
        a.meets
            .cmp(&b.meets)
            .then_with(|| b.tally().misses().cmp(&a.tally().misses()))
            .then_with(|| a.name.cmp(&b.name))
    });
    let tiers: Vec<TierGate> = [Usage::Foundational, Usage::Expert]
        .into_iter()
        .map(|usage| tier_gate(usage, &functions))
        .collect();
    UsageGates {
        suites: suites.into_iter().collect(),
        met: tiers.iter().all(|t| t.met),
        tiers,
        functions,
    }
}

/// `usage`'s gate from the function rows (sorted worst first).
fn tier_gate(usage: Usage, functions: &[FunctionGate]) -> TierGate {
    let mine: Vec<&FunctionGate> = functions.iter().filter(|f| f.usage == usage).collect();
    let mut pooled = Tally::default();
    for f in &mine {
        pooled.merge(&f.tally());
    }
    let bars = Bars::pooled(usage);
    let (type_met, envelope_met) = (pooled.meets_type(bars), pooled.meets_envelope(bars));
    let mut failing: Vec<&FunctionGate> = mine
        .iter()
        .copied()
        .filter(|f| {
            !f.meets || (!type_met && f.wrong_type > 0) || (!envelope_met && f.outside > 0)
        })
        .collect();
    failing.sort_by(|a, b| {
        b.tally()
            .misses()
            .cmp(&a.tally().misses())
            .then_with(|| a.name.cmp(&b.name))
    });
    TierGate {
        usage,
        functions: mine.len(),
        cells: pooled.cells,
        wrong_type: pooled.wrong_type,
        outside: pooled.outside,
        type_rate: pooled.type_rate(),
        envelope_rate: pooled.envelope_rate(),
        pooled_bar: bars.into(),
        function_bar: Bars::per_function(usage).into(),
        met: pooled.cells > 0 && type_met && envelope_met && mine.iter().all(|f| f.meets),
        failing_functions: failing.iter().map(|f| f.name.clone()).collect(),
    }
}

/// Failing functions listed per tier in report.md; the JSON keeps every one.
const SHOWN: usize = 30;

/// `usage`'s name in the report.
fn tier_name(usage: Usage) -> &'static str {
    match usage {
        Usage::Foundational => "foundational",
        Usage::Expert => "expert",
    }
}

/// A share as a percentage with enough digits to show a 99.99% bar's misses.
fn pct(share: f64) -> String {
    format!("{:.4}%", share * 100.0)
}

/// The report section: one row per tier, then each tier's failing functions, worst first.
pub fn write_markdown(gates: &UsageGates, md: &mut String) {
    if gates.tiers.is_empty() {
        return;
    }
    let _ = writeln!(md, "\n## Usage tiers (decision 0355)\n");
    let _ = writeln!(
        md,
        "**Usage tiers: {}.** Phase 1's exit gates (they replace Gate A and Gate B, whose numbers stay below as information). A cell is right on type when its type, error, text or boolean matches Excel, and inside the envelope when its number is within 12 significant digits of Excel's or within 1e-9 of the 50-digit math. Counted: the Excel cells of {}, attributed as decision 0118 says; left out: the decision 0134 exceptions, deviations, the pivot suites and GETPIVOTDATA. The accuracy tables' points are added by `make phase1-exit`.\n",
        if gates.met { "met" } else { "not met" },
        gates.suites.join(", ")
    );
    let _ = writeln!(
        md,
        "| tier | functions | cells | wrong type | type right | outside | inside the envelope | bars: pooled (type, envelope); per function | met |\n|---|---|---|---|---|---|---|---|---|"
    );
    for t in &gates.tiers {
        let bar = |b: Bar| {
            if b.envelope == 0.0 {
                format!("{}, pooled only", pct(b.type_share))
            } else {
                format!("{}, {}", pct(b.type_share), pct(b.envelope))
            }
        };
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} | {}; {} | {} |",
            tier_name(t.usage),
            t.functions,
            t.cells,
            t.wrong_type,
            pct(t.type_rate),
            t.outside,
            pct(t.envelope_rate),
            bar(t.pooled_bar),
            bar(t.function_bar),
            if t.met { "met" } else { "not met" }
        );
    }
    let by_name: BTreeMap<&str, &FunctionGate> = gates
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    for t in gates.tiers.iter().filter(|t| !t.failing_functions.is_empty()) {
        let _ = writeln!(
            md,
            "\nFailing {} functions ({}), worst first:\n",
            tier_name(t.usage),
            t.failing_functions.len()
        );
        for name in t.failing_functions.iter().take(SHOWN) {
            if let Some(f) = by_name.get(name.as_str()) {
                let _ = writeln!(
                    md,
                    "- {}: {} cells, {} wrong type ({}), {} outside ({})",
                    f.name,
                    f.cells,
                    f.wrong_type,
                    pct(f.type_rate),
                    f.outside,
                    pct(f.envelope_rate)
                );
            }
        }
        if t.failing_functions.len() > SHOWN {
            let _ = writeln!(md, "- and {} more", t.failing_functions.len() - SHOWN);
        }
    }
}

#[cfg(test)]
mod tests;
