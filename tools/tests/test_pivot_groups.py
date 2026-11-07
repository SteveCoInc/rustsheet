"""Discrete (manual) and date-level groups of a defined pivot (pivot_groups.py, T-4170): the parts follow the
Excel-authored caches the module docstring cites, and the builder refuses what it cannot write. The Rust mirror
(crates/rustsheet-eval/src/pivot_definition/groups.rs) has the same refusals in its tests."""

import io
import re
import zipfile
from typing import Any

import pytest

from rustsheet_tools.pivot_groups import date_labels
from rustsheet_tools.spec2xlsx import build_xlsx

_DATA: dict[str, object] = {
    "A1": "Product",
    "B1": "S",
    "C1": "Date",
    "A2": "A",
    "B2": 1,
    "C2": 43480.0,
    "A3": "B",
    "B3": 2,
    "C3": 43600.5,
    "A4": "C",
    "B4": 4,
    "C4": 43900.0,
    "A5": "a",
    "B5": 8,
    "C5": 44000.0,
}
_GROUPS = {"name": "Product2", "groups": [{"name": "Group1", "items": ["B", "C"]}]}
_DATES = {"levels": ["months", "quarters", "years"], "start": 43466, "end": 44196}


def _parts(definition: dict[str, Any]) -> dict[str, str]:
    pivot = {"name": "Pv", "source": "Data!A1:C5", "location": "A3", "definition": definition}
    spec = {
        "sheets": [
            {"name": "Data", "cells": dict(_DATA)},
            {"name": "P", "cells": {}, "pivot_tables": [pivot]},
        ]
    }
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith(".xml")}


def _discrete(**extra: Any) -> dict[str, Any]:
    return {
        "rows": ["Product2", "Product"],
        "data": [{"field": "S"}],
        "fields": {"Product": {"discrete": _GROUPS}},
    } | extra


def test_a_discrete_group_adds_a_field_as_excel_saves_it() -> None:
    parts = _parts(_discrete())
    cache = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    # The base is listed (A, B, C; `a` is A) and names the added field; the added field lists the ungrouped
    # item first, then the group, and maps each base item to its group item.
    assert (
        '<cacheField name="Product" numFmtId="0"><sharedItems count="3"><s v="A"/><s v="B"/><s v="C"/>'
        '</sharedItems><fieldGroup par="3"/></cacheField>'
    ) in cache
    assert (
        '<cacheField name="Product2" numFmtId="0" databaseField="0"><fieldGroup base="0"><discretePr count="3">'
        '<x v="0"/><x v="1"/><x v="1"/></discretePr><groupItems count="2"><s v="A"/><s v="Group1"/></groupItems>'
        "</fieldGroup></cacheField></cacheFields>"
    ) in cache
    # The records hold the database fields only.
    records = parts["xl/pivotCache/pivotCacheRecords1.xml"]
    assert records.count("<r>") == 4 and records.count("<x ") == 4
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert '<rowFields count="2"><field x="3"/><field x="0"/></rowFields>' in table
    assert '<pivotFields count="4">' in table


def test_the_added_field_lists_its_items_in_first_source_order_even_off_the_axes() -> None:
    groups = {"name": "G", "groups": [{"name": "Last", "items": ["C"]}, {"name": "First", "items": ["A"]}]}
    table = _parts(
        {"rows": ["Product"], "data": [{"field": "S"}], "fields": {"Product": {"discrete": groups}}}
    )["xl/pivotTables/pivotTable1.xml"]
    # Group items: B, Last, First. Source order A (First), B, C (Last).
    assert '<pivotField showAll="0"><items count="4"><item x="2"/><item x="0"/><item x="1"/>' in table


def test_an_added_fields_entry_sets_its_subtotals_and_hidden_items() -> None:
    table = _parts(
        _discrete(
            fields={"Product": {"discrete": _GROUPS}, "Product2": {"subtotals": ["count"], "hidden": ["A"]}}
        )
    )["xl/pivotTables/pivotTable1.xml"]
    assert (
        '<pivotField axis="axisRow" defaultSubtotal="0" countSubtotal="1" showAll="0"><items count="3">'
        '<item x="0" h="1"/><item x="1"/><item t="count"/></items></pivotField>'
    ) in table


def test_date_levels_group_in_place_and_add_a_field_per_level() -> None:
    parts = _parts(
        {
            "rows": ["Years", "Quarters", "Date"],
            "data": [{"field": "S"}],
            "fields": {"Date": {"date_group": _DATES}},
        }
    )
    cache = parts["xl/pivotCache/pivotCacheDefinition1.xml"]
    names = re.findall(r'<cacheField name="([^"]*)"', cache)
    assert names == ["Product", "S", "Date", "Quarters", "Years"]
    assert (
        '<cacheField name="Date" numFmtId="0"><sharedItems containsSemiMixedTypes="0" containsNonDate="0"'
        ' containsDate="1" containsString="0" minDate="2019-01-15T00:00:00" maxDate="2020-06-18T00:00:00"/>'
        '<fieldGroup par="4" base="2"><rangePr autoStart="0" autoEnd="0" groupBy="months"'
        ' startDate="2019-01-01T00:00:00" endDate="2020-12-31T00:00:00"/><groupItems count="14">'
        '<s v="&lt;1/1/2019"/><s v="Jan"/>'
    ) in cache
    assert '<cacheField name="Quarters" numFmtId="0" databaseField="0"><fieldGroup base="2">' in cache
    assert (
        '<groupItems count="4"><s v="&lt;1/1/2019"/><s v="2019"/><s v="2020"/><s v="&gt;12/31/2020"/>'
        "</groupItems></fieldGroup></cacheField></cacheFields>"
    ) in cache
    records = parts["xl/pivotCache/pivotCacheRecords1.xml"]
    assert '<r><n v="1"/><d v="2019-01-15T00:00:00"/></r>' in records.replace('<x v="0"/>', "")
    assert '<d v="2019-05-15T12:00:00"/>' in records
    table = parts["xl/pivotTables/pivotTable1.xml"]
    assert '<rowFields count="3"><field x="4"/><field x="3"/><field x="2"/></rowFields>' in table


def test_one_level_groups_in_place_without_par() -> None:
    cache = _parts(
        {
            "rows": ["Date"],
            "data": [{"field": "S"}],
            "fields": {"Date": {"date_group": {**_DATES, "levels": ["days"]}}},
        }
    )["xl/pivotCache/pivotCacheDefinition1.xml"]
    assert '<fieldGroup base="2"><rangePr autoStart="0" autoEnd="0" groupBy="days"' in cache
    assert '<groupItems count="368">' in cache


def test_date_labels_are_excels_layouts() -> None:
    hours = date_labels("hours", 43466, 43467)
    assert (hours[0], hours[1], hours[12], hours[13], hours[-2], hours[-1]) == (
        "<1/1/2019",
        "12 AM",
        "11 AM",
        "12 PM",
        "11 PM",
        ">1/2/2019",
    )
    days = date_labels("days", 61, 43831)
    assert len(days) == 368 and days[60] == "29-Feb" and days[-2] == "31-Dec" and days[0] == "<3/1/1900"
    assert date_labels("seconds", 43466, 43467)[1:3] == [":00", ":01"]
    assert len(date_labels("minutes", 43466, 43467)) == 62
    assert date_labels("years", 42736.5, 44196) == [
        "<1/1/2017",
        "2017",
        "2018",
        "2019",
        "2020",
        ">12/31/2020",
    ]


@pytest.mark.parametrize(
    ("fields", "message"),
    [
        ({"Product": {"discrete": {"name": "P2"}}}, "discrete is {name, groups}"),
        ({"Product": {"discrete": {**_GROUPS, "colour": 1}}}, "discrete is {name, groups}"),
        ({"Product": {"discrete": {"name": "", "groups": []}}}, "name is a non-empty text"),
        ({"Product": {"discrete": {"name": "P2", "groups": "A"}}}, "groups is a list of 1 to 256"),
        ({"Product": {"discrete": {"name": "P2", "groups": []}}}, "groups is a list of 1 to 256"),
        (
            {"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": ["A"]}] * 257}}},
            "groups is a list of 1 to 256",
        ),
        ({"Product": {"discrete": {"name": "P2", "groups": [{"name": "G"}]}}}, "group is {name, items}"),
        ({"Product": {"discrete": {"name": "P2", "groups": [["A"]]}}}, "group is {name, items}"),
        (
            {"Product": {"discrete": {"name": "P2", "groups": [{"name": 3, "items": ["A"]}]}}},
            "non-empty text",
        ),
        (
            {"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": "A"}]}}},
            "list of 1 to 256",
        ),
        (
            {"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": ["A"] * 257}]}}},
            "list of 1 to 256",
        ),
        (
            {"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": ["Z"]}]}}},
            "'Z' is not in",
        ),
        (
            {"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": ["A", "a"]}]}}},
            "'a' is in two groups",
        ),
        (
            {"Product": {"discrete": {"name": "P2", "groups": [{"name": "c", "items": ["A"]}]}}},
            "must differ from each other and from the ungrouped items",
        ),
        (
            {"Product": {"discrete": {"name": "s", "groups": [{"name": "G", "items": ["A"]}]}}},
            "group field names must differ",
        ),
        ({"Product": {"discrete": _GROUPS, "group": {"start": 0}}}, "takes one of"),
        ({"Product": {"discrete": _GROUPS}, "Product2": {"date_group": _DATES}}, "entry takes only"),
        ({"Product": {"discrete": _GROUPS}, "Product3": {}}, "no field 'Product3'"),
        ({"Date": {"date_group": {"levels": ["months"]}}}, "date_group is {levels, start, end}"),
        ({"Date": {"date_group": {**_DATES, "levels": "months"}}}, "levels lists some of"),
        ({"Date": {"date_group": {**_DATES, "levels": ["weeks"]}}}, "levels lists some of"),
        ({"Date": {"date_group": {**_DATES, "levels": []}}}, "levels lists some of"),
        ({"Date": {"date_group": {**_DATES, "levels": ["years", "months"]}}}, "finest to the coarsest"),
        ({"Date": {"date_group": {**_DATES, "levels": ["months", "months"]}}}, "finest to the coarsest"),
        ({"Date": {"date_group": {**_DATES, "start": "2019"}}}, "start and end are date serials"),
        ({"Date": {"date_group": {**_DATES, "end": True}}}, "start and end are date serials"),
        ({"Date": {"date_group": {**_DATES, "start": 60}}}, "61 <= start < end <= 2958465"),
        ({"Date": {"date_group": {**_DATES, "end": 43466}}}, "61 <= start < end <= 2958465"),
        ({"Date": {"date_group": {**_DATES, "end": 2958466}}}, "61 <= start < end <= 2958465"),
        ({"Product": {"date_group": _DATES}}, "holds date serials"),
    ],
)
def test_refusals(fields: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=re.escape(message)):
        _parts({"rows": ["Product"], "data": [{"field": "S"}], "fields": fields})


def test_combinations_the_builder_refuses() -> None:
    fields = {"Product": {"discrete": _GROUPS}}
    for definition, message in (
        ({"data": [{"field": "Product2"}], "fields": fields}, "is not a data field"),
        (
            {"data": [{"field": "S"}], "fields": fields, "calculated": [{"name": "T", "formula": "S*2"}]},
            "do not combine",
        ),
        (
            {
                "rows": ["Product"],
                "data": [{"field": "S"}],
                "fields": fields,
                "calculated_items": [{"field": "Product", "name": "AB", "formula": "A+B"}],
            },
            "do not combine",
        ),
        (
            {
                "pages": [{"field": "Date"}],
                "data": [{"field": "S"}],
                "fields": {"Date": {"date_group": _DATES}},
            },
            "date-grouped",
        ),
    ):
        with pytest.raises(ValueError, match=re.escape(message)):
            _parts(definition)


def test_an_added_fields_item_order_lists_its_items_first() -> None:
    """T-4164's `item_order` on a field a group adds: the items it lists first, then the rest in the group's own
    order (first source item)."""
    table = _parts(
        _discrete(fields={"Product": {"discrete": _GROUPS}, "Product2": {"item_order": ["Group1"]}})
    )["xl/pivotTables/pivotTable1.xml"]
    # Group items A (0), Group1 (1): Group1 first, then A.
    assert '<items count="3"><item x="1"/><item x="0"/><item t="default"/></items>' in table
    for order, message in (
        ("Group1", "item_order of 'Product2' is a list of items"),
        (["B"], "item_order item 'B' is not in field 'Product2'"),
        (["A", "a"], "item_order of 'Product2' repeats an item"),
    ):
        with pytest.raises(ValueError, match=re.escape(message)):
            _parts(_discrete(fields={"Product": {"discrete": _GROUPS}, "Product2": {"item_order": order}}))


def test_texts_fold_as_the_rust_mirror_folds_them() -> None:
    """`fold` is `str.lower`, the mirror's `to_lowercase`: `MASS` is `mass`, but `Maß` stays apart from it
    (casefold would merge them). groups_tests.rs pins the same."""
    pivot = {
        "name": "Pv",
        "source": "Data!A1:B4",
        "location": "A3",
        "definition": {"rows": ["K"], "data": [{"field": "V"}]},
    }
    cells = {"A1": "K", "B1": "V", "A2": "Maß", "B2": 1, "A3": "mass", "B3": 2, "A4": "MASS", "B4": 4}
    spec = {"sheets": [{"name": "Data", "cells": cells}, {"name": "P", "cells": {}, "pivot_tables": [pivot]}]}
    cache = (
        zipfile.ZipFile(io.BytesIO(build_xlsx(spec))).read("xl/pivotCache/pivotCacheDefinition1.xml").decode()
    )
    assert '<sharedItems count="2"><s v="Maß"/><s v="mass"/></sharedItems>' in cache


def test_a_bound_past_the_double_range_is_refused() -> None:
    with pytest.raises(ValueError, match=re.escape("start and end are date serials")):
        _parts({"data": [{"field": "S"}], "fields": {"Date": {"date_group": {**_DATES, "end": 10**400}}}})
