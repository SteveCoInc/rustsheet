//! A pivot's layout: in [`lines`], its layout lines and location (SPEC-0032 stage 7, T-4147),
//! in [`render`], the cells they give (T-4149), in [`records_behind`], the records behind one
//! value cell (T-4184), and in [`chart_data`], what a pivot chart plots (T-4186). GETPIVOTDATA
//! reads the `PivotResult` itself (`functions::lookup_and_reference::pivot_index`, T-4154).
//!
//! There is one layout path. A grouped spec pivot (T-4235), which has no rendered cells because
//! the case builder cannot know Excel's groups, is computed, laid out and rendered by the same
//! stages as any other pivot (the facade's `lay_out_grouped`); oracle reads of Excel's own
//! layout hold it in step (probes/t4228-pivot-range-groups, probes/t4235-pivot-range-rounding,
//! probes/t4311-pivot-showall-column-and-groups).

mod chart;
mod lines;
mod records;
mod render;

pub use chart::{chart_data, ChartCategory, ChartSeries, PivotChartData, TOTAL_SERIES};
pub use lines::{lay_out, AxisLine, PivotLayout, MAX_PIVOT_CELLS};
pub use records::records_behind;
pub(crate) use render::item_label;
pub(super) use render::item_text;
pub use render::{
    render, CellFormat, PivotCell, RenderedCell, ALL_ITEMS, BLANK_ITEM, COLUMN_LABELS, GRAND_TOTAL,
    MAX_PIVOT_TEXT_BYTES, MULTIPLE_ITEMS, ROW_LABELS, VALUES,
};
