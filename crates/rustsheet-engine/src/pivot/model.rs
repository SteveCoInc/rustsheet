//! The pivot table definition model (SPEC-0032 "Model (T-4139)").
//!
//! A [`PivotTable`] holds, typed, every `pivotTableDefinition` attribute PV1 reads: the form flags
//! per pivot and per field, subtotals and their placement, grand totals, the items with `h`, `sd`,
//! `s` and `t`, page field selections, data field names, aggregates and number formats, and the
//! captions. Defaults are ECMA-376's (`sml.xsd`) as Excel reads them (MS-OI29500 §18.10.1.45,
//! .69, .73; docs/research/pivot-tables-2026-11-04/excel-docs.md §7). Everything else in the part
//! is kept verbatim in [`PivotTable::opaque`] for export (T-4176); the importer counts it toward
//! its limits.

use bitcode::{Decode, Encode};

/// `dataField@baseItem` when absent: no base item (MS-OI29500 §18.10.1.22, §18.10.1.44).
pub const NO_BASE_ITEM: u32 = 1_048_832;

/// A pivot table on a worksheet.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PivotTable {
    /// The pivot's name.
    pub name: String,
    /// The worksheet's stable `sheet_id`, not its index (api-design.md principle 3).
    pub sheet_id: u32,
    /// `<location ref>`, 0-based.
    pub location: CellArea,
    /// The rest of `<location>`: the header and data offsets and the page-field grid.
    pub geometry: PivotGeometry,
    /// An index into [`Workbook::pivot_caches`](crate::types::Workbook::pivot_caches).
    pub cache: u32,
    /// One per cache field.
    pub fields: Vec<PivotField>,
    /// `<rowFields>`.
    pub row_fields: Vec<AxisField>,
    /// `<colFields>`.
    pub column_fields: Vec<AxisField>,
    /// `<pageFields>`, in file order.
    pub page_fields: Vec<PageField>,
    /// `<dataFields>`.
    pub data_fields: Vec<DataField>,
    /// The pivot-wide layout flags and captions.
    pub options: PivotOptions,
    /// The definition's elements and attributes the model does not read, verbatim.
    ///
    /// The export (T-4176) writes each entry back into the part as it is, unescaped and
    /// unchecked, so an entry must be XML the importer captured from a definition part (or a
    /// copy of one): well-formed, with its values already escaped. Nothing else may add one; a
    /// pivot made through the API holds none.
    pub opaque: Vec<OpaqueEntry>,
}

impl PivotTable {
    /// A pivot named `name` at `location` on the sheet `sheet_id`, over the cache at index
    /// `cache`, with no fields and the schema's options.
    #[must_use]
    pub fn new(name: String, sheet_id: u32, location: CellArea, cache: u32) -> Self {
        PivotTable {
            name,
            sheet_id,
            location,
            geometry: PivotGeometry::default(),
            cache,
            fields: Vec::new(),
            row_fields: Vec::new(),
            column_fields: Vec::new(),
            page_fields: Vec::new(),
            data_fields: Vec::new(),
            options: PivotOptions::default(),
            opaque: Vec::new(),
        }
    }
}

/// The attributes of `<location>` besides `ref`: where the body's header and data start, measured
/// from the location's top-left cell, and the page fields' grid. `Default` is all zero, what the
/// importer reads for a missing or malformed attribute (the schema requires the three offsets and
/// defaults the page counts to 0). Excel's own values per layout are in docs/excel-semantics/
/// pivots.md "Pivot geometry" (probes/t4138-pivot-geometry); the layout stage computes them
/// (`pivot::layout::lines`).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct PivotGeometry {
    /// `firstHeaderRow`: the row of the column fields' header.
    pub first_header_row: u32,
    /// `firstDataRow`: the first row of values (and of the row items).
    pub first_data_row: u32,
    /// `firstDataCol`: the first column of values (past the row items' columns).
    pub first_data_column: u32,
    /// `rowPageCount`: the rows of the page-field grid above the location.
    pub row_page_count: u32,
    /// `colPageCount`: the columns of that grid.
    pub column_page_count: u32,
}

/// The pivot-wide attributes of `pivotTableDefinition` that change its layout or its text.
/// `Default` is the schema's.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PivotOptions {
    /// `compact` (default on): row fields share one column.
    pub compact: bool,
    /// `outline` (default off).
    pub outline: bool,
    /// `outlineData` (default off): the Values field in outline form.
    pub outline_data: bool,
    /// `compactData` (default on): the Values field in compact form.
    pub compact_data: bool,
    /// `rowGrandTotals` (default on): the grand-total row.
    pub row_grand_totals: bool,
    /// `colGrandTotals` (default on): the grand-total column.
    pub column_grand_totals: bool,
    /// `dataOnRows` (default off): the Values field is on rows.
    pub data_on_rows: bool,
    /// `dataPosition`: the Values field's place among its axis's fields. Absent when the file
    /// leaves it out; MS-OI29500 §18.10.1.73 reads that as 0.
    pub data_position: Option<u32>,
    /// `rowHeaderCaption` (`Row Labels` when absent).
    pub row_header_caption: Option<String>,
    /// `colHeaderCaption` (`Column Labels` when absent).
    pub column_header_caption: Option<String>,
    /// `grandTotalCaption` (`Grand Total` when absent).
    pub grand_total_caption: Option<String>,
    /// `dataCaption`: the Values field's caption (required by the schema; Excel writes
    /// `Values`).
    pub data_caption: Option<String>,
    /// `showMissing` (default on): an empty value cell shows `missingCaption`.
    pub show_missing: bool,
    /// `missingCaption`.
    pub missing_caption: Option<String>,
    /// `showError` (default off): an error value cell shows `errorCaption`.
    pub show_error: bool,
    /// `errorCaption`.
    pub error_caption: Option<String>,
    /// `pageWrap` (default 0): page fields per column (or row) before wrapping; 0 never wraps.
    pub page_wrap: u32,
    /// `pageOverThenDown` (default off): page fields fill across before down.
    pub page_over_then_down: bool,
}

impl Default for PivotOptions {
    fn default() -> Self {
        PivotOptions {
            compact: true,
            outline: false,
            outline_data: false,
            compact_data: true,
            row_grand_totals: true,
            column_grand_totals: true,
            data_on_rows: false,
            data_position: None,
            row_header_caption: None,
            column_header_caption: None,
            grand_total_caption: None,
            data_caption: None,
            show_missing: true,
            missing_caption: None,
            show_error: false,
            error_caption: None,
            page_wrap: 0,
            page_over_then_down: false,
        }
    }
}

/// How a pivot shows one cache field.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PivotField {
    /// The axis the field is on, if any.
    pub axis: Option<Axis>,
    /// A bitset over the cache field's `items`: bit `i` set means item `i` is hidden. It is the
    /// one record of `item@h` for an item that names a cache item ([`PivotField::item_hidden`]).
    pub hidden: Vec<u64>,
    /// `showAll`: whether the pivot shows the field's items that no record holds. ECMA-376's
    /// default is on; Excel writes `showAll="0"`. GETPIVOTDATA on such an item of an ungrouped
    /// field is 0 when on and `#REF!` when off (probes/t4248-getpivotdata-recordless-item).
    pub show_all: bool,
    /// `compact` (default on).
    pub compact: bool,
    /// `outline` (default on).
    pub outline: bool,
    /// `subtotalTop` (default on): subtotals above the items in compact and outline form.
    pub subtotal_top: bool,
    /// `defaultSubtotal` (default on): the automatic subtotal, by each data field's function.
    pub default_subtotal: bool,
    /// The custom subtotals (`sumSubtotal`, `countASubtotal`, ...), each once, in the schema's
    /// attribute order. `countASubtotal` is read as [`Aggregate::Count`] and `countSubtotal` as
    /// [`Aggregate::CountNums`], what Excel computes (see [`ItemType::Function`]).
    pub custom_subtotals: Vec<Aggregate>,
    /// `sortType` (default manual).
    pub sort_type: SortType,
    /// `insertBlankRow` (default off).
    pub insert_blank_row: bool,
    /// `numFmtId` of the item labels; MS-OI29500 §18.10.1.69 reads absent as 0.
    pub number_format: Option<u32>,
    /// The x14 extension's `fillDownLabels` (default off; "Repeat Item Labels"): the field's
    /// item label shows on every line of its children, not on its first line only. It is read
    /// from the field's kept `extLst`, which stays in [`PivotTable::opaque`] and is what the
    /// export writes (docs/excel-semantics/pivots.md "Repeat item labels",
    /// probes/t4698-pivot-repeat-labels).
    pub fill_down_labels: bool,
    /// `<items>`, in file order: the order the field shows its items in.
    pub items: Vec<PivotItem>,
}

impl Default for PivotField {
    fn default() -> Self {
        PivotField::new(None, 0)
    }
}

impl PivotField {
    /// A field on `axis` with a hidden bitset sized for `item_count` items, none hidden, no
    /// listed items and the schema's defaults (`showAll` on).
    #[must_use]
    pub fn new(axis: Option<Axis>, item_count: usize) -> Self {
        PivotField {
            axis,
            hidden: vec![0; item_count.div_ceil(64)],
            show_all: true,
            compact: true,
            outline: true,
            subtotal_top: true,
            default_subtotal: true,
            custom_subtotals: Vec::new(),
            sort_type: SortType::Manual,
            insert_blank_row: false,
            number_format: None,
            fill_down_labels: false,
            items: Vec::new(),
        }
    }

    /// Hides item `item`; an item past the bitset's size (sized at import) is ignored.
    pub fn hide(&mut self, item: u32) {
        if let Some(word) = self.hidden.get_mut(item as usize / 64) {
            *word |= 1 << (item % 64);
        }
    }

    /// Whether item `item` is hidden; an item past the bitset reads as not hidden.
    #[must_use]
    pub fn is_hidden(&self, item: u32) -> bool {
        self.hidden
            .get(item as usize / 64)
            .is_some_and(|word| word & (1 << (item % 64)) != 0)
    }

    /// `item@h` of a listed item: whether the cache item it names is hidden. An item that names
    /// no cache item (a subtotal or blank line) is never hidden.
    #[must_use]
    pub fn item_hidden(&self, item: &PivotItem) -> bool {
        item.kind == ItemType::Data && item.index.is_some_and(|x| self.is_hidden(x))
    }
}

/// One `<item>` of a pivot field's `<items>`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct PivotItem {
    /// `t` (default data): a cache item, or the subtotal, grand-total or blank line.
    pub kind: ItemType,
    /// `x`: the cache item, for a data item.
    pub index: Option<u32>,
    /// `sd` (default on): the item is expanded (MS-OI29500 §18.10.1.45).
    pub expanded: bool,
    /// `s` (default off): a manual filter applies to the item (MS-OI29500 §18.10.1.45 reads it
    /// so, not as "has a string value").
    pub filtered: bool,
}

impl PivotItem {
    /// A data item for cache item `index`, expanded and not filtered.
    #[must_use]
    pub fn data(index: u32) -> Self {
        PivotItem {
            kind: ItemType::Data,
            index: Some(index),
            expanded: true,
            filtered: false,
        }
    }
}

/// `ST_ItemType`: what an `<item>` (or an axis line) stands for.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemType {
    /// `data`: a cache item.
    Data,
    /// `default`: the automatic subtotal, by each data field's own function.
    Default,
    /// A custom subtotal line (`sum`, `countA`, `avg`, ..., `varP`), by its function. `countA` is
    /// [`Aggregate::Count`] (COUNTA) and `count` [`Aggregate::CountNums`], and Excel labels both
    /// `Count` (probes/t4622-pivot-count-subtotals; docs/excel-semantics/pivots.md "Which function
    /// the countA and count custom subtotals compute").
    Function(Aggregate),
    /// `grand`.
    Grand,
    /// `blank`.
    Blank,
}

/// `ST_FieldSortType`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum SortType {
    /// `manual`: the items' listed order.
    Manual,
    /// `ascending`.
    Ascending,
    /// `descending`.
    Descending,
}

/// A `<pageField>`: a report filter's selection.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct PageField {
    /// `fld`: the pivot field.
    pub field: u32,
    /// `item`: the selected item, an index into the field's [`PivotField::items`]; `None` is
    /// `(All)` (MS-OI29500 §18.10.1.62).
    pub item: Option<u32>,
    /// `hier`: the OLAP hierarchy; Excel writes -1 for other sources.
    pub hierarchy: Option<i32>,
}

/// An inclusive, 0-based cell rectangle on one sheet.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct CellArea {
    /// First row.
    pub top: u32,
    /// First column.
    pub left: u32,
    /// Last row.
    pub bottom: u32,
    /// Last column.
    pub right: u32,
}

/// A pivot axis.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Axis {
    /// Rows.
    Row,
    /// Columns.
    Column,
    /// The report filter (page) area.
    Page,
}

/// A `<field x>` of `<rowFields>`/`<colFields>`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AxisField {
    /// A cache field, by index.
    Field(u32),
    /// `x="-2"`: the Values field.
    Values,
}

/// A value field of a pivot. `Default` is the schema's (`subtotal="sum"`, `showDataAs="normal"`,
/// `baseField="-1"`, `baseItem="1048832"`).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct DataField {
    /// `name`: the caption (`Sum of Sales`).
    pub caption: String,
    /// `fld`: the cache field it summarizes.
    pub field: u32,
    /// `subtotal`: how it summarizes.
    pub function: Aggregate,
    /// `showDataAs`, or the x14 extension's `pivotShowAs` ([`ShowDataAs`]).
    pub show_data_as: ShowDataAs,
    /// `baseField`: the field a show-values-as calculation compares against; -1 for none.
    pub base_field: i32,
    /// `baseItem`: the item it compares against; [`NO_BASE_ITEM`] for none, 1048828 for
    /// (previous) and 1048829 for (next) (MS-OI29500 §18.10.1.22).
    pub base_item: u32,
    /// `numFmtId` of the values, which their cells show them in; absent is the application's
    /// default (MS-OI29500 §18.10.1.22 d, as docs/research/pivot-tables-2026-11-04/excel-docs.md
    /// §1 quotes it), General for a data field over numbers (pivots.md "Value cells show their
    /// data field's number format", probes/t4169-pivot-number-formats).
    pub number_format: Option<u32>,
}

impl Default for DataField {
    fn default() -> Self {
        DataField {
            caption: String::new(),
            field: 0,
            function: Aggregate::Sum,
            show_data_as: ShowDataAs::Normal,
            base_field: -1,
            base_item: NO_BASE_ITEM,
            number_format: None,
        }
    }
}

/// A data field's summary function.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Aggregate {
    /// `sum`.
    Sum,
    /// `count`.
    Count,
    /// `average`.
    Average,
    /// `max`.
    Max,
    /// `min`.
    Min,
    /// `product`.
    Product,
    /// `countNums`.
    CountNums,
    /// `stdDev`.
    StdDev,
    /// `stdDevp`.
    StdDevP,
    /// `var`.
    Var,
    /// `varp`.
    VarP,
}

/// `ST_DataConsolidateFunction`'s names of the functions, in the schema's order.
const CONSOLIDATE_NAMES: [(&str, Aggregate); 11] = [
    ("sum", Aggregate::Sum),
    ("count", Aggregate::Count),
    ("average", Aggregate::Average),
    ("max", Aggregate::Max),
    ("min", Aggregate::Min),
    ("product", Aggregate::Product),
    ("countNums", Aggregate::CountNums),
    ("stdDev", Aggregate::StdDev),
    ("stdDevp", Aggregate::StdDevP),
    ("var", Aggregate::Var),
    ("varp", Aggregate::VarP),
];

impl Aggregate {
    /// The function a `dataField@subtotal` (`ST_DataConsolidateFunction`) names.
    #[must_use]
    pub fn from_consolidate_name(name: &str) -> Option<Self> {
        let mut names = CONSOLIDATE_NAMES.iter();
        names.find(|(n, _)| *n == name).map(|(_, f)| *f)
    }

    /// The `dataField@subtotal` name of the function, what [`Aggregate::from_consolidate_name`]
    /// reads.
    #[must_use]
    pub fn consolidate_name(self) -> &'static str {
        let mut names = CONSOLIDATE_NAMES.iter();
        names.find(|(_, f)| *f == self).map_or("sum", |(n, _)| *n)
    }

    /// Excel's word for the function: a new data field's caption starts with it (`Sum of Sales`,
    /// the case builder's `_AGGREGATE_VERBS`) and a custom subtotal line ends with it, `Count`
    /// for both `countA` and `count` (probes/t4622-pivot-count-subtotals B1!A6:A7).
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Sum => "Sum",
            Self::Count | Self::CountNums => "Count",
            Self::Average => "Average",
            Self::Max => "Max",
            Self::Min => "Min",
            Self::Product => "Product",
            Self::StdDev => "StdDev",
            Self::StdDevP => "StdDevp",
            Self::Var => "Var",
            Self::VarP => "Varp",
        }
    }
}

/// `ST_ShowDataAs`, and the six Excel 2010 modes of the x14 dataField extension's `pivotShowAs`:
/// the calculation a data field shows instead of its plain values. Excel writes an x14 mode as
/// `pivotShowAs` alone, with no `showDataAs` (docs/excel-semantics/pivots.md "Show-values-as
/// modes as Excel for the web renders them"). The importer reads it from the extension, which it
/// keeps verbatim, so a writer gives `showDataAs` only the ECMA modes ([`ShowDataAs::is_x14`]).
/// The engine is `pivot::show_as` (T-4163).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShowDataAs {
    /// `normal`: the plain values.
    Normal,
    /// `difference`.
    Difference,
    /// `percent`.
    Percent,
    /// `percentDiff`.
    PercentDiff,
    /// `runTotal`.
    RunTotal,
    /// `percentOfRow`.
    PercentOfRow,
    /// `percentOfCol`.
    PercentOfColumn,
    /// `percentOfTotal`.
    PercentOfTotal,
    /// `index`.
    Index,
    // The x14 modes, appended so the encoded indices of the ECMA ones stay valid.
    /// x14 `percentOfParentRow`.
    PercentOfParentRow,
    /// x14 `percentOfParentCol`.
    PercentOfParentColumn,
    /// x14 `percentOfParent`.
    PercentOfParent,
    /// x14 `percentOfRunningTotal`.
    PercentOfRunningTotal,
    /// x14 `rankAscending`.
    RankAscending,
    /// x14 `rankDescending`.
    RankDescending,
}

/// The modes' names, the ECMA nine in `ST_ShowDataAs`'s order, then the six x14 ones.
const SHOW_DATA_AS_NAMES: [(&str, ShowDataAs); 15] = [
    ("normal", ShowDataAs::Normal),
    ("difference", ShowDataAs::Difference),
    ("percent", ShowDataAs::Percent),
    ("percentDiff", ShowDataAs::PercentDiff),
    ("runTotal", ShowDataAs::RunTotal),
    ("percentOfRow", ShowDataAs::PercentOfRow),
    ("percentOfCol", ShowDataAs::PercentOfColumn),
    ("percentOfTotal", ShowDataAs::PercentOfTotal),
    ("index", ShowDataAs::Index),
    ("percentOfParentRow", ShowDataAs::PercentOfParentRow),
    ("percentOfParentCol", ShowDataAs::PercentOfParentColumn),
    ("percentOfParent", ShowDataAs::PercentOfParent),
    ("percentOfRunningTotal", ShowDataAs::PercentOfRunningTotal),
    ("rankAscending", ShowDataAs::RankAscending),
    ("rankDescending", ShowDataAs::RankDescending),
];

impl ShowDataAs {
    /// The mode a `dataField@showDataAs` or x14 `pivotShowAs` names (an x14 one only in
    /// `pivotShowAs`: the importer checks [`ShowDataAs::is_x14`]).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let mut names = SHOW_DATA_AS_NAMES.iter();
        names.find(|(n, _)| *n == name).map(|(_, s)| *s)
    }

    /// The mode's name, what [`ShowDataAs::from_name`] reads.
    #[must_use]
    pub fn name(self) -> &'static str {
        let mut names = SHOW_DATA_AS_NAMES.iter();
        names
            .find(|(_, s)| *s == self)
            .map_or("normal", |(n, _)| *n)
    }

    /// Whether the mode is one of the six only the x14 extension's `pivotShowAs` can name.
    #[must_use]
    pub fn is_x14(self) -> bool {
        matches!(
            self,
            Self::PercentOfParentRow
                | Self::PercentOfParentColumn
                | Self::PercentOfParent
                | Self::PercentOfRunningTotal
                | Self::RankAscending
                | Self::RankDescending
        )
    }
}

/// Where in the definition an [`OpaqueEntry`] was read: the typed element it belongs to.
/// Indices count the elements of that kind in file order.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum OpaqueOwner {
    /// The `pivotTableDefinition` root.
    Definition,
    /// `<location>`.
    Location,
    /// `<pivotFields>`.
    PivotFields,
    /// The `n`-th `<pivotField>`.
    Field(u32),
    /// The `<items>` of the `n`-th `<pivotField>`.
    Items(u32),
    /// The `item`-th `<item>` of the `field`-th `<pivotField>`.
    Item {
        /// The pivot field.
        field: u32,
        /// The item's position in the field's `<items>`.
        item: u32,
    },
    /// `<rowFields>`.
    RowFields,
    /// The `n`-th `<field>` of `<rowFields>`.
    RowField(u32),
    /// `<colFields>`.
    ColumnFields,
    /// The `n`-th `<field>` of `<colFields>`.
    ColumnField(u32),
    /// `<pageFields>`.
    PageFields,
    /// The `n`-th `<pageField>`.
    PageField(u32),
    /// `<dataFields>`.
    DataFields,
    /// The `n`-th `<dataField>`.
    DataField(u32),
}

/// What an [`OpaqueEntry`] holds, as written in the file.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum OpaqueNode {
    /// The owner's attributes the model does not read, each ` name="value"` with its value
    /// still escaped, in file order. The collections' `count` attributes are not kept: export
    /// writes them from the lists.
    Attributes(String),
    /// A child element of the owner the model does not read (an unknown element, `extLst`,
    /// `rowItems`, ...), with everything inside it. Its place among the owner's typed children is
    /// not recorded: `CT_pivotTableDefinition` and its parts are sequences, so export (T-4176)
    /// writes it where its name falls in the schema's order, the invariant it relies on.
    Element(String),
}

/// One unread piece of a pivot definition, kept for round trip (SPEC-0032 "Model").
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct OpaqueEntry {
    /// The typed element it belongs to.
    pub owner: OpaqueOwner,
    /// The verbatim XML.
    pub node: OpaqueNode,
}

impl OpaqueEntry {
    /// Whether the entry is the definition's saved `rowItems` or `colItems`: the lines of the
    /// layout the file's cells show. They describe a pivot only until the engine lays it out
    /// again, which drops them ([`crate::model::Model::plan_pivot_render`] and the refresh); the
    /// export (T-4176) writes the engine's lines for a pivot without them.
    #[must_use]
    pub fn is_layout_lines(&self) -> bool {
        self.owner == OpaqueOwner::Definition
            && matches!(&self.node, OpaqueNode::Element(xml)
                if matches!(element_name(xml), "rowItems" | "colItems"))
    }

    /// The bytes the entry holds, counted toward the import limits: its text and its own size.
    #[must_use]
    pub fn cost(&self) -> usize {
        let text = match &self.node {
            OpaqueNode::Attributes(xml) | OpaqueNode::Element(xml) => xml.len(),
        };
        text.saturating_add(std::mem::size_of::<OpaqueEntry>())
    }

    /// Whether the entry may hold a cache field index, which a refresh that renumbers the fields
    /// cannot remap (T-4694; how Excel remaps them is T-4712's). An element holds one when any
    /// attribute in it is named like a field index: `field`, `fld` or a name ending in either,
    /// ignoring case and prefix. ECMA-376's are `pivotArea@field` and `reference@field` (in
    /// `formats`, `conditionalFormats`, `chartFormats` and a field's `autoSortScope`),
    /// `filter@fld`, `@mpFld` and `@iMeasureFld`, and `mp@field` (`pivotHierarchies`); an
    /// unknown attribute so named counts too, and so does anything the scan cannot read to its
    /// end (an open quote, tag, comment, CDATA section or processing instruction, or a
    /// declaration). `rowItems` and `colItems` hold none (`x@v` are item indices of the axis's
    /// n-th field, `i@i` a data field's). Kept attributes hold none either: of the attributes
    /// the schema gives the typed elements (eval/catalog/pivot-schema-nodes.txt), the ones so
    /// named are `pageField@fld` and `dataField@fld`/`@baseField`, which the importer types,
    /// and `pivotField@dataField` and `@serverField`, two flags, allowed on a field's own
    /// attributes; the name rule still catches one the schema does not have.
    #[must_use]
    pub fn holds_field_index(&self) -> bool {
        match &self.node {
            OpaqueNode::Element(xml) => names_a_field(xml, false, |_| false),
            OpaqueNode::Attributes(xml) => {
                let field = matches!(self.owner, OpaqueOwner::Field(_));
                names_a_field(xml, true, |name| {
                    field && matches!(name, "dataField" | "serverField")
                })
            }
        }
    }
}

impl OpaqueOwner {
    /// The pivot field the owner belongs to: a `<pivotField>`, its `<items>` or one of them.
    #[must_use]
    pub fn field(self) -> Option<u32> {
        match self {
            OpaqueOwner::Field(field)
            | OpaqueOwner::Items(field)
            | OpaqueOwner::Item { field, .. } => Some(field),
            _ => None,
        }
    }

    /// The owner moved to pivot field `to`, when it belongs to one ([`OpaqueOwner::field`]).
    #[must_use]
    pub fn with_field(self, to: u32) -> Self {
        match self {
            OpaqueOwner::Field(_) => OpaqueOwner::Field(to),
            OpaqueOwner::Items(_) => OpaqueOwner::Items(to),
            OpaqueOwner::Item { item, .. } => OpaqueOwner::Item { field: to, item },
            other => other,
        }
    }
}

/// The local name of verbatim element `xml`'s tag: `rowItems` for `<x:rowItems count="1">`.
#[must_use]
pub fn element_name(xml: &str) -> &str {
    let tag = xml.strip_prefix('<').unwrap_or(xml);
    let end = tag
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(tag.len());
    let name = tag.get(..end).unwrap_or_default();
    name.rsplit(':').next().unwrap_or(name)
}

/// Whether verbatim `xml` has an attribute whose local name is `field` or `fld` or ends in one,
/// ignoring case, other than one `allowed` names, or cannot be read to its end. `attributes` is
/// whether the text is a kept attribute list, which starts and ends inside a tag; an element's
/// text must end outside one. One pass, so linear in the text (directive 5): a value is skipped by
/// its quotes, a comment, CDATA section or processing instruction to its end, and text between
/// tags is not read. An unterminated one, an open quote or tag, or a declaration (`<!DOCTYPE`,
/// which the importer never keeps) is refused (`true`): a scan that fails reads as a field index.
fn names_a_field(xml: &str, attributes: bool, allowed: impl Fn(&str) -> bool) -> bool {
    let is_name =
        |b: u8| b.is_ascii_alphanumeric() || matches!(b, b':' | b'_' | b'-' | b'.') || b >= 0x80;
    let bytes = xml.as_bytes();
    let mut in_tag = attributes;
    let mut quote = None;
    // The last name read in the tag, as a byte range, and whether the previous byte was in it.
    let mut name: Option<(usize, usize)> = None;
    let mut in_name = false;
    let mut at = 0;
    while let Some(&b) = bytes.get(at) {
        at += 1;
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
            continue;
        }
        let was_in_name = std::mem::replace(&mut in_name, false);
        if !in_tag {
            if b == b'<' {
                let rest = xml.get(at..).unwrap_or_default();
                let skipped = [("!--", "-->"), ("![CDATA[", "]]>"), ("?", "?>")]
                    .into_iter()
                    .find(|(open, _)| rest.starts_with(open))
                    .map(|(open, close)| {
                        let body = rest.get(open.len()..).unwrap_or_default();
                        body.find(close).map(|end| open.len() + end + close.len())
                    });
                match skipped {
                    Some(Some(len)) => at += len,
                    // Unterminated.
                    Some(None) => return true,
                    None if rest.starts_with('!') => return true,
                    None => {
                        in_tag = true;
                        name = None;
                    }
                }
            }
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b'>' => {
                in_tag = false;
                name = None;
            }
            b'=' => {
                let local = name
                    .and_then(|(start, end)| xml.get(start..end))
                    .map(|n| n.rsplit(':').next().unwrap_or(n));
                if let Some(local) = local {
                    let lower = local.to_ascii_lowercase();
                    if (lower.ends_with("field") || lower.ends_with("fld")) && !allowed(local) {
                        return true;
                    }
                }
                name = None;
            }
            b if is_name(b) => {
                in_name = true;
                name = match name {
                    Some((start, _)) if was_in_name => Some((start, at)),
                    _ => Some((at - 1, at)),
                };
            }
            b if b.is_ascii_whitespace() => {}
            _ => name = None,
        }
    }
    quote.is_some() || in_tag != attributes
}
