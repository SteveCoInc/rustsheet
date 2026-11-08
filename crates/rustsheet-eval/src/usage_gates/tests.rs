use super::*;
use crate::catalog::{CatalogFunction, Scope};
use crate::runner::CellResult;
use crate::value::Value;

/// A catalog of in-scope functions with their tiers.
fn catalog(functions: &[(&str, Usage)]) -> Catalog {
    Catalog::from_functions(functions.iter().map(|(name, usage)| CatalogFunction {
        usage: *usage,
        name: (*name).into(),
        category: "math".into(),
        milestone: "M5".into(),
        scope: Scope::In,
        reason: None,
        oracle: None,
        v15: None,
        v15_reason: None,
        returns: None,
        params: Vec::new(),
    }))
}

/// An Excel cell `formula` scoring `engine` against Excel's `excel`.
fn cell(n: usize, formula: &str, excel: Value, engine: Value) -> CellResult {
    let mut c = CellResult::for_test(
        &format!("A{n}"),
        Some(formula),
        crate::value::compare(&excel, &engine),
    );
    c.expected = excel;
    c.actual = engine;
    c
}

/// `right` passing cells, `wrong` cells with a wrong type and `outside` numbers 1e-6 off, all
/// calling `formula`, numbered from `from`.
fn cells(
    formula: &str,
    from: usize,
    right: usize,
    wrong: usize,
    outside: usize,
) -> Vec<CellResult> {
    let num = Value::Number(1.0);
    let mut out = Vec::new();
    for i in 0..right {
        out.push(cell(from + i, formula, num.clone(), num.clone()));
    }
    for i in 0..wrong {
        let error = Value::Error("#NUM!".into());
        out.push(cell(from + right + i, formula, num.clone(), error));
    }
    for i in 0..outside {
        let off = Value::Number(1.0 + 1e-6);
        out.push(cell(from + right + wrong + i, formula, num.clone(), off));
    }
    out
}

fn case(id: &str, suite: &str, cells: Vec<CellResult>) -> CaseResult {
    CaseResult {
        cells,
        ..CaseResult::for_test(id, suite)
    }
}

fn function<'a>(gates: &'a UsageGates, name: &str) -> &'a FunctionGate {
    gates
        .functions
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name}"))
}

fn tier(gates: &UsageGates, usage: Usage) -> &TierGate {
    gates
        .tiers
        .iter()
        .find(|t| t.usage == usage)
        .unwrap_or_else(|| panic!("{usage:?}"))
}

#[test]
fn the_counted_suites() {
    for suite in [
        "probes",
        "probes-numeric",
        "gen-m1",
        "ironcalc-calc",
        "corpus-bench",
        "calipers",
        "poi",
        "smoke",
        "fixtures",
    ] {
        assert!(counts(suite), "{suite}");
    }
    for suite in [
        "gen-pivot",
        "pivot-corpus-lo",
        "pivot-corpus-closedxml",
        "pivot-export",
        "accuracy",
        "other",
    ] {
        assert!(!counts(suite), "{suite}");
    }
}

#[test]
fn an_expert_function_at_exactly_99_percent_meets_its_bar() {
    let bars = Bars::per_function(Usage::Expert);
    let at = Tally {
        cells: 100,
        wrong_type: 1,
        outside: 1,
    };
    assert!(at.meets(bars));
    assert!(!Tally {
        wrong_type: 2,
        ..at
    }
    .meets(bars));
    assert!(!Tally { outside: 2, ..at }.meets(bars));
    assert!(Tally::default().meets(bars), "no cell meets");
}

#[test]
fn the_expert_tier_at_exactly_99_9_percent_meets_its_pooled_bar() {
    let bars = Bars::pooled(Usage::Expert);
    let at = Tally {
        cells: 1000,
        wrong_type: 1,
        outside: 1,
    };
    assert!(at.meets(bars));
    assert!(!Tally {
        wrong_type: 2,
        ..at
    }
    .meets(bars));
    assert!(!Tally { outside: 2, ..at }.meets(bars));
}

#[test]
fn the_foundational_bars_are_every_type_and_99_99_percent_pooled() {
    let pooled = Bars::pooled(Usage::Foundational);
    let at = Tally {
        cells: 10_000,
        wrong_type: 0,
        outside: 1,
    };
    assert!(at.meets(pooled));
    assert!(!Tally { outside: 2, ..at }.meets(pooled));
    assert!(!Tally {
        wrong_type: 1,
        ..at
    }
    .meets(pooled));
    // Per function, no envelope bar: a function of 10 cells, 9 outside, meets it.
    let per = Bars::per_function(Usage::Foundational);
    assert!(Tally {
        cells: 10,
        wrong_type: 0,
        outside: 9
    }
    .meets(per));
    assert!(!Tally {
        cells: 10,
        wrong_type: 1,
        outside: 0
    }
    .meets(per));
    assert_eq!(
        Bar::from(pooled),
        Bar {
            type_share: 1.0,
            envelope: 0.9999
        }
    );
    assert!(Bar::from(per).envelope.abs() < f64::EPSILON);
}

#[test]
fn tiers_functions_and_bars_from_a_run() {
    let catalog = catalog(&[
        ("SUM", Usage::Foundational),
        ("ABS", Usage::Foundational),
        ("IRR", Usage::Expert),
        ("RATE", Usage::Expert),
        ("GETPIVOTDATA", Usage::Expert),
        ("BESSELJ", Usage::Expert),
    ]);
    let mut probes = cells("ABS(1)", 1, 30, 0, 1);
    // IRR: 100 cells at exactly 99%; RATE: 98 of 100 right.
    probes.extend(cells("IRR(A1:A3)", 100, 99, 1, 0));
    probes.extend(cells("RATE(1,2,3)", 300, 98, 0, 2));
    let mut left_out = cells("GETPIVOTDATA(\"x\",A1)+ABS(1)", 500, 0, 5, 0);
    // A deviation, a hand-written expectation and a listed exception count nowhere.
    let mut deviation = cell(600, "ABS(2)", Value::Number(1.0), Value::Number(2.0));
    deviation.verdict = Verdict::Deviation;
    let mut hand = cell(
        601,
        "ABS(3)",
        Value::Number(1.0),
        Value::Error("#NUM!".into()),
    );
    hand.from_excel = false;
    let listed = cell(
        602,
        "ABS(4)",
        Value::Number(1.0),
        Value::Error("#NUM!".into()),
    );
    left_out.extend([deviation, hand, listed]);
    let mut exceptions = Phase1Exceptions::default();
    exceptions.cells.insert(
        ("probes/p.case.json".into(), "S!A602".into()),
        crate::phase1_exceptions::Phase1Exception {
            line: 1,
            decision: "0134".into(),
            task: "T-1".into(),
        },
    );
    probes.extend(left_out);
    let results = [
        case("probes/p.case.json", "probes", probes),
        // A pivot suite is left out whatever its cells say.
        case(
            "gen-pivot/g.case.json",
            "gen-pivot",
            cells("SUM(1)", 1, 0, 3, 0),
        ),
        // SUM below the evidence floor of attributed cells: counted by every cell calling it.
        case(
            "gen-m5/g.case.json",
            "gen-m5",
            cells("SUM(ABS(1),IRR(A1:A2))", 1, 9, 1, 0),
        ),
    ];
    let gates = summarize(&catalog, &results, &exceptions);
    assert_eq!(gates.suites, ["gen-m5", "probes"]);
    let irr = function(&gates, "IRR");
    assert_eq!((irr.cells, irr.wrong_type, irr.outside), (100, 1, 0));
    assert!(irr.meets);
    assert!((irr.type_rate - 0.99).abs() < 1e-12);
    let rate = function(&gates, "RATE");
    assert_eq!((rate.cells, rate.outside), (100, 2));
    assert!(!rate.meets);
    // ABS: 31 attributed probe cells (the gen-m5 ones call IRR too); one outside.
    let abs = function(&gates, "ABS");
    assert_eq!((abs.cells, abs.wrong_type, abs.outside), (31, 0, 1));
    assert!(
        abs.meets,
        "no per-function envelope bar for a foundational function"
    );
    let sum = function(&gates, "SUM");
    assert_eq!((sum.cells, sum.wrong_type), (10, 1));
    assert!(!sum.meets);
    let none = function(&gates, "BESSELJ");
    assert_eq!((none.cells, none.type_rate, none.meets), (0, 1.0, true));
    assert_eq!(function(&gates, "GETPIVOTDATA").cells, 0);
    let foundational = tier(&gates, Usage::Foundational);
    assert_eq!(
        (
            foundational.functions,
            foundational.cells,
            foundational.wrong_type,
            foundational.outside
        ),
        (2, 41, 1, 1)
    );
    // SUM's wrong type fails both its bar and the pooled 100%; ABS's outside cell is 1 of 41,
    // below the pooled 99.99%, so it is listed too.
    assert_eq!(foundational.failing_functions, ["ABS", "SUM"]);
    assert!(!foundational.met);
    let expert = tier(&gates, Usage::Expert);
    assert_eq!(expert.functions, 4);
    assert_eq!(expert.failing_functions, ["RATE", "IRR"]);
    assert!(!expert.met);
    assert!((expert.function_bar.type_share - 0.99).abs() < 1e-12);
    assert!(!gates.met);
    // Failing functions first, most misses first.
    assert_eq!(gates.functions[0].name, "RATE");
    let mut md = String::new();
    write_markdown(&gates, &mut md);
    assert!(md.contains("## Usage tiers (decision 0355)"), "{md}");
    assert!(md.contains("**Usage tiers: not met.**"), "{md}");
    assert!(
        md.contains("| foundational | 2 | 41 | 1 | 97.5610% | 1 | 97.5610% | 100.0000%, 99.9900%; 100.0000%, pooled only | not met |"),
        "{md}"
    );
    assert!(
        md.contains("- RATE: 100 cells, 0 wrong type (100.0000%), 2 outside (98.0000%)"),
        "{md}"
    );
}

#[test]
fn a_clean_run_meets_both_tiers() {
    let catalog = catalog(&[("ABS", Usage::Foundational), ("IRR", Usage::Expert)]);
    let mut probes = cells("ABS(1)", 1, 30, 0, 0);
    probes.extend(cells("IRR(A1:A3)", 100, 2000, 1, 1));
    let gates = summarize(
        &catalog,
        &[case("probes/p.case.json", "probes", probes)],
        &Phase1Exceptions::default(),
    );
    assert!(gates
        .tiers
        .iter()
        .all(|t| t.met && t.failing_functions.is_empty()));
    assert!(gates.met);
    // No counted cell: nothing measured, never met.
    let empty = summarize(&catalog, &[], &Phase1Exceptions::default());
    assert!(!empty.met && empty.tiers.iter().all(|t| !t.met));
}

#[test]
fn a_cell_takes_its_sole_calls_tier_else_the_strictest() {
    let catalog = catalog(&[
        ("ABS", Usage::Foundational),
        ("IRR", Usage::Expert),
        ("RATE", Usage::Expert),
    ]);
    let of = |f: &str| catalog.function(f).map(|c| c.usage);
    let calls = |names: &[&str]| {
        names
            .iter()
            .map(|n| (*n).to_string())
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(cell_usage(&calls(&["IRR"]), of), Usage::Expert);
    // IF is glue: the sole call is IRR.
    assert_eq!(cell_usage(&calls(&["IF", "IRR"]), of), Usage::Expert);
    assert_eq!(cell_usage(&calls(&["ABS", "IRR"]), of), Usage::Foundational);
    assert_eq!(cell_usage(&calls(&["IRR", "RATE"]), of), Usage::Expert);
    // No catalog function (an operator, a spill extent, a user name): foundational.
    assert_eq!(cell_usage(&calls(&[]), of), Usage::Foundational);
    assert_eq!(cell_usage(&calls(&["MYNAME"]), of), Usage::Foundational);
    let mut row = TierTallies::default();
    row.tally_mut(Usage::Expert).cells = 1000;
    row.tally_mut(Usage::Expert).miss(CellEnvelope::WrongType);
    assert!(row.meets_bars(), "1 of 1000 expert cells");
    row.tally_mut(Usage::Foundational)
        .add(CellEnvelope::Outside);
    assert!(!row.meets_bars(), "1 of 1 foundational cell outside");
    assert_eq!(row.text(), "F 0/1 of 1, E 1/0 of 1000");
}
