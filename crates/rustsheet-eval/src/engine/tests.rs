//! Unit tests of the RustSheet adapter (`super`): spec cases built and evaluated in memory.

use rustsheet_engine::Model;

use super::*;

#[test]
fn lone_units_render_as_xhhhh_escapes_and_pairs_stay_text() {
    // "a😀" then the lone low surrogate of a cut pair, then "b" (text-surrogate-positions A6).
    let units = [0x61, 0xD83D, 0xDE00, 0xDE00, 0x62];
    assert_eq!(render_lone_units(&units), "a😀_xDE00_b");
    assert_eq!(render_lone_units(&[0xD83D]), "_xD83D_");
}

/// Workbook with `Sheet1!A1 = TODAY()` and an optional `METADATA` NOW row.
fn workbook(now_serial: Option<f64>) -> Vec<u8> {
    let mut model = Model::new_empty("case", "en", "UTC", "en").expect("model");
    model
        .update_cell_with_formula(0, 1, 1, "=TODAY()".into())
        .expect("formula");
    if let Some(serial) = now_serial {
        model.add_sheet("METADATA").expect("sheet");
        model.update_cell_with_text(1, 1, 1, "NOW").expect("label");
        model
            .update_cell_with_number(1, 1, 2, serial)
            .expect("serial");
    }
    model.evaluate();
    let cursor = std::io::Cursor::new(Vec::new());
    rustsheet_xlsx::export::save_xlsx_to_writer(&model, cursor)
        .expect("save")
        .into_inner()
}

fn today(bytes: &[u8]) -> Value {
    let evaluated = RustSheetEngine
        .evaluate_xlsx(bytes, None)
        .expect("evaluate");
    evaluated
        .value("Sheet1", Addr { row: 1, col: 1 })
        .expect("sheet exists")
}

#[test]
fn metadata_now_row_pins_the_clock() {
    assert_eq!(today(&workbook(Some(45000.75))), Value::Number(45000.0));
}

#[test]
fn without_metadata_the_clock_is_a_fixed_default() {
    // 8 November 2022 (the engine's mock default).
    assert_eq!(today(&workbook(None)), Value::Number(44873.0));
}

fn spec(json: &str) -> CaseSpec {
    serde_json::from_str(json).expect("spec")
}

#[test]
fn spec_sheet_ops_apply_in_order_after_the_build() {
    // probes/sheet-ops (T-1608): references follow the renamed sheet, the moved sheet keeps
    // them, and an added sheet is empty.
    let spec = spec(
        r#"{"format":"rustsheet-case/v1",
            "sheets":[{"name":"Calc","cells":{"A1":"=Data!A1","A2":"=FORMULATEXT(A1)",
                                             "A3":"=SHEET(Inputs!A1)","A4":"=COUNTA(New!A1:B2)"}},
                      {"name":"Data","cells":{"A1":5}}],
            "sheet_ops":[{"op":"rename","sheet":"Data","to":"Inputs"},
                         {"op":"move","sheet":"Inputs","to":0},
                         {"op":"add","sheet":"New"}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    let calc = |row| evaluated.value("Calc", Addr { row, col: 1 });
    assert_eq!(calc(1), Some(Value::Number(5.0)));
    assert_eq!(calc(2), Some(Value::Text("=Inputs!A1".into())));
    assert_eq!(calc(3), Some(Value::Number(1.0)));
    assert_eq!(
        evaluated.value("New", Addr { row: 1, col: 1 }),
        Some(Value::Empty)
    );
}

#[test]
fn add_table_adds_a_named_table_after_a_convert() {
    // T-3629: Sales is converted, then added again on K8:L11 under its old name; adding it a
    // second time is refused because the name is taken, which shows the first add made it.
    let json = |adds: usize| {
        let add = r#"{"op":"add_table","sheet":"S","table":"Sales","ref":"K8:L11"}"#;
        format!(
            r#"{{"format":"rustsheet-case/v1",
                "sheets":[{{"name":"S","cells":{{"B3":"Region","C3":"Qty","B4":"E","C4":1,
                                                "K8":"Region","L8":"Qty","K9":"a","L9":10}},
                           "tables":[{{"name":"Sales","ref":"B3:C4"}}]}}],
                "sheet_ops":[{{"op":"convert_table","sheet":"S","table":"Sales"}},{}]}}"#,
            vec![add; adds].join(",")
        )
    };
    assert!(RustSheetEngine.evaluate_spec(&spec(&json(1)), None).is_ok());
    assert!(RustSheetEngine
        .evaluate_spec(&spec(&json(2)), None)
        .is_err());
}

#[test]
fn structural_sheet_ops_shift_cells_and_rewrite_references() {
    // T-2323: A3 refers to B2; a row delete above, a column insert left of it and a delete
    // of the sheet and the table named by the later ops all replay in order.
    let spec = spec(
        r#"{"format":"rustsheet-case/v1",
            "sheets":[{"name":"Calc","cells":{"A1":1,"B2":5,"A3":"=B2*2","B3":"=SUM(A1:A2)",
                                 "D1":"x","E1":"y","D2":1,"E2":2},
                       "tables":[{"name":"Sales","ref":"D1:E2"}]},
                      {"name":"Gone","cells":{"A1":1}}],
            "sheet_ops":[{"op":"delete_row","sheet":"Calc","row":1},
                         {"op":"insert_column","sheet":"Calc","column":"A"},
                         {"op":"delete_column","sheet":"Calc","column":"A"},
                         {"op":"delete_sheet","sheet":"Gone"},
                         {"op":"delete_table","sheet":"Calc","table":"Sales"}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    // The deleted row 1 took A1; B2 moved to B1 and A3 to A2, still pointing at it.
    assert_eq!(
        evaluated.value("Calc", Addr { row: 2, col: 1 }),
        Some(Value::Number(10.0))
    );
    assert_eq!(
        evaluated.value("Calc", Addr { row: 1, col: 2 }),
        Some(Value::Number(5.0))
    );
    // T-2341: the table's cells are cleared, so D1 is empty (table-delete-table F!A6).
    assert_eq!(
        evaluated.value("Calc", Addr { row: 1, col: 4 }),
        Some(Value::Empty)
    );
}

#[test]
fn bad_structural_ops_are_adapter_errors() {
    for op in [
        r#"{"op":"convert_table","sheet":"Calc","table":"Nope"}"#,
        r#"{"op":"delete_table","sheet":"Calc","table":"Nope"}"#,
        r#"{"op":"delete_row","sheet":"Calc","row":0}"#,
        r#"{"op":"delete_row","sheet":"Calc","row":1,"count":0}"#,
        r#"{"op":"delete_row","sheet":"Nope","row":1}"#,
        r#"{"op":"insert_column","sheet":"Calc","column":"A1"}"#,
        r#"{"op":"delete_column","sheet":"Calc","column":"XFE"}"#,
        r#"{"op":"delete_sheet","sheet":"Calc"}"#,
    ] {
        let json = format!(
            r#"{{"format":"rustsheet-case/v1",
                "sheets":[{{"name":"Calc","cells":{{"A1":"h","A2":1}},
                            "tables":[{{"name":"Sales","ref":"A1:A2"}}]}}],
                "sheet_ops":[{op}]}}"#
        );
        assert!(
            RustSheetEngine.evaluate_spec(&spec(&json), None).is_err(),
            "{op}"
        );
    }
}

#[test]
fn unknown_or_malformed_sheet_ops_are_refused() {
    for ops in [
        r#"[{"op":"delete","sheet":"Data"}]"#,
        r#"[{"op":"move","sheet":"Data","to":-1}]"#,
        r#"[{"op":"add","sheet":"New","to":1}]"#,
    ] {
        let json = format!(r#"{{"format":"rustsheet-case/v1","sheets":[],"sheet_ops":{ops}}}"#);
        assert!(serde_json::from_str::<CaseSpec>(&json).is_err(), "{ops}");
    }
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"Calc","cells":{}}],
            "sheet_ops":[{"op":"rename","sheet":"Nope","to":"X"}]}"#,
    );
    assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
}

#[test]
fn spec_names_are_defined_before_cells() {
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","names":{"src":"=Data!$A$1"},
            "sheets":[{"name":"Calc","cells":{"A1":"=src*2"}},
                      {"name":"Data","cells":{"A1":5}}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    assert_eq!(
        evaluated.value("Calc", Addr { row: 1, col: 1 }),
        Some(Value::Number(10.0))
    );
}

#[test]
fn sheet_qualified_spec_names_are_sheet_scoped() {
    // probes/multiarea-printarea's spelling (T-1193): the name is local to Sheet1.
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","names":{"Sheet1!Print_Area":"=Sheet1!$A$1,Sheet1!$B$2"},
            "sheets":[{"name":"Sheet1","cells":{"A1":1,"B2":5,"H1":"=SUM(Print_Area)"}},
                      {"name":"Other","cells":{"A1":"=SUM(Print_Area)"}}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    assert_eq!(
        evaluated.value("Sheet1", Addr { row: 1, col: 8 }),
        Some(Value::Number(6.0))
    );
    assert_eq!(
        evaluated.value("Other", Addr { row: 1, col: 1 }),
        Some(Value::Error("#NAME?".into()))
    );
}

#[test]
fn a_spec_name_on_an_unknown_sheet_fails_the_case() {
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","names":{"Nope!x":"=1"},
            "sheets":[{"name":"Sheet1","cells":{}}]}"#,
    );
    assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
}

#[test]
fn stored_error_cells_load_as_constants_not_text() {
    let spec = spec(
        r##"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":{"t":"e","v":"#N/A","stored":true},
            "A2":{"t":"e","v":"#GETTING_DATA","stored":true},
            "A3":{"t":"e","v":"#BUSY!","stored":true},
            "A4":{"t":"e","v":"#N/A"},
            "A5":{"t":"e","v":"#UNKNOWN!","stored":true},
            "B2":"=ERROR.TYPE(A2)",
            "B3":"=ERROR.TYPE(A3)"}}]}"##,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    let at = |row| evaluated.value("S", Addr { row, col: 1 });
    assert_eq!(at(1), Some(Value::Error("#N/A".into())));
    assert_eq!(at(2), Some(Value::Error("#N/A".into())));
    assert_eq!(
        evaluated.value("S", Addr { row: 2, col: 2 }),
        Some(Value::Number(8.0))
    );
    // Its own kind with ERROR.TYPE 0 (SPEC-0003 "Stored `#BUSY!`", T-1222).
    assert_eq!(at(3), Some(Value::Error("#BUSY!".into())));
    assert_eq!(
        evaluated.value("S", Addr { row: 3, col: 2 }),
        Some(Value::Number(0.0))
    );
    assert_eq!(at(4), Some(Value::Error("#N/A".into())));
    // Unknown to the stored-name table: `#ERROR!` (xlsx import's fallback), not the text.
    assert_eq!(at(5), Some(Value::Error("#ERROR!".into())));
}

#[test]
fn a_bad_spec_name_fails_the_case() {
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","names":{"2real":"=1"},
            "sheets":[{"name":"Sheet1","cells":{}}]}"#,
    );
    assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
}

#[test]
fn iterative_cases_run_the_oracle_calculations() {
    let parse = |json: &str| serde_json::from_str::<CaseSpec>(json).expect("spec");
    let plain = parse(r#"{"format":"rustsheet-case/v1","sheets":[]}"#);
    assert_eq!(calculations(&plain), 1);
    let off =
        parse(r#"{"format":"rustsheet-case/v1","sheets":[],"calculation":{"iterate":false}}"#);
    assert_eq!(calculations(&off), 1);
    let on = parse(r#"{"format":"rustsheet-case/v1","sheets":[],"calculation":{"iterate":true}}"#);
    assert_eq!(calculations(&on), ORACLE_CALCULATIONS);
}

#[test]
fn spec_date_system_and_number_formats_build() {
    // No value reads either yet (SPEC-0005 step 3); the facade test covers the format.
    let ok = spec(
        r#"{"format":"rustsheet-case/v1","date_system":1904,"sheets":[{"name":"S",
            "cells":{"A1":-1.5,"B1":"=A1*2"},"number_formats":{"A1":"[h]:mm","C3":"0.0"}}]}"#,
    );
    let evaluated = RustSheetEngine.evaluate_spec(&ok, None).expect("evaluates");
    assert_eq!(
        evaluated.value("S", Addr { row: 1, col: 2 }),
        Some(Value::Number(-3.0))
    );
    let bad_system = spec(
        r#"{"format":"rustsheet-case/v1","date_system":1901,"sheets":[{"name":"S","cells":{}}]}"#,
    );
    assert!(RustSheetEngine.evaluate_spec(&bad_system, None).is_err());
    let bad_cell = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{},
            "number_formats":{"nope":"0"}}]}"#,
    );
    assert!(RustSheetEngine.evaluate_spec(&bad_cell, None).is_err());
}

#[test]
fn spec_pivot_rendered_cells_reach_the_engine() {
    let pivot = |rendered: &str| {
        spec(&format!(
            r#"{{"format":"rustsheet-case/v1","sheets":[{{"name":"S","cells":{{
                "A1":"Key","B1":"Val","A2":"a","B2":1,"A3":"b","B3":2,"H1":"=D2"}},
                "pivot_tables":[{{"name":"Pv","source":"A1:B3","location":"D1","row":"Key",
                "value":"Val","rendered":{rendered}}}]}}]}}"#
        ))
    };
    let ok = pivot(r#"{"D1":"Row Labels","E1":"Sum of Val","D2":"a","E2":1}"#);
    let evaluated = RustSheetEngine.evaluate_spec(&ok, None).expect("evaluates");
    // The rendered cell is a constant a formula reads.
    assert_eq!(
        evaluated.value("S", Addr { row: 1, col: 8 }),
        Some(Value::Text("a".into()))
    );
    // Without rendered cells the pivot has no location.
    let err = RustSheetEngine.evaluate_spec(&pivot("{}"), None).err();
    assert!(err.is_some_and(|e| e.contains("bake-pivots")));
}

#[test]
fn spec_hidden_rows_and_tables_reach_the_engine() {
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":"Key","B1":"Val","A2":"a","B2":1,"A3":"b","B3":2,"A4":"c","B4":4,
            "D1":"=SUBTOTAL(109,B2:B4)","D2":"=SUBTOTAL(9,B2:B4)","D3":"=SUM(Data[Val])"},
            "hidden_rows":[3],"tables":[{"name":"Data","ref":"A1:B4","filter":true}]}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    let at = |row| evaluated.value("S", Addr { row, col: 4 });
    // Row 3 is filtered out of the table: both SUBTOTAL forms skip it.
    assert_eq!(at(1), Some(Value::Number(5.0)));
    assert_eq!(at(2), Some(Value::Number(5.0)));
    assert_eq!(at(3), Some(Value::Number(7.0)));
}

#[test]
fn spec_typed_formulas_go_in_after_the_tables() {
    // A typed `=...` reads a structured reference, so it goes in after the tables; typed text
    // that is not a formula stays a value (`(4)` is -4). T-2955, T-2801.
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":"Col","A2":3,"A3":{"entered":"(4)"},
            "C1":{"entered":"=SUM(T[Col])"},"C2":{"entered":"=A3*2"}},
            "tables":[{"name":"T","ref":"A1:A3"}]}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    assert_eq!(
        evaluated.value("S", Addr { row: 1, col: 3 }),
        Some(Value::Number(-1.0))
    );
    assert_eq!(
        evaluated.value("S", Addr { row: 2, col: 3 }),
        Some(Value::Number(-8.0))
    );
}

#[test]
fn spec_r1c1_entries_read_relative_to_their_cell_and_unparsable_ones_stay_blank() {
    // `R1C2` is $B$1 and `R[-2]C[1]` the cell two up and one right (T-3113); R1C1 text that
    // does not parse is left blank, as Excel's refused PATCH leaves it (decision 0179).
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "B1":5,"A2":{"entered_r1c1":"=R1C2+1"},"A3":{"entered_r1c1":"=R[-2]C[1]*2"},
            "A5":{"entered_r1c1":"=1+"},"C5":"=ISBLANK(A5)"}}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    let at = |row, col| evaluated.value("S", Addr { row, col });
    assert_eq!(at(2, 1), Some(Value::Number(6.0)));
    assert_eq!(at(3, 1), Some(Value::Number(10.0)));
    assert_eq!(at(5, 3), Some(Value::Bool(true)));
}

#[test]
fn spec_entries_are_typed_one_at_a_time_after_the_load() {
    // Excel recalculates after the load and after each typed entry, and a reader of a cycle
    // keeps its value from before: `ISBLANK` of a typed self-reference stays TRUE, the cycle
    // H1:H2 typed one cell at a time keeps H1's 1 (probes/t3898-typed-circular-readers, T-3898).
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":"=ISBLANK(B2)","B2":{"entered":"=B2+1"},"A3":{"entered":"=ISBLANK(B2)"},
            "H1":{"entered":"=H2+1"},"H2":{"entered":"=H1+1"},"A4":"=ISBLANK(H1)"}}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    let at = |row, col| evaluated.value("S", Addr { row, col });
    assert_eq!(at(1, 1), Some(Value::Bool(true)));
    assert_eq!(at(2, 2), Some(Value::Number(0.0)));
    assert_eq!(at(3, 1), Some(Value::Number(0.0)));
    assert_eq!(at(1, 8), Some(Value::Number(1.0)));
    assert_eq!(at(2, 8), Some(Value::Number(0.0)));
    assert_eq!(at(4, 1), Some(Value::Bool(false)));
}

#[test]
fn spec_totals_row_is_part_of_the_table() {
    // The facade takes the totals row (T-1671): `#All` covers it and `#Totals` reads it.
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":"Key","B1":"Val","A2":"a","B2":1,"A3":"b","B3":2,"A4":"Total",
            "B4":"=SUBTOTAL(109,Data[Val])","D1":"=ROWS(Data[#All])",
            "D2":"=Data[[#Totals],[Val]]"},
            "tables":[{"name":"Data","ref":"A1:B4","totals_row":true}]}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    // B4 sums the two data rows (3); `#All` has four rows; `#Totals` reads B4.
    for (row, col, want) in [(4, 2, 3.0), (1, 4, 4.0), (2, 4, 3.0)] {
        assert_eq!(
            evaluated.value("S", Addr { row, col }),
            Some(Value::Number(want))
        );
    }
}

#[test]
fn spec_legacy_formulas_are_implicitly_intersected() {
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":1,"A2":2,"H7":"=A1:A2","H8":"=FORMULATEXT(H7)","H9":"=A1:A2"},
            "legacy_formulas":["H7"]}]}"#,
    );
    let evaluated = RustSheetEngine
        .evaluate_spec(&spec, None)
        .expect("evaluates");
    let at = |row| evaluated.value("S", Addr { row, col: 8 });
    assert_eq!(at(7), Some(Value::Error("#VALUE!".into())));
    assert_eq!(at(8), Some(Value::Text("=@A1:A2".into())));
    // H9 is not listed: a dynamic-array formula that spills.
    assert_eq!(at(9), Some(Value::Number(1.0)));
}

#[test]
fn a_zero_hidden_row_fails_the_case() {
    let spec = spec(
        r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{},"hidden_rows":[0]}]}"#,
    );
    assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
}
