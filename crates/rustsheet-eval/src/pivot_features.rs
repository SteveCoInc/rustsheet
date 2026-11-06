//! Pivot feature coverage (T-4126, M16): one row per pivot catalog id (eval/catalog/pivot-features.tsv,
//! T-4125) with the Excel-evidence pivot cells scored for it and how many pass, so "implemented"
//! always means measured. A spec case names the catalog ids it exercises in its `pivot_features`
//! list. Every scored sidecar pivot-area cell of the case (T-4124) counts toward each listed id, the
//! way the function catalog's coverage counts a cell toward the functions its formula calls. A row
//! in scope with no cell is uncovered. Gating on these rows is the M16 exit row's task, not this one.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::runner::CaseResult;

/// The catalog, relative to the repository root.
pub const CATALOG_PATH: &str = "eval/catalog/pivot-features.tsv";

/// The catalog columns the report reads.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogRow {
    /// `PV-nnnn`.
    pub id: String,
    pub family: String,
    /// The OOXML path or enumeration value the row stands for.
    pub ooxml: String,
    /// `in`, `out (...)` or `round-trip only (...)` (decision 0274).
    pub scope: String,
    /// The M16 stage (`PV1`...), `out` or `round-trip`.
    pub stage: String,
    /// `todo`, `done`, `retired`...
    pub status: String,
}

impl CatalogRow {
    fn in_scope(&self) -> bool {
        self.scope == "in" && self.status != "retired"
    }
}

/// Reads the catalog under `root`; an absent file is an empty catalog (a test root has none).
pub fn load(root: &Path) -> Result<Vec<CatalogRow>> {
    let path = root.join(CATALOG_PATH);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path).with_context(|| path.display().to_string())?;
    parse(&text).with_context(|| path.display().to_string())
}

fn parse(text: &str) -> Result<Vec<CatalogRow>> {
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let col = |name: &str| header.iter().position(|h| *h == name);
    let (Some(id), Some(family), Some(ooxml), Some(scope), Some(stage), Some(status)) = (
        col("id"),
        col("family"),
        col("ooxml"),
        col("scope"),
        col("stage"),
        col("status"),
    ) else {
        bail!("the pivot catalog header lacks id, family, ooxml, scope, stage or status");
    };
    lines
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(n, line)| {
            let cells: Vec<&str> = line.split('\t').collect();
            let get = |i: usize| -> Result<String> {
                cells
                    .get(i)
                    .map(|s| (*s).to_string())
                    .with_context(|| format!("line {}: too few cells", n + 2))
            };
            Ok(CatalogRow {
                id: get(id)?,
                family: get(family)?,
                ooxml: get(ooxml)?,
                scope: get(scope)?,
                stage: get(stage)?,
                status: get(status)?,
            })
        })
        .collect()
}

/// One catalog row's measured evidence.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureRow {
    pub id: String,
    pub family: String,
    pub ooxml: String,
    pub scope: String,
    pub stage: String,
    /// Excel-evidence pivot cells of the cases that name this id.
    pub cells: usize,
    /// Of those, the passing ones.
    pub passed: usize,
    /// The cases that name this id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<String>,
    /// In scope with no cell: the row is not measured yet.
    pub uncovered: bool,
}

/// The report's `pivot_features` section.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PivotFeatures {
    /// One per catalog row, in catalog order.
    pub rows: Vec<FeatureRow>,
    /// Ids that cases name but the catalog lacks, with the cases naming them.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unknown_ids: BTreeMap<String, Vec<String>>,
}

/// The Excel-evidence pivot cells of one case and how many pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PivotCellTally {
    pub cells: usize,
    pub passed: usize,
}

impl PivotCellTally {
    /// Counts the scored cells of `result` whose `sheet!A1` key is one of the sidecar's pivot-area
    /// keys and whose expectation came from Excel.
    #[must_use]
    pub fn of(result: &CaseResult, pivot_keys: &BTreeSet<String>) -> Self {
        let mut tally = Self::default();
        for c in result.cells.iter().filter(|c| c.from_excel) {
            let key =
                crate::cellref::Addr::parse(&c.cell).map(|a| crate::case::cell_key(&c.sheet, a));
            if key.is_some_and(|k| pivot_keys.contains(&k)) {
                tally.cells += 1;
                tally.passed += usize::from(c.verdict.is_pass());
            }
        }
        tally
    }
}

/// The section for `catalog` over the run's `cases`.
#[must_use]
pub fn summarize(catalog: &[CatalogRow], cases: &[CaseResult]) -> PivotFeatures {
    let mut per_id: BTreeMap<&str, (PivotCellTally, Vec<String>)> = BTreeMap::new();
    for case in cases {
        for id in &case.pivot_features {
            let entry = per_id.entry(id.as_str()).or_default();
            entry.0.cells += case.pivot_cells.cells;
            entry.0.passed += case.pivot_cells.passed;
            entry.1.push(case.id.clone());
        }
    }
    let known: BTreeSet<&str> = catalog.iter().map(|r| r.id.as_str()).collect();
    let unknown_ids = per_id
        .iter()
        .filter(|(id, _)| !known.contains(*id))
        .map(|(id, (_, cases))| ((*id).to_string(), cases.clone()))
        .collect();
    let rows = catalog
        .iter()
        .map(|r| {
            let (tally, cases) = per_id.get(r.id.as_str()).cloned().unwrap_or_default();
            FeatureRow {
                id: r.id.clone(),
                family: r.family.clone(),
                ooxml: r.ooxml.clone(),
                scope: r.scope.clone(),
                stage: r.stage.clone(),
                cells: tally.cells,
                passed: tally.passed,
                cases,
                uncovered: r.in_scope() && tally.cells == 0,
            }
        })
        .collect();
    PivotFeatures { rows, unknown_ids }
}

/// report.md's "Pivot features" section: in-scope rows by stage, every measured row, and the
/// uncovered rows by stage and family (each id is in report.json).
pub fn write_markdown(features: &PivotFeatures, md: &mut String) {
    if features.rows.is_empty() {
        return;
    }
    let in_scope: Vec<&FeatureRow> = features.rows.iter().filter(|r| r.scope == "in").collect();
    let _ = writeln!(
        md,
        "## Pivot features\n\nOne row per in-scope pivot catalog id (T-4126, eval/catalog/pivot-features.tsv): \
         {} of {} measured.\n",
        in_scope.iter().filter(|r| !r.uncovered).count(),
        in_scope.len()
    );
    let _ = writeln!(
        md,
        "A case's pivot cells count toward every id it names, so the stage totals overlap.\n"
    );
    let _ = writeln!(md, "| stage | in scope | measured | cells | passing |");
    let _ = writeln!(md, "|---|---|---|---|---|");
    let mut by_stage: BTreeMap<&str, [usize; 4]> = BTreeMap::new();
    for r in &in_scope {
        let s = by_stage.entry(r.stage.as_str()).or_default();
        s[0] += 1;
        s[1] += usize::from(!r.uncovered);
        s[2] += r.cells;
        s[3] += r.passed;
    }
    for (stage, [n, measured, cells, passed]) in &by_stage {
        let _ = writeln!(md, "| {stage} | {n} | {measured} | {cells} | {passed} |");
    }
    let measured: Vec<&FeatureRow> = features.rows.iter().filter(|r| r.cells > 0).collect();
    if !measured.is_empty() {
        let _ = writeln!(md, "\n| id | family | OOXML | cells | passing | cases |");
        let _ = writeln!(md, "|---|---|---|---|---|---|");
        for r in measured {
            let _ = writeln!(
                md,
                "| {} | {} | `{}` | {} | {} | {} |",
                r.id,
                r.family,
                r.ooxml,
                r.cells,
                r.passed,
                r.cases.join(", ")
            );
        }
    }
    let mut uncovered: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for r in in_scope.iter().filter(|r| r.uncovered) {
        *uncovered
            .entry((r.stage.as_str(), r.family.as_str()))
            .or_default() += 1;
    }
    if !uncovered.is_empty() {
        let list: Vec<String> = uncovered
            .iter()
            .map(|((stage, family), n)| format!("{stage} {family} {n}"))
            .collect();
        let _ = writeln!(
            md,
            "\nUncovered in-scope rows (report.json lists each): {}.",
            list.join(", ")
        );
    }
    if !features.unknown_ids.is_empty() {
        let list: Vec<String> = features
            .unknown_ids
            .iter()
            .map(|(id, cases)| format!("{id} ({})", cases.join(", ")))
            .collect();
        let _ = writeln!(
            md,
            "\nIds cases name that the catalog lacks: {}.",
            list.join(", ")
        );
    }
    let _ = writeln!(md);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::CaseStatus;

    const TSV: &str = "id\tfamily\tooxml\tdefault\taffects\tweb\tscope\tstage\tstatus\tevidence\n\
        PV-0001\tlayout\tpivotTableDefinition@compact\ttrue\tlayout\tunknown\tin\tPV1\ttodo\t\n\
        PV-0002\taggregation\tST_DataConsolidateFunction=sum\t\tvalues\tunknown\tin\tPV1\ttodo\t\n\
        PV-0003\tformatting\tpivotTableStyleInfo@name\t\tnone\tunknown\tround-trip only (decision 0274)\tround-trip\ttodo\t\n";

    fn case(id: &str, features: &[&str], cells: usize, passed: usize) -> CaseResult {
        let mut c = CaseResult::empty(id, "probes", CaseStatus::Fail);
        c.pivot_features = features.iter().map(|s| (*s).to_string()).collect();
        c.pivot_cells = PivotCellTally { cells, passed };
        c
    }

    #[test]
    fn rows_count_the_cells_of_every_case_naming_them() {
        let catalog = parse(TSV).expect("catalog");
        let got = summarize(
            &catalog,
            &[
                case("probes/a", &["PV-0002"], 10, 4),
                case("probes/b", &["PV-0002", "PV-9999"], 5, 5),
            ],
        );
        let sum = &got.rows[1];
        assert_eq!((sum.cells, sum.passed, sum.uncovered), (15, 9, false));
        assert_eq!(sum.cases, vec!["probes/a", "probes/b"]);
        // In scope with no cell: uncovered. Round-trip only: never uncovered.
        assert!(got.rows[0].uncovered);
        assert!(!got.rows[2].uncovered);
        assert_eq!(got.unknown_ids["PV-9999"], vec!["probes/b"]);
    }

    #[test]
    fn the_markdown_lists_stages_measured_rows_and_uncovered_families() {
        let catalog = parse(TSV).expect("catalog");
        let got = summarize(&catalog, &[case("probes/a", &["PV-0002"], 10, 4)]);
        let mut md = String::new();
        write_markdown(&got, &mut md);
        assert!(md.contains("## Pivot features"), "{md}");
        assert!(md.contains("1 of 2 measured"), "{md}");
        assert!(md.contains("| PV1 | 2 | 1 | 10 | 4 |"), "{md}");
        assert!(
            md.contains(
                "| PV-0002 | aggregation | `ST_DataConsolidateFunction=sum` | 10 | 4 | probes/a |"
            ),
            "{md}"
        );
        assert!(
            md.contains("Uncovered in-scope rows (report.json lists each): PV1 layout 1."),
            "{md}"
        );
    }

    #[test]
    fn the_tally_counts_excel_pivot_cells_only() {
        use crate::report::tests::cell;
        use crate::value::Verdict;
        let at = |a1: &str, verdict: Verdict, from_excel: bool| {
            let mut c = cell("", verdict);
            c.cell = a1.into();
            c.from_excel = from_excel;
            c
        };
        let mut result = CaseResult::empty("probes/a", "probes", CaseStatus::Fail);
        result.cells = vec![
            at("A1", Verdict::Pass, true),             // a passing pivot cell
            at("A2", Verdict::NumericMismatch, true),  // a failing pivot cell
            at("A3", Verdict::NumericMismatch, false), // a pivot cell with a hand-written expectation
            at("B9", Verdict::Pass, true),             // not a pivot cell
        ];
        let keys: BTreeSet<String> = ["A1", "A2", "A3"]
            .iter()
            .filter_map(|a| crate::cellref::Addr::parse(a))
            .map(|a| crate::case::cell_key("S", a))
            .collect();
        assert_eq!(
            PivotCellTally::of(&result, &keys),
            PivotCellTally {
                cells: 2,
                passed: 1
            }
        );
    }

    #[test]
    fn a_header_without_the_columns_is_refused() {
        assert!(parse("id\tfamily\n").is_err());
    }

    #[test]
    fn an_absent_catalog_is_empty() {
        let dir = std::env::temp_dir().join("rustsheet-no-pivot-catalog");
        assert!(load(&dir).expect("load").is_empty());
    }
}
