//! Table parts (`xl/tables/tableN.xml`), so the structured references the exported formulas keep
//! name tables the file has (SPEC-0016 step 3, T-2196).
//!
//! ```xml
//! <table xmlns="..." id="1" name="Sales" displayName="Sales" ref="A1:B4" totalsRowCount="1">
//!   <tableColumns count="2">
//!     <tableColumn id="1" name="Item" totalsRowLabel="Total"/>
//!     <tableColumn id="2" name="Qty" totalsRowFunction="custom">
//!       <totalsRowFormula>SUBTOTAL(109,Sales[Qty])</totalsRowFormula>
//!     </tableColumn>
//!   </tableColumns>
//!   <tableStyleInfo name="TableStyleMedium2" showFirstColumn="0" ... />
//! </table>
//! ```

use std::collections::HashSet;

use rustsheet_engine::{
    expressions::{
        parser::{parse_range, stringify::to_excel_string},
        types::CellReferenceRC,
        utils::number_to_column,
    },
    types::{Cell, DateGroupItem, FilterColumn, FilterCriterion, Table, Workbook, Worksheet},
    Model, ParsedFormula,
};

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

/// The content type of a table part.
pub(crate) const TABLE_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.table+xml";

/// The relationship type from a worksheet to one of its table parts.
pub(crate) const TABLE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table";

/// The tables of each worksheet, by sheet index, in the order they are exported (by name). The
/// `n`-th table of the list flattened in sheet order is the part `xl/tables/table{n}.xml` with
/// `id="{n}"` (from 1). A table on no sheet of the workbook, or whose `ref` is not a range, is
/// not exported: formulas cannot resolve a reference to it, so the export writes them as `#REF!`.
pub(crate) fn tables_by_sheet(workbook: &Workbook) -> Vec<Vec<&Table>> {
    workbook
        .worksheets
        .iter()
        .map(|worksheet| {
            let mut tables: Vec<&Table> = workbook
                .tables
                .values()
                .filter(|t| t.is_on_sheet(&worksheet.name) && parse_range(&t.reference).is_ok())
                .collect();
            tables.sort_by(|a, b| a.name.cmp(&b.name));
            tables
        })
        .collect()
}

/// The `<tableParts>` section of a worksheet with `count` tables, whose relationships follow the
/// worksheet's `first_id - 1` external hyperlinks in its rels part, or an empty string.
pub(crate) fn get_table_parts_section(count: usize, first_id: usize) -> String {
    if count == 0 {
        return String::new();
    }
    let parts: Vec<String> = (first_id..first_id + count)
        .map(|id| format!("<tablePart r:id=\"rId{id}\"/>"))
        .collect();
    format!(
        "<tableParts count=\"{count}\">{}</tableParts>",
        parts.join("")
    )
}

/// A differential-format attribute, written only when the index is one of the workbook's `dxfs`
/// (an imported index is not checked, and Excel refuses a part pointing past the list).
fn dxf_attribute(name: &str, dxf_id: Option<u32>, dxf_count: usize) -> String {
    match dxf_id {
        Some(id) if usize::try_from(id).is_ok_and(|i| i < dxf_count) => {
            format!(" {name}=\"{id}\"")
        }
        _ => String::new(),
    }
}

/// What the totals row holds for a column, as the table part records it: a formula is written as
/// `totalsRowFunction="custom"` with its `<totalsRowFormula>`, a text cell as `totalsRowLabel`,
/// and a number, boolean or error constant, or a value spilled into the cell, as
/// `totalsRowFunction="custom"` alone. Excel refuses a
/// table with a totals row whose cells lack it (xlsx-storage.md "Structured references in the
/// file"), and refuses a constant written as a label or with a `<totalsRowFormula>`; the bare
/// `custom` loads with the cell kept a constant (excel-graph uploads, 2026-11-06, T-4005;
/// xlsx-storage.md "Table parts in the export"). An empty cell needs nothing.
fn totals_row_metadata(
    cell: Option<&Cell>,
    context: &CellReferenceRC,
    parsed_formulas: &[ParsedFormula],
    shared_strings: &[String],
) -> (String, String) {
    let Some(cell) = cell else {
        return (String::new(), String::new());
    };
    if let Some(f) = cell.get_formula() {
        let Some(parsed) = usize::try_from(f).ok().and_then(|i| parsed_formulas.get(i)) else {
            return (String::new(), String::new());
        };
        let formula = to_excel_string(&parsed.evaluated, context);
        return (
            " totalsRowFunction=\"custom\"".to_string(),
            format!(
                "<totalsRowFormula>{}</totalsRowFormula>",
                escape_xml(&formula)
            ),
        );
    }
    match cell {
        Cell::SharedString { si, .. } => match usize::try_from(*si)
            .ok()
            .and_then(|i| shared_strings.get(i))
        {
            Some(label) => (
                format!(" totalsRowLabel=\"{}\"", escape_xml(label)),
                String::new(),
            ),
            None => (String::new(), String::new()),
        },
        // A value spilled into the totals row from a dynamic array outside it takes the same
        // bare `custom` as a constant: Excel refuses the file without it and loads it, keeping
        // the spill, with it (excel-graph, T-4421; xlsx-storage.md "Table parts in the export").
        Cell::NumberCell { .. }
        | Cell::BooleanCell { .. }
        | Cell::ErrorCell { .. }
        | Cell::SpillCell { .. } => (" totalsRowFunction=\"custom\"".to_string(), String::new()),
        _ => (String::new(), String::new()),
    }
}

/// The criterion written for a table filtered through the API with no kept criterion
/// (`has_filters` and no `filter_columns`): a values filter on its first column that keeps the
/// formatted text of each visible data row, plus `blank="1"` when a visible one is blank, which
/// is the form spec2xlsx writes and Excel reads as filtered (T-4363; xlsx-storage.md, "Table parts
/// in the export"). Excel does not re-apply it on load, so a text that is on a visible and a
/// hidden row (which a re-applied filter would show) changes nothing there. `None` when the
/// table is not such a table, has no header row or its range does not parse.
pub(crate) fn api_filter_column(model: &Model, sheet: u32, table: &Table) -> Option<FilterColumn> {
    if !table.has_filters || !table.filter_columns.is_empty() || table.header_row_count == 0 {
        return None;
    }
    let (column, top, _, bottom) = parse_range(&table.reference).ok()?;
    let totals = i32::try_from(table.totals_row_count).ok()?;
    let mut values: Vec<String> = Vec::new();
    let mut blank = false;
    let first = top.checked_add(1)?;
    let last = bottom.checked_sub(totals)?;
    // One pass over the row records, not `Model::is_row_hidden` (a linear scan) per data row, which
    // is quadratic on a big table (T-4748). The first record of a row decides, as it does there.
    let mut seen: HashSet<i32> = HashSet::new();
    let mut hidden: HashSet<i32> = HashSet::new();
    if let Ok(worksheet) = model.workbook.worksheet(sheet) {
        for record in &worksheet.rows {
            if (first..=last).contains(&record.r) && seen.insert(record.r) && record.hidden {
                hidden.insert(record.r);
            }
        }
    }
    for row in first..=last {
        if hidden.contains(&row) {
            continue;
        }
        let text = model
            .get_formatted_cell_value(sheet, row, column)
            .unwrap_or_default();
        if text.is_empty() {
            blank = true;
        } else {
            values.push(text);
        }
    }
    values.sort_unstable();
    values.dedup();
    Some(FilterColumn {
        col_id: 0,
        criterion: FilterCriterion::Values {
            values,
            blank,
            calendar_type: None,
            date_groups: Vec::new(),
        },
    })
}

/// The `<autoFilter>` of a table with a header row, over its header and data rows, as Excel writes
/// it, with the imported filter columns written back (`Table::filter_columns`). A table that is
/// filtered with no kept criterion (`Txn::add_table` with `filtered`, which makes every hidden row
/// of the sheet filtered for SUBTOTAL) is written with `api_filter`, the criterion
/// [`api_filter_column`] lists, which Excel reads as filtered (T-4363); a criterion-less
/// `<filterColumn>` only when there is none, which Excel reads as not filtered (T-4003;
/// xlsx-storage.md "Table parts in the export").
fn auto_filter(
    table: &Table,
    (c1, r1, c2, r2): (i32, i32, i32, i32),
    dxf_count: usize,
    api_filter: Option<&FilterColumn>,
) -> String {
    if table.header_row_count == 0 {
        return String::new();
    }
    let last_row = if table.totals_row_count > 0 {
        r2 - 1
    } else {
        r2
    };
    let (Some(left), Some(right)) = (number_to_column(c1), number_to_column(c2)) else {
        return String::new();
    };
    let filter = if !table.filter_columns.is_empty() {
        table
            .filter_columns
            .iter()
            .map(|column| filter_column(column, dxf_count))
            .collect()
    } else if table.has_filters {
        // A criterion-less filterColumn only when the filter could not be listed (a reference that
        // does not parse).
        api_filter.map_or_else(
            || "<filterColumn colId=\"0\"/>".to_string(),
            |column| filter_column(column, dxf_count),
        )
    } else {
        String::new()
    };
    format!("<autoFilter ref=\"{left}{r1}:{right}{last_row}\">{filter}</autoFilter>")
}

/// A `<filterColumn>` as ECMA-376-1:2016 section 18.3.2.7 writes it, with its kept criterion
/// (T-2838, T-4004). A color filter's `dxfId` is written only when it is one of the workbook's
/// `dxf_count` differential formats.
fn filter_column(column: &FilterColumn, dxf_count: usize) -> String {
    let col_id = column.col_id;
    let criterion = match &column.criterion {
        FilterCriterion::Values {
            values,
            blank,
            calendar_type,
            date_groups,
        } => {
            let mut xml = String::from("<filters");
            if *blank {
                xml.push_str(" blank=\"1\"");
            }
            xml.push_str(&optional_attribute(
                "calendarType",
                calendar_type.as_deref(),
            ));
            xml.push('>');
            for value in values {
                xml.push_str(&format!("<filter val=\"{}\"/>", escape_xml(value)));
            }
            for group in date_groups {
                xml.push_str(&date_group_item(group));
            }
            xml.push_str("</filters>");
            xml
        }
        FilterCriterion::Custom { and, conditions } => {
            let mut xml = String::from(if *and {
                "<customFilters and=\"1\">"
            } else {
                "<customFilters>"
            });
            for condition in conditions {
                xml.push_str("<customFilter");
                xml.push_str(&optional_attribute(
                    "operator",
                    condition.operator.as_deref(),
                ));
                xml.push_str(&format!(" val=\"{}\"/>", escape_xml(&condition.val)));
            }
            xml.push_str("</customFilters>");
            xml
        }
        FilterCriterion::Top10 {
            top,
            percent,
            val,
            filter_val,
        } => format!(
            "<top10{}{} val=\"{}\"{}/>",
            if *top { "" } else { " top=\"0\"" },
            if *percent { " percent=\"1\"" } else { "" },
            escape_xml(val),
            optional_attribute("filterVal", filter_val.as_deref()),
        ),
        FilterCriterion::Dynamic {
            kind,
            val,
            max_val,
            val_iso,
            max_val_iso,
        } => format!(
            "<dynamicFilter type=\"{}\"{}{}{}{}/>",
            escape_xml(kind),
            optional_attribute("val", val.as_deref()),
            optional_attribute("valIso", val_iso.as_deref()),
            optional_attribute("maxVal", max_val.as_deref()),
            optional_attribute("maxValIso", max_val_iso.as_deref()),
        ),
        FilterCriterion::Color { dxf_id, cell_color } => format!(
            "<colorFilter{}{}/>",
            dxf_attribute("dxfId", *dxf_id, dxf_count),
            if *cell_color { "" } else { " cellColor=\"0\"" },
        ),
        FilterCriterion::Icon { icon_set, icon_id } => format!(
            "<iconFilter iconSet=\"{}\"{}/>",
            escape_xml(icon_set),
            icon_id
                .map(|id| format!(" iconId=\"{id}\""))
                .unwrap_or_default(),
        ),
        FilterCriterion::Other => return format!("<filterColumn colId=\"{col_id}\"/>"),
    };
    format!("<filterColumn colId=\"{col_id}\">{criterion}</filterColumn>")
}

/// A `<dateGroupItem>` (ECMA-376-1:2016 section 18.3.2.4).
fn date_group_item(group: &DateGroupItem) -> String {
    let field = |name: &str, value: Option<u16>| {
        value
            .map(|v| format!(" {name}=\"{v}\""))
            .unwrap_or_default()
    };
    format!(
        "<dateGroupItem year=\"{}\"{}{}{}{}{} dateTimeGrouping=\"{}\"/>",
        group.year,
        field("month", group.month),
        field("day", group.day),
        field("hour", group.hour),
        field("minute", group.minute),
        field("second", group.second),
        escape_xml(&group.grouping),
    )
}

/// ` name="value"`, escaped, or an empty string when there is no value.
fn optional_attribute(name: &str, value: Option<&str>) -> String {
    value
        .map(|v| format!(" {name}=\"{}\"", escape_xml(v)))
        .unwrap_or_default()
}

/// The part `xl/tables/table{id}.xml` of `table`, which is on `worksheet`.
pub(crate) fn get_table_xml(
    table: &Table,
    id: usize,
    worksheet: &Worksheet,
    parsed_formulas: &[ParsedFormula],
    workbook: &Workbook,
    api_filter: Option<&FilterColumn>,
) -> String {
    let dxf_count = workbook.styles.dxfs.len();
    let area = parse_range(&table.reference).unwrap_or((1, 1, 1, 1));
    let (first_column, _, _, last_row) = area;
    let totals_row = (table.totals_row_count > 0).then_some(last_row);
    let mut columns = String::new();
    for (index, column) in table.columns.iter().enumerate() {
        let (totals_attributes, totals_formula) = match totals_row {
            Some(row) => {
                let column_index = i32::try_from(index)
                    .ok()
                    .and_then(|i| first_column.checked_add(i))
                    .unwrap_or(first_column);
                let context = CellReferenceRC {
                    sheet: worksheet.get_name(),
                    row,
                    column: column_index,
                };
                totals_row_metadata(
                    worksheet.cell(row, column_index),
                    &context,
                    parsed_formulas,
                    &workbook.shared_strings,
                )
            }
            None => (String::new(), String::new()),
        };
        // The column's own formula, kept in file form by the fill that set it (an exception in a
        // data row does not change it, xlsx-storage.md), not read back from a data cell.
        let calculated_formula = column
            .calculated_column_formula
            .as_ref()
            .map(|f| {
                format!(
                    "<calculatedColumnFormula>{}</calculatedColumnFormula>",
                    escape_xml(f)
                )
            })
            .unwrap_or_default();
        columns.push_str(&format!(
            "<tableColumn id=\"{}\" name=\"{}\"{totals_attributes}{}{}{}>{calculated_formula}{totals_formula}</tableColumn>",
            column.id,
            escape_xml(&column.name),
            dxf_attribute("headerRowDxfId", column.header_row_dxf_id, dxf_count),
            dxf_attribute("dataDxfId", column.data_dxf_id, dxf_count),
            dxf_attribute("totalsRowDxfId", column.totals_row_dxf_id, dxf_count),
        ));
    }
    let header_row_count = if table.header_row_count == 1 {
        String::new()
    } else {
        format!(" headerRowCount=\"{}\"", table.header_row_count)
    };
    let totals_row_count = if table.totals_row_count == 0 {
        String::new()
    } else {
        format!(" totalsRowCount=\"{}\"", table.totals_row_count)
    };
    let style = &table.style_info;
    let style_name = match &style.name {
        Some(name) => format!(" name=\"{}\"", escape_xml(name)),
        None => String::new(),
    };
    format!(
        "{XML_DECLARATION}\n<table xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
id=\"{id}\" name=\"{}\" displayName=\"{}\" ref=\"{}\"{header_row_count}{totals_row_count}{}{}{}>{}\
<tableColumns count=\"{}\">{columns}</tableColumns>\
<tableStyleInfo{style_name} showFirstColumn=\"{}\" showLastColumn=\"{}\" showRowStripes=\"{}\" showColumnStripes=\"{}\"/>\
</table>",
        escape_xml(&table.name),
        escape_xml(&table.display_name),
        escape_xml(&table.reference),
        dxf_attribute("headerRowDxfId", table.header_row_dxf_id, dxf_count),
        dxf_attribute("dataDxfId", table.data_dxf_id, dxf_count),
        dxf_attribute("totalsRowDxfId", table.totals_row_dxf_id, dxf_count),
        auto_filter(table, area, dxf_count, api_filter),
        table.columns.len(),
        u8::from(style.show_first_column),
        u8::from(style.show_last_column),
        u8::from(style.show_row_stripes),
        u8::from(style.show_column_stripes),
    )
}
