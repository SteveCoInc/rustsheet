#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_offset_reference() {
    let mut model = new_empty_model();
    model.set_a1("B1", "12");
    model.set_a1("B2", "13");
    model.set_a1("B3", "15");

    model.set_a1("A1", "=SUM(B1:OFFSET($B$1,3,0))");
    model.set_a1("A2", "=SUM(OFFSET(A1, 1, 1):B3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"40");
    assert_eq!(model.get_text_a1("A2"), *"28");
}

// `@` meets OFFSET in the two directions that route through `get_reference`'s
// fallback arm (cast.rs): the `@` node must reach it as a reference (the
// intersected 1x1 range), not be dereferenced to a value.

#[test]
fn fn_offset_of_at_range() {
    // `@` UNDER OFFSET: the ref argument is an @-wrapped range. In D2, @B1:B4
    // intersects to B2; OFFSET(B2, 1, 0) -> B3.
    let mut model = new_empty_model();
    model.set_a1("B1", "10");
    model.set_a1("B2", "20");
    model.set_a1("B3", "30");
    model.set_a1("B4", "40");
    model.set_a1("D2", "=OFFSET(@B1:B4, 1, 0)");
    model.evaluate();

    assert_eq!(model.get_text_a1("D2"), *"30");
}

#[test]
fn fn_rows_of_at_offset() {
    // `@` OVER OFFSET: OFFSET(B1,0,0,4,1) -> B1:B4; `@` in D2 intersects it to
    // B2, a single cell, so ROWS is 1 (not 4).
    let mut model = new_empty_model();
    model.set_a1("B1", "10");
    model.set_a1("B2", "20");
    model.set_a1("B3", "30");
    model.set_a1("B4", "40");
    model.set_a1("D2", "=ROWS(@OFFSET(B1, 0, 0, 4, 1))");
    model.evaluate();

    assert_eq!(model.get_text_a1("D2"), *"1");
}

// An empty height or width argument keeps the reference's size
// (poi/FormulaEvalTestData_Copy.xlsx EverythingTests row 1008).
#[test]
fn fn_offset_empty_height_width() {
    let mut model = new_empty_model();
    model.set_a1("K7", "1");
    model.set_a1("L7", "1.001");
    model.set_a1("K8", "2");
    model.set_a1("L8", "2.9999");
    model.set_a1("A1", "=SUM(OFFSET(K7:L8, 0,0,1,))");
    model.set_a1("A2", "=SUM(OFFSET(K7:L8, 0,0,,1))");
    model.set_a1("A3", "=SUM(OFFSET(K7:L8, 0,0,,))");
    model.set_a1("A4", "=SUM(OFFSET(K7:L8, 0,0,))");
    model.set_a1("A5", "=SUM(OFFSET(K7:L8, 0,0,1,1))");
    model.set_a1("A6", "=SUM(OFFSET(K7:L8, 0,0,1))");
    model.set_a1("A7", "=SUM(OFFSET(K7:L8, 0,0))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"2.001");
    assert_eq!(model.get_text_a1("A2"), *"3");
    assert_eq!(model.get_text_a1("A3"), *"7.0009");
    assert_eq!(model.get_text_a1("A4"), *"7.0009");
    assert_eq!(model.get_text_a1("A5"), *"1");
    assert_eq!(model.get_text_a1("A6"), *"2.001");
    assert_eq!(model.get_text_a1("A7"), *"7.0009");
}

// Huge arguments saturate instead of overflowing i32 (untrusted input must not panic).
#[test]
fn fn_offset_huge_arguments_are_ref_errors() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=OFFSET(B2,1E10,0)");
    model.set_a1("A2", "=OFFSET(B2,-1E10,0)");
    model.set_a1("A3", "=OFFSET(B2,0,1E10)");
    model.set_a1("A4", "=OFFSET(B2,0,-1E10)");
    model.set_a1("A5", "=OFFSET(B2,1,0,1E10)");
    model.set_a1("A6", "=OFFSET(B2,1,0,1,1E10)");
    model.set_a1("A7", "=OFFSET(B2,1,0,-1E10)");
    model.set_a1("A8", "=OFFSET(B2,1,0,1,-1E10)");
    model.set_a1("A9", "=OFFSET(B2,1E300,1E300,1E300,1E300)");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8"] {
        assert_eq!(model.get_text_a1(cell), *"#REF!", "{cell}");
    }
    // A9 only pins "no panic": the result for 1E300 arguments has no oracle evidence.
}

// A negative height/width extends the area up/left from the anchor; fractions and 0 are quirky.
// Excel: probes/offset-negative-height-width (docs/excel-semantics/references.md).
#[test]
fn fn_offset_negative_height_width() {
    let mut model = new_empty_model();
    for r in 1..=12 {
        model.set_a1(&format!("A{r}"), &r.to_string());
        model.set_a1(&format!("B{r}"), &(r * 100).to_string());
    }
    model.set_a1("D1", "=SUM(OFFSET(A10,0,0,-3,1))");
    model.set_a1("D2", "=SUM(OFFSET(B10,0,0,1,-2))");
    model.set_a1("D3", "=SUM(OFFSET(B10,0,0,-2,-2))");
    model.set_a1("D4", "=SUM(OFFSET(A5:B6,1,1,-2,-2))");
    model.set_a1("D5", "=SUM(OFFSET(A2,0,0,-3,1))");
    model.set_a1("D6", "=OFFSET(A10,0,0,0,1)");
    model.set_a1("D7", "=SUM(OFFSET(A10,0,0,0.5,1))");
    model.set_a1("D8", "=SUM(OFFSET(A10,0,0,-0.5,1))");
    model.set_a1("D9", "=SUM(OFFSET(A10,0,0,-2.9,1))");
    model.set_a1("D10", "=SUM(OFFSET(A5:B6,0,0,,-2))");
    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), *"27");
    assert_eq!(model.get_text_a1("D2"), *"1010");
    assert_eq!(model.get_text_a1("D3"), *"1919");
    assert_eq!(model.get_text_a1("D4"), *"1111");
    assert_eq!(model.get_text_a1("D5"), *"#REF!");
    assert_eq!(model.get_text_a1("D6"), *"#REF!");
    assert_eq!(model.get_text_a1("D7"), *"19");
    assert_eq!(model.get_text_a1("D8"), *"21");
    assert_eq!(model.get_text_a1("D9"), *"19");
    assert_eq!(model.get_text_a1("D10"), *"#REF!");
}

// SPEC-0014: OFFSET lifted over an array is an array of references. It never reaches a cell:
// spilled, every reference element is `#VALUE!` and an error element stays itself.
#[test]
fn fn_offset_lifted_spills_its_value_view() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("C1", "=OFFSET(A1,{0;-1;1},0)");
    model.set_a1("D1", "=ROWS(OFFSET(A1,{0;1},NA()))");
    model.set_a1("E1", "=INDEX(OFFSET(A1,{0;1},0),2,1)");
    model.set_a1("F1", "=ISREF(OFFSET(A1,{0;1},0))");
    model.set_a1("G1", "=ISREF(INDEX(OFFSET(A1,{0;1},0),1,1))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C2"), *"#REF!");
    assert_eq!(model.get_text_a1("C3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("D1"), *"2");
    assert_eq!(model.get_text_a1("E1"), *"2");
    assert_eq!(model.get_text_a1("F1"), *"FALSE");
    assert_eq!(model.get_text_a1("G1"), *"TRUE");
}

// The broadcast shape is checked before any element is computed (CLAUDE.md directive 5).
#[test]
fn fn_offset_lift_past_the_array_bound_is_an_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=ROWS(OFFSET(B1,SEQUENCE(2000),SEQUENCE(1,1000)))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
}

#[test]
fn ref_element_is_small() {
    // SPEC-0014 "Bounds": no larger than the `ArrayNode` it stands in for.
    assert!(std::mem::size_of::<crate::reference_operators::RefElement>() <= 32);
}

// SPEC-0014 rules 4-5 (T-2063): SUM, AVERAGE, MIN, MAX and PRODUCT read each element's top-left
// cell; COUNT counts no element, COUNTA every one; SUMPRODUCT reads an element as 0 and an error
// element as its error (probes/reference-lift-consumers E1-E29, -b2 E117-E189).
#[test]
fn fn_offset_lifted_into_top_left_aggregates() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "100");
    model.set_a1("A3", "10000");
    model.set_a1("A4", "'5");
    model.set_a1("A6", "TRUE");
    model.set_a1("A7", "7");
    model.set_a1("C1", "=SUM(OFFSET(A1,0,0,{1;2}))");
    model.set_a1("C2", "=AVERAGE(OFFSET(A1,{0;1},0))");
    model.set_a1("C3", "=MAX(OFFSET(A1,0,0,{1;3}))");
    model.set_a1("C4", "=MIN(OFFSET(A4,0,0,{1;3}))");
    model.set_a1("C5", "=PRODUCT(OFFSET(A1,{0;1},0))");
    model.set_a1("C6", "=SUM(OFFSET(A1,{0;1},0),1)");
    model.set_a1("C7", "=AVERAGE(OFFSET(A1,{0;-1},0))");
    model.set_a1("C8", "=COUNT(OFFSET(A1,0,0,{1;2}))");
    model.set_a1("C9", "=COUNTA(OFFSET(A5,0,0,{1;2}))");
    model.set_a1("C10", "=SUMPRODUCT(OFFSET(A1,{0;1},0))");
    model.set_a1("C11", "=SUMPRODUCT(OFFSET(A1,{0;-1},0))");
    model.set_a1("C12", "=SUM(OFFSET(A4,{0;3},0))");
    model.set_a1("C13", "=SUM(OFFSET(A6,0,0,{1;2}))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"2");
    assert_eq!(model.get_text_a1("C2"), *"50.5");
    assert_eq!(model.get_text_a1("C3"), *"1");
    assert_eq!(model.get_text_a1("C4"), *"0");
    assert_eq!(model.get_text_a1("C5"), *"100");
    assert_eq!(model.get_text_a1("C6"), *"102");
    assert_eq!(model.get_text_a1("C7"), *"#REF!");
    assert_eq!(model.get_text_a1("C8"), *"0");
    assert_eq!(model.get_text_a1("C9"), *"2");
    assert_eq!(model.get_text_a1("C10"), *"0");
    assert_eq!(model.get_text_a1("C11"), *"#REF!");
    assert_eq!(model.get_text_a1("C12"), *"7");
    assert_eq!(model.get_text_a1("C13"), *"0");
}

// STDEV.S, VAR.S and SUMSQ read each element of an array of references by its top-left cell, as
// SUM does, and so do VAR.P, STDEV.P and legacy VAR; AVERAGEA, MAXA, MINA and the VARA family read
// that cell as an array element, so a blank or TRUE is skipped, AVERAGEA counts text as 0 and the
// others skip it. Evidence: T-2824, probes/t2065-lift-* and probes/t2816-lift-*; T-3533,
// probes/t3533-lift-*; excel-graph. The data is the probes': A1:A3 1, 100, 10000, B1:B3 2, 200,
// 20000, A4 "5", A5 blank, A6 TRUE, A7 7, A8 -3 (t3533 only).
#[test]
fn fn_offset_lifted_into_the_statistical_aggregates_reads_top_left_cells() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "100"),
        ("A3", "10000"),
        ("B1", "2"),
        ("B2", "200"),
        ("B3", "20000"),
        ("A4", "'5"),
        ("A6", "TRUE"),
        ("A7", "7"),
        ("A8", "-3"),
    ] {
        model.set_a1(cell, value);
    }
    let cases = [
        // t2065: single-cell elements.
        ("=STDEV.S(OFFSET(A1,{0;1},0))", "70.00357134"),
        ("=VAR.S(OFFSET(A1,{0;1},0))", "4900.5"),
        ("=SUMSQ(OFFSET(A1,{0;1},0))", "10001"),
        ("=AVERAGEA(OFFSET(A4,{0;1;2},0))", "0"),
        ("=MAXA(OFFSET(A4,{0;1;2},0))", "0"),
        // t2816: multi-cell elements count by their top-left cell, not by the formula's row.
        ("=STDEV.S(OFFSET(A1,0,0,{1;2}))", "0"),
        ("=VAR.S(OFFSET(A1,0,0,{1;2}))", "0"),
        ("=SUMSQ(OFFSET(A1,0,0,{1;2}))", "2"),
        ("=SUMSQ(OFFSET(A2,0,0,{1;2}))", "20000"),
        // t2816: text is 0, a blank or TRUE element is skipped.
        ("=AVERAGEA(OFFSET(A4,{0;3},0))", "3.5"),
        ("=MAXA(OFFSET(A4,{0;3},0))", "7"),
        ("=AVERAGEA(OFFSET(A5,{0;1},0))", "#DIV/0!"),
        ("=MAXA(OFFSET(A5,{0;1},0))", "0"),
        // t3533: text, a blank and TRUE are skipped, multi-cell elements read their top-left cell.
        ("=MAXA(OFFSET(A4,{0;4},0))", "-3"),
        ("=MINA(OFFSET(A4,{0;3},0))", "7"),
        ("=MINA(OFFSET(A6,{0;1},0))", "7"),
        ("=MINA(OFFSET(A1,{0;1},0,1,2))", "1"),
        ("=VAR.P(OFFSET(A4,{0;3;4},0))", "25"),
        ("=STDEV.P(OFFSET(A1,{0;1},0,1,2))", "49.5"),
        ("=VAR(OFFSET(A5,{0;2;3},0))", "50"),
        ("=VARA(OFFSET(A4,{0;3;4},0))", "50"),
        ("=VARPA(OFFSET(A6,{0;1;2},0))", "25"),
        ("=STDEVA(OFFSET(A5,{0;2;3},0))", "7.071067812"),
        ("=STDEVPA(OFFSET(A1,{0;1},0,1,2))", "49.5"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("E{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("E{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

// MEDIAN keeps the value view of an array of references, every reference element `#VALUE!`, as
// Excel (probes/t2065-lift-median, lookup.md "MEDIAN, LARGE and SMALL over a lifted OFFSET").
#[test]
fn fn_offset_lifted_into_median_is_its_value_view() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "100");
    model.set_a1("C1", "=MEDIAN(OFFSET(A1,{0;1},0))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#VALUE!");
}

// LET, a LAMBDA parameter, IF and CHOOSE hand an array of references on, and OFFSET over one
// lifts again (SPEC-0014 rule 7; probes/reference-lift-consumers E89, E145, E149,
// reference-lift-pass-through).
#[test]
fn fn_offset_grid_passes_through_let_if_choose_and_nested_offset() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "100");
    model.set_a1("A3", "10000");
    model.set_a1("C1", "=LET(x,OFFSET(A1,{0;1},0),INDEX(x,2,1))");
    model.set_a1(
        "C2",
        "=LET(x,OFFSET(A1,{0;1},0),y,x,SUM(INDEX(y,2,1),INDEX(x,1,1)))",
    );
    model.set_a1("C3", "=LET(f,LAMBDA(r,INDEX(r,2,1)),f(OFFSET(A1,{0;1},0)))");
    model.set_a1("C4", "=ROWS(OFFSET(OFFSET(A1,{0;1},0),1,0))");
    model.set_a1("C5", "=SUM(OFFSET(OFFSET(A1,{0;1},0),{0,1},0))");
    model.set_a1(
        "C6",
        "=ERROR.TYPE(INDEX(OFFSET(OFFSET(A1,{0;-1},0),1,0),2,1))",
    );
    model.set_a1("C7", "=SUM(OFFSET(LET(x,OFFSET(A1,{0;1},0),x),1,0))");
    model.set_a1("C8", "=SUMPRODUCT(IF(TRUE,OFFSET(A1,{0;1},0)))");
    model.set_a1("C9", "=LET(x,OFFSET(A1,{0;1},0),ISREF(x))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"100");
    assert_eq!(model.get_text_a1("C2"), *"101");
    assert_eq!(model.get_text_a1("C3"), *"100");
    assert_eq!(model.get_text_a1("C4"), *"2");
    assert_eq!(model.get_text_a1("C5"), *"10201");
    assert_eq!(model.get_text_a1("C6"), *"4");
    assert_eq!(model.get_text_a1("C7"), *"10100");
    assert_eq!(model.get_text_a1("C8"), *"0");
    assert_eq!(model.get_text_a1("C9"), *"FALSE");
}

// Each level of a deep OFFSET nest over an array of references is one lift of the grid's
// elements, so the work grows with depth times elements, not exponentially (SPEC-0014 "Bounds").
#[test]
fn fn_offset_deep_nest_over_a_grid_stays_bounded() {
    let mut model = new_empty_model();
    let depth = 60;
    let mut formula = "OFFSET(A1,SEQUENCE(1000)-1,0)".to_string();
    for _ in 0..depth {
        formula = format!("OFFSET({formula},{{0,1}},0)");
    }
    model.set_a1("C1", &format!("=ROWS({formula})"));
    model.set_a1("C2", &format!("=COLUMNS({formula})"));
    model.set_a1("C3", &format!("=COUNTA({formula})"));
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"1000");
    assert_eq!(model.get_text_a1("C2"), *"2");
    assert_eq!(model.get_text_a1("C3"), *"2000");
}

// SPEC-0014 rule 6: a function that takes a reference calls itself once per element of an array
// of references, the whole element in its place (probes/reference-lift-consumers E57-E77,
// E97-E113, E165, E169; -b1c E85-E93; -b1d E97, E101; -b2 E125-E133, E193, E197, E225).
#[test]
fn fn_offset_lifted_into_per_element_callers() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "100");
    model.set_a1("A3", "10000");
    model.set_a1("B1", "2");
    model.set_a1("B2", "200");
    model.set_a1("A4", "'5");
    model.set_a1("C1", "=COUNTIF(OFFSET(A1,{0;1},0),\">0\")");
    model.set_a1(
        "D1",
        "=SUMIF(OFFSET(A1,0,0,{1;2}),\">0\",OFFSET(B1,0,0,{1;2}))",
    );
    model.set_a1(
        "E1",
        "=SUMIFS(OFFSET(B1,{0;1},0),OFFSET(A1,{0;1},0),\">0\")",
    );
    model.set_a1("F1", "=SUBTOTAL(9,OFFSET(A1,{0;1},0),A1)");
    model.set_a1("G1", "=ROW(OFFSET(A1,{0;1},0))");
    model.set_a1("H1", "=SUMPRODUCT(SUBTOTAL(9,OFFSET(A1,{0;1;2},0)))");
    model.set_a1("I1", "=T(OFFSET(A4,{0;1},0))");
    model.set_a1("J1", "=SHEET(OFFSET(A1,{0;1},0))");
    model.set_a1("K1", "=SUM(VSTACK(OFFSET(A1,{0;1},0)))");
    model.set_a1("L1", "=AREAS(OFFSET(A1,{0;-1},0))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"1");
    assert_eq!(model.get_text_a1("C2"), *"1");
    assert_eq!(model.get_text_a1("D1"), *"2");
    assert_eq!(model.get_text_a1("D2"), *"202");
    assert_eq!(model.get_text_a1("E1"), *"2");
    assert_eq!(model.get_text_a1("E2"), *"200");
    assert_eq!(model.get_text_a1("F1"), *"2");
    assert_eq!(model.get_text_a1("F2"), *"101");
    assert_eq!(model.get_text_a1("G1"), *"1");
    assert_eq!(model.get_text_a1("G2"), *"2");
    assert_eq!(model.get_text_a1("H1"), *"10101");
    assert_eq!(model.get_text_a1("I1"), *"5");
    assert_eq!(model.get_text_a1("I2"), *"");
    assert_eq!(model.get_text_a1("J1"), *"#N/A");
    assert_eq!(model.get_text_a1("K1"), *"101");
    assert_eq!(model.get_text_a1("L1"), *"1");
    assert_eq!(model.get_text_a1("L2"), *"#REF!");
}

// COUNTIFS, AVERAGEIF and MAXIFS lift per element too, one value each (probes/t2065-lift-countifs,
// -averageif, -maxifs, excel-graph 2026-10-26), and charge every criteria walk of every element.
#[test]
fn fn_offset_lifted_into_countifs_averageif_maxifs() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "100");
    model.set_a1("A1048576", "1");
    model.set_a1("C1", "=COUNTIFS(OFFSET(A1,{0;1},0),\">0\")");
    model.set_a1("D1", "=AVERAGEIF(OFFSET(A1,0,0,{1;2}),\">0\")");
    model.set_a1(
        "E1",
        "=MAXIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),\">0\")",
    );
    model.set_a1("F1", "=COUNTIFS(OFFSET(A:A,0,0,,SEQUENCE(1,1000)),\">0\")");
    model.set_a1(
        "G1",
        "=MAXIFS(OFFSET(A:A,0,0,,SEQUENCE(1,1000)),A:A,\">0\")",
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"1");
    assert_eq!(model.get_text_a1("C2"), *"1");
    assert_eq!(model.get_text_a1("D1"), *"1");
    assert_eq!(model.get_text_a1("D2"), *"50.5");
    assert_eq!(model.get_text_a1("E1"), *"1");
    assert_eq!(model.get_text_a1("E2"), *"100");
    assert_eq!(model.get_text_a1("F1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("G1"), *"#ERROR!");
}

// AVERAGEIFS and MINIFS lift like SUMIFS and MAXIFS: the first range and every criteria range,
// one value each (probes/t4396-lift-*, excel-graph 2026-11-05). A lifted first range against a
// two-cell criteria range is a shape mismatch per element, and the bound charges every pair.
#[test]
fn fn_offset_lifted_into_averageifs_minifs() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "100");
    model.set_a1("B1", "2");
    model.set_a1("B2", "200");
    model.set_a1("A1048576", "1");
    model.set_a1(
        "C1",
        "=AVERAGEIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),\">0\",OFFSET(B1,{0;1},0),\">0\")",
    );
    model.set_a1(
        "D1",
        "=MINIFS(OFFSET(A1,0,0,{1;2}),OFFSET(A1,0,0,{1;2}),\">0\")",
    );
    model.set_a1("E1", "=AVERAGEIFS(OFFSET(A1,{0;1},0),A1:A2,\">0\")");
    model.set_a1("F1", "=MINIFS(A1,OFFSET(A1,{0;1},0),\">0\")");
    model.set_a1(
        "G1",
        "=AVERAGEIFS(OFFSET(A:A,0,0,,SEQUENCE(1,1000)),A:A,\">0\")",
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"1");
    assert_eq!(model.get_text_a1("C2"), *"100");
    assert_eq!(model.get_text_a1("D1"), *"1");
    assert_eq!(model.get_text_a1("D2"), *"1");
    assert_eq!(model.get_text_a1("E1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("E2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F1"), *"1");
    assert_eq!(model.get_text_a1("F2"), *"1");
    assert_eq!(model.get_text_a1("G1"), *"#ERROR!");
}

// SPEC-0014 "Bounds": the elements' cells are charged before any element runs, so a SUBTOTAL
// over a thousand whole columns is the bound's error, not 10^9 reads (a DEVIATION, lookup.md).
#[test]
fn fn_offset_lifted_per_element_walk_past_the_budget_is_refused() {
    let mut model = new_empty_model();
    model.set_a1("A1048576", "1");
    model.set_a1("C1", "=SUBTOTAL(9,OFFSET(A:A,0,0,,SEQUENCE(1,1000)))");
    model.set_a1("D1", "=SUBTOTAL(9,OFFSET(A:A,0,0,,SEQUENCE(1,3)))");
    // A reference beside the grid is walked by every element, and charged so.
    model.set_a1("E5", "=SUBTOTAL(9,OFFSET(B1,SEQUENCE(1000),0),A:A)");
    // A criteria argument is read once, not once per element.
    model.set_a1("F5", "=COUNTIF(OFFSET(A1,{0;1},0),SUM(1,2))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("D1"), *"1");
    assert_eq!(model.get_text_a1("E5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("F5"), *"0");
}

// A per-element consumer reads a lifting argument once and puts back what it gave, so a nest of
// them costs one evaluation per level, not two (2^depth).
#[test]
fn fn_offset_per_element_consumers_nest_linearly() {
    let mut model = new_empty_model();
    model.set_a1("A1", "0");
    let mut formula = "0".to_string();
    for _ in 0..60 {
        formula = format!("N(OFFSET(A1,{formula},0))");
    }
    model.set_a1("C1", &format!("={formula}"));
    // IF, CHOOSE and LET are not read ahead: each is evaluated once, by the call.
    let mut formula = "1".to_string();
    for _ in 0..60 {
        formula = format!("N(IF(TRUE,{formula}))");
    }
    model.set_a1("D1", &format!("={formula}"));
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"0");
    assert_eq!(model.get_text_a1("D1"), *"1");
}
