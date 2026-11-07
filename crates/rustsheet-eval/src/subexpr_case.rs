//! `rustsheet-eval subexpr-case`: localize a visibly-off cell whose formula nests several calls
//! (T-3333; docs/architecture/synthetic-generator.md §11 stage 1, §9.3). One probe case holds
//! every subexpression of the formula in a cell of its own over the source case's inputs, so one
//! oracle fill evaluates every node; `--culprit` then names the innermost visibly-off node whose
//! children all match Excel.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use rustsheet_engine::{subexpressions, Subexpression, SubexpressionKind};
use serde_json::{json, Value as Json};

use crate::case::{split_qualified, CaseSpec};
use crate::cellref::Addr;
use crate::closeness::Visible;
use crate::engine::{Engine, RustSheetEngine};
use crate::runner::{run_suites, CellResult, RunOptions};
use crate::suite::{Suite, SuiteKind};
use crate::volatile::is_volatile;

/// The sheet the probe case adds for the subexpression cells.
pub const SHEET: &str = "Subexpr";

/// Why a subexpression gets no cell of its own, or `None` when it gets one.
fn skipped(node: &Subexpression) -> Option<&'static str> {
    match node.kind {
        SubexpressionKind::Literal => Some("literal"),
        SubexpressionKind::Reference => Some("reference"),
        SubexpressionKind::Scoped => Some("reads a LET or LAMBDA name"),
        SubexpressionKind::Positional => Some("depends on its own cell"),
        SubexpressionKind::Computed if is_volatile(&node.text, false) => Some("volatile"),
        SubexpressionKind::Computed => None,
    }
}

/// The subexpressions of the formula `formula` written in `sheet`!`addr` of a workbook with
/// `sheets`.
fn nodes(formula: &str, sheets: &[String], sheet: &str, addr: Addr) -> Vec<Subexpression> {
    let row = i32::try_from(addr.row).unwrap_or(i32::MAX);
    let column = i32::try_from(addr.col).unwrap_or(i32::MAX);
    subexpressions(formula, sheets, sheet, row, column)
}

/// A probe case and where it goes.
#[derive(Debug)]
pub struct ProbeCase {
    /// File stem, `subexpr-<source stem>-<sheet>-<cell>`.
    pub stem: String,
    /// The case JSON.
    pub json: Json,
    /// One line per subexpression: its cell (or why it has none) and its text.
    pub lines: Vec<String>,
}

/// Builds the probe case of `cell` (`Sheet!A1`) of the spec case `source` (its JSON text), whose
/// id is `source_id` and file stem `stem`.
///
/// # Errors
/// When the case does not parse, uses sheet edits, names its own sheet [`SHEET`], or `cell` is
/// not a formula cell of it.
pub fn build(source_id: &str, stem: &str, source: &str, cell: &str) -> Result<ProbeCase> {
    let spec: CaseSpec =
        serde_json::from_str(source).with_context(|| format!("{source_id}: not a spec case"))?;
    let mut json: Json = serde_json::from_str(source)?;
    if !spec.sheet_ops.is_empty() {
        bail!("{source_id}: a case with sheet_ops moves cells after load; not supported");
    }
    let (sheet, address) =
        split_qualified(cell).with_context(|| format!("{cell}: expected Sheet!A1"))?;
    let addr = Addr::parse(&address).with_context(|| format!("{cell}: not a cell address"))?;
    let address = addr.to_string();
    let Some(sheet_spec) = spec
        .sheets
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(&sheet))
    else {
        bail!("{source_id}: no sheet {sheet}");
    };
    let Some(formula) = sheet_spec
        .cells
        .get(&address)
        .and_then(crate::case::CellInput::formula)
    else {
        bail!("{source_id}: {cell} holds no formula");
    };
    if spec
        .sheets
        .iter()
        .any(|s| s.name.eq_ignore_ascii_case(SHEET))
    {
        bail!("{source_id}: already has a sheet named {SHEET}");
    }
    let legacy = sheet_spec.legacy_formulas.contains(&address);
    let mut names: Vec<String> = spec.sheets.iter().map(|s| s.name.clone()).collect();
    names.push(SHEET.to_string());
    let tree = nodes(formula, &names, &sheet_spec.name, addr);

    // One cell per distinct checkable subexpression, children first; the whole formula last,
    // whatever it is.
    let mut cells: BTreeMap<String, Json> = BTreeMap::new();
    let mut check = Vec::new();
    let mut lines = Vec::new();
    let mut seen = BTreeSet::new();
    let last = tree.len().saturating_sub(1);
    for (i, node) in tree.iter().enumerate() {
        let reason = if i == last { None } else { skipped(node) };
        if let Some(reason) = reason {
            lines.push(format!("skipped ({reason}): {}", node.text));
            continue;
        }
        if !seen.insert(node.text.clone()) {
            continue;
        }
        let at = format!("A{}", check.len() + 1);
        lines.push(format!("{SHEET}!{at}: ={}", node.text));
        cells.insert(at.clone(), Json::String(format!("={}", node.text)));
        check.push(format!("{SHEET}!{at}"));
    }

    let milestones: Vec<String> = spec
        .tags
        .iter()
        .filter(|t| is_milestone_tag(t))
        .cloned()
        .collect();
    let mut tags = milestones;
    tags.extend(["probe".to_string(), "subexpr".to_string()]);
    let mut sheet_json = json!({ "name": SHEET, "cells": cells });
    if legacy {
        sheet_json["legacy_formulas"] = json!(cells.keys().collect::<Vec<_>>());
    }
    let description = format!(
        "T-3333 subexpr-case of {source_id} {}!{address}: each cell of {SHEET} holds one \
         subexpression of its formula, children first, the whole formula last, every reference \
         absolute and sheet-qualified; `rustsheet-eval subexpr-case --culprit` names the innermost \
         visibly-off one. Cells: {}",
        sheet_spec.name,
        lines.join("; ")
    );
    let Some(object) = json.as_object_mut() else {
        bail!("{source_id}: not a JSON object");
    };
    object.insert("description".into(), Json::String(description));
    object.insert("tags".into(), json!(tags));
    object.insert("expected".into(), json!({}));
    object.insert("check".into(), json!(check));
    for key in ["expected_source", "invariant", "text"] {
        object.remove(key);
    }
    if let Some(sheets) = object.get_mut("sheets").and_then(Json::as_array_mut) {
        sheets.push(sheet_json);
    }
    Ok(ProbeCase {
        stem: format!(
            "subexpr-{stem}-{}-{}",
            sheet_spec.name.to_ascii_lowercase(),
            address.to_ascii_lowercase()
        ),
        json,
        lines,
    })
}

/// `M9`, `M8a`: a milestone tag (decision 0051), which the probe case keeps so it scores in its
/// source case's milestone.
fn is_milestone_tag(tag: &str) -> bool {
    let Some(rest) = tag.strip_prefix('M') else {
        return false;
    };
    let digits = rest.trim_end_matches(|c: char| c.is_ascii_lowercase());
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// The innermost visibly-off subexpressions of the probe case `probe` (its JSON text), given the
/// scored `cells` of its run: each off node (decision 0083's rule, [`Visible::is_off`]) whose
/// checked children all match. Empty when nothing is off.
///
/// # Errors
/// When the case does not parse or has no [`SHEET`] cells.
pub fn culprits(probe: &str, cells: &[CellResult]) -> Result<Vec<String>> {
    let spec: CaseSpec = serde_json::from_str(probe).context("not a spec case")?;
    let Some(sheet) = spec.sheets.iter().find(|s| s.name == SHEET) else {
        bail!("not a subexpr case: no {SHEET} sheet");
    };
    let by_text: BTreeMap<&str, &str> = sheet
        .cells
        .iter()
        .filter_map(|(at, input)| input.formula().map(|f| (f, at.as_str())))
        .collect();
    let last = format!("A{}", sheet.cells.len());
    let Some(root) = sheet
        .cells
        .get(&last)
        .and_then(crate::case::CellInput::formula)
    else {
        bail!("not a subexpr case: {SHEET}!{last} holds no formula");
    };
    let names: Vec<String> = spec.sheets.iter().map(|s| s.name.clone()).collect();
    let tree = nodes(root, &names, SHEET, Addr { row: 1, col: 1 });
    let off = |text: &str| -> bool {
        by_text.get(text).is_some_and(|at| {
            let key = format!("{SHEET}!{at}");
            cells.iter().any(|c| c.key() == key && Visible::is_off(c))
        })
    };
    Ok(tree
        .iter()
        .filter(|node| off(&node.text) && !node.children.iter().any(|&c| off(&tree[c].text)))
        .map(|node| {
            let at = by_text.get(node.text.as_str()).copied().unwrap_or_default();
            let cell = cells.iter().find(|c| c.key() == format!("{SHEET}!{at}"));
            match cell {
                Some(c) => format!(
                    "{SHEET}!{at}: ={}\n    expected: {}\n    actual  : {}",
                    node.text, c.expected, c.actual
                ),
                None => format!("{SHEET}!{at}: ={}", node.text),
            }
        })
        .collect())
}

/// The `subexpr-case` command's arguments.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// The source spec case (`gen-m9/<stem>.case.json`), or with `--culprit` the probe case.
    pub case_id: String,
    /// The formula cell, `Sheet!A1` (not with `--culprit`).
    pub cell: Option<String>,
    /// Where to write the case (default: eval/suites/probes/cases/subexpr-<stem>-<sheet>-<cell>).
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Run the probe case against its oracle sidecar and print its culprits instead.
    #[arg(long)]
    pub culprit: bool,
}

/// The `subexpr-case` command: writes the probe case of `cell` of the spec case `case_id`, or with
/// `--culprit` runs the probe case `case_id`; returns the text to print.
///
/// # Errors
/// When the case is not found or not a spec case, or [`build`] or [`culprits`] refuses it.
pub fn run(root: &Path, args: Args) -> Result<String> {
    let Args {
        case_id,
        cell,
        out,
        culprit,
    } = args;
    let case_id = case_id.as_str();
    let suite_name = case_id.split('/').next().unwrap_or_default();
    let suite = Suite::load_named(root, suite_name)?;
    if suite.manifest.kind != SuiteKind::Spec {
        bail!("{case_id}: {suite_name} is not a spec suite (an xlsx case has no spec to copy)");
    }
    let Some(case) = suite.cases()?.into_iter().find(|c| c.id == case_id) else {
        bail!("case {case_id} not found");
    };
    let text = std::fs::read_to_string(&case.path).with_context(|| case_id.to_string())?;
    if culprit {
        let engine: Arc<dyn Engine + Send> = Arc::new(RustSheetEngine);
        let opts = RunOptions {
            timeout: Duration::from_secs(120),
            filter: Some(case_id.to_string()),
        };
        let results = run_suites(&engine, &[suite], &opts);
        let Some(result) = results.iter().find(|r| r.id == case_id) else {
            bail!("case {case_id} did not run");
        };
        let found = culprits(&text, &result.cells)?;
        if found.is_empty() {
            return Ok("no divergent node\n".to_string());
        }
        let mut printed = found.join("\n");
        printed.push('\n');
        return Ok(printed);
    }
    let Some(cell) = cell.as_deref() else {
        bail!("subexpr-case needs a cell (Sheet!A1) unless --culprit");
    };
    let stem = case
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .map_or("case", |n| n.trim_end_matches(".case.json"));
    let probe = build(case_id, stem, &text, cell)?;
    let path = out.unwrap_or_else(|| {
        root.join("eval/suites/probes/cases")
            .join(format!("{}.case.json", probe.stem))
    });
    let mut json = serde_json::to_string_pretty(&probe.json)?;
    json.push('\n');
    std::fs::write(&path, json).with_context(|| path.display().to_string())?;
    let mut printed = format!("wrote {}\n", path.display());
    for line in &probe.lines {
        printed.push_str(line);
        printed.push('\n');
    }
    Ok(printed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Value, Verdict};

    const SOURCE: &str = r#"{
        "format": "rustsheet-case/v1",
        "description": "src",
        "tags": ["generated", "M9", "SQRT"],
        "sheets": [
            {"name": "Data", "cells": {"A1": 4, "B2": 9}},
            {"name": "Calc", "cells": {
                "C5": "=ROUND(SQRT(Data!A1)+ABS(B6),2)+SUM($D$1:D2)",
                "C6": "=LET(x,SQRT(4),x+1)*RAND()+ROW()",
                "C7": 3
            }}
        ],
        "expected": {},
        "check": ["Calc!C5"]
    }"#;

    fn probe(cell: &str) -> ProbeCase {
        build("gen-m9/abc", "abc", SOURCE, cell).unwrap()
    }

    fn subexpr_cells(case: &ProbeCase) -> Vec<String> {
        let sheets = case.json["sheets"].as_array().unwrap();
        let sheet = sheets.iter().find(|s| s["name"] == SHEET).unwrap();
        (1..)
            .map_while(|i| sheet["cells"][format!("A{i}")].as_str().map(str::to_string))
            .collect()
    }

    #[test]
    fn relative_and_absolute_references_keep_their_targets() {
        let case = probe("Calc!C5");
        assert_eq!(case.stem, "subexpr-abc-calc-c5");
        let cells = subexpr_cells(&case);
        // B6 and D2 written in C5 are Calc!$B$6 and Calc!$D$2 from Subexpr; children first.
        assert!(cells.contains(&"=SQRT(Data!$A$1)".to_string()), "{cells:?}");
        assert!(cells.contains(&"=ABS(Calc!$B$6)".to_string()), "{cells:?}");
        assert!(
            cells.contains(&"=SUM(Calc!$D$1:$D$2)".to_string()),
            "{cells:?}"
        );
        assert_eq!(
            cells.last().map(String::as_str),
            Some("=ROUND(SQRT(Data!$A$1)+ABS(Calc!$B$6),2)+SUM(Calc!$D$1:$D$2)")
        );
        let pos = |t: &str| cells.iter().position(|c| c == t).unwrap();
        assert!(pos("=SQRT(Data!$A$1)") < pos("=SQRT(Data!$A$1)+ABS(Calc!$B$6)"));
        // The source's inputs, the milestone tag and a check list of the new cells.
        assert_eq!(case.json["sheets"][0]["cells"]["A1"], 4);
        assert_eq!(case.json["tags"], json!(["M9", "probe", "subexpr"]));
        assert_eq!(case.json["check"].as_array().unwrap().len(), cells.len());
        assert_eq!(case.json["expected"], json!({}));
    }

    #[test]
    fn let_lambda_volatile_and_positional_nodes_are_skipped() {
        let case = probe("Calc!C6");
        let cells = subexpr_cells(&case);
        assert_eq!(
            cells,
            [
                "=SQRT(4)".to_string(),
                "=LET(x,SQRT(4),x+1)*RAND()+ROW()".to_string()
            ]
        );
        let lines = case.lines.join("\n");
        for skipped in [
            "(volatile): RAND()",
            "(depends on its own cell): ROW()",
            "(reads a LET or LAMBDA name): x+1",
        ] {
            assert!(lines.contains(skipped), "{lines}");
        }
    }

    #[test]
    fn a_cell_without_a_formula_or_an_unknown_sheet_is_refused() {
        for cell in ["Calc!C7", "Calc!Z9", "Nope!A1", "A1"] {
            assert!(build("gen-m9/abc", "abc", SOURCE, cell).is_err(), "{cell}");
        }
    }

    fn results(case: &ProbeCase, off: &[&str]) -> Vec<CellResult> {
        let cells = subexpr_cells(case);
        cells
            .iter()
            .enumerate()
            .map(|(i, formula)| {
                let at = format!("A{}", i + 1);
                let is_off = off.iter().any(|o| formula == &format!("={o}"));
                let mut cell = CellResult::for_test(
                    &at,
                    Some(&formula[1..]),
                    if is_off {
                        Verdict::NumericMismatch
                    } else {
                        Verdict::Pass
                    },
                );
                cell.sheet = SHEET.to_string();
                cell.expected = Value::Number(1.0);
                cell.actual = Value::Number(if is_off { 1.001 } else { 1.0 });
                cell
            })
            .collect()
    }

    #[test]
    fn the_culprit_is_the_off_node_whose_children_match() {
        let case = probe("Calc!C5");
        let text = serde_json::to_string(&case.json).unwrap();
        // SQRT is off, and so is everything above it; ABS and SUM match.
        let off = [
            "SQRT(Data!$A$1)",
            "SQRT(Data!$A$1)+ABS(Calc!$B$6)",
            "ROUND(SQRT(Data!$A$1)+ABS(Calc!$B$6),2)",
            "ROUND(SQRT(Data!$A$1)+ABS(Calc!$B$6),2)+SUM(Calc!$D$1:$D$2)",
        ];
        let got = culprits(&text, &results(&case, &off)).unwrap();
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].contains("=SQRT(Data!$A$1)\n"), "{got:?}");
        assert!(culprits(&text, &results(&case, &[])).unwrap().is_empty());
    }
}
