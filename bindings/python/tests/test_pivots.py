"""Tests for the pivot table API (``Transaction.add_pivot_table``, ``set_pivot_table``,
``refresh_pivot_table``, ``Workbook.pivot_table`` and ``Workbook.pivot_cell_records``; SPEC-0032
"API", T-4178, T-4184)."""

import pytest
import rustsheet as rs

SOURCE = [
    ("Region", "Product", "Sales"),
    ("East", "A", 10),
    ("West", "B", 20),
    ("East", "B", 5),
    ("North", "A", 7),
]

BY_REGION = [
    ["Row Labels", "Sum of Sales"],
    ["East", 15.0],
    ["West", 20.0],
    ["North", 7.0],
    ["Grand Total", 42.0],
]


@pytest.fixture
def wb() -> rs.Workbook:
    wb = rs.Workbook()
    with wb.edit() as tx:
        for r, row in enumerate(SOURCE, start=1):
            for col, value in zip("ABC", row):
                tx[f"Sheet1!{col}{r}"] = value
        tx.add_pivot_table("Pv", source="Sheet1!A1:C5", location="Sheet1!E1", row="Region", value="Sales")
    return wb


def test_add_lays_out_the_pivot_and_reports_its_cells(wb):
    assert wb.values("Sheet1!E1:F5") == BY_REGION
    # The fixture's one edit wrote the source and the pivot.
    written = {c["ref"]: c["after"] for c in wb.changes["inputs"]}
    assert written["Sheet1!E1"] == "Row Labels"
    assert written["Sheet1!F5"] == "42"
    assert len(written) == 15 + 10


def test_pivot_table_reads_the_pivot_back(wb):
    view = wb.pivot_table("Sheet1", "Pv")
    assert view is not None
    assert view["name"] == "Pv"
    assert view["sheet"] == "Sheet1"
    assert view["location"] == "Sheet1!E1:F5"
    assert view["source"] == "Sheet1!A1:C5"
    assert view["row_fields"] == ["Region"]
    assert view["column_fields"] == []
    assert view["data_fields"] == ["Sum of Sales"]
    assert "cells" not in view
    cells = wb.pivot_table_cells("Sheet1", "Pv")
    assert cells["Sheet1!E2"] == "East"
    assert cells["Sheet1!F5"] == 42.0
    assert len(cells) == 10
    with pytest.raises(rs.EditError, match="Nope"):
        wb.pivot_table_cells("Sheet1", "Nope")
    assert wb.pivot_table("Sheet1", "Nope") is None
    with pytest.raises(rs.RefError):
        wb.pivot_table("Nope", "Pv")


def test_set_replaces_the_fields_and_clears_stale_cells(wb):
    with wb.edit() as tx:
        tx.set_pivot_table(
            "Sheet1",
            "Pv",
            source="Sheet1!A1:C5",
            location="Sheet1!E1",
            row="Product",
            value="Sales",
            function="count",
            caption="Count of Sales",
        )
    assert wb.values("Sheet1!E1:F5") == [
        ["Row Labels", "Count of Sales"],
        ["A", 2.0],
        ["B", 2.0],
        ["Grand Total", 4.0],
        [None, None],
    ]
    cleared = {c["ref"] for c in wb.changes["inputs"] if c["after"] is None}
    assert {"Sheet1!E5", "Sheet1!F5"} <= cleared
    view = wb.pivot_table("Sheet1", "Pv")
    assert view is not None
    assert view["location"] == "Sheet1!E1:F4"
    assert view["row_fields"] == ["Product"]


def test_set_with_a_column_field_grows_the_pivot(wb):
    with wb.edit() as tx:
        tx.set_pivot_table(
            "Sheet1",
            "Pv",
            source="Sheet1!A1:C5",
            location="Sheet1!E1",
            row="Region",
            column="Product",
            value="Sales",
        )
    view = wb.pivot_table("Sheet1", "Pv")
    assert view is not None
    assert view["column_fields"] == ["Product"]
    assert view["location"] == "Sheet1!E1:H6"
    assert wb["Sheet1!H6"] == 42.0


def test_refresh_reads_a_source_edit(wb):
    wb.set("Sheet1!C2", 100)
    assert wb["Sheet1!F2"] == 15.0
    with wb.edit() as tx:
        tx.refresh_pivot_table("Sheet1", "Pv")
    assert wb["Sheet1!F2"] == 105.0
    assert wb["Sheet1!F5"] == 132.0


def test_unknown_pivots_and_bad_definitions_raise_and_roll_back(wb):
    with pytest.raises(rs.EditError, match="Nope"):
        with wb.edit() as tx:
            tx.refresh_pivot_table("Sheet1", "Nope")
    with pytest.raises(rs.EditError, match="Nope"):
        with wb.edit() as tx:
            tx["Sheet1!A2"] = "South"
            tx.set_pivot_table(
                "Sheet1",
                "Nope",
                source="Sheet1!A1:C5",
                location="Sheet1!E1",
                row="Region",
                value="Sales",
            )
    assert wb["Sheet1!A2"] == "East"
    with pytest.raises(rs.EditError):
        with wb.edit() as tx:
            tx.add_pivot_table("Pv", source="Sheet1!A1:C5", location="Sheet1!J1", row="Region", value="Sales")
    assert wb["Sheet1!J1"] is None
    with pytest.raises(rs.EditError, match="Nope"):
        with wb.edit() as tx:
            tx.add_pivot_table(
                "Other", source="Sheet1!A1:C5", location="Sheet1!J1", row="Nope", value="Sales"
            )
    assert wb.pivot_table("Sheet1", "Other") is None
    assert wb.values("Sheet1!E1:F5") == BY_REGION


def test_bad_arguments_raise_at_once(wb):
    with wb.edit() as tx:
        with pytest.raises(ValueError, match="function"):
            tx.add_pivot_table(
                "X",
                source="Sheet1!A1:C5",
                location="Sheet1!J1",
                row="Region",
                value="Sales",
                function="median",
            )
        with pytest.raises(TypeError, match="value"):
            tx.add_pivot_table("X", source="Sheet1!A1:C5", location="Sheet1!J1", row="Region")
        with pytest.raises(TypeError, match="rows"):
            tx.add_pivot_table(
                "X",
                source="Sheet1!A1:C5",
                location="Sheet1!J1",
                row="Region",
                value="Sales",
                rows="Region",
            )
    assert wb.pivot_table("Sheet1", "X") is None


# probes/getpivotdata-real-pivot: Excel's GETPIVOTDATA over Sum of Sales by Region and Product on
# these records gives East/A 14, West/B 7, East 22, A 34 and the grand total 49 (oracle H5, H14,
# H3, H4, H1), and 0 for West/C, which no record reaches (H13).
REAL_PIVOT = [
    ("Region", "Product", "Sales"),
    ("East", "A", 10),
    ("West", "A", 20),
    ("East", "B", 5),
    ("West", "B", 7),
    ("East", "C", 3),
    ("East", "A", 4),
]


def test_pivot_cell_records_are_the_records_behind_a_value_cell():
    wb = rs.Workbook()
    with wb.edit() as tx:
        for r, row in enumerate(REAL_PIVOT, start=1):
            for col, value in zip("ABC", row):
                tx[f"Sheet1!{col}{r}"] = value
        tx.add_pivot_table(
            "Pv", source="Sheet1!A1:C7", location="Sheet1!E1", row="Region", column="Product", value="Sales"
        )
    sales = [row[2] for row in REAL_PIVOT[1:]]
    # F3 East/A, G4 West/B, I3 East, F5 A, I5 the grand total.
    for a1, excel, records in [
        ("F3", 14, [0, 5]),
        ("G4", 7, [3]),
        ("I3", 22, [0, 2, 4, 5]),
        ("Sheet1!F5", 34, [0, 1, 5]),
        ("I5", 49, [0, 1, 2, 3, 4, 5]),
    ]:
        assert wb.pivot_cell_records("Sheet1", "Pv", a1) == records
        assert sum(sales[i] for i in records) == excel
        assert wb[f"Sheet1!{a1.split('!')[-1]}"] == excel
    assert wb.pivot_cell_records("Sheet1", "Pv", "H4") == []
    # Labels, headers and cells outside the pivot are no value cells.
    for a1 in ["E3", "F2", "E1", "J3"]:
        with pytest.raises(rs.EditError, match="value cell"):
            wb.pivot_cell_records("Sheet1", "Pv", a1)
    with pytest.raises(rs.EditError, match="Nope"):
        wb.pivot_cell_records("Sheet1", "Nope", "F3")
    with pytest.raises(rs.RefError):
        wb.pivot_cell_records("Nope", "Pv", "F3")
