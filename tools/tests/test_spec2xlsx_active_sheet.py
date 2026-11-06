"""The workbook's active sheet (T-4394)."""

import io
import re
import zipfile

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx

SHEETS = [{"name": n, "cells": {"A1": 1}} for n in ("First", "S2", "Last")]


def _parts(spec: dict[str, object]) -> dict[str, str]:
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith(".xml")}


def _selected(parts: dict[str, str]) -> list[str]:
    return sorted(n for n, x in parts.items() if n.startswith("xl/worksheets/") and 'tabSelected="1"' in x)


def test_active_sheet_sets_the_active_tab_and_the_only_selected_sheet() -> None:
    parts = _parts({"active_sheet": "Last", "sheets": SHEETS})
    assert re.search(r'<workbookView [^>]*activeTab="2"', parts["xl/workbook.xml"])
    assert _selected(parts) == ["xl/worksheets/sheet3.xml"]


def test_without_active_sheet_the_first_sheet_is_active() -> None:
    parts = _parts({"sheets": SHEETS})
    assert "activeTab" not in parts["xl/workbook.xml"]
    assert _selected(parts) == ["xl/worksheets/sheet1.xml"]


@pytest.mark.parametrize("name", ["Missing", 1, None])
def test_an_active_sheet_that_is_not_a_spec_sheet_is_refused(name: object) -> None:
    with pytest.raises(ValueError, match="active_sheet"):
        build_xlsx({"active_sheet": name, "sheets": SHEETS})
