#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_hyp_geom_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: PDF (non-cumulative)
    model.set_a1("A1", "=HYPGEOM.DIST(1, 4, 12, 20, FALSE)");

    // Valid: CDF (cumulative)
    model.set_a1("A2", "=HYPGEOM.DIST(1, 4, 12, 20, TRUE)");

    // Wrong number of arguments -> #ERROR!
    model.set_a1("A3", "=HYPGEOM.DIST(1, 4, 12, 20)");
    model.set_a1("A4", "=HYPGEOM.DIST(1, 4, 12, 20, TRUE, FALSE)");

    // sample_s > number_sample -> 1 for the cdf (t1049-hypgeomdist A35)
    model.set_a1("A5", "=HYPGEOM.DIST(5, 4, 12, 20, TRUE)");

    // Domain errors:

    // population_s > number_pop -> #NUM!
    model.set_a1("A6", "=HYPGEOM.DIST(1, 4, 25, 20, TRUE)");

    // number_sample > number_pop -> #NUM!
    model.set_a1("A7", "=HYPGEOM.DIST(1, 25, 12, 20, TRUE)");

    model.evaluate();

    // PDF: P(X = 1)
    assert_eq!(model.get_text_a1("A1"), *"0.13869969");

    // CDF: P(X <= 1)
    assert_eq!(model.get_text_a1("A2"), *"0.153147575");

    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"1");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
    assert_eq!(model.get_text_a1("A7"), *"#NUM!");
}

#[test]
fn hyp_geom_dist_domain_edges_and_boolean_sample() {
    let mut model = new_empty_model();
    // probes-numeric/t1048-hypgeomdist and t1049-hypgeomdist cells.
    model.set_a1("A1", "=HYPGEOM.DIST(5,4,8,20,FALSE)");
    model.set_a1("A2", "=HYPGEOM.DIST(1,0,8,20,FALSE)");
    model.set_a1("A3", "=HYPGEOM.DIST(0,0,8,20,FALSE)");
    model.set_a1("A4", "=HYPGEOM.DIST(0,4,0,20,FALSE)");
    model.set_a1("A5", "=HYPGEOM.DIST(-0.5,4,8,20,TRUE)");
    model.set_a1("A6", "=HYPGEOM.DIST(TRUE,4,8,20,TRUE)");
    model.set_a1("A7", "=HYPGEOMDIST(TRUE,4,8,20)");
    model.set_a1("A8", "=HYPGEOM.DIST(1,TRUE,8,20,TRUE)");
    model.evaluate();
    let expected = [
        ("A1", "0"),
        ("A2", "0"),
        ("A3", "1"),
        ("A4", "1"),
        ("A5", "#NUM!"),
        ("A6", "0.465428277"),
        ("A7", "0.363261094"),
        // A TRUE count reads as 1 (statistics.md T-2255, gen-m9 ee2497a5e683c5d9 A356); the value is hand-written.
        ("A8", "1"),
    ];
    for (cell, value) in expected {
        assert_eq!(model.get_text_a1(cell), *value, "{cell}");
    }
}

#[test]
fn a_negative_or_huge_count_is_num() {
    let mut model = new_empty_model();
    // probes/t2429-hypgeom-huge-count and t2429-hypgeom-threshold: a count of 2^31 - 1 or more is
    // #NUM! in any argument, even a sample_s above the possible range (gen-m9 a612e89321a6df9c
    // A395), and a negative fraction in any count is #NUM! before truncation; a text cumulative is
    // read first.
    let cells = [
        ("=HYPGEOMDIST(1E15,2,2,17)", "#NUM!"),
        ("=HYPGEOMDIST(2147483646.9,2,2,17)", "0"),
        ("=HYPGEOMDIST(2147483647,2,2,17)", "#NUM!"),
        ("=HYPGEOM.DIST(2147483647,2,2,17,TRUE)", "#NUM!"),
        ("=HYPGEOMDIST(1,2,3,2147483646)", "2.79397E-09"),
        ("=HYPGEOMDIST(1,2,3,2147483647)", "#NUM!"),
        ("=HYPGEOMDIST(1,2147483647,3,2147483647)", "#NUM!"),
        ("=HYPGEOMDIST(1,3,2147483647,2147483647)", "#NUM!"),
        ("=HYPGEOM.DIST(1,2,-0.5,10,TRUE)", "#NUM!"),
        ("=HYPGEOM.DIST(1,-0.5,3,10,TRUE)", "#NUM!"),
        ("=HYPGEOMDIST(1,2,3,-0.5)", "#NUM!"),
        ("=HYPGEOM.DIST(1E15,2,3,10,\"abc\")", "#VALUE!"),
        ("=HYPGEOM.DIST(1,-0.5,3,10,\"abc\")", "#VALUE!"),
        ("=HYPGEOM.DIST(1,2,3,10,\"true\")", "0.933333333"),
    ];
    for (row, (formula, _)) in cells.iter().enumerate() {
        model.set_a1(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, value)) in cells.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", row + 1)),
            *value,
            "{formula}"
        );
    }
}

#[test]
fn an_empty_population_and_a_pmf_past_the_walk() {
    let mut model = new_empty_model();
    // T-4502, Gate A tables (excel-graph): an empty sample of an empty population is 1
    // (accuracy/HYPGEOMDIST/s1 A115, accuracy/HYPGEOM.DIST/s1 A138), while counts above an empty
    // population stay #NUM! ("Legacy distribution names sweep, second family"); past the
    // double-double walk the pmf is a number (accuracy/HYPGEOMDIST/s1 A199, Excel
    // 1.1128856930493011E-07), no longer statrs' inf / inf, and so is the cdf (accuracy/
    // HYPGEOM.DIST/s1 A164, Excel 0.4538325650607541).
    let cells = [
        ("=HYPGEOMDIST(0,0,0,0)", "1"),
        ("=HYPGEOM.DIST(0,0,0,0,FALSE)", "1"),
        ("=HYPGEOMDIST(1,4,8,0)", "#NUM!"),
        ("=HYPGEOMDIST(4478,5613,21568,27873)", "1.11289E-07"),
        (
            "=HYPGEOM.DIST(43,976865,29261,643257045,TRUE)",
            "0.453832565",
        ),
    ];
    for (row, (formula, _)) in cells.iter().enumerate() {
        model.set_a1(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, value)) in cells.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", row + 1)),
            *value,
            "{formula}"
        );
    }
}

#[test]
fn a_sample_below_the_possible_range_is_zero() {
    let mut model = new_empty_model();
    // gen-m9 c468f5046aadfbbc A85 and probes/t2436 35e8168aae2b00f8: the pmf and the cdf are 0
    // below number_sample + population_s - number_pop; the other domain errors stay #NUM!.
    let cells = [
        ("=HYPGEOMDIST(3,5,5,6)", "0"),
        ("=HYPGEOMDIST(2,5,10,11)", "0"),
        ("=HYPGEOM.DIST(1,2,7,7,FALSE)", "0"),
        ("=HYPGEOM.DIST(1,3,6,7,TRUE)", "0"),
        ("=HYPGEOMDIST(1,8,3,7)", "#NUM!"),
        ("=HYPGEOMDIST(1,3,8,7)", "#NUM!"),
    ];
    for (row, (formula, _)) in cells.iter().enumerate() {
        model.set_a1(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, value)) in cells.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", row + 1)),
            *value,
            "{formula}"
        );
    }
}
