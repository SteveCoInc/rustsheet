//! Probe milestones (decision 0051): every `probes` case, and every checked cell of
//! `probes/catalog/*`, counts toward exactly one milestone's exit criterion. The first rule that
//! applies decides:
//!
//! 1. a case on the frozen list `eval/milestones/m1-probes.txt` is M1;
//! 2. a cell line of [`MAPPING_PATH`] (`case`, `Sheet!A1`, milestone, subject) sets that cell's
//!    milestone, and a case line (`case`, milestone, subject) sets the case's;
//! 3. a `probes/catalog/*` cell takes the highest catalog milestone of the functions its formula
//!    calls, apart from the observing wrappers OR, IFERROR, ERROR.TYPE and INDEX (M2 when none);
//! 4. any other case takes its case.json milestone tag (the highest one when it has several);
//! 5. a case with no milestone tag is M2.
//!
//! The report scores each milestone the way the baseline does, so its known failures equal the
//! F lines of `eval/baseline/probes.txt` that the rules assign to it. Each row also counts its
//! cells by usage tier ([`MilestoneRow::tiers`], decision 0355 item 4): the misses are the cells
//! R1(c) counts, a wrong type or a number outside the envelope ([`R1cCells::miss`]).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::baseline::Baseline;
use crate::case::split_qualified;
use crate::catalog::{Catalog, Usage};
use crate::cellref::Addr;
use crate::closeness::Visible;
use crate::formula_calls::functions_in;
use crate::milestone_lists::MilestoneList;
use crate::r1c::{R1c, R1cCells};
use crate::runner::{CaseResult, CellKind, CellResult};
use crate::suite::BaselineMode;
use crate::usage_gates::{cell_usage, TierTallies};
use crate::value::Verdict;

/// The mapping file, relative to the repository root.
pub const MAPPING_PATH: &str = "eval/milestones/probe-milestones.tsv";
/// The suite whose cases the rules assign.
pub const SUITE: &str = "probes";
/// The frozen list whose cases are M1 (rule 1).
pub const FROZEN_M1_LIST: &str = "m1-probes";
/// Cases scored per cell by rule 3.
const CATALOG_PREFIX: &str = "probes/catalog/";
/// Functions that only observe a catalog cell's result, so they do not set its milestone.
const OBSERVING_WRAPPERS: [&str; 4] = ["OR", "IFERROR", "ERROR.TYPE", "INDEX"];
/// Rule 5's milestone, and rule 3's for a catalog cell that calls no catalog function.
pub(crate) const DEFAULT_MILESTONE: u32 = 2;
/// The milestones that always get a report row; M0, M1 and M2 share the first one. Any other
/// milestone a case is assigned gets a row of its own after these, so nothing drops out.
pub(crate) const REPORT_ROWS: [u32; 8] = [2, 5, 6, 7, 8, 9, 10, 11];

/// Parses `M<digits>`. Numeric order is the phase order of docs/milestones.md for every
/// milestone a probe can take (M0–M2 and M5–M11).
pub(crate) fn parse_milestone(s: &str) -> Option<u32> {
    let digits = s.strip_prefix('M')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// The data lines of a tab-separated file under `eval/milestones/`: each line's 1-based number
/// and its fields, skipping blank lines and `#` comment lines.
pub(crate) fn data_lines(text: &str) -> impl Iterator<Item = (usize, Vec<&str>)> {
    text.lines()
        .enumerate()
        .filter(|(_, raw)| !raw.trim().is_empty() && !raw.starts_with('#'))
        .map(|(n, raw)| (n + 1, raw.split('\t').collect()))
}

/// The report row a milestone counts in: M0 and M1 count with M2 (decision 0051, item 3).
pub(crate) fn row_of(milestone: u32) -> u32 {
    milestone.max(DEFAULT_MILESTONE)
}

pub(crate) fn row_label(row: u32) -> String {
    if row == DEFAULT_MILESTONE {
        "M0-M2".to_string()
    } else {
        format!("M{row}")
    }
}

/// One line of the mapping file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assigned {
    /// 1-based line number.
    pub line: usize,
    /// The milestone number (`M9` → 9).
    pub milestone: u32,
}

/// The parsed mapping file: case lines and cell lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mapping {
    /// Case id → the case's milestone.
    pub cases: BTreeMap<String, Assigned>,
    /// (case id, `Sheet!A1`) → that cell's milestone.
    pub cells: BTreeMap<(String, String), Assigned>,
}

/// Why the mapping file or one of its lines is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MappingError {
    /// The file cannot be read.
    #[error("{MAPPING_PATH}: {message}")]
    Unreadable { message: String },
    /// A line is malformed.
    #[error("{MAPPING_PATH} line {line}: {why}")]
    Line { line: usize, why: &'static str },
    /// A case, or a cell of a case, is listed twice.
    #[error("{MAPPING_PATH} line {line}: {item} is already listed on line {first}")]
    Duplicate {
        line: usize,
        item: String,
        first: usize,
    },
    /// A line names a case that the `probes` suite does not discover.
    #[error("{MAPPING_PATH} line {line}: no case {case}")]
    UnknownCase { line: usize, case: String },
    /// A cell line names a cell that its case does not check.
    #[error("{MAPPING_PATH} line {line}: {case} has no checked cell {cell}")]
    UnknownCell {
        line: usize,
        case: String,
        cell: String,
    },
}

impl Mapping {
    /// Parses the mapping file's text. Lines are tab-separated: `case, milestone, subject`, or
    /// `case, Sheet!A1, milestone, subject` for one cell; `#` starts a comment line.
    ///
    /// # Errors
    /// The first malformed line, or the first that repeats a case or a cell.
    pub fn parse(text: &str) -> Result<Mapping, MappingError> {
        let mut mapping = Mapping::default();
        for (line, fields) in data_lines(text) {
            let bad = |why| MappingError::Line { line, why };
            let (case, cell, milestone, subject) = match fields.as_slice() {
                [case, milestone, subject] => (*case, None, *milestone, *subject),
                [case, cell, milestone, subject] => (*case, Some(*cell), *milestone, *subject),
                _ => {
                    return Err(bad(
                        "expected `case<TAB>milestone<TAB>subject` or `case<TAB>Sheet!A1<TAB>milestone<TAB>subject`",
                    ))
                }
            };
            if !case.starts_with("probes/") || case.contains(char::is_whitespace) {
                return Err(bad("the case id must be `probes/<path>`"));
            }
            if subject.trim().is_empty() {
                return Err(bad("the subject is empty"));
            }
            let milestone =
                parse_milestone(milestone).ok_or(bad("the milestone must be `M<digits>`"))?;
            let assigned = Assigned { line, milestone };
            let first = match cell {
                None => mapping.cases.insert(case.to_string(), assigned),
                Some(cell) => {
                    let key = split_qualified(cell)
                        .and_then(|(sheet, a1)| Some(format!("{sheet}!{}", Addr::parse(&a1)?)))
                        .ok_or(bad("the cell must be `Sheet!A1`"))?;
                    mapping.cells.insert((case.to_string(), key), assigned)
                }
            };
            if let Some(first) = first {
                return Err(MappingError::Duplicate {
                    line,
                    item: cell.map_or_else(|| case.to_string(), |c| format!("{case} {c}")),
                    first: first.line,
                });
            }
        }
        Ok(mapping)
    }

    /// Reads [`MAPPING_PATH`] under `repo_root`; an absent file is an empty mapping.
    ///
    /// # Errors
    /// When the file is unreadable or malformed.
    pub fn load(repo_root: &Path) -> Result<Mapping, MappingError> {
        let path = repo_root.join(MAPPING_PATH);
        if !path.exists() {
            return Ok(Mapping::default());
        }
        let text = std::fs::read_to_string(&path).map_err(|e| MappingError::Unreadable {
            message: e.to_string(),
        })?;
        Mapping::parse(&text)
    }

    /// Checks that every line names a case the suites discover (`known` holds their case ids).
    ///
    /// # Errors
    /// The first line, in file order, that names an unknown case.
    pub fn check_known(&self, known: &BTreeSet<String>) -> Result<(), MappingError> {
        let cases = self.cases.iter().map(|(c, a)| (c, a.line));
        let cells = self.cells.iter().map(|((c, _), a)| (c, a.line));
        match cases
            .chain(cells)
            .filter(|(c, _)| !known.contains(*c))
            .min_by_key(|(_, line)| *line)
        {
            Some((case, line)) => Err(MappingError::UnknownCase {
                line,
                case: case.clone(),
            }),
            None => Ok(()),
        }
    }

    /// Checks that every cell line names a checked cell of its case, so a typo cannot leave a cell
    /// counted at its case's milestone unnoticed. `expectations` maps a case id to the cells the
    /// case holds an expectation for (the keys of its `expected` and of its oracle sidecar, as
    /// `Sheet!A1`; `case::expectation_cells`). A case it does not list is not checked here:
    /// [`Mapping::check_known`] rejects unknown cases.
    ///
    /// # Errors
    /// The first line, in file order, whose cell its case holds no expectation for.
    pub fn check_cells(
        &self,
        expectations: &BTreeMap<String, BTreeSet<String>>,
    ) -> Result<(), MappingError> {
        match self
            .cells
            .iter()
            .filter(|((case, cell), _)| {
                expectations
                    .get(case)
                    .is_some_and(|cells| !cells.contains(cell))
            })
            .min_by_key(|(_, a)| a.line)
        {
            Some(((case, cell), a)) => Err(MappingError::UnknownCell {
                line: a.line,
                case: case.clone(),
                cell: cell.clone(),
            }),
            None => Ok(()),
        }
    }
}

/// Applies rules 1–5.
#[derive(Debug, Clone, Default)]
pub struct Rules {
    frozen_m1: BTreeSet<String>,
    mapping: Mapping,
    /// Catalog function name → milestone number.
    catalog: BTreeMap<String, u32>,
    /// Catalog function name → usage tier (decision 0355).
    usage: BTreeMap<String, Usage>,
}

impl Rules {
    /// The rules from the frozen lists (only [`FROZEN_M1_LIST`] matters), the mapping file and
    /// the catalog.
    #[must_use]
    pub fn new(lists: &[MilestoneList], mapping: Mapping, catalog: &Catalog) -> Rules {
        Rules {
            frozen_m1: lists
                .iter()
                .filter(|l| l.name == FROZEN_M1_LIST)
                .flat_map(|l| l.cases.iter().map(|c| c.id.clone()))
                .collect(),
            mapping,
            catalog: catalog
                .functions()
                .filter_map(|f| Some((f.name.clone(), parse_milestone(&f.milestone)?)))
                .collect(),
            usage: catalog
                .functions()
                .map(|f| (f.name.clone(), f.usage))
                .collect(),
        }
    }

    /// The usage tier of cell `c` of `r` ([`cell_usage`] of the functions it calls).
    fn cell_usage(&self, r: &CaseResult, c: &CellResult) -> Usage {
        let calls = crate::function_scope::cell_calls(r, c).unwrap_or_default();
        cell_usage(&calls, |f| self.usage.get(f).copied())
    }

    /// The milestone of the whole case: rules 1, 2 (case lines), 4 and 5. A catalog case's own
    /// milestone (for items that are not cells) is M2.
    fn case_milestone(&self, r: &CaseResult) -> u32 {
        if self.frozen_m1.contains(&r.id) {
            return 1;
        }
        if let Some(a) = self.mapping.cases.get(&r.id) {
            return a.milestone;
        }
        if r.id.starts_with(CATALOG_PREFIX) {
            return DEFAULT_MILESTONE;
        }
        r.tags
            .iter()
            .filter_map(|t| parse_milestone(t))
            .max()
            .unwrap_or(DEFAULT_MILESTONE)
    }

    /// The milestone of one scored cell of `r`.
    fn cell_milestone(&self, r: &CaseResult, cell: &CellResult) -> u32 {
        if self.frozen_m1.contains(&r.id) {
            return 1;
        }
        if let Some(a) = self.mapping.cells.get(&(r.id.clone(), cell.key())) {
            return a.milestone;
        }
        if r.id.starts_with(CATALOG_PREFIX) && !self.mapping.cases.contains_key(&r.id) {
            return cell
                .formula
                .as_deref()
                .map(functions_in)
                .unwrap_or_default()
                .iter()
                .filter(|f| !OBSERVING_WRAPPERS.contains(&f.as_str()))
                .filter_map(|f| self.catalog.get(f).copied())
                .max()
                .unwrap_or(DEFAULT_MILESTONE);
        }
        self.case_milestone(r)
    }
}

/// How the probe cases assigned to one milestone scored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneRow {
    /// `M0-M2`, `M5`, … `M11`.
    pub milestone: String,
    /// Cases with at least one item in this milestone (a catalog case can count in several).
    pub cases: usize,
    /// Scored cells.
    pub cells: usize,
    /// Cells that match Excel bit-exact.
    pub bit_exact: usize,
    /// Cells that gave their pinned by-design value (decision 0036).
    pub deviations: usize,
    /// Known-failure items, counted as the baseline counts its F lines.
    pub known_failures: usize,
    /// The cases behind `known_failures`: case id and its known-failure items in this row,
    /// most first, then by id.
    pub failing_cases: Vec<(String, usize)>,
    /// Scored cells that are visibly off (decision 0083): the Phase 1 exit counts these
    /// (decision 0114).
    #[serde(default)]
    pub visibly_off: usize,
    /// Those cells, as `case!Sheet!A1`, in case order: what `scripts/exit_owners.py` lists for the
    /// row (decision 0135).
    #[serde(default)]
    pub visibly_off_cells: Vec<String>,
    /// Of `visibly_off`, the cells decision 0188 R1(c) counts ([`crate::r1c`]): the row's Phase 1
    /// exit reads these (T-3427).
    #[serde(default)]
    pub r1c_off: usize,
    /// Those cells, as `case!Sheet!A1`, in case order.
    #[serde(default)]
    pub r1c_off_cells: Vec<String>,
    /// Of `visibly_off`, the numeric cells inside Gate A's envelope: Phase 1b evidence.
    #[serde(default)]
    pub phase1b: usize,
    /// The R3 cells where the engine returns the math (decision 0188 R3): the `r3_math` bucket,
    /// not part of `visibly_off` (decision 0310).
    #[serde(default)]
    pub r3_pass: usize,
    /// Of `visibly_off`, the cells only open M14 tasks own (decision 0188 R7).
    #[serde(default)]
    pub m14_owned: usize,
    /// The row's non-deviation cells by usage tier, with the misses R1(c) counts (a wrong type
    /// or a number outside the envelope); the M14-owned cells leave it. The row's probe exit
    /// applies the tier bars to it, pooled within the row (decision 0355 item 4).
    #[serde(default)]
    pub tiers: TierTallies,
    /// Cells the spill-extent check failed (filled by the engine, left empty by Excel; decision
    /// 0063): the M7 exit requires none (decision 0140).
    #[serde(default)]
    pub spill_extent_failures: usize,
    /// Displayed-text items scored for the row's cases (T-1610; information, not a gate).
    #[serde(default)]
    pub text_items: usize,
    /// Text items whose text differs from Excel's.
    #[serde(default)]
    pub text_failing: usize,
    /// Those items, as `case!Sheet!A1`, in case order: what `scripts/exit_owners.py` lists for the
    /// row (decision 0146 item 4).
    #[serde(default)]
    pub text_failing_cells: Vec<String>,
    /// Cases of the row that did not run to scoring, as `case!*` ([`CaseResult::blocked_key`]),
    /// in case order: the row's exit measure is not met while there is one (decision 0174).
    #[serde(default)]
    pub blocked_cases: Vec<String>,
}

/// `case!Sheet!A1` when cell `c` of case `r` is visibly off (decision 0114): neither bit-exact nor
/// a pinned deviation, and outside what `Visible` tolerates.
pub(crate) fn visibly_off_name(r: &CaseResult, c: &CellResult) -> Option<String> {
    Visible::is_failing_off(c).then(|| crate::runner::cell_id(&r.id, &c.key()))
}

/// Scores the `probes` cases of `results` per milestone; `mode` is the suite's baseline mode and
/// `r1c` the run's R1(c) classes. Empty when the run executed no `probes` case.
#[must_use]
pub fn summarize(
    rules: &Rules,
    results: &[CaseResult],
    mode: BaselineMode,
    r1c: &R1cCells,
) -> Vec<MilestoneRow> {
    let probes: Vec<&CaseResult> = results.iter().filter(|r| r.suite == SUITE).collect();
    if probes.is_empty() {
        return Vec::new();
    }
    let mut rows: BTreeMap<u32, MilestoneRow> = REPORT_ROWS
        .iter()
        .map(|&m| (m, MilestoneRow::default()))
        .collect();
    for r in probes {
        let mut rows_of_case: BTreeSet<u32> = BTreeSet::new();
        let cell_rows: BTreeMap<String, u32> = r
            .cells
            .iter()
            .map(|c| (c.key(), row_of(rules.cell_milestone(r, c))))
            .collect();
        for c in &r.cells {
            let row = cell_rows
                .get(&c.key())
                .copied()
                .unwrap_or(DEFAULT_MILESTONE);
            let s = rows.entry(row).or_default();
            s.cells += 1;
            if c.kind == CellKind::SpillExtent {
                s.spill_extent_failures += 1;
            }
            let usage = rules.cell_usage(r, c);
            if c.verdict.is_pass() {
                s.bit_exact += 1;
                s.tiers.tally_mut(usage).cells += 1;
            } else if c.verdict == Verdict::Deviation {
                s.deviations += 1;
            } else if let Some(name) = visibly_off_name(r, c) {
                s.visibly_off += 1;
                match r1c.class(&name) {
                    R1c::Counts => {
                        s.r1c_off += 1;
                        s.r1c_off_cells.push(name.clone());
                    }
                    R1c::Phase1b => s.phase1b += 1,
                    R1c::R3Pass => s.r3_pass += 1,
                    R1c::M14Owned => s.m14_owned += 1,
                }
                if r1c.class(&name) != R1c::M14Owned {
                    let tally = s.tiers.tally_mut(usage);
                    tally.cells += 1;
                    if let Some(miss) = r1c.miss(&name) {
                        tally.miss(miss);
                    }
                }
                s.visibly_off_cells.push(name);
            } else {
                // A failure that is not visibly off: within 15 digits (13 for decision 0267's
                // family), an unsupported result, or math over Excel by design (decision 0310,
                // R1(c)'s R3 pass). R1(c) counts none of them.
                if Visible::is_r3_math(c) {
                    s.r3_pass += 1;
                }
                s.tiers.tally_mut(usage).cells += 1;
            }
            rows_of_case.insert(row);
        }
        let case_row = row_of(rules.case_milestone(r));
        if let Some(key) = r.blocked_key() {
            rows.entry(case_row).or_default().blocked_cases.push(key);
        }
        // A text item takes its cell's milestone, as its value cell does (decision 0146 item 4).
        for t in &r.text {
            let key = format!("{}!{}", t.sheet, t.cell);
            let row = cell_rows.get(&key).copied().unwrap_or(case_row);
            let s = rows.entry(row).or_default();
            s.text_items += 1;
            if !t.pass {
                s.text_failing += 1;
                s.text_failing_cells
                    .push(crate::runner::cell_id(&r.id, &key));
            }
            rows_of_case.insert(row);
        }
        if r.cells.is_empty() {
            rows_of_case.insert(case_row);
        }
        // Items that are not cells (`*`, `!Panic`, …) count at the case's own milestone.
        for (_, item) in Baseline::from_results(mode, &[r]).failing {
            let row = cell_rows.get(&item).copied().unwrap_or(case_row);
            let s = rows.entry(row).or_default();
            s.known_failures += 1;
            // Only this case appends during this iteration, so the row's last entry is this case's if present.
            match s.failing_cases.last_mut() {
                Some((id, n)) if *id == r.id => *n += 1,
                _ => s.failing_cases.push((r.id.clone(), 1)),
            }
            rows_of_case.insert(row);
        }
        for row in rows_of_case {
            rows.entry(row).or_default().cases += 1;
        }
    }
    rows.into_iter()
        .map(|(row, mut s)| {
            s.failing_cases
                .sort_by(|(a, na), (b, nb)| nb.cmp(na).then_with(|| a.cmp(b)));
            MilestoneRow {
                milestone: row_label(row),
                ..s
            }
        })
        .collect()
}

/// The report.md section for `rows`; nothing when there are none.
pub fn write_markdown(rows: &[MilestoneRow], md: &mut String) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(
        md,
        "\n## Probe milestones (decision 0051)\n\n`probes` cases (and `probes/catalog/*` cells) by \
         the milestone whose exit they count toward: the frozen M1 list, then `{MAPPING_PATH}`, \
         then each catalog cell's functions, then the case's milestone tag, else M2. Known \
         failures count items as the baseline counts F lines. `did not run` counts the cases that \
         failed to load or evaluate, panicked or timed out (decision 0174).\n"
    );
    let _ = writeln!(
        md,
        "| milestone | cases | scored cells | bit-exact | deviations | known failures (cells) | visibly off | text items | failing text | did not run |\n\
         |---|---|---|---|---|---|---|---|---|---|"
    );
    for s in rows {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            s.milestone,
            s.cases,
            s.cells,
            s.bit_exact,
            s.deviations,
            s.known_failures,
            s.visibly_off,
            s.text_items,
            s.text_failing,
            s.blocked_cases.len()
        );
    }
    for s in rows.iter().filter(|s| !s.failing_cases.is_empty()) {
        let _ = writeln!(md, "\nFailing cases in {}:\n", s.milestone);
        for (id, n) in &s.failing_cases {
            let _ = writeln!(md, "- `{id}`: {n}");
        }
    }
}

#[cfg(test)]
mod tests;
