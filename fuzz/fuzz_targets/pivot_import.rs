//! Imports a workbook whose first pivot cache and pivot table parts are untrusted, then reads
//! the pivot back through GETPIVOTDATA (T-4133; SPEC-0015's limits, directive 5).
//!
//! Random bytes almost never reach the pivot reader inside a zip, so the input is the three
//! pivot parts themselves: the cache definition, the cache records and the table definition,
//! separated by NUL bytes. They replace those parts of crates/rustsheet-xlsx/tests/pivot_tables.xlsx
//! (a part the input leaves out keeps the fixture's bytes), and the package is rebuilt in memory.
#![no_main]

use std::io::{Cursor, Read, Write};

use libfuzzer_sys::fuzz_target;
use rustsheet::{Input, LoadOptions, Value, Workbook};
use zip::write::SimpleFileOptions;

/// The workbook the parts go into: a Data sheet and two pivots on sheet P.
const TEMPLATE: &[u8] = include_bytes!("../../crates/rustsheet-xlsx/tests/pivot_tables.xlsx");

/// The template's parts the input replaces, in input order.
const PARTS: [&str; 3] = [
    "xl/pivotCache/pivotCacheDefinition1.xml",
    "xl/pivotCache/pivotCacheRecords1.xml",
    "xl/pivotTables/pivotTable1.xml",
];

/// The template with `parts` (as many as given) in place of [`PARTS`]; `None` if the template
/// cannot be read or written, which is no finding.
fn package(parts: &[&[u8]]) -> Option<Vec<u8>> {
    let mut template = zip::ZipArchive::new(Cursor::new(TEMPLATE)).ok()?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for index in 0..template.len() {
        let mut entry = template.by_index(index).ok()?;
        let name = entry.name().ok()?.to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).ok()?;
        let given = PARTS
            .iter()
            .position(|part| *part == name)
            .and_then(|position| parts.get(position));
        out.start_file(name, options).ok()?;
        out.write_all(given.map_or(bytes.as_slice(), |part| part))
            .ok()?;
    }
    Some(out.finish().ok()?.into_inner())
}

/// GETPIVOTDATA reads the pivot model the import built: the grand total, an item and a missing
/// field, beside the table (A3:E7 in the template) and on the other sheet.
const FORMULAS: [(&str, &str); 4] = [
    ("P!H1", "=GETPIVOTDATA(\"Sum of Sales\",P!A3)"),
    (
        "P!H2",
        "=GETPIVOTDATA(\"Sum of Sales\",P!A3,\"Region\",\"East\")",
    ),
    (
        "P!H3",
        "=GETPIVOTDATA(\"Sales\",P!A3,\"Region\",\"West\",\"Product\",\"A\")",
    ),
    (
        "Data!H1",
        "=GETPIVOTDATA(\"Sum of Sales\",P!$A$3,\"Nothing\",1)",
    ),
];

fuzz_target!(|data: &[u8]| {
    let parts: Vec<&[u8]> = data.splitn(PARTS.len(), |byte| *byte == 0).collect();
    let Some(bytes) = package(&parts) else {
        return;
    };
    let opts = LoadOptions {
        recalc: true,
        max_uncompressed_bytes: 16 << 20,
        max_zip_entries: 500,
        ..LoadOptions::default()
    };
    let Ok(mut wb) = Workbook::from_xlsx_bytes(&bytes, &opts) else {
        return;
    };
    for (cell, formula) in FORMULAS {
        if wb.set(cell, Input::Formula(formula.to_string())).is_ok() {
            let _: Option<Value> = wb.value(cell).ok();
        }
    }
    for sheet in wb.sheets() {
        if let Ok(Some(range)) = wb.used_range(sheet.id) {
            let _ = wb.values(range);
        }
    }
    let _ = wb.to_xlsx_bytes(rustsheet::SaveOptions::default());
});
