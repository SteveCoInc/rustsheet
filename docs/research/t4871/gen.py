"""Generate probes/t4871-empty-mode-args (T-4871): written-empty arguments of XLOOKUP, XMATCH and EXPAND.

Sheet1 A1:A3 = 1, 2, 3 (ascending), B1:B3 = 10, 20, 30, C1:C3 = 3, 2, 1 (descending). Column E holds the probes.
Each probe is paired with the same call where the empty argument is written as its documented default (or left out
when trailing): the engine's value of that call is H_omitted, the prediction (docs/research/t4871/predictions.tsv);
H_zero, the engine before T-4871 (the empty argument read as 0), is the alternative. EXPAND is read through ROWS,
COLUMNS and INDEX so nothing spills.
Run: python3 docs/research/t4871/gen.py case > eval/suites/probes/cases/t4871-empty-mode-args.case.json
     python3 docs/research/t4871/gen.py omitted   (prints cell<TAB>probe<TAB>default-filled call)
"""

import json
import sys

A, B, C = "A1:A3", "B1:B3", "C1:C3"
# (probe, the same call with the empty argument as its default)
PAIRS = [
    (f"XLOOKUP(2,{A},{B},)", f"XLOOKUP(2,{A},{B})"),
    (f"XLOOKUP(5,{A},{B},)", f"XLOOKUP(5,{A},{B})"),
    (f"XLOOKUP(2,{A},{B},,)", f"XLOOKUP(2,{A},{B})"),
    (f"XLOOKUP(2.5,{A},{B},,)", f"XLOOKUP(2.5,{A},{B})"),
    (f"XLOOKUP(2,{A},{B},,,)", f"XLOOKUP(2,{A},{B})"),
    (f"XLOOKUP(5,{A},{B},,,)", f"XLOOKUP(5,{A},{B})"),
    (f'XLOOKUP(2,{A},{B},"nf",0,)', f'XLOOKUP(2,{A},{B},"nf",0)'),
    (f'XLOOKUP(2.5,{A},{B},"nf",,-1)', f'XLOOKUP(2.5,{A},{B},"nf",0,-1)'),
    (f"XLOOKUP(2.5,{A},{B},,-1,)", f"XLOOKUP(2.5,{A},{B},NA(),-1)"),
    (f"XLOOKUP(2.5,{A},{B},,1,)", f"XLOOKUP(2.5,{A},{B},NA(),1)"),
    (f'XLOOKUP(2,{C},{B},"nf",,-1)', f'XLOOKUP(2,{C},{B},"nf",0,-1)'),
    (f"XMATCH(2,{A},)", f"XMATCH(2,{A})"),
    (f"XMATCH(2,{A},,)", f"XMATCH(2,{A})"),
    (f"XMATCH(2,{A},0,)", f"XMATCH(2,{A},0)"),
    (f"XMATCH(2.5,{A},-1,)", f"XMATCH(2.5,{A},-1)"),
    (f"XMATCH(2.5,{C},1,)", f"XMATCH(2.5,{C},1)"),
    (f"XMATCH(2,{C},,)", f"XMATCH(2,{C})"),
    (f"XMATCH(2.5,{A},,2)", f"XMATCH(2.5,{A},0,2)"),
    (f"XMATCH(2,{A},,-1)", f"XMATCH(2,{A},0,-1)"),
    (f"LET(e,,XMATCH(2,{A},0,e))", f"XMATCH(2,{A},0)"),
    ("ROWS(EXPAND(A1:B1,3,,0))", "ROWS(EXPAND(A1:B1,3,2,0))"),
    ("COLUMNS(EXPAND(A1:B1,3,,0))", "COLUMNS(EXPAND(A1:B1,3,2,0))"),
    ("ROWS(EXPAND(A1:A2,,3,0))", "ROWS(EXPAND(A1:A2,2,3,0))"),
    ("COLUMNS(EXPAND(A1:A2,,3,0))", "COLUMNS(EXPAND(A1:A2,2,3,0))"),
    ("ROWS(EXPAND(A1:A2,,,0))", "ROWS(EXPAND(A1:A2,2,1,0))"),
    ("COLUMNS(EXPAND(A1:A2,,,0))", "COLUMNS(EXPAND(A1:A2,2,1,0))"),
    ('INDEX(EXPAND(A1:B1,3,,"p"),3,2)', 'INDEX(EXPAND(A1:B1,3,2,"p"),3,2)'),
    ("INDEX(EXPAND(A1:A2,,3),2,3)", "INDEX(EXPAND(A1:A2,2,3),2,3)"),
    ("INDEX(EXPAND(A1:B1,3,),3,1)", "INDEX(EXPAND(A1:B1,3),3,1)"),
    ("ROWS(EXPAND(5,,,0))", "ROWS(EXPAND(5,1,1,0))"),
    ("ROWS(EXPAND(A1:B1,,))", "ROWS(EXPAND(A1:B1,1,2))"),
]


def main() -> None:
    if sys.argv[1:] == ["omitted"]:
        for i, (probe, filled) in enumerate(PAIRS, start=1):
            print(f"E{i}\t={probe}\t={filled}")
        return
    cells: dict[str, object] = {}
    for i, (a, b, c) in enumerate([(1, 10, 3), (2, 20, 2), (3, 30, 1)], start=1):
        cells[f"A{i}"], cells[f"B{i}"], cells[f"C{i}"] = a, b, c
    for i, (probe, _) in enumerate(PAIRS, start=1):
        cells[f"E{i}"] = f"={probe}"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "Written-empty arguments of XLOOKUP, XMATCH and EXPAND (T-4871). Sheet1 A1:A3 = 1..3, B1:B3 = 10..30, "
            "C1:C3 = 3..1; probes in E. Predictions in docs/research/t4871/predictions.tsv committed before the "
            "excel-graph fill. H_omitted: an empty argument takes its default; H_zero: it is read as 0 (the engine "
            "before T-4871)."
        ),
        "tags": ["M6", "lookup", "probe"],
        "expected": {},
        "check": [f"Sheet1!E{i}" for i in range(1, len(PAIRS) + 1)],
        "sheets": [{"name": "Sheet1", "cells": cells}],
    }
    print(json.dumps(case, indent=2))


main()
