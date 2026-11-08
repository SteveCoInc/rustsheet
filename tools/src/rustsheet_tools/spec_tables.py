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
    criterion-less `<filterColumn>`, or with `"color": true` a `<colorFilter/>` (T-4558). `"criteria"` writes
    one values filter per listed column instead (`_criteria_columns`, T-4755)."""
    name = table["name"]
    r1, c1, last_data, c2 = bounds
    ref_abs = f"{xl_rowcol_to_cell(r1, c1)}:{xl_rowcol_to_cell(last_data, c2)}"
    if "criteria" in table:
        if "filter_column" in table:
            msg = f"table {name}: criteria and filter_column cannot be combined"
            raise ValueError(msg)
        column = "".join(_criteria_columns(table, cells, hidden, bounds))
        return f'<autoFilter ref="{ref_abs}">{column}</autoFilter>'
    form = table.get("filter_column", {})
    if not isinstance(form, dict) or set(form) - {"col_id", "criterion", "color", "cell_color"}:
        msg = f"table {name}: filter_column takes only col_id, criterion, color and cell_color, not {form!r}"
        raise ValueError(msg)
    col_id = form.get("col_id", 0)
    if not isinstance(col_id, int) or isinstance(col_id, bool) or col_id < 0:
        msg = f"table {name}: filter_column col_id must be a whole number >= 0, not {col_id!r}"
        raise ValueError(msg)
    if form.get("color", False) is True:
        # A colour filter without a dxfId (T-4558); `"cell_color": false` writes cellColor="0" (a font colour).
        attr = ' cellColor="0"' if form.get("cell_color", True) is False else ""
        column = f'<filterColumn colId="{col_id}"><colorFilter{attr}/></filterColumn>'
        return f'<autoFilter ref="{ref_abs}">{column}</autoFilter>'
    if form.get("criterion", True) is False:
        return f'<autoFilter ref="{ref_abs}"><filterColumn colId="{col_id}"/></autoFilter>'
    if 0 < col_id <= c2 - c1:
        # The values filter lists the first column's values, so it sits on that column or past the table.
        msg = f"table {name}: a values filter's col_id must be 0 or past the table's last column"
        raise ValueError(msg)
    rows = range(r1 + 1, last_data + 1)
    column = _values_column(name, cells, (c1, col_id), rows, {row + 1 for row in rows} & hidden)
    return f'<autoFilter ref="{ref_abs}">{column}</autoFilter>'


def _criteria_columns(
    table: dict[str, Any], cells: dict[str, Any], hidden: set[int], bounds: tuple[int, int, int, int]
) -> list[str]:
    """The `<filterColumn>`s of `"criteria": [{"column": 0, "hides": [3, 5]}, {"column": 2, "hides": [7]}]`:
    a values filter on each listed table column (0-based) that rejects exactly the 1-based sheet rows in
    its `hides`, or with `"values": ["10.00"]` one listing exactly that text (what number-formatted cells
    display, T-4973), or with `"custom": {"and": false, "conditions": [{"operator": "equal", "val": "a"}]}`
    a custom filter. Every `hides` row must be hidden; a hidden data row no criterion hides is a row hidden by
    hand, which the filters keep (T-4755)."""
    name = table["name"]
    r1, c1, last_data, c2 = bounds
    criteria = table["criteria"]
    if not isinstance(criteria, list) or not criteria:
        msg = f"table {name}: criteria must be a non-empty list, not {criteria!r}"
        raise ValueError(msg)
    data_rows = {row + 1 for row in range(r1 + 1, last_data + 1)}
    columns: list[str] = []
    seen: set[int] = set()
    for criterion in criteria:
        if (
            not isinstance(criterion, dict)
            or not {"column", "hides"} <= set(criterion)
            or set(criterion) - {"column", "hides", "values", "custom"}
            or ("values" in criterion and "custom" in criterion)
        ):
            msg = (
                f"table {name}: each criterion takes column and hides and at most one of values and "
                f"custom, not {criterion!r}"
            )
            raise ValueError(msg)
        offset, hides = criterion["column"], criterion["hides"]
        if (
            not isinstance(offset, int)
            or isinstance(offset, bool)
            or not 0 <= offset <= c2 - c1
            or offset in seen
        ):
            msg = (
                f"table {name}: criterion column {offset!r} must be a distinct column of the table (0-based)"
            )
            raise ValueError(msg)
        seen.add(offset)
        if not isinstance(hides, list) or not set(hides) <= data_rows & hidden:
            msg = f"table {name}: criterion hides {hides!r} must list hidden data rows of the table"
            raise ValueError(msg)
        if "custom" in criterion:
            columns.append(_custom_column(name, offset, criterion["custom"]))
        elif "values" in criterion:
            columns.append(_listed_column(name, offset, criterion["values"]))
        else:
            columns.append(
                _values_column(name, cells, (c1 + offset, offset), range(r1 + 1, last_data + 1), set(hides))
            )
    return columns


def _values_column(
    name: str, cells: dict[str, Any], column: tuple[int, int], rows: range, rejected: set[int]
) -> str:
    """A `<filterColumn>` under colId `column[1]` whose values filter keeps the values sheet column `column[0]`
    (0-based) holds in the 0-based `rows` and rejects those of the 1-based rows in `rejected`."""
    sheet_column, col_id = column
    kept: set[str] = set()
    dropped: set[str] = set()
    for row in rows:
        a1 = xl_rowcol_to_cell(row, sheet_column)
        (dropped if row + 1 in rejected else kept).add(filter_value(cells.get(a1), a1))
    if kept & dropped:
        msg = f"table {name}: values {sorted(kept & dropped)} are both kept and hidden by the filter"
        raise ValueError(msg)
    values = "".join(f'<filter val="{escape(v, {chr(34): "&quot;"})}"/>' for v in sorted(kept))
    return f'<filterColumn colId="{col_id}"><filters>{values}</filters></filterColumn>'


def _quote(text: str) -> str:
    return escape(text, {chr(34): "&quot;"})


def _listed_column(name: str, col_id: int, values: Any) -> str:
    """A `<filterColumn>` whose values filter lists exactly the text `values` (T-4973)."""
    if not isinstance(values, list) or not all(isinstance(v, str) for v in values):
        msg = f"table {name}: criterion values must be a list of text, not {values!r}"
        raise ValueError(msg)
    items = "".join(f'<filter val="{_quote(v)}"/>' for v in values)
    return f'<filterColumn colId="{col_id}"><filters>{items}</filters></filterColumn>'


def _custom_column(name: str, col_id: int, custom: Any) -> str:
    """A `<filterColumn>` holding `<customFilters>` of one or two conditions (T-4973)."""
    conditions = custom.get("conditions") if isinstance(custom, dict) else None
    if (
        not isinstance(custom, dict)
        or set(custom) - {"and", "conditions"}
        or not isinstance(conditions, list)
        or not 1 <= len(conditions) <= 2
        or not all(
            isinstance(c, dict) and set(c) <= {"operator", "val"} and isinstance(c.get("val"), str)
            for c in conditions
        )
    ):
        msg = f"table {name}: criterion custom takes and and one or two conditions of operator and val"
        raise ValueError(msg)
    items = "".join(
        "<customFilter"
        + (f' operator="{_quote(c["operator"])}"' if "operator" in c else "")
        + f' val="{_quote(c["val"])}"/>'
        for c in conditions
    )
    flag = ' and="1"' if custom.get("and", False) else ""
    return f'<filterColumn colId="{col_id}"><customFilters{flag}>{items}</customFilters></filterColumn>'
