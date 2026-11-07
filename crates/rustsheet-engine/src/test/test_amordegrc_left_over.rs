#![allow(clippy::unwrap_used)]
//! AMORDEGRC below period 1 returns the value the last AMORDEGRC or AMORLINC Excel ran before it
//! left, in Excel's fresh-load order (docs/excel-semantics/financial.md "AMORDEGRC below period
//! 1", decision 0131; the order is SPEC-0017's, `excel_order.rs`). Excel runs the cells bottom row
//! first and each row right to left.

use crate::test::util::new_empty_model;
use crate::Model;

const FRACTION: &str = "=AMORDEGRC(10000,39000,39400,500,0.3,0.3,0)";
/// 485, and it leaves 485.
const CHARGE_485: &str = "=AMORDEGRC(2400,39679,39813,300,2,0.15,1)";
/// 776, and it leaves 776.
const CHARGE_776: &str = "=AMORDEGRC(2400,39679,39813,300,1,0.15,1)";
/// AMORLINC's 360, and it leaves 132 (its first period's charge, rounded).
const LINEAR_132: &str = "=AMORLINC(2400,39679,39813,300,1,0.15,1)";

fn evaluated(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for (cell, formula) in cells {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    model
}

/// The fraction's value in A1 with `below` in A2 and the 485 charge in A3.
fn above(below: &str) -> String {
    let model = evaluated(&[("A1", FRACTION), ("A2", below), ("A3", CHARGE_485)]);
    model.get_text_a1("A1")
}

#[test]
fn alone_it_is_0_and_runs_one_pass_only_without_a_reader() {
    let model = evaluated(&[("A1", FRACTION)]);
    assert_eq!(model.get_text_a1("A1"), "0");
    let model = evaluated(&[("A1", CHARGE_485), ("A2", CHARGE_776)]);
    assert_eq!(model.evaluation.passes_in_last_evaluation, 1);
    assert!(!model.evaluation.excel_order);
}

#[test]
fn it_reads_the_cell_excel_ran_last() {
    // probes/amordegrc-fraction-state-p, -q, -n and alone-e.
    assert_eq!(above(CHARGE_776), "776");
    let model = evaluated(&[("A1", FRACTION), ("B1", CHARGE_485)]);
    assert_eq!(model.get_text_a1("A1"), "485");
    let model = evaluated(&[("A1", CHARGE_485), ("A2", FRACTION)]);
    assert_eq!(model.get_text_a1("A2"), "0");
    let mut model = new_empty_model();
    model.new_sheet();
    model.set_a1("A1", FRACTION);
    model.set_a1("Sheet2!A1", CHARGE_485);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "485");
}

#[test]
fn a_precedent_runs_first() {
    // probes/amordegrc-fraction-state-o.
    let model = evaluated(&[
        ("A1", CHARGE_485),
        ("A2", "=AMORDEGRC(10000,39000,39400,500,0.3+0*A1,0.3,0)"),
    ]);
    assert_eq!(model.get_text_a1("A2"), "485");
}

#[test]
fn what_each_call_leaves() {
    // AMORLINC leaves its first period's charge rounded (131.803... and 360); a period-0
    // AMORDEGRC, an error and a period past the life leave the value as it was
    // (probes/amordegrc-fraction-state-j, -y, -z, -m, -u, -v).
    assert_eq!(above("=AMORLINC(2400,39679,39813,300,1,0.15,1)"), "132");
    assert_eq!(above("=AMORLINC(2400,39813,39813,300,1,0.15,1)"), "360");
    assert_eq!(above("=AMORDEGRC(46557,0,39881,440,0,0.1)"), "485");
    assert_eq!(above("=AMORDEGRC(2400,39679,39813,-1,2,0.15,1)"), "485");
    assert_eq!(above("=AMORDEGRC(2400,39679,39813,300,20,0.15,1)"), "485");
    assert_eq!(above("=AMORLINC(2400,39679,39813,300,20,0.15,1)"), "485");
    // Other functions leave it as it was (-aa, -ab).
    assert_eq!(above("=EFFECT(1,1)"), "485");
    assert_eq!(above("=SUM(1,2)"), "485");
}

#[test]
fn a_salvage_equal_to_the_cost_leaves_the_value_and_reads_none() {
    // Both functions, at any period, leave the value as it was, and an AMORDEGRC below period 1
    // is 0 rather than the value (probes/amor-salvage-equal-cost F1 to F8, T-4504).
    assert_eq!(above("=AMORDEGRC(2400,39679,39813,2400,1,0.15,1)"), "485");
    assert_eq!(above("=AMORDEGRC(2400,39679,39813,2400,0,0.15,1)"), "485");
    assert_eq!(above("=AMORLINC(2400,39679,39813,2400,1,0.15,1)"), "485");
    assert_eq!(above("=AMORLINC(2400,39679,39813,2400,0,0.15,1)"), "485");
    let model = evaluated(&[
        ("A1", "=AMORDEGRC(2400,39679,39813,2400,0.5,0.15,1)"),
        ("A2", CHARGE_485),
    ]);
    assert_eq!(model.get_text_a1("A1"), "0");
}

#[test]
fn lifted_over_an_array_it_is_0() {
    // probes/amordegrc-fraction-state-x.
    let model = evaluated(&[
        (
            "A1",
            "=SUM(AMORDEGRC(10000,39000,39400,500,{0.3,0.5},0.3,0))",
        ),
        ("A2", CHARGE_485),
    ]);
    assert_eq!(model.get_text_a1("A1"), "0");
}

#[test]
fn a_stopped_run_leaves_the_value_as_it_was() {
    // A2 leaves 776 and then reads A1, not yet run: that run stops and A1 must read A3's 485.
    let model = evaluated(&[
        ("A1", FRACTION),
        ("A2", &format!("{CHARGE_776}+0*A1")),
        ("A3", CHARGE_485),
    ]);
    assert_eq!(model.get_text_a1("A1"), "485");
    assert_eq!(model.get_text_a1("A2"), "776");
}

#[test]
fn it_shares_the_order_with_weeknum_but_not_the_state() {
    // A WEEKNUM reader and a fraction on one sheet: each reads its own state.
    let model = evaluated(&[
        ("A1", FRACTION),
        ("B1", "=WEEKNUM(100,99)"),
        ("A2", CHARGE_776),
        ("B2", "=WEEKNUM(100,2)"),
    ]);
    assert_eq!(model.get_text_a1("A1"), "776");
    assert_eq!(model.get_text_a1("B1"), "16");
}

/// A1 holds a lifted AMORDEGRC over `periods`.
fn lifted(periods: &str) -> Model {
    let formula = format!("=AMORDEGRC(2400,39679,39813,300,{periods},0.15,1)");
    let mut model = new_empty_model();
    model.set_a1("A1", &formula);
    model.evaluate();
    model
}

#[test]
fn a_lifted_call_reads_what_its_own_earlier_elements_left() {
    // probes/amordegrc-fraction-state-ae and -af: {1,0.5} is 776 + 776, {0.5,1} is 0 + 776.
    let model = evaluated(&[("A1", "=SUM(AMORDEGRC(2400,39679,39813,300,{1,0.5},0.15,1))")]);
    assert_eq!(model.get_text_a1("A1"), "1552");
    let model = evaluated(&[("A1", "=SUM(AMORDEGRC(2400,39679,39813,300,{0.5,1},0.15,1))")]);
    assert_eq!(model.get_text_a1("A1"), "776");
}

/// An error element leaves the value as it was, as a scalar error does: `{1,-1,0.5}` is 776,
/// `#NUM!`, 776 (probes/amordegrc-fraction-state-as, -at; T-3393).
#[test]
fn an_error_element_stays_an_error_beside_a_fraction() {
    let model = lifted("{-1,0.5}");
    assert_eq!(model.get_text_a1("A1"), "#NUM!");
    assert_eq!(model.get_text_a1("B1"), "0");
    let model = lifted("{1,-1,0.5}");
    assert_eq!(model.get_text_a1("A1"), "776");
    assert_eq!(model.get_text_a1("B1"), "#NUM!");
    assert_eq!(model.get_text_a1("C1"), "776");
}

#[test]
fn the_calls_excel_lists_empty_the_value_after_reading_their_arguments() {
    // probes/amordegrc-fraction-state-ag, -an, -az to -cw: each of these calls between the
    // fraction and the 485 charge empties the value it reads.
    for below in [
        "=SEQUENCE(1,2)",
        "=SEQUENCE(1,1)",
        "=SUM(SEQUENCE(1,2))",
        "=MUNIT(1)",
        "=SORT({2,1})",
        "=SUM(XLOOKUP(1,{1,2},{3,4}))",
        "=SUM(VSTACK(1,2))",
        "=SUM(TAKE({1,2},1))",
        "=SUM(DROP({1,2},1))",
        "=SUM(TEXTJOIN(\",\",,{1,2}))",
        "=SUM(MMULT({1},{1}))",
        "=SUM(LAMBDA(x,x)(1))",
        "=SUM(MAP({1,2},LAMBDA(x,x)))",
    ] {
        assert_eq!(above(below), "0", "{below}");
    }
    // ... and these keep it (-aq, -ax, -ba, -bg, -ca, -ce, -cm to -cp, -cw, and an IF arm that is
    // not taken, -be).
    for below in [
        "={1,2}",
        "=TRANSPOSE({1;2})",
        "=ABS({1,2})",
        "=COLUMN(A1:B1)",
        "=SUM(MDETERM({1}))",
        "=SUM(LET(x,1,x))",
        "=SUM(INDEX({1,2},1,1))",
        "=SUM(MATCH(1,{1,2},0))",
        "=SUM(SUMPRODUCT({1,2}))",
        "=SUM(ROWS({1,2}))",
        "=IF(FALSE,SEQUENCE(1,2),1)",
    ] {
        assert_eq!(above(below), "485", "{below}");
    }
    // The call empties the value after its arguments ran: a charge inside it is not read
    // (-cx, -da).
    let model = evaluated(&[
        ("A1", FRACTION),
        (
            "A2",
            "=SUM(SEQUENCE(1,1,AMORDEGRC(2400,39679,39813,300,2,0.15,1)))",
        ),
    ]);
    assert_eq!(model.get_text_a1("A1"), "0");
}

#[test]
fn a_dynamic_array_anchor_leaves_its_value_at_its_place_in_the_order() {
    // probes/amordegrc-fraction-state-ah: the anchor in A2 empties the value and its charge sets
    // 485 after the AMORLINC in A3 ran, so the fraction reads 485, not the AMORLINC's 132.
    let below = "=SEQUENCE(1,2)+AMORDEGRC(2400,39679,39813,300,2,0.15,1)";
    let model = evaluated(&[("A1", FRACTION), ("A2", below), ("A3", LINEAR_132)]);
    assert_eq!(model.get_text_a1("A1"), "485");
    // -am: the charge first and SEQUENCE second leaves it empty.
    let below = "=AMORDEGRC(2400,39679,39813,300,2,0.15,1)+SEQUENCE(1,2)";
    let model = evaluated(&[("A1", FRACTION), ("A2", below), ("A3", LINEAR_132)]);
    assert_eq!(model.get_text_a1("A1"), "0");
    // -ap: an anchor that runs after the 776 charge below it leaves its own 485.
    let below = "=SEQUENCE(1,2)*0+AMORDEGRC(2400,39679,39813,300,2,0.15,1)";
    let model = evaluated(&[("A1", FRACTION), ("A2", below), ("A3", CHARGE_776)]);
    assert_eq!(model.get_text_a1("A1"), "485");
    // -ao: a SEQUENCE below the charge runs first and empties nothing the fraction reads.
    let model = evaluated(&[
        ("A1", FRACTION),
        ("A2", CHARGE_485),
        ("A3", "=SEQUENCE(1,2)"),
    ]);
    assert_eq!(model.get_text_a1("A1"), "485");
}
