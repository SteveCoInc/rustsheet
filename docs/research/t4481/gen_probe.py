"""Writes probes/t4481-at-spill-operand-edges (T-4481): the `@` and `#` operand forms references.md
left unprobed. Column H is the formula, I its FORMULATEXT and J ERROR.TYPE of it.

`(expr)#` is written as `ANCHORARRAY((expr))`, which is how Excel stores it: spec2xlsx copies a
`#` after a parenthesis into the file as is, and Excel refuses that file text.
`=ANCHORARRAY((1/0))` and `=ANCHORARRAY((NA()))` are not in the case: Excel for the web refuses a
workbook holding either (createSession 501 `FileCorruptTryRepair`, one scratch workbook each, with
`=ANCHORARRAY((A1:A2))` loading in the same shape as a control; 2026-11-06).

Usage: python3 gen_probe.py <cases dir>
"""

import json
import sys
from pathlib import Path

# (row, formula). Rows 2 and 3 meet D1:D3 and F1:F3, so an `@` of them picks that row's cell;
# from row 10 on no row meets them, and every form is 4 rows from the next in case it spills.
FORMS = [
    (2, "=(@rg)"),
    (3, "=@(D1#)"),
    (10, "=@(INDEX(F1:F3,2))"),
    (14, "=@INDEX(F1:F3,2)"),
    (18, "=@(D1#)"),
    (22, "=@D1#"),
    (26, "=@(rg)"),
    (30, "=@rg"),
    (34, "=@(one)"),
    (38, "=(@rg)"),
    (42, "=@TRUE"),
    (46, "=@(TRUE)"),
    (50, "=@FALSE+1"),
    (54, "=ANCHORARRAY((@(A1 B1)))"),
    (58, "=ANCHORARRAY((@(F1:F3 F2)))"),
    (62, "=(@(A1 B1)):INDEX(F1:F3,5)"),
    (66, "=INDEX(F1:F3,5):(@(A1 B1))"),
]


def main(out: Path) -> None:
    cells: dict[str, object] = {"D1": "=SEQUENCE(3)", "F1": 1, "F2": 2, "F3": 3}
    for row, formula in FORMS:
        cells[f"H{row}"] = formula
        cells[f"I{row}"] = f"=FORMULATEXT(H{row})"
        cells[f"J{row}"] = f"=ERROR.TYPE(H{row})"
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4481: `@` and `#` operand forms. D1 = SEQUENCE(3), F1:F3 = 1, 2, 3, names rg = "
        "Sheet1!$F$1:$F$3 and one = Sheet1!$F$2; A1 and B1 are blank. H is the formula, I its "
        "FORMULATEXT, J ERROR.TYPE of it. Rows 2 and 3 meet D1:D3 and F1:F3; rows 10 on do not.",
        "tags": ["M7", "references", "at", "probe"],
        "names": {"rg": "=Sheet1!$F$1:$F$3", "one": "=Sheet1!$F$2"},
        "sheets": [{"name": "Sheet1", "cells": cells}],
        "expected": {},
    }
    path = out / "t4481-at-spill-operand-edges.case.json"
    path.write_text(json.dumps(case, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
