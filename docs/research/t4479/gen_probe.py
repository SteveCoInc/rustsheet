"""Writes the T-4479 probes: which of several bad arguments SORT and SORTBY report.

probes/t4479-sort-bad-argument-order holds the forms the task names and a full grid of one bad
kind per argument; probes/t4479-sort-bad-argument-holdout is a seeded sample of other
combinations, with other values of each kind, frozen before its fill (decision 0188 R4).
Each result is read as TEXTJOIN, or ERROR.TYPE on an error (as probes/t4447 does), except the
lifted by_col forms, which spill and are written as they are.

Usage: python3 gen_probe.py <cases dir>
"""

import itertools
import json
import random
import sys
from pathlib import Path

# A1:A3 = 3, 1, 2 (sorted 1,2,3); C1:C3 = c, a, b; Z1 and Z2 are blank.
DATA: dict[str, object] = {"A1": 3, "A2": 1, "A3": 2, "C1": "c", "C2": "a", "C3": "b"}


def read(call: str) -> str:
    return f'=IFERROR(TEXTJOIN(",",,{call}),ERROR.TYPE({call}))'


# The task's named forms, plain (T) or spilled (S).
NAMED = [
    ("T", 'SORT(A1:A3,1,"a"+1,NA())'),
    ("T", 'SORT(A1:A3,"a"+1,1/0)'),
    ("S", 'SORT(A1:A3,1,"a"+1,{TRUE,FALSE})'),
    ("T", 'SORT(A1:A3,1,"a",NA())'),
    ("T", 'SORT(A1:A3,"a"+1,"a",NA())'),
    ("T", 'SORT(A1:A3,"1",1/0)'),
    ("T", 'SORT(A1:A3,1,"-1")'),
    ("S", 'SORT(A1:A3,1,"a",{TRUE,FALSE})'),
    ("S", 'SORT(A1:A3,1,1/0,{TRUE,FALSE})'),
    ("T", 'SORTBY(A1:A3,C1:C3,"a",C1:C3,NA())'),
    ("T", 'SORTBY(A1:A3,C1:C3,"a"+1,C1:C3,NA())'),
    ("T", 'SORTBY(A1:A3,C1:C3,"a",NA(),1)'),
    ("T", 'SORTBY(A1:A3,C1:C3,"a",C1:C2,1)'),
    ("T", 'SORTBY(A1:A3,C1:C3,TRUE,C1:C3,NA())'),
    ("T", 'SORTBY(A1:A3,C1:C3,"a")'),
    ("T", 'SORTBY(A1:A3,C1:C3,TRUE)'),
    ("T", 'SORTBY(A1:A3,C1:C3,FALSE)'),
    ("T", 'SORTBY(A1:A3,C1:C3,"-1")'),
    ("T", 'SORTBY(A1:A3,C1:C3,"1",C1:C3,NA())'),
    ("T", 'SORTBY(A1:A3,C1:C3,Z1,C1:C3,NA())'),
    ("T", 'SORTBY(A1:A3,C1:C3,"a",C1:C3,"b")'),
    ("T", 'SORT(A1:A3,"x",0,NA())'),
    ("T", 'SORT(A1:A3,5,"x",1/0)'),
    ("T", "SORT(A1:A3,0,Z1,NA())"),
    ("T", 'SORTBY(A1:A3,C1:C3,0,C1:C3,"x",C1:C3,NA())'),
    ("T", 'SORTBY(A1:A3,C1:C2,"x",C1:C3,1/0)'),
]

# One bad value per kind: text, out of range, blank, error. Each argument's error differs, so the
# error that wins names its argument.
SORT_INDEX = {"text": '"x"', "range": "0", "blank": "Z1", "error": "NA()"}
SORT_ORDER = {"text": '"x"', "range": "5", "blank": "Z1", "error": "1/0"}
SORT_BY_COL = {"text": '"x"', "error": "SQRT(-1)"}
SORTBY_ORDER1 = {"text": '"x"', "range": "0", "blank": "Z1", "error": "SQRT(-1)"}
SORTBY_KEY2 = {"short": "C1:C2", "error": "NA()"}
SORTBY_ORDER2 = {"text": '"x"', "range": "2", "blank": "Z1", "error": "1/0"}

# The hold-out's values of each kind, none used above.
H_INDEX = {"text": '"zz"', "range": "-1", "blank": "Z2", "error": "#REF!", "range2": "2"}
H_ORDER = {"text": '"zz"', "range": "-2", "blank": "Z2", "error": "#NULL!", "range2": "0.5"}
H_BY_COL = {"text": '"y"', "error": "#NAME?", "ok": "FALSE"}
H_KEY1 = {"ok": "C1:C3", "short": "A1:A2", "error": "#REF!", "wide": "A1:C3"}
H_ORDER1 = {"text": '"zz"', "range": "-2", "blank": "Z2", "error": "#NULL!", "ok": "-1"}
H_KEY2 = {"short": "C2:C3", "error": "#NAME?", "ok": "C1:C3"}
H_ORDER2 = {"text": '"y"', "range": "3", "blank": "Z2", "error": "#NUM!"}


def case(description: str, tag: str, calls: list[tuple[str, str]]) -> dict[str, object]:
    cells: dict[str, object] = dict(DATA)
    row = 1
    for kind, call in calls:
        cells[f"G{row}"] = call
        cells[f"H{row}"] = read(call) if kind == "T" else f"={call}"
        row += 1 if kind == "T" else 2
    return {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M7", "probe", "lookup", tag],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }


def main(out: Path) -> None:
    calls = list(NAMED)
    for i, o, b in itertools.product(SORT_INDEX.values(), SORT_ORDER.values(), SORT_BY_COL.values()):
        calls.append(("T", f"SORT(A1:A3,{i},{o},{b})"))
    for o1, k2, o2 in itertools.product(
        SORTBY_ORDER1.values(), SORTBY_KEY2.values(), SORTBY_ORDER2.values()
    ):
        calls.append(("T", f"SORTBY(A1:A3,C1:C3,{o1},{k2},{o2})"))
    main_case = case(
        "T-4479: which of several bad arguments SORT and SORTBY report, over A1:A3 = 3,1,2 and "
        "C1:C3 = c,a,b (Z1 blank). Column G is the call, H its result as TEXTJOIN, or ERROR.TYPE "
        "on an error; the lifted by_col rows spill. Rows after the named forms are a grid of one "
        "bad kind (text, out of range, blank, error) per argument, each argument with its own error.",
        "T-4479",
        calls,
    )
    rng = random.Random(4479)
    sort_grid = list(itertools.product(H_INDEX.values(), H_ORDER.values(), H_BY_COL.values()))
    sortby_grid = list(
        itertools.product(H_KEY1.values(), H_ORDER1.values(), H_KEY2.values(), H_ORDER2.values())
    )
    held = [("T", f"SORT(A1:A3,{i},{o},{b})") for i, o, b in rng.sample(sort_grid, 30)]
    held += [
        ("T", f"SORTBY(A1:A3,{k1},{o1},{k2},{o2})") for k1, o1, k2, o2 in rng.sample(sortby_grid, 40)
    ]
    holdout = case(
        "T-4479 hold-out: a seeded sample (random.Random(4479)) of other three- and four-argument "
        "combinations of SORT and SORTBY bad arguments, with values not used in "
        "t4479-sort-bad-argument-order; predictions frozen before the fill. Same layout (Z2 blank).",
        "T-4479",
        held,
    )
    for name, body in [
        ("t4479-sort-bad-argument-order", main_case),
        ("t4479-sort-bad-argument-holdout", holdout),
    ]:
        (out / f"{name}.case.json").write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
