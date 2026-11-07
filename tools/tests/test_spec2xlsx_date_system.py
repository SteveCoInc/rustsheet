"""Workbook date system and per-cell number formats (T-1268)."""

import pytest
from spec_parts import spec_parts

from rustsheet_tools.spec2xlsx import build_xlsx


def test_date_system_1904_is_written_and_1900_is_not() -> None:
    sheets = [{"name": "S", "cells": {"A1": 1}}]
    assert 'date1904="1"' in spec_parts({"date_system": 1904, "sheets": sheets})["xl/workbook.xml"]
    assert "date1904" not in spec_parts({"date_system": 1900, "sheets": sheets})["xl/workbook.xml"]
    assert "date1904" not in spec_parts({"sheets": sheets})["xl/workbook.xml"]


@pytest.mark.parametrize("system", [1901, "1904", True])
def test_other_date_systems_are_refused(system: object) -> None:
    with pytest.raises(ValueError, match="date_system"):
        build_xlsx({"date_system": system, "sheets": [{"name": "S", "cells": {}}]})


def test_number_formats_reach_value_formula_and_blank_cells() -> None:
    parts = spec_parts(
        {
            "sheets": [
                {
                    "name": "S",
                    "cells": {"A1": -1.5, "B1": "=A1*2", "C1": "x"},
                    "number_formats": {"A1": "[h]:mm", "B1": "[h]:mm", "D4": "yyyy-mm-dd"},
                }
            ]
        }
    )
    styles = parts["xl/styles.xml"]
    assert 'formatCode="[h]:mm"' in styles
    assert 'formatCode="yyyy-mm-dd"' in styles
    sheet = parts["xl/worksheets/sheet1.xml"]
    # One style per format code: A1 and B1 share it; C1 keeps the default; D4 is a styled blank.
    assert '<c r="A1" s="1">' in sheet
    assert '<c r="B1" s="1"' in sheet
    assert '<c r="C1" t="s">' in sheet
    assert '<c r="D4" s="2"/>' in sheet


def test_an_entered_cell_cannot_have_a_number_format() -> None:
    sheet = {"name": "S", "cells": {"A1": {"entered": "(1)"}}, "number_formats": {"A1": "0"}}
    with pytest.raises(ValueError, match="entered"):
        build_xlsx({"sheets": [sheet]})


def test_an_integer_number_format_writes_that_built_in_id() -> None:
    # T-4731: a built-in id (below 164) is the cell's `numFmtId` as is, with no `numFmt` entry, so
    # Excel shows its own built-in for it (ids 27-36 and 50-81 have no code in the file).
    parts = spec_parts(
        {"sheets": [{"name": "S", "cells": {"A1": 1.25, "B1": 2}, "number_formats": {"A1": 27, "B1": 14}}]}
    )
    styles = parts["xl/styles.xml"]
    assert "<numFmts" not in styles
    assert '<xf numFmtId="27"' in styles
    assert '<xf numFmtId="14"' in styles


@pytest.mark.parametrize("fmt", [-1, 164, True, 1.5])
def test_a_number_format_id_must_be_a_built_in_id(fmt: object) -> None:
    sheet = {"name": "S", "cells": {"A1": 1}, "number_formats": {"A1": fmt}}
    with pytest.raises(ValueError, match="built-in id"):
        build_xlsx({"sheets": [sheet]})
