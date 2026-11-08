//! The usage-tier gates of decision 0355 item 3 as the report writes them (SPEC-0037 design 3,
//! T-5092): report.json `usage_gates` and the report.md "Usage tiers" section, Phase 1's exit
//! measure in place of decision 0188's Gate A and Gate B.
//!
//! The counting is [`gate_tiers`]'s, the one computation the CI tier lock reads too: each scored,
//! non-deviation Excel cell of the probe suites and Gate B's suites, toward its formula's sole
//! non-glue call (decision 0118), classed by `cell_envelope`; left out are the decision 0134
//! exceptions, the pivot suites and GETPIVOTDATA (decision 0354). There is no evidence-floor
//! fallback: a cell counts toward one function at most, so the pooled counts are cells, never
//! counted twice. This module only writes the full run's tiers out, with each function's row and
//! the functions behind the pooled misses ([`TierGate::contributors`]). The accuracy tables'
//! points join the tiers in `scripts/phase1_exit.py`, which reads eval/runs/accuracy.json.
//!
//! [`TierTallies`] carries the same pooled bars to the milestone rows (decision 0355 item 4), each
//! cell placed in a tier by [`cell_usage`]. A report, never a CI gate.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, Usage};
use crate::closeness::sole_call;
use crate::gate_tiers::{self, usage_name, Bars, Cells, PIVOT_FUNCTION};
use crate::milestone_exit::RATE_DEN;
use crate::phase1_exceptions::Phase1Exceptions;
use crate::runner::CaseResult;

/// Functions a tier's [`TierGate::contributors`] lists.
const CONTRIBUTORS: usize = 20;

/// A pair of bars as report.json writes them: `{"type": 1.0, "envelope": 0.9999}`; 0 is no bar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Bar {
    /// The share right on type, error and text.
    #[serde(rename = "type")]
    pub type_share: f64,
    /// The share of numbers inside the envelope.
    pub envelope: f64,
}

impl Bar {
    /// The bar of `type_num` and `envelope_num` over [`RATE_DEN`].
    fn of(type_num: usize, envelope_num: usize) -> Bar {
        Bar {
            type_share: ratio(type_num, RATE_DEN),
            envelope: ratio(envelope_num, RATE_DEN),
        }
    }
}

/// `num / den` as a double; both stay far below 2^52, so the casts are exact.
#[allow(clippy::cast_precision_loss)]
fn ratio(num: usize, den: usize) -> f64 {
    num as f64 / den as f64
}

/// `(cells − misses) / cells`, the share decision 0355 measures; 1 without cells.
fn rate(cells: usize, misses: usize) -> f64 {
    if cells == 0 {
        1.0
    } else {
        ratio(cells.saturating_sub(misses), cells)
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
/// bars ([`Bars::of`]) within the row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierTallies {
    /// The row's foundational cells.
    pub foundational: Cells,
    /// The row's expert cells.
    pub expert: Cells,
}

impl TierTallies {
    /// The cells of `usage`.
    pub fn tally_mut(&mut self, usage: Usage) -> &mut Cells {
        match usage {
            Usage::Foundational => &mut self.foundational,
            Usage::Expert => &mut self.expert,
        }
    }

    /// Whether both tiers meet their pooled bars; a tier without cells or misses does. A tier
    /// with more misses than cells (a row whose cells were not counted, as in a report from before
    /// the tiers) never does, so nothing reads as met early.
    #[must_use]
    pub fn meets_bars(&self) -> bool {
        [
            (Usage::Foundational, &self.foundational),
            (Usage::Expert, &self.expert),
        ]
        .into_iter()
        .all(|(usage, cells)| {
            let bars = Bars::of(usage);
            cells.wrong_type + cells.outside <= cells.cells
                && cells.meets(bars.pooled_type, bars.pooled_envelope)
        })
    }

    /// `F <wrong type>/<outside> of <cells>, E …` for the report table.
    #[must_use]
    pub fn text(&self) -> String {
        let one = |t: &Cells| format!("{}/{} of {}", t.wrong_type, t.outside, t.cells);
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
    /// Its attributed cells (decision 0118, sole call).
    pub cells: usize,
    /// Of those, a wrong type, error, text or boolean.
    pub wrong_type: usize,
    /// Of those, a number outside the envelope.
    pub outside: usize,
    /// `(cells − wrong_type) / cells`; 1 without cells.
    pub type_rate: f64,
    /// `(cells − outside) / cells`; 1 without cells.
    pub envelope_rate: f64,
    /// Both shares meet the tier's per-function bars; true with no counted cell (coverage and
    /// the evidence floor gate that elsewhere).
    pub meets: bool,
}

/// One tier's gate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TierGate {
    /// The tier.
    pub usage: Usage,
    /// In-scope catalog functions of the tier, GETPIVOTDATA left out (decision 0354).
    pub functions: usize,
    /// Their counted cells, pooled.
    pub cells: usize,
    /// Of those, a wrong type, error, text or boolean.
    pub wrong_type: usize,
    /// Of those, a number outside the envelope.
    pub outside: usize,
    /// `(cells − wrong_type) / cells`; 1 without cells.
    pub type_rate: f64,
    /// `(cells − outside) / cells`; 1 without cells.
    pub envelope_rate: f64,
    /// The pooled bars.
    pub pooled_bar: Bar,
    /// Each function's bars; 0 is no bar (the foundational envelope bar is pooled only).
    pub function_bar: Bar,
    /// The functions below a per-function bar, most misses first.
    pub failing_functions: Vec<String>,
    /// The functions with any miss, most first, at most 20: where the pooled misses come from
    /// (additive to SPEC-0037's schema, for the dashboard).
    #[serde(default)]
    pub contributors: Vec<String>,
    /// Both pooled bars and every function's bars hold.
    pub met: bool,
}

/// report.json's `usage_gates`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageGates {
    /// The counted suites the run scored.
    pub suites: Vec<String>,
    /// Foundational, then expert.
    pub tiers: Vec<TierGate>,
    /// Every in-scope catalog function but GETPIVOTDATA: those below their bars first, then
    /// most misses first.
    pub functions: Vec<FunctionGate>,
    /// Every tier is met.
    pub met: bool,
}

/// Both kinds of miss of `f`.
fn misses(f: &FunctionGate) -> usize {
    f.wrong_type + f.outside
}

/// The tier gates over every cell of `results` (not only the failing ones the report keeps);
/// `exceptions` lists the decision 0134 cells left out.
#[must_use]
pub fn summarize(
    catalog: &Catalog,
    results: &[CaseResult],
    exceptions: &Phase1Exceptions,
) -> UsageGates {
    let counted = gate_tiers::function_cells(catalog, results, exceptions);
    let mut functions: Vec<FunctionGate> = catalog
        .in_scope_functions()
        .filter(|f| f.name != PIVOT_FUNCTION)
        .map(|f| {
            let c = counted.get(&f.name).copied().unwrap_or_default();
            let bars = Bars::of(f.usage);
            FunctionGate {
                name: f.name.clone(),
                usage: f.usage,
                cells: c.cells,
                wrong_type: c.wrong_type,
                outside: c.outside,
                type_rate: rate(c.cells, c.wrong_type),
                envelope_rate: rate(c.cells, c.outside),
                meets: c.meets(bars.function_type, bars.function_envelope),
            }
        })
        .collect();
    functions.sort_by(|a, b| {
        a.meets
            .cmp(&b.meets)
            .then_with(|| misses(b).cmp(&misses(a)))
            .then_with(|| a.name.cmp(&b.name))
    });
    let tiers: Vec<TierGate> = gate_tiers::tiers(catalog, &counted)
        .into_iter()
        .map(|t| {
            let bars = Bars::of(t.usage);
            let mut contributors: Vec<&FunctionGate> = functions
                .iter()
                .filter(|f| f.usage == t.usage && misses(f) > 0)
                .collect();
            contributors
                .sort_by(|a, b| misses(b).cmp(&misses(a)).then_with(|| a.name.cmp(&b.name)));
            TierGate {
                usage: t.usage,
                functions: t.functions,
                cells: t.cells.cells,
                wrong_type: t.cells.wrong_type,
                outside: t.cells.outside,
                type_rate: rate(t.cells.cells, t.cells.wrong_type),
                envelope_rate: rate(t.cells.cells, t.cells.outside),
                pooled_bar: Bar::of(bars.pooled_type, bars.pooled_envelope),
                function_bar: Bar::of(bars.function_type, bars.function_envelope),
                failing_functions: t.failing_functions,
                contributors: contributors
                    .into_iter()
                    .take(CONTRIBUTORS)
                    .map(|f| f.name.clone())
                    .collect(),
                met: t.met,
            }
        })
        .collect();
    let suites: BTreeSet<&str> = results
        .iter()
        .map(|r| r.suite.as_str())
        .filter(|s| gate_tiers::counts_suite(s))
        .collect();
    UsageGates {
        suites: suites.into_iter().map(String::from).collect(),
        met: tiers.iter().all(|t| t.met),
        tiers,
        functions,
    }
}

/// Functions listed per list in report.md; the JSON keeps every one.
const SHOWN: usize = 30;

/// A share as a percentage with enough digits to show a 99.99% bar's misses.
fn pct(share: f64) -> String {
    format!("{:.4}%", share * 100.0)
}

/// A bar as the report table shows it.
fn bar_text(b: Bar) -> String {
    if b.envelope == 0.0 {
        format!("{}, envelope pooled only", pct(b.type_share))
    } else {
        format!("{}, {}", pct(b.type_share), pct(b.envelope))
    }
}

/// The function lines of `names`, at most [`SHOWN`].
fn list(by_name: &BTreeMap<&str, &FunctionGate>, names: &[String], md: &mut String) {
    for f in names
        .iter()
        .take(SHOWN)
        .filter_map(|n| by_name.get(n.as_str()))
    {
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
    if names.len() > SHOWN {
        let _ = writeln!(md, "- and {} more", names.len() - SHOWN);
    }
}

/// The report section: one row per tier, then each tier's failing functions and the other
/// functions behind its pooled misses, worst first.
pub fn write_markdown(gates: &UsageGates, md: &mut String) {
    if gates.tiers.is_empty() {
        return;
    }
    let _ = writeln!(md, "\n## Usage tiers (decision 0355)\n");
    let _ = writeln!(
        md,
        "**Usage tiers: {}.** Phase 1's exit gates (they replace Gate A and Gate B, whose numbers stay below as information). A cell is right on type when its type, error, text or boolean matches Excel; a number is inside the envelope when it is within 12 significant digits of Excel's or within 1e-9 of the 50-digit math. Counted: the Excel cells of {}, each toward its formula's sole non-glue call (decision 0118); left out: the decision 0134 exceptions, deviations, the pivot suites and GETPIVOTDATA. `make phase1-exit` adds the accuracy tables' points.\n",
        if gates.met { "met" } else { "not met" },
        gates.suites.join(", ")
    );
    let _ = writeln!(
        md,
        "| tier | functions | cells | wrong type | type right | outside | inside the envelope | bars: pooled (type, envelope); per function | met |\n|---|---|---|---|---|---|---|---|---|"
    );
    for t in &gates.tiers {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} | {}; {} | {} |",
            usage_name(t.usage),
            t.functions,
            t.cells,
            t.wrong_type,
            pct(t.type_rate),
            t.outside,
            pct(t.envelope_rate),
            bar_text(t.pooled_bar),
            bar_text(t.function_bar),
            if t.met { "met" } else { "not met" }
        );
    }
    let by_name: BTreeMap<&str, &FunctionGate> = gates
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    for t in &gates.tiers {
        let name = usage_name(t.usage);
        if !t.failing_functions.is_empty() {
            let n = t.failing_functions.len();
            let _ = writeln!(
                md,
                "\n{name} functions below their bars ({n}), worst first:\n"
            );
            list(&by_name, &t.failing_functions, md);
        }
        let others: Vec<String> = t
            .contributors
            .iter()
            .filter(|c| !t.failing_functions.contains(c))
            .cloned()
            .collect();
        if !others.is_empty() {
            let _ = writeln!(
                md,
                "\nOther {name} functions with a miss in the pooled counts, most first:\n"
            );
            list(&by_name, &others, md);
        }
    }
}

#[cfg(test)]
mod tests;
