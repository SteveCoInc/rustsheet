//! The round-trip scenarios (SPEC-0033 D1, D8): the one table of their names and what each runs,
//! and the runner that plays a case through the scenarios its suite lists. R0 always runs first,
//! because idempotence and every R1 edit are measured against it.

use std::collections::BTreeMap;

use rustsheet::{LoadOptions, SaveOptions, Workbook};

use super::package::Package;
use super::r1::{self, Edit};
use super::{score, Item, Sides, Verdict};
use crate::engine::harness_options;

/// The plain round trip.
pub(crate) const R0: &str = "R0";

/// What a scenario runs.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Run {
    /// Load, recalculate, save (R0).
    Plain,
    /// Save our own R0 output again (idempotence, D8).
    Again,
    /// Apply these standard edits before saving (R1, D1).
    Edits(&'static [Edit]),
}

/// A scenario family, as the rates line reports them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Family {
    R0,
    R1,
    Again,
}

/// Every scenario the harness implements, by name. R2 (structural edits) is M4.
pub(crate) const SCENARIOS: &[(&str, Run)] = &[
    (R0, Run::Plain),
    ("R0/again", Run::Again),
    ("R1/E1", Run::Edits(&[Edit::E1])),
    ("R1/E2", Run::Edits(&[Edit::E2])),
    ("R1/E3", Run::Edits(&[Edit::E3])),
    ("R1/E4", Run::Edits(&[Edit::E4])),
    ("R1/E5", Run::Edits(&[Edit::E5])),
    ("R1/E6", Run::Edits(&[Edit::E6])),
    (
        "R1/all",
        Run::Edits(&[Edit::E1, Edit::E2, Edit::E3, Edit::E4, Edit::E5, Edit::E6]),
    ),
];

/// The scenario called `name`, with its name as the table holds it.
pub(crate) fn find(name: &str) -> Option<(&'static str, Run)> {
    SCENARIOS.iter().copied().find(|(n, _)| *n == name)
}

/// The family of the scenario called `name`, or `None` for a name the table lacks.
pub(crate) fn family(name: &str) -> Option<Family> {
    find(name).map(|(_, run)| match run {
        Run::Plain => Family::R0,
        Run::Again => Family::Again,
        Run::Edits(_) => Family::R1,
    })
}

/// Why a case could not run R0.
pub(super) enum R0Error {
    /// The source package itself cannot be read: the case is unusable.
    Source(String),
    /// The engine refused to load or save it.
    Engine(String),
}

/// What a case's run gives back: its items (R0's, then each further scenario's failures), the
/// source items each scenario compared, and for each R1 scenario the writes the engine refused
/// and the edits that had no target.
pub(super) struct CaseRun {
    pub(super) items: Vec<Item>,
    pub(super) checked: BTreeMap<&'static str, usize>,
    pub(super) skipped: BTreeMap<&'static str, (usize, usize)>,
}

/// Runs R0 and then each further scenario of `scenarios` on one case. Only R0 failing makes the
/// case fail to run: a further scenario that fails is one `scenario error` item, so R0's items
/// still count.
pub(super) fn run_scenarios(
    bytes: &[u8],
    name: Option<&str>,
    scenarios: &[String],
) -> Result<CaseRun, R0Error> {
    let source = Package::read(bytes).map_err(|e| R0Error::Source(e.to_string()))?;
    let (mut items, checked, out) = r0(&source, bytes, name)?;
    let mut run = CaseRun {
        items: Vec::new(),
        checked: BTreeMap::from([(R0, checked)]),
        skipped: BTreeMap::new(),
    };
    let plain = items.clone();
    for (label, kind) in scenarios.iter().filter_map(|s| find(s)) {
        let result = match kind {
            // Always run first: every other scenario is measured against it.
            Run::Plain => continue,
            Run::Again => again(&out, name, label),
            Run::Edits(edits) => {
                run_r1(&source, bytes, name, label, edits, &plain).map(|(found, outcome)| {
                    run.skipped
                        .insert(label, (outcome.refused, outcome.missing));
                    found
                })
            }
        };
        match result {
            Ok(Some((found, checked))) => {
                run.checked.insert(label, checked);
                items.extend(found);
            }
            // No edit had a target: the scenario is `n/a` for the case.
            Ok(None) => {}
            Err(R0Error::Source(e) | R0Error::Engine(e)) => items.push(Item {
                scenario: label,
                kind: "scenario",
                key: "error".into(),
                verdict: Verdict::PackageInvalid,
                detail: Some(e),
                candidate: None,
                weight: 0,
            }),
        }
    }
    run.items = items;
    Ok(run)
}

/// Idempotence (D8): our own R0 output, saved again, keeps every item.
fn again(
    out: &[u8],
    name: Option<&str>,
    label: &'static str,
) -> Result<Option<(Vec<Item>, usize)>, R0Error> {
    let source = Package::read(out).map_err(|e| R0Error::Engine(e.to_string()))?;
    let (found, checked, _) = r0(&source, out, name)?;
    let failing = found
        .into_iter()
        .filter(|i| !i.verdict.passes())
        .map(|i| Item {
            scenario: label,
            ..i
        })
        .collect();
    Ok(Some((failing, checked)))
}

/// Loads `bytes` with the harness clock and recalculates, as every scenario starts.
fn load(bytes: &[u8], name: Option<&str>) -> Result<Workbook, R0Error> {
    let opts = LoadOptions {
        recalc: false,
        workbook: harness_options(name).map_err(R0Error::Engine)?,
        ..LoadOptions::default()
    };
    let mut workbook = Workbook::from_xlsx_bytes(bytes, &opts)
        .map_err(|e| R0Error::Engine(format!("import failed: {e}")))?;
    workbook.pin_clock_from_metadata();
    workbook.recalculate();
    Ok(workbook)
}

fn save(workbook: &Workbook) -> Result<Vec<u8>, R0Error> {
    workbook
        .to_xlsx_bytes(SaveOptions::default())
        .map_err(|e| R0Error::Engine(format!("save failed: {e}")))
}

/// Scenario R0 (SPEC-0033 D1): load with the harness clock, recalculate, save; then validate the
/// output and score the source's parts, elements and attributes. Returns the items, the number
/// of source items compared and the saved package.
pub(super) fn r0(
    source: &Package,
    bytes: &[u8],
    name: Option<&str>,
) -> Result<(Vec<Item>, usize, Vec<u8>), R0Error> {
    let workbook = load(bytes, name)?;
    let out = save(&workbook)?;
    let sides = Sides {
        source,
        source_bytes: bytes,
        saved: &workbook,
        name: name.unwrap_or("workbook"),
    };
    let (items, checked) = score(R0, &sides, &out);
    Ok((items, checked, out))
}

/// One R1 scenario (D1, D5): load, recalculate, apply `edits`, save, score, and keep the delta
/// violations against the plain save's items. `None` when no edit applied (`n/a`); the outcome
/// counts the refused writes and the edits with no target.
fn run_r1(
    source: &Package,
    bytes: &[u8],
    name: Option<&str>,
    label: &'static str,
    edits: &[Edit],
    plain: &[Item],
) -> Result<(Option<(Vec<Item>, usize)>, r1::Outcome), R0Error> {
    let mut workbook = load(bytes, name)?;
    let (delta, outcome) = r1::apply(&mut workbook, edits).map_err(R0Error::Engine)?;
    let Some(delta) = delta else {
        return Ok((None, outcome));
    };
    let out = save(&workbook)?;
    let sides = Sides {
        source,
        source_bytes: bytes,
        saved: &workbook,
        name: name.unwrap_or("workbook"),
    };
    let (items, checked) = score(label, &sides, &out);
    let found = r1::violations(label, plain, items, &delta);
    Ok((Some((found, checked)), outcome))
}
