//! Text cells holding lone surrogates: Python's `str` can hold them, Rust's `String` cannot, so
//! they cross the binding as their exact UTF-16 units (SPEC-0010).

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use rustsheet::{RangeRef, RefError, Value, Workbook};

/// The little-endian UTF-16 bytes of `units`, which Python decodes with `surrogatepass`.
fn utf16_le_bytes(units: &[u16]) -> Vec<u8> {
    units.iter().flat_map(|u| u.to_le_bytes()).collect()
}

/// A Python `str` holding exactly `units`, lone surrogates included (`str` can, `String` cannot).
pub(crate) fn lone_str_to_py(py: Python<'_>, units: &[u16]) -> PyResult<Py<PyAny>> {
    let bytes = PyBytes::new(py, &utf16_le_bytes(units));
    Ok(bytes
        .call_method1("decode", ("utf-16-le", "surrogatepass"))?
        .unbind())
}

/// Cheap guard: a lone surrogate always shows as U+FFFD in the lossy text.
pub(crate) fn may_hold_lone_surrogate(text: &str) -> bool {
    text.contains('\u{FFFD}')
}

/// Exact UTF-16 units of lone-surrogate cells, keyed by (row, column) offset in a range.
pub(crate) type LoneCells = HashMap<(usize, usize), std::sync::Arc<[u16]>>;

/// The exact UTF-16 units of every lone-surrogate text cell of `arr`, the values of range `r`,
/// keyed by (row, column) offset in the range.
pub(crate) fn lone_cells(
    wb: &Workbook,
    r: &str,
    arr: &rustsheet::Array,
) -> Result<LoneCells, RefError> {
    let range = wb.range_ref(r)?;
    let mut lone = HashMap::new();
    for row in 0..arr.rows() {
        for col in 0..arr.cols() {
            let Some(Value::Text(s)) = arr.get(row, col) else {
                continue;
            };
            if !may_hold_lone_surrogate(s) {
                continue;
            }
            let at = |origin: u32, offset: usize| {
                u32::try_from(offset)
                    .ok()
                    .and_then(|o| origin.checked_add(o))
            };
            let (Some(rr), Some(cc)) = (at(range.top, row), at(range.left, col)) else {
                continue;
            };
            let cell = RangeRef::new(range.sheet, rr, cc, rr, cc)?;
            if let Some(units) = wb.text_utf16(cell)? {
                lone.insert((row, col), units);
            }
        }
    }
    Ok(lone)
}

#[cfg(test)]
mod tests {
    use super::utf16_le_bytes;

    #[test]
    fn lone_units_become_little_endian_bytes() {
        assert_eq!(utf16_le_bytes(&[0x61, 0xDE00]), [0x61, 0x00, 0x00, 0xDE]);
    }
}
