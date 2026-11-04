"""A case's `api_names` (T-3264): workbook-scoped defined names added through the API after the
workbook loads, not stored in it (the spec's `names`).

`"api_names": [{"name": "N", "formula": "=ABS(1)"}]` at the top level is recorded by spec2xlsx in the
docProps custom properties `rustsheet.apiname.<n>` as JSON `[name, formula]`, n from 1 in spec
order. The excel-graph backend replays each with Graph `workbook/names/add` after the typed entries
and sheet ops, before recalculating; a 400 `InvalidArgument` is Excel refusing the name, which then
does not exist (decision 0179). The engine adapter replays each with `Txn::define_name`."""

from __future__ import annotations

from typing import Any

import xlsxwriter

from rustsheet_tools.custom_properties import json_properties, set_json_property

API_NAME_PROPERTY = "rustsheet.apiname"


def api_name(entry: Any) -> tuple[str, str]:
    """A spec `api_names` entry (or a recorded `[name, formula]`) as `(name, formula)`; raises
    ValueError for anything else."""
    if isinstance(entry, list) and len(entry) == 2:
        entry = {"name": entry[0], "formula": entry[1]}
    if (
        not isinstance(entry, dict)
        or set(entry) != {"name", "formula"}
        or not isinstance(entry["name"], str)
        or not entry["name"]
        or not isinstance(entry["formula"], str)
    ):
        msg = f"api name {entry!r} must be {{'name': <non-empty text>, 'formula': <text>}}"
        raise ValueError(msg)
    return entry["name"], entry["formula"]


def record_api_names(wb: xlsxwriter.Workbook, spec: dict[str, Any]) -> None:
    """Records the spec's `api_names` as custom properties for the oracle backend to replay."""
    entries = spec.get("api_names", [])
    if not isinstance(entries, list):
        msg = f"api_names must be a list: {entries!r}"
        raise ValueError(msg)
    for n, entry in enumerate(entries, start=1):
        set_json_property(wb, f"{API_NAME_PROPERTY}.{n}", list(api_name(entry)), f"api name {n}")


def api_names(xlsx: bytes) -> list[tuple[str, str]]:
    """The `(name, formula)` pairs `build_xlsx` recorded in `xlsx`, in spec order.

    Raises ValueError when the package or a property is malformed (xlsx inputs are untrusted).
    """
    return [api_name(value) for value in json_properties(xlsx, API_NAME_PROPERTY)]
