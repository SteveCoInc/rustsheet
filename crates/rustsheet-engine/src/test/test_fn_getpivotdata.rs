#![allow(clippy::unwrap_used)]

//! GETPIVOTDATA against pivots built in code (SPEC-0015 "GETPIVOTDATA", T-2083). Excel:
//! probes/getpivotdata-real-pivot sheet P (docs/excel-semantics/web.md "GETPIVOTDATA against a
//! pivot table"); each test names the probe cells of its SPEC-0015 rule row.

use crate::pivot::{
    Aggregate, Axis, AxisField, CacheBuilder, CacheItem, CellArea, DataField, PivotCache,
    PivotField, PivotTable, RecordValue, MAX_RECORD_VALUES,
};
use crate::test::util::new_empty_model;
use crate::Model;

/// The probe's Data!A1:C7 records: Region, Product, Sales.
const RECORDS: [(&str, &str, f64); 6] = [
    ("East", "A", 10.0),
    ("West", "A", 20.0),
    ("East", "B", 5.0),
    ("West", "B", 7.0),
    ("East", "C", 3.0),
    ("East", "A", 4.0),
];

fn cache_of(records: &[(&str, &str, CacheItem)]) -> PivotCache {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in ["Region", "Product", "Sales"] {
        builder.add_field(name.to_string()).unwrap();
    }
    for (region, product, sales) in records {
        let text = |s: &str| RecordValue::Inline(CacheItem::Text(s.to_string()));
        builder
            .push_record(vec![
                text(region),
                text(product),
                RecordValue::Inline(sales.clone()),
            ])
            .unwrap();
    }
    builder.finish()
}

fn cache() -> PivotCache {
    let records: Vec<_> = RECORDS
        .iter()
        .map(|(r, p, s)| (*r, *p, CacheItem::Number(*s)))
        .collect();
    cache_of(&records)
}

/// A pivot of cache `cache` at the 0-based `location`, Region rows, optional Product columns.
fn pivot(
    cache: u32,
    location: CellArea,
    column: bool,
    caption: &str,
    function: Aggregate,
) -> PivotTable {
    let fields = [Some(Axis::Row), column.then_some(Axis::Column), None]
        .into_iter()
        .map(|axis| PivotField::new(axis, 3))
        .collect();
    PivotTable {
        fields,
        row_fields: vec![AxisField::Field(0)],
        column_fields: if column {
            vec![AxisField::Field(1)]
        } else {
            Vec::new()
        },
        data_fields: vec![DataField {
            caption: caption.to_string(),
            field: 2,
            function,
            ..DataField::default()
        }],
        ..PivotTable::new(caption.to_string(), 1, location, cache)
    }
}

/// The probe's two pivots on Sheet1: Sum of Sales at A3:E7 (Region rows, Product columns) and
/// Average of Sales at A20:B23 (Region rows). The rendered cells are left out but for the
/// labels the probe reads (A5 `East`); a second sheet stands for the source.
fn model() -> Model {
    let mut model = new_empty_model();
    model.new_sheet();
    model.workbook.pivot_caches = vec![cache()];
    model.workbook.pivot_tables = vec![
        pivot(
            0,
            CellArea {
                top: 2,
                left: 0,
                bottom: 6,
                right: 4,
            },
            true,
            "Sum of Sales",
            Aggregate::Sum,
        ),
        pivot(
            0,
            CellArea {
                top: 19,
                left: 0,
                bottom: 22,
                right: 1,
            },
            false,
            "Average of Sales",
            Aggregate::Average,
        ),
    ];
    model.set_a1("A3", "Sum of Sales");
    model.set_a1("A5", "East");
    model.set_a1("Z5", "=1/0");
    model
}

/// Each formula's text after evaluation, against [`model`].
fn check(cases: &[(&str, &str)]) {
    let mut model = model();
    for (i, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("H{}", 40 + i), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("H{}", 40 + i)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn the_data_field_is_the_caption_or_the_source_name_ignoring_case() {
    check(&[
        (r#"=GETPIVOTDATA("Sales",$A$3)"#, "49"),        // H1
        (r#"=GETPIVOTDATA("Sum of Sales",$A$3)"#, "49"), // H2
        (r#"=GETPIVOTDATA("sales",$A$3,"region","east")"#, "22"), // H10
        (r#"=GETPIVOTDATA("Average of Sales",$A$20)"#, "8.166666667"), // J2
        (r#"=GETPIVOTDATA(" Sales ",$A$3)"#, "#REF!"),   // H19
    ]);
}

#[test]
fn another_aggregates_caption_or_an_unknown_field_is_ref() {
    check(&[
        (r#"=GETPIVOTDATA("Sum of Sales",$A$20)"#, "#REF!"), // J4
        (r#"=GETPIVOTDATA("Profit",$A$3)"#, "#REF!"),        // H9
        (r#"=GETPIVOTDATA("Region",$A$3)"#, "#REF!"),
    ]);
}

#[test]
fn a_blank_cell_is_the_only_data_field() {
    check(&[
        ("=GETPIVOTDATA(C3,$A$3)", "49"), // J7: a blank cell inside the pivot
        // A blank cell outside it is `#REF!` (probes/t2085-getpivotdata-data-fields Q!A17).
        ("=GETPIVOTDATA(Z1,$A$3)", "#REF!"),
        // `""` and an omitted argument: SPEC-0015 unknown 8.
        (r#"=GETPIVOTDATA("",$A$3)"#, "#REF!"),
        ("=GETPIVOTDATA(,$A$3)", "#REF!"),
    ]);
}

#[test]
fn pairs_pick_the_grand_total_a_total_or_an_intersection() {
    check(&[
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region","East")"#, "22"), // H3
        (r#"=GETPIVOTDATA("Sales",$A$3,"Product","A")"#, "34"),   // H4
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Region","East","Product","A")"#,
            "14",
        ), // H5
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Product","A","Region","East")"#,
            "14",
        ), // H6
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Region","West","Product","B")"#,
            "7",
        ), // H14
        (r#"=GETPIVOTDATA("Sales",$A$20,"Region","East")"#, "5.5"), // J1
        (r#"=GETPIVOTDATA("Sales",$A$20)"#, "8.166666667"),       // J5
    ]);
}

#[test]
fn names_ignore_case_and_an_item_may_be_a_label_cell() {
    check(&[
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Product","a","Region","EAST")"#,
            "14",
        ), // J8
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region",A5)"#, "22"), // J6
    ]);
}

#[test]
fn an_odd_count_of_five_or_more_arguments_is_error_but_three_stay_ref() {
    check(&[
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Region","East","Product")"#,
            "#ERROR!",
        ),
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Region","East","Product","A","Sales")"#,
            "#ERROR!",
        ),
        (
            r#"=GETPIVOTDATA("Sales",$A$20,"Product","A","Region")"#,
            "#ERROR!",
        ),
        (
            r#"=GETPIVOTDATA("Sales",$H$30,"Product","A","Region")"#,
            "#ERROR!",
        ), // no pivot
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region")"#, "#REF!"), // H15
    ]);
}

#[test]
fn unknown_items_fields_repeats_and_odd_pairs_are_ref() {
    check(&[
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region","North")"#, "#REF!"), // H7
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region"," East")"#, "#REF!"), // H18
        (r#"=GETPIVOTDATA("Sales",$A$3,"Market","East")"#, "#REF!"),  // H8
        (r#"=GETPIVOTDATA("Sales",$A$20,"Product","A")"#, "#REF!"),   // J3
        (r#"=GETPIVOTDATA("Sales",$A$3,"Sales",5)"#, "#REF!"),
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Region","East","Region","West")"#,
            "#REF!",
        ), // H17
        (
            r#"=GETPIVOTDATA("Sales",$A$3,"Region","East","Region","East")"#,
            "#REF!",
        ), // SPEC-0015 unknown 5
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region")"#, "#REF!"), // H15
        // Numbers and booleans as items, and errors in a pair: SPEC-0015 unknown 3, unprobed.
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region",1)"#, "#REF!"),
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region",TRUE)"#, "#REF!"),
        (r#"=GETPIVOTDATA("Sales",$A$3,"Region",NA())"#, "#REF!"),
        (r#"=GETPIVOTDATA("Sales",$A$3,NA(),"East")"#, "#N/A"), // T-4823: a field error is the result
    ]);
}

#[test]
fn an_empty_sum_intersection_is_zero() {
    check(&[(
        r#"=GETPIVOTDATA("Sales",$A$3,"Region","West","Product","C")"#,
        "0",
    )]); // H13
}

#[test]
fn the_reference_is_any_cell_of_the_pivot_or_a_range_meeting_it() {
    check(&[
        (r#"=GETPIVOTDATA("Sales",$C$6)"#, "49"),      // H11
        (r#"=GETPIVOTDATA("Sales",$A$3:$E$7)"#, "49"), // H16
        (r#"=GETPIVOTDATA("Sales",$E$7)"#, "49"),
        (r#"=GETPIVOTDATA("Sales",$F$1:$G$30)"#, "#REF!"),
        (r#"=GETPIVOTDATA("Sales",$H$30)"#, "#REF!"), // H12
        (r#"=GETPIVOTDATA("Sales",Sheet2!$A$3)"#, "#REF!"), // H20
        (r#"=GETPIVOTDATA("Sales",$A$8)"#, "#REF!"),
        // A range over both pivots: SPEC-0015 unknown 9.
        (r#"=GETPIVOTDATA("Sales",$A$1:$B$30)"#, "#REF!"),
        (r#"=GETPIVOTDATA("Sales","A3")"#, "#REF!"),
        // Errors in the first two arguments propagate (T-1703).
        (r"=GETPIVOTDATA(NA(),$A$3)", "#N/A"),
        (r#"=GETPIVOTDATA("Sales",1/0)"#, "#DIV/0!"),
    ]);
}

#[test]
fn an_array_of_items_lifts_and_spills() {
    let mut model = model();
    model.set_a1(
        "L1",
        r#"=GETPIVOTDATA("Sales",$A$3,"Region",{"East";"West"})"#,
    );
    model.set_a1(
        "L5",
        r#"=GETPIVOTDATA("Sales",$A$3,"Product",{"A","B","C"})"#,
    );
    model.set_a1("L8", r#"=GETPIVOTDATA("Sales",$A$3,{"Region"},"East")"#);
    model.evaluate();
    let texts: Vec<String> = ["L1", "L2", "L5", "M5", "N5", "L8"]
        .iter()
        .map(|cell| model.get_text_a1(cell))
        .collect();
    // An array field name is one `#REF!` (SPEC-0015 unknown 7).
    assert_eq!(texts, ["22", "27", "34", "12", "3", "#REF!"]);
}

/// The aggregate over the East records (10, 5, 3, 4) of a Region-rows pivot.
fn east(function: Aggregate) -> String {
    let mut model = model();
    model.workbook.pivot_tables[1].data_fields[0].function = function;
    model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$20,"Region","East")"#);
    model.evaluate();
    model.get_text_a1("H1")
}

#[test]
fn every_aggregate_runs_over_the_picked_records() {
    // Count, Max, Min, Product, Var and StdDev are unprobed against a pivot (T-2085); these are
    // the worksheet functions' answers over the same numbers.
    for (function, expected) in [
        (Aggregate::Count, "4"),
        (Aggregate::CountNums, "4"),
        (Aggregate::Max, "10"),
        (Aggregate::Min, "3"),
        (Aggregate::Product, "600"),
        (Aggregate::Var, "9.666666667"),
        (Aggregate::VarP, "7.25"),
        (Aggregate::StdDev, "3.109126351"),
        (Aggregate::StdDevP, "2.692582404"),
    ] {
        assert_eq!(east(function), expected, "{function:?}");
    }
}

#[test]
fn every_aggregate_with_no_records_is_zero() {
    // probes/getpivotdata-empty-intersection and probes/t2085-getpivotdata-data-fields Q!B6:J6
    // (SPEC-0015 unknown 2).
    for (function, expected) in [
        (Aggregate::Average, "0"),
        (Aggregate::Count, "0"),
        (Aggregate::Max, "0"),
        (Aggregate::Min, "0"),
        (Aggregate::Var, "0"),
        (Aggregate::StdDev, "0"),
    ] {
        let mut model = model();
        model.workbook.pivot_tables[0].data_fields[0].function = function;
        model.set_a1(
            "H1",
            r#"=GETPIVOTDATA("Sales",$A$3,"Region","West","Product","C")"#,
        );
        model.evaluate();
        assert_eq!(model.get_text_a1("H1"), expected, "{function:?}");
    }
}

#[test]
fn a_record_without_a_number_in_the_data_field_is_ref() {
    // SPEC-0015 unknown 6: only the call whose records hold the text is `#REF!`.
    let mut model = model();
    model.workbook.pivot_caches = vec![cache_of(&[
        ("East", "A", CacheItem::Number(1.0)),
        ("West", "A", CacheItem::Text("x".to_string())),
        ("West", "B", CacheItem::Missing),
    ])];
    model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$3,"Region","East")"#);
    model.set_a1("H2", r#"=GETPIVOTDATA("Sales",$A$3,"Region","West")"#);
    model.set_a1("H3", r#"=GETPIVOTDATA("Sales",$A$3,"Product","B")"#);
    model.evaluate();
    let texts: Vec<String> = ["H1", "H2", "H3"]
        .iter()
        .map(|c| model.get_text_a1(c))
        .collect();
    assert_eq!(texts, ["1", "#REF!", "#REF!"]);
}

#[test]
fn a_non_finite_result_is_num() {
    let mut model = model();
    model.workbook.pivot_caches = vec![cache_of(&[
        ("East", "A", CacheItem::Number(1e308)),
        ("East", "A", CacheItem::Number(1e308)),
    ])];
    model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$3)"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("H1"), "#NUM!");
}

#[test]
fn unprobed_pivot_shapes_are_ref() {
    // SPEC-0015 unknowns 1 (several data fields) and 4 (page fields, hidden items), and a
    // malformed cache index.
    type Change = fn(&mut PivotTable);
    let changes: [Change; 4] = [
        |p| p.data_fields.push(p.data_fields[0].clone()),
        |p| p.fields[2].axis = Some(Axis::Page),
        |p| p.fields[0].hide(1),
        |p| p.cache = 7,
    ];
    for change in changes {
        let mut model = model();
        change(&mut model.workbook.pivot_tables[0]);
        model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$3)"#);
        model.evaluate();
        assert_eq!(model.get_text_a1("H1"), "#REF!");
    }
}

#[test]
fn two_fields_on_an_axis_answer_only_the_grand_total() {
    // calipers formula_stress_test Database!B16 (two row and two column fields) is the grand
    // total; a pair is SPEC-0015 unknown 10.
    let mut model = model();
    let table = &mut model.workbook.pivot_tables[0];
    table.fields[1].axis = Some(Axis::Row);
    table.row_fields.push(AxisField::Field(1));
    table.column_fields.clear();
    model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$3)"#);
    model.set_a1("H2", r#"=GETPIVOTDATA("Sales",$A$3,"Region","East")"#);
    model.set_a1(
        "H3",
        r#"=GETPIVOTDATA("Sales",$A$3,"Region","East","Product","A")"#,
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("H1"), "49");
    assert_eq!(model.get_text_a1("H2"), "#REF!");
    assert_eq!(model.get_text_a1("H3"), "#REF!");
}

#[test]
fn an_omitted_item_is_the_blank_item() {
    // calipers pv23_041 Liability Summary G5:G7: `GETPIVOTDATA(...,"Remediation Status",)` is
    // the `(blank)` row, and so is `"(blank)"` (probes/t4236-pivot-date-groups Calc B15, J11).
    // `""` and a blank cell are unprobed, `#REF!`.
    let mut model = model();
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in ["Region", "Product", "Sales"] {
        builder.add_field(name.to_string()).unwrap();
    }
    for (region, sales) in [(Some("East"), 1.0), (None, 2.0), (None, 4.0)] {
        let region = region.map_or(CacheItem::Missing, |r| CacheItem::Text(r.to_string()));
        builder
            .push_record(vec![
                RecordValue::Inline(region),
                RecordValue::Inline(CacheItem::Text("A".to_string())),
                RecordValue::Inline(CacheItem::Number(sales)),
            ])
            .unwrap();
    }
    model.workbook.pivot_caches = vec![builder.finish()];
    model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$3,"Region",)"#);
    model.set_a1(
        "H2",
        r#"=GETPIVOTDATA("Sales",$A$3,"Product","A","Region",)"#,
    );
    model.set_a1("H3", r#"=GETPIVOTDATA("Sales",$A$3,"Product",)"#);
    model.set_a1("H4", r#"=GETPIVOTDATA("Sales",$A$3,"Region","")"#);
    model.set_a1("H5", r#"=GETPIVOTDATA("Sales",$A$3,"Region",$Z$99)"#);
    model.set_a1("H6", r#"=GETPIVOTDATA("Sales",$A$3,"Region","(blank)")"#);
    model.evaluate();
    let texts: Vec<String> = ["H1", "H2", "H3", "H4", "H5", "H6"]
        .iter()
        .map(|c| model.get_text_a1(c))
        .collect();
    assert_eq!(texts, ["6", "6", "#REF!", "#REF!", "#REF!", "6"]);
}

#[test]
fn two_items_differing_only_in_case_are_ref() {
    let mut model = model();
    model.workbook.pivot_caches = vec![cache_of(&[
        ("East", "A", CacheItem::Number(1.0)),
        ("east", "A", CacheItem::Number(2.0)),
    ])];
    model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$3,"Region","East")"#);
    model.set_a1("H2", r#"=GETPIVOTDATA("Sales",$A$3)"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("H1"), "#REF!");
    assert_eq!(model.get_text_a1("H2"), "3");
}

#[test]
fn a_change_to_the_pivots_reaches_every_getpivotdata_cell() {
    let mut model = model();
    model.set_a1("H1", r#"=GETPIVOTDATA("Sales",$A$3,"Region","East")"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("H1"), "22");
    model.workbook.pivot_caches = vec![cache_of(&[("East", "A", CacheItem::Number(100.0))])];
    model.evaluate();
    assert_eq!(model.get_text_a1("H1"), "100");
    model.workbook.pivot_tables.clear();
    model.evaluate();
    assert_eq!(model.get_text_a1("H1"), "#REF!");
}

#[test]
fn many_calls_build_one_index_per_pivot_and_evaluation() {
    let mut model = model();
    for row in 1..=200 {
        let region = if row % 2 == 0 { "East" } else { "West" };
        model.set_a1(
            &format!("J{row}"),
            &format!(r#"=GETPIVOTDATA("Sales",$A$3,"Region","{region}")"#),
        );
    }
    model.set_a1("K1", r#"=GETPIVOTDATA("Sales",$A$20)"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("J2"), "22");
    assert_eq!(model.get_text_a1("J1"), "27");
    let builds = model.evaluation.pivot_indexes.builds();
    assert_eq!(builds, 2);
    model.evaluate();
    assert_eq!(model.evaluation.pivot_indexes.builds(), builds + 2);
}

#[test]
fn getpivotdata_follows_the_pivot_through_structural_edits() {
    // SPEC-0015 "Data model" (T-2086): the pivot moves with its cells, so a formula whose
    // reference moved with them keeps answering; an edit through the pivot is refused.
    let mut model = model();
    model.set_a1(
        "Sheet2!A1",
        r#"=GETPIVOTDATA("Sales",Sheet1!$A$3,"Region","East")"#,
    );
    model.set_a1("Sheet2!A2", r#"=GETPIVOTDATA("Sales",Sheet1!$E$7)"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("Sheet2!A1"), "22");
    model.insert_rows(0, 1, 2).unwrap();
    model.delete_columns(0, 6, 2).unwrap();
    model.rename_sheet_by_index(0, "Moved").unwrap();
    assert!(model.insert_rows(0, 7, 1).is_err());
    model.evaluate();
    assert_eq!(
        model.get_formula_a1("Sheet2!A1"),
        r#"=GETPIVOTDATA("Sales",Moved!$A$5,"Region","East")"#
    );
    assert_eq!(model.get_text_a1("Sheet2!A1"), "22");
    assert_eq!(model.get_text_a1("Sheet2!A2"), "49");
    // Deleting every row of the second pivot (A22:B25 now) drops it with its cells.
    model.set_a1("Sheet2!A3", r#"=GETPIVOTDATA("Sales",Moved!$A$22)"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("Sheet2!A3"), "8.166666667");
    model.delete_rows(0, 21, 6).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("Sheet2!A3"), "#REF!");
    assert_eq!(model.get_text_a1("Sheet2!A1"), "22");
}

#[test]
fn getpivotdata_follows_the_pivot_through_moves() {
    // T-2401: a move shifts a pivot like the cells it covers; a move that splits one is refused.
    let mut model = model();
    model.set_a1(
        "Sheet2!A1",
        r#"=GETPIVOTDATA("Sales",Sheet1!$A$3,"Region","East")"#,
    );
    model.evaluate();
    // Row 1 jumps over rows 2..=11, so the pivots (A3:E7, A20:B23) move up by one or stay.
    model.move_rows_action(0, 1, 1, 10).unwrap();
    // Row 3 and column B lie inside the first pivot (now rows 2-6): moving them splits it.
    assert!(model.move_rows_action(0, 3, 1, 1).is_err());
    assert!(model.move_columns_action(0, 2, 1, 1).is_err());
    let areas = |model: &Model| -> Vec<(u32, u32, u32, u32)> {
        let tables = &model.workbook.pivot_tables;
        tables
            .iter()
            .map(|p| {
                (
                    p.location.top,
                    p.location.left,
                    p.location.bottom,
                    p.location.right,
                )
            })
            .collect()
    };
    assert_eq!(areas(&model), vec![(1, 0, 5, 4), (19, 0, 22, 1)]);
    // The pivot's whole span moves down by two.
    model.move_rows_action(0, 2, 5, 2).unwrap();
    assert_eq!(areas(&model), vec![(3, 0, 7, 4), (19, 0, 22, 1)]);
    // Column F jumps back over A:E, so the pivots shift right by one.
    model.move_columns_action(0, 6, 1, -5).unwrap();
    assert_eq!(areas(&model), vec![(3, 1, 7, 5), (19, 1, 22, 2)]);
    model.evaluate();
    assert_eq!(
        model.get_formula_a1("Sheet2!A1"),
        r#"=GETPIVOTDATA("Sales",Sheet1!$B$4,"Region","East")"#
    );
    assert_eq!(model.get_text_a1("Sheet2!A1"), "22");
}

/// T-4823; Excel: web.md "GETPIVOTDATA error arguments (T-4006)", gen-m6 cells.
#[test]
fn error_arguments_follow_the_t4006_order() {
    check(&[
        (
            r#"=GETPIVOTDATA("Sum of Sales",$A$3,#NAME?,"East")"#,
            "#NAME?",
        ), // field, typed
        (r#"=GETPIVOTDATA("Sum of Sales",$A$3,Z5,"East")"#, "#DIV/0!"),
        (
            r#"=GETPIVOTDATA("Sum of Sales",$A$3,"Region",#DIV/0!)"#,
            "#REF!",
        ), // item
        (r"=GETPIVOTDATA(#REF!,$A$3,#NAME?,2)", "#REF!"),
        (r#"=GETPIVOTDATA(#NULL!,$A$3,"Region",#VALUE!)"#, "#NULL!"), // typed df beats item
        (r#"=GETPIVOTDATA(Z5,$A$3,"Region","East")"#, "#REF!"),       // referenced df
    ]);
}

/// Pivot of the probe's records with two data fields over Sales, Sum of Sales and Count of
/// Sales, Region on the rows and `Values` on the columns, at A3 (probes/t2085-getpivotdata-data-fields
/// pivot M, DF A1, A3, A8 to A11). The case shape is DF's; the Values-name and caption rules it
/// checks beyond DF's cells are UNVERIFIED (T-4911).
fn two_data_field_model() -> Model {
    let mut model = new_empty_model();
    model.new_sheet();
    model.workbook.pivot_caches = vec![cache()];
    let mut table = pivot(
        0,
        CellArea {
            top: 2,
            left: 0,
            bottom: 6,
            right: 2,
        },
        false,
        "Sum of Sales",
        Aggregate::Sum,
    );
    table.column_fields = vec![AxisField::Values];
    table.data_fields.push(DataField {
        caption: "Count of Sales".to_string(),
        field: 2,
        function: Aggregate::Count,
        ..DataField::default()
    });
    model.workbook.pivot_tables = vec![table];
    model
}

#[test]
fn several_data_fields_pick_by_source_name_caption_and_the_values_pair() {
    let cases = [
        // A source name two data fields share picks the first (DF A1).
        (r#"=GETPIVOTDATA("Sales",A3)"#, "49"),
        // A caption picks its data field, ignoring case (DF A3, A12).
        (r#"=GETPIVOTDATA("Count of Sales",A3)"#, "6"),
        (r#"=GETPIVOTDATA("COUNT OF SALES",A3,"Region","East")"#, "4"),
        // The Values pair names the data field by its caption (DF A8); another data field's
        // caption is #REF! (DF A9, A11).
        (r#"=GETPIVOTDATA("Sales",A3,"Values","Sum of Sales")"#, "49"),
        (r#"=GETPIVOTDATA("Sales",A3,"values","SUM OF SALES")"#, "49"),
        (
            r#"=GETPIVOTDATA("Count of Sales",A3,"Values","Count of Sales")"#,
            "6",
        ),
        (
            r#"=GETPIVOTDATA("Sales",A3,"Values","Count of Sales")"#,
            "#REF!",
        ),
        (
            r#"=GETPIVOTDATA("Sales",A3,"Region","East","Values","Count of Sales")"#,
            "#REF!",
        ),
        // "Values" is no data field (DF A10).
        (r#"=GETPIVOTDATA("Values",A3)"#, "#REF!"),
    ];
    let mut model = two_data_field_model();
    for (i, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("H{}", 40 + i), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("H{}", 40 + i)),
            *expected,
            "{formula}"
        );
    }
}
