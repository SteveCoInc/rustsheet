"""Oracle sidecars capture Excel's rendered pivot areas (T-4123): `pivot_area` and `pivot_areas` over a
`rustsheet-eval snapshot` JSON."""

from typing import Any

from rustsheet_tools.snapshot import MAX_PIVOT_AREA_CELLS, pivot_area, pivot_areas


def _pivot(location: str, **extra: Any) -> dict[str, Any]:
    return {
        "name": "Pv",
        "sheet": "P",
        "location": location,
        "first_header_row": 1,
        "first_data_row": 1,
        "first_data_col": 1,
        "row_page_count": 0,
        "col_page_count": 0,
        "row_items": [],
        "col_items": [],
        **extra,
    }


def test_area_is_the_location_and_the_page_rows_above_it() -> None:
    assert pivot_area(_pivot("A3:B5")) == (2, 0, 4, 1)
    # Two page fields above A5 (rows 1-2, then a blank row 3, as Excel lays them out); the area is at
    # least two columns wide for the field name and its item.
    assert pivot_area(_pivot("A5:A7", row_page_count=2, col_page_count=1)) == (1, 0, 6, 1)


def test_areas_hold_every_cell_with_blanks_marked_empty() -> None:
    snap = {
        "sheets": [
            {
                "name": "P",
                "cells": {
                    "A3": {"value": {"t": "s", "v": "Row Labels"}, "formula": None, "formula_kind": None},
                    "B3": {"value": {"t": "s", "v": "Sum of Sales"}, "formula": None, "formula_kind": None},
                    "A4": {"value": {"t": "s", "v": "East"}, "formula": None, "formula_kind": None},
                    "B4": {"value": {"t": "n", "v": 14}, "formula": None, "formula_kind": None},
                    "C9": {"value": {"t": "n", "v": 1}, "formula": None, "formula_kind": None},
                },
            }
        ],
        "pivots": [_pivot("A3:B4")],
    }
    (pivot,) = pivot_areas(snap)
    assert pivot["location"] == "A3:B4"
    assert pivot["cells"] == {
        "A3": {"t": "s", "v": "Row Labels"},
        "B3": {"t": "s", "v": "Sum of Sales"},
        "A4": {"t": "s", "v": "East"},
        "B4": {"t": "n", "v": 14},
    }
    blank = pivot_areas({**snap, "pivots": [_pivot("A3:C3")]})[0]
    assert blank["cells"]["C3"] == {"t": "empty"}


def test_areas_take_the_displayed_text_of_their_own_cells() -> None:
    # T-4169: the backend's sheet texts are cropped to each pivot's area; a pivot with none has no `text`.
    snap = {"sheets": [{"name": "P", "cells": {}}], "pivots": [_pivot("A3:B4")]}
    texts = {"P": {"A3": "Row Labels", "B4": "$1.50 ", "C9": "outside"}}
    (pivot,) = pivot_areas(snap, texts)
    assert pivot["text"] == {"A3": "Row Labels", "B4": "$1.50 "}
    assert "text" not in pivot_areas(snap, {"Other": {"A3": "x"}})[0]
    assert "text" not in pivot_areas(snap)[0]


def test_no_pivots_and_oversized_areas_capture_nothing() -> None:
    assert pivot_areas({"sheets": []}) == []
    side = int(MAX_PIVOT_AREA_CELLS**0.5) + 2
    huge = _pivot(f"A1:A{side * side}")
    assert pivot_areas({"sheets": [{"name": "P", "cells": {}}], "pivots": [huge]}) == []


def test_a_malformed_location_skips_that_pivot_only() -> None:
    snap = {"sheets": [{"name": "P", "cells": {}}], "pivots": [_pivot("nonsense"), _pivot("A1:A1")]}
    (kept,) = pivot_areas(snap)
    assert kept["location"] == "A1:A1"


def test_cache_fields_keep_shared_item_types_and_flags() -> None:
    # T-4612: the sidecar records each shared item's type (`<d>` apart from `<n>`) and the
    # `<sharedItems>` flags exactly as the snapshot read them.
    fields = [
        {
            "name": "K",
            "shared_items": [{"t": "n", "v": "45000"}, {"t": "d", "v": "2023-03-15T00:00:00"}, {"t": "m"}],
            "shared_flags": {"containsDate": "1", "containsSemiMixedTypes": "0"},
        }
    ]
    snap: dict[str, Any] = {
        "sheets": [{"name": "P", "cells": {}}],
        "pivots": [_pivot("A3:A3", cache_fields=fields)],
    }
    (pivot,) = pivot_areas(snap)
    assert pivot["cache_fields"] == fields
