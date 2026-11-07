use rustsheet_engine::types::Workbook;

use super::external_links;
use super::rich_errors::{rich_value_parts, METADATA_RELATIONSHIP};
use super::xml_constants::{XML_DECLARATION, XML_WORKSHEET};

/// The relationship id of the first externalLink part: they come right after the sheets, as
/// Excel numbers them (eval/suites/fixtures/cases/external-link-cached-text.xlsx: sheets rId1–rId3,
/// externalLink rId4), and the pivot caches after them (T-4143).
pub(crate) fn first_external_link_id(workbook: &Workbook) -> usize {
    workbook.worksheets.len() + 1
}

/// The relationship id of the first pivot cache, right after the externalLink parts (`PivotPlan`).
pub(crate) fn first_pivot_cache_id(workbook: &Workbook) -> usize {
    first_external_link_id(workbook) + workbook.external_books.len()
}

/// With `rich`, the workbook also relates to the rich-value parts (T-1014). `pivot_caches` are
/// the relationships to the pivot caches, numbered from [`first_pivot_cache_id`] (`PivotPlan`).
pub(crate) fn get_workbook_xml_rels(
    workbook: &Workbook,
    rich: bool,
    pivot_caches: Vec<String>,
) -> String {
    let mut relationships_str: Vec<String> = vec![];
    let worksheet_count = workbook.worksheets.len() + 1;
    for id in 1..worksheet_count {
        relationships_str.push(format!(
            "<Relationship Id=\"rId{id}\" Type=\"{XML_WORKSHEET}\" Target=\"worksheets/sheet{id}.xml\"/>"
        ));
    }
    relationships_str.extend(external_links::workbook_relationships(
        workbook,
        first_external_link_id(workbook),
    ));
    let mut id = first_pivot_cache_id(workbook) + pivot_caches.len();
    relationships_str.extend(pivot_caches);
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>")
    );
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings\" Target=\"sharedStrings.xml\"/>")
    );
    id += 1;
    relationships_str.push(format!(
        "<Relationship Id=\"rId{id}\" Type=\"{METADATA_RELATIONSHIP}\" Target=\"metadata.xml\"/>"
    ));
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"theme/theme1.xml\"/>")
    );
    if rich {
        for (part, _, relationship, _) in rich_value_parts() {
            id += 1;
            let target = part.trim_start_matches("xl/");
            relationships_str.push(format!(
                "<Relationship Id=\"rId{id}\" Type=\"{relationship}\" Target=\"{target}\"/>"
            ));
        }
    }
    format!(
        "{XML_DECLARATION}\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{}</Relationships>",
        relationships_str.join("")
    )
}
