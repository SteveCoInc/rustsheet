"""Pivot tables in spec cases (T-1705)."""

import re
from typing import Any

import pytest
from spec_parts import spec_parts

from rustsheet_tools.pivot import rendered_cells
from rustsheet_tools.spec2xlsx import build_xlsx
from rustsheet_tools.spec_cli import bake_pivots

_DATA: dict[str, object] = {
    "A1": "Region",
    "B1": "Product",
    "C1": "Sales",
    "A2": "East",
    "B2": "A",
    "C2": 10,
    "A3": "West",
    "B3": "A",
    "C3": 20,
    "A4": "East",
    "B4": "B",
    "C4": 5.5,
}


def _spec(pivot: dict[str, Any], cells: dict[str, object] | None = None) -> dict[str, Any]:
    return {
        "sheets": [
            {"name": "Data", "cells": dict(_DATA)},
            {"name": "P", "cells": cells or {"H1": '=GETPIVOTDATA("Sales",$A$3)'}, "pivot_tables": [pivot]},
        ]
    }


_PIVOT = {
    "name": "Pv",
    "source": "Data!A1:C4",
    "location": "A3",
    "row": "Region",
    "column": "Product",
    "value": "Sales",
}


_PARTS = (".xml", ".rels")  # relationships too: the pivot parts are linked through them


def _cell(sheet_xml: str, a1: str) -> str:
    m = re.search(rf'<c r="{a1}"[^>]*?(?:/>|>.*?</c>)', sheet_xml)
    assert m, a1
    return m.group(0)


def test_pivot_parts_are_linked_and_byte_stable() -> None:
    spec = _spec(_PIVOT)
    assert build_xlsx(spec) == build_xlsx(spec)
    parts = spec_parts(spec, _PARTS)
    assert (
        '<pivotCaches><pivotCache cacheId="1" r:id="rIdPivotCache1"/></pivotCaches>'
        in parts["xl/workbook.xml"]
    )
    assert 'Target="pivotCache/pivotCacheDefinition1.xml"' in parts["xl/_rels/workbook.xml.rels"]
    assert 'Target="../pivotTables/pivotTable1.xml"' in parts["xl/worksheets/_rels/sheet2.xml.rels"]
    assert 'Target="pivotCacheRecords1.xml"' in parts["xl/pivotCache/_rels/pivotCacheDefinition1.xml.rels"]
    for kind in ("pivotCacheDefinition", "pivotCacheRecords", "pivotTable"):
        assert f"spreadsheetml.{kind}+xml" in parts["[Content_Types].xml"]
    definition = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert '<worksheetSource ref="A1:C4" sheet="Data"/>' in definition
    assert '<sharedItems count="2"><s v="East"/><s v="West"/></sharedItems>' in definition
    assert 'minValue="5.5" maxValue="20"' in definition
    records = parts["xl/pivotCache/pivotCacheRecords1.xml"]
    assert '<r><x v="0"/><x v="0"/><n v="10"/></r>' in records
    assert '<r><x v="0"/><x v="1"/><n v="5.5"/></r>' in records
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert '<location ref="A3:D7" firstHeaderRow="1" firstDataRow="2" firstDataCol="1"/>' in table
    assert '<dataField name="Sum of Sales" fld="2" baseField="0" baseItem="0"/>' in table


def test_pivot_cells_are_rendered_in_compact_form() -> None:
    # Layout as excel-graph rendered it (probes/getpivotdata-real-pivot, T-1705): caption, column
    # labels, items, aggregates, blank for an empty intersection, grand totals.
    sheet = spec_parts(_spec(_PIVOT), _PARTS)["xl/worksheets/sheet2.xml"]
    assert 't="s"' in _cell(sheet, "A3")
    assert "<v>15.5</v>" in _cell(sheet, "D5")  # East total: 10 + 5.5
    assert "<v>35.5</v>" in _cell(sheet, "D7")  # grand total
    assert not re.search(r'<c r="C6"', sheet)  # West has no B
    assert "<v>5.5</v>" in _cell(sheet, "C7")


def test_rows_only_average_pivot() -> None:
    pivot = {k: v for k, v in _PIVOT.items() if k != "column"} | {"function": "average", "location": "A10"}
    parts = spec_parts(_spec(pivot), _PARTS)
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert '<location ref="A10:B13" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"/>' in table
    assert 'subtotal="average"' in table
    assert '<colItems count="1"><i/></colItems>' in table
    assert "<v>11.833333333333334</v>" in _cell(parts["xl/worksheets/sheet2.xml"], "B13")


@pytest.mark.parametrize(
    ("change", "message"),
    [
        ({"row": "Sales"}, "must hold only text"),
        ({"value": "Region"}, "must hold only numbers"),
        ({"column": "Region"}, "must differ"),
        ({"function": "median"}, "is not one of"),
        ({"source": "Nope!A1:C4"}, "is not in the spec"),
        ({"source": "Data!A1:C1"}, "no data rows"),
        ({"source": "Data!A1:C5"}, "has a blank cell"),
        ({"location": "G1"}, "overlaps"),
        ({"filter": "x"}, "unknown keys"),
    ],
)
def test_malformed_pivots_are_refused(change: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=message):
        build_xlsx(_spec(_PIVOT | change, {"H1": "=1"}))


def test_rendered_cells_must_match_the_layout() -> None:
    spec = _spec(_PIVOT, {"H1": "=1"})
    baked = bake_pivots(spec)
    assert baked
    rendered = spec["sheets"][1]["pivot_tables"][0]["rendered"]
    assert rendered["A3"] == "Sum of Sales" and rendered["B4"] == "A"
    assert not bake_pivots(spec)  # already baked
    build_xlsx(spec)
    rendered["B5"] = 99
    with pytest.raises(ValueError, match="differ from its layout"):
        build_xlsx(spec)


_GROUP = {"start": 0, "end": 10, "interval": 5, "items": ["<0", "0-4", "5-9", ">10"]}
_GROUPED = {"name": "Pg", "source": "Data!B1:C4", "location": "A3", "row": "Sales", "value": "Sales"}


def _grouped_spec(group: object) -> dict[str, Any]:
    data = {"A1": "N", "B1": "Sales", "A2": 1, "B2": 2, "A3": 7.5, "B3": 4, "A4": 12, "B4": 8}
    pivot = _GROUPED | {"source": "Data!A1:B4", "row": "N", "group": group}
    return {
        "sheets": [
            {"name": "Data", "cells": data},
            {"name": "P", "cells": {"H1": "=1"}, "pivot_tables": [pivot]},
        ]
    }


def test_range_grouped_pivot() -> None:
    # T-4228: the row field groups by `<rangePr>`, its items index the group labels, and the
    # builder renders no cells (Excel lays a grouped pivot out on load).
    spec = _grouped_spec(_GROUP)
    parts = spec_parts(spec, _PARTS)
    definition = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert (
        'minValue="1" maxValue="12"/><fieldGroup base="0"><rangePr autoStart="0" autoEnd="0" startNum="0"'
        ' endNum="10" groupInterval="5"/><groupItems count="4"><s v="&lt;0"/><s v="0-4"/><s v="5-9"/>'
        '<s v="&gt;10"/></groupItems></fieldGroup></cacheField>'
    ) in definition
    assert '<r><n v="7.5"/><n v="4"/></r>' in parts["xl/pivotCache/pivotCacheRecords1.xml"]
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert '<location ref="A3:B8" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"/>' in table
    assert (
        '<pivotField axis="axisRow" showAll="0"><items count="5"><item x="0"/><item x="1"/>'
        '<item x="2"/><item x="3"/><item t="default"/></items></pivotField>'
    ) in table
    assert '<rowItems count="5"><i><x/></i><i><x v="1"/></i>' in table
    assert not re.search(r'<c r="A3"', parts["xl/worksheets/sheet2.xml"])
    assert not bake_pivots(spec)  # nothing to bake


def test_group_items_may_repeat_or_be_numbers() -> None:
    # T-4316 probes how Excel reads a label listed twice and a non-text `<groupItems>` item.
    spec = _grouped_spec(_GROUP | {"items": ["<0", "0-4", 3, "0-4", ">10"]})
    definition = spec_parts(spec, _PARTS)["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert (
        '<groupItems count="5"><s v="&lt;0"/><s v="0-4"/><n v="3"/><s v="0-4"/><s v="&gt;10"/></groupItems>'
    ) in definition


@pytest.mark.parametrize(
    ("group", "message"),
    [
        ({"start": 0, "end": 10, "interval": 5}, "exactly the keys"),
        (_GROUP | {"start": "0"}, "are numbers"),
        (_GROUP | {"interval": True}, "are numbers"),
        (_GROUP | {"start": 10}, "start < end"),
        (_GROUP | {"interval": 0}, "interval > 0"),
        (_GROUP | {"items": ["<0", ">10"]}, "three or more texts or numbers"),
        (_GROUP | {"items": ["a", True, "b"]}, "three or more texts or numbers"),
        (_GROUP | {"items": ["a", None, "b"]}, "three or more texts or numbers"),
        ("range", "exactly the keys"),
    ],
)
def test_malformed_groups_are_refused(group: object, message: str) -> None:
    with pytest.raises(ValueError, match=message):
        build_xlsx(_grouped_spec(group))


def test_grouped_pivot_refusals() -> None:
    with pytest.raises(ValueError, match="no column field"):
        build_xlsx(_spec(_PIVOT | {"group": _GROUP}))
    with pytest.raises(ValueError, match="must hold only numbers"):
        build_xlsx(_spec({k: v for k, v in _PIVOT.items() if k != "column"} | {"group": _GROUP}))
    spec = _grouped_spec(_GROUP)
    spec["sheets"][1]["pivot_tables"][0]["rendered"] = {"A3": "Row Labels"}
    with pytest.raises(ValueError, match="differ from its layout"):
        build_xlsx(spec)


_DATE_GROUP = {
    "by": "months",
    "start": 43466,
    "end": 43830.5,
    "interval": 1,
    "items": ["<", *"ABCDEFGHIJKL", ">"],
}


def test_date_grouped_pivot_with_a_blank() -> None:
    # T-4236: a date group writes groupBy with startDate/endDate, caches the row field as dates
    # (the OLE epoch, whole seconds) and a blank row-field cell as `<m/>`.
    spec = _grouped_spec(_DATE_GROUP)
    del spec["sheets"][0]["cells"]["A3"]
    spec["sheets"][0]["cells"]["A2"] = 43480.99999
    parts = spec_parts(spec, _PARTS)
    definition = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert (
        '<sharedItems containsSemiMixedTypes="0" containsNonDate="0" containsDate="1" containsString="0"'
        ' containsBlank="1" minDate="1900-01-11T00:00:00" maxDate="2019-01-15T23:59:59"/>'
        '<fieldGroup base="0"><rangePr autoStart="0" autoEnd="0" groupBy="months"'
        ' startDate="2019-01-01T00:00:00" endDate="2019-12-31T12:00:00"/><groupItems count="14">'
    ) in definition
    records = parts["xl/pivotCache/pivotCacheRecords1.xml"]
    assert '<r><d v="2019-01-15T23:59:59"/><n v="2"/></r><r><m/><n v="4"/></r>' in records


@pytest.mark.parametrize(
    ("group", "message"),
    [
        (_DATE_GROUP | {"by": "weeks"}, "with interval 1"),
        (_DATE_GROUP | {"interval": 7}, "with interval 1"),
        (_DATE_GROUP | {"start": -1}, "0 to 9999-12-31"),
    ],
)
def test_malformed_date_groups_are_refused(group: object, message: str) -> None:
    with pytest.raises(ValueError, match=message):
        build_xlsx(_grouped_spec(group))


def test_only_a_grouped_row_field_may_hold_blanks() -> None:
    spec = _grouped_spec(_GROUP)
    del spec["sheets"][0]["cells"]["B3"]
    with pytest.raises(ValueError, match="has a blank cell"):
        build_xlsx(spec)
    spec = _spec(_PIVOT)
    del spec["sheets"][0]["cells"]["A3"]
    with pytest.raises(ValueError, match="has a blank cell"):
        build_xlsx(spec)


_ROWS_ONLY = {k: v for k, v in _PIVOT.items() if k != "column"}


@pytest.mark.parametrize("show_all", [False, True])
def test_unused_row_items(show_all: bool) -> None:
    # T-4248: a shared item no record uses stays in the cache; refreshOnLoad is left off so Excel
    # keeps it, and the layout shows it (with a blank aggregate) only under showAll.
    pivot = _ROWS_ONLY | {"unused_items": ["North"], "show_all": show_all}
    spec = _spec(pivot, {"H1": "=1"})
    parts = spec_parts(spec, _PARTS)
    definition = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert '<s v="North"/>' in definition
    assert "refreshOnLoad" not in definition
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert f'<pivotField axis="axisRow" showAll="{int(show_all)}">' in table
    rendered = rendered_cells(spec, spec["sheets"][1], pivot)
    assert ("North" in rendered.values()) is show_all
    if show_all:
        assert '<location ref="A3:B7"' in table and '<rowItems count="4">' in table
    else:
        assert '<location ref="A3:B6"' in table and '<rowItems count="3">' in table


def test_pivot_without_unused_items_refreshes_on_load() -> None:
    definition = spec_parts(_spec(_ROWS_ONLY, {"H1": "=1"}), _PARTS)[
        "xl/pivotCache/pivotCacheDefinition1.xml"
    ]
    assert 'refreshOnLoad="1"' in definition


@pytest.mark.parametrize("unused", [["East"], ["east"], ["North", "North"], ["North", "NORTH"], [1]])
def test_malformed_unused_items_are_refused(unused: list[object]) -> None:
    with pytest.raises(ValueError, match="unused_items are distinct texts"):
        build_xlsx(_spec(_ROWS_ONLY | {"unused_items": unused}, {"H1": "=1"}))


def test_unused_items_on_a_grouped_field_are_refused() -> None:
    spec = _grouped_spec(_GROUP)
    spec["sheets"][1]["pivot_tables"][0]["unused_items"] = ["x"]
    with pytest.raises(ValueError, match="unused_items are distinct texts"):
        build_xlsx(spec)
