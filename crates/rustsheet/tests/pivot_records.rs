//! The drill-down API (SPEC-0032 "API", T-4184): `Workbook::pivot_cell_records`, the source
//! records behind a pivot value cell. The cells each test lists hold Excel's values, read from a
//! probe's oracle sidecar (cited per test): the cell must show that value and the data field's
//! function over its records' source values must give it. Every other value cell of each pivot is
//! a consistency check against the engine's own value of the cell, not Excel evidence, and a grand
//! total covers every record.

use std::error::Error;

use rustsheet::{
    CellRef, Input, LoadOptions, PivotError, PivotFunction, PivotTableInput, SheetId, Value,
    Workbook, WorkbookOptions,
};

type Res = Result<(), Box<dyn Error>>;

/// The function of the data field the tests use.
#[derive(Clone, Copy, Debug)]
enum Function {
    Sum,
    Count,
    Average,
}

/// `function` over the source values of `records`, the source's records starting at row
/// `first_row` (1-based) in column `column` of `sheet`: Count counts every non-blank value, Sum
/// and Average take the numbers only (docs/excel-semantics/pivots.md "Aggregates over text,
/// blanks, booleans and errors").
fn aggregate(
    wb: &Workbook,
    source: (&str, &str, u32),
    function: Function,
    records: &[u32],
) -> Result<f64, Box<dyn Error>> {
    let (sheet, column, first_row) = source;
    let mut numbers = Vec::new();
    let mut count = 0_u32;
    for &record in records {
        let value = wb.value(format!("{sheet}!{column}{}", first_row + record).as_str())?;
        if value != Value::Empty {
            count += 1;
        }
        if let Value::Number(n) = value {
            numbers.push(n);
        }
    }
    let sum: f64 = numbers.iter().sum();
    Ok(match function {
        Function::Sum => sum,
        Function::Count => f64::from(count),
        Function::Average => sum / f64::from(u32::try_from(numbers.len())?),
    })
}

/// Checks that every value cell of the pivot `name` on `sheet` holds `function` of its records,
/// and that each `(a1, value)` of `excel` is that cell's value and its records' too. Returns the
/// count of value cells checked.
fn check(
    wb: &Workbook,
    (sheet, name): (SheetId, &str),
    source: (&str, &str, u32),
    function: Function,
    excel: &[(&str, f64)],
) -> Result<usize, Box<dyn Error>> {
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-12 * b.abs().max(1.0);
    let mut checked = 0;
    for (cell, value) in wb.pivot_table_cells(sheet, name)? {
        match wb.pivot_cell_records(sheet, name, cell) {
            Ok(records) => {
                let Value::Number(shown) = value else {
                    return Err(format!("{cell}: value cell {value:?}").into());
                };
                let mine = aggregate(wb, source, function, &records)?;
                assert!(close(mine, shown), "{cell}: shows {shown}, records {mine}");
                checked += 1;
            }
            Err(PivotError::NotAValueCell) => {}
            Err(e) => return Err(e.into()),
        }
    }
    for &(a1, expected) in excel {
        let cell = wb.cell_ref(a1)?;
        assert_eq!(wb.value(a1)?, Value::Number(expected), "{a1}");
        let records = wb.pivot_cell_records(sheet, name, cell)?;
        let mine = aggregate(wb, source, function, &records)?;
        assert!(
            close(mine, expected),
            "{a1}: Excel {expected}, records {mine}"
        );
    }
    Ok(checked)
}

fn loaded() -> Result<Workbook, Box<dyn Error>> {
    let bytes = include_bytes!("../../rustsheet-xlsx/tests/pivot_tables.xlsx");
    Ok(Workbook::from_xlsx_bytes(bytes, &LoadOptions::default())?)
}

/// crates/rustsheet-xlsx/tests/pivot_tables.xlsx (probes/getpivotdata-real-pivot): over Data!A1:C7
/// (East A 10, West A 20, East B 5, West B 7, East C 3, East A 4), `PivotSum` is Sum of Sales by
/// Region and Product at P!A3:E7 and `PivotAvg` Average of Sales by Region at P!A20:B23. Excel's
/// values are the sidecar's GETPIVOTDATA answers on those cells.
#[test]
fn a_loaded_pivots_value_cells_give_excels_values() -> Res {
    let wb = loaded()?;
    let p = wb.sheet_id("P")?;
    let sales = ("Data", "C", 2);
    let sums = [
        ("P!B5", 14.0), // H5, H6: East/A
        ("P!C6", 7.0),  // H14: West/B
        ("P!E5", 22.0), // H3: East
        ("P!E6", 27.0), // L2: West
        ("P!B7", 34.0), // H4, L5: A
        ("P!C7", 12.0), // M5: B
        ("P!D7", 3.0),  // N5: C
        ("P!E7", 49.0), // H1, H2: the grand total
    ];
    assert_eq!(
        check(&wb, (p, "PivotSum"), sales, Function::Sum, &sums)?,
        11
    );
    let cell = |a1: &str| wb.cell_ref(a1);
    assert_eq!(wb.pivot_cell_records(p, "PivotSum", cell("P!B5")?)?, [0, 5]);
    assert_eq!(wb.pivot_cell_records(p, "PivotSum", cell("P!C7")?)?, [2, 3]);
    assert_eq!(
        wb.pivot_cell_records(p, "PivotSum", cell("P!E5")?)?,
        [0, 2, 4, 5]
    );
    // West/C is a value cell no record reaches (H13: GETPIVOTDATA answers 0 there).
    assert_eq!(wb.pivot_cell_records(p, "PivotSum", cell("P!D6")?)?, []);
    // The grand total covers every record.
    let all = wb.pivot_cell_records(p, "PivotSum", cell("P!E7")?)?;
    assert_eq!(all, [0, 1, 2, 3, 4, 5]);

    // J1: East, 5.5; J2, J5: the grand total, 49 / 6.
    let averages = [("P!B21", 5.5), ("P!B23", 8.166_666_666_666_666)];
    let checked = check(&wb, (p, "PivotAvg"), sales, Function::Average, &averages)?;
    assert_eq!(checked, 3);
    assert_eq!(
        wb.pivot_cell_records(p, "PivotAvg", cell("P!B22")?)?,
        [1, 3]
    );
    Ok(())
}

/// Writes `cells` (A1, input) in one edit.
fn write(wb: &mut Workbook, cells: &[(&str, Input)]) -> Res {
    wb.edit(|tx| {
        for (a1, input) in cells {
            tx.set(*a1, input.clone())?;
        }
        Ok(())
    })?;
    Ok(())
}

fn add(wb: &mut Workbook, input: &PivotTableInput) -> Res {
    wb.edit(|tx| tx.add_pivot_table(input))?;
    Ok(())
}

/// probes/t4128-pivot-spike R: Sum of Sales by Region and Product over DataF!A1:C7, whose C2 is
/// `=1000`; Excel's refreshed layout of it (the sidecar's `pivots` cells R!B5:E7) shows East/A
/// 1004. The pivot here is created through the API over the same records, C2 a constant 1000.
#[test]
fn an_api_created_pivot_gives_excels_values() -> Res {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    wb.edit(|tx| {
        tx.add_sheet("DataF")?;
        tx.add_sheet("R")
    })?;
    let records = [
        ("East", "A", 1000.0),
        ("West", "A", 20.0),
        ("East", "B", 5.0),
        ("West", "B", 7.0),
        ("East", "C", 3.0),
        ("East", "A", 4.0),
    ];
    let mut cells = vec![
        ("DataF!A1".to_string(), Input::from("Region")),
        ("DataF!B1".to_string(), Input::from("Product")),
        ("DataF!C1".to_string(), Input::from("Sales")),
    ];
    for (row, (region, product, sales)) in (2..).zip(records) {
        cells.push((format!("DataF!A{row}"), Input::from(region)));
        cells.push((format!("DataF!B{row}"), Input::from(product)));
        cells.push((format!("DataF!C{row}"), Input::from(sales)));
    }
    let cells: Vec<(&str, Input)> = cells.iter().map(|(a, i)| (a.as_str(), i.clone())).collect();
    write(&mut wb, &cells)?;
    let source = wb.range_ref("DataF!A1:C7")?;
    let mut input = PivotTableInput::new(
        "Refreshed",
        source,
        wb.range_ref("R!A3")?,
        "Region",
        "Sales",
        PivotFunction::Sum,
    );
    input.column = Some("Product".into());
    add(&mut wb, &input)?;
    let r = wb.sheet_id("R")?;
    let excel = [
        ("R!B5", 1004.0),
        ("R!C5", 5.0),
        ("R!D5", 3.0),
        ("R!E5", 1012.0),
        ("R!B6", 20.0),
        ("R!C6", 7.0),
        ("R!E6", 27.0),
        ("R!B7", 1024.0),
        ("R!C7", 12.0),
        ("R!D7", 3.0),
        ("R!E7", 1039.0),
    ];
    let sales = ("DataF", "C", 2);
    let checked = check(&wb, (r, "Refreshed"), sales, Function::Sum, &excel)?;
    assert_eq!(checked, 11);
    // R!D6, West/C, is empty in Excel's layout: a value cell with no record.
    assert_eq!(wb.value("R!D6")?, Value::Empty);
    assert_eq!(
        wb.pivot_cell_records(r, "Refreshed", wb.cell_ref("R!D6")?)?,
        []
    );
    Ok(())
}

/// probes/t4135-pivot-aggregates: Data!A1:D5 holds Region, Kind, Num and Txt (East B 5 4, East A
/// 10 10, East A 4 "abc", West A 7 7), and its `A_count`, `A_average` and `A_sum` pivots show each
/// field under one aggregate by Region and Kind. Here each is one data field of its own, created
/// through the API, and Excel's value per Region and Kind is the sidecar's for that field
/// (`A_count` C6, I6 to I8, O6 to O8 for Txt; `A_average` and `A_sum` B6, B8, H6 to H8, N6 to N8 for
/// Num).
#[test]
fn count_and_average_pivots_give_excels_values() -> Res {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    let rows: [(&str, &str, f64, Input); 4] = [
        ("East", "B", 5.0, Input::from(4.0)),
        ("East", "A", 10.0, Input::from(10.0)),
        ("East", "A", 4.0, Input::from("abc")),
        ("West", "A", 7.0, Input::from(7.0)),
    ];
    let mut cells = vec![
        ("A1".to_string(), Input::from("Region")),
        ("B1".to_string(), Input::from("Kind")),
        ("C1".to_string(), Input::from("Num")),
        ("D1".to_string(), Input::from("Txt")),
    ];
    for (row, (region, kind, num, txt)) in (2..).zip(rows) {
        cells.push((format!("A{row}"), Input::from(region)));
        cells.push((format!("B{row}"), Input::from(kind)));
        cells.push((format!("C{row}"), Input::from(num)));
        cells.push((format!("D{row}"), txt));
    }
    let cells: Vec<(&str, Input)> = cells.iter().map(|(a, i)| (a.as_str(), i.clone())).collect();
    write(&mut wb, &cells)?;
    let sheet = wb.sheet_id("Sheet1")?;
    // Each pivot at its anchor (F1, F8, F15): the caption, then Column Labels; Row Labels, B, A,
    // Grand Total; East; West; Grand Total. So its value cells are G3:I5 from F1.
    let cases = [
        // East/B, East/A, East, West/A, West, A, the grand total.
        (
            "Ct",
            "Txt",
            PivotFunction::Count,
            Function::Count,
            "D",
            [1.0, 2.0, 3.0, 1.0, 1.0, 3.0, 4.0],
        ),
        (
            "Av",
            "Num",
            PivotFunction::Average,
            Function::Average,
            "C",
            [5.0, 7.0, 19.0 / 3.0, 7.0, 7.0, 7.0, 6.5],
        ),
        (
            "Sm",
            "Num",
            PivotFunction::Sum,
            Function::Sum,
            "C",
            [5.0, 14.0, 19.0, 7.0, 7.0, 21.0, 26.0],
        ),
    ];
    for ((name, field, function, aggregate, column, excel), top) in
        cases.into_iter().zip([1, 8, 15])
    {
        let input = PivotTableInput {
            column: Some("Kind".into()),
            ..PivotTableInput::new(
                name,
                wb.range_ref("A1:D5")?,
                wb.range_ref(&format!("F{top}"))?,
                "Region",
                field,
                function,
            )
        };
        add(&mut wb, &input)?;
        let at = |column: &str, row: u32| format!("{column}{}", top + row);
        let places = [
            at("G", 2),
            at("H", 2),
            at("I", 2),
            at("H", 3),
            at("I", 3),
            at("H", 4),
            at("I", 4),
        ];
        let excel: Vec<(&str, f64)> = places.iter().map(String::as_str).zip(excel).collect();
        let checked = check(&wb, (sheet, name), ("Sheet1", column, 2), aggregate, &excel)?;
        // Every cell of the 3 by 3 value area but West/B, which no record reaches.
        assert_eq!(checked, 8, "{name}");
        let records = |a1: String| -> Result<Vec<u32>, Box<dyn Error>> {
            Ok(wb.pivot_cell_records(sheet, name, wb.cell_ref(a1.as_str())?)?)
        };
        assert_eq!(records(at("I", 4))?, [0, 1, 2, 3]);
        assert_eq!(records(at("H", 2))?, [1, 2]);
        assert_eq!(records(at("G", 3))?, []);
    }
    Ok(())
}

#[test]
fn a_cell_that_is_no_value_cell_and_an_unknown_pivot_are_errors() -> Res {
    let wb = loaded()?;
    let p = wb.sheet_id("P")?;
    let records = |cell: CellRef| wb.pivot_cell_records(p, "PivotSum", cell);
    // The data caption, the column header, the row header, a column label, a row label, a
    // grand-total label, cells beside and below the pivot, and a cell of another sheet.
    for a1 in [
        "P!A3", "P!B3", "P!A4", "P!B4", "P!A5", "P!A7", "P!F5", "P!B8", "Data!B5",
    ] {
        let err = records(wb.cell_ref(a1)?).err();
        assert_eq!(err, Some(PivotError::NotAValueCell), "{a1}");
    }
    let err = wb.pivot_cell_records(p, "Nope", wb.cell_ref("P!B5")?).err();
    assert!(
        matches!(&err, Some(PivotError::UnknownPivot(n)) if n == "Nope"),
        "{err:?}"
    );
    Ok(())
}

/// A pivot that collapses to `#SPILL!` (it would meet a foreign cell) has no value cell, not
/// even its anchor.
#[test]
fn a_spilled_pivot_has_no_value_cell() -> Res {
    let mut wb = loaded()?;
    let p = wb.sheet_id("P")?;
    // Region by Product from P!K3 covers K3:O7; P!M5 is in the way.
    write(&mut wb, &[("P!M5", Input::from(1.0))])?;
    let input = PivotTableInput {
        column: Some("Product".into()),
        ..PivotTableInput::new(
            "Wide",
            wb.range_ref("Data!A1:C7")?,
            wb.range_ref("P!K3")?,
            "Region",
            "Sales",
            PivotFunction::Sum,
        )
    };
    add(&mut wb, &input)?;
    let spill = Value::Error(rustsheet::ErrorKind::Spill.into());
    assert_eq!(wb.value("P!K3")?, spill);
    for a1 in ["P!K3", "P!L5", "P!O7"] {
        let err = wb.pivot_cell_records(p, "Wide", wb.cell_ref(a1)?).err();
        assert_eq!(err, Some(PivotError::NotAValueCell), "{a1}");
    }
    Ok(())
}
