#![allow(clippy::unwrap_used, reason = "tests")]
//! A workbook's links to other workbooks survive a save: the externalLink parts, their
//! External-mode relationships and `<externalReferences>` are written from the cached books, so
//! loading the saved file gives the same books and the same values for the cells that read them
//! (T-4143; the round-trip census docs/research/xlsx-round-trip-2026-11-04.md §2.2).

use std::io::{Cursor, Read};

use rustsheet_xlsx::base::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::{load_from_xlsx, load_from_xlsx_bytes};

const CASES: [&str; 7] = [
    "external-link-cached-ranges",
    "external-link-cached-text",
    "external-link-cached-index-array",
    "external-link-cached-lookups",
    "external-link-cached-3d-span",
    "external-link-uncached-lookups",
    "external-link-unsorted-approx-lookups",
];

fn part(bytes: &[u8], name: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut file = archive.by_name(name).ok()?;
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    Some(xml)
}

#[test]
fn external_books_and_their_readers_survive_a_save() {
    for case in CASES {
        let path = format!("../../eval/suites/fixtures/cases/{case}.xlsx");
        let mut model = load_from_xlsx(&path, "en", "UTC", "en").unwrap();
        assert!(!model.workbook.external_books.is_empty(), "{case}: no book");
        assert!(
            model
                .workbook
                .external_books
                .iter()
                .all(|b| !b.target.is_empty()),
            "{case}: a book without its target"
        );
        let saved = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
            .unwrap()
            .into_inner();
        let workbook = load_from_xlsx_bytes(&saved, "saved", "en", "UTC").unwrap();
        let mut again = Model::from_workbook(workbook, "en").unwrap();
        assert_eq!(
            model.workbook.external_books, again.workbook.external_books,
            "{case}: the books changed"
        );
        model.evaluate();
        again.evaluate();
        for (sheet, worksheet) in model.workbook.worksheets.iter().enumerate() {
            let sheet = u32::try_from(sheet).unwrap();
            for (row, column, _) in worksheet.sheet_data.cells() {
                assert_eq!(
                    model.get_formatted_cell_value(sheet, row, column),
                    again.get_formatted_cell_value(sheet, row, column),
                    "{case}: sheet {sheet} R{row}C{column}"
                );
            }
        }
    }
}

#[test]
fn the_package_holds_one_external_link_part_per_book() {
    for case in CASES {
        let path = format!("../../eval/suites/fixtures/cases/{case}.xlsx");
        let model = load_from_xlsx(&path, "en", "UTC", "en").unwrap();
        let books = &model.workbook.external_books;
        let saved = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
            .unwrap()
            .into_inner();
        let content_types = part(&saved, "[Content_Types].xml").unwrap();
        let workbook = part(&saved, "xl/workbook.xml").unwrap();
        let workbook_rels = part(&saved, "xl/_rels/workbook.xml.rels").unwrap();
        let references = workbook
            .split_once("<externalReferences>")
            .and_then(|(_, rest)| rest.split_once("</externalReferences>"))
            .map_or_else(
                || panic!("{case}: no externalReferences"),
                |(inner, _)| inner,
            );
        // `<externalReferences>` sits between `<sheets>` and `<definedNames>` (CT_Workbook).
        let at = |tag: &str| workbook.find(tag).unwrap();
        assert!(at("</sheets>") < at("<externalReferences>"), "{case}");
        assert!(at("</externalReferences>") < at("<definedNames>"), "{case}");
        let ids: Vec<&str> = references
            .split("r:id=\"")
            .skip(1)
            .filter_map(|s| s.split_once('"').map(|(id, _)| id))
            .collect();
        assert_eq!(ids.len(), books.len(), "{case}");
        for (index, (id, book)) in ids.iter().zip(books).enumerate() {
            let n = index + 1;
            let name = format!("externalLinks/externalLink{n}.xml");
            assert!(
                workbook_rels.contains(&format!("Id=\"{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLink\" Target=\"{name}\"")),
                "{case}: no relationship {id} to {name}"
            );
            assert!(
                content_types.contains(&format!("PartName=\"/xl/{name}\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.externalLink+xml\"")),
                "{case}: no override for {name}"
            );
            assert!(part(&saved, &format!("xl/{name}")).is_some(), "{case}");
            let rels = part(
                &saved,
                &format!("xl/externalLinks/_rels/externalLink{n}.xml.rels"),
            )
            .unwrap();
            assert!(
                rels.contains(&format!(
                    "Target=\"{}\" TargetMode=\"External\"",
                    book.target
                )),
                "{case}: {rels}"
            );
        }
        // Every relationship id of the workbook part is distinct.
        let mut all: Vec<&str> = workbook_rels
            .split("Id=\"")
            .skip(1)
            .filter_map(|s| s.split_once('"').map(|(id, _)| id))
            .collect();
        let count = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), count, "{case}: a repeated relationship id");
    }
}
