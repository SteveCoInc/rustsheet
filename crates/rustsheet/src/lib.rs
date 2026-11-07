//! RustSheet public API v0 (SPEC-0001): a typed facade over the calculation engine.
//!
//! Engine types never appear in public signatures; engine `String` errors are classified into
//! the typed errors defined here.
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod edit;
mod error;
mod pivot_read;
mod pivot_view;
mod refs;
mod source;
mod spill;
mod value;
mod workbook;

#[doc(hidden)]
pub use edit::PivotParts;
pub use edit::{
    ChangeSet, Input, InputChange, PivotDateGroupBy, PivotError, PivotFunction, PivotRangeGroup,
    PivotTableInput, SheetChange, Txn, ValueChange,
};
pub use error::{EditError, LoadError, RefError, SaveError, UnsupportedKind};
#[doc(hidden)]
pub use pivot_read::PivotPlace;
pub use pivot_view::PivotView;
pub use refs::{
    parse_cell, parse_range, resolve_sheet_by_name, CellRef, RangeRef, SheetId, Target,
};
pub use rustsheet_engine::clock::{Clock, ClockError};
pub use rustsheet_engine::types::{DateSystem, Iteration};
pub use value::{Array, ArrayShapeError, ErrorKind, ParseErrorKindError, Value, XlError};
pub use workbook::{
    CellInfo, LoadOptions, Password, SaveOptions, SheetMeta, Workbook, WorkbookOptions,
};

/// Every worksheet function the engine implements, by its en-US name (`SUM`, `ERROR.TYPE`), sorted.
#[must_use]
pub fn implemented_functions() -> Vec<String> {
    rustsheet_engine::implemented_function_names()
}
