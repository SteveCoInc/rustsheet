//! Writes the workbook of the probe request `xls-name-as-cell` with the xls crate's test writer
//! (`rustsheet_xls::test_support`): a BIFF8 file whose defined names are cells of the xlsx grid.
//! Usage: generate <output .xls> [<the reader's .xlsx of it>], or, for the file Excel wrote that goes with it,
//! generate --excel-authored <namesdemo.xls> <output .xls> [<the reader's .xlsx of it>]. Not part of the
//! workspace; see the README beside it.
use std::io::{Cursor, Write};

use rustsheet_xls::test_support::ptg::{ADD, MUL};
use rustsheet_xls::test_support::{
    bound_sheet, extern_sheet, formula, name, number, ptg, sheet, sup_book_internal, CachedValue,
    Record, Workbook,
};

/// The workbook stream in a version 3 compound file (512-byte sectors), the container Excel writes for an
/// .xls. The test writer's `Workbook::file` writes version 4 (4,096-byte sectors): this crate's reader takes
/// both, but a reader that expects Excel's container need not (xlrd 2.0.1 refuses it), and this file goes to
/// Excel. The stream is padded to 4,096 bytes as the test writer pads it, so that it lies in regular sectors.
fn excel_container(stream: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut file = cfb::CompoundFile::create_with_version(cfb::Version::V3, Cursor::new(Vec::new()))?;
    {
        let mut workbook = file.create_stream("Workbook")?;
        workbook.write_all(stream)?;
        workbook.write_all(&vec![0u8; 4096usize.saturating_sub(stream.len())])?;
    }
    file.flush()?;
    Ok(file.into_inner().into_inner())
}

/// `fixture` (crates/rustsheet-xls/tests/fixtures/namesdemo.xls, a workbook Excel wrote) with its name `vrai`
/// spelled `USA1`, a cell of the xlsx grid: four bytes in place, so that every other byte is Excel's. The `Lbl`
/// records stay in Excel's order (`Union`, `USA1`, `Year_Tot`). None if the fixture is not the file expected.
fn excel_authored(fixture: &[u8]) -> Option<Vec<u8>> {
    // A `Lbl` record's name starts 15 bytes into its data: after the record's type and size, `cch` is the
    // fourth byte of the data, and the byte before the name says its characters are one byte each.
    let is_the_name = |at: &usize| {
        *at >= 19 && fixture[at - 19..at - 17] == [0x18, 0] && fixture[at - 12] == 4 && fixture[at - 1] == 0
    };
    let found: Vec<usize> = (0..fixture.len().saturating_sub(3))
        .filter(|at| &fixture[*at..at + 4] == b"vrai")
        .filter(is_the_name)
        .collect();
    let [at] = found[..] else { return None };
    let mut out = fixture.to_vec();
    out[at..at + 4].copy_from_slice(b"USA1");
    Some(out)
}

/// The tokens of `a op b`.
fn binary(a: Vec<u8>, b: Vec<u8>, op: u8) -> Vec<u8> {
    [a, b, vec![op]].concat()
}

/// A worksheet of `rows` by `columns` used cells, `selected` or not. The caller selects one sheet, so that
/// Excel does not open the two as a group.
fn sized_sheet(rows: u32, columns: u16, selected: bool, records: Vec<Record>) -> Vec<Record> {
    let mut out = sheet(records);
    for record in &mut out {
        match record.kind {
            // `Dimensions`: the row past the last used one, and the column past the last used one.
            0x0200 => {
                record.data[4..8].copy_from_slice(&rows.to_le_bytes());
                record.data[10..12].copy_from_slice(&columns.to_le_bytes());
            }
            // `Window2`: a sheet that is not selected is not the one displayed either.
            0x023E if !selected => record.data[0..2].copy_from_slice(&0x00B6u16.to_le_bytes()),
            _ => {}
        }
    }
    out
}

fn main() -> std::io::Result<()> {
    let out = std::env::args()
        .nth(1)
        .expect(
            "usage: generate <output .xls> [<transcoded .xlsx>], \
             or generate --excel-authored <namesdemo.xls> <output .xls> [<transcoded .xlsx>]",
        );
    if out == "--excel-authored" {
        let usage = "usage: generate --excel-authored <namesdemo.xls> <output .xls> [<transcoded .xlsx>]";
        let fixture = std::fs::read(std::env::args().nth(2).expect(usage))?;
        let renamed = excel_authored(&fixture).expect("not namesdemo.xls: one name `vrai` expected");
        std::fs::write(std::env::args().nth(3).expect(usage), &renamed)?;
        return read_back(&renamed, std::env::args().nth(4), &["xl/workbook.xml"]);
    }
    // `PtgRef3d`, reference class, absolute: S!$A$1 (the writer's `reference_3d` is relative, and a relative
    // reference in a name is an offset from the cell that uses it).
    let on_s_a1 = vec![0x3A, 0, 0, 0, 0, 0, 0];
    // The names, in the order of their `Lbl` records: a formula names them by 1-based index.
    // (name, sheet scope: 0 is the workbook, definition)
    let names: Vec<(&str, u16, Vec<u8>)> = vec![
        ("TAX2024", 0, on_s_a1),    // 1: a cell of the xlsx grid, defined as S!$A$1
        ("USA1", 0, ptg::int(7)),   // 2: a cell of the xlsx grid, a constant
        ("Rate", 0, ptg::num(0.5)), // 3: control
        ("XFD1", 0, ptg::int(3)),   // 4: the xlsx grid's last column
        ("XFE1", 0, ptg::int(4)),   // 5: one column past it: a name in both
        ("IW1", 0, ptg::int(5)),    // 6: the first column past BIFF8's IV
        ("A65537", 0, ptg::int(9)), // 7: the first row past BIFF8's 65536
        ("_TAX2024", 0, ptg::int(13)), // 8: the name a rename of 1 would want
        ("Total", 0, binary(ptg::name(1), ptg::int(2), MUL)), // 9: a name defined with name 1
        ("TAX2024", 2, ptg::int(11)), // 10: the same spelling, scoped to sheet T
        ("XLRate", 0, ptg::int(2)), // 11: starts with XL
        ("R1C300", 0, ptg::int(4)), // 12: a cell of the xlsx grid, spelled in R1C1
        ("A1048576", 0, ptg::int(6)), // 13: the xlsx grid's last row
        ("A1048577", 0, ptg::int(8)), // 14: one row past it: a name in both
        ("tax2025", 0, ptg::int(10)), // 15: a cell of the xlsx grid, in lower case
    ];
    let mut book = Workbook::with_defaults();
    // After `BOF`: `CodePage` (UTF-16), and `Window1` (position, size, flags, active and first tab, selected
    // tabs, tab ratio).
    let window = [0u16, 0, 0x4000, 0x2000, 0x0038, 0, 0, 1, 600];
    book.globals
        .insert(1, Record::new(0x0042, 1200u16.to_le_bytes().to_vec()));
    book.globals.insert(
        2,
        Record::new(
            0x003D,
            window.iter().flat_map(|v| v.to_le_bytes()).collect(),
        ),
    );
    book.globals.push(bound_sheet("S", 0, 0));
    book.globals.push(bound_sheet("T", 0, 0));
    book.globals.push(sup_book_internal(2));
    // ixti 0: sheet S. ixti 1: the workbook, which is how Excel writes the qualifier of another sheet's name
    // (namesdemo.xls: `PtgNameX` through an entry (0, -2, -2)).
    book.globals.push(extern_sheet(&[(0, 0, 0), (0, -2, -2)]));
    for (text, scope, rgce) in &names {
        book.globals.push(name(text, *scope, rgce));
    }
    let n = |index: u32| ptg::name(index);
    let cell = |row: u16, cached: f64, rgce: Vec<u8>| {
        formula(row, 1, 15, CachedValue::Number(cached), &rgce)
    };
    let s = sized_sheet(
        14,
        2,
        true,
        vec![
            number(0, 0, 15, 5.0),                                     // A1 5
            cell(0, 6.0, binary(n(1), ptg::int(1), ADD)),              // B1 =TAX2024+1
            cell(1, 14.0, binary(n(2), ptg::int(2), MUL)),             // B2 =USA1*2
            cell(2, 1.0, binary(n(3), ptg::int(2), MUL)),              // B3 =Rate*2
            cell(3, 7.0, binary(n(4), n(5), ADD)),                     // B4 =XFD1+XFE1
            cell(4, 12.0, [n(1), n(2), ptg::func_var(4, 2)].concat()), // B5 =SUM(TAX2024,USA1)
            cell(5, 10.0, n(9)),                                       // B6 =Total
            cell(
                6,
                5.0,
                [ptg::str("TAX2024"), ptg::func_var(148, 1)].concat(),
            ), // B7 =INDIRECT("TAX2024")
            cell(7, 14.0, binary(n(6), n(7), ADD)),                    // B8 =IW1+A65537
            cell(8, 13.0, n(8)),                                       // B9 =_TAX2024
            cell(9, 4.0, binary(n(11), ptg::int(2), MUL)),             // B10 =XLRate*2
            cell(10, 5.0, binary(n(12), ptg::int(1), ADD)),            // B11 =R1C300+1
            cell(11, 14.0, binary(n(13), n(14), ADD)),                 // B12 =A1048576+A1048577
            cell(12, 11.0, binary(n(15), ptg::int(1), ADD)),           // B13 =tax2025+1
            cell(13, 11.0, ptg::name_x(1, 10)),                        // B14 =T!TAX2024, by `PtgNameX`
        ],
    );
    // T!A1 =TAX2024 (the sheet's own).
    let t = sized_sheet(
        1,
        1,
        false,
        vec![formula(0, 0, 15, CachedValue::Number(11.0), &n(10))],
    );
    book.sheets = vec![s, t];
    // `BOF` as Excel writes it (MS-XLS 2.4.21: `rupYear` MUST be 0x07CC or 0x07CD, `fWin` MUST be 1,
    // `verLowestBiff` MUST be 6): the twelve bytes after `vers` and `dt` of an Excel-authored file
    // (crates/rustsheet-xls/tests/fixtures/namesdemo.xls). The test writer's `bof` leaves them 0.
    const EXCEL_BOF_TAIL: [u8; 12] = [0xA9, 0x1F, 0xCD, 0x07, 0xC9, 0xC0, 0, 0, 0x06, 0x03, 0, 0];
    for record in book.globals.iter_mut().chain(book.sheets.iter_mut().flatten()) {
        if record.kind == 0x0809 {
            record.data[4..16].copy_from_slice(&EXCEL_BOF_TAIL);
        }
    }
    std::fs::write(&out, excel_container(&book.stream())?)?;
    let bytes = std::fs::read(&out)?;
    let sheets = ["xl/workbook.xml", "xl/worksheets/sheet1.xml", "xl/worksheets/sheet2.xml"];
    read_back(&bytes, std::env::args().nth(2), &sheets)
}

/// Prints what this repository's reader makes of `bytes`, an .xls: its report and `parts` of its .xlsx, which
/// goes to `xlsx` if there is one.
fn read_back(bytes: &[u8], xlsx: Option<String>, parts: &[&str]) -> std::io::Result<()> {
    match rustsheet_xls::transcode(bytes, &Default::default()) {
        Ok(done) => {
            println!("transcoded: {} bytes of xlsx", done.package.len());
            if let Some(path) = xlsx {
                std::fs::write(path, &done.package)?;
            }
            println!("{:#?}", done.report);
            for part in parts {
                println!(
                    "--- {part}\n{}",
                    rustsheet_xls::test_support::part(&done.package, part).unwrap_or_default()
                );
            }
        }
        Err(error) => println!("refused: {error:?}"),
    }
    Ok(())
}
