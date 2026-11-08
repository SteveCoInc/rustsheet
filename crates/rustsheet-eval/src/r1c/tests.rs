use super::*;
use crate::accuracy::envelope::MathValue;
use crate::accuracy::lists::R3Mark;
use crate::value::compare;

const CASE: &str = "probes/t.case.json";

/// A scored cell: Excel's `excel` against the engine's `engine`, with an optional math value.
fn cell(excel: Value, engine: Value, math: Option<f64>) -> CellResult {
    let mut c = CellResult::for_test("A1", Some("F(1)"), compare(&excel, &engine));
    c.expected = excel;
    c.actual = engine;
    c.math = math.map(MathValue::Number);
    c
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn marks(id: &str) -> R3Marks {
    R3Marks::from([(
        id.to_string(),
        R3Mark {
            line: 1,
            semantics_file: "docs/excel-semantics/statistics.md".into(),
            anchor: "x".into(),
        },
    )])
}

fn owned(case: &str, key: &str) -> Phase1Exceptions {
    let mut e = Phase1Exceptions::default();
    e.desktop_owned
        .insert((case.to_string(), key.to_string()), "T-2425".into());
    e
}

fn class(c: &CellResult, r3: &R3Marks, exceptions: &Phase1Exceptions) -> Option<R1c> {
    classify(CASE, c, r3, exceptions)
}

const MATH: f64 = 0.123_456_789_012_345_6;

#[test]
fn a_cell_that_is_not_visibly_off_has_no_class() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    assert_eq!(class(&cell(n(MATH), n(MATH), None), &none.0, &none.1), None);
    // 1e-15 relative: within 15 digits, so not visibly off.
    let close = cell(n(MATH), n(MATH * (1.0 + 1e-15)), None);
    assert_eq!(class(&close, &none.0, &none.1), None);
}

#[test]
fn a_type_or_error_cell_counts_even_inside_the_envelope() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    let error = cell(n(MATH), Value::Error("#NUM!".into()), Some(MATH));
    assert_eq!(class(&error, &none.0, &none.1), Some(R1c::Counts));
    let text = cell(Value::Text("a".into()), n(MATH), Some(MATH));
    assert_eq!(class(&text, &none.0, &none.1), Some(R1c::Counts));
}

#[test]
fn a_number_inside_the_envelope_is_phase_1b_evidence() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    // 1e-12 off Excel (visibly off) and 1e-12 off the math (within ε); Excel 2e-12 off the math.
    let c = cell(n(MATH * (1.0 + 1e-12)), n(MATH * (1.0 - 1e-12)), Some(MATH));
    assert_eq!(class(&c, &none.0, &none.1), Some(R1c::Phase1b));
}

#[test]
fn a_number_outside_the_envelope_counts() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    let far = cell(n(MATH), n(MATH * (1.0 + 1e-6)), Some(MATH));
    assert_eq!(class(&far, &none.0, &none.1), Some(R1c::Counts));
    // No math value: Excel's 12 digits alone decide. 1e-10 relative is past half a unit in the
    // 12th digit of 0.123…; 1e-12 relative is within it.
    let no_math = cell(n(MATH * (1.0 + 1e-10)), n(MATH), None);
    assert_eq!(class(&no_math, &none.0, &none.1), Some(R1c::Counts));
    let close = cell(n(MATH * (1.0 + 1e-12)), n(MATH), None);
    assert_eq!(class(&close, &none.0, &none.1), Some(R1c::Phase1b));
}

#[test]
fn a_number_on_the_math_is_phase_1b_whatever_excel_says() {
    // The engine returns the math; Excel is 1e-6 off it: more than ε, less than G = 1e-4. Before
    // decision 0355 this counted (a rule Excel applies might hide in the gap); item 2 makes a
    // value within ε of the math good whatever Excel says.
    let c = cell(n(MATH * (1.0 + 1e-6)), n(MATH), Some(MATH));
    assert_eq!(
        class(&c, &R3Marks::new(), &Phase1Exceptions::default()),
        Some(R1c::Phase1b)
    );
}

#[test]
fn excel_more_than_g_off_the_math_is_an_r3_pass_when_marked() {
    // The engine returns the math; Excel is 1e-3 off it (more than G = 1e-4).
    let c = cell(n(MATH * 1.001), n(MATH), Some(MATH));
    let none = Phase1Exceptions::default();
    assert_eq!(class(&c, &R3Marks::new(), &none), Some(R1c::Phase1b));
    let marked = marks(&cell_id(CASE, "S!A1"));
    assert_eq!(class(&c, &marked, &none), Some(R1c::R3Pass));
    // An R3 cell where the engine misses the math still counts.
    let miss = cell(n(MATH * 1.001), n(MATH * (1.0 + 1e-6)), Some(MATH));
    assert_eq!(class(&miss, &marked, &none), Some(R1c::Counts));
}

#[test]
fn a_cell_only_m14_tasks_own_leaves_the_row_whatever_its_values() {
    let exceptions = owned(CASE, "S!A1");
    let error = cell(n(MATH), Value::Error("#NUM!".into()), None);
    assert_eq!(
        class(&error, &R3Marks::new(), &exceptions),
        Some(R1c::M14Owned)
    );
    let other_case = owned("probes/other.case.json", "S!A1");
    assert_eq!(
        class(&error, &R3Marks::new(), &other_case),
        Some(R1c::Counts)
    );
}

/// R7 names the milestone probe rows only: an M14-owned ironcalc-calc cell still counts.
#[test]
fn an_m14_owned_cell_outside_the_probe_rows_still_counts() {
    let calc = "ironcalc-calc/BESSEL.xlsx";
    let error = cell(n(MATH), Value::Error("#NUM!".into()), None);
    let exceptions = owned(calc, "S!A1");
    assert_eq!(
        classify(calc, &error, &R3Marks::new(), &exceptions),
        Some(R1c::Counts)
    );
}

/// An R3 mark needs its evidence on the cell, Excel more than G off the math (decision 0188 R3);
/// below G the mark is ignored and Excel's value stays the target.
#[test]
fn an_r3_mark_without_its_evidence_does_not_pass() {
    let marked = marks(&cell_id(CASE, "S!A1"));
    let none = Phase1Exceptions::default();
    // Excel 1e-6 off the math, the engine on it: more than ε, less than G. No R3 pass, but the
    // engine is within ε of the math, so inside the envelope.
    let below_g = cell(n(MATH * (1.0 + 1e-6)), n(MATH), Some(MATH));
    assert_eq!(class(&below_g, &marked, &none), Some(R1c::Phase1b));
    // Below G, the engine 1e-6 off the math and Excel: the mark excuses nothing.
    let off = cell(n(MATH * (1.0 + 1e-6)), n(MATH * (1.0 - 1e-6)), Some(MATH));
    assert_eq!(class(&off, &marked, &none), Some(R1c::Counts));
    // Excel 1e-12 off the math, the engine 1e-12 the other way: Phase 1b, mark or not.
    let tiny = cell(n(MATH * (1.0 + 1e-12)), n(MATH * (1.0 - 1e-12)), Some(MATH));
    assert_eq!(class(&tiny, &marked, &none), Some(R1c::Phase1b));
}

#[test]
fn the_run_classes_and_the_phase_1b_counts_per_suite() {
    let mut probe = CaseResult::for_test(CASE, "probes");
    let inside = cell(n(MATH * (1.0 + 1e-12)), n(MATH * (1.0 - 1e-12)), Some(MATH));
    let mut far = cell(n(MATH), n(MATH * 1.5), Some(MATH));
    far.cell = "A2".into();
    probe.cells = vec![inside, far];
    let mut numeric = CaseResult::for_test("probes-numeric/n.case.json", "probes-numeric");
    numeric.cells = vec![cell(n(MATH), Value::Error("#NUM!".into()), None)];
    let mut generated = CaseResult::for_test("gen-m9/g.case.json", "gen-m9");
    generated.cells = vec![cell(n(MATH), n(MATH * 1.5), None)];
    let run = R1cCells::build(
        &[probe, numeric, generated],
        &R3Marks::new(),
        &Phase1Exceptions::default(),
    );
    assert_eq!(run.class("probes/t.case.json!S!A1"), R1c::Phase1b);
    assert!(run.counts("probes/t.case.json!S!A2"));
    assert!(run.counts("probes/never-classified.case.json!S!A1"));
    assert_eq!(
        run.left_out().into_iter().collect::<Vec<_>>(),
        [("probes/t.case.json!S!A1".to_string(), R1c::Phase1b)]
    );
    let evidence = run.phase1b();
    assert_eq!(
        evidence.suites.keys().collect::<Vec<_>>(),
        ["probes", "probes-numeric"]
    );
    assert_eq!(
        evidence.suites["probes"],
        R1cCounts {
            counts: 1,
            phase1b: 1,
            r3_pass: 0,
            m14_owned: 0
        }
    );
    assert_eq!(evidence.suites["probes-numeric"].counts, 1);
    let line = evidence.line().unwrap_or_default();
    assert!(line.contains("probes 1 counted, 1 Phase 1b evidence, 0 R3 passing, 0 M14-owned"));
    assert_eq!(Phase1bEvidence::default().line(), None);
}

/// The cell after the run marks it ([`mark_r3_math`]) and its closeness counts.
fn bucketed(c: CellResult, r3: &R3Marks) -> (CellResult, crate::closeness::Closeness) {
    let mut case = CaseResult::for_test(CASE, "probes");
    case.cells = vec![c];
    mark_r3_math(std::slice::from_mut(&mut case), r3);
    let c = case.cells.remove(0);
    let mut counts = crate::closeness::Closeness::default();
    counts.add(&c);
    (c, counts)
}

/// Decision 0310: the run marks an R3 cell where the engine returns the math once, and the
/// closeness buckets count it as `r3_math`, not visibly off; it is inside the envelope on the
/// math, and R1(c) still classifies it as an R3 pass.
#[test]
fn a_marked_r3_cell_on_the_math_is_r3_math_not_visibly_off() {
    let marked = marks(&cell_id(CASE, "S!A1"));
    let none = Phase1Exceptions::default();
    // Marked, Excel 1e-3 off the math (more than G), the engine on it: r3_math.
    let (on, counts) = bucketed(cell(n(MATH * 1.001), n(MATH), Some(MATH)), &marked);
    assert_eq!(on.target, Target::Math);
    assert_eq!(
        (counts.r3_math, counts.visible(), counts.scored()),
        (1, 0, 1)
    );
    assert!(!Visible::is_off(&on) && !Visible::is_failing_off(&on));
    assert!(Visible::is_r3_math(&on));
    assert_eq!(cell_envelope(&on), CellEnvelope::Inside);
    assert_eq!(classify(CASE, &on, &marked, &none), Some(R1c::R3Pass));
    // The run's classes keep the R3 pass for the Phase 1b line and `r1c_left_out`.
    let mut case = CaseResult::for_test(CASE, "probes");
    case.cells = vec![on];
    let run = R1cCells::build(&[case], &marked, &none);
    assert_eq!(run.class(&cell_id(CASE, "S!A1")), R1c::R3Pass);
    assert_eq!(run.phase1b().suites["probes"].r3_pass, 1);
}

#[test]
fn a_marked_cell_off_the_math_stays_visibly_off() {
    let marked = marks(&cell_id(CASE, "S!A1"));
    let none = Phase1Exceptions::default();
    // Marked, but the engine 1e-6 off the math: still visibly off, and R1(c) counts it.
    let (miss, counts) = bucketed(
        cell(n(MATH * 1.001), n(MATH * (1.0 + 1e-6)), Some(MATH)),
        &marked,
    );
    // T-5097: held to the math (`MathMissed`), but no r3_math pass, so its bucket is unchanged.
    assert_eq!(miss.target, Target::MathMissed);
    assert_eq!((counts.r3_math, counts.visible_numeric), (0, 1));
    assert_eq!(cell_envelope(&miss), CellEnvelope::Outside);
    assert_eq!(classify(CASE, &miss, &marked, &none), Some(R1c::Counts));
    // Marked, but Excel only 1e-6 off the math (below G): the mark is ignored.
    let (below_g, counts) = bucketed(cell(n(MATH * (1.0 + 1e-6)), n(MATH), Some(MATH)), &marked);
    assert_eq!(below_g.target, Target::Excel);
    assert_eq!((counts.r3_math, counts.visible_numeric), (0, 1));
}

/// T-5097: an R3 cell where the engine moved to Excel's value (within 12 digits of it, off the math)
/// is outside the envelope and counts; before, Excel's side let it in as Phase 1b.
#[test]
fn a_marked_cell_at_excels_value_off_the_math_counts() {
    let marked = marks(&cell_id(CASE, "S!A1"));
    let none = Phase1Exceptions::default();
    let excel = MATH * 1.001;
    let (back, counts) = bucketed(
        cell(n(excel), n(excel * (1.0 + 1e-13)), Some(MATH)),
        &marked,
    );
    assert_eq!(back.target, Target::MathMissed);
    assert_eq!(counts.r3_math, 0);
    assert_eq!(cell_envelope(&back), CellEnvelope::Outside);
    assert_eq!(classify(CASE, &back, &marked, &none), Some(R1c::Counts));
    // Unmarked, the same cell is inside on Excel's side: Phase 1b evidence.
    let (plain, _) = bucketed(
        cell(n(excel), n(excel * (1.0 + 1e-13)), Some(MATH)),
        &R3Marks::new(),
    );
    assert_eq!(plain.target, Target::Excel);
    assert_eq!(
        classify(CASE, &plain, &R3Marks::new(), &none),
        Some(R1c::Phase1b)
    );
}

#[test]
fn an_unmarked_or_error_cell_keeps_its_bucket() {
    let marked = marks(&cell_id(CASE, "S!A1"));
    let none = Phase1Exceptions::default();
    // Unmarked: unchanged, visibly off.
    let (plain, counts) = bucketed(cell(n(MATH * 1.001), n(MATH), Some(MATH)), &R3Marks::new());
    assert_eq!(plain.target, Target::Excel);
    assert_eq!((counts.r3_math, counts.visible_numeric), (0, 1));
    // An error, even marked with a math value, never qualifies: R3 never applies to an error.
    let (error, counts) = bucketed(
        cell(n(MATH * 1.001), Value::Error("#NUM!".into()), Some(MATH)),
        &marked,
    );
    assert_eq!(error.target, Target::Excel);
    assert_eq!((counts.r3_math, counts.visible_other), (0, 1));
    assert_eq!(classify(CASE, &error, &marked, &none), Some(R1c::Counts));
}

#[test]
fn a_counted_cell_misses_as_a_wrong_type_or_outside() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    let mut probe = CaseResult::for_test(CASE, "probes");
    let mut far = cell(n(MATH), n(MATH * 1.5), Some(MATH));
    far.cell = "A2".into();
    let mut error = cell(n(MATH), Value::Error("#NUM!".into()), None);
    error.cell = "A3".into();
    let inside = cell(n(MATH * (1.0 + 1e-12)), n(MATH * (1.0 - 1e-12)), Some(MATH));
    probe.cells = vec![inside, far, error];
    let run = R1cCells::build(&[probe], &none.0, &none.1);
    assert_eq!(run.miss("probes/t.case.json!S!A1"), None);
    assert_eq!(
        run.miss("probes/t.case.json!S!A2"),
        Some(CellEnvelope::Outside)
    );
    assert_eq!(
        run.miss("probes/t.case.json!S!A3"),
        Some(CellEnvelope::WrongType)
    );
    // A cell the run did not classify reads as a wrong type: nothing reads as met early.
    assert_eq!(
        run.miss("probes/never-classified.case.json!S!A1"),
        Some(CellEnvelope::WrongType)
    );
}
