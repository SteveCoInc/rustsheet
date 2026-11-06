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
mod xmldiff;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read as _;
use std::path::Path;
use std::time::Duration;

use rayon::prelude::*;
use rustsheet::{LoadOptions, SaveOptions, Workbook};
use serde::Serialize;

use crate::baseline::{self as ratchet, Baseline};
use crate::engine::harness_options;
use crate::runner::{run_in_worker, CaseStatus};
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
    /// The candidate normalization (SPEC-0033 D4.4, `N1`–`N12`) the item would fall under once
    /// T-4503 evidences it: tagged, never applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<&'static str>,
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
    /// Source items compared: parts, elements and attributes (the baseline's checked count).
    pub checked: usize,
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
            items: Vec::new(),
        }
    }
}

/// Runs every case of `suites` in parallel, each isolated against panics and `timeout`.
#[must_use]
pub fn run_suites(suites: &[Suite], filter: Option<&str>, timeout: Duration) -> Vec<CaseResult> {
    let mut jobs = Vec::new();
    let mut failed = Vec::new();
    for suite in suites {
        match suite.cases() {
            Ok(cases) => jobs.extend(cases.into_iter().map(|c| (suite, c))),
            Err(e) => failed.push(CaseResult {
                id: format!("{}/<discovery>", suite.manifest.name),
                suite: suite.manifest.name.clone(),
                status: CaseStatus::InvalidCase,
                message: Some(e.to_string()),
                millis: 0,
                checked: 0,
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
    let (outcome, millis) = run_in_worker(format!("roundtrip:{}", case.id), timeout, move || {
        r0(&bytes, name.as_deref())
    });
    let mut result = match outcome {
        Err((status, message)) => CaseResult::new(case, suite, status, Some(message)),
        Ok(Err(R0Error::Source(e))) => {
            CaseResult::new(case, suite, CaseStatus::InvalidCase, Some(e))
        }
        Ok(Err(R0Error::Engine(e))) => {
            CaseResult::new(case, suite, CaseStatus::EngineError, Some(e))
        }
        Ok(Ok((items, checked))) => {
            let status = if items.iter().all(|i| i.verdict.passes()) {
                CaseStatus::Pass
            } else {
                CaseStatus::Fail
            };
            CaseResult {
                checked,
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
/// output and score the source's parts, elements and attributes.
fn r0(bytes: &[u8], name: Option<&str>) -> Result<(Vec<Item>, usize), R0Error> {
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

/// The items of one scenario and the number of source items compared: validator problems of
/// `out`, then the source's parts with what differs inside them.
fn score(scenario: &'static str, source: &Package, out: &[u8]) -> (Vec<Item>, usize) {
    let mut items = Vec::new();
    let problem = |p: Problem| Item {
        scenario,
        kind: "package",
        key: p.check.to_string(),
        verdict: Verdict::PackageInvalid,
        detail: Some(p.detail),
        candidate: None,
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
    // V8: our own importer opens what we wrote.
    if let Err(e) = Workbook::from_xlsx_bytes(out, &LoadOptions::default()) {
        items.push(problem(Problem {
            check: "V8",
            detail: format!("the output does not load: {e}"),
        }));
    }
    let (parts, checked) = part_items(scenario, source, &output);
    items.extend(parts);
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
    };
    for (key, part) in &src {
        checked += 1;
        let Some(out_part) = out.get(key) else {
            items.push(item("part", key.clone(), Verdict::Lost, None));
            continue;
        };
        let (was, now) = (source.content_type(part), output.content_type(out_part));
        if was != now {
            let detail = Some(format!("content type {was:?} became {now:?}"));
            items.push(item("part", key.clone(), Verdict::Changed, detail));
            continue;
        }
        let (mut verdict, mut detail) = (Verdict::Kept, None);
        match (source.xml.get(*part), output.xml.get(*out_part)) {
            (Some(src_xml), Some(out_xml)) => match xmldiff::diff(src_xml, out_xml) {
                Ok(part_diff) => {
                    checked += part_diff.checked;
                    for diff in part_diff.differences {
                        let verdict = match diff.verdict {
                            "lost" => Verdict::Lost,
                            "added" => Verdict::Added,
                            _ => Verdict::Changed,
                        };
                        items.push(item(
                            diff.kind,
                            format!("{key}:{}", diff.path),
                            verdict,
                            diff.detail,
                        ));
                    }
                    if part_diff.truncated > 0 {
                        let more = Some(format!("{} more differences", part_diff.truncated));
                        items.push(item(
                            "element",
                            format!("{key}:<truncated>"),
                            Verdict::Changed,
                            more,
                        ));
                    }
                }
                Err(why) => {
                    (verdict, detail) = (Verdict::Changed, Some(format!("not comparable: {why}")));
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

/// The results as a cells-mode baseline (SPEC-0033 D8): a case's checked count is the number of
/// source items it compared, and each failing item is one known failure. A case that did not run is one
/// `!Status` failure, as in the cell suites.
#[must_use]
pub fn baseline(results: &[&CaseResult]) -> Baseline {
    let mut b = Baseline::default();
    for r in results {
        b.checked.insert(r.id.clone(), r.checked);
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
    let results = run_suites(suites, filter, timeout);
    let out = root.join("eval/runs/latest");
    std::fs::create_dir_all(&out)?;
    std::fs::write(
        out.join("roundtrip.json"),
        serde_json::to_string_pretty(&results)?,
    )?;
    std::fs::write(out.join("roundtrip.md"), report_markdown(&results))?;
    let mut ok = true;
    let mut lines = Vec::new();
    for suite in suites {
        let name = &suite.manifest.name;
        let mine: Vec<&CaseResult> = results.iter().filter(|r| &r.suite == name).collect();
        let items: usize = mine.iter().map(|r| r.checked).sum();
        let current = baseline(&mine);
        lines.push(format!(
            "[{name}] {} cases, {items} items, {} failing",
            mine.len(),
            current.failing.len()
        ));
        if mode == Mode::Report {
            continue;
        }
        let path = root.join("eval/baseline").join(format!("{name}.txt"));
        let old = Baseline::read(&path)?;
        let delta = ratchet::diff(&old, &current);
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
            Mode::Update if !delta.regressions.is_empty() => {
                ok = false;
                lines.push("  refusing to record regressions".into());
            }
            Mode::Update => current.write(&path)?,
            _ => {}
        }
    }
    lines.push(format!("report: {}", out.join("roundtrip.md").display()));
    Ok((ok, lines))
}
