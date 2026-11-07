// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::manual_let_else,
    clippy::ref_option,
    clippy::too_many_lines
)]
#![cfg_attr(test, expect(clippy::float_cmp))]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

//! # A note on shared formulas
//! Although both Excel and IronCalc uses shared formulas they are used in a slightly different way that cannot be mapped 1-1
//! In IronCalc _all_ formulas are shared and there is a list of shared formulas much like there is a list of shared strings.
//! In Excel the situation in more nuanced. A shared formula is shared amongst a rage of cells.
//! The top left cell would be the "mother" cell that would contain the shared formula:
//! ```xml
//! <c r="F4" t="str">
//!    <f t="shared" ref="F4:F8" si="42">A4+C4</f>
//!    <v>123</v>
//! </c>
//! ```
//! Cells in the range F4:F8 will then link to that formula like so:
//! ```xml
//! <c r="F6">
//!   <f t="shared" si="42"/>
//!   <v>1</v>
//! </c>
//! ```
//! Formula in F6 would then be 'A6+C6'
use std::collections::HashMap;

use itertools::Itertools;

use rustsheet_engine::{
    expressions::{
        parser::{
            static_analysis::{add_implicit_intersection, remove_redundant_implicit_intersection},
            stringify::{to_excel_array_formula_string, to_excel_string, to_rc_format},
            Node,
        },
        token::Error,
        types::CellReferenceRC,
        utils::number_to_column,
    },
    types::{ArrayKind, Cell, FormulaValue, Link, SpillValue, Worksheet},
    ParsedFormula,
};

use crate::export::conditional_formatting::get_conditional_formatting_xml;

use super::{
    escape::{escape_text, escape_xml},
    rich_errors::vm_attribute,
    tables::{get_table_parts_section, TABLE_RELATIONSHIP},
    xml_constants::XML_DECLARATION,
};

/// The number written to a cell's `<v>`: a kept subnormal of either sign is saved as 0, as Excel for the web
/// does (`EXP(-745)` is saved `<v>0</v>` while `(EXP(-745))=0` stays FALSE in memory; numbers.md
/// "Worksheet EXP (T-1931)", decision 0089, probes/subnormal-saved-as-zero). Only the evidenced domain is
/// flushed: negative and spilled/array kept subnormals are saved as 0 too (T-4355,
/// probes/subnormal-saved-negative-and-array); desktop Excel's save is unprobed, and `-0.0` keeps
/// its spelling. A stored positive constant is already 0 once loaded (numbers.md "A subnormal cell constant
/// reads as 0"), so the constant arm changes nothing observable.
fn saved_number(v: f64) -> f64 {
    if v != 0.0 && v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

/// The literal an error cell is written with. Excel's stored form ([`Error::stored_literal`]),
/// except the kinds Excel rejects as stored literals, which are saved as `#VALUE!`:
/// - `#SPILL!`, `#CALC!`, `#CONNECT!`, `#BLOCKED!` and `#FIELD!` (T-0999). Excel keeps the kind in
///   `vm` rich-value metadata, which the cell also gets ([`vm_attribute`], T-1014).
/// - The engine-only `#CIRC!`, `#ERROR!` and `#N/IMPL` (T-1133, docs/excel-semantics/syntax.md).
///
/// The eval harness must not use this fallback: it scores through `Error::stored_literal`.
pub(crate) fn exported_error_literal(error: &Error) -> &'static str {
    match error {
        Error::SPILL
        | Error::CALC
        | Error::CONNECT
        | Error::BLOCKED
        | Error::FIELD
        | Error::CIRC
        | Error::ERROR
        | Error::NIMPL => Error::VALUE.stored_literal(),
        _ => error.stored_literal(),
    }
}

fn get_range_str(row: i32, column: i32, width: i32, height: i32) -> Option<String> {
    let column1 = number_to_column(column)?;
    let column2 = number_to_column(column + width - 1)?;
    let row2 = row + height - 1;
    Some(format!("{column1}{row}:{column2}{row2}"))
}

fn get_cell_style_attribute(s: i32) -> String {
    if s == 0 {
        String::new()
    } else {
        format!(" s=\"{s}\"")
    }
}

/// The links of the worksheet sorted by (row, column) so that both the
/// `<hyperlinks>` section and the sheet rels assign relationship ids in the
/// same, deterministic order.
fn get_sorted_links(worksheet: &Worksheet) -> Vec<((i32, i32), &Link)> {
    worksheet
        .links
        .iter()
        .map(|(k, v)| (*k, v))
        .sorted_by_key(|x| x.0)
        .collect()
}

/// An external link whose target is `scheme://host/path#location` is exported with the
/// fragment in the `location` attribute of the `<hyperlink>` element, matching what
/// Excel does. Returns (target, location).
fn split_external_target(target: &str) -> (&str, Option<&str>) {
    match target.split_once('#') {
        Some((t, location)) => (t, Some(location)),
        None => (target, None),
    }
}

/// The `<hyperlinks>` section of the worksheet or an empty string if there are none.
/// External links reference a relationship (`rId1`, `rId2`...) in the sheet rels part,
/// see [`get_worksheet_xml_rels`].
fn get_hyperlinks_section(worksheet: &Worksheet) -> String {
    let links = get_sorted_links(worksheet);
    if links.is_empty() {
        return String::new();
    }
    let mut hyperlinks_str: Vec<String> = vec![];
    let mut rel_id = 0;
    for ((row, column), link) in links {
        let column_name = match number_to_column(column) {
            Some(name) => name,
            None => continue,
        };
        let cell_name = format!("{column_name}{row}");
        match link {
            Link::External { target, tooltip } => {
                rel_id += 1;
                let location_attribute = match split_external_target(target).1 {
                    Some(location) => format!(" location=\"{}\"", escape_xml(location)),
                    None => String::new(),
                };
                let tooltip_attribute = get_tooltip_attribute(tooltip);
                hyperlinks_str.push(format!(
                    "<hyperlink ref=\"{cell_name}\" r:id=\"rId{rel_id}\"{location_attribute}{tooltip_attribute}/>"
                ));
            }
            Link::Internal { location, tooltip } => {
                let location = escape_xml(location);
                let tooltip_attribute = get_tooltip_attribute(tooltip);
                hyperlinks_str.push(format!(
                    "<hyperlink ref=\"{cell_name}\" location=\"{location}\"{tooltip_attribute}/>"
                ));
            }
        }
    }
    format!("<hyperlinks>{}</hyperlinks>", hyperlinks_str.join(""))
}

fn get_tooltip_attribute(tooltip: &Option<String>) -> String {
    match tooltip {
        Some(tooltip) => format!(" tooltip=\"{}\"", escape_xml(tooltip)),
        None => String::new(),
    }
}

/// The number of external links of the worksheet: their relationships come first in its rels
/// part, `rId1` to `rId{n}`.
fn external_link_count(worksheet: &Worksheet) -> usize {
    worksheet
        .links
        .values()
        .filter(|link| matches!(link, Link::External { .. }))
        .count()
}

/// The rels part of the worksheet (`xl/worksheets/_rels/sheetN.xml.rels`) holding one
/// relationship per external link, then one per table part (`tables` are the part numbers of
/// the sheet's tables), or `None` if the worksheet has neither.
/// `pivot_tables` are the sheet's relationships to its pivot tables, numbered after its other
/// ones from the id it is given (`PivotPlan::sheet_relationships`).
pub(crate) fn get_worksheet_xml_rels(
    worksheet: &Worksheet,
    tables: std::ops::Range<usize>,
    pivot_tables: impl FnOnce(usize) -> Vec<String>,
) -> Option<String> {
    let mut relationships_str: Vec<String> = vec![];
    let mut rel_id = 0;
    for (_, link) in get_sorted_links(worksheet) {
        if let Link::External { target, .. } = link {
            rel_id += 1;
            let target = escape_xml(split_external_target(target).0);
            relationships_str.push(format!(
                "<Relationship Id=\"rId{rel_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\" Target=\"{target}\" TargetMode=\"External\"/>"
            ));
        }
    }
    for table in tables {
        rel_id += 1;
        relationships_str.push(format!(
            "<Relationship Id=\"rId{rel_id}\" Type=\"{TABLE_RELATIONSHIP}\" Target=\"../tables/table{table}.xml\"/>"
        ));
    }
    relationships_str.extend(pivot_tables(rel_id + 1));
    if relationships_str.is_empty() {
        return None;
    }
    Some(format!(
        "{XML_DECLARATION}\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{}</Relationships>",
        relationships_str.join("")
    ))
}

/// `to_formula_string` is `to_excel_array_formula_string` for the formula of an array, CSE or
/// dynamic: it keeps every implicit intersection, as nothing puts them back on import.
fn get_formula_attribute(
    sheet_name: String,
    row: i32,
    column: i32,
    parsed_formula: &Node,
    to_formula_string: fn(&Node, &CellReferenceRC) -> String,
) -> String {
    let cell_ref = CellReferenceRC {
        sheet: sheet_name,
        row,
        column,
    };
    let formula = &to_formula_string(parsed_formula, &cell_ref);
    escape_xml(formula).to_string()
}

/// Whether reading the stored text of `node` as a legacy formula would change it: a plain `<f>` is
/// read by Excel and by the import with [`add_implicit_intersection`], which can add an `@` the
/// formula does not have (`=SUM(B1:B6+0)` would become `=SUM(@B1:B6+0)`). Such a formula is
/// saved as a one-cell dynamic array instead, as Excel saves it (T-1552).
///
/// The text a plain `<f>` holds ([`remove_redundant_implicit_intersection`]) is read back and
/// compared with `node` in RC form, which does not print the `automatic` flag of an `@`.
fn legacy_reading_intersects(node: &Node) -> bool {
    let mut legacy = node.clone();
    remove_redundant_implicit_intersection(&mut legacy, true);
    add_implicit_intersection(&mut legacy, true);
    to_rc_format(&legacy) != to_rc_format(node)
}

pub(crate) fn get_worksheet_xml(
    worksheet: &Worksheet,
    parsed_formulas: &[ParsedFormula],
    dimension: &str,
    is_sheet_selected: bool,
    table_count: usize,
) -> String {
    let mut sheet_data_str: Vec<String> = vec![];
    let mut cols_str: Vec<String> = vec![];
    let mut merged_cells_str: Vec<String> = vec![];

    for col in &worksheet.cols {
        // <col min="4" max="4" width="12" customWidth="1"/>
        let min = col.min;
        let max = col.max;
        let width = col.width;
        let custom_width = i32::from(col.custom_width);
        let column_style = match col.style {
            Some(s) => format!(" style=\"{s}\""),
            None => String::new(),
        };
        let hidden = if col.hidden { " hidden=\"1\"" } else { "" };
        cols_str.push(format!(
            "<col min=\"{min}\" max=\"{max}\" width=\"{width}\" customWidth=\"{custom_width}\"{column_style}{hidden}/>"
        ));
    }

    // this is a bit of an overkill. A dictionary of the row styles by row_index
    let mut row_style_dict = HashMap::new();
    for row in &worksheet.rows {
        // {
        //     "height": 13,
        //     "r": 7,
        //     "custom_format": false,
        //     "custom_height": true,
        //     "s": 0
        //     "hidden": false,
        //   },
        row_style_dict.insert(row.r, row.clone());
    }

    // A row with no cells still carries its height / hidden flag / style, so it is written too
    // (Ported-From: ironcalc@f428b2ebbc57fcf292f3e67027b24e1c866cb7c4).
    let mut row_indices = worksheet.sheet_data.rows();
    row_indices.extend(row_style_dict.keys().copied());
    row_indices.sort_unstable();
    row_indices.dedup();

    let legacy_reading = legacy_readings(parsed_formulas);
    let table_formulas = super::data_tables::anchor_formulas(&worksheet.data_tables);
    let cells = CellWriter {
        worksheet,
        parsed_formulas,
        legacy_reading: &legacy_reading,
        table_formulas: &table_formulas,
    };
    for row_index in row_indices {
        let mut row_data_str: Vec<String> = vec![];
        for (column_index, cell) in worksheet.sheet_data.cells_in_row(row_index) {
            if let Some(xml) = cell_xml(&cells, row_index, column_index, cell) {
                row_data_str.push(xml);
            }
        }
        let row_style_str = match row_style_dict.get(&row_index) {
            Some(row_style) => {
                let hidden_str = if row_style.hidden {
                    r#" hidden="1""#
                } else {
                    ""
                };
                format!(
                    r#" s="{}" ht="{}" customHeight="{}" customFormat="{}"{}"#,
                    row_style.s,
                    row_style.height,
                    i32::from(row_style.custom_height),
                    i32::from(row_style.custom_format),
                    hidden_str,
                )
            }
            None => String::new(),
        };
        sheet_data_str.push(format!(
            "<row r=\"{row_index}\"{row_style_str}>{}</row>",
            row_data_str.join("")
        ));
    }
    let sheet_data = sheet_data_str.join("");

    for merged_cell in &worksheet.merged_cells {
        let first_column = number_to_column(merged_cell.column).unwrap_or("A".to_string());
        let last_column = number_to_column(merged_cell.last_column()).unwrap_or("A".to_string());
        merged_cells_str.push(format!(
            "<mergeCell ref=\"{first_column}{}:{last_column}{}\"/>",
            merged_cell.row,
            merged_cell.last_row()
        ));
    }
    let merged_cells_count = merged_cells_str.len();

    let cols = cols_str.join("");
    let cols = if cols.is_empty() {
        String::new()
    } else {
        format!("<cols>{cols}</cols>")
    };

    let tab_selected = if is_sheet_selected {
        " tabSelected=\"1\""
    } else {
        ""
    };

    let show_grid_lines = if !worksheet.show_grid_lines {
        " showGridLines=\"0\""
    } else {
        ""
    };

    let mut active_cell = "A1".to_string();
    let mut sqref = "A1".to_string();

    let views = &worksheet.views;
    if let Some(view) = views.get(&0) {
        let range = view.range;
        let row = view.row;
        let column = view.column;
        let column_name = number_to_column(column).unwrap_or("A".to_string());
        active_cell = format!("{column_name}{row}");

        let column_start = number_to_column(range[1]).unwrap_or("A".to_string());
        let column_end = number_to_column(range[3]).unwrap_or("A".to_string());
        if range[0] == range[2] && range[1] == range[3] {
            sqref = format!("{column_start}{}", range[0]);
        } else {
            sqref = format!("{}{}:{}{}", column_start, range[0], column_end, range[2]);
        }
    }

    let merge_cells_section = if merged_cells_count > 0 {
        format!(
            "<mergeCells count=\"{}\">{}</mergeCells>",
            merged_cells_count,
            merged_cells_str.join("")
        )
    } else {
        String::new()
    };

    let frozen_rows = worksheet.frozen_rows;
    let frozen_columns = worksheet.frozen_columns;

    let pane = if frozen_rows > 0 && frozen_columns > 0 {
        // There are both frozen rows and columns. There are four panes.
        // The first column is the first column after the last frozen column.
        let first_column = number_to_column(frozen_columns + 1).unwrap_or("A".to_string());
        // This is the top left cell of the bottom right pane.
        let top_left_cell = format!("{}{}", first_column, frozen_rows + 1);
        // The meaning of the next two is irrelevant for IronCalc.
        let top_right_active_cell = format!("{first_column}1");
        let bottom_left_active_cell = format!("A{}", frozen_rows + 1);
        // The bottom right active cell is the "true" selected cell and it does not need to reside on this pane.
        format!(
            "<pane xSplit=\"{frozen_columns}\" ySplit=\"{frozen_rows}\" topLeftCell=\"{top_left_cell}\" activePane=\"bottomRight\" state=\"frozen\"/>\
             <selection pane=\"topRight\" activeCell=\"{top_right_active_cell}\" sqref=\"{top_right_active_cell}\"/>\
             <selection pane=\"bottomLeft\" activeCell=\"{bottom_left_active_cell}\" sqref=\"{bottom_left_active_cell}\"/>\
             <selection pane=\"bottomRight\" activeCell=\"{active_cell}\" sqref=\"{sqref}\"/>",
        )
    } else if frozen_rows > 0 {
        // Only frozen rows
        let top_left_cell = format!("A{}", frozen_rows + 1);
        format!(
            "<pane ySplit=\"{frozen_rows}\" topLeftCell=\"{top_left_cell}\" activePane=\"bottomLeft\" state=\"frozen\"/>\
            <selection pane=\"bottomLeft\" activeCell=\"{active_cell}\" sqref=\"{sqref}\"/>",
        )
    } else if frozen_columns > 0 {
        let top_left_cell = format!(
            "{}1",
            number_to_column(frozen_columns + 1).unwrap_or("A".to_string())
        );
        format!(
            "<pane xSplit=\"{frozen_columns}\" topLeftCell=\"{top_left_cell}\" activePane=\"topRight\" state=\"frozen\"/>\
             <selection pane=\"topRight\" activeCell=\"{active_cell}\" sqref=\"{sqref}\"/>"
        )
    } else {
        // No frozen rows or columns
        format!(r#"<selection activeCell="{active_cell}" sqref="{sqref}"/>"#)
    };

    let (cf_sections, cf_ext_lst) =
        get_conditional_formatting_xml(&worksheet.conditional_formatting);

    let hyperlinks_section = get_hyperlinks_section(worksheet);
    let table_parts_section =
        get_table_parts_section(table_count, external_link_count(worksheet) + 1);

    format!(
        "{XML_DECLARATION}\
<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
  <dimension ref=\"{dimension}\"/>\
  <sheetViews>\
    <sheetView workbookViewId=\"0\"{show_grid_lines}{tab_selected}>\
        {pane}\
    </sheetView>\
  </sheetViews>\
  {cols}\
  <sheetData>\
  {sheet_data}\
  </sheetData>\
  {merge_cells_section}\
  {cf_sections}\
  {hyperlinks_section}\
  {table_parts_section}\
  {cf_ext_lst}\
</worksheet>"
    )
}

/// Per shared formula, whether a legacy reading of it would add an `@` (T-1552).
pub(crate) fn legacy_readings(parsed_formulas: &[ParsedFormula]) -> Vec<bool> {
    parsed_formulas
        .iter()
        .map(|formula| legacy_reading_intersects(&formula.evaluated))
        .collect()
}

/// What writing one cell needs from its sheet: one definition for the regenerating export and the
/// source-preserving patcher (SPEC-0034 D3.2).
pub(crate) struct CellWriter<'a> {
    pub(crate) worksheet: &'a Worksheet,
    pub(crate) parsed_formulas: &'a [ParsedFormula],
    /// Per shared formula, whether a legacy reading of it would add an `@` (T-1552).
    pub(crate) legacy_reading: &'a [bool],
    /// The data-table formula of each anchor cell (`data_tables::anchor_formulas`).
    pub(crate) table_formulas: &'a HashMap<(i32, i32), String>,
}

/// The `<c>` element the export writes for `cell` at (`row_index`, `column_index`), or `None` for
/// a cell it does not write (a formula index or spill range that does not resolve).
pub(crate) fn cell_xml(
    writer: &CellWriter<'_>,
    row_index: i32,
    column_index: i32,
    cell: &Cell,
) -> Option<String> {
    let CellWriter {
        worksheet,
        parsed_formulas,
        legacy_reading,
        table_formulas,
    } = *writer;
    let mut row_data_str: Vec<String> = vec![];
    let formula_attribute = |node: &Node, to_formula_string| {
        get_formula_attribute(
            worksheet.get_name(),
            row_index,
            column_index,
            node,
            to_formula_string,
        )
    };
    let column_name = number_to_column(column_index).unwrap();
    let cell_name = format!("{column_name}{row_index}");
    // The `cm` attribute and `<f>` element of a cell's (non-array) formula. One a legacy
    // reading would intersect is saved as a one-cell dynamic array, as Excel saves it
    // (calc_tests/LOOKUP_AND_REFERENCE/ROW_COLUM.xlsx sheet6 F3 `SUM(ROW(C2:C4))`, T-1552).
    let plain_formula = |f: i32| {
        let index = usize::try_from(f).ok()?;
        let node = parsed_formulas.get(index)?.written();
        Some(if *legacy_reading.get(index)? {
            let formula = formula_attribute(node, to_excel_array_formula_string);
            (
                r#" cm="1""#,
                format!(r#"<f t="array" ref="{cell_name}">{formula}</f>"#),
            )
        } else {
            let formula = formula_attribute(node, to_excel_string);
            ("", format!("<f>{formula}</f>"))
        })
    };
    match cell {
        Cell::EmptyCell { s } => {
            // they only hold the style
            let style = get_cell_style_attribute(*s);
            row_data_str.push(format!("<c r=\"{cell_name}\"{style}/>"));
        }
        Cell::BooleanCell { v, s }
        | Cell::SpillCell {
            v: SpillValue::Boolean(v),
            s,
            ..
        } => {
            // <c r="A8" t="b" s="1">
            //     <v>1</v>
            // </c>
            let b = i32::from(*v);
            let style = get_cell_style_attribute(*s);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\" t=\"b\"{style}><v>{b}</v></c>"
            ));
        }
        Cell::NumberCell { v, s }
        | Cell::SpillCell {
            v: SpillValue::Number(v),
            s,
            ..
        } => {
            // Normally the type number is left out. Example:
            // <c r="C6" s="1">
            //     <v>3</v>
            // </c>
            let style = get_cell_style_attribute(*s);
            let v = saved_number(*v);
            row_data_str.push(format!("<c r=\"{cell_name}\"{style}><v>{v}</v></c>"));
        }
        Cell::ErrorCell { ei, s }
        | Cell::SpillCell {
            v: SpillValue::Error(ei),
            s,
            ..
        } => {
            let style = get_cell_style_attribute(*s);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\" t=\"e\"{style}{vm}><v>{ei}</v></c>",
                ei = exported_error_literal(ei),
                vm = vm_attribute(ei)
            ));
        }
        Cell::SharedString { si, s } => {
            // Example:
            // <c r="A1" s="1" t="s">
            //    <v>5</v>
            // </c>
            // Cell on A1 contains a string (t="s") of style="1". The string is the 6th in the list of shared strings
            let style = get_cell_style_attribute(*s);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\" t=\"s\"{style}><v>{si}</v></c>"
            ));
        }
        Cell::CellFormula {
            v: FormulaValue::Unevaluated,
            ..
        }
        | Cell::ArrayFormula {
            v: FormulaValue::Unevaluated,
            ..
        } => {
            // TODO: We should NOT panic here.
            // Is there like an "import error" we can set in Excel?
            panic!("Model needs to be evaluated before saving!");
        }
        Cell::CellFormula {
            f,
            v: FormulaValue::Boolean(v),
            s,
        } => {
            // <c r="A4" t="b" s="3">
            //   <f>ISTEXT(A5)</f>
            //   <v>1</v>
            // </c>
            let style = get_cell_style_attribute(*s);
            let (cm, formula) = plain_formula(*f)?;
            let b = i32::from(*v);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\" t=\"b\"{style}{cm}>{formula}<v>{b}</v></c>"
            ));
        }
        Cell::CellFormula {
            f,
            v: FormulaValue::Number(v),
            s,
        } => {
            // Note again type is skipped
            // <c r="C4" s="3">
            //   <f>A5+C3</f>
            //   <v>123</v>
            // </c>
            let (cm, formula) = plain_formula(*f)?;
            let style = get_cell_style_attribute(*s);
            let v = saved_number(*v);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\"{style}{cm}>{formula}<v>{v}</v></c>"
            ));
        }
        Cell::CellFormula {
            f,
            v: FormulaValue::Text(v),
            s,
        } => {
            // <c r="C6" t="str" s="5">
            //   <f>CONCATENATE(A1, A2)</f>
            //   <v>Hello world!</v>
            // </c>
            let (cm, formula) = plain_formula(*f)?;
            let style = get_cell_style_attribute(*s);
            let escaped_v = escape_text(v);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\" t=\"str\"{style}{cm}>{formula}<v>{escaped_v}</v></c>"
            ));
        }
        Cell::CellFormula {
            f,
            v: FormulaValue::Error { ei, .. },
            s,
        } => {
            // <c r="C6" t="e" s="4">
            //   <f>A1/A3<f/>
            //   <v>#DIV/0!</v>
            // </c>
            let (cm, formula) = plain_formula(*f)?;
            let style = get_cell_style_attribute(*s);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\" t=\"e\"{style}{cm}{vm}>{formula}<v>{ei}</v></c>",
                ei = exported_error_literal(ei),
                vm = vm_attribute(ei)
            ));
        }
        Cell::SpillCell {
            v: SpillValue::Text(v),
            s,
            ..
        } => {
            // inline string
            // <c r="C6" t="str" s="5">
            //   <v>Hello world!</v>
            // </c>
            let style = get_cell_style_attribute(*s);
            let escaped_v = escape_text(v);
            row_data_str.push(format!(
                "<c r=\"{cell_name}\" t=\"str\"{style}><v>{escaped_v}</v></c>"
            ));
        }
        // The difference between dynamic and array formulas is that dynamic formulas have cm="1"
        Cell::ArrayFormula {
            f,
            v: FormulaValue::Boolean(v),
            s,
            r,
            kind,
        } => {
            // Dynamic: <c r="D1" s="3" t="b" cm="1">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>TRUE</v>
            //          </c>
            // CSE:     <c r="D1" s="3" t="b">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>TRUE</v>
            //          </c>
            let node = parsed_formulas.get(*f as usize)?;
            let formula = formula_attribute(node.written(), to_excel_array_formula_string);
            let style = get_cell_style_attribute(*s);
            let range = get_range_str(row_index, column_index, r.0, r.1)?;
            let b = i32::from(*v);
            let cm = if matches!(kind, ArrayKind::Dynamic) {
                r#" cm="1""#
            } else {
                ""
            };
            row_data_str.push(format!(
                r#"<c r="{cell_name}"{style} t="b"{cm}><f t="array" ref="{range}">{formula}</f><v>{b}</v></c>"#
            ));
        }
        Cell::ArrayFormula {
            f,
            v: FormulaValue::Number(v),
            s,
            r,
            kind,
        } => {
            // Dynamic: <c r="D1" s="3" cm="1">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>123</v>
            //          </c>
            // CSE:     <c r="D1" s="3">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>123</v>
            //          </c>
            let node = parsed_formulas.get(*f as usize)?;
            let formula = formula_attribute(node.written(), to_excel_array_formula_string);
            let style = get_cell_style_attribute(*s);
            let range = get_range_str(row_index, column_index, r.0, r.1)?;
            let cm = if matches!(kind, ArrayKind::Dynamic) {
                r#" cm="1""#
            } else {
                ""
            };
            let v = saved_number(*v);
            row_data_str.push(format!(
                r#"<c r="{cell_name}"{style}{cm}><f t="array" ref="{range}">{formula}</f><v>{v}</v></c>"#
            ));
        }
        Cell::ArrayFormula {
            f,
            v: FormulaValue::Text(v),
            s,
            r,
            kind,
        } => {
            // Dynamic: <c r="D1" s="3" t="str" cm="1">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>Hello world!</v>
            //          </c>
            // CSE:     <c r="D1" s="3" t="str">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>Hello world!</v>
            //          </c>
            let node = parsed_formulas.get(*f as usize)?;
            let formula = formula_attribute(node.written(), to_excel_array_formula_string);
            let style = get_cell_style_attribute(*s);
            let range = get_range_str(row_index, column_index, r.0, r.1)?;
            let escaped_v = escape_text(v);
            let cm = if matches!(kind, ArrayKind::Dynamic) {
                r#" cm="1""#
            } else {
                ""
            };
            row_data_str.push(format!(
                r#"<c r="{cell_name}"{style} t="str"{cm}><f t="array" ref="{range}">{formula}</f><v>{escaped_v}</v></c>"#
            ));
        }
        Cell::ArrayFormula {
            f,
            v: FormulaValue::Error { ei, .. },
            s,
            r,
            kind,
        } => {
            // Dynamic: <c r="D1" s="3" t="e" cm="1">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>#N/A!</v>
            //          </c>
            // CSE:     <c r="D1" s="3" t="e">
            //            <f t="array" ref="D1:D5">A1:A5</f>
            //            <v>#N/A!</v>
            //          </c>
            let node = parsed_formulas.get(*f as usize)?;
            let formula = formula_attribute(node.written(), to_excel_array_formula_string);
            let style = get_cell_style_attribute(*s);
            let range = get_range_str(row_index, column_index, r.0, r.1)?;
            let cm = if matches!(kind, ArrayKind::Dynamic) {
                r#" cm="1""#
            } else {
                ""
            };
            row_data_str.push(format!(
                r#"<c r="{cell_name}"{style} t="e"{cm}{vm}><f t="array" ref="{range}">{formula}</f><v>{ei}</v></c>"#,
                ei = exported_error_literal(ei),
                vm = vm_attribute(ei)
            ));
        }
    }
    if let Some(formula) = table_formulas.get(&(row_index, column_index)) {
        if let Some(last) = row_data_str.last_mut() {
            *last = super::data_tables::with_formula(last, formula);
        }
    }
    row_data_str.pop()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Subnormals of either sign are flushed (T-2211, T-4355); `-0.0` keeps its spelling.
    #[test]
    fn saved_number_flushes_subnormals_of_either_sign() {
        assert_eq!(saved_number(f64::from_bits(1)), 0.0);
        assert_eq!(saved_number(f64::MIN_POSITIVE), f64::MIN_POSITIVE);
        assert_eq!(saved_number(-f64::from_bits(1)), 0.0);
        assert_eq!(saved_number(-f64::MIN_POSITIVE), -f64::MIN_POSITIVE);
        assert!(saved_number(-0.0).is_sign_negative());
    }

    #[test]
    fn exported_error_literal_falls_back_for_kinds_excel_rejects() {
        for error in [
            Error::SPILL,
            Error::CALC,
            Error::CONNECT,
            Error::BLOCKED,
            Error::FIELD,
            Error::CIRC,
            Error::ERROR,
            Error::NIMPL,
        ] {
            assert_eq!(exported_error_literal(&error), "#VALUE!", "{error}");
        }
        assert_eq!(exported_error_literal(&Error::GETTINGDATA), "#N/A");
        assert_eq!(exported_error_literal(&Error::DIV), "#DIV/0!");
        // Excel loads a stored #BUSY! constant (T-0999), so it is written as is (SPEC-0003).
        assert_eq!(exported_error_literal(&Error::BUSY), "#BUSY!");
    }
}
