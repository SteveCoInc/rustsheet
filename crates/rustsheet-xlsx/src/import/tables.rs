use std::io::Read;

use super::package::Package;

use super::xml::XmlNode;
use rustsheet_engine::types::{FilterColumn, FilterCriterion, Table, TableColumn, TableStyleInfo};

use crate::error::XlsxError;

use super::filter_criteria;
use super::util::{get_bool, get_bool_false};

// <table name="Table" displayName="Table" totalsRowCount ref="A1:D6">
//   <autoFilter ref="A1:D6">
//       <filterColumn colId="0">
//            <customFilters><customFilter operator="greaterThan" val=20></customFilter></customFilters>
//       </filterColumn>
//   </autoFilter>
//   <tableColumns count="5">
//      <tableColumn name="Monday" totalsRowFunction="sum" />
//      ...
//   </tableColumns>
//   <tableStyleInfo name="TableStyle5"/>
// </table>

/// Reads a table in an Excel workbook
pub(crate) fn load_table<R: Read + std::io::Seek>(
    archive: &mut Package<R>,
    path: &str,
    sheet_name: &str,
) -> Result<Table, XlsxError> {
    let file = archive.open(path)?;
    let document = XmlNode::parse(file)?;

    // table
    let table = &document;

    // Name and display name are normally the same and are unique in a workbook
    // They also need to be different from any defined name
    // A part written by LibreOffice can carry only displayName; Excel reaches the table through it
    // (docs/excel-semantics/xlsx-storage.md), so each attribute stands in for the other.
    let (name, display_name) = match (table.attribute("name"), table.attribute("displayName")) {
        (Some(n), Some(d)) => (n.to_string(), d.to_string()),
        (Some(n), None) => (n.to_string(), n.to_string()),
        (None, Some(d)) => (d.to_string(), d.to_string()),
        (None, None) => {
            return Err(XlsxError::Xml(
                "Corrupt XML structure: missing table name".to_string(),
            ))
        }
    };

    // Range of the table, including the totals if any and headers.
    let reference = table
        .attribute("ref")
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure: missing table ref".to_string()))?
        .to_string();

    // Either 0 or 1, indicates if the table has a formula for totals at the bottom of the table
    let totals_row_count = match table.attribute("totalsRowCount") {
        Some(s) => s.parse::<u32>().map_err(|_| {
            XlsxError::Xml("Corrupt XML structure: Invalid totalsRowCount".to_string())
        })?,
        None => 0,
    };

    // Either 0 or 1, indicates if the table has headers at the top of the table
    let header_row_count = match table.attribute("headerRowCount") {
        Some(s) => s.parse::<u32>().map_err(|_| {
            XlsxError::Xml("Corrupt XML structure: Invalid headerRowCount".to_string())
        })?,
        None => 1,
    };

    // style index of the header row of the table
    let header_row_dxf_id = if let Some(index_str) = table.attribute("headerRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // style index of the header row of the table
    let data_dxf_id = if let Some(index_str) = table.attribute("headerRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // style index of the totals row of the table
    let totals_row_dxf_id = if let Some(index_str) = table.attribute("totalsRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // Missing in Calc: styles can also be defined via a name:
    // headerRowCellStyle, dataCellStyle, totalsRowCellStyle

    // Missing in Calc: styles can also be applied to the borders:
    // headerRowBorderDxfId, tableBorderDxfId, totalsRowBorderDxfId

    // TODO: Conformant implementations should panic if header_row_dxf_id or data_dxf_id are out of bounds.

    // Note that filters are non dynamic: the criteria are kept for the export, and SUBTOTAL only
    // needs to know whether the table is filtered (`has_filters`, set below once the columns are
    // read).
    let auto_filter = table
        .descendants()
        .filter(|n| n.has_tag_name("autoFilter"))
        .collect::<Vec<&XmlNode>>();
    let filter_columns = auto_filter
        .first()
        .map(|filter| {
            filter
                .children()
                .filter(|n| n.has_tag_name("filterColumn"))
                .filter_map(filter_column)
                .collect()
        })
        .unwrap_or_default();

    // tableColumn
    let table_column = table
        .descendants()
        .filter(|n| n.has_tag_name("tableColumn"))
        .collect::<Vec<&XmlNode>>();
    let mut columns = Vec::new();
    for table_column in table_column {
        let column_name = table_column.attribute("name").ok_or_else(|| {
            XlsxError::Xml("Corrupt XML structure: missing column name".to_string())
        })?;
        let id = table_column.attribute("id").ok_or_else(|| {
            XlsxError::Xml("Corrupt XML structure: missing column id".to_string())
        })?;
        let id = id
            .parse::<u32>()
            .map_err(|_| XlsxError::Xml("Corrupt XML structure: invalid id".to_string()))?;

        // style index of the header row of the table
        let header_row_dxf_id = if let Some(index_str) = table_column.attribute("headerRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // style index of the header row of the table column
        let data_dxf_id = if let Some(index_str) = table_column.attribute("headerRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // style index of the totals row of the table column
        let totals_row_dxf_id = if let Some(index_str) = table_column.attribute("totalsRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // NOTE: Same as before, we should panic if indices to differential formatting records are out of bounds
        // Missing in Calc: styles can also be defined via a name:
        // headerRowCellStyle, dataCellStyle, totalsRowCellStyle

        columns.push(TableColumn {
            id,
            name: column_name.to_string(),
            totals_row_label: None,
            header_row_dxf_id,
            data_dxf_id,
            totals_row_function: None,
            totals_row_dxf_id,
            // Kept as file text; Excel does not fill the column on load (xlsx-storage.md,
            // "Excel does not fill a calculated column on load"), but a formula typed after load
            // re-fills it (references.md "Calculated columns" rule 9, probes/t3033-*), so it
            // marks the column as calculated.
            calculated_column_formula: table_column
                .child("calculatedColumnFormula")
                .and_then(XmlNode::text)
                .map(str::to_string),
        });
    }

    let has_filters = auto_filter
        .first()
        .is_some_and(|filter| has_active_criterion(filter, columns.len()));

    // tableInfo
    let table_info = table
        .descendants()
        .filter(|n| n.has_tag_name("tableInfo"))
        .collect::<Vec<&XmlNode>>();
    let style_info = match table_info.first() {
        Some(node) => {
            let name = node.attribute("name").map(|s| s.to_string());
            TableStyleInfo {
                name,
                show_first_column: get_bool_false(node, "showFirstColumn"),
                show_last_column: get_bool_false(node, "showLastColumn"),
                show_row_stripes: get_bool(node, "showRowStripes"),
                show_column_stripes: get_bool_false(node, "showColumnStripes"),
            }
        }
        None => TableStyleInfo {
            name: None,
            show_first_column: false,
            show_last_column: false,
            show_row_stripes: true,
            show_column_stripes: false,
        },
    };
    Ok(Table {
        name,
        display_name,
        reference,
        totals_row_count,
        header_row_count,
        header_row_dxf_id,
        data_dxf_id,
        totals_row_dxf_id,
        columns,
        style_info,
        has_filters,
        filter_columns,
        sheet_name: sheet_name.to_string(),
    })
}

/// Whether the table's `<autoFilter>` makes the sheet's hidden rows filtered for SUBTOTAL and
/// AGGREGATE: Excel does so only for a `<filterColumn>` that holds a criterion element and whose
/// `colId` is inside the table's `width` columns. A criterion-less `<filterColumn colId="0"/>`,
/// one with `colId` past the table, or an autoFilter with no filterColumn leaves them manual
/// hides (excel-graph, T-3081 and T-4003; docs/excel-semantics/numbers.md "SUBTOTAL and
/// AGGREGATE over hidden and filtered rows").
fn has_active_criterion(auto_filter: &XmlNode, width: usize) -> bool {
    auto_filter
        .children()
        .filter(|n| n.has_tag_name("filterColumn"))
        .any(|column| {
            let inside = column
                .attribute("colId")
                .and_then(|id| id.parse::<usize>().ok())
                .is_some_and(|id| id < width);
            inside && column.children().next().is_some()
        })
}

/// A `<filterColumn>`'s column and criterion; `None` for one without a valid `colId`, which
/// Excel's schema requires. The criterion is its first element, read by
/// [`filter_criteria::criterion`]; the column's `hiddenButton` and `showButton` are not kept.
fn filter_column(node: &XmlNode) -> Option<FilterColumn> {
    let col_id = node.attribute("colId")?.parse::<u32>().ok()?;
    let criterion = node
        .children()
        .next()
        .and_then(filter_criteria::criterion)
        .unwrap_or(FilterCriterion::Other);
    Some(FilterColumn { col_id, criterion })
}

#[cfg(test)]
mod tests {
    use super::{filter_column, has_active_criterion, XmlNode};
    use rustsheet_engine::types::FilterCriterion;

    fn criterion(xml: &str) -> FilterCriterion {
        let node = XmlNode::parse_str(xml).unwrap();
        filter_column(&node).unwrap().criterion
    }

    /// A values filter with date groups keeps them, so the export does not write it as a bare
    /// values list that shows nothing (T-2838, T-4004); an unknown criterion element is
    /// [`FilterCriterion::Other`].
    #[test]
    fn date_grouped_values_filters_keep_their_groups() {
        let dated = r#"<filterColumn colId="1"><filters><dateGroupItem year="2024" dateTimeGrouping="year"/></filters></filterColumn>"#;
        assert!(matches!(
            criterion(dated),
            FilterCriterion::Values { values, date_groups, .. } if values.is_empty() && date_groups.len() == 1
        ));
        let unknown = r#"<filterColumn colId="1"><unknownFilter/></filterColumn>"#;
        assert_eq!(criterion(unknown), FilterCriterion::Other);
    }

    /// Only a filterColumn with a criterion inside the table makes its hidden rows filtered, as
    /// Excel reads the T-3081 uploads (probes/t4003-filter-column-criterion).
    #[test]
    fn only_a_criterion_inside_the_table_filters() {
        let active = |xml: &str| has_active_criterion(&XmlNode::parse_str(xml).unwrap(), 2);
        let values =
            r#"<filterColumn colId="1"><filters><filter val="a"/></filters></filterColumn>"#;
        assert!(active(&format!("<autoFilter>{values}</autoFilter>")));
        let custom = r#"<filterColumn colId="0"><customFilters><customFilter operator="greaterThan" val="3"/></customFilters></filterColumn>"#;
        assert!(active(&format!("<autoFilter>{custom}</autoFilter>")));
        let top10 = r#"<filterColumn colId="0"><top10 val="2"/></filterColumn>"#;
        assert!(active(&format!("<autoFilter>{top10}</autoFilter>")));
        let bare = r#"<filterColumn colId="0"/>"#;
        assert!(!active(&format!("<autoFilter>{bare}</autoFilter>")));
        let past = r#"<filterColumn colId="5"><filters><filter val="a"/></filters></filterColumn>"#;
        assert!(!active(&format!("<autoFilter>{past}</autoFilter>")));
        assert!(!active(
            r#"<autoFilter><filterColumn colId="2"><filters/></filterColumn></autoFilter>"#
        ));
        assert!(active(&format!(
            "<autoFilter>{bare}{past}{values}</autoFilter>"
        )));
        assert!(!active("<autoFilter/>"));
    }

    /// Filter values are decoded from the file's `_xHHHH_` escapes, so the export's encoding
    /// gives the same text back.
    #[test]
    fn filter_values_decode_xlsx_escapes() {
        let escaped =
            r#"<filterColumn colId="0"><filters><filter val="a_x0041_"/></filters></filterColumn>"#;
        assert_eq!(
            criterion(escaped),
            FilterCriterion::Values {
                values: vec!["aA".to_string()],
                blank: false,
                calendar_type: None,
                date_groups: Vec::new(),
            }
        );
        assert!(
            filter_column(&XmlNode::parse_str(r#"<filterColumn colId="x"/>"#).unwrap()).is_none()
        );
    }
}
