//! The pivot table API of the facade (SPEC-0032 "API", T-4178) for Python: the transaction's
//! `add_pivot_table`, `set_pivot_table` and `refresh_pivot_table` ops, the dictionaries
//! `Workbook.pivot_table` and `Workbook.pivot_table_cells` return, and the records
//! `Workbook.pivot_cell_records` returns (T-4184).

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use rustsheet::{
    parse_cell, parse_range, resolve_sheet_by_name, CellRef, EditError, PivotFunction,
    PivotTableInput, PivotView, SheetId, Txn, Value, Workbook,
};

use crate::facade::{find_sheet, name_of, sheet_names, sheet_pairs, value_to_py};

/// A pivot as the Python call gives it: ranges in A1, the function by name.
pub(crate) struct PivotSpec {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) location: String,
    pub(crate) row: String,
    pub(crate) value: String,
    pub(crate) column: Option<String>,
    pub(crate) function: PivotFunction,
    pub(crate) caption: Option<String>,
}

/// A buffered pivot op of a transaction.
pub(crate) enum PivotOp {
    /// `add_pivot_table`.
    Add(PivotSpec),
    /// `set_pivot_table`: the pivot's sheet and name, and what it becomes.
    Set(String, String, PivotSpec),
    /// `refresh_pivot_table`: the pivot's sheet and name.
    Refresh(String, String),
}

/// The pivot `name` of the keywords `definition`: `source`, `location`, `row` and `value`, and
/// optionally `column`, `function` and `caption`.
pub(crate) fn spec(name: String, definition: Option<&Bound<'_, PyDict>>) -> PyResult<PivotSpec> {
    const KEYS: [&str; 7] = [
        "source", "location", "row", "value", "column", "function", "caption",
    ];
    let get = |key: &str| -> PyResult<Option<String>> {
        match definition.map(|d| d.get_item(key)).transpose()?.flatten() {
            Some(value) if !value.is_none() => value.extract().map(Some),
            _ => Ok(None),
        }
    };
    let required = |key: &str| {
        get(key)?.ok_or_else(|| PyTypeError::new_err(format!("missing keyword argument {key:?}")))
    };
    if let Some(definition) = definition {
        for key in definition.keys() {
            let key: String = key.extract()?;
            if !KEYS.contains(&key.as_str()) {
                return Err(PyTypeError::new_err(format!(
                    "unexpected keyword argument {key:?}"
                )));
            }
        }
    }
    let function = get("function")?.unwrap_or_else(|| "sum".to_string());
    let function = PivotFunction::from_name(&function).ok_or_else(|| {
        PyValueError::new_err(format!(
            "function must be \"sum\", \"count\", \"average\", \"max\" or \"min\", not {function:?}"
        ))
    })?;
    Ok(PivotSpec {
        name,
        source: required("source")?,
        location: required("location")?,
        row: required("row")?,
        value: required("value")?,
        column: get("column")?,
        function,
        caption: get("caption")?,
    })
}

/// `spec` with its ranges resolved against `sheets`.
fn input(spec: PivotSpec, sheets: &[(SheetId, String)]) -> Result<PivotTableInput, EditError> {
    let pairs = sheet_pairs(sheets);
    let by_name = |n: Option<&str>| resolve_sheet_by_name(&pairs, n);
    let source = parse_range(&spec.source, by_name)?;
    let location = parse_range(&spec.location, by_name)?;
    let mut input = PivotTableInput::new(
        spec.name,
        source,
        location,
        spec.row,
        spec.value,
        spec.function,
    );
    input.column = spec.column;
    if let Some(caption) = spec.caption {
        input.caption = caption;
    }
    Ok(input)
}

/// Applies `op` in `txn`, its sheet names resolved against `sheets`.
pub(crate) fn apply(
    txn: &mut Txn<'_>,
    op: PivotOp,
    sheets: &[(SheetId, String)],
) -> Result<(), EditError> {
    let sheet = |name: &str| find_sheet(sheets, name).map(|i| sheets[i].0);
    match op {
        PivotOp::Add(spec) => txn.add_pivot_table(&input(spec, sheets)?),
        PivotOp::Set(on, name, spec) => {
            let id = sheet(&on)?;
            txn.set_pivot_table(id, &name, &input(spec, sheets)?)
        }
        PivotOp::Refresh(on, name) => txn.refresh_pivot_table(sheet(&on)?, &name),
    }
}

/// `view` as `Workbook.pivot_table` returns it, its ranges in A1 with the names of `sheets`.
pub(crate) fn view_to_py<'py>(
    py: Python<'py>,
    view: &PivotView,
    sheets: &[(SheetId, String)],
) -> PyResult<Bound<'py, PyDict>> {
    let sheet = |id: SheetId| name_of(sheets, id).cloned().unwrap_or_default();
    let d = PyDict::new(py);
    d.set_item("name", view.name())?;
    d.set_item("sheet", sheet(view.sheet()))?;
    let location = view.location();
    d.set_item("location", location.to_a1_in(&sheet(location.sheet)))?;
    let source = view.source().map(|r| r.to_a1_in(&sheet(r.sheet)));
    d.set_item("source", source)?;
    d.set_item("row_fields", view.row_fields())?;
    d.set_item("column_fields", view.column_fields())?;
    d.set_item("data_fields", view.data_fields())?;
    Ok(d)
}

/// `cells` as `Workbook.pivot_table_cells` returns them: A1, with the names of `sheets`, to value.
pub(crate) fn cells_to_py<'py>(
    py: Python<'py>,
    cells: Vec<(CellRef, Value)>,
    sheets: &[(SheetId, String)],
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    for (cell, value) in cells {
        let sheet = name_of(sheets, cell.sheet).cloned().unwrap_or_default();
        out.set_item(cell.to_a1_in(&sheet), value_to_py(py, value)?)?;
    }
    Ok(out)
}

/// A pivot's cells, as `Workbook.pivot_table_cells` reads them.
type Cells = Vec<(CellRef, Value)>;

/// `Workbook.pivot_table_cells`'s read: the pivot's cells and the workbook's sheets.
pub(crate) fn cells(
    wb: &Workbook,
    sheet: &str,
    name: &str,
) -> Result<(Cells, Vec<(SheetId, String)>), EditError> {
    let cells = wb.pivot_table_cells(wb.sheet_id(sheet)?, name)?;
    Ok((cells, sheet_names(wb)))
}

/// `Workbook.pivot_cell_records`: `a1` on `sheet` unless it names its own sheet.
pub(crate) fn cell_records(
    wb: &Workbook,
    sheet: &str,
    name: &str,
    a1: &str,
) -> Result<Vec<u32>, EditError> {
    let id = wb.sheet_id(sheet)?;
    let cell = parse_cell(a1, |on| match on {
        None => Some(id),
        Some(on) => wb.sheet_id(on).ok(),
    })?;
    Ok(wb.pivot_cell_records(id, name, cell)?)
}
