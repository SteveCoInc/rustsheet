#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_t_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: cumulative (left-tail CDF)
    model._set("A1", "=T.DIST(2, 10, TRUE)");
    // Valid: probability density function (PDF)
    model._set("B1", "=T.DIST(2, 10, FALSE)");

    // Wrong number of arguments
    model._set("A2", "=T.DIST(2, 10)");
    model._set("A3", "=T.DIST(2, 10, TRUE, FALSE)");

    // Domain error: df < 1 -> #NUM!
    model._set("A4", "=T.DIST(2, 0, TRUE)");
    model._set("A5", "=T.DIST(2, -1, TRUE)");
    // The density divides by the truncated df: (-1, 1) is #DIV/0!, -1 and below #NUM!
    // (probes-numeric m9-t-edges A27-A42).
    model._set("A6", "=T.DIST(2, 0.5, FALSE)");
    model._set("A7", "=T.DIST(2, -0.5, FALSE)");
    model._set("A8", "=T.DIST(2, -1, FALSE)");
    model._set("A9", "=T.DIST(2, 0.5, TRUE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.963305983");
    assert_eq!(model._get_text("B1"), *"0.061145766");

    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
    assert_eq!(model._get_text("A7"), *"#DIV/0!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
    assert_eq!(model._get_text("A9"), *"#NUM!");
}

#[test]
fn test_fn_t_dist_rt_smoke() {
    let mut model = new_empty_model();

    // Valid: right tail probability
    model._set("A1", "=T.DIST.RT(2, 10)");

    // Wrong number of arguments
    model._set("A2", "=T.DIST.RT(2)");
    model._set("A3", "=T.DIST.RT(2, 10, TRUE)");

    // Domain error: df < 1
    model._set("A4", "=T.DIST.RT(2, 0)");
    model._set("A5", "=T.DIST.RT(2, -1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.036694017");

    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
}

#[test]
fn test_fn_t_dist_2t_smoke() {
    let mut model = new_empty_model();

    // Valid: two-tailed probability
    model._set("A1", "=T.DIST.2T(2, 10)");

    // In the limit case of x = 0, the two-tailed probability is 1.0
    model._set("A4", "=T.DIST.2T(0, 10)");

    // Wrong number of arguments
    model._set("A2", "=T.DIST.2T(2)");
    model._set("A3", "=T.DIST.2T(2, 10, TRUE)");

    // Domain errors:
    // x < 0 -> #NUM!
    model._set("A5", "=T.DIST.2T(-0.001, 10)");
    // df < 1 -> #NUM!
    model._set("A6", "=T.DIST.2T(2, 0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.073388035");
    assert_eq!(model._get_text("A4"), *"1");

    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
}

#[test]
fn test_fn_t_inv_smoke() {
    let mut model = new_empty_model();

    // Valid: upper and lower tail
    model._set("A1", "=T.INV(0.95, 10)");
    model._set("A2", "=T.INV(0.05, 10)");
    // limit case:
    model._set("B2", "=T.INV(0.95, 1)");

    // Wrong number of arguments
    model._set("A3", "=T.INV(0.95)");
    model._set("A4", "=T.INV(0.95, 10, 1)");

    // Domain errors:
    // p <= 0 or >= 1
    model._set("A5", "=T.INV(0, 10)");
    model._set("A6", "=T.INV(1, 10)");
    // df < 1
    model._set("A7", "=T.INV(0.95, 0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.812461123");
    assert_eq!(model._get_text("A2"), *"-1.812461123");
    assert_eq!(model._get_text("B2"), *"6.313751515");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
}

#[test]
fn test_fn_t_inv_2t_smoke() {
    let mut model = new_empty_model();

    // Valid: two-tailed critical values
    model._set("A1", "=T.INV.2T(0.1, 10)");
    model._set("A2", "=T.INV.2T(0.05, 10)");

    // p = 1 should give t = 0 (both tails outside are 1.0, so cut at the mean)
    model._set("A3", "=T.INV.2T(1, 10)");

    model._set("A7", "=T.INV.2T(1.5, 10)");

    // Wrong number of arguments
    model._set("A4", "=T.INV.2T(0.1)");
    model._set("A5", "=T.INV.2T(0.1, 10, 1)");

    // Domain errors:
    // p <= 0 or p >= 2
    model._set("A6", "=T.INV.2T(0, 10)");
    // df < 1
    model._set("A8", "=T.INV.2T(0.1, 0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.812461123");
    assert_eq!(model._get_text("A2"), *"2.228138852");
    assert_eq!(model._get_text("A3"), *"0");

    // Excel returns the negative quantile T.INV(0.25, 10) for p in (1, 2) (m9-t-edges A43, T-2096).
    assert_eq!(model._get_text("A7"), *"-0.699812061");

    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
}

#[test]
fn test_fn_t_dist_arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=T.DIST()");
    model._set("A2", "=T.DIST(8)");
    model._set("A3", "=T.DIST(8, 3)");
    model._set("A4", "=T.DIST(8, 3, TRUE)");
    model._set("A5", "=T.DIST(8, 3, TRUE, 1)");

    model._set("B1", "=T.INV()");
    model._set("B2", "=T.INV(A4)");
    model._set("B3", "=T.INV(A4, 3)");
    model._set("B4", "=T.INV(A4, 3, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"0.997961711");
    assert_eq!(model._get_text("A5"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"8");
    assert_eq!(model._get_text("B4"), *"#ERROR!");
}

// T-1518: nightly fuzz input `T.INV(0.9,5E10)` hung; Excel gives #NUM! above df = 1E10
// (probes-numeric case t1518-t-df-cut), with the pdf staying finite.
#[test]
fn test_t_family_df_upper_limit() {
    let mut model = new_empty_model();
    let over = [
        "=T.INV(0.9,5E10)",
        "=T.INV(0.9,10000000001)",
        "=T.INV.2T(0.1,1.2E10)",
        "=T.DIST(2,1.2E10,TRUE)",
        "=T.DIST.RT(2,1.2E10)",
        "=T.DIST.2T(2,1.2E10)",
        "=CONFIDENCE.T(0.05,1,10000000002)",
    ];
    for (i, f) in over.iter().enumerate() {
        model._set(&format!("A{}", i + 1), f);
    }
    // At the limit (and a fractional df that truncates to it) the functions still evaluate.
    model._set("B1", "=T.INV(0.9,1E10)");
    model._set("B2", "=T.INV(0.9,10000000000.5)");
    model._set("B3", "=T.DIST(2,1E10,TRUE)");
    model._set("B4", "=T.DIST(2,1E11,FALSE)");
    model._set("B5", "=CONFIDENCE.T(0.05,1,10000000001)");
    model.evaluate();

    for i in 1..=over.len() {
        assert_eq!(
            model._get_text(&format!("A{i}")),
            *"#NUM!",
            "{}",
            over[i - 1]
        );
    }
    // TODO(T-0967): Excel's own values here (1.2815518424115688) are not the exact quantile;
    // only the bounded time and the non-error result are asserted.
    for c in ["B1", "B2", "B5"] {
        assert!(model._get_text(c).parse::<f64>().is_ok(), "{c}");
    }
    assert_eq!(model._get_text("B3"), *"0.977249868");
    assert_eq!(model._get_text("B4"), *"0.053990967");
}

#[test]
fn t_inv_2t_above_one_is_the_negative_quantile_of_the_mirror() {
    let mut model = new_empty_model();
    // m9-t-edges A2 and A43: the engine's own mirror is T.INV.2T(2 - p, df), negated bit for bit.
    let pairs = [
        (1.5, 5.0),
        (1.5, 10.0),
        (1.9, 5.0),
        (1.0001, 5.0),
        (1.999, 5.0),
    ];
    for (row, (p, df)) in pairs.iter().enumerate() {
        let row = row + 1;
        model._set(&format!("A{row}"), &format!("=T.INV.2T({p},{df})"));
        model._set(
            &format!("B{row}"),
            &format!("=-T.INV.2T({}, {df})", 2.0 - p),
        );
        model._set(&format!("C{row}"), &format!("=TINV({p},{df})=A{row}"));
        model._set(&format!("D{row}"), &format!("=A{row}=B{row}"));
    }
    model._set("A10", "=T.INV.2T(2,5)");
    model._set("A11", "=T.INV.2T(2.5,5)");
    model._set("A12", "=T.INV.2T(0,5)");
    model._set("A13", "=T.INV.2T(1,5)");
    model.evaluate();
    for row in 1..=pairs.len() {
        assert_eq!(model._get_text(&format!("C{row}")), *"TRUE", "row {row}");
        assert_eq!(model._get_text(&format!("D{row}")), *"TRUE", "row {row}");
    }
    assert_eq!(model._get_text("A2"), *"-0.699812061");
    assert_eq!(model._get_text("A10"), *"#NUM!");
    assert_eq!(model._get_text("A11"), *"#NUM!");
    assert_eq!(model._get_text("A12"), *"#NUM!");
    assert_eq!(model._get_text("A13"), *"0");
}

#[test]
fn t_density_flushes_a_subnormal_result() {
    // Excel's df >= 3 density is 0 where its last product is subnormal ((x)=0 is TRUE, x*1E300 is 0;
    // Gate A T.DIST/r4-T-2585 A138), while df 2's EXP keeps it (statistics.md "Student t density is
    // Loader's dt", T-2585).
    let mut model = new_empty_model();
    model._set("A1", "=T.DIST(-1.1002374143529E+20,15,FALSE)");
    model._set("A2", "=T.DIST(1E+78,3,FALSE)*1E300");
    model._set("A3", "=T.DIST(1E+104,2,FALSE)*1E300");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"1E-12");
}

#[test]
fn t_density_is_num_where_x_squared_overflows() {
    // Excel's density squares x first: from 2^512, where fl(x x) overflows, it is #NUM! at every df
    // and either sign, while 2^512's predecessor is a number (0 here: the density underflows)
    // (probes/t3554-tdist-density-huge-x A2:B13 and A20:B24; statistics.md "T.DIST's density is
    // #NUM! at a huge x").
    let mut model = new_empty_model();
    model._set("H1", "=2^512");
    model._set("H2", "=2^512*(1-2^-52)");
    for (row, df) in ["2", "24", "1E+10", "1E+300"].iter().enumerate() {
        let row = row + 1;
        model._set(&format!("A{row}"), &format!("=T.DIST(H1,{df},FALSE)"));
        model._set(&format!("B{row}"), &format!("=T.DIST(-H1,{df},FALSE)"));
        model._set(&format!("C{row}"), &format!("=T.DIST(H2,{df},FALSE)"));
        model._set(&format!("D{row}"), &format!("=T.DIST(-H2,{df},FALSE)"));
    }
    // Gate A accuracy/T.DIST/s1 A85 and e-T-3425 A103.
    model._set("E1", "=T.DIST(1.01778553276442E+236,2,FALSE)");
    model._set("E2", "=T.DIST(-4.62236657470614E+224,11,FALSE)");
    model._set("E3", "=T.DIST(1E+160,1,FALSE)");
    model.evaluate();
    for row in 1..=4 {
        assert_eq!(model._get_text(&format!("A{row}")), *"#NUM!", "row {row}");
        assert_eq!(model._get_text(&format!("B{row}")), *"#NUM!", "row {row}");
        assert_eq!(model._get_text(&format!("C{row}")), *"0", "row {row}");
        assert_eq!(model._get_text(&format!("D{row}")), *"0", "row {row}");
    }
    for cell in ["E1", "E2", "E3"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
}

#[test]
fn t_density_at_df_one_is_div0_where_its_denominator_overflows() {
    // df 1's density is 1/fl(pi fl(1 + fl(x x))): #DIV/0! once the denominator overflows (from
    // 7.6E+153, while 7.55E+153 is a number), and a subnormal quotient is 0
    // (probes/t3554-tdist-density-df1-div0; statistics.md "T.DIST's density is #NUM! at a huge x").
    let mut model = new_empty_model();
    model._set("A1", "=T.DIST(7.6E+153,1,FALSE)");
    model._set("A2", "=T.DIST(-1E+154,1,FALSE)");
    model._set("A3", "=T.DIST(7.6E+153,1.5,FALSE)");
    model._set("A4", "=T.DIST(7.55E+153,1,FALSE)");
    model._set("A5", "=T.DIST(-3.8E+153,1,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0");
}
