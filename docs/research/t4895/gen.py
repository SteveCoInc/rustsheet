"""Generate the T-4895 probes: a column delete that leaves a filter criterion, with a row hidden by hand and a
number-formatted kept column (probes/t4895-*).

Table Sales on S!A1:C7 (Cat a,b,a,c,a,b; Val 1-6; Ext 10-60), values filters on Cat keeping "a" and "c" (rows 3, 7)
and on Ext rejecting 40 (row 5), as probes/t4755-two-criteria-filtered-table-*. Sheet F reads it with SUBTOTAL 3,
103, 9 and 109 over Sales[Val] and per data row. Variants:
- hand-hidden: row 4 ("a", 30), which both criteria keep, is also written hidden (control, and delete_column A);
- number-format: Ext shows as 0.00 (10.00 .. 60.00) while spec2xlsx's Ext filter lists the General text
  "10", "20", "30", "50", "60" (control, and delete_column A).
A6 (COLUMNS) and the number-format control were added after the first fill (second predictions commit).
Predictions (docs/research/t4895/predictions.tsv) are the engine's values, committed before the excel-graph fill.
Run: python3 docs/research/t4895/gen.py (writes eval/suites/probes/cases/t4895-*.case.json)
"""

import json
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval" / "suites" / "probes" / "cases"
CAT = ["a", "b", "a", "c", "a", "b"]
DELETE_A = [{"op": "delete_column", "sheet": "S", "column": "A"}]


def readers() -> dict[str, str]:
    cells = {
        "A1": "=SUBTOTAL(3,Sales[Val])",
        "A2": "=SUBTOTAL(103,Sales[Val])",
        "A3": "=SUBTOTAL(9,Sales[Val])",
        "A4": "=SUBTOTAL(109,Sales[Val])",
        "A5": "=ROWS(Sales)",
        "A6": "=COLUMNS(Sales)",
    }
    for row in range(2, 8):
        cells |= {f"B{row}": f"=SUBTOTAL(3,S!B{row})", f"C{row}": f"=SUBTOTAL(103,S!B{row})"}
    return cells


def case(
    description: str,
    hidden: list[int],
    ops: list[dict[str, str]],
    formats: dict[str, str],
    criteria: list[dict] | None = None,
) -> dict:
    data = {"A1": "Cat", "B1": "Val", "C1": "Ext"}
    for i, cat in enumerate(CAT, start=2):
        data |= {f"A{i}": cat, f"B{i}": i - 1, f"C{i}": 10 * (i - 1)}
    sheet: dict = {"name": "S", "cells": data, "hidden_rows": hidden}
    if formats:
        sheet["number_formats"] = formats
    sheet["tables"] = [
        {
            "name": "Sales",
            "ref": "A1:C7",
            "filter": True,
            "criteria": criteria or [{"column": 0, "hides": [3, 7]}, {"column": 2, "hides": [5]}],
        }
    ]
    f = readers()
    spec = {
        "format": "rustsheet-case/v1",
        "description": description + " (T-4895)",
        "tags": ["M8", "tables", "probe"],
        "sheets": [sheet, {"name": "F", "cells": f}],
        "expected": {},
        "check": [f"F!{c}" for c in f],
    }
    if ops:
        spec["sheet_ops"] = ops
    return spec


BASE = (
    "Table Sales on S!A1:C7 (Cat a,b,a,c,a,b; Val 1-6; Ext 10-60); values filters on Cat keeping a and c (rows 3, 7) "
    "and on Ext rejecting 40 (row 5). F!A1:A4 SUBTOTAL 3, 103, 9, 109 over Sales[Val], A5 ROWS(Sales), A6 COLUMNS(Sales), "
    "B2:B7 SUBTOTAL(3,S!Bn) and C2:C7 SUBTOTAL(103,S!Bn) per data row. "
)
SPECS = {
    "t4895-two-criteria-hand-hidden-control": case(
        BASE + "Row 4 (a, 30), which both criteria keep, is also written hidden. No edit.", [3, 4, 5, 7], [], {}
    ),
    "t4895-two-criteria-hand-hidden-delete-one": case(
        BASE + "Row 4 (a, 30), which both criteria keep, is also written hidden. The edit: delete_column at A "
        "deletes Cat; the Ext criterion stays.",
        [3, 4, 5, 7],
        DELETE_A,
        {},
    ),
    "t4895-two-criteria-number-format-control": case(
        BASE + "Ext is formatted 0.00 (shows 10.00 .. 60.00) while its filter lists the General text 10, 20, 30, 50, "
        "60. No edit.",
        [3, 5, 7],
        [],
        {f"C{r}": "0.00" for r in range(2, 8)},
    ),
    "t4895-two-criteria-number-format-delete-one": case(
        BASE + "Ext is formatted 0.00 (shows 10.00 .. 60.00) while its filter lists the General text 10, 20, 30, 50, "
        "60. The edit: delete_column at A deletes Cat; the Ext criterion stays.",
        [3, 5, 7],
        DELETE_A,
        {f"C{r}": "0.00" for r in range(2, 8)},
    ),
}

# T-4973: the T-4895 / T-4939 uploads as spec cases (custom filter and display-text values list).
DELETE_C = [{"op": "delete_column", "sheet": "S", "column": "C"}]
EXT = {"column": 2, "hides": [5]}
FORMAT_C = {f"C{r}": "0.00" for r in range(2, 8)}
DISPLAY = {"column": 2, "hides": [5], "values": [f"{v}.00" for v in (10, 20, 30, 50, 60)]}


def custom(a: str, c: str) -> dict:
    conditions = [{"val": a}, {"val": c}]  # no operator: the uploads Excel filled had none
    return {"column": 0, "hides": [3, 7], "custom": {"and": False, "conditions": conditions}}


SPECS |= {
    "t4895-custom-filter-delete-one": case(
        BASE + "Cat's criterion is a custom filter, equals a or equals c. The edit: delete_column at C "
        "deletes Ext; the Cat criterion stays.",
        [3, 5, 7],
        DELETE_C,
        {},
        [custom("a", "c"), EXT],
    ),
    "t4939-display-text-filter-control": case(
        BASE + "Ext is formatted 0.00 and its filter lists the displayed text 10.00 .. 60.00. No edit.",
        [3, 5, 7],
        [],
        FORMAT_C,
        [{"column": 0, "hides": [3, 7]}, DISPLAY],
    ),
    "t4939-display-text-filter-delete-one": case(
        BASE + "Ext is formatted 0.00 and its filter lists the displayed text 10.00 .. 60.00. The edit: "
        "delete_column at A deletes Cat; the Ext criterion stays.",
        [3, 5, 7],
        DELETE_A,
        FORMAT_C,
        [{"column": 0, "hides": [3, 7]}, DISPLAY],
    ),
}

if __name__ == "__main__":
    for name, spec in SPECS.items():
        (CASES / f"{name}.case.json").write_text(json.dumps(spec, indent=2) + "\n")
