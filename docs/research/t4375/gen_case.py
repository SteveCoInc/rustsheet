"""T-4375: XNPV behind a refused first value, later walk errors, and one-cell argument forms
(eval/suites/probes/cases/t4375-xnpv-refused-first-and-one-cell.case.json). Deterministic; run with `python3 -I`.

Sheet D: value columns B..E, date columns F..O, one-cell holders Q1..Q3 (#N/A, "30", "1/1/2025").
Calc row r: A range form, B array-constant form (when writable), C computed-array form.
"""

from __future__ import annotations

import json
from pathlib import Path

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4375-xnpv-refused-first-and-one-cell.case.json"
GOOD = [39448, 39508, 39600, 39700]
NA, DIV0 = "=NA()", "=1/0"
data: dict[str, object] = {}
calc: dict[str, object] = {}
notes: dict[str, str] = {}


def col(i: int) -> str:
    return chr(65 + i)


def column(i: int, items: list[object]) -> str:
    for r, it in enumerate(items):
        if it is not None:
            data[f"{col(i)}{r + 1}"] = it
    return col(i)


# (label, items, constant text or None when not writable)
VALUES = [
    ('values <"x",30,#N/A,50>', ["x", 30, NA, 50], '{"x";30;#N/A;50}'),
    ("values <empty,30,40,50>", [None, 30, 40, 50], None),
    ("values <TRUE,30,40,50>", [True, 30, 40, 50], "{TRUE;30;40;50}"),
    ('values <"x",30,40,50>', ["x", 30, 40, 50], '{"x";30;40;50}'),
]
DATES = [
    ("good dates", GOOD, "{39448;39508;39600;39700}"),
    ("dates #DIV/0! at 2", [39448, DIV0, 39600, 39700], "{39448;#DIV/0!;39600;39700}"),
    ("dates -1 at 2", [39448, -1, 39600, 39700], "{39448;-1;39600;39700}"),
    ("dates 2958466 at 2", [39448, 2958466, 39600, 39700], "{39448;2958466;39600;39700}"),
    ("dates #DIV/0! at 4", [39448, 39508, 39600, DIV0], "{39448;39508;39600;#DIV/0!}"),
    ("dates -1 at 4", [39448, 39508, 39600, -1], "{39448;39508;39600;-1}"),
    ("dates 2958466 at 4", [39448, 39508, 39600, 2958466], "{39448;39508;39600;2958466}"),
    ("dates 0 at 2", [39448, 0, 39600, 39700], "{39448;0;39600;39700}"),
    ("dates empty at 2", [39448, None, 39600, 39700], None),
    ("dates before first at 2", [39448, 39400, 39600, 39700], "{39448;39400;39600;39700}"),
]
vcols = [column(1 + i, v[1]) for i, v in enumerate(VALUES)]  # B..E
dcols = [column(5 + i, d[1]) for i, d in enumerate(DATES)]  # F..O
ncol = col(16)
data[f"{ncol}1"], data[f"{ncol}2"], data[f"{ncol}3"] = NA, "30", "1/1/2025"
numfirst = column(17, [-100, 30, NA, 50])  # R


def emit(label: str, vrng: str, drng: str, vconst: str | None, dconst: str | None, row: int) -> None:
    calc[f"A{row}"] = f"=XNPV(0.1,D!{vrng},D!{drng})"
    if vconst and dconst:
        calc[f"B{row}"] = f"=XNPV(0.1,{vconst},{dconst})"
    calc[f"C{row}"] = f"=XNPV(0.1,IF(TRUE,D!{vrng}),IF(TRUE,D!{drng}))"
    notes[str(row)] = label


row = 1
for (vl, _, vc), vcol in zip(VALUES, vcols):
    for (dl, _, dc), dcol in zip(DATES[:7], dcols):
        emit(f"{vl}; {dl}", f"{vcol}1:{vcol}4", f"{dcol}1:{dcol}4", vc, dc, row)
        row += 1
for dl, _, dc in DATES[7:]:
    dcol = dcols[[d[0] for d in DATES].index(dl)]
    emit(f"values <-100,30,#N/A,50>; {dl}", f"{numfirst}1:{numfirst}4", f"{dcol}1:{dcol}4",
         "{-100;30;#N/A;50}", dc, row)
    row += 1
# One-cell forms of Q1 (#N/A), Q2 (numeric text), Q3 (date text) as values against D!F1 and as dates against -100.
forms = [
    ("one-cell range", "D!Q{n}:Q{n}"),
    ("OFFSET", "OFFSET(D!Q1,{o},0)"),
    ("INDEX", "INDEX(D!Q1:Q3,{n})"),
    ("name", "OneCell{n}x"),
]
for n in (1, 2, 3):
    for label, f in forms:
        ref = f.format(n=n, o=n - 1)
        calc[f"A{row}"] = f"=XNPV(0.1,{ref},D!F1)"
        calc[f"B{row}"] = f"=XNPV(0.1,D!{numfirst}1,{ref})"
        notes[str(row)] = f"{label} of Q{n}: as values (A), as dates (B)"
        row += 1
case = {
    "format": "rustsheet-case/v1",
    "description": "T-4375: XNPV behind a refused first value with later errors or bad dates, a number-first walk "
    "with later 0/empty/before-first dates and an error, and one-cell arguments (range, OFFSET, INDEX, name) holding "
    "an error, numeric text or date text (docs/research/t4375/gen_case.py; rows: "
    + "; ".join(f"{k} {v}" for k, v in notes.items()) + ")",
    "tags": ["M9", "financial", "xnpv", "probe", "T-4375"],
    "names": {f"OneCell{n}x": f"=D!$Q${n}" for n in (1, 2, 3)},
    "expected": {},
    "sheets": [{"name": "Calc", "cells": calc}, {"name": "D", "cells": data}],
}
OUT.write_text(json.dumps(case, indent=1) + "\n")
