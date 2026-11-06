"""Sheet settings spec2xlsx hands straight to XlsxWriter: column widths (T-1610), formula conditional formats
and custom data validations (T-3263), and the active sheet (T-4394)."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

import xlsxwriter.utility


def set_column_widths(ws: Any, sheet: dict[str, Any]) -> None:
    """Applies a sheet's `column_widths` (column letter -> width in characters; T-1610)."""
    for column, width in sheet.get("column_widths", {}).items():
        _, col = xlsxwriter.utility.xl_cell_to_rowcol(f"{column}1")
        ws.set_column(col, col, float(width))


def add_range_formulas(ws: Any, sheet: dict[str, Any], formula_text: Callable[[str], str]) -> None:
    """Writes a sheet's `conditional_formats` (type formula) and `data_validations` (type custom; T-3263), each
    formula as `formula_text` stores it. XlsxWriter returns a negative status instead of raising when it refuses
    one; that is an error here."""
    for rule in sheet.get("conditional_formats", ()):
        options = {"type": "formula", "criteria": formula_text(rule["formula"])}
        if ws.conditional_format(rule["ref"], options) != 0:
            msg = f"conditional format on {rule['ref']} was not written"
            raise ValueError(msg)
    for rule in sheet.get("data_validations", ()):
        options = {"validate": "custom", "value": formula_text(rule["formula"])}
        if ws.data_validation(rule["ref"], options) != 0:
            msg = f"data validation on {rule['ref']} was not written"
            raise ValueError(msg)


def activate_sheet(spec: dict[str, Any], worksheets: list[Any]) -> None:
    """Makes the spec's `active_sheet` the workbook's active sheet (T-4394); the first sheet is by default."""
    if "active_sheet" not in spec:
        return
    name = spec["active_sheet"]
    names = [sheet["name"] for sheet in spec["sheets"]]
    if not isinstance(name, str) or name not in names:
        msg = f"active_sheet must name a sheet of the spec: {name!r}"
        raise ValueError(msg)
    worksheets[names.index(name)].activate()
