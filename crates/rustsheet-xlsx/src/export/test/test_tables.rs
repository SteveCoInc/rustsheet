//! Tables and structured references through the export (SPEC-0016 step 3, T-2196).

use std::fs;

use rustsheet_engine::{
    types::{CustomFilter, DateGroupItem, Dxf, FilterColumn, FilterCriterion},
    Model,
};

use super::test_export::{new_empty_model, part_text};
use crate::{export::save_to_xlsx, import::load_from_xlsx};

/// Table `Sales` over A1:C4 of Sheet1, with a totals row (a label and a SUBTOTAL), a column whose
/// name needs an escape (`C#`) and formulas outside it with each kind of structured reference.
fn model_with_table() -> Model {
    let mut model = new_empty_model();
    let cells = [
        (1, 1, "Item"),
        (1, 2, "Qty"),
        (1, 3, "C#"),
        (2, 1, "a"),
        (2, 2, "2"),
        (2, 3, "10"),
        (3, 1, "b"),
        (3, 2, "3"),
        (3, 3, "20"),
        (4, 1, "Total"),
    ];
    for (row, column, input) in cells {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    model
        .add_table_with_totals(0, "Sales", (1, 1, 4, 3), false, true)
        .unwrap();
    let formulas = [
        (4, 2, "=SUBTOTAL(109,Sales[Qty])"),
        (2, 5, "=Sales[@Qty]*2"),
        (5, 5, "=ROWS(Sales)"),
        (6, 5, "=SUM(Sales[[#Totals],[Qty]])"),
        (7, 5, "=SUM(Sales[C'#])"),
        (8, 5, "=SUM(Sales[#All])"),
    ];
    for (row, column, input) in formulas {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    model.evaluate();
    model
}

#[test]
fn table_parts_and_structured_references_round_trip() {
    let model = model_with_table();
    let temp_file_name = "temp_file_test_table_parts.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let sheet_xml = part_text(temp_file_name, "xl/worksheets/sheet1.xml");
    let table_xml = part_text(temp_file_name, "xl/tables/table1.xml");
    let rels = part_text(temp_file_name, "xl/worksheets/_rels/sheet1.xml.rels");
    let content_types = part_text(temp_file_name, "[Content_Types].xml");
    let mut reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    // The file forms of xlsx-storage.md "Structured references in the file": a bare name as
    // `Sales[]`, `@` as `[#This Row]`, the escape before the special character.
    for formula in [
        "<f>Sales[[#This Row],[Qty]]*2</f>",
        "<f>ROWS(Sales[])</f>",
        "<f>SUM(Sales[[#Totals],[Qty]])</f>",
        "<f>SUM(Sales[C&apos;#])</f>",
        "<f>SUM(Sales[#All])</f>",
        "<f>SUBTOTAL(109,Sales[Qty])</f>",
    ] {
        assert!(sheet_xml.contains(formula), "{formula} in {sheet_xml}");
    }
    assert!(
        sheet_xml.contains(r#"<tableParts count="1"><tablePart r:id="rId1"/></tableParts>"#),
        "{sheet_xml}"
    );
    assert!(rels.contains(r#"Target="../tables/table1.xml""#), "{rels}");
    assert!(
        content_types.contains(r#"PartName="/xl/tables/table1.xml""#),
        "{content_types}"
    );
    // The totals row needs its metadata, or Excel refuses the file.
    assert!(
        table_xml.contains(r#"<tableColumn id="1" name="Item" totalsRowLabel="Total">"#),
        "{table_xml}"
    );
    assert!(
        table_xml.contains(
            r#"totalsRowFunction="custom"><totalsRowFormula>SUBTOTAL(109,Sales[Qty])</totalsRowFormula>"#
        ),
        "{table_xml}"
    );
    assert!(
        table_xml.contains(r#"ref="A1:C4" totalsRowCount="1""#),
        "{table_xml}"
    );
    assert!(
        table_xml.contains(r#"<autoFilter ref="A1:C3"></autoFilter>"#),
        "{table_xml}"
    );

    assert_eq!(reloaded.workbook.tables, model.workbook.tables);
    reloaded.evaluate();
    for (row, column) in [(4, 2), (2, 5), (5, 5), (6, 5), (7, 5), (8, 5)] {
        assert_eq!(
            reloaded.get_cell_formula(0, row, column).unwrap(),
            model.get_cell_formula(0, row, column).unwrap(),
            "formula of row {row} column {column}"
        );
        assert_eq!(
            reloaded.get_formatted_cell_value(0, row, column).unwrap(),
            model.get_formatted_cell_value(0, row, column).unwrap(),
            "value of row {row} column {column}"
        );
    }
    assert_eq!(reloaded.get_formatted_cell_value(0, 2, 5).unwrap(), "4");
    assert_eq!(reloaded.get_formatted_cell_value(0, 5, 5).unwrap(), "2");
    assert_eq!(reloaded.get_formatted_cell_value(0, 7, 5).unwrap(), "30");
}

/// A totals cell holding a number, boolean or error constant is written as
/// `totalsRowFunction="custom"` with no `<totalsRowFormula>`, the form Excel loads with the cell
/// kept a constant (xlsx-storage.md "Table parts in the export", T-4005); without it Excel refuses
/// the file.
#[test]
fn totals_row_constants_get_custom_metadata() {
    let mut model = new_empty_model();
    let cells = [
        (1, 1, "Item"),
        (1, 2, "N"),
        (1, 3, "B"),
        (1, 4, "E"),
        (2, 1, "a"),
        (2, 2, "1"),
        (2, 3, "2"),
        (2, 4, "3"),
        (3, 1, "Total"),
        (3, 2, "42"),
        (3, 3, "TRUE"),
        (3, 4, "#N/A"),
    ];
    for (row, column, input) in cells {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    model
        .add_table_with_totals(0, "Sales", (1, 1, 3, 4), false, true)
        .unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_totals_constants.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let table_xml = part_text(temp_file_name, "xl/tables/table1.xml");
    let mut reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert!(
        table_xml.contains(r#"<tableColumn id="1" name="Item" totalsRowLabel="Total">"#),
        "{table_xml}"
    );
    for name in ["N", "B", "E"] {
        let column = format!(r#"name="{name}" totalsRowFunction="custom"></tableColumn>"#);
        assert!(table_xml.contains(&column), "{column} in {table_xml}");
    }
    assert!(!table_xml.contains("totalsRowFormula"), "{table_xml}");

    assert_eq!(reloaded.workbook.tables, model.workbook.tables);
    reloaded.evaluate();
    for (column, value) in [(2, "42"), (3, "TRUE"), (4, "#N/A")] {
        assert_eq!(reloaded.get_cell_formula(0, 3, column).unwrap(), None);
        assert_eq!(
            reloaded.get_formatted_cell_value(0, 3, column).unwrap(),
            value
        );
    }
}

/// An API-filtered table with no kept criterion is written with a values filter on its first
/// column keeping its visible data rows' text, which Excel reads as filtered (T-4003), and so
/// does our import: the round trip keeps SUBTOTAL 9 equal to 109 (T-4363).
#[test]
fn a_filtered_table_without_a_criterion_exports_its_visible_values() {
    let mut model = new_empty_model();
    for (row, input) in [(1, "Qty"), (2, "1"), (3, "2"), (4, ""), (5, "4"), (6, "1")] {
        if !input.is_empty() {
            model.set_user_input(0, row, 1, input.to_string()).unwrap();
        }
    }
    model.add_table(0, "Sales", (1, 1, 6, 1), true).unwrap();
    // Row 3 (2) hidden; rows 2 and 6 hold the same 1; row 4 is blank.
    model.set_row_hidden(0, 3, true).unwrap();
    for (row, function) in [(8, 9), (9, 109)] {
        model
            .set_user_input(0, row, 3, format!("=SUBTOTAL({function},Sales[Qty])"))
            .unwrap();
    }
    model.evaluate();
    let temp_file_name = "temp_file_test_filtered_table.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let table_xml = part_text(temp_file_name, "xl/tables/table1.xml");
    let mut reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert!(
        table_xml.contains(
            r#"<autoFilter ref="A1:A6"><filterColumn colId="0"><filters blank="1"><filter val="1"/><filter val="4"/></filters></filterColumn></autoFilter>"#
        ),
        "{table_xml}"
    );
    assert!(reloaded.workbook.tables["Sales"].has_filters);
    reloaded.evaluate();
    for row in [8, 9] {
        assert_eq!(
            reloaded.get_formatted_cell_value(0, row, 3).unwrap(),
            "6",
            "row {row}"
        );
        assert_eq!(model.get_formatted_cell_value(0, row, 3).unwrap(), "6");
    }
}

#[test]
fn tables_on_two_sheets_get_their_own_parts() {
    let mut model = new_empty_model();
    model.add_sheet("Second").unwrap();
    for sheet in 0..2 {
        for (row, input) in [(1, "Qty"), (2, "1"), (3, "2")] {
            model
                .set_user_input(sheet, row, 1, input.to_string())
                .unwrap();
        }
    }
    model.add_table(0, "First", (1, 1, 3, 1), false).unwrap();
    model.add_table(1, "Other", (1, 1, 3, 1), false).unwrap();
    model
        .set_user_input(0, 5, 1, "=SUM(Other[Qty])+SUM(First[Qty])".to_string())
        .unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_tables_two_sheets.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let second = part_text(temp_file_name, "xl/tables/table2.xml");
    let second_rels = part_text(temp_file_name, "xl/worksheets/_rels/sheet2.xml.rels");
    let mut reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert!(second.contains(r#"id="2" name="Other""#), "{second}");
    assert!(
        second_rels.contains(r#"Target="../tables/table2.xml""#),
        "{second_rels}"
    );
    assert_eq!(reloaded.workbook.tables, model.workbook.tables);
    reloaded.evaluate();
    assert_eq!(reloaded.get_formatted_cell_value(0, 5, 1).unwrap(), "6");
}

/// The Excel-authored tables.xlsx keeps the text of every formula through an export, and writes
/// structured references as Excel's own file has them.
#[test]
fn excel_authored_tables_keep_their_formula_text() {
    let model = load_from_xlsx("tests/calc_test_tables/tables.xlsx", "en", "UTC", "en").unwrap();
    let temp_file_name = "temp_file_test_excel_tables.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    // Excel's forms (the original's `<f>`): `#Totals` of a table without a totals row (cached
    // `#REF!`), a column name that ends with a space, an `@` inside a column name.
    for (part, formula) in [
        ("xl/worksheets/sheet2.xml", "<f>SUM(Table2[#Totals])</f>"),
        ("xl/worksheets/sheet3.xml", "<f>SUM(Table3[[the two ]])</f>"),
        (
            "xl/worksheets/sheet8.xml",
            "<f>SUM(Table9[Totals &apos;[&apos;]^&amp;*@&apos;#$,,&lt;&gt;])</f>",
        ),
    ] {
        let xml = part_text(temp_file_name, part);
        assert!(xml.contains(formula), "{formula} in {part}");
    }
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert_eq!(reloaded.workbook.tables.len(), model.workbook.tables.len());
    let mut formulas = 0;
    for (sheet, worksheet) in model.workbook.worksheets.iter().enumerate() {
        let sheet = u32::try_from(sheet).unwrap();
        for (row, column, cell) in worksheet.sheet_data.cells() {
            if cell.get_formula().is_none() {
                continue;
            }
            formulas += 1;
            // The export drops the space after an argument comma (`SUBTOTAL(9, T[C])`), as it
            // did before table parts; only that spacing is set aside.
            let text = |m: &Model| {
                m.get_cell_formula(sheet, row, column)
                    .unwrap()
                    .map(|f| f.replace(", ", ","))
            };
            assert_eq!(
                text(&reloaded),
                text(&model),
                "{} row {row} column {column}",
                worksheet.name
            );
        }
    }
    assert!(formulas > 100, "{formulas}");
}

/// T-4157: the Excel-authored tables.xlsx carries `dataDxfId` on table columns and `headerRowDxfId`
/// alone on Table7; import reads each from its own attribute and the export writes every one back.
#[test]
fn excel_authored_table_dxf_ids_round_trip() {
    let model = load_from_xlsx("tests/calc_test_tables/tables.xlsx", "en", "UTC", "en").unwrap();
    let table7 = &model.workbook.tables["Table7"];
    assert_eq!(
        (table7.header_row_dxf_id, table7.data_dxf_id),
        (Some(17), None),
        "the header style is not copied into the data style"
    );
    let data_ids: Vec<_> = table7.columns.iter().map(|c| c.data_dxf_id).collect();
    assert_eq!(
        data_ids,
        [13, 11, 9, 7, 5, 3].map(Some),
        "Table7 column dataDxfId"
    );

    let temp_file_name = "temp_file_test_excel_table_dxf_ids.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    for (name, table) in &model.workbook.tables {
        let back = &reloaded.workbook.tables[name];
        assert_eq!(
            (
                table.header_row_dxf_id,
                table.data_dxf_id,
                table.totals_row_dxf_id
            ),
            (
                back.header_row_dxf_id,
                back.data_dxf_id,
                back.totals_row_dxf_id
            ),
            "{name}"
        );
        let ids = |t: &rustsheet_engine::types::Table| {
            t.columns
                .iter()
                .map(|c| (c.header_row_dxf_id, c.data_dxf_id, c.totals_row_dxf_id))
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(back), ids(table), "{name} columns");
    }
}

/// The filters of the Excel-authored tables.xlsx (a custom filter on Table4, a values filter on
/// Table16) are written back as the file had them, and read again (T-2838).
#[test]
fn excel_authored_table_filters_keep_their_criteria() {
    let model = load_from_xlsx("tests/calc_test_tables/tables.xlsx", "en", "UTC", "en").unwrap();
    let temp_file_name = "temp_file_test_excel_table_filters.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let parts: String = (1..=model.workbook.tables.len())
        .map(|id| part_text(temp_file_name, &format!("xl/tables/table{id}.xml")))
        .collect();
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    for filter in [
        r#"<filterColumn colId="0"><customFilters><customFilter operator="greaterThan" val="20"/></customFilters></filterColumn>"#,
        r#"<filterColumn colId="0"><filters><filter val="Germany"/><filter val="Spain"/></filters></filterColumn>"#,
    ] {
        assert!(parts.contains(filter), "{filter} in {parts}");
    }
    assert_eq!(
        model.workbook.tables["Table4"].filter_columns,
        vec![FilterColumn {
            col_id: 0,
            criterion: FilterCriterion::Custom {
                and: false,
                conditions: vec![CustomFilter {
                    operator: Some("greaterThan".to_string()),
                    val: "20".to_string(),
                }],
            },
        }]
    );
    for (name, table) in &model.workbook.tables {
        assert_eq!(
            reloaded.workbook.tables[name].filter_columns, table.filter_columns,
            "{name}"
        );
        assert_eq!(
            reloaded.workbook.tables[name].has_filters, table.has_filters,
            "{name}"
        );
    }
}

/// Criteria with a blank choice, an AND of two conditions, an operand that needs escaping and a
/// kind whose criterion is not kept survive an export.
#[test]
fn filter_columns_of_every_kind_survive_an_export() {
    let mut model = new_empty_model();
    for (row, input) in [(1, "Qty"), (2, "1"), (3, "2")] {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
        model.set_user_input(0, row, 2, input.to_string()).unwrap();
        model.set_user_input(0, row, 3, input.to_string()).unwrap();
    }
    model.set_user_input(0, 1, 2, "B".to_string()).unwrap();
    model.set_user_input(0, 1, 3, "C".to_string()).unwrap();
    model.add_table(0, "Sales", (1, 1, 3, 3), true).unwrap();
    let filter_columns = vec![
        FilterColumn {
            col_id: 0,
            criterion: FilterCriterion::Values {
                values: vec!["1".to_string(), "a<&\"b".to_string()],
                blank: true,
                calendar_type: None,
                date_groups: Vec::new(),
            },
        },
        FilterColumn {
            col_id: 1,
            criterion: FilterCriterion::Custom {
                and: true,
                conditions: vec![
                    CustomFilter {
                        operator: Some("greaterThanOrEqual".to_string()),
                        val: "1".to_string(),
                    },
                    CustomFilter {
                        operator: None,
                        val: "*x*".to_string(),
                    },
                ],
            },
        },
        FilterColumn {
            col_id: 2,
            criterion: FilterCriterion::Other,
        },
    ];
    if let Some(table) = model.workbook.tables.get_mut("Sales") {
        table.filter_columns.clone_from(&filter_columns);
    }
    let temp_file_name = "temp_file_test_filter_kinds.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    let table = &reloaded.workbook.tables["Sales"];
    assert_eq!(table.filter_columns, filter_columns);
    assert!(table.has_filters);
}

/// Top 10, dynamic, color, icon and date-group criteria are written back as the file had them, so
/// Excel still calls the table's hidden rows filtered after a round trip (T-4004; xlsx-storage.md
/// "Table parts in the export"). A color filter's `dxfId` is written only for an existing dxf.
#[test]
fn top10_dynamic_color_icon_and_date_criteria_survive_an_export() {
    let mut model = new_empty_model();
    for column in 1..=6 {
        for (row, input) in [
            (1, format!("H{column}")),
            (2, "1".to_string()),
            (3, "2".to_string()),
        ] {
            model.set_user_input(0, row, column, input).unwrap();
        }
    }
    model.add_table(0, "Sales", (1, 1, 3, 6), true).unwrap();
    model.workbook.styles.dxfs.push(Dxf::default());
    let filter_columns = vec![
        FilterColumn {
            col_id: 0,
            criterion: FilterCriterion::Top10 {
                top: false,
                percent: true,
                val: "2".to_string(),
                filter_val: Some("1.5".to_string()),
            },
        },
        FilterColumn {
            col_id: 1,
            criterion: FilterCriterion::Dynamic {
                kind: "today".to_string(),
                val: Some("45000".to_string()),
                max_val: Some("45001".to_string()),
                val_iso: Some("2023-03-15T00:00:00".to_string()),
                max_val_iso: Some("2023-03-16T00:00:00".to_string()),
            },
        },
        FilterColumn {
            col_id: 2,
            criterion: FilterCriterion::Color {
                dxf_id: Some(0),
                cell_color: false,
            },
        },
        FilterColumn {
            col_id: 3,
            criterion: FilterCriterion::Icon {
                icon_set: "3Arrows".to_string(),
                icon_id: Some(1),
            },
        },
        FilterColumn {
            col_id: 4,
            criterion: FilterCriterion::Values {
                values: vec!["x".to_string()],
                blank: false,
                calendar_type: Some("gregorian".to_string()),
                date_groups: vec![DateGroupItem {
                    grouping: "day".to_string(),
                    year: 2024,
                    month: Some(2),
                    day: Some(29),
                    hour: None,
                    minute: None,
                    second: None,
                }],
            },
        },
        FilterColumn {
            col_id: 5,
            criterion: FilterCriterion::Top10 {
                top: true,
                percent: false,
                val: "1".to_string(),
                filter_val: None,
            },
        },
    ];
    if let Some(table) = model.workbook.tables.get_mut("Sales") {
        table.filter_columns.clone_from(&filter_columns);
    }
    let temp_file_name = "temp_file_test_top10_dynamic_color_icon.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let xml = part_text(temp_file_name, "xl/tables/table1.xml");
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    for part in [
        r#"<filterColumn colId="0"><top10 top="0" percent="1" val="2" filterVal="1.5"/></filterColumn>"#,
        r#"<filterColumn colId="1"><dynamicFilter type="today" val="45000" valIso="2023-03-15T00:00:00" maxVal="45001" maxValIso="2023-03-16T00:00:00"/></filterColumn>"#,
        r#"<filterColumn colId="2"><colorFilter dxfId="0" cellColor="0"/></filterColumn>"#,
        r#"<filterColumn colId="3"><iconFilter iconSet="3Arrows" iconId="1"/></filterColumn>"#,
        r#"<filterColumn colId="4"><filters calendarType="gregorian"><filter val="x"/><dateGroupItem year="2024" month="2" day="29" dateTimeGrouping="day"/></filters></filterColumn>"#,
        r#"<filterColumn colId="5"><top10 val="1"/></filterColumn>"#,
    ] {
        assert!(xml.contains(part), "{part} not in {xml}");
    }
    let table = &reloaded.workbook.tables["Sales"];
    assert_eq!(table.filter_columns, filter_columns);
    assert!(table.has_filters);
}

/// A color filter whose `dxfId` is not one of the workbook's differential formats is written
/// without it, as the table's own dxf attributes are: Excel refuses a part pointing past the list.
#[test]
fn a_color_filter_past_the_dxfs_drops_its_dxf_id() {
    let mut model = new_empty_model();
    for (row, input) in [(1, "Qty"), (2, "1"), (3, "2")] {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    model.add_table(0, "Sales", (1, 1, 3, 1), true).unwrap();
    if let Some(table) = model.workbook.tables.get_mut("Sales") {
        table.filter_columns = vec![FilterColumn {
            col_id: 0,
            criterion: FilterCriterion::Color {
                dxf_id: Some(7),
                cell_color: true,
            },
        }];
    }
    let temp_file_name = "temp_file_test_color_filter_past_dxfs.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let xml = part_text(temp_file_name, "xl/tables/table1.xml");
    fs::remove_file(temp_file_name).unwrap();
    assert!(
        xml.contains(r#"<filterColumn colId="0"><colorFilter/></filterColumn>"#),
        "{xml}"
    );
}

/// A table filtered by a criterion from its file stays filtered while a criterion is left inside
/// it, and stops being filtered when a column delete takes the last one, as the export then
/// writes no filterColumn (T-4003).
#[test]
fn deleting_the_filtered_column_unfilters_the_table() {
    let mut model = new_empty_model();
    for (row, input) in [(1, "Cat"), (2, "a"), (3, "b"), (4, "a")] {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    for (row, input) in [(1, "Val"), (2, "1"), (3, "2"), (4, "4")] {
        model.set_user_input(0, row, 2, input.to_string()).unwrap();
    }
    for (row, input) in [(1, "Tag"), (2, "x"), (3, "y"), (4, "x")] {
        model.set_user_input(0, row, 3, input.to_string()).unwrap();
    }
    model.add_table(0, "Sales", (1, 1, 4, 3), true).unwrap();
    if let Some(table) = model.workbook.tables.get_mut("Sales") {
        table.filter_columns = vec![FilterColumn {
            col_id: 0,
            criterion: FilterCriterion::Values {
                values: vec!["a".to_string()],
                blank: false,
                calendar_type: None,
                date_groups: Vec::new(),
            },
        }];
    }
    model.delete_columns(0, 3, 1).unwrap();
    assert!(model.workbook.tables["Sales"].has_filters);
    model.delete_columns(0, 1, 1).unwrap();
    let table = &model.workbook.tables["Sales"];
    assert!(table.filter_columns.is_empty());
    assert!(!table.has_filters);

    let temp_file_name = "temp_file_test_deleted_filter_column.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let table_xml = part_text(temp_file_name, "xl/tables/table1.xml");
    fs::remove_file(temp_file_name).unwrap();
    assert!(!table_xml.contains("filterColumn"), "{table_xml}");
}

/// A column edit never makes a table filtered: one whose only filterColumn has no criterion is
/// not filtered on import (as Excel reads it, T-4003) and stays so after an unrelated delete.
#[test]
fn a_column_edit_does_not_filter_a_criterion_less_table() {
    let mut model = new_empty_model();
    for (column, header) in [(1, "Cat"), (2, "Val"), (3, "Tag")] {
        model
            .set_user_input(0, 1, column, header.to_string())
            .unwrap();
        model.set_user_input(0, 2, column, "1".to_string()).unwrap();
    }
    model.add_table(0, "Sales", (1, 1, 2, 3), false).unwrap();
    if let Some(table) = model.workbook.tables.get_mut("Sales") {
        table.filter_columns = vec![FilterColumn {
            col_id: 0,
            criterion: FilterCriterion::Other,
        }];
    }
    model.delete_columns(0, 3, 1).unwrap();
    let table = &model.workbook.tables["Sales"];
    assert_eq!(table.filter_columns.len(), 1);
    assert!(!table.has_filters);
}

/// A calculated column is written with its `calculatedColumnFormula` in file form and read back
/// as a calculated column (T-3033; xlsx-storage.md "A calculated column is saved as
/// `calculatedColumnFormula` plus one formula per row"); a column that was not filled gets none.
#[test]
fn a_calculated_column_keeps_its_column_formula() {
    let mut model = new_empty_model();
    for (row, column, input) in [(1, 1, "Qty"), (1, 2, "Dbl"), (2, 1, "2"), (3, 1, "3")] {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    model.add_table(0, "T", (1, 1, 3, 2), false).unwrap();
    model
        .set_user_input(0, 2, 2, "=T[@Qty]*4".to_string())
        .unwrap();
    // The fill on typed entry is the facade's; here the column is marked as the fill records it,
    // in file form, and the data row holds a different formula (an exception does not change it).
    model.workbook.tables.get_mut("T").unwrap().columns[1].calculated_column_formula =
        Some("T[[#This Row],[Qty]]*2".to_string());
    model.evaluate();

    let temp_file_name = "temp_file_test_calculated_column.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let table_xml = part_text(temp_file_name, "xl/tables/table1.xml");
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert!(
        table_xml
            .contains("<calculatedColumnFormula>T[[#This Row],[Qty]]*2</calculatedColumnFormula>"),
        "{table_xml}"
    );
    assert_eq!(table_xml.matches("calculatedColumnFormula>").count(), 2);
    let columns = &reloaded.workbook.tables["T"].columns;
    assert_eq!(columns[0].calculated_column_formula, None);
    assert_eq!(
        columns[1].calculated_column_formula.as_deref(),
        Some("T[[#This Row],[Qty]]*2")
    );
}

/// After a row or column insert or delete, the exported `calculatedColumnFormula` is the text
/// Excel saves for the same edit: the first data cell's formula as the edit displaced it
/// (T-4274 probes/table-edits/t4274-calc-column-*, the tables of their cached excel-graph
/// workbooks). The table is t2957-a1-reference's: T on A1:C4, C2 typed so the column fills.
#[test]
fn a_calculated_column_formula_follows_row_and_column_edits() {
    type Edit = fn(&mut rustsheet_engine::UserModel) -> Result<(), String>;
    let cases: [(&str, Edit, &str); 5] = [
        ("=A2*2", |u| u.insert_columns(0, 1, 1), "B2*2"),
        ("=A2*2", |u| u.delete_columns(0, 2, 1), "A2*2"),
        ("=A2*2", |u| u.insert_rows(0, 1, 1), "A3*2"),
        ("=A2*2", |u| u.insert_rows(0, 3, 1), "A2*2"),
        (
            "=[@Col]*2",
            |u| u.insert_columns(0, 1, 1),
            "T[[#This Row],[Col]]*2",
        ),
    ];
    for (index, (typed, edit, saved)) in cases.into_iter().enumerate() {
        let mut model = new_empty_model();
        let cells = [
            (1, 1, "Col"),
            (1, 2, "Other"),
            (1, 3, "Calc"),
            (2, 1, "1"),
            (3, 1, "2"),
        ];
        for (row, column, input) in cells.into_iter().chain([(4, 1, "4")]) {
            model
                .set_user_input(0, row, column, input.to_string())
                .unwrap();
        }
        model.add_table(0, "T", (1, 1, 4, 3), false).unwrap();
        model
            .set_typed_user_input(0, 2, 3, typed.to_string())
            .unwrap();
        let mut user = rustsheet_engine::UserModel::from_model(model);
        edit(&mut user).unwrap();
        let temp_file_name = format!("temp_file_test_calculated_column_edit_{index}.xlsx");
        save_to_xlsx(user.get_model(), &temp_file_name).unwrap();
        let table_xml = part_text(&temp_file_name, "xl/tables/table1.xml");
        fs::remove_file(&temp_file_name).unwrap();
        let element = format!("<calculatedColumnFormula>{saved}</calculatedColumnFormula>");
        assert!(
            table_xml.contains(&element),
            "{typed} case {index}: {table_xml}"
        );
    }
}
