"""Spec table options spec2xlsx reads: the header-row flag (T-3260) and a filtered table's autoFilter (T-1036,
T-4003)."""

from __future__ import annotations

from typing import Any
from xml.sax.saxutils import escape

from xlsxwriter.utility import xl_rowcol_to_cell


def has_header_row(table: dict[str, Any]) -> bool:
    """Whether a spec table has a header row: unless it says `"header_row": false` (T-3260)."""
    return table.get("header_row", True) is not False


def filter_value(raw: Any, a1: str) -> str:
    """The text a values filter lists for a first-column cell (General format: 3 → "3")."""
    if isinstance(raw, str) and raw and not raw.startswith("="):
        return raw
    if isinstance(raw, int | float) and not isinstance(raw, bool) and float(raw).is_integer():
        return str(int(raw))
    msg = f"filtered table column cell {a1} must be literal text or a whole number, not {raw!r}"
    raise ValueError(msg)


def auto_filter(
    table: dict[str, Any], cells: dict[str, Any], hidden: set[int], bounds: tuple[int, int, int, int]
) -> str:
    """The `<autoFilter>` of a `"filter": true` table over 0-based `bounds` (header row, first column, last data
    row, last column): a values filter on the first column keeping exactly the visible data rows, under
    `"filter_column": {"col_id": 5}` (0 or past the table, T-4003), or with `"criterion": false` there a
    criterion-less `<filterColumn>`."""
    name = table["name"]
    r1, c1, last_data, c2 = bounds
    ref_abs = f"{xl_rowcol_to_cell(r1, c1)}:{xl_rowcol_to_cell(last_data, c2)}"
    form = table.get("filter_column", {})
    if not isinstance(form, dict) or set(form) - {"col_id", "criterion"}:
        msg = f"table {name}: filter_column takes only col_id and criterion, not {form!r}"
        raise ValueError(msg)
    col_id = form.get("col_id", 0)
    if not isinstance(col_id, int) or isinstance(col_id, bool) or col_id < 0:
        msg = f"table {name}: filter_column col_id must be a whole number >= 0, not {col_id!r}"
        raise ValueError(msg)
    if form.get("criterion", True) is False:
        return f'<autoFilter ref="{ref_abs}"><filterColumn colId="{col_id}"/></autoFilter>'
    if 0 < col_id <= c2 - c1:
        # The values filter lists the first column's values, so it sits on that column or past the table.
        msg = f"table {name}: a values filter's col_id must be 0 or past the table's last column"
        raise ValueError(msg)
    kept: set[str] = set()
    dropped: set[str] = set()
    for row in range(r1 + 1, last_data + 1):
        a1 = xl_rowcol_to_cell(row, c1)
        (dropped if row + 1 in hidden else kept).add(filter_value(cells.get(a1), a1))
    if kept & dropped:
        msg = f"table {name}: values {sorted(kept & dropped)} are both kept and hidden by the filter"
        raise ValueError(msg)
    values = "".join(f'<filter val="{escape(v, {chr(34): "&quot;"})}"/>' for v in sorted(kept))
    column = f'<filterColumn colId="{col_id}"><filters>{values}</filters></filterColumn>'
    return f'<autoFilter ref="{ref_abs}">{column}</autoFilter>'
