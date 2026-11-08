use super::*;
use crate::catalog::{CatalogFunction, Scope};
use crate::milestone_lists::ListedCase;
use crate::runner::CaseStatus;
use crate::value::Value;

#[test]
fn parses_case_and_cell_lines_and_skips_comments() {
    let mapping = Mapping::parse(
        "# header\n\nprobes/a.case.json\tM9\tGEOMEAN\nprobes/b.case.json\tSheet1!$H$28\tM9\tGEOMEAN\n",
    )
    .expect("parse");
    assert_eq!(
        mapping.cases.get("probes/a.case.json"),
        Some(&Assigned {
            line: 3,
            milestone: 9
        })
    );
    assert_eq!(
        mapping
            .cells
            .get(&("probes/b.case.json".to_string(), "Sheet1!H28".to_string())),
        Some(&Assigned {
            line: 4,
            milestone: 9
        })
    );
}

#[test]
fn rejects_duplicates_and_bad_milestones() {
    assert_eq!(
        Mapping::parse("probes/a.case.json\tM9\tx\nprobes/a.case.json\tM6\ty\n"),
        Err(MappingError::Duplicate {
            line: 2,
            item: "probes/a.case.json".into(),
            first: 1
        })
    );
    let dup_cell = "probes/a.case.json\tS!A1\tM9\tx\nprobes/a.case.json\tS!$A$1\tM6\ty\n";
    assert!(matches!(
        Mapping::parse(dup_cell),
        Err(MappingError::Duplicate {
            line: 2,
            first: 1,
            ..
        })
    ));
    for bad in ["M", "9", "m9", "M9a", "M-1", "Mx"] {
        let err = Mapping::parse(&format!("probes/a.case.json\t{bad}\tx\n")).expect_err(bad);
        assert_eq!(
            err.to_string(),
            format!("{MAPPING_PATH} line 1: the milestone must be `M<digits>`"),
            "{bad}"
        );
    }
    for bad in [
        "probes/a.case.json\tM9\n",
        "a.case.json\tM9\tx\n",
        "probes/a.case.json\tA1\tM9\tx\n",
        "probes/a.case.json\tM9\t \n",
    ] {
        assert!(
            matches!(Mapping::parse(bad), Err(MappingError::Line { line: 1, .. })),
            "{bad}"
        );
    }
}

#[test]
fn an_unknown_case_names_its_line() {
    let mapping =
        Mapping::parse("# c\nprobes/a.case.json\tM9\tx\nprobes/gone.case.json\tS!A1\tM9\tx\n")
            .expect("parse");
    let known: BTreeSet<String> = ["probes/a.case.json".to_string()].into();
    assert_eq!(
        mapping
            .check_known(&known)
            .expect_err("unknown")
            .to_string(),
        format!("{MAPPING_PATH} line 3: no case probes/gone.case.json")
    );
    let known: BTreeSet<String> = ["probes/a.case.json", "probes/gone.case.json"]
        .map(String::from)
        .into();
    assert_eq!(mapping.check_known(&known), Ok(()));
}

#[test]
fn an_unchecked_cell_names_its_line() {
    let mapping = Mapping::parse(
        "# c\nprobes/a.case.json\tS!A1\tM9\tx\nprobes/a.case.json\tS!B2\tM9\tx\nprobes/b.case.json\tS!Z9\tM9\tx\n",
    )
    .expect("parse");
    let cells = |keys: &[&str]| keys.iter().map(ToString::to_string).collect();
    let mut expectations = BTreeMap::from([("probes/a.case.json".to_string(), cells(&["S!A1"]))]);
    assert_eq!(
        mapping
            .check_cells(&expectations)
            .expect_err("unchecked")
            .to_string(),
        format!("{MAPPING_PATH} line 3: probes/a.case.json has no checked cell S!B2")
    );
    // A case the expectations do not list (probes/b) is left to `check_known`.
    expectations.insert("probes/a.case.json".to_string(), cells(&["S!A1", "S!B2"]));
    assert_eq!(mapping.check_cells(&expectations), Ok(()));
}

#[test]
fn the_committed_mapping_parses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mapping = Mapping::load(&root).unwrap_or_else(|e| panic!("{e}"));
    assert!(mapping.cases.len() >= 100);
    assert_eq!(
        mapping
            .cells
            .get(&(
                "probes/subtotal-reference-operands-d.case.json".to_string(),
                "Sheet1!H28".to_string()
            ))
            .map(|a| a.milestone),
        Some(9)
    );
}

fn cell(key: &str, formula: &str, verdict: Verdict) -> CellResult {
    let (sheet, a1) = key.split_once('!').unwrap_or(("S", key));
    CellResult {
        sheet: sheet.into(),
        cell: a1.into(),
        formula: Some(formula.into()),
        anchor_formula: None,
        expected: Value::Number(1.0),
        actual: Value::Number(1.0),
        verdict,
        from_excel: true,
        listed_deviation: verdict == Verdict::Deviation,
        graph_subnormal: false,
        kind: CellKind::Scored,
        math: None,
        target: crate::runner::Target::Excel,
    }
}

fn case(id: &str, tags: &[&str], status: CaseStatus, cells: Vec<CellResult>) -> CaseResult {
    CaseResult {
        id: id.into(),
        suite: SUITE.into(),
        status,
        message: None,
        expected_source: String::new(),
        checked: cells.len(),
        passed: cells.iter().filter(|c| c.verdict.is_pass()).count(),
        deviations: cells
            .iter()
            .filter(|c| c.verdict == Verdict::Deviation)
            .count(),
        skipped_volatile: 0,
        skipped_add_in: 0,
        millis: 0,
        tags: tags.iter().map(|t| (*t).to_string()).collect(),
        cells,
        text: Vec::new(),
        defined_names: BTreeMap::new(),
        arguments: Vec::new(),
        reference_forms: crate::poi_m8_subset::ReferenceFormCells::default(),
        pivot_layout: Vec::new(),
        layout_checked: None,
        pivot_freshness: Vec::new(),
        pivot_features: Vec::new(),
        pivot_cells: crate::pivot_features::PivotCellTally::default(),
    }
}

fn rules(mapping: &str) -> Rules {
    let frozen = MilestoneList {
        name: FROZEN_M1_LIST.into(),
        cases: vec![ListedCase {
            line: 1,
            id: "probes/frozen.case.json".into(),
        }],
    };
    let catalog = Catalog::from_functions(
        [
            ("GEOMEAN", "M9"),
            ("XLOOKUP", "M6"),
            ("SUM", "M2"),
            ("OR", "M2"),
        ]
        .map(|(name, milestone)| CatalogFunction {
            usage: crate::catalog::Usage::default(),
            name: name.into(),
            category: "math".into(),
            milestone: milestone.into(),
            scope: Scope::In,
            reason: None,
            oracle: None,
            v15: None,
            v15_reason: None,
            returns: None,
            params: Vec::new(),
        }),
    );
    Rules::new(
        &[frozen],
        Mapping::parse(mapping).expect("mapping"),
        &catalog,
    )
}

fn pass(key: &str, formula: &str) -> CellResult {
    cell(key, formula, Verdict::Pass)
}

#[test]
fn rule_1_a_frozen_case_stays_m1_even_when_listed_or_tagged_later() {
    let rules = rules("probes/frozen.case.json\tM9\tx\n");
    let r = case(
        "probes/frozen.case.json",
        &["M7"],
        CaseStatus::Pass,
        vec![pass("S!A1", "GEOMEAN(1)")],
    );
    assert_eq!(rules.case_milestone(&r), 1);
    assert_eq!(rules.cell_milestone(&r, &r.cells[0]), 1);
}

#[test]
fn rule_2_a_line_beats_the_tag_and_a_cell_line_beats_the_case() {
    let rules = rules("probes/a.case.json\tM9\tx\nprobes/b.case.json\tS!A2\tM9\ty\n");
    let a = case("probes/a.case.json", &["M1"], CaseStatus::Pass, vec![]);
    assert_eq!(rules.case_milestone(&a), 9);
    let b = case(
        "probes/b.case.json",
        &["M2"],
        CaseStatus::Pass,
        vec![pass("S!A1", "SUM(1)"), pass("S!A2", "GEOMEAN(5,7)")],
    );
    assert_eq!(rules.cell_milestone(&b, &b.cells[0]), 2);
    assert_eq!(rules.cell_milestone(&b, &b.cells[1]), 9);
}

#[test]
fn rule_3_a_catalog_cell_counts_at_its_functions_milestone() {
    let rules = rules("");
    let r = case(
        "probes/catalog/availability-01.case.json",
        &["M0"],
        CaseStatus::Pass,
        vec![
            pass("S!A1", "OR(ISERROR(_xlfn.XLOOKUP(1,A:A,B:B)))"),
            pass("S!A2", "IFERROR(GEOMEAN(1),\"SUM(\")"),
            pass("S!A3", "ERROR.TYPE(#N/A)"),
            pass("S!A4", "1+1"),
            pass("S!A5", "OR(SUM(1),GEOMEAN(2))"),
        ],
    );
    let got: Vec<u32> = r
        .cells
        .iter()
        .map(|c| rules.cell_milestone(&r, c))
        .collect();
    assert_eq!(got, [6, 9, 2, 2, 9]);
}

#[test]
fn rules_4_and_5_the_highest_tag_else_m2() {
    let rules = rules("");
    let tagged = case(
        "probes/t.case.json",
        &["lookup", "M6", "M9"],
        CaseStatus::Pass,
        vec![],
    );
    assert_eq!(rules.case_milestone(&tagged), 9);
    let untagged = case("probes/u.case.json", &["lookup"], CaseStatus::Pass, vec![]);
    assert_eq!(rules.case_milestone(&untagged), 2);
}

/// A row's tier counts: `foundational` and `expert` cells, `wrong_type` of the expert ones.
fn tiers(foundational: usize, expert: usize, wrong_type: usize) -> TierTallies {
    TierTallies {
        foundational: crate::usage_gates::Tally {
            cells: foundational,
            ..Default::default()
        },
        expert: crate::usage_gates::Tally {
            cells: expert,
            wrong_type,
            outside: 0,
        },
    }
}

#[test]
fn scores_rows_as_the_baseline_counts_them() {
    let rules = rules("probes/d.case.json\tS!A2\tM9\tGEOMEAN\n");
    let results = [
        // M2 cells and one M9 cell override; both failures count where their cell does.
        case(
            "probes/d.case.json",
            &["M2"],
            CaseStatus::Fail,
            vec![
                cell("S!A1", "SUM(1)", Verdict::NumericMismatch),
                cell("S!A2", "GEOMEAN(5,7)", Verdict::NumericMismatch),
                cell("S!A3", "SUM(2)", Verdict::Deviation),
            ],
        ),
        // A whole-case failure counts at the case's milestone.
        case("probes/p.case.json", &["M7"], CaseStatus::Panic, vec![]),
        case(
            "probes/catalog/c.case.json",
            &["M0"],
            CaseStatus::Fail,
            vec![
                pass("S!A1", "XLOOKUP(1,A1,B1)"),
                cell("S!A2", "GEOMEAN(1)", Verdict::TypeMismatch),
            ],
        ),
        case(
            "probes/m1.case.json",
            &["M1"],
            CaseStatus::Pass,
            vec![pass("S!A1", "1")],
        ),
        case(
            "probes/m12.case.json",
            &["M12"],
            CaseStatus::Pending,
            vec![],
        ),
        // Another suite is ignored.
        CaseResult {
            suite: "other".into(),
            ..case("other/x.case.json", &["M5"], CaseStatus::Panic, vec![])
        },
    ];
    let rows = summarize(&rules, &results, BaselineMode::Cells, &R1cCells::default());
    let row =
        |m: &str, cases, cells, bit_exact, deviations, failing: &[(&str, usize)]| MilestoneRow {
            milestone: m.into(),
            cases,
            cells,
            bit_exact,
            deviations,
            known_failures: failing.iter().map(|f| f.1).sum(),
            failing_cases: failing.iter().map(|f| (f.0.to_owned(), f.1)).collect(),
            ..MilestoneRow::default()
        };
    // The test catalog's functions are all expert: SUM(1) is an expert cell, `1` (no call)
    // a foundational one, and the deviation counts in neither tier.
    assert_eq!(
        rows,
        [
            MilestoneRow {
                tiers: tiers(1, 1, 0),
                ..row("M0-M2", 2, 3, 1, 1, &[("probes/d.case.json", 1)])
            },
            row("M5", 0, 0, 0, 0, &[]),
            MilestoneRow {
                tiers: tiers(0, 1, 0),
                ..row("M6", 1, 1, 1, 0, &[])
            },
            // A panicked case did not run (decision 0174); the pending M12 case did.
            MilestoneRow {
                blocked_cases: vec!["probes/p.case.json!*".into()],
                ..row("M7", 1, 0, 0, 0, &[("probes/p.case.json", 1)])
            },
            row("M8", 0, 0, 0, 0, &[]),
            MilestoneRow {
                visibly_off: 1,
                visibly_off_cells: vec!["probes/catalog/c.case.json!S!A2".into()],
                // An unclassified visibly-off cell counts in R1(c) (decision 0188).
                r1c_off: 1,
                r1c_off_cells: vec!["probes/catalog/c.case.json!S!A2".into()],
                // GEOMEAN's type mismatch is the R1(c) miss; the other GEOMEAN cell is within
                // 15 digits.
                tiers: tiers(0, 2, 1),
                ..row(
                    "M9",
                    2,
                    2,
                    0,
                    0,
                    &[("probes/catalog/c.case.json", 1), ("probes/d.case.json", 1)],
                )
            },
            row("M10", 0, 0, 0, 0, &[]),
            row("M11", 0, 0, 0, 0, &[]),
            row("M12", 1, 0, 0, 0, &[]),
        ]
    );
    let mut md = String::new();
    write_markdown(&rows, &mut md);
    assert!(md.contains("## Probe milestones (decision 0051)"), "{md}");
    assert!(
        md.contains("| M0-M2 | 2 | 3 | 1 | 1 | 1 | 0 | 0 | 0 |"),
        "{md}"
    );
    assert!(
        md.contains("| M9 | 2 | 2 | 0 | 0 | 2 | 1 | 0 | 0 |"),
        "{md}"
    );
    assert!(
        md.contains("Failing cases in M9:\n\n- `probes/catalog/c.case.json`: 1\n- `probes/d.case.json`: 1\n"),
        "{md}"
    );
    assert!(summarize(
        &rules,
        &results[5..],
        BaselineMode::Cells,
        &R1cCells::default()
    )
    .is_empty());
}

#[test]
fn text_items_are_counted_in_their_cases_row() {
    let rules = rules("");
    let text = |cell: &str, pass| crate::text_expect::TextResult {
        sheet: "S".into(),
        cell: cell.into(),
        expected: "1".into(),
        actual: Some("1".into()),
        pass,
    };
    let with_text = CaseResult {
        text: vec![text("A1", true), text("A2", false)],
        ..case("probes/t.case.json", &["M9"], CaseStatus::Pass, vec![])
    };
    let plain = case("probes/n.case.json", &["M5"], CaseStatus::Pass, vec![]);
    let rows = summarize(
        &rules,
        &[with_text, plain],
        BaselineMode::Cells,
        &R1cCells::default(),
    );
    let find = |m: &str| rows.iter().find(|r| r.milestone == m).expect(m);
    let m9 = find("M9");
    assert_eq!((m9.text_items, m9.text_failing), (2, 1));
    assert_eq!(m9.text_failing_cells, ["probes/t.case.json!S!A2"]);
    let m5 = find("M5");
    assert_eq!((m5.text_items, m5.text_failing), (0, 0));
    assert!(m5.text_failing_cells.is_empty());
    let mut md = String::new();
    write_markdown(&rows, &mut md);
    assert!(
        md.contains("| M9 | 1 | 0 | 0 | 0 | 0 | 0 | 2 | 1 |"),
        "{md}"
    );
}

#[test]
fn spill_extent_cells_are_counted_in_their_row() {
    let rules = rules("");
    let extent = CellResult {
        formula: None,
        expected: Value::Empty,
        kind: CellKind::SpillExtent,
        ..cell("S!B1", "", Verdict::TypeMismatch)
    };
    let results = [case(
        "probes/s.case.json",
        &["M7"],
        CaseStatus::Fail,
        vec![pass("S!A1", "SEQUENCE(1)"), extent],
    )];
    let rows = summarize(&rules, &results, BaselineMode::Cells, &R1cCells::default());
    let m7 = rows.iter().find(|r| r.milestone == "M7").expect("M7 row");
    assert_eq!((m7.cells, m7.spill_extent_failures), (2, 1));
    assert!(rows
        .iter()
        .filter(|r| r.milestone != "M7")
        .all(|r| r.spill_extent_failures == 0));
}

mod r1c;
