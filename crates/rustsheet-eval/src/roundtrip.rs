//! Round-trip suites (SPEC-0033): each workbook is loaded and saved again, the output is checked
//! by the package validator (D6), and the source's content is scored as fidelity items (D2) that
//! ratchet through the ordinary cells-mode [`Baseline`].
//!
//! This first slice (T-4172) runs scenario R0 and scores `part` items: present, and with the
//! same content type. Element and attribute items (T-4509), semantic items (T-4513) and R1
//! (T-4514) build on the same results.

mod package;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use rustsheet::{LoadOptions, SaveOptions, Workbook};
use serde::Serialize;

use crate::baseline::{self as ratchet, Baseline};
use crate::engine::harness_options;
use crate::runner::{panic_message, CaseStatus};
use crate::suite::{CaseFile, Suite};

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

    fn label(self) -> &'static str {
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
        // calcChain is the first candidate normalization (SPEC-0033 D4.4, N1): tagged until
        // T-4503's evidence lets the differ normalize it.
        if self.kind == "part" && self.key.ends_with("/calcChain[0]") {
            key.push_str(" candidate-N1");
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
            items: Vec::new(),
        }
    }
}

/// Runs every case of `suites` in parallel, each isolated against panics and `timeout`.
#[must_use]
pub fn run_suites(suites: &[Suite], filter: Option<&str>, timeout: Duration) -> Vec<CaseResult> {
    let mut jobs = Vec::new();
    for suite in suites {
        match suite.cases() {
            Ok(cases) => jobs.extend(cases.into_iter().map(|c| (suite, c))),
            Err(e) => {
                let case = CaseFile {
                    id: format!("{}/<discovery>", suite.manifest.name),
                    path: suite.root(),
                    sidecar: std::path::PathBuf::new(),
                };
                jobs.push((suite, case));
                eprintln!("{}: case discovery failed: {e}", suite.manifest.name);
            }
        }
    }
    if let Some(f) = filter {
        jobs.retain(|(_, c)| c.id.contains(f));
    }
    jobs.par_iter()
        .map(|(suite, case)| run_case(suite, case, timeout))
        .collect()
}

/// Runs one case: R0 in a worker thread, so a panic or a hang is a verdict, not a crash.
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
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    let spawned = std::thread::Builder::new()
        .name(format!("roundtrip:{}", case.id))
        .stack_size(256 * 1024 * 1024) // the evaluator recurses on deep dependency chains
        .spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(|| r0(&bytes, name.as_deref())));
            let _ = tx.send(outcome);
        });
    if let Err(e) = spawned {
        return CaseResult::new(
            case,
            suite,
            CaseStatus::EngineError,
            Some(format!("could not spawn worker: {e}")),
        );
    }
    let received = rx.recv_timeout(timeout);
    let millis = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let mut result = match received {
        Err(_) => CaseResult::new(
            case,
            suite,
            CaseStatus::Timeout,
            Some(format!("exceeded {}s", timeout.as_secs())),
        ),
        Ok(Err(panic)) => CaseResult::new(
            case,
            suite,
            CaseStatus::Panic,
            Some(panic_message(panic.as_ref())),
        ),
        Ok(Ok(Err(R0Error::Source(e)))) => {
            CaseResult::new(case, suite, CaseStatus::InvalidCase, Some(e))
        }
        Ok(Ok(Err(R0Error::Engine(e)))) => {
            CaseResult::new(case, suite, CaseStatus::EngineError, Some(e))
        }
        Ok(Ok(Ok(items))) => {
            let status = if items.iter().all(|i| i.verdict.passes()) {
                CaseStatus::Pass
            } else {
                CaseStatus::Fail
            };
            CaseResult {
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

/// Scenario R0 (SPEC-0033 D1): load with the harness clock, recalculate, save; then validate the
/// output and score the source's parts.
fn r0(bytes: &[u8], name: Option<&str>) -> Result<Vec<Item>, R0Error> {
    let source = Package::read(bytes).map_err(|e| R0Error::Source(e.to_string()))?;
    let opts = LoadOptions {
        recalc: false,
        workbook: harness_options(name).map_err(R0Error::Engine)?,
        ..LoadOptions::default()
    };
    let mut workbook = Workbook::from_xlsx_bytes(bytes, &opts)
        .map_err(|e| R0Error::Engine(format!("import failed: {e}")))?;
    workbook.pin_clock_from_metadata();
    workbook.recalculate();
    let out = workbook
        .to_xlsx_bytes(SaveOptions::default())
        .map_err(|e| R0Error::Engine(format!("save failed: {e}")))?;
    Ok(score("R0", &source, &out))
}

/// The items of one scenario: validator problems of `out`, then the source's parts.
fn score(scenario: &'static str, source: &Package, out: &[u8]) -> Vec<Item> {
    let mut items = Vec::new();
    let problem = |p: Problem| Item {
        scenario,
        kind: "package",
        key: p.check.to_string(),
        verdict: Verdict::PackageInvalid,
        detail: Some(p.detail),
    };
    let output = match Package::read(out) {
        Ok(pkg) => pkg,
        Err(e) => {
            items.push(problem(Problem {
                check: "V8",
                detail: format!("the output is not a readable package: {e}"),
            }));
            return items;
        }
    };
    items.extend(validate(&output).into_iter().map(problem));
    // V8: our own importer opens what we wrote.
    if let Err(e) = Workbook::from_xlsx_bytes(out, &LoadOptions::default()) {
        items.push(problem(Problem {
            check: "V8",
            detail: format!("the output does not load: {e}"),
        }));
    }
    items.extend(part_items(scenario, source, &output));
    items
}

/// One `part` item per content part of `source`, then one `added` item per part only `output`
/// has (D2): parts match by key, never by name.
fn part_items(scenario: &'static str, source: &Package, output: &Package) -> Vec<Item> {
    let src = source.keyed_parts();
    let out = output.keyed_parts();
    let mut items = Vec::new();
    for (key, part) in &src {
        let (verdict, detail) = match out.get(key) {
            None => (Verdict::Lost, None),
            Some(o) => {
                let (a, b) = (source.content_type(part), output.content_type(o));
                if a == b {
                    (Verdict::Kept, None)
                } else {
                    (
                        Verdict::Changed,
                        Some(format!("content type {a:?} became {b:?}")),
                    )
                }
            }
        };
        items.push(Item {
            scenario,
            kind: "part",
            key: key.clone(),
            verdict,
            detail,
        });
    }
    for key in out.keys().filter(|k| !src.contains_key(*k)) {
        items.push(Item {
            scenario,
            kind: "part",
            key: key.clone(),
            verdict: Verdict::Added,
            detail: None,
        });
    }
    items
}

/// The results as a cells-mode baseline (SPEC-0033 D8): a case's checked count is its number of
/// items, and each failing item is one known failure. A case that did not run is one
/// `!Status` failure, as in the cell suites.
#[must_use]
pub fn baseline(results: &[&CaseResult]) -> Baseline {
    let mut b = Baseline::default();
    for r in results {
        b.checked.insert(r.id.clone(), r.items.len());
        match r.status {
            CaseStatus::Pass => {}
            CaseStatus::Fail => {
                for i in r.items.iter().filter(|i| !i.verdict.passes()) {
                    b.failing.insert((r.id.clone(), i.baseline_key()));
                }
            }
            other => {
                b.failing.insert((r.id.clone(), format!("!{other:?}")));
            }
        }
    }
    b
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
        let _ = writeln!(
            md,
            "## {suite}\n\n{} cases: {status:?}\n\nItems by verdict: {verdicts:?}\n",
            cases.len()
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

/// What `rustsheet-eval roundtrip` does with a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Write the report only.
    Report,
    /// Also fail when a suite's baseline is out of date or regressed.
    Check,
    /// Also rewrite the baselines, refusing regressions.
    Update,
}

/// Runs `suites`, writes `eval/runs/latest/roundtrip.{json,md}` under `root`, and in `Check` or
/// `Update` mode compares each suite with `eval/baseline/<suite>.txt` as the cell suites do.
/// A `--filter` run never touches a baseline. Returns whether the gate passed.
///
/// # Errors
/// When the report or a baseline cannot be read or written.
pub fn command(
    root: &Path,
    suites: &[Suite],
    filter: Option<&str>,
    timeout: Duration,
    mode: Mode,
) -> anyhow::Result<bool> {
    if filter.is_some() && mode != Mode::Report {
        anyhow::bail!(
            "--filter is not allowed for baseline operations (partial runs look like regressions)"
        );
    }
    let results = run_suites(suites, filter, timeout);
    let out = root.join("eval/runs/latest");
    std::fs::create_dir_all(&out)?;
    std::fs::write(
        out.join("roundtrip.json"),
        serde_json::to_string_pretty(&results)?,
    )?;
    std::fs::write(out.join("roundtrip.md"), report_markdown(&results))?;
    let mut ok = true;
    for suite in suites {
        let name = &suite.manifest.name;
        let mine: Vec<&CaseResult> = results.iter().filter(|r| &r.suite == name).collect();
        let items: usize = mine.iter().map(|r| r.items.len()).sum();
        let current = baseline(&mine);
        println!(
            "[{name}] {} cases, {items} items, {} failing",
            mine.len(),
            current.failing.len()
        );
        if mode == Mode::Report {
            continue;
        }
        let path = root.join("eval/baseline").join(format!("{name}.txt"));
        let old = Baseline::read(&path)?;
        let delta = ratchet::diff(&old, &current);
        for r in delta.regressions.iter().take(25) {
            println!("  REGRESSION {r}");
        }
        match mode {
            Mode::Check if delta.is_dirty() => {
                ok = false;
                if delta.regressions.is_empty() {
                    println!("  baseline is out of date: run `rustsheet-eval roundtrip --update-baseline`");
                }
            }
            Mode::Update if !delta.regressions.is_empty() => {
                ok = false;
                println!("  refusing to record regressions");
            }
            Mode::Update => current.write(&path)?,
            _ => {}
        }
    }
    println!("report: {}", out.join("roundtrip.md").display());
    Ok(ok)
}
