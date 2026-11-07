//! Round-trip suites (SPEC-0033): each workbook is loaded and saved again, the output is checked
//! by the package validator (D6), and the source's content is scored as fidelity items (D2) that
//! ratchet through the ordinary cells-mode [`Baseline`].
//!
//! This first slice (T-4172) runs scenario R0 and scores `part` items: present, and with the
//! same content type. Element and attribute items (T-4509), semantic items (T-4513) and R1
//! (T-4514) build on the same results.

mod exceptions;
mod package;
mod r1;
mod semantic;
#[cfg(test)]
mod tests;
mod xmldiff;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read as _;
use std::path::Path;
use std::time::Duration;

use rayon::prelude::*;
use rustsheet::{LoadOptions, SaveOptions, Workbook};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

use crate::baseline::{self as ratchet, Baseline};
use crate::engine::harness_options;
use crate::runner::{run_in_worker, CaseStatus};
use crate::suite::{BaselineMode, CaseFile, Suite};
use crate::xlsx_snapshot::read_snapshot;

use package::{validate, Package, Problem};

/// What happened to one fidelity item (SPEC-0033 D2). This slice gives only the verdicts a
/// part-level comparison can reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Kept,
    Changed,
    Lost,
    Added,
    /// The output failed a package-validator check (D6); the item is the check.
    PackageInvalid,
}

impl Verdict {
    fn passes(self) -> bool {
        self == Verdict::Kept
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Verdict::Kept => "kept",
            Verdict::Changed => "changed",
            Verdict::Lost => "lost",
            Verdict::Added => "added",
            Verdict::PackageInvalid => "package-invalid",
        }
    }
}

/// One scored fidelity item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    /// `R0` (D1).
    pub scenario: &'static str,
    /// `part`, or `package` for a validator check.
    pub kind: &'static str,
    /// The item's key (D2): a part key, or a validator check id.
    pub key: String,
    pub verdict: Verdict,
    /// What differs, for a failing item: content types, a validator detail. Kept out of the
    /// baseline line, which must stay stable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The candidate normalization (SPEC-0033 D4.4, `N1`–`N12`) the item would fall under once
    /// T-4503 evidences it: tagged, never applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<&'static str>,
    /// Source items the item covers that were not kept: a lost element's whole subtree, a lost
    /// XML part's elements, a `<truncated>` item's hidden differences, 1 for another change or
    /// loss, and 0 for an addition or a validator check ([`KeptRate`]).
    pub weight: usize,
}

impl Item {
    /// The item as a baseline line names it: stable across runs, without the detail.
    #[must_use]
    pub fn baseline_key(&self) -> String {
        let mut key = format!(
            "{} {} {} {}",
            self.scenario,
            self.kind,
            self.key,
            self.verdict.label()
        );
        if let Some(n) = self.candidate {
            key.push_str(" candidate-");
            key.push_str(n);
        }
        key
    }
}

/// One case of a round-trip suite.
#[derive(Debug, Clone, Serialize)]
pub struct CaseResult {
    pub id: String,
    pub suite: String,
    pub status: CaseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub millis: u64,
    /// Source items compared: parts, elements and attributes (the baseline's checked count),
    /// summed over the scenarios the case ran.
    pub checked: usize,
    /// The items each scenario compared, so each scenario's rate can be told apart.
    pub scenarios: BTreeMap<&'static str, usize>,
    /// The part items, and every element, attribute and validator item that did not pass.
    pub items: Vec<Item>,
}

impl CaseResult {
    fn new(case: &CaseFile, suite: &Suite, status: CaseStatus, message: Option<String>) -> Self {
        CaseResult {
            id: case.id.clone(),
            suite: suite.manifest.name.clone(),
            status,
            message,
            millis: 0,
            checked: 0,
            scenarios: BTreeMap::new(),
            items: Vec::new(),
        }
    }
}

/// Runs every case of `suites` in parallel, each isolated against panics and `timeout`.
#[must_use]
pub fn run_suites(suites: &[&Suite], filter: Option<&str>, timeout: Duration) -> Vec<CaseResult> {
    let mut jobs = Vec::new();
    let mut failed = Vec::new();
    for suite in suites {
        match suite.cases() {
            Ok(cases) => jobs.extend(cases.into_iter().map(|c| (*suite, c))),
            Err(e) => failed.push(CaseResult {
                id: format!("{}/<discovery>", suite.manifest.name),
                suite: suite.manifest.name.clone(),
                status: CaseStatus::InvalidCase,
                message: Some(e.to_string()),
                millis: 0,
                checked: 0,
                scenarios: BTreeMap::new(),
                items: Vec::new(),
            }),
        }
    }
    if let Some(f) = filter {
        jobs.retain(|(_, c)| c.id.contains(f));
    }
    let mut results: Vec<CaseResult> = jobs
        .par_iter()
        .map(|(suite, case)| run_case(suite, case, timeout))
        .collect();
    results.extend(failed);
    results
}

/// Runs one case: R0 in an isolated worker, so a panic or a hang is a verdict, not a crash.
fn run_case(suite: &Suite, case: &CaseFile, timeout: Duration) -> CaseResult {
    let limit = LoadOptions::default().max_uncompressed_bytes;
    let mut bytes = Vec::new();
    let read = std::fs::File::open(&case.path)
        .and_then(|f| f.take(limit.saturating_add(1)).read_to_end(&mut bytes));
    if let Err(e) = read {
        return CaseResult::new(case, suite, CaseStatus::InvalidCase, Some(e.to_string()));
    }
    let name = case
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    let scenarios = suite.manifest.scenarios.clone();
    let (outcome, millis) = run_in_worker(format!("roundtrip:{}", case.id), timeout, move || {
        run_scenarios(&bytes, name.as_deref(), &scenarios)
    });
    let mut result = match outcome {
        Err((status, message)) => CaseResult::new(case, suite, status, Some(message)),
        Ok(Err(R0Error::Source(e))) => {
            CaseResult::new(case, suite, CaseStatus::InvalidCase, Some(e))
        }
        Ok(Err(R0Error::Engine(e))) => {
            CaseResult::new(case, suite, CaseStatus::EngineError, Some(e))
        }
        Ok(Ok((items, scenarios))) => {
            let status = if items.iter().all(|i| i.verdict.passes()) {
                CaseStatus::Pass
            } else {
                CaseStatus::Fail
            };
            CaseResult {
                checked: scenarios.values().sum(),
                scenarios,
                items,
                ..CaseResult::new(case, suite, status, None)
            }
        }
    };
    result.millis = millis;
    result
}

enum R0Error {
    /// The source package itself cannot be read: the case is unusable.
    Source(String),
    /// The engine refused to load or save it.
    Engine(String),
}

/// Runs R0 and then each further scenario of `scenarios` on one case. Returns the items (R0's,
/// then each scenario's failures) and how many source items each scenario compared.
fn run_scenarios(
    bytes: &[u8],
    name: Option<&str>,
    scenarios: &[String],
) -> Result<(Vec<Item>, BTreeMap<&'static str, usize>), R0Error> {
    let (mut items, checked, out) = r0(bytes, name)?;
    let mut counts = BTreeMap::from([("R0", checked)]);
    let plain = items.clone();
    for scenario in scenarios {
        let Some(label) = crate::suite::ROUNDTRIP_SCENARIOS
            .iter()
            .copied()
            .find(|s| s == scenario)
        else {
            continue;
        };
        if label == "R0/again" {
            // Idempotence (D8): our own R0 output, saved again, keeps every item.
            let (again, checked, _) = r0(&out, name)?;
            counts.insert(label, checked);
            items.extend(
                again
                    .into_iter()
                    .filter(|i| !i.verdict.passes())
                    .map(|i| Item {
                        scenario: label,
                        ..i
                    }),
            );
        } else if let Some(edits) = r1::edits_for(label) {
            let Some((edited, checked)) = run_r1(bytes, name, label, edits, &plain)? else {
                continue;
            };
            counts.insert(label, checked);
            items.extend(edited);
        }
    }
    Ok((items, counts))
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
fn r0(bytes: &[u8], name: Option<&str>) -> Result<(Vec<Item>, usize, Vec<u8>), R0Error> {
    let source = Package::read(bytes).map_err(|e| R0Error::Source(e.to_string()))?;
    let workbook = load(bytes, name)?;
    let out = save(&workbook)?;
    let sides = Sides {
        source: &source,
        source_bytes: bytes,
        saved: &workbook,
        name: name.unwrap_or("workbook"),
    };
    let (items, checked) = score("R0", &sides, &out);
    Ok((items, checked, out))
}

/// One R1 scenario (D1, D5): load, recalculate, apply `edits`, save, score, and keep the delta
/// violations against the plain save's items. `None` when no edit had a target (`n/a`).
fn run_r1(
    bytes: &[u8],
    name: Option<&str>,
    label: &'static str,
    edits: &[r1::Edit],
    plain: &[Item],
) -> Result<Option<(Vec<Item>, usize)>, R0Error> {
    let source = Package::read(bytes).map_err(|e| R0Error::Source(e.to_string()))?;
    let mut workbook = load(bytes, name)?;
    let Some(delta) = r1::apply(&mut workbook, edits).map_err(R0Error::Engine)? else {
        return Ok(None);
    };
    let out = save(&workbook)?;
    let sides = Sides {
        source: &source,
        source_bytes: bytes,
        saved: &workbook,
        name: name.unwrap_or("workbook"),
    };
    let (items, checked) = score(label, &sides, &out);
    Ok(Some((r1::violations(label, plain, items, &delta), checked)))
}

/// What a scenario's output is compared with: the source package and bytes, and the workbook
/// that was saved (its recalculated values, D3.1).
struct Sides<'a> {
    source: &'a Package,
    source_bytes: &'a [u8],
    saved: &'a Workbook,
    name: &'a str,
}

/// The items of one scenario and the number of source items compared: validator problems of
/// `out`, the source's parts with what differs inside them, then the semantic items.
fn score(scenario: &'static str, sides: &Sides<'_>, out: &[u8]) -> (Vec<Item>, usize) {
    let mut items = Vec::new();
    let problem = |p: Problem| Item {
        scenario,
        kind: "package",
        key: p.check.to_string(),
        verdict: Verdict::PackageInvalid,
        detail: Some(p.detail),
        candidate: None,
        weight: 0,
    };
    let output = match Package::read(out) {
        Ok(pkg) => pkg,
        Err(e) => {
            items.push(problem(Problem {
                check: "V8",
                detail: format!("the output is not a readable package: {e}"),
            }));
            return (items, 1);
        }
    };
    items.extend(validate(&output).into_iter().map(problem));
    let (parts, mut checked) = part_items(scenario, sides.source, &output);
    items.extend(parts);
    // V8: our own importer opens what we wrote, as stored (no recalculation).
    let as_loaded = LoadOptions {
        recalc: false,
        ..LoadOptions::default()
    };
    let reloaded = match Workbook::from_xlsx_bytes(out, &as_loaded) {
        Ok(w) => w,
        Err(e) => {
            items.push(problem(Problem {
                check: "V8",
                detail: format!("the output does not load: {e}"),
            }));
            return (items, checked);
        }
    };
    // Semantic items (D2, D3): the source and the output, both as loaded, then the saved and the
    // reloaded values. A source the model cannot load scores no semantic items.
    if let (Ok(src), Ok(dst)) = (
        semantic::load_model(sides.source_bytes, sides.name),
        semantic::load_model(out, sides.name),
    ) {
        items.extend(semantic::validate_indexes(&dst).into_iter().map(problem));
        let (mut src_facts, mut dst_facts) =
            (semantic::model_facts(&src), semantic::model_facts(&dst));
        if let (Ok(a), Ok(b)) = (read_snapshot(sides.source_bytes), read_snapshot(out)) {
            src_facts.extend(semantic::stored_facts(&a));
            dst_facts.extend(semantic::stored_facts(&b));
        }
        let saved = semantic::value_facts(sides.saved, &src_facts);
        let back = semantic::value_facts(&reloaded, &dst_facts);
        for (facts_a, facts_b) in [(&src_facts, &dst_facts), (&saved, &back)] {
            let (diffs, n) = semantic::compare(facts_a, facts_b);
            checked += n;
            items.extend(diffs.into_iter().map(|(key, verdict, detail)| Item {
                scenario,
                kind: "semantic",
                key,
                verdict,
                detail,
                candidate: None,
                weight: xmldiff::unit_weight(verdict),
            }));
        }
    }
    (items, checked)
}

/// One `part` item per content part of `source`, then one `added` item per part only `output`
/// has (D2): parts match by key, never by name. A kept XML part is diffed for `element` and
/// `attribute` items (D4.1–D4.4), another part compared by bytes (D4.5). Returns the items and
/// the number of source items compared.
fn part_items(scenario: &'static str, source: &Package, output: &Package) -> (Vec<Item>, usize) {
    let src = source.keyed_parts();
    let out = output.keyed_parts();
    let mut items = Vec::new();
    let mut checked = 0;
    let item = |kind, key: String, verdict, detail| Item {
        scenario,
        kind,
        candidate: xmldiff::candidate(&key),
        key,
        verdict,
        detail,
        weight: xmldiff::unit_weight(verdict),
    };
    for (key, part) in &src {
        checked += 1;
        // A part lost, retyped or not comparable keeps none of the items a kept part would have
        // counted: its elements and attributes count, and weigh, with it.
        let whole = |verdict, detail| {
            let inside = source
                .xml
                .get(*part)
                .and_then(|xml| xmldiff::element_count(xml).ok())
                .unwrap_or(0);
            let it = Item {
                weight: 1 + inside,
                ..item("part", key.clone(), verdict, detail)
            };
            (it, inside)
        };
        let Some(out_part) = out.get(key) else {
            let (it, inside) = whole(Verdict::Lost, None);
            checked += inside;
            items.push(it);
            continue;
        };
        let (was, now) = (source.content_type(part), output.content_type(out_part));
        if was != now {
            let detail = Some(format!("content type {was:?} became {now:?}"));
            let (it, inside) = whole(Verdict::Changed, detail);
            checked += inside;
            items.push(it);
            continue;
        }
        let (mut verdict, mut detail) = (Verdict::Kept, None);
        match (source.xml.get(*part), output.xml.get(*out_part)) {
            (Some(src_xml), Some(out_xml)) => match xmldiff::diff(src_xml, out_xml) {
                Ok(part_diff) => {
                    checked += part_diff.checked;
                    for diff in part_diff.differences {
                        items.push(Item {
                            weight: diff.weight,
                            ..item(
                                diff.kind,
                                format!("{key}:{}", diff.path),
                                diff.verdict,
                                diff.detail,
                            )
                        });
                    }
                    if part_diff.truncated > 0 {
                        let more = Some(format!("{} more differences", part_diff.truncated));
                        items.push(Item {
                            weight: part_diff.truncated_weight,
                            ..item(
                                "element",
                                format!("{key}:<truncated>"),
                                Verdict::Changed,
                                more,
                            )
                        });
                    }
                }
                Err(why) => {
                    let (it, inside) =
                        whole(Verdict::Changed, Some(format!("not comparable: {why}")));
                    checked += inside;
                    items.push(it);
                    continue;
                }
            },
            _ => {
                if source.digests.get(*part) != output.digests.get(*out_part) {
                    (verdict, detail) = (Verdict::Changed, Some("the bytes differ".to_string()));
                }
            }
        }
        items.push(item("part", key.clone(), verdict, detail));
    }
    for key in out.keys().filter(|k| !src.contains_key(*k)) {
        items.push(item("part", key.clone(), Verdict::Added, None));
    }
    (items, checked)
}

/// The share of a run's source items that were kept (SPEC-0033 D2): the source items compared,
/// less those the failing items cover ([`Item::weight`]: a lost subtree counts every element and
/// attribute in it), over the items compared. `added` items and validator failures cover no source item, so
/// the baseline gates them but they do not lower the rate. A case that did not run compares
/// nothing, so it is counted in `unscored`, never as kept. M15's goal is stated as this figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct KeptRate {
    /// Source items compared.
    pub checked: usize,
    /// Source items covered by `changed` or `lost` items.
    pub not_kept: usize,
    /// Cases that did not run (`!Status`), whose items are not in either count.
    pub unscored: usize,
}

impl KeptRate {
    /// The rate over `results`.
    #[must_use]
    pub fn of(results: &[&CaseResult]) -> KeptRate {
        let mut rate = KeptRate::default();
        for r in results {
            if !matches!(r.status, CaseStatus::Pass | CaseStatus::Fail) {
                rate.unscored += 1;
                continue;
            }
            rate.checked += r.checked;
            rate.not_kept += r.items.iter().map(|i| i.weight).sum::<usize>();
        }
        rate
    }

    /// The rate of the scenarios `which` selects (`R0`, the `R1/` edits, `R0/again`): the items
    /// those scenarios compared, less the weight of their failing items.
    #[must_use]
    pub fn of_scenarios(results: &[&CaseResult], which: impl Fn(&str) -> bool) -> KeptRate {
        let mut rate = KeptRate::default();
        for r in results {
            if !matches!(r.status, CaseStatus::Pass | CaseStatus::Fail) {
                rate.unscored += 1;
                continue;
            }
            rate.checked += r
                .scenarios
                .iter()
                .filter(|(s, _)| which(s))
                .map(|(_, n)| n)
                .sum::<usize>();
            rate.not_kept += r
                .items
                .iter()
                .filter(|i| which(i.scenario))
                .map(|i| i.weight)
                .sum::<usize>();
        }
        rate
    }

    /// The kept share in percent; 100 when nothing was compared.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // item counts stay far below 2^52
    pub fn percent(self) -> f64 {
        if self.checked == 0 {
            return 100.0;
        }
        100.0 * (self.checked.saturating_sub(self.not_kept)) as f64 / self.checked as f64
    }
}

/// A scenario family as the rates line reports it: its label and which scenarios it holds.
type Family = (&'static str, fn(&str) -> bool);

/// Each scenario family's kept rate, in run order: `R0`, the `R1/` edits together (their delta
/// violations over the items they compared), and `R0/again` (idempotence), when the cases ran it.
fn rates_line(results: &[&CaseResult]) -> String {
    let families: [Family; 3] = [
        ("R0", |s| s == "R0"),
        ("R1", |s| s.starts_with("R1/")),
        ("R0/again", |s| s == "R0/again"),
    ];
    families
        .iter()
        .filter(|(_, which)| results.iter().any(|r| r.scenarios.keys().any(|s| which(s))))
        .map(|(label, which)| {
            format!(
                "{label} {:.2}% kept",
                KeptRate::of_scenarios(results, which).percent()
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The results as a baseline (SPEC-0033 D8). A case that did not run is one `!Status` failure,
/// as in the cell suites.
///
/// - `cells`: a case's checked count is the number of source items it compared, and each failing
///   item is one known failure.
/// - `cases`, for the corpora, whose item lists would make a baseline of tens of megabytes: a
///   failing case is one line, `* <n> <digest>`. n weighs its failing items as [`KeptRate`] does,
///   with at least 1 for each, and the digest names their set of keys; see [`diff`].
#[must_use]
pub fn baseline(results: &[&CaseResult], mode: BaselineMode) -> Baseline {
    let mut b = Baseline::default();
    if mode == BaselineMode::None {
        return b;
    }
    for r in results {
        b.checked.insert(r.id.clone(), r.checked);
        match r.status {
            CaseStatus::Pass => {}
            CaseStatus::Fail => {
                let failing = r.items.iter().filter(|i| !i.verdict.passes());
                if mode == BaselineMode::Cases {
                    b.failing.insert((r.id.clone(), case_line(failing)));
                } else {
                    for i in failing {
                        b.failing.insert((r.id.clone(), i.baseline_key()));
                    }
                }
            }
            other => {
                b.failing.insert((r.id.clone(), format!("!{other:?}")));
            }
        }
    }
    b
}

/// The item of a failing case in a `cases`-mode round-trip baseline, before its count.
const CASE_ITEM: &str = "*";

/// `* <n> <digest>` for a failing case's items: their weight, at least 1 each, and the first 16
/// hex digits of the SHA-256 of their sorted baseline keys.
fn case_line<'a>(failing: impl Iterator<Item = &'a Item>) -> String {
    let mut n = 0;
    let mut keys = Vec::new();
    for i in failing {
        n += i.weight.max(1);
        keys.push(i.baseline_key());
    }
    keys.sort_unstable();
    let mut hash = Sha256::new();
    for k in &keys {
        hash.update(k.as_bytes());
        hash.update(b"\n");
    }
    let digest = crate::fetch::hex(&hash.finalize()[..8]);
    format!("{CASE_ITEM} {n} {digest}")
}

/// The ratchet's delta between two round-trip baselines: [`ratchet::diff`] with three changes.
/// - In `cases` mode a changed line whose n fell is a fix, whatever its digest. A line whose n
///   held or rose is a regression: a different digest with the same n is a loss traded for a
///   fix. A fix that outweighs a new loss passes; the cells-mode fixtures suite names each item.
/// - A case whose checked count fell is a regression, since a round trip's count includes what
///   the output let it compare: an unreadable output compares one item.
/// - A case that newly did not run (`!Status`) is a regression whatever its count.
#[must_use]
pub fn diff(old: &Baseline, new: &Baseline) -> ratchet::Delta {
    let mut delta = ratchet::diff(old, new);
    let items = |b: &'_ Baseline, case: &str| -> Vec<String> {
        b.failing
            .range((case.to_string(), String::new())..)
            .take_while(|(c, _)| c == case)
            .map(|(_, item)| item.clone())
            .collect()
    };
    let count = |b: &Baseline, case: &str| {
        items(b, case).iter().find_map(|item| {
            let rest = item.strip_prefix(CASE_ITEM)?.trim_start();
            rest.split(' ').next()?.parse::<usize>().ok()
        })
    };
    // The old line of a case whose count fell is already in `fixed` (old minus new).
    delta.regressions.retain(|line| {
        let Some((case, _)) = line.split_once('\t') else {
            return true;
        };
        !matches!((count(old, case), count(new, case)), (Some(a), Some(b)) if b < a)
    });
    for (case, &now) in &new.checked {
        let Some(&was) = old.checked.get(case) else {
            continue;
        };
        let line = |what: String| format!("{case}\t{what}");
        if now < was {
            delta
                .regressions
                .push(line(format!("checked fell from {was} to {now}")));
        }
        let not_run = |b: &Baseline| items(b, case).into_iter().find(|i| i.starts_with('!'));
        if let (None, Some(status)) = (not_run(old), not_run(new)) {
            let it = line(status);
            if !delta.regressions.contains(&it) {
                delta.regressions.push(it);
            }
        }
    }
    delta
}

/// The Markdown summary of a run: per suite, case statuses, items by verdict, the most often lost
/// part kinds, and every validator failure.
#[must_use]
pub fn report_markdown(results: &[CaseResult]) -> String {
    let mut by_suite: BTreeMap<&str, Vec<&CaseResult>> = BTreeMap::new();
    for r in results {
        by_suite.entry(&r.suite).or_default().push(r);
    }
    let mut md = String::from("# Round-trip fidelity (SPEC-0033)\n\n");
    for (suite, cases) in by_suite {
        let mut status: BTreeMap<String, usize> = BTreeMap::new();
        let mut verdicts: BTreeMap<&str, usize> = BTreeMap::new();
        let mut lost: BTreeMap<String, usize> = BTreeMap::new();
        let mut invalid = Vec::new();
        for c in &cases {
            *status.entry(format!("{:?}", c.status)).or_default() += 1;
            for i in &c.items {
                *verdicts.entry(i.verdict.label()).or_default() += 1;
                if i.verdict == Verdict::Lost {
                    let kind = i.key.rsplit('/').next().unwrap_or(&i.key);
                    let kind = kind.split('[').next().unwrap_or(kind).to_string();
                    *lost.entry(kind).or_default() += 1;
                }
                if i.verdict == Verdict::PackageInvalid {
                    invalid.push(format!(
                        "{} {}: {}",
                        c.id,
                        i.key,
                        i.detail.as_deref().unwrap_or("")
                    ));
                }
            }
        }
        let rate = KeptRate::of(&cases);
        let _ = writeln!(
            md,
            "## {suite}\n\n{} cases: {status:?}\n\n{} ({} source items compared over all scenarios, {} not kept; {} cases not scored)\n\nItems by verdict: {verdicts:?}\n",
            cases.len(),
            rates_line(&cases),
            rate.checked,
            rate.not_kept,
            rate.unscored
        );
        let mut lost: Vec<_> = lost.into_iter().collect();
        lost.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        if !lost.is_empty() {
            md.push_str("| lost part kind | parts |\n|---|---|\n");
            for (kind, n) in lost.iter().take(20) {
                let _ = writeln!(md, "| {kind} | {n} |");
            }
            md.push('\n');
        }
        for line in invalid.iter().take(50) {
            let _ = writeln!(md, "- package-invalid {line}");
        }
        if !invalid.is_empty() {
            md.push('\n');
        }
    }
    md
}

/// The mode flags of `rustsheet-eval roundtrip`.
#[derive(clap::Args, Debug, Clone, Copy)]
pub struct Flags {
    /// Fail when a suite's baseline is out of date or regressed.
    #[arg(long, conflicts_with = "update")]
    pub check: bool,
    /// Rewrite the baselines (ratchet), refusing regressions.
    #[arg(long = "update-baseline")]
    pub update: bool,
    /// With `--update-baseline`: record regressions too. Only under a decision record (CLAUDE.md
    /// directive 2, SPEC-0033 "Ratchet"; decision 0328).
    #[arg(long, requires = "update")]
    pub allow_regressions: bool,
}

impl Flags {
    /// The [`Mode`] these flags ask for.
    #[must_use]
    pub fn mode(self) -> Mode {
        match (self.check, self.update) {
            (true, _) => Mode::Check,
            (_, true) => Mode::Update {
                allow_regressions: self.allow_regressions,
            },
            _ => Mode::Report,
        }
    }
}

/// What `rustsheet-eval roundtrip` does with a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Write the report only.
    Report,
    /// Also fail when a suite's baseline is out of date or regressed.
    Check,
    /// Also rewrite the baselines, refusing regressions unless `allow_regressions` (only under a
    /// decision record, SPEC-0033 "Ratchet").
    Update {
        /// Record the run even when the delta has regressions (`--allow-regressions`).
        allow_regressions: bool,
    },
}

/// Runs `suites`, writes `eval/runs/latest/roundtrip.{json,md}` under `root`, and in `Check` or
/// `Update` mode compares each suite with `eval/baseline/<suite>.txt` as the cell suites do.
/// A `--filter` run never touches a baseline. Returns whether the gate passed, and the lines to
/// print.
///
/// # Errors
/// When the report or a baseline cannot be read or written.
pub fn command(
    root: &Path,
    suites: &[Suite],
    filter: Option<&str>,
    timeout: Duration,
    mode: Mode,
) -> anyhow::Result<(bool, Vec<String>)> {
    if filter.is_some() && mode != Mode::Report {
        anyhow::bail!(
            "--filter is not allowed for baseline operations (partial runs look like regressions)"
        );
    }
    let mut lines = Vec::new();
    // A suite over another suite's fetched cache (calipers, poi, corpus-bench) is skipped while
    // that cache is absent, so a run never records an empty baseline over a real one. A gated
    // suite's cases must be there.
    let (suites, missing): (Vec<&Suite>, Vec<&Suite>) =
        suites.iter().partition(|s| s.root().is_dir());
    for s in missing {
        if s.manifest.gate && mode != Mode::Report {
            anyhow::bail!("{}: no cases at {}", s.manifest.name, s.root().display());
        }
        lines.push(format!(
            "skipping {}: {} is not there (fetch the suite it reads)",
            s.manifest.name,
            s.root().display()
        ));
    }
    let mut results = run_suites(&suites, filter, timeout);
    let mut ok = true;
    for suite in &suites {
        let excused = exceptions::load(suite)?;
        let mut mine: Vec<&mut CaseResult> = results
            .iter_mut()
            .filter(|r| r.suite == suite.manifest.name)
            .collect();
        for stale in exceptions::apply(&mut mine, &excused) {
            if mode != Mode::Report && filter.is_none() {
                ok = false;
            }
            lines.push(format!(
                "  STALE EXCEPTION {} `{}` ({}): it excuses nothing; remove it",
                stale.case, stale.item, stale.owner
            ));
        }
    }
    let out = root.join("eval/runs/latest");
    std::fs::create_dir_all(&out)?;
    std::fs::write(
        out.join("roundtrip.json"),
        serde_json::to_string_pretty(&results)?,
    )?;
    std::fs::write(out.join("roundtrip.md"), report_markdown(&results))?;
    for suite in suites {
        let name = &suite.manifest.name;
        let mine: Vec<&CaseResult> = results.iter().filter(|r| &r.suite == name).collect();
        let rate = KeptRate::of(&mine);
        let current = baseline(&mine, suite.manifest.baseline);
        lines.push(format!(
            "[{name}] {} cases ({} not scored), {} items, {}, {} failing",
            mine.len(),
            rate.unscored,
            rate.checked,
            rates_line(&mine),
            current.failing.len()
        ));
        if mode == Mode::Report {
            continue;
        }
        let path = root.join("eval/baseline").join(format!("{name}.txt"));
        let old = Baseline::read(&path)?;
        let delta = diff(&old, &current);
        lines.extend(
            delta
                .regressions
                .iter()
                .take(25)
                .map(|r| format!("  REGRESSION {r}")),
        );
        match mode {
            Mode::Check if delta.is_dirty() => {
                ok = false;
                if delta.regressions.is_empty() {
                    lines.push("  baseline is out of date: run `make eval-baseline` and commit eval/baseline/".into());
                }
            }
            Mode::Update {
                allow_regressions: false,
            } if !delta.regressions.is_empty() => {
                ok = false;
                lines.push("  refusing to record regressions".into());
            }
            // A case gone from a corpus (the `cases` suites) is a cache fetched under an older include, not a
            // fix: refetch the suite (`rustsheet-eval fetch --suite <cell suite>`).
            Mode::Update { .. }
                if suite.manifest.baseline == BaselineMode::Cases
                    && !delta.removed_cases.is_empty() =>
            {
                ok = false;
                lines.extend(
                    delta
                        .removed_cases
                        .iter()
                        .take(25)
                        .map(|c| format!("  REMOVED {c}")),
                );
                lines.push("  refusing to drop cases: refetch the suite they come from".into());
            }
            Mode::Update { .. } => current.write(&path)?,
            _ => {}
        }
    }
    lines.push(format!("report: {}", out.join("roundtrip.md").display()));
    Ok((ok, lines))
}
