//! The `xl/externalLinks/externalLinkN.xml` parts of the workbooks this one links to, written from
//! what the import cached for them ([`Workbook::external_books`]), so a saved `[n]` reference still
//! resolves, with its cached values, in Excel (T-4143). The other workbook is never opened
//! (directive 5): the part holds the cache as it was loaded.
//!
//! Book `n` of the model is part `externalLinkN.xml` and the `n`th `<externalReference>` of the
//! workbook part, so `[n]` in a saved formula names the same book it did in the source.

use std::collections::BTreeMap;

use rustsheet_engine::expressions::utils::number_to_column;
use rustsheet_engine::types::{ExternalBook, SpillValue, Workbook};

use super::escape::{escape_text, escape_xml};
use super::worksheets::{exported_error_literal, saved_number};
use super::xml_constants::XML_DECLARATION;

/// The content type of an externalLink part.
pub(super) const EXTERNAL_LINK_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.externalLink+xml";

/// The type of the workbook's relationship to an externalLink part.
pub(super) const EXTERNAL_LINK_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLink";

/// The type of the externalLink part's relationship to the other workbook, as Excel writes it
/// (eval/suites/fixtures/cases/external-link-cached-text.xlsx).
const EXTERNAL_LINK_PATH_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLinkPath";

/// The package path of the externalLink part of book `index` (0-based).
pub(super) fn external_link_part(index: usize) -> String {
    format!("xl/externalLinks/externalLink{}.xml", index + 1)
}

/// The package path of the relationships part of book `index`'s externalLink part.
pub(super) fn external_link_rels_part(index: usize) -> String {
    format!("xl/externalLinks/_rels/externalLink{}.xml.rels", index + 1)
}

/// Whether `book` is written as an `<externalBook>`. A book equal to the default holds nothing to
/// write: it is a link the import could not read as a workbook (DDE, OLE, a missing part). It is
/// written as an empty `<externalLink/>` so the books after it keep their `[n]`; whether Excel
/// opens such a part without a repair is unprobed (xlsx-storage.md, T-4143 entry; T-4937).
fn writes_book(book: &ExternalBook) -> bool {
    *book != ExternalBook::default()
}

/// The `<externalReferences>` element of the workbook part, one `<externalReference>` per book in
/// order with the relationship ids `first_id`, `first_id + 1`, …; empty when there is no book.
pub(super) fn external_references_xml(workbook: &Workbook, first_id: usize) -> String {
    if workbook.external_books.is_empty() {
        return String::new();
    }
    let references: Vec<String> = (0..workbook.external_books.len())
        .map(|index| format!("<externalReference r:id=\"rId{}\"/>", first_id + index))
        .collect();
    format!(
        "<externalReferences>{}</externalReferences>",
        references.concat()
    )
}

/// The workbook's relationships to the externalLink parts, ids from `first_id`.
pub(super) fn workbook_relationships(workbook: &Workbook, first_id: usize) -> Vec<String> {
    (0..workbook.external_books.len())
        .map(|index| {
            format!(
                "<Relationship Id=\"rId{}\" Type=\"{EXTERNAL_LINK_RELATIONSHIP}\" Target=\"externalLinks/externalLink{}.xml\"/>",
                first_id + index,
                index + 1
            )
        })
        .collect()
}

/// The content-type overrides of the externalLink parts.
pub(super) fn content_types(workbook: &Workbook) -> Vec<String> {
    (0..workbook.external_books.len())
        .map(|index| {
            format!(
                "<Override PartName=\"/{}\" ContentType=\"{EXTERNAL_LINK_CONTENT_TYPE}\"/>",
                external_link_part(index)
            )
        })
        .collect()
}

/// The relationships part of `book`'s externalLink part: its `TargetMode="External"` link to the
/// other workbook. `None` for a book written as an empty `<externalLink/>`.
pub(super) fn external_link_rels_xml(book: &ExternalBook) -> Option<String> {
    writes_book(book).then(|| {
        format!(
            "{XML_DECLARATION}\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"{EXTERNAL_LINK_PATH_RELATIONSHIP}\" Target=\"{}\" TargetMode=\"External\"/></Relationships>",
            escape_xml(&book.target)
        )
    })
}

/// The externalLink part of `book`: its sheet names, defined names and cached cells, in the
/// element order of `CT_ExternalBook`, the cells by sheet, row and column.
pub(super) fn external_link_xml(book: &ExternalBook) -> String {
    let open = format!(
        "{XML_DECLARATION}\n<externalLink xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">"
    );
    if !writes_book(book) {
        return format!("{open}</externalLink>");
    }
    let mut xml = open;
    xml.push_str("<externalBook xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" r:id=\"rId1\">");
    if !book.sheet_names.is_empty() {
        xml.push_str("<sheetNames>");
        for name in &book.sheet_names {
            xml.push_str(&format!("<sheetName val=\"{}\"/>", escape_xml(name)));
        }
        xml.push_str("</sheetNames>");
    }
    if !book.defined_names.is_empty() {
        xml.push_str("<definedNames>");
        for name in &book.defined_names {
            let sheet_id = name
                .sheet_id
                .map(|id| format!(" sheetId=\"{id}\""))
                .unwrap_or_default();
            xml.push_str(&format!(
                "<definedName name=\"{}\" refersTo=\"{}\"{sheet_id}/>",
                escape_xml(&name.name),
                escape_xml(&name.formula)
            ));
        }
        xml.push_str("</definedNames>");
    }
    let data = sheet_data(book);
    if !data.is_empty() {
        xml.push_str("<sheetDataSet>");
        for (sheet, rows) in &data {
            let refresh_error = if book.refresh_error_sheets.contains(sheet) {
                " refreshError=\"1\""
            } else {
                ""
            };
            if rows.is_empty() {
                xml.push_str(&format!("<sheetData sheetId=\"{sheet}\"{refresh_error}/>"));
                continue;
            }
            xml.push_str(&format!("<sheetData sheetId=\"{sheet}\"{refresh_error}>"));
            for (row, cells) in rows {
                xml.push_str(&format!("<row r=\"{row}\">"));
                for (column, value) in cells {
                    xml.push_str(&cell_xml(*row, *column, value));
                }
                xml.push_str("</row>");
            }
            xml.push_str("</sheetData>");
        }
        xml.push_str("</sheetDataSet>");
    }
    xml.push_str("</externalBook></externalLink>");
    xml
}

/// The cached cells of `book` by sheet, row and column, with an entry (possibly empty) for every
/// sheet the source had a `sheetData` for, holds a cell or is marked `refreshError`.
type SheetData<'a> = BTreeMap<u32, BTreeMap<i32, BTreeMap<i32, &'a SpillValue>>>;

fn sheet_data(book: &ExternalBook) -> SheetData<'_> {
    let mut data = SheetData::new();
    for sheet in book.data_sheets.iter().chain(&book.refresh_error_sheets) {
        data.entry(*sheet).or_default();
    }
    for ((sheet, row, column), value) in &book.cells {
        data.entry(*sheet)
            .or_default()
            .entry(*row)
            .or_default()
            .insert(*column, value);
    }
    data
}

/// A cached `<cell>`, typed as the import reads it back (`str`, `b`, `e`, or a number untyped).
/// A cell whose column has no name is left out: the import never caches one.
fn cell_xml(row: i32, column: i32, value: &SpillValue) -> String {
    let Some(column) = number_to_column(column) else {
        return String::new();
    };
    let (kind, text) = match value {
        SpillValue::Number(v) => ("", saved_number(*v).to_string()),
        SpillValue::Text(text) => (" t=\"str\"", escape_text(text).into_owned()),
        SpillValue::Boolean(b) => (" t=\"b\"", if *b { "1" } else { "0" }.to_string()),
        SpillValue::Error(error) => (" t=\"e\"", exported_error_literal(error).to_string()),
    };
    format!("<cell r=\"{column}{row}\"{kind}><v>{text}</v></cell>")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rustsheet_engine::expressions::token::Error;
    use rustsheet_engine::types::DefinedName;
    use std::collections::HashMap;

    fn book() -> ExternalBook {
        ExternalBook {
            target: "a&b.xlsx".to_string(),
            sheet_names: vec!["Uses".to_string(), "Defines".to_string()],
            defined_names: vec![DefinedName {
                name: "NR_To_A1".to_string(),
                formula: "='Defines'!$A$1".to_string(),
                sheet_id: Some(1),
                hidden: false,
                comment: None,
            }],
            cells: HashMap::from([
                ((1, 2, 2), SpillValue::Number(142.0)),
                ((0, 1, 1), SpillValue::Text("<x>".into())),
                ((0, 1, 2), SpillValue::Boolean(true)),
                ((1, 2, 1), SpillValue::Error(Error::NA)),
            ]),
            refresh_error_sheets: vec![2],
            data_sheets: vec![0, 1, 2, 4],
        }
    }

    #[test]
    fn writes_the_cache_in_schema_order() {
        let xml = external_link_xml(&book());
        let body = xml.split_once('\n').unwrap().1;
        assert_eq!(
            body,
            "<externalLink xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\
             <externalBook xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" r:id=\"rId1\">\
             <sheetNames><sheetName val=\"Uses\"/><sheetName val=\"Defines\"/></sheetNames>\
             <definedNames><definedName name=\"NR_To_A1\" refersTo=\"=&apos;Defines&apos;!$A$1\" sheetId=\"1\"/></definedNames>\
             <sheetDataSet>\
             <sheetData sheetId=\"0\"><row r=\"1\"><cell r=\"A1\" t=\"str\"><v>&lt;x&gt;</v></cell><cell r=\"B1\" t=\"b\"><v>1</v></cell></row></sheetData>\
             <sheetData sheetId=\"1\"><row r=\"2\"><cell r=\"A2\" t=\"e\"><v>#N/A</v></cell><cell r=\"B2\"><v>142</v></cell></row></sheetData>\
             <sheetData sheetId=\"2\" refreshError=\"1\"/>\
             <sheetData sheetId=\"4\"/>\
             </sheetDataSet></externalBook></externalLink>"
        );
        let rels = external_link_rels_xml(&book()).unwrap();
        assert!(rels.contains("Target=\"a&amp;b.xlsx\" TargetMode=\"External\""));
    }

    #[test]
    fn an_unread_link_keeps_its_place_as_an_empty_part() {
        let empty = ExternalBook::default();
        assert!(external_link_xml(&empty).ends_with("main\"></externalLink>"));
        assert_eq!(external_link_rels_xml(&empty), None);
    }
}
