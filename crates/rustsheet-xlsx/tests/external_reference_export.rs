#![allow(clippy::unwrap_used, reason = "tests")]
//! A formula over another workbook is saved as the source file stores it, `[1]Sort!$B:$C`, not
//! `'[1]Sort'!$B:$C`, which Excel would read as a sheet named `[1]Sort` (T-4140; the round-trip
//! census docs/research/xlsx-round-trip-2026-11-04.md §2.2).

use std::collections::BTreeSet;
use std::io::{Cursor, Read};

use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx;

/// The text of every `<f>` element holding a `[1]` book index, across the worksheets of `bytes`,
/// with XML entities decoded and `_xlfn.` prefixes dropped: how a writer escapes `"` and which
/// functions it prefixes is not what this test is about (the fixtures' hand-built `Refs` sheets
/// prefix SUMIFS, which Excel does not).
fn external_formulas(bytes: Vec<u8>) -> BTreeSet<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut formulas = BTreeSet::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        if !file.name().unwrap().starts_with("xl/worksheets/sheet") {
            continue;
        }
        let mut xml = String::new();
        file.read_to_string(&mut xml).unwrap();
        for element in xml.split("<f").skip(1) {
            let Some((_, rest)) = element.split_once('>') else {
                continue;
            };
            let Some((text, _)) = rest.split_once("</f>") else {
                continue;
            };
            if text.contains("[1]") {
                let text = (text.replace("&quot;", "\"").replace("&gt;", ">"))
                    .replace("&lt;", "<")
                    .replace("&apos;", "'")
                    .replace("&amp;", "&")
                    .replace("_xlfn.", "");
                formulas.insert(text);
            }
        }
    }
    formulas
}

#[test]
fn external_references_are_saved_as_the_source_stores_them() {
    let cases = [
        "external-link-cached-ranges",
        "external-link-cached-text",
        "external-link-cached-index-array",
        "external-link-cached-lookups",
        "external-link-cached-3d-span",
        "external-link-uncached-lookups",
        "external-link-unsorted-approx-lookups",
    ];
    for case in cases {
        let path = format!("../../eval/suites/fixtures/cases/{case}.xlsx");
        let source = external_formulas(std::fs::read(&path).unwrap());
        assert!(!source.is_empty(), "{case}: no external formula");
        let model = load_from_xlsx(&path, "en", "UTC", "en").unwrap();
        let saved = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
            .unwrap()
            .into_inner();
        let saved = external_formulas(saved);
        let missing: Vec<_> = source.difference(&saved).collect();
        assert!(
            missing.is_empty(),
            "{case}: not saved as stored: {missing:?}"
        );
    }
}
