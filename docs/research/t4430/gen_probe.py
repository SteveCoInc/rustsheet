"""T-4430: write probes/t4430-legacy-at-name-body-rest, the defined-name bodies T-3856 left to engine choice.

Run from the repo root: python3 -I docs/research/t4430/gen_probe.py.

Layout as probes/t3856-legacy-at-name-body: Z1:Z6 = 1..6, one name per row from 30, columns (value, FORMULATEXT
beside it) A/B root, C/D ABS, E/F SUM, G/H IF arm, I/J ABS(IF arm), K/L SUM(IF arm), M/N INDEX(Z1:Z6,name), O/P
IF(TRUE,INDEX(Z1:Z6,name)), Q/R ABS of that, S/T SUM of that. Row 45: INDEX over the array-bodied n2d. Z10 =
Z1:Z6*1 shows the formulas stayed legacy.

`nRel` = Sheet1!Z3 is relative. A defined name's relative reference is stored relative to A1, so from column c,
row 30 it reads the cell c columns right of Z and 29 rows below Z3. Each such cell on row 32 (Z32, AB32, ..., AR32,
the value columns' offsets) holds 2, so 2 means the shifted read and 3 means Z3.
"""

import json

OUT = "eval/suites/probes/cases/t4430-legacy-at-name-body-rest.case.json"

NAMES = {
    "nRef": "=#REF!",
    "nDiv": "=#DIV/0!",
    "nPlus": "=+3",
    "nNN": "=--3",
}
SUPPORT = {"n1": "={3}", "nR": "={3,4}", "nC": "={3;4}", "n2d": "={1,2;3,4}", "nRel": "=Sheet1!Z3"}
SHAPES = [
    "{n}",
    "ABS({n})",
    "SUM({n})",
    "IF(TRUE,{n})",
    "ABS(IF(TRUE,{n}))",
    "SUM(IF(TRUE,{n}))",
    "INDEX(Z1:Z6,{n})",
    "IF(TRUE,INDEX(Z1:Z6,{n}))",
    "ABS(IF(TRUE,INDEX(Z1:Z6,{n})))",
    "SUM(IF(TRUE,INDEX(Z1:Z6,{n})))",
]
INDEXES = [
    "INDEX(n1,1,1)",
    "INDEX(n1,1)",
    "INDEX(nR,1,2)",
    "INDEX(nC,2,1)",
    "INDEX(nR,1)",
    "INDEX(nR,2)",
    "INDEX(nC,1)",
    "INDEX(n2d,5,5)",
    "INDEX(n2d,1,1,1)",
    "INDEX(n2d,1,5)",
]
WRAPS = ["{x}", "ABS({x})", "IF(TRUE,{x})", "{x}+0", "SUM({x})"]


def col(i: int) -> str:
    s = ""
    i += 1
    while i:
        i, r = divmod(i - 1, 26)
        s = chr(65 + r) + s
    return s


def main() -> None:
    cells: dict[str, object] = {f"Z{i}": i for i in range(1, 7)}
    legacy: list[str] = []
    for k in range(0, 20, 2):
        cells[f"{col(25 + k)}32"] = 2
    for r, name in enumerate(NAMES, start=30):
        for k, shape in enumerate(SHAPES):
            v, f = col(2 * k), col(2 * k + 1)
            cells[f"{v}{r}"] = "=" + shape.format(n=name)
            cells[f"{f}{r}"] = f"=FORMULATEXT({v}{r})"
            legacy.append(f"{v}{r}")
    for w, wrap in enumerate(WRAPS):
        for k, ix in enumerate(INDEXES):
            v, f = col(2 * k), col(2 * k + 1)
            cells[f"{v}{45 + w}"] = "=" + wrap.format(x=ix)
            cells[f"{f}{45 + w}"] = f"=FORMULATEXT({v}{45 + w})"
            legacy.append(f"{v}{45 + w}")
    cells["Z10"] = "=Z1:Z6*1"
    legacy.append("Z10")
    rows = ", ".join(f"row {r}: {n} {b}" for r, (n, b) in enumerate(NAMES.items(), start=30))
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4430: legacy (no cm) formulas over the defined-name bodies T-3856 left unprobed, which show the "
            "implicit-intersection @. Z1:Z6 = 1..6; n1 ={3}, nR ={3,4}, nC ={3;4}, n2d ={1,2;3,4}. One name per "
            f"row from 30: {rows}. nRel is relative (stored relative to A1): each value column's shifted cell on row "
            "32 (Z32, AB32, ..., AR32) holds 2. Columns (value, FORMULATEXT beside it) as t3856-legacy-at-name-body: "
            "A/B root, C/D ABS, E/F SUM, G/H IF arm, I/J ABS(IF arm), K/L SUM(IF arm), M/N INDEX(Z1:Z6,name), O/P "
            "IF(TRUE,INDEX(Z1:Z6,name)), Q/R ABS of that, S/T SUM of that. Rows 45-49 wrap (raw, ABS, IF(TRUE,), +0, SUM) the INDEXES in columns A, C, ...: " + ", ".join(INDEXES) + ". Z10 = "
            "Z1:Z6*1 proves the formulas stayed legacy."
        ),
        "tags": ["M8", "references", "probe"],
        "names": {**SUPPORT, **NAMES},
        "expected": {},
        "sheets": [{"name": "Sheet1", "legacy_formulas": legacy, "cells": cells}],
    }
    with open(OUT, "w") as fh:
        json.dump(case, fh, indent=2, ensure_ascii=False)
        fh.write("\n")
    print(len(legacy), "legacy formulas")


if __name__ == "__main__":
    main()
