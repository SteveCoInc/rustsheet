use std::io::Read;

use super::package::Package;

use super::sheet_data::{check_stored_length, check_string_literals, MAX_NAME_BODY_UNITS};
use super::xml::XmlNode;
use rustsheet_engine::types::{DateSystem, DefinedName, Iteration, SheetState};

use crate::error::XlsxError;
use crate::util::name_from_xlsx;

use super::shared_strings::decode_xlsx_escapes;
use super::util::{get_attribute, parse_bool_with_default};

pub(crate) struct Sheet {
    pub(crate) name: String,
    pub(crate) sheet_id: u32,
    pub(crate) id: String,
    pub(crate) state: SheetState,
}

pub(crate) struct WorkbookXML {
    pub(crate) worksheets: Vec<Sheet>,
    pub(crate) defined_names: Vec<DefinedName>,
    /// The `r:id` of each `externalReference`, in order: the first is the book formulas call `[1]`.
    pub(crate) external_references: Vec<String>,
    /// The `cacheId` and `r:id` of each `<pivotCaches><pivotCache>`, in order; either is `None`
    /// when the element lacks it (the importer skips that cache).
    pub(crate) pivot_caches: Vec<(Option<String>, Option<String>)>,
    /// The date system `<workbookPr date1904>` selects.
    pub(crate) date_system: DateSystem,
    /// `calcPr@iterate` and its count and delta.
    pub(crate) iteration: Option<Iteration>,
}

/// The iteration settings of a `calcPr` element: on when `iterate` is `1` or `true`, with
/// `iterateCount` and `iterateDelta` each falling back to Excel's default when absent,
/// unparsable or outside Excel's range ([`Iteration::within_excel_range`]; workbooks are
/// untrusted).
fn iteration_from(calc_pr: Option<&XmlNode>) -> Option<Iteration> {
    let node = calc_pr?;
    if !matches!(node.attribute("iterate"), Some("1" | "true")) {
        return None;
    }
    let default = Iteration::default();
    let count = node
        .attribute("iterateCount")
        .and_then(|v| v.parse::<u32>().ok());
    let delta = node
        .attribute("iterateDelta")
        .and_then(|v| v.parse::<f64>().ok());
    Some(
        Iteration {
            max_iterations: count.unwrap_or(default.max_iterations),
            max_change: delta.unwrap_or(default.max_change),
        }
        .within_excel_range(),
    )
}

/// The date system a `workbookPr@date1904` attribute selects: `1` and `true` are 1904; absent,
/// `0`, `false` and anything else are 1900, never an error (workbooks are untrusted).
#[must_use]
pub fn date_system_from_attribute(value: Option<&str>) -> DateSystem {
    if matches!(value, Some("1" | "true")) {
        DateSystem::Excel1904
    } else {
        DateSystem::Excel1900
    }
}

pub(super) fn load_workbook<R: Read + std::io::Seek>(
    archive: &mut Package<R>,
) -> Result<WorkbookXML, XlsxError> {
    let file = archive.open("xl/workbook.xml")?;
    let doc = XmlNode::parse(file)?;
    let mut defined_names = Vec::new();
    let mut sheets = Vec::new();
    let date_system = date_system_from_attribute(
        doc.descendants()
            .find(|n| n.has_tag_name("workbookPr"))
            .and_then(|n| n.attribute("date1904")),
    );
    // Get the sheets
    let sheet_nodes: Vec<&XmlNode> = doc
        .descendants()
        .filter(|n| n.has_tag_name("sheet"))
        .collect();
    for sheet in sheet_nodes {
        let name = get_attribute(sheet, "name")?.to_string();
        // The engine stores a reference to a deleted sheet under the sheet name `*#REF!`
        // (`DELETED_SHEET`, T-2612). Excel refuses `*` in a sheet name, so no Excel file holds
        // one; refusing it here keeps a crafted file from capturing those references (T-4655).
        if name.contains('*') {
            return Err(XlsxError::Workbook(format!(
                "sheet name \"{name}\" contains \"*\", which Excel does not allow"
            )));
        }
        let sheet_id = get_attribute(sheet, "sheetId")?.to_string();
        let sheet_id = sheet_id.parse::<u32>()?;
        let id = sheet
            .relationship_id()
            .ok_or_else(|| XlsxError::Xml("Missing \"r:id\" XML attribute".to_string()))?
            .to_string();
        let state = match sheet.attribute("state") {
            Some("visible") | None => SheetState::Visible,
            Some("hidden") => SheetState::Hidden,
            Some("veryHidden") => SheetState::VeryHidden,
            Some(state) => return Err(XlsxError::Xml(format!("Unknown sheet state: {state}"))),
        };
        sheets.push(Sheet {
            name,
            sheet_id,
            id,
            state,
        });
    }
    // Get the defined names
    let name_nodes: Vec<&XmlNode> = doc
        .descendants()
        .filter(|n| n.has_tag_name("definedName"))
        .collect();
    for node in name_nodes {
        let name = name_from_xlsx(get_attribute(node, "name")?).to_string();
        let formula = node.text().unwrap_or("").to_string();
        check_stored_length(
            &formula,
            MAX_NAME_BODY_UNITS,
            &format!("definedName {name} body"),
        )?;
        check_string_literals(&formula, &format!("definedName {name} body"))?;
        // NOTE: In Excel the `localSheetId` is just the index of the worksheet and unrelated to the sheetId
        let sheet_id = match node.attribute("localSheetId") {
            Some(s) => {
                let index = s.parse::<usize>()?;
                let sheet = sheets.get(index).ok_or_else(|| {
                    XlsxError::Workbook(format!(
                        "definedName {name}: no sheet at localSheetId {index}"
                    ))
                })?;
                Some(sheet.sheet_id)
            }
            None => None,
        };
        // Excel still evaluates a hidden name; only its Name Manager hides it (T-3301). The
        // comment is an `ST_Xstring`, so `_xHHHH_` escapes decode, as `escape_xml` wrote them.
        let hidden = parse_bool_with_default(node.attribute("hidden"), false);
        let comment = node.attribute("comment").map(decode_xlsx_escapes);
        defined_names.push(DefinedName {
            name,
            formula,
            sheet_id,
            hidden,
            comment,
        });
    }
    let external_references = doc
        .descendants()
        .filter(|n| n.has_tag_name("externalReference"))
        // One without an id still takes its index, so the books after it keep theirs.
        .map(|n| n.relationship_id().unwrap_or_default().to_string())
        .collect();
    // Only the workbook's own list: `<x14:pivotCaches>` in an extension list holds slicer caches.
    let pivot_caches = doc
        .child("pivotCaches")
        .map(|caches| {
            caches
                .children()
                .filter(|n| n.has_tag_name("pivotCache"))
                .map(|n| {
                    (
                        n.attribute("cacheId").map(str::to_string),
                        n.relationship_id().map(str::to_string),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let iteration = iteration_from(doc.descendants().find(|n| n.has_tag_name("calcPr")));
    Ok(WorkbookXML {
        worksheets: sheets,
        defined_names,
        external_references,
        pivot_caches,
        iteration,
        date_system,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]
    use super::*;

    fn iteration(calc_pr: &str) -> Option<Iteration> {
        iteration_from(Some(&XmlNode::parse_str(calc_pr).unwrap()))
    }

    /// Each attribute falls back to Excel's default on its own when absent, unparsable or outside
    /// Excel's range (`Iteration::within_excel_range`, T-4048); `iterate` off is no iteration.
    #[test]
    fn calc_pr_iteration_settings() {
        let default = Iteration::default();
        assert_eq!(iteration(r#"<calcPr iterate="0" iterateCount="5"/>"#), None);
        assert_eq!(iteration(r#"<calcPr iterate="1"/>"#), Some(default));
        assert_eq!(
            iteration(r#"<calcPr iterate="true" iterateCount="32767" iterateDelta="0"/>"#),
            Some(Iteration {
                max_iterations: 32_767,
                max_change: 0.0,
            })
        );
        for count in ["0", "32768", "4294967295", "4294967296", "-1", "x"] {
            let xml = format!(r#"<calcPr iterate="1" iterateCount="{count}" iterateDelta="0.5"/>"#);
            assert_eq!(
                iteration(&xml),
                Some(Iteration {
                    max_iterations: default.max_iterations,
                    max_change: 0.5,
                }),
                "iterateCount {count}"
            );
        }
        for delta in ["-0.1", "NaN", "inf", "x"] {
            let xml = format!(r#"<calcPr iterate="1" iterateCount="7" iterateDelta="{delta}"/>"#);
            assert_eq!(
                iteration(&xml),
                Some(Iteration {
                    max_iterations: 7,
                    max_change: default.max_change,
                }),
                "iterateDelta {delta}"
            );
        }
    }
}
