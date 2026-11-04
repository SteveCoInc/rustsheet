"""A case's `api_names` (T-3264): defined names added through the API after the workbook loads, not
stored in it (the spec's `names`).

`"api_names": [{"name": "N", "formula": "=ABS(1)"}]` at the top level is recorded by spec2xlsx in the
docProps custom properties `rustsheet.apiname.<n>` as JSON `[name, formula]`, n from 1 in spec
order. An entry with `"sheet": "S"` is scoped to sheet S (T-4014) and recorded as
`[name, formula, sheet]`. The excel-graph backend replays each with Graph `workbook/names/add`, or
`workbook/worksheets('S')/names/add` for a sheet-scoped one, after the typed entries and sheet ops,
before recalculating; a 400 `InvalidArgument` is Excel refusing the name, which then does not exist
(decision 0179). The engine adapter replays each with `Txn::define_name` or
`Txn::define_sheet_name`."""

from __future__ import annotations

from typing import Any, NamedTuple

import xlsxwriter

from rustsheet_tools.custom_properties import json_properties, set_json_property

API_NAME_PROPERTY = "rustsheet.apiname"


class ApiName(NamedTuple):
    """One name to define through the API: `sheet` is its sheet scope, or None for the workbook."""

    name: str
    formula: str
    sheet: str | None = None


def api_name(entry: Any) -> ApiName:
    """A spec `api_names` entry (or a recorded `[name, formula]` or `[name, formula, sheet]`) as an
    [`ApiName`]; raises ValueError for anything else."""
    if isinstance(entry, list) and len(entry) in (2, 3):
        entry = dict(zip(("name", "formula", "sheet"), entry, strict=False))
    if (
        not isinstance(entry, dict)
        or set(entry) - {"sheet"} != {"name", "formula"}
        or not isinstance(entry["name"], str)
        or not entry["name"]
        or not isinstance(entry["formula"], str)
        or ("sheet" in entry and (not isinstance(entry["sheet"], str) or not entry["sheet"]))
    ):
        msg = (
            f"api name {entry!r} must be {{'name': <non-empty text>, 'formula': <text>}}"
            " with an optional 'sheet': <non-empty text>"
        )
        raise ValueError(msg)
    return ApiName(entry["name"], entry["formula"], entry.get("sheet"))


def record_api_names(wb: xlsxwriter.Workbook, spec: dict[str, Any]) -> None:
    """Records the spec's `api_names` as custom properties for the oracle backend to replay."""
    entries = spec.get("api_names", [])
    if not isinstance(entries, list):
        msg = f"api_names must be a list: {entries!r}"
        raise ValueError(msg)
    for n, entry in enumerate(entries, start=1):
        name, formula, sheet = api_name(entry)
        recorded = [name, formula] if sheet is None else [name, formula, sheet]
        set_json_property(wb, f"{API_NAME_PROPERTY}.{n}", recorded, f"api name {n}")


def api_names(xlsx: bytes) -> list[ApiName]:
    """The names `build_xlsx` recorded in `xlsx`, in spec order.

    Raises ValueError when the package or a property is malformed (xlsx inputs are untrusted).
    """
    return [api_name(value) for value in json_properties(xlsx, API_NAME_PROPERTY)]
