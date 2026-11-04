"""Case builder v2: pivots given by a `definition` (T-4122). Excel renders them on open; these tests pin the
package shape Excel for the web was seen to accept (docs/research/pivot-tables-2026-11-04/oracle-spike.md
and T-4122's runs) and the refusals."""

import io
import re
import zipfile
from typing import Any

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx

_DATA: dict[str, object] = {
    "A1": "Region",
    "B1": "Product",
    "C1": "Sales",
    "D1": "Flag",
    "A2": "East",
    "B2": "A",
    "C2": 10,
    "D2": True,
    "A3": "West",
    "B3": "A",
    "C3": 20,
    "D3": "x",
    "A4": "east",
    "B4": "B",
    "C4": 5.5,
    "D4": {"t": "e", "v": "#N/A"},
    "A5": "East",
    "B5": "C",
    "C5": 3,
}


def _spec(
    definition: dict[str, Any], *, cells: dict[str, object] | None = None, location: str = "A3"
) -> dict[str, Any]:
    pivot = {"name": "Pv", "source": "Data!A1:D5", "location": location, "definition": definition}
    return {
        "sheets": [
            {"name": "Data", "cells": dict(_DATA)},
            {"name": "P", "cells": cells or {}, "pivot_tables": [pivot]},
        ]
    }


def _parts(spec: dict[str, Any]) -> dict[str, str]:
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith((".xml", ".rels"))}


def _shared(cache: str, name: str) -> str:
    m = re.search(
        rf'<cacheField name="{name}"[^>]*>(<sharedItems[^>]*/>|<sharedItems[^>]*[^/]>.*?</sharedItems>)',
        cache,
    )
    assert m, name
    return m.group(1)


def test_parts_are_linked_and_no_cells_are_rendered() -> None:
    parts = _parts(_spec({"rows": ["Region"], "data": [{"field": "Sales"}]}))
    assert '<pivotCache cacheId="1" r:id="rIdPivotCache1"/>' in parts["xl/workbook.xml"]
    assert "pivotTable1.xml" in parts["xl/worksheets/_rels/sheet2.xml.rels"]
    assert (
        "<c " not in parts["xl/worksheets/sheet2.xml"]
    )  # Excel renders the pivot, the builder writes no cell
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert "<rowItems" not in table and "<colItems" not in table
    assert '<dataField name="Sum of Sales" fld="2" baseField="0" baseItem="0"/>' in table
    assert 'refreshOnLoad="1"' in parts["xl/pivotCache/pivotCacheDefinition1.xml"]


def test_case_variant_texts_are_one_shared_item_and_records_index_it() -> None:
    parts = _parts(_spec({"rows": ["Region"], "data": [{"field": "Sales"}]}))
    cache = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert _shared(cache, "Region") == '<sharedItems count="2"><s v="East"/><s v="West"/></sharedItems>'
    records = parts["xl/pivotCache/pivotCacheRecords1.xml"]
    assert records.count('<r><x v="0"/>') == 3  # East, east, East
    axis = re.search(
        r'<pivotField axis="axisRow"[^>]*><items count="3">', parts["xl/pivotTables/pivotTable1.xml"]
    )
    assert axis


def test_shared_item_flags_follow_excel_saved_caches() -> None:
    cache = _parts(_spec({"rows": ["Flag"], "data": [{"field": "Sales"}]}))[
        "xl/pivotCache/pivotCacheDefinition1.xml"
    ]
    # Booleans and errors count as strings; a blank makes the field semi-mixed but not `containsString="0"`.
    assert _shared(cache, "Flag").startswith(
        '<sharedItems containsBlank="1" count="4"><b v="1"/><s v="x"/><e v="#N/A"/><m/>'
    )
    sales = _shared(cache, "Sales")
    assert sales == (
        '<sharedItems containsSemiMixedTypes="0" containsString="0" containsNumber="1"'
        ' minValue="3" maxValue="20"/>'
    )
    numbers_and_blank = _parts(_spec({"rows": ["Region"], "data": [{"field": "Sales"}]}, cells=None))
    assert numbers_and_blank  # built
    mixed = {**_DATA, "D3": 7}
    spec = _spec({"rows": ["Flag"], "data": [{"field": "Sales"}]})
    spec["sheets"][0]["cells"] = mixed
    flag = _shared(_parts(spec)["xl/pivotCache/pivotCacheDefinition1.xml"], "Flag")
    assert 'containsMixedTypes="1" containsNumber="1" containsInteger="1"' in flag


def test_several_data_fields_put_values_on_the_columns_and_names_default() -> None:
    table = _parts(
        _spec(
            {
                "rows": ["Region", "Product"],
                "data": [{"field": "Sales"}, {"field": "Sales", "subtotal": "countNums"}],
            }
        )
    )["xl/pivotTables/pivotTable1.xml"]
    assert '<colFields count="1"><field x="-2"/></colFields>' in table
    assert 'name="Count of Sales" fld="2" subtotal="countNums"' in table


def test_layout_attributes_fields_items_and_raw_slots() -> None:
    filters = (
        '<filters count="1"><filter fld="1" type="count" evalOrder="-1" id="1" iMeasureFld="0">'
        '<autoFilter ref="A1"><filterColumn colId="0"><top10 val="2" filterVal="2"/></filterColumn>'
        "</autoFilter></filter></filters>"
    )
    table = _parts(
        _spec(
            {
                "table": {"outline": False, "compact": False},
                "rows": ["Region", "Product"],
                "data": [{"field": "Sales", "showDataAs": "percentOfTotal", "numFmtId": 10}],
                "fields": {
                    "Product": {
                        "attrs": {"sortType": "descending"},
                        "hidden": ["C"],
                        "subtotals": ["sum", "count"],
                    },
                    "Region": {"collapsed": ["West"], "order": ["West"]},
                },
                "xml": {"filters": filters},
            }
        )
    )["xl/pivotTables/pivotTable1.xml"]
    assert ' outline="0" outlineData="1"' not in table and ' outline="0"' in table and ' compact="0"' in table
    assert (
        '<pivotField axis="axisRow" defaultSubtotal="0" sumSubtotal="1" countSubtotal="1" showAll="0"'
        ' sortType="descending"><items count="5"><item x="0"/><item x="1"/><item x="2" h="1"/>'
        '<item t="sum"/><item t="count"/></items>'
    ) in table
    assert '<items count="3"><item x="0" sd="0"/><item x="1"/><item t="default"/></items>' in table
    assert 'showDataAs="percentOfTotal" baseField="0" baseItem="0" numFmtId="10"' in table
    assert table.index("<pivotTableStyleInfo") < table.index("<filters")


def test_calculated_fields_page_fields_and_numeric_groups() -> None:
    parts = _parts(
        _spec(
            {
                "pages": [{"field": "Region", "item": "West"}, {"field": "Flag"}],
                "rows": ["Sales"],
                "data": [{"field": "Double"}],
                "calculated": [{"name": "Double", "formula": "Sales*2"}],
                "fields": {"Sales": {"group": {"start": 0, "end": 20, "interval": 10}}},
            },
            location="A5",
        )
    )
    cache = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert '<cacheField name="Double" numFmtId="0" formula="Sales*2" databaseField="0"/>' in cache
    assert (
        '<fieldGroup base="2"><rangePr autoStart="0" autoEnd="0" startNum="0" endNum="20" groupInterval="10"/>'
        in cache
    )
    assert (
        '<groupItems count="5"><s v="&lt;0"/><s v="0-10"/><s v="10-20"/><s v="20-30"/><s v="&gt;20"/>'
        in cache
    )
    # The calculated field is not in the records; a grouped numeric field is inline.
    assert parts["xl/pivotCache/pivotCacheRecords1.xml"].startswith(
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<pivotCacheRecords'
    )
    assert '<r><x v="0"/><x v="0"/><n v="10"/><x v="0"/></r>' in parts["xl/pivotCache/pivotCacheRecords1.xml"]
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert (
        '<pageFields count="2"><pageField fld="0" item="1" hier="-1"/><pageField fld="3" hier="-1"/></pageFields>'
        in table
    )
    assert 'rowPageCount="2" colPageCount="1"' in table
    assert '<pivotField dataField="1" dragToRow="0" dragToCol="0" dragToPage="0" showAll="0"/>' in table


@pytest.mark.parametrize(
    ("definition", "message"),
    [
        ({"rows": ["Region"], "data": []}, "at least one data field"),
        ({"rows": ["Nope"], "data": [{"field": "Sales"}]}, "no field 'Nope'"),
        ({"rows": ["Region"], "columns": ["Region"], "data": [{"field": "Sales"}]}, "two axes"),
        ({"rows": ["Values"], "data": [{"field": "Sales"}]}, "two or more data fields"),
        ({"rows": ["Region"], "data": [{"field": "Sales", "subtotal": "median"}]}, "subtotal 'median'"),
        ({"rows": ["Region"], "data": [{"field": "Sales"}, {"field": "Sales"}]}, "names must differ"),
        (
            {"rows": ["Region"], "data": [{"field": "Sales"}], "table": {"cacheId": 3}},
            "may not set attribute 'cacheId'",
        ),
        (
            {"rows": ["Region"], "data": [{"field": "Sales"}], "xml": {"filters": "<!DOCTYPE x><filters/>"}},
            "declarations",
        ),
        (
            {"rows": ["Region"], "data": [{"field": "Sales"}], "xml": {"filters": "<formats/>"}},
            "one <filters> element",
        ),
        (
            {"rows": ["Region"], "data": [{"field": "Sales"}], "fields": {"Region": {"hidden": ["North"]}}},
            "'North'",
        ),
        (
            {"rows": ["Region"], "data": [{"field": "Sales"}], "fields": {"Region": {"group": {"start": 0}}}},
            "grouped field",
        ),
        (
            {"pages": [{"field": "Region", "item": "North"}], "data": [{"field": "Sales"}]},
            "page item 'North'",
        ),
        ({"rows": ["Region"], "data": [{"field": "Sales"}], "wat": 1}, "unknown keys ['wat']"),
    ],
)
def test_refusals(definition: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=re.escape(message)):
        build_xlsx(_spec(definition))


def test_page_fields_need_room_and_the_anchor_must_be_empty() -> None:
    with pytest.raises(ValueError, match="free rows above"):
        build_xlsx(_spec({"pages": [{"field": "Region"}], "data": [{"field": "Sales"}]}, location="A1"))
    with pytest.raises(ValueError, match=r"#SPILL!"):
        build_xlsx(_spec({"rows": ["Region"], "data": [{"field": "Sales"}]}, cells={"A3": 1}))
