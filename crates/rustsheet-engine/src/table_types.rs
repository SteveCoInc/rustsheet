//! A worksheet table's metadata (`Workbook::tables`) and the autoFilter criteria it keeps from its
//! file (`Table::filter_columns`), which the export writes back and nothing re-applies.

use bitcode::{Decode, Encode};

use crate::types::sheet_names_match;

/// A table on a worksheet (ECMA-376-1:2016 section 18.5.1.2).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Table {
    pub name: String,
    pub display_name: String,
    pub sheet_name: String,
    pub reference: String,
    pub totals_row_count: u32,
    pub header_row_count: u32,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub columns: Vec<TableColumn>,
    pub style_info: TableStyleInfo,
    pub has_filters: bool,
    /// The criteria of the table's autoFilter as the file stored them, kept so an export writes
    /// back the filter Excel read; they hide no row here (filters are not re-applied). Column
    /// inserts and deletes inside the table move them with their columns.
    pub filter_columns: Vec<FilterColumn>,
}

impl Table {
    /// Whether the table is on the sheet `sheet_name` (sheet names match ignoring case).
    pub fn is_on_sheet(&self, sheet_name: &str) -> bool {
        sheet_names_match(&self.sheet_name, sheet_name)
    }
}

/// The rows a table spans: its reference and its totals row count, which a row delete changes and
/// the delete's undo puts back (T-3634). The insert that undoes a delete of a table's bottom rows
/// is just below the table's data, where an insert does not grow a table
/// (probes/table-edits/table-insert-row-below), so the rows alone cannot restore the extent.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct TableExtent {
    /// The table's key in `Workbook::tables`.
    pub name: String,
    pub reference: String,
    pub totals_row_count: u32,
}

/// One column of a table (ECMA-376-1:2016 section 18.5.1.3).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct TableColumn {
    pub id: u32,
    pub name: String,
    pub totals_row_label: Option<String>,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub totals_row_function: Option<String>,
    /// The column's calculated-column formula, Excel's `calculatedColumnFormula`, in file form
    /// (`T[[#This Row],[Qty]]*2`, without `=`), the same whether xlsx import read it or a typed
    /// formula filled the column (T-3033). Only whether it is set is read by the engine. Set when
    /// a typed formula filled the column (T-3024); only a column that has one is re-filled by a
    /// different typed formula, since a column of identical stored formulas without it is not a
    /// calculated column (docs/excel-semantics/references.md, "Calculated columns", rule 8).
    pub calculated_column_formula: Option<String>,
}

impl Default for TableColumn {
    fn default() -> Self {
        TableColumn {
            id: 0,
            name: "Column".to_string(),
            totals_row_label: None,
            totals_row_function: None,
            data_dxf_id: None,
            header_row_dxf_id: None,
            totals_row_dxf_id: None,
            calculated_column_formula: None,
        }
    }
}

/// A table's style name and banding flags (ECMA-376-1:2016 section 18.5.1.5).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct TableStyleInfo {
    pub name: Option<String>,
    pub show_first_column: bool,
    pub show_last_column: bool,
    pub show_row_stripes: bool,
    pub show_column_stripes: bool,
}

/// One `<filterColumn>` of a table's autoFilter (ECMA-376-1:2016 section 18.3.2.7).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct FilterColumn {
    /// The 0-based column within the filter's range.
    pub col_id: u32,
    pub criterion: FilterCriterion,
}

/// The criterion of a [`FilterColumn`]. Each kind is kept as the file wrote it, after the
/// import checked it against the schema's types and enumerations, so the export writes back the
/// filter Excel read and Excel still calls the table's hidden rows filtered (T-4004). Nothing
/// re-applies a criterion.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum FilterCriterion {
    /// `<filters>`: the rows whose text is one of `values`, or whose date falls in one of
    /// `date_groups`, plus blank rows when `blank`; `calendar_type` is the file's `calendarType`.
    Values {
        values: Vec<String>,
        blank: bool,
        calendar_type: Option<String>,
        date_groups: Vec<DateGroupItem>,
    },
    /// `<customFilters>`: one or two conditions, all of which must hold when `and`.
    Custom {
        and: bool,
        conditions: Vec<CustomFilter>,
    },
    /// A criterion the import could not check (an unknown element, or a value outside its
    /// schema type): applied, but not kept, so an export writes the column with no criterion.
    Other,
    /// `<top10>`: the `val` top (or bottom) items, or percent of items when `percent`;
    /// `filter_val` is the cut the file recorded. Both numbers are kept as finite `xsd:double`
    /// text.
    Top10 {
        top: bool,
        percent: bool,
        val: String,
        filter_val: Option<String>,
    },
    /// `<dynamicFilter>`: `kind` is the file's `type` (one of `ST_DynamicFilterType`); `val` and
    /// `max_val` finite `xsd:double` text, `val_iso` and `max_val_iso` date-time text.
    Dynamic {
        kind: String,
        val: Option<String>,
        max_val: Option<String>,
        val_iso: Option<String>,
        max_val_iso: Option<String>,
    },
    /// `<colorFilter>`: the cell (or, when not `cell_color`, font) color of the differential
    /// format `dxf_id`.
    Color {
        dxf_id: Option<u32>,
        cell_color: bool,
    },
    /// `<iconFilter>`: icon `icon_id` of `icon_set` (one of `ST_IconSetType`); no `icon_id`
    /// keeps the cells with no icon.
    Icon {
        icon_set: String,
        icon_id: Option<u32>,
    },
}

impl FilterCriterion {
    /// A values filter that keeps nothing: no value, no date group and no blank. Excel reads the
    /// `<filters/>` it is written as as no criterion (excel-graph, T-4749;
    /// docs/excel-semantics/xlsx-storage.md, "Table parts in the export").
    #[must_use]
    pub fn is_empty_values(&self) -> bool {
        matches!(
            self,
            Self::Values { values, blank: false, date_groups, .. }
                if values.is_empty() && date_groups.is_empty()
        )
    }
}

/// One `<dateGroupItem>` of a values filter: the dates in `year` (and the finer fields down to
/// `grouping`, one of `ST_DateTimeGrouping`).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct DateGroupItem {
    pub grouping: String,
    pub year: u16,
    pub month: Option<u16>,
    pub day: Option<u16>,
    pub hour: Option<u16>,
    pub minute: Option<u16>,
    pub second: Option<u16>,
}

/// One `<customFilter>`: `operator` is the file's attribute (absent means `equal`) and `val`
/// the operand as text, both kept verbatim for the export only; re-applying a filter would need
/// a typed operator.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct CustomFilter {
    pub operator: Option<String>,
    pub val: String,
}
