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
import os
import sys

OUT = "eval/suites/probes/cases/t4430-legacy-at-name-body-rest.case.json"

NAMES = {
    "nRef": "=#REF!",
    "nDiv": "=#DIV/0!",
    "nPlus": "=+3",
    "nNN": "=--3",
}
SUPPORT = {"nOne": "={3}", "nRow": "={3,4}", "nCol": "={3;4}", "n2d": "={1,2;3,4}", "nRel": "=Sheet1!Z3"}
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
    "INDEX(nOne,1,1)",
    "INDEX(nOne,1)",
    "INDEX(nRow,1,2)",
    "INDEX(nCol,2,1)",
    "INDEX(nRow,1)",
    "INDEX(nRow,2)",
    "INDEX(nCol,1)",
    "INDEX(n2d,5,5)",
    "INDEX(n2d,1,1,1)",
    "INDEX(n2d,1,5)",
]
WRAPS = ["{x}", "ABS({x})", "IF(TRUE,{x})", "{x}+0", "SUM({x})"]


sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "t4273"))
from gen_probe import col, name_rows, write_case  # noqa: E402  shared helpers (CLAUDE.md directive 3)


def main() -> None:
    cells, legacy = name_rows(NAMES)
    for w, wrap in enumerate(WRAPS):
        for k, ix in enumerate(INDEXES):
            v, f = col(2 * k), col(2 * k + 1)
            cells[f"{v}{45 + w}"] = "=" + wrap.format(x=ix)
            cells[f"{f}{45 + w}"] = f"=FORMULATEXT({v}{45 + w})"
            legacy.append(f"{v}{45 + w}")
    cells["Z10"] = "=Z1:Z6*1"
    legacy.append("Z10")
    rows = ", ".join(f"row {r}: {n} {b}" for r, (n, b) in enumerate(NAMES.items(), start=30))
    description = (
        "T-4430: legacy (no cm) formulas over the defined-name bodies T-4273 left unprobed, which show the "
        "implicit-intersection @. Z1:Z6 = 1..6; nOne ={3}, nRow ={3,4}, nCol ={3;4}, n2d ={1,2;3,4}. One name per "
        f"row from 30: {rows}. Columns (value, FORMULATEXT beside it) as t3856-legacy-at-name-body: A/B root, C/D "
        "ABS, E/F SUM, G/H IF arm, I/J ABS(IF arm), K/L SUM(IF arm), M/N INDEX(Z1:Z6,name), O/P "
        "IF(TRUE,INDEX(Z1:Z6,name)), Q/R ABS of that, S/T SUM of that. Rows 45-49 wrap (raw, ABS, IF(TRUE,), +0, "
        "SUM) the INDEXES in columns A, C, ...: " + ", ".join(INDEXES) + ". Z10 = Z1:Z6*1 proves the formulas "
        "stayed legacy."
    )
    write_case(OUT, description, {**SUPPORT, **NAMES}, cells, legacy)


if __name__ == "__main__":
    main()
