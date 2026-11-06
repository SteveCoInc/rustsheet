//! Writes the seed corpus of the `pivot_import` fuzz target: the pivot parts of
//! crates/rustsheet-xlsx/tests/pivot_tables.xlsx joined by NUL bytes (the target's input), as
//! they are and with the shapes a hostile file uses: huge counts, indices past the shared items,
//! a numeric grouping, a calculated field and a page field (T-4133).
//!
//! Usage: `cargo run --bin pivot_seeds -- <directory>` (`seed_corpus.py` runs it).

use std::io::{Cursor, Read};
use std::path::PathBuf;

const TEMPLATE: &[u8] = include_bytes!("../../crates/rustsheet-xlsx/tests/pivot_tables.xlsx");

/// The parts the target replaces, in its input order; the second pivot's parts give a second seed.
const PIVOTS: [[&str; 3]; 2] = [
    [
        "xl/pivotCache/pivotCacheDefinition1.xml",
        "xl/pivotCache/pivotCacheRecords1.xml",
        "xl/pivotTables/pivotTable1.xml",
    ],
    [
        "xl/pivotCache/pivotCacheDefinition2.xml",
        "xl/pivotCache/pivotCacheRecords2.xml",
        "xl/pivotTables/pivotTable2.xml",
    ],
];

/// Edits applied to the first pivot's parts, one seed each: (part index, from, to).
const EDITS: &[(usize, &str, &str)] = &[
    (1, r#"count="6""#, r#"count="4294967295""#),
    (0, r#"recordCount="6""#, r#"recordCount="99999999999""#),
    (1, r#"<x v="2"/>"#, r#"<x v="4000000000"/>"#),
    (2, r#"<item x="1"/>"#, r#"<item x="-7"/>"#),
    (2, r#"<field x="1"/>"#, r#"<field x="-2"/>"#),
    (2, r#"fld="2""#, r#"fld="9""#),
    (2, r#"ref="A3:E7""#, r#"ref="XFD1048576:A1""#),
    (
        0,
        r#"minValue="3" maxValue="20"/>"#,
        r#"minValue="3" maxValue="20"/><fieldGroup base="2"><rangePr startNum="0" endNum="1E+300" groupInterval="1E-300"/><groupItems count="1"><s v="0-1"/></groupItems></fieldGroup>"#,
    ),
    (
        0,
        "</cacheFields>",
        r#"<cacheField name="Calc" formula="Sales*2+'Calc'" databaseField="0"/></cacheFields>"#,
    ),
    (
        2,
        "<dataFields",
        r#"<pageFields count="1"><pageField fld="1" item="99"/></pageFields><dataFields"#,
    ),
];

fn part(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut text = String::new();
    archive.by_name(name)?.read_to_string(&mut text)?;
    Ok(text)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or("usage: pivot_seeds <directory>")?
        .into();
    std::fs::create_dir_all(&directory)?;
    let mut archive = zip::ZipArchive::new(Cursor::new(TEMPLATE))?;
    let mut seeds: Vec<Vec<String>> = Vec::new();
    for names in PIVOTS {
        seeds.push(
            names
                .iter()
                .map(|name| part(&mut archive, name))
                .collect::<Result<_, _>>()?,
        );
    }
    let first = seeds[0].clone();
    for (index, from, to) in EDITS {
        let mut parts = first.clone();
        if !parts[*index].contains(from) {
            return Err(format!("seed edit {from:?} matches nothing in part {index}").into());
        }
        parts[*index] = parts[*index].replacen(from, to, 1);
        seeds.push(parts);
    }
    for (index, parts) in seeds.iter().enumerate() {
        std::fs::write(
            directory.join(format!("pivot-{index:02}")),
            parts.join("\0"),
        )?;
    }
    Ok(())
}
