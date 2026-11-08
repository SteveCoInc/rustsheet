//! The baseline's tests: the diff kinds, the `E` and `W` lines, the modes and the file.

use super::*;

fn b(checked: &[(&str, usize)], failing: &[(&str, &str)]) -> Baseline {
    Baseline {
        checked: checked
            .iter()
            .map(|(c, n)| ((*c).to_string(), *n))
            .collect(),
        failing: failing
            .iter()
            .map(|(c, i)| ((*c).to_string(), (*i).to_string()))
            .collect(),
        ..Baseline::default()
    }
}

fn items(items: &[(&str, &str)]) -> Items {
    items
        .iter()
        .map(|(c, i)| ((*c).to_string(), (*i).to_string()))
        .collect()
}

/// `base` recording `E` lines, `outside` its items outside the envelope.
fn with_outside(base: Baseline, outside: &[(&str, &str)]) -> Baseline {
    Baseline {
        outside: items(outside),
        envelope: true,
        ..base
    }
}

/// `base` recording `W` lines too, `wrong` its wrong-type items.
fn with_wrong(base: Baseline, wrong: &[(&str, &str)]) -> Baseline {
    Baseline {
        wrong_type: items(wrong),
        typed: true,
        ..base
    }
}

/// Scored cell `S!<cell>`: Excel's 1 against the engine's `engine`.
fn scored(cell: &str, engine: Value) -> CellResult {
    CellResult {
        verdict: crate::value::compare(&Value::Number(1.0), &engine),
        actual: engine,
        ..CellResult::for_test(cell, Some("F(1)"), Verdict::Pass)
    }
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

/// One part in 1e13 off Excel: a bit-level failure inside the 12-digit envelope.
const DRIFT: f64 = 1.0 + 1e-13;

/// A failing case `id` scoring `cells`.
fn failing_case(id: &str, cells: Vec<CellResult>) -> CaseResult {
    CaseResult {
        status: CaseStatus::Fail,
        checked: cells.len(),
        cells,
        ..CaseResult::for_test(id, "s")
    }
}

use crate::value::{Value, Verdict};

/// In an unchanged case: a new `F` item inside the envelope is drift, a new `E` item a
/// regression (also when the item drifted before), and an `E` item that still fails moved into
/// the envelope (decision 0355 item 6).
#[test]
fn drift_regressions_and_moves_into_the_envelope() {
    let old = with_outside(
        b(&[("a", 9)], &[("a", "S!A2"), ("a", "S!A4"), ("a", "S!A5")]),
        &[("a", "S!A4")],
    );
    let new = with_outside(
        b(
            &[("a", 9)],
            &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3"), ("a", "S!A4")],
        ),
        &[("a", "S!A2"), ("a", "S!A3")],
    );
    let d = diff(&old, &new);
    assert_eq!(d.drift, ["a\tS!A1"]);
    assert_eq!(d.regressions, ["a\tS!A2", "a\tS!A3"]);
    assert_eq!(d.into_envelope, ["a\tS!A4"]);
    assert_eq!(d.fixed, ["a\tS!A5"]);
    assert!(d.new_failures.is_empty());
    assert!(!diff(&new, &new).is_dirty());
    // Drift alone is no regression, but the baseline is out of date until it is recorded.
    let drifted = with_outside(
        b(
            &[("a", 9)],
            &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A4"), ("a", "S!A5")],
        ),
        &[("a", "S!A4")],
    );
    let d = diff(&old, &drifted);
    assert!(d.regressions.is_empty());
    assert_eq!(d.drift, ["a\tS!A1"]);
    assert!(d.is_dirty());
    // So is a move into the envelope alone.
    let into = with_outside(old.clone(), &[]);
    let d = diff(&old, &into);
    assert_eq!(d.into_envelope, ["a\tS!A4"]);
    assert!(d.regressions.is_empty() && d.fixed.is_empty() && d.is_dirty());
}

/// In an unchanged case an item outside as a number that becomes a wrong type is a
/// regression though it stays an `E` item; a new `W` item is one too; a `W` item that stays
/// outside as a number is the right type now, and one that leaves the envelope is into the
/// envelope only (decision 0355 item 6).
#[test]
fn wrong_type_transitions() {
    let failing = [("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3")];
    let typed = |outside: &[(&str, &str)], wrong: &[(&str, &str)]| {
        with_wrong(with_outside(b(&[("a", 5)], &failing), outside), wrong)
    };
    // A1 outside as a number, A2 a wrong type, A3 a wrong type.
    let old = typed(
        &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3")],
        &[("a", "S!A2"), ("a", "S!A3")],
    );
    // A1 now a wrong type; A2 now a number off Excel; A3 now inside the envelope.
    let new = typed(&[("a", "S!A1"), ("a", "S!A2")], &[("a", "S!A1")]);
    let d = diff(&old, &new);
    assert_eq!(d.regressions, ["a\tS!A1"]);
    assert_eq!(d.right_type, ["a\tS!A2"]);
    assert_eq!(d.into_envelope, ["a\tS!A3"]);
    assert!(d.drift.is_empty() && d.fixed.is_empty() && d.new_failures.is_empty());
    // A cell newly failing as a wrong type is a regression once, not drift.
    let added = with_wrong(
        with_outside(
            b(
                &[("a", 5)],
                &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3"), ("a", "S!A4")],
            ),
            &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3"), ("a", "S!A4")],
        ),
        &[("a", "S!A2"), ("a", "S!A3"), ("a", "S!A4")],
    );
    let d = diff(&old, &added);
    assert_eq!(d.regressions, ["a\tS!A4"]);
    assert!(d.drift.is_empty() && d.right_type.is_empty());
    // The right type alone dirties the baseline, and needs no `--allow-regressions`.
    let right = typed(
        &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3")],
        &[("a", "S!A3")],
    );
    let d = diff(&old, &right);
    assert!(d.regressions.is_empty() && d.is_dirty());
    assert_eq!(d.right_type, ["a\tS!A2"]);
    // In a changed case the new wrong type is new coverage.
    let changed = typed(
        &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3")],
        &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3")],
    );
    let changed = Baseline {
        checked: [("a".to_string(), 6)].into(),
        ..changed
    };
    let d = diff(&old, &changed);
    assert!(d.regressions.is_empty());
    assert!(
        d.new_failures.contains(&"a\tS!A1".to_string()),
        "{:?}",
        d.new_failures
    );
}

/// A baseline with `E` lines but no `W` lines counts every `E` item as a wrong type: its
/// numbers off Excel are the right type on the next run, and no wrong type is new.
#[test]
fn a_baseline_without_w_lines_counts_every_e_item_a_wrong_type() {
    let old = with_outside(
        b(&[("a", 3)], &[("a", "S!A1"), ("a", "S!A2")]),
        &[("a", "S!A1"), ("a", "S!A2")],
    );
    assert!(!old.typed);
    assert_eq!(old.wrong_type_items(), &old.outside);
    let new = with_wrong(old.clone(), &[("a", "S!A2")]);
    let d = diff(&old, &new);
    assert!(d.regressions.is_empty());
    assert_eq!(d.right_type, ["a\tS!A1"]);
    // And one with neither kind counts every `F` item as both.
    let bare = b(&[("a", 3)], &[("a", "S!A1")]);
    assert_eq!(bare.wrong_type_items(), &bare.failing);
}

/// In a new or changed case every new item, inside the envelope or not, is new coverage.
#[test]
fn a_changed_case_records_new_e_items_as_new_coverage() {
    let old = with_outside(b(&[("a", 3)], &[("a", "S!A1")]), &[]);
    let new = with_outside(b(&[("a", 4), ("c", 1)], &[("a", "S!A1")]), &[("a", "S!A1")]);
    let new = Baseline {
        failing: items(&[("a", "S!A1"), ("c", "S!C1")]),
        ..new
    };
    let d = diff(&old, &new);
    assert!(d.regressions.is_empty() && d.drift.is_empty());
    assert!(d.new_failures.contains(&"a\tS!A1".to_string()));
    assert!(d.new_failures.contains(&"c\tS!C1".to_string()));
}

/// A baseline written before the `E` kind holds each `F` item to the bit-level rule it was
/// recorded under, so the migration run refuses what the old gate refused and records the
/// rest; two baselines without `E` lines (the round-trip ratchet's) ratchet each item exactly.
#[test]
fn a_baseline_without_e_lines_counts_every_f_item_outside() {
    let old = b(&[("a", 3)], &[("a", "S!A1"), ("a", "S!A2")]);
    assert!(!old.envelope);
    let new = with_outside(
        b(
            &[("a", 3)],
            &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3"), ("a", "S!B1")],
        ),
        &[("a", "S!A2"), ("a", "S!B1")],
    );
    let d = diff(&old, &new);
    assert_eq!(d.into_envelope, ["a\tS!A1"]);
    assert_eq!(d.drift, ["a\tS!A3"]);
    assert_eq!(d.regressions, ["a\tS!B1"]);
    let exact = b(&[("a", 3)], &[("a", "S!A1"), ("a", "S!A2"), ("a", "S!A3")]);
    let d = diff(&old, &exact);
    assert_eq!(d.regressions, ["a\tS!A3"]);
    assert!(d.drift.is_empty() && d.into_envelope.is_empty());
}

/// A failing cell is an `F` item, and an `E` item too only when `cell_envelope` puts it
/// outside: a number off 12 digits of Excel, or a wrong type or error.
#[test]
fn cells_are_e_items_only_outside_the_envelope() {
    let r = failing_case(
        "s/a",
        vec![
            scored("A1", n(1.0)),
            scored("A2", n(DRIFT)),
            scored("A3", n(2.0)),
            scored("A4", Value::Error("#NUM!".into())),
        ],
    );
    let base = Baseline::from_results(BaselineMode::Cells, &[&r]);
    assert!(base.envelope);
    assert_eq!(
        base.failing,
        items(&[("s/a", "S!A2"), ("s/a", "S!A3"), ("s/a", "S!A4")])
    );
    assert_eq!(base.outside, items(&[("s/a", "S!A3"), ("s/a", "S!A4")]));
    assert_eq!(base.outside_items(), &base.outside);
    // Only the error is a `W` item: the number off Excel is the right type.
    assert!(base.typed);
    assert_eq!(base.wrong_type, items(&[("s/a", "S!A4")]));
    assert_eq!(base.wrong_type_items(), &base.wrong_type);
}

/// A case that did not run is never drift, in either mode.
#[test]
fn whole_case_items_are_always_outside() {
    for mode in [BaselineMode::Cells, BaselineMode::Cases] {
        let crashed = CaseResult {
            status: CaseStatus::Panic,
            ..CaseResult::for_test("s/x", "s")
        };
        let base = Baseline::from_results(mode, &[&crashed]);
        assert_eq!(base.failing, items(&[("s/x", "!Panic")]));
        assert_eq!(base.outside, base.failing);
        assert_eq!(base.wrong_type, base.failing);
    }
}

/// In a `cases`-mode suite a case fails `*` only when one of its cells is outside the
/// envelope: a corpus case with only drift is no failure, and its old `*` line is a fix.
#[test]
fn a_cases_mode_case_fails_only_outside_the_envelope() {
    let drift = failing_case("c/a", vec![scored("A1", n(1.0)), scored("A2", n(DRIFT))]);
    let off = failing_case("c/b", vec![scored("A1", n(DRIFT)), scored("A2", n(2.0))]);
    let base = Baseline::from_results(BaselineMode::Cases, &[&drift, &off]);
    assert_eq!(base.failing, items(&[("c/b", "*")]));
    assert_eq!(base.outside, base.failing);
    let old = b(&[("c/a", 2), ("c/b", 2)], &[("c/a", "*"), ("c/b", "*")]);
    let d = diff(&old, &base);
    assert_eq!(d.fixed, ["c/a\t*"]);
    assert!(d.regressions.is_empty() && d.into_envelope.is_empty());
    let was = Baseline::from_results(BaselineMode::Cases, &[&drift]);
    let now = Baseline::from_results(
        BaselineMode::Cases,
        &[&failing_case(
            "c/a",
            vec![scored("A1", n(1.0)), scored("A2", n(2.0))],
        )],
    );
    assert_eq!(diff(&was, &now).regressions, ["c/a\t*"]);
}

/// `base` with displayed-text counts and failures.
fn with_text(base: Baseline, checked: &[(&str, usize)], failing: &[(&str, &str)]) -> Baseline {
    Baseline {
        text_checked: checked
            .iter()
            .map(|(c, n)| ((*c).to_string(), *n))
            .collect(),
        text_failing: failing
            .iter()
            .map(|(c, i)| ((*c).to_string(), (*i).to_string()))
            .collect(),
        ..base
    }
}

#[test]
fn text_cells_ratchet_like_values() {
    // A text cell that passed in an unchanged case and now fails is a regression even when
    // every value still passes (decision 0070).
    let old = with_text(b(&[("a", 3)], &[]), &[("a", 2)], &[("a", "S!A1")]);
    let new = with_text(b(&[("a", 3)], &[]), &[("a", 2)], &[("a", "S!A2")]);
    let d = diff(&old, &new);
    assert_eq!(d.regressions, vec!["a\ttext S!A2"]);
    assert_eq!(d.fixed, vec!["a\ttext S!A1"]);
    // More text cells in the case: new coverage, to be recorded, not a regression.
    let grown = with_text(b(&[("a", 3)], &[]), &[("a", 3)], &[("a", "S!A2")]);
    let d = diff(&old, &grown);
    assert!(d.regressions.is_empty());
    assert!(d.new_failures.contains(&"a\ttext S!A2".to_string()));
    assert!(d.is_dirty());
    // Text lines leave the value lists alone.
    assert_eq!(new.failing, BTreeSet::new());
}

/// `base` with pivot-layout counts and failures.
fn with_layout(base: Baseline, checked: &[(&str, usize)], failing: &[(&str, &str)]) -> Baseline {
    let t = with_text(Baseline::default(), checked, failing);
    Baseline {
        layout_checked: t.text_checked,
        layout_failing: t.text_failing,
        ..base
    }
}

#[test]
fn pivot_layouts_ratchet_like_values() {
    // A pivot whose layout was Excel's in an unchanged case and now is not is a regression
    // even when every cell passes; one that now is Excel's is a fix (T-4944).
    let old = with_layout(b(&[("a", 3)], &[]), &[("a", 2)], &[("a", "K!Top")]);
    let new = with_layout(b(&[("a", 3)], &[]), &[("a", 2)], &[("a", "S!Pv")]);
    let d = diff(&old, &new);
    assert_eq!(d.regressions, vec!["a\tlayout S!Pv"]);
    assert_eq!(d.fixed, vec!["a\tlayout K!Top"]);
    // A case that now compares more pivots is new coverage, to be recorded.
    let grown = with_layout(b(&[("a", 3)], &[]), &[("a", 3)], &[("a", "S!Pv")]);
    let d = diff(&old, &grown);
    assert!(d.regressions.is_empty());
    assert!(d.new_failures.contains(&"a\tlayout S!Pv".to_string()));
    // A case new to the layout lines (an existing case once it compares them) is too.
    let d = diff(&b(&[("a", 3)], &[]), &new);
    assert!(d.regressions.is_empty());
    assert_eq!(d.new_cases, vec!["layout a"]);
    assert!(d.is_dirty());
    assert!(new.failing.is_empty() && new.text_failing.is_empty());
}

#[test]
fn only_a_case_that_compares_its_pivot_lines_records_their_layout() {
    use crate::pivot_scoring::{LayoutVerdict, PivotLayout};
    let layout = |verdict| PivotLayout {
        sheet: "K".into(),
        name: "Top".into(),
        excel_location: "A3:D11".into(),
        engine_location: Some("A3:D11".into()),
        excel_lines: (7, 2),
        engine_lines: Some((7, 2)),
        verdict,
    };
    let export = |verdict| CaseResult {
        pivot_layout: vec![layout(verdict)],
        layout_checked: Some(2),
        ..CaseResult::for_test("x/a", "x")
    };
    let lines = Baseline::from_results(BaselineMode::Cells, &[&export(LayoutVerdict::LinesDiffer)]);
    assert_eq!(lines.layout_checked.get("x/a"), Some(&2));
    let items: Vec<_> = lines
        .layout_failing
        .iter()
        .map(|(_, i)| i.as_str())
        .collect();
    assert_eq!(items, ["K!Top"]);
    // A location verdict is the same item, so fixing the location alone changes nothing.
    let moved = Baseline::from_results(
        BaselineMode::Cells,
        &[&export(LayoutVerdict::LocationDiffers)],
    );
    assert!(!diff(&moved, &lines).is_dirty());
    // A case whose lines are not compared (every other suite) records no layout line.
    let other = CaseResult {
        pivot_layout: vec![layout(LayoutVerdict::LinesNotEmitted)],
        ..CaseResult::for_test("x/a", "x")
    };
    let plain = Baseline::from_results(BaselineMode::Cells, &[&other]);
    assert!(plain.layout_checked.is_empty() && plain.layout_failing.is_empty());
    // An ungated suite records nothing.
    let none = Baseline::from_results(BaselineMode::None, &[&export(LayoutVerdict::LinesDiffer)]);
    assert_eq!(none, Baseline::default());
}

#[test]
fn classifies_regressions_new_failures_and_fixes() {
    let old = b(&[("a", 3), ("b", 2)], &[("a", "S!A1")]);
    let new = b(
        &[("a", 3), ("b", 3), ("c", 1)],
        &[("a", "S!A2"), ("b", "S!B1"), ("c", "S!C1")],
    );
    let d = diff(&old, &new);
    assert_eq!(d.regressions, vec!["a\tS!A2"]);
    assert!(d.new_failures.contains(&"b\tS!B1".to_string()));
    assert!(d.new_failures.contains(&"c\tS!C1".to_string()));
    assert_eq!(d.fixed, vec!["a\tS!A1"]);
    assert!(d.removed_cases.is_empty());
    assert_eq!(d.new_cases, vec!["c"]);
}

#[test]
fn new_passing_case_makes_the_baseline_dirty() {
    // A case with no failures adds only its `C` line, which must still be recorded (T-1059).
    let old = b(&[("a", 3)], &[]);
    let new = b(&[("a", 3), ("b", 2)], &[]);
    let d = diff(&old, &new);
    assert_eq!(d.new_cases, vec!["b"]);
    assert!(d.new_failures.is_empty());
    assert!(d.is_dirty());
    assert!(!diff(&new, &new).is_dirty());
}

#[test]
fn a_passing_case_records_its_failing_text_cells() {
    use crate::text_expect::TextResult;
    let text = |cell: &str, pass: bool| TextResult {
        sheet: "S".into(),
        cell: cell.into(),
        expected: "1".into(),
        actual: Some(if pass { "1" } else { "2" }.into()),
        pass,
    };
    let run = |pass: bool| CaseResult {
        text: vec![text("A1", true), text("A2", pass)],
        ..CaseResult::for_test("s/a", "s")
    };
    let before = run(true);
    let after = run(false);
    let old = Baseline::from_results(BaselineMode::Cells, &[&before]);
    let new = Baseline::from_results(BaselineMode::Cells, &[&after]);
    assert_eq!(new.text_checked.get("s/a"), Some(&2));
    assert!(new.failing.is_empty());
    // The check fails: the case's values all pass, yet one text cell regressed.
    assert_eq!(diff(&old, &new).regressions, vec!["s/a\ttext S!A2"]);
    // An ungated suite records nothing.
    assert_eq!(
        Baseline::from_results(BaselineMode::None, &[&after]),
        Baseline::default()
    );
}

#[test]
fn roundtrips_through_disk() {
    let dir = std::env::temp_dir().join(format!("rs-baseline-{}", std::process::id()));
    let path = dir.join("x.txt");
    let base = with_text(
        b(&[("a", 3)], &[("a", "Sheet 1!A1")]),
        &[("a", 2)],
        &[("a", "Sheet 1!B2")],
    );
    let base = with_layout(base, &[("a", 1)], &[("a", "Sheet 1!Pivot 1")]);
    base.write(&path).unwrap();
    assert_eq!(Baseline::read(&path).unwrap(), base);
    // The header says which kind the file is, so a baseline with no `E` item still reads
    // back as recording them, and one without the kind keeps the old header.
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with(HEADER) && !text.contains(ENVELOPE_MARK));
    for outside in [&[("a", "Sheet 1!A1")][..], &[]] {
        for wrong in [outside, &[]] {
            let enveloped = with_wrong(with_outside(base.clone(), outside), wrong);
            enveloped.write(&path).unwrap();
            let text = std::fs::read_to_string(&path).unwrap();
            assert!(text.starts_with(ENVELOPE_HEADER), "{text}");
            assert!(text.contains(WRONG_TYPE_MARK), "{text}");
            assert_eq!(
                text.contains("\nE\ta\tSheet 1!A1\n"),
                !outside.is_empty(),
                "{text}"
            );
            assert_eq!(
                text.contains("\nW\ta\tSheet 1!A1\n"),
                !wrong.is_empty(),
                "{text}"
            );
            assert_eq!(Baseline::read(&path).unwrap(), enveloped);
        }
    }
    // A baseline read with `E` lines but no `W` lines writes each `E` item as a `W` item.
    let legacy = with_outside(base.clone(), &[("a", "Sheet 1!A1")]);
    legacy.write(&path).unwrap();
    assert_eq!(
        Baseline::read(&path).unwrap(),
        with_wrong(legacy, &[("a", "Sheet 1!A1")])
    );
    let _ = std::fs::remove_dir_all(dir);
}
