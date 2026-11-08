//! `rustsheet-eval` — the Excel-compatibility harness CLI.
//!
//! ```text
//! rustsheet-eval suites                         # list suites
//! rustsheet-eval run [--suite S] [--filter X]   # run + write eval/runs/latest/{report.json,report.md}
//! rustsheet-eval check                          # gate: fail on any baseline, visible-parity or tier regression
//! rustsheet-eval check --evidence-floor         # the gate and the evidence floor from one run
//! rustsheet-eval update-baseline                # ratchet baselines, visible parity and tiers (refuses regressions)
//! rustsheet-eval evidence-floor [--update]      # ratchet functions below the evidence floor
//! rustsheet-eval show <case-id> [--all]         # per-cell detail for one case
//! rustsheet-eval snapshot <file.xlsx>           # dump stored formulas/values as JSON
//! rustsheet-eval cases <suite>                  # cases + sidecar paths as JSON (for tools)
//! rustsheet-eval export <case> --out <xlsx>     # the package an engine-export case saves
//! rustsheet-eval accuracy [--json PATH]         # Gate A: reference tables within the envelope
//! ```
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};

use rustsheet_eval::accuracy;
use rustsheet_eval::accuracy::lists::{load_r3_marks, R3Marks};
use rustsheet_eval::baseline::{self, Baseline};
use rustsheet_eval::calibrate::compare_snapshots;
use rustsheet_eval::catalog::Catalog;
use rustsheet_eval::catalog_coverage;
use rustsheet_eval::deviations;
use rustsheet_eval::engine::{Engine, RustSheetEngine};
use rustsheet_eval::evidence_floor::{self, Lock, EVIDENCE_FLOOR, LOCK_PATH};
use rustsheet_eval::exit_exceptions::ExitExceptions;
use rustsheet_eval::fetch::fetch_suite;
use rustsheet_eval::gate_tiers;
use rustsheet_eval::lift_kinds;
use rustsheet_eval::milestone_exit;
use rustsheet_eval::milestone_lists::{self, MilestoneList};
use rustsheet_eval::phase1_exceptions::Phase1Exceptions;
use rustsheet_eval::pivot_features;
use rustsheet_eval::probe_milestones::{self, Mapping, Rules};
use rustsheet_eval::r1c::{mark_r3_math, R1cCells};
use rustsheet_eval::report::RunReport;
use rustsheet_eval::roundtrip;
use rustsheet_eval::run_meta::RunMeta;
use rustsheet_eval::runner::{run_suites, CaseResult, RunOptions};
use rustsheet_eval::select::{to_fetch, Selection};
use rustsheet_eval::suite::{BaselineMode, FetchState, Suite, ACCURACY_DIR};
use rustsheet_eval::text_expect;
use rustsheet_eval::v15_checkpoint;
use rustsheet_eval::visible_parity::{self, VisibleCount};
use rustsheet_eval::xlsx_snapshot::{read_snapshot, WorkbookSnapshot};

#[derive(Parser)]
#[command(
    name = "rustsheet-eval",
    about = "RustSheet Excel-compatibility harness"
)]
struct Cli {
    /// Repository root (defaults to the nearest ancestor containing `eval/suites`).
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List suites and case counts.
    Suites,
    /// Download pinned external suites (`[fetch]` in suite.toml) into eval/.cache/suites.
    Fetch {
        /// Suites to fetch (repeatable). Default: every external suite.
        #[arg(long = "suite")]
        suites: Vec<String>,
        /// Only suites whose cache is missing or stale; a current cache is left alone.
        #[arg(long)]
        missing: bool,
        /// Only gate suites (`gate = true`); never calipers (SPEC-0035 D3).
        #[arg(long)]
        gates: bool,
    },
    /// Run suites and write the report to eval/runs/latest.
    Run {
        #[command(flatten)]
        sel: Selection,
        /// Output directory (default: eval/runs/latest).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Failing examples shown per function in report.md.
        #[arg(long, default_value_t = 3)]
        examples: usize,
        /// Also leave out of the Phase 1 rate the cells of eval/runs/desktop-owned.tsv, which
        /// `scripts/desktop_owned.py` writes (decision 0143; `make eval` passes it).
        #[arg(long)]
        desktop_owned: bool,
    },
    /// Fail if anything in the committed baselines no longer passes.
    Check {
        #[command(flatten)]
        sel: Selection,
        /// Also judge the evidence floor (as `evidence-floor` does) from the same run: the other
        /// vendored suites run beside the gate suites, once (T-2761). `--timeout` applies to the
        /// whole run; standalone `evidence-floor` uses the 60 s default, as `make eval-gates` does.
        #[arg(long)]
        evidence_floor: bool,
    },
    /// Rewrite baselines to the current passing set (ratchet).
    UpdateBaseline {
        #[command(flatten)]
        sel: Selection,
        /// Permit removing previously passing items (requires PR justification).
        #[arg(long)]
        allow_regressions: bool,
        /// Also lower the visible-parity lock's cases to their counts and drop their lines at 0.
        /// The steward's daily run only: a PR leaves drops to it (decisions 0093 and 0219).
        #[arg(long)]
        lower_visible_lock: bool,
        /// A file listing the cases to treat as new for the visible-parity lock, one case id or
        /// repository-relative case file path per line (the case files a PR adds); only these
        /// then get a line recorded whatever their count (decision 0219).
        #[arg(long, value_name = "FILE")]
        new_cases: Option<PathBuf>,
    },
    /// Fail if more in-scope functions are below the evidence floor than
    /// quality/evidence-floor.json allows, or if the lock is stale (T-0056, decision 0062).
    /// Counts the vendored suites only, so a fetched external suite cannot move it.
    EvidenceFloor {
        /// Record a lower count in the lock.
        #[arg(long)]
        update: bool,
        /// With --update, also record a higher count (needs a decision record).
        #[arg(long)]
        allow_increase: bool,
    },
    /// Print the engine's SPEC-0011 argument-kind table, or write it to
    /// eval/catalog/lift-kinds.json (T-1776).
    LiftKinds {
        /// Write the file instead of printing.
        #[arg(long)]
        write: bool,
    },
    /// Show every scored cell of one case.
    Show {
        case_id: String,
        /// Include passing cells.
        #[arg(long)]
        all: bool,
    },
    /// Write a probe case of every subexpression of `cell` of a spec case, or with `--culprit`
    /// print the innermost visibly-off node of a filled one (T-3333).
    SubexprCase(rustsheet_eval::subexpr_case::Args),
    /// Print the stored formulas/values of an xlsx as JSON.
    Snapshot { path: PathBuf },
    /// Write the package an engine-export case saves (T-4177): the file the oracle fill sends
    /// to Excel and the case scores.
    Export {
        case: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Print one suite's cases as JSON, as the harness discovers them: case and sidecar paths
    /// and, for xlsx cases, what an oracle fill needs to know (tools call this instead of
    /// re-implementing discovery).
    Cases { suite: String },
    /// Oracle calibration: compare an Excel-authored xlsx's cached values with the same
    /// workbook recalculated by an oracle backend; prints per-cell verdicts as JSON.
    Compare {
        cached: PathBuf,
        recalculated: PathBuf,
    },
    /// Score another writer's save of a workbook against ours (T-4890, the Excel re-save probe).
    PackageDiff(roundtrip::PackageDiff),
    /// Round-trip fidelity (SPEC-0033): load and save each workbook of the suites under
    /// eval/roundtrip, validate the output and score what it kept; writes
    /// eval/runs/latest/roundtrip.{json,md}. `--check` gates the baselines, `--update-baseline`
    /// ratchets them (refusing regressions).
    Roundtrip {
        #[command(flatten)]
        sel: Selection,
        #[command(flatten)]
        flags: roundtrip::Flags,
    },
    /// Gate A (SPEC-0029, decision 0188 R2): fail when a reference-table point of eval/accuracy
    /// is outside the envelope, a known-failure line is stale, an R3 mark lacks evidence or a
    /// table is defective.
    Accuracy {
        /// Where to write every point's class and values (default: eval/runs/accuracy.json).
        #[arg(long)]
        json: Option<PathBuf>,
        /// Per-batch timeout in seconds.
        #[arg(long, default_value_t = 60)]
        timeout: u64,
    },
}

fn find_root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(r) = explicit {
        return Ok(r);
    }
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join("eval/suites").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            bail!("could not find eval/suites in any ancestor; pass --root");
        }
    }
}

/// `rustsheet-eval fetch`: downloads the external suites [`to_fetch`] picks. Every suite is
/// tried; the run fails at the end, naming each that failed (T-4725).
fn cmd_fetch(root: &Path, suites: &[String], missing: bool, gates: bool) -> Result<ExitCode> {
    let all = Suite::load_all(&root.join("eval/suites"))?;
    let (fetch, current) = to_fetch(&all, suites, missing, gates);
    if fetch.is_empty() && current.is_empty() {
        bail!("no matching external suites");
    }
    for s in current {
        println!("{} is current", s.manifest.name);
    }
    let mut failed = Vec::new();
    for s in fetch {
        match fetch_suite(s) {
            Ok(files) => println!(
                "fetched {} ({files} files) -> {}",
                s.manifest.name,
                s.cache_dir.display()
            ),
            Err(e) => {
                eprintln!("fetching {} failed: {e:#}", s.manifest.name);
                failed.push(s.manifest.name.as_str());
            }
        }
    }
    if !failed.is_empty() {
        bail!("fetch failed: {}", failed.join(", "));
    }
    Ok(ExitCode::SUCCESS)
}

fn select(root: &Path, sel: &Selection, gate_only: bool) -> Result<Vec<Suite>> {
    select_from(Suite::load_all(&root.join("eval/suites"))?, sel, gate_only)
}

/// [`rustsheet_eval::select::select_from`], printing its fetched and skipped lines.
fn select_from(all: Vec<Suite>, sel: &Selection, gate_only: bool) -> Result<Vec<Suite>> {
    let chosen = rustsheet_eval::select::select_from(all, sel, gate_only)?;
    chosen.fetched.iter().for_each(|line| println!("{line}"));
    chosen.skipped.iter().for_each(|line| eprintln!("{line}"));
    Ok(chosen.suites)
}

fn execute(suites: &[Suite], sel: &Selection) -> Vec<CaseResult> {
    // Panics are captured per case; keep the default hook from spamming stderr.
    std::panic::set_hook(Box::new(|_| {}));
    let engine: Arc<dyn Engine + Send> = Arc::new(RustSheetEngine);
    let opts = RunOptions {
        timeout: Duration::from_secs(sel.timeout),
        filter: sel.filter.clone(),
    };
    run_suites(&engine, suites, &opts)
}

/// [`execute`], with each cell's [`rustsheet_eval::runner::Target`] set from the R3 marks `r3`
/// before any report counts the closeness buckets (decision 0310).
fn execute_marked(suites: &[Suite], sel: &Selection, r3: &R3Marks) -> Vec<CaseResult> {
    let mut results = execute(suites, sel);
    mark_r3_math(&mut results, r3);
    results
}

/// Fails unless every suite's `deviations` lines name an existing case and cite an accepted
/// `Status: DEVIATION` semantics entry (decision 0036).
fn check_deviations(root: &Path, suites: &[Suite]) -> Result<()> {
    for suite in suites {
        deviations::load_checked(suite, root)
            .with_context(|| format!("suite {}", suite.manifest.name))?;
    }
    Ok(())
}

/// The frozen milestone probe lists (decision 0044), the probe-milestone rules (decision 0051),
/// the named exit exceptions (decision 0087) and the Phase 1 rate exceptions (decision 0134).
struct Milestones {
    lists: Vec<MilestoneList>,
    rules: Rules,
    exceptions: ExitExceptions,
    phase1_exceptions: Phase1Exceptions,
    /// `eval/accuracy/math-over-excel.tsv`: the R3 cells R1(c) and Gate B count as passing
    /// where the engine returns the math (decision 0188 R3), and the closeness buckets count as
    /// `r3_math` (decision 0310).
    r3: R3Marks,
    /// eval/catalog/pivot-features.tsv (T-4126); empty when the root has none.
    pivot_catalog: Vec<pivot_features::CatalogRow>,
}

/// Loads the frozen lists and the probe-milestone mapping, and fails when a line of either names
/// a case that no suite discovers, or a cell line names a cell its case holds no expectation for.
/// A malformed line of the exit exceptions or the Phase 1 rate exceptions fails too.
fn load_milestones(root: &Path, catalog: &Catalog) -> Result<Milestones> {
    let lists = milestone_lists::load_all(root)?;
    let exceptions = ExitExceptions::load(root)?;
    let phase1_exceptions = Phase1Exceptions::load(root)?;
    let r3 = load_r3_marks(&root.join(ACCURACY_DIR))?;
    let mapping = Mapping::load(root)?;
    if !lists.is_empty() || mapping != Mapping::default() {
        let mut files = BTreeMap::new();
        for suite in Suite::load_all(&root.join("eval/suites"))? {
            files.extend(suite.cases()?.into_iter().map(|c| (c.id.clone(), c)));
        }
        let known: BTreeSet<String> = files.keys().cloned().collect();
        milestone_lists::check_known(&lists, &known)?;
        mapping.check_known(&known)?;
        // A cell line must name a cell its case scores (T-1568).
        let mut expectations = BTreeMap::new();
        for (case, _) in mapping.cells.keys() {
            if let Some(file) = files.get(case) {
                let cells = rustsheet_eval::case::expectation_cells(file)
                    .map_err(|e| anyhow!("{case}: {e}"))?;
                expectations.insert(case.clone(), cells);
            }
        }
        mapping.check_cells(&expectations)?;
    }
    let rules = Rules::new(&lists, mapping, catalog);
    Ok(Milestones {
        lists,
        rules,
        exceptions,
        phase1_exceptions,
        r3,
        pivot_catalog: pivot_features::load(root)?,
    })
}

/// Fills the report's milestone sections from the run's `results`.
fn score_milestones(
    milestones: &Milestones,
    catalog: &Catalog,
    suites: &[Suite],
    results: &[CaseResult],
    report: &mut RunReport,
) {
    let modes: BTreeMap<String, BaselineMode> = suites
        .iter()
        .map(|s| (s.manifest.name.clone(), s.manifest.baseline))
        .collect();
    report.milestone_lists = milestones
        .lists
        .iter()
        .map(|l| milestone_lists::summarize(l, results, &modes))
        .collect();
    let probes_mode = modes
        .get(probe_milestones::SUITE)
        .copied()
        .unwrap_or(BaselineMode::Cells);
    report.r1c = R1cCells::build(results, &milestones.r3, &milestones.phase1_exceptions);
    report.phase1b_evidence = report.r1c.phase1b();
    report.r1c_left_out = report.r1c.left_out();
    report.probe_milestones =
        probe_milestones::summarize(&milestones.rules, results, probes_mode, &report.r1c);
    if let Some(scope) = report.function_scope.as_mut() {
        scope.apply_exceptions(&milestones.exceptions);
    }
    let generated: BTreeMap<String, Vec<String>> = suites
        .iter()
        .filter(|s| s.manifest.origin == "generated")
        .map(|s| (s.manifest.name.clone(), s.manifest.milestone.clone()))
        .collect();
    report.catalog_coverage =
        catalog_coverage::summarize(catalog, results, &report.evidence, &generated);
    report.pivot_features = pivot_features::summarize(&milestones.pivot_catalog, results);
    let generated_names: Vec<&str> = generated.keys().map(String::as_str).collect();
    report.v15_checkpoint = v15_checkpoint::summarize(
        catalog,
        results,
        report,
        &generated_names,
        &milestones.phase1_exceptions,
    );
    report.milestone_exit = milestone_exit::summarize(report);
}

fn print_summary(report: &RunReport) {
    println!(
        "{:<28} {:>6} {:>9} {:>9} {:>10} {:>8}",
        "suite", "cases", "cells", "passed", "deviations", "pass%"
    );
    for s in &report.suites {
        println!(
            "{:<28} {:>6} {:>9} {:>9} {:>10} {:>7.2}%",
            s.name,
            s.cases,
            s.cells,
            s.cells_passed,
            s.deviations,
            100.0 * s.cell_pass_rate()
        );
        if !s.graph_subnormal_passes.is_empty() {
            println!(
                "  note: {} cell(s) passed by decision 0042 (graph 0 vs kept subnormal; each has a FALSE (x)=0 companion)",
                s.graph_subnormal_passes.len()
            );
        }
        for line in &s.stale_deviations {
            println!("  warning: stale deviations line (the cell matches Excel): {line}");
        }
    }
    for l in report.milestone_lists.iter().filter(|l| l.cases_run > 0) {
        println!(
            "milestone list {}: {}/{} cases run, {} cells, {} bit-exact, {} deviations, {} known failures in {} cases",
            l.name,
            l.cases_run,
            l.cases,
            l.cells,
            l.bit_exact,
            l.deviations,
            l.known_failures,
            l.failing_cases.len()
        );
    }
    if let Some(r) = report.probe_milestones.first() {
        println!(
            "probe milestone {} (decision 0051): {} cases, {} cells, {} bit-exact, {} deviations, {} known failures",
            r.milestone, r.cases, r.cells, r.bit_exact, r.deviations, r.known_failures
        );
    }
}

fn baseline_path(root: &Path, suite: &str) -> PathBuf {
    root.join("eval/baseline").join(format!("{suite}.txt"))
}

/// The ids of the cases of `suites` that the `--new-cases` file names (decision 0219).
fn listed_new_cases(root: &Path, suites: &[Suite], file: &Path) -> Result<BTreeSet<String>> {
    let listed = visible_parity::read_listed(file)?;
    let mut cases = Vec::new();
    for suite in suites.iter().filter(|s| s.manifest.fetch.is_none()) {
        cases.extend(suite.cases()?);
    }
    let ids = visible_parity::listed_ids(
        root,
        cases.iter().map(|c| (c.id.as_str(), c.path.as_path())),
        &listed,
    );
    let (ids, with_lines) = visible_parity::split_listed(&visible_parity::Lock::read(root)?, ids);
    println!(
        "visible parity: {} of the {} --new-cases entries name a case of the gated suites without a lock line; only those are new",
        ids.len(),
        listed.len()
    );
    for id in &with_lines {
        println!(
            "  {id} is listed but already has a line in {}: not new, so its line may not rise",
            visible_parity::LOCK_PATH
        );
    }
    Ok(ids)
}

/// Holds `count` to the visible-parity lock, recording it on `update` when
/// [`visible_parity::judge`] allows (T-0061, decisions 0088, 0093 and 0219); returns whether the
/// gate passes.
fn visible_parity_gate(
    root: &Path,
    count: &VisibleCount,
    update: Option<visible_parity::Update>,
) -> Result<bool> {
    use visible_parity::{Lock, LOCK_PATH};
    let lock = Lock::read(root)?;
    println!(
        "visible parity: {} in-scope cells outside the envelope ({} in new cases), {LOCK_PATH} allows {} in {} cases",
        count.cells(),
        count.new_coverage(),
        lock.total(),
        lock.cases.len()
    );
    let updating = update.is_some();
    let judgement = visible_parity::judge(&lock, count, update);
    for o in &judgement.over {
        println!(
            "  REGRESSION visible parity {}: {} cells outside the envelope, the lock allows {}",
            o.case, o.off, o.allowed
        );
    }
    if !judgement.over.is_empty() {
        let over: usize = judgement.over.iter().map(|o| o.off - o.allowed).sum();
        println!(
            "  {} cases have {over} more cells outside the envelope than {LOCK_PATH} allows. A new case needs its line: run `make eval-baseline` and commit eval/baseline/",
            judgement.over.len()
        );
    }
    if judgement.stale_measure {
        println!(
            "  {LOCK_PATH} still counts visibly-off cells (before decision 0355): run `make eval-baseline` and commit eval/baseline/"
        );
    }
    if let Some(next) = &judgement.record {
        next.write(root)?;
        println!(
            "  updated {LOCK_PATH} ({} cells in {} cases): commit it",
            next.total(),
            next.cases.len()
        );
    } else if updating
        && std::fs::read_to_string(root.join(LOCK_PATH))
            .ok()
            .as_deref()
            != Some(lock.render().as_str())
    {
        // The lines hold, but the file is not what the tool writes (a changed header, say): an
        // update rewrites it in canonical form, so the file is only ever written by the tool.
        lock.write(root)?;
        println!("  rewrote {LOCK_PATH} in canonical form: commit it");
    }
    Ok(judgement.holds())
}

/// Prints one suite's baseline summary: its counts, each regression and the drift items (decision
/// 0355 item 6), at most 25 of each.
fn print_delta(name: &str, old: &Baseline, current: &Baseline, delta: &baseline::Delta) {
    println!(
        "[{name}] known failures {} -> {} ({} -> {} outside the envelope), text {} -> {}, layout {} -> {}  (fixed {}, into envelope {}, drift {}, new-coverage failures {}, REGRESSIONS {}, new cases {}, removed cases {})",
        old.failing.len(),
        current.failing.len(),
        old.outside_items().len(),
        current.outside_items().len(),
        old.text_failing.len(),
        current.text_failing.len(),
        old.layout_failing.len(),
        current.layout_failing.len(),
        delta.fixed.len(),
        delta.into_envelope.len(),
        delta.drift.len(),
        delta.new_failures.len(),
        delta.regressions.len(),
        delta.new_cases.len(),
        delta.removed_cases.len()
    );
    for r in delta.regressions.iter().take(25) {
        println!("  REGRESSION {r}");
    }
    // Drift is allowed, but each item is shown: the baseline's diff carries all of them.
    for d in delta.drift.iter().take(25) {
        println!("  drift {}", d.replacen('\t', " ", 1));
    }
    if delta.drift.len() > 25 {
        println!(
            "  ... and {} more drift items (the baseline's diff lists them)",
            delta.drift.len() - 25
        );
    }
}

/// Holds the vendored gated suites' usage tiers to [`gate_tiers::LOCK_PATH`], recording them on
/// update (`allow_regressions` given) as [`gate_tiers::judge`] allows (decision 0355 item 6);
/// returns whether the gate passes.
fn tier_gate(
    root: &Path,
    tiers: &[gate_tiers::Tier],
    allow_regressions: Option<bool>,
) -> Result<bool> {
    use gate_tiers::{Lock, LOCK_PATH};
    let lock = Lock::read(root)?;
    println!("usage tiers on the vendored gated suites (decision 0355; the full report's tiers are the exit measure):");
    for t in tiers {
        println!("  {}", t.line());
    }
    let (holds, judgement) = gate_tiers::judge(&lock, tiers, allow_regressions);
    for usage in &judgement.regressions {
        println!(
            "  REGRESSION usage tier {}: {LOCK_PATH} records it met",
            gate_tiers::usage_name(*usage)
        );
    }
    if !judgement.regressions.is_empty() && allow_regressions == Some(false) {
        println!("  refusing to record a tier unmet without --allow-regressions");
    }
    if allow_regressions.is_none() && judgement.out_of_date && judgement.regressions.is_empty() {
        println!(
            "  {LOCK_PATH} is out of date: run `make eval-baseline` and commit eval/baseline/"
        );
    }
    if let Some(next) = &judgement.record {
        next.write(root)?;
        println!("  updated {LOCK_PATH}: commit it");
    } else if allow_regressions.is_some()
        && holds
        && std::fs::read_to_string(root.join(LOCK_PATH))
            .ok()
            .as_deref()
            != Some(lock.render().as_str())
    {
        // As for the visible-parity lock: only the tool writes the file, in canonical form.
        lock.write(root)?;
        println!("  rewrote {LOCK_PATH} in canonical form: commit it");
    }
    Ok(holds)
}

/// The baseline gate (`update` `None`) or the baseline ratchet; with `floor`, also the evidence
/// floor judged from the same run, so `check --evidence-floor` passes exactly when `check` and
/// `evidence-floor` both pass (T-2761).
fn cmd_check(
    root: &Path,
    sel: &Selection,
    update: Option<visible_parity::Update>,
    new_cases: Option<&Path>,
    floor: bool,
) -> Result<ExitCode> {
    if sel.filter.is_some() {
        bail!(
            "--filter is not allowed for baseline operations (partial runs look like regressions)"
        );
    }
    if floor && !sel.suites.is_empty() {
        bail!("--evidence-floor counts every vendored suite; it cannot be combined with --suite");
    }
    let gate = select(root, sel, true)?;
    check_deviations(root, &gate)?;
    let catalog = Catalog::load(root)?;
    let milestones = load_milestones(root, &catalog)?;
    let gate_len = gate.len();
    let run = if floor {
        evidence_floor::plan_run(gate, Suite::load_all(&root.join("eval/suites"))?)
    } else {
        gate
    };
    // `plan_run` keeps the gate suites first, unchanged.
    let suites = &run[..gate_len];
    let mut results = execute_marked(&run, sel, &milestones.r3);
    let floor_results = floor.then(|| evidence_floor::floor_results(&run, &results));
    if floor {
        // The baseline gate judges the gate suites' results only, as a plain `check` does.
        results.retain(|r| suites.iter().any(|s| s.manifest.name == r.suite));
    }
    let mut report = RunReport::build(
        RustSheetEngine.name(),
        &results,
        &RustSheetEngine.implemented_functions(),
        &catalog,
    );
    score_milestones(&milestones, &catalog, suites, &results, &mut report);
    print_summary(&report);
    let mut failed = false;
    let mut visible = VisibleCount::default();
    let listed = match new_cases {
        Some(file) => Some(listed_new_cases(root, suites, file)?),
        None => None,
    };
    for suite in suites {
        if suite.manifest.baseline == BaselineMode::None {
            continue;
        }
        let name = &suite.manifest.name;
        let mine: Vec<&CaseResult> = results.iter().filter(|r| &r.suite == name).collect();
        let current = Baseline::from_results(suite.manifest.baseline, &mine);
        let path = baseline_path(root, name);
        let old = Baseline::read(&path)?;
        if suite.manifest.fetch.is_none() {
            for r in &mine {
                let new_case = visible_parity::is_new_case(
                    listed.as_ref(),
                    old.checked.contains_key(&r.id),
                    &r.id,
                );
                visible.add(&catalog, r, new_case);
            }
        }
        let delta = baseline::diff(&old, &current);
        print_delta(name, &old, &current, &delta);
        match update {
            None => {
                if delta.is_dirty() {
                    failed = true;
                    if delta.regressions.is_empty() {
                        println!("  baseline is out of date: run `make eval-baseline` and commit eval/baseline/");
                    }
                }
            }
            Some(u) => {
                if !delta.regressions.is_empty() && !u.allow_regressions {
                    failed = true;
                    println!("  refusing to record regressions without --allow-regressions");
                } else {
                    current.write(&path)?;
                }
            }
        }
    }
    // A partial run (`--suite`) would undercount, so only a full gated run is held to the locks.
    if sel.suites.is_empty() && !visible_parity_gate(root, &visible, update)? {
        failed = true;
    }
    if sel.suites.is_empty() {
        let vendored = results.iter().filter(|r| {
            suites
                .iter()
                .any(|s| s.manifest.name == r.suite && s.manifest.fetch.is_none())
        });
        let cells = gate_tiers::function_cells(&catalog, vendored, &milestones.phase1_exceptions);
        let tiers = gate_tiers::tiers(&catalog, &cells);
        if !tier_gate(root, &tiers, update.map(|u| u.allow_regressions))? {
            failed = true;
        }
    }
    if let Some(floor_results) = floor_results {
        if !evidence_floor_gate(root, &catalog, &floor_results, false, false)? {
            failed = true;
        }
    }
    Ok(ExitCode::from(u8::from(failed)))
}

fn cmd_lift_kinds(root: &Path, write: bool) -> Result<ExitCode> {
    let text = lift_kinds::render()?;
    if write {
        let path = root.join(lift_kinds::LIFT_KINDS_PATH);
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        println!("wrote {}", lift_kinds::LIFT_KINDS_PATH);
    } else {
        print!("{text}");
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_evidence_floor(root: &Path, update: bool, allow_increase: bool) -> Result<ExitCode> {
    let sel = Selection {
        suites: Suite::load_all(&root.join("eval/suites"))?
            .into_iter()
            .filter(evidence_floor::counts)
            .map(|s| s.manifest.name)
            .collect(),
        filter: None,
        timeout: 60,
        ..Selection::default()
    };
    let suites = select(root, &sel, false)?;
    let catalog = Catalog::load(root)?;
    let results = execute(&suites, &sel);
    Ok(
        if evidence_floor_gate(root, &catalog, &results, update, allow_increase)? {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    )
}

/// Holds the number of functions below the floor in `results` (a run of the vendored suites) to
/// [`LOCK_PATH`], recording it on `update` when [`evidence_floor::may_record`] allows; returns
/// whether the gate passes (decision 0062).
fn evidence_floor_gate(
    root: &Path,
    catalog: &Catalog,
    results: &[CaseResult],
    update: bool,
    allow_increase: bool,
) -> Result<bool> {
    let report = RunReport::build(
        RustSheetEngine.name(),
        results,
        &RustSheetEngine.implemented_functions(),
        catalog,
    );
    let count = report.below_floor.len();
    let lock = Lock::read(root)?;
    let verdict = evidence_floor::judge(lock, count);
    println!(
        "evidence floor: {count} in-scope functions with fewer than {EVIDENCE_FLOOR} Excel cells (lock {} at floor {})",
        lock.below_floor, lock.floor
    );
    let new_lock = Lock {
        floor: EVIDENCE_FLOOR,
        below_floor: count,
    };
    let ok = match (verdict, update) {
        (evidence_floor::Verdict::Unchanged, _) => true,
        (_, true) if !evidence_floor::may_record(verdict, allow_increase) => {
            println!("  refusing to record an increase or a new floor without --allow-increase (needs a decision record)");
            false
        }
        (_, true) => {
            new_lock.write(root)?;
            println!("  updated {LOCK_PATH}: commit it");
            true
        }
        (evidence_floor::Verdict::Improved, false) => {
            println!("  fewer functions are below the floor: run `make eval-baseline` and commit {LOCK_PATH}");
            false
        }
        (evidence_floor::Verdict::Regressed, false) => {
            println!("  REGRESSION: more functions are below the floor than {LOCK_PATH} allows; the report's \"Below the evidence floor\" list names them");
            false
        }
        (evidence_floor::Verdict::FloorChanged, false) => {
            println!("  {LOCK_PATH} was taken at another floor: rerun with --update --allow-increase under a decision record");
            false
        }
    };
    Ok(ok)
}

fn cmd_show(root: &Path, case_id: &str, all: bool) -> Result<ExitCode> {
    let suite_name = case_id.split('/').next().unwrap_or_default();
    let sel = Selection {
        suites: vec![suite_name.to_string()],
        filter: Some(case_id.to_string()),
        timeout: 120,
        ..Selection::default()
    };
    // `load_named` also finds Gate A's tables, which `select` (eval/suites only) never lists.
    let suite = Suite::load_named(root, suite_name)?;
    if suite.fetch_state() == FetchState::NotFetched {
        bail!(
            "suite {suite_name} is not fetched (run `rustsheet-eval fetch --suite {suite_name}`)"
        );
    }
    let results = execute(&[suite], &sel);
    let Some(r) = results.iter().find(|r| r.id == case_id) else {
        bail!("case {case_id} not found");
    };
    println!(
        "{}  status={:?}  checked={} passed={} deviations={}  source={}",
        r.id, r.status, r.checked, r.passed, r.deviations, r.expected_source
    );
    if let Some(m) = &r.message {
        println!("message: {m}");
    }
    for c in r.cells.iter().filter(|c| all || !c.verdict.is_pass()) {
        println!(
            "{:<20} {:?}\n    formula : ={}\n    expected: {}\n    actual  : {}",
            c.key(),
            c.verdict,
            c.formula.as_deref().unwrap_or(""),
            c.expected,
            c.actual
        );
    }
    let mut text = String::new();
    text_expect::write_case(&r.text, all, &mut text);
    print!("{text}");
    Ok(ExitCode::SUCCESS)
}

/// Gate A's check (SPEC-0029): prints the summary and failures, writes the JSON, and fails on
/// any failing point, stale known-failure line, bad R3 mark or table defect.
fn cmd_accuracy(root: &Path, json: Option<PathBuf>, timeout: u64) -> Result<ExitCode> {
    // Panics are captured per batch; keep the default hook from spamming stderr.
    std::panic::set_hook(Box::new(|_| {}));
    let json = json.unwrap_or_else(|| root.join(accuracy::JSON_PATH));
    let engine: Arc<dyn Engine + Send> = Arc::new(RustSheetEngine);
    let check = accuracy::run(root, &engine, Duration::from_secs(timeout))?;
    print!("{}", check.render());
    check.write_json(&json)?;
    Ok(ExitCode::from(u8::from(!check.passes())))
}

fn load_snapshot(path: &Path) -> Result<WorkbookSnapshot> {
    let bytes = std::fs::read(path).with_context(|| path.display().to_string())?;
    Ok(read_snapshot(&bytes)?)
}

fn cmd_cases(root: &Path, suite: &str) -> Result<ExitCode> {
    println!("{}", rustsheet_eval::case_listing::list(root, suite)?);
    Ok(ExitCode::SUCCESS)
}

/// `rustsheet-eval export`: the package an engine-export case saves, written to `out`.
fn cmd_export(case: &Path, out: &Path) -> Result<ExitCode> {
    let bytes = rustsheet_eval::export_case::export(case).map_err(|e| anyhow!(e))?;
    std::fs::write(out, bytes).with_context(|| out.display().to_string())?;
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    if let Cmd::Export { case, out } = &cli.cmd {
        return cmd_export(case, out);
    }
    if let Cmd::Snapshot { path } = &cli.cmd {
        println!("{}", serde_json::to_string_pretty(&load_snapshot(path)?)?);
        return Ok(ExitCode::SUCCESS);
    }
    if let Cmd::Compare {
        cached,
        recalculated,
    } = &cli.cmd
    {
        let comparison = compare_snapshots(&load_snapshot(cached)?, &load_snapshot(recalculated)?);
        println!("{}", serde_json::to_string(&comparison)?);
        return Ok(ExitCode::SUCCESS);
    }
    if let Cmd::PackageDiff(diff) = &cli.cmd {
        let (items, summary) = diff.report()?;
        print!("{items}");
        eprintln!("{summary}");
        return Ok(ExitCode::SUCCESS);
    }
    let root = find_root(cli.root)?;
    match cli.cmd {
        Cmd::Suites => cmd_suites(&root),
        Cmd::Fetch {
            suites,
            missing,
            gates,
        } => cmd_fetch(&root, &suites, missing, gates),
        Cmd::Run {
            sel,
            out,
            examples,
            desktop_owned,
        } => {
            let suites = select(&root, &sel, false)?;
            check_deviations(&root, &suites)?;
            let catalog = Catalog::load(&root)?;
            let mut milestones = load_milestones(&root, &catalog)?;
            if desktop_owned {
                milestones.phase1_exceptions =
                    std::mem::take(&mut milestones.phase1_exceptions).load_desktop_owned(&root)?;
            }
            let started = std::time::SystemTime::now();
            let results = execute_marked(&suites, &sel, &milestones.r3);
            let mut report = RunReport::build(
                RustSheetEngine.name(),
                &results,
                &RustSheetEngine.implemented_functions(),
                &catalog,
            );
            score_milestones(&milestones, &catalog, &suites, &results, &mut report);
            let all = Suite::load_all(&root.join("eval/suites"))?;
            report.run = RunMeta::new(&root, started, &all, &suites);
            let out = out.unwrap_or_else(|| root.join("eval/runs/latest"));
            std::fs::create_dir_all(&out)?;
            std::fs::write(
                out.join("report.json"),
                serde_json::to_string_pretty(&report)?,
            )?;
            std::fs::write(out.join("report.md"), report.to_markdown(examples))?;
            print_summary(&report);
            println!("report: {}", out.join("report.md").display());
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Check {
            sel,
            evidence_floor,
        } => cmd_check(&root, &sel, None, None, evidence_floor),
        Cmd::UpdateBaseline {
            sel,
            allow_regressions,
            lower_visible_lock,
            new_cases,
        } => cmd_check(
            &root,
            &sel,
            Some(visible_parity::Update {
                allow_regressions,
                lower: lower_visible_lock,
            }),
            new_cases.as_deref(),
            false,
        ),
        Cmd::EvidenceFloor {
            update,
            allow_increase,
        } => cmd_evidence_floor(&root, update, allow_increase),
        Cmd::LiftKinds { write } => cmd_lift_kinds(&root, write),
        Cmd::Show { case_id, all } => cmd_show(&root, &case_id, all),
        Cmd::SubexprCase(args) => rustsheet_eval::subexpr_case::run(&root, args).map(|text| {
            print!("{text}");
            ExitCode::SUCCESS
        }),
        Cmd::Cases { suite } => cmd_cases(&root, &suite),
        Cmd::Accuracy { json, timeout } => cmd_accuracy(&root, json, timeout),
        Cmd::Roundtrip { sel, flags } => cmd_roundtrip(&root, &sel, &flags),
        Cmd::Snapshot { .. } | Cmd::Compare { .. } | Cmd::PackageDiff(_) | Cmd::Export { .. } => {
            unreachable!("handled above")
        }
    }
}

/// `rustsheet-eval suites`: the cell suites, then the round-trip suites (SPEC-0033).
fn cmd_suites(root: &Path) -> Result<ExitCode> {
    let mut all = Suite::load_all(&root.join("eval/suites"))?;
    all.extend(Suite::load_roundtrip(root)?);
    for s in all {
        let n = s.cases().map_or(0, |c| c.len());
        let cases = match s.fetch_state() {
            FetchState::NotFetched => rustsheet_eval::run_meta::NOT_FETCHED.to_string(),
            _ => format!("{n} cases"),
        };
        println!(
            "{:<28} {:>12}  gate={:<5} license={:<20} {}",
            s.manifest.name, cases, s.manifest.gate, s.manifest.license, s.manifest.description
        );
    }
    Ok(ExitCode::SUCCESS)
}

/// `rustsheet-eval roundtrip` (SPEC-0033): the gate suites with `--check`, else the selection.
fn cmd_roundtrip(root: &Path, sel: &Selection, flags: &roundtrip::Flags) -> Result<ExitCode> {
    let suites = select_from(Suite::load_roundtrip(root)?, sel, flags.check)?;
    let timeout = Duration::from_secs(sel.timeout);
    let (ok, lines) = roundtrip::command(root, &suites, sel.filter.as_deref(), timeout, flags)?;
    for line in lines {
        println!("{line}");
    }
    Ok(ExitCode::from(u8::from(!ok)))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
