"""Generate the T-4408 probe case (holiday given as text returned by a formula).

Run from the repository root: python3 docs/research/t4408/gen_probe.py
"""

import json
from pathlib import Path

CASES = Path("eval/suites/probes/cases")

# 45000 is Wednesday 2023-03-15; 45001 a Thursday, 45003 a Saturday.
FORMS = {
    45001: ['TEXT(45001,"0")', 'TEXT(45001,"m/d/yyyy")', '45001&""', 'IF(TRUE,"3/16/2023")'],
    45003: ['TEXT(45003,"0")', 'TEXT(45003,"m/d/yyyy")', '45003&""', 'IF(TRUE,"3/18/2023")'],
}
FUNCS = [
    "WORKDAY(45000,5,{h})",
    "WORKDAY.INTL(45000,5,1,{h})",
    "WORKDAY.INTL(45000,5,13,{h})",
    "NETWORKDAYS(45000,45010,{h})",
    "NETWORKDAYS.INTL(45000,45010,1,{h})",
    "NETWORKDAYS.INTL(45000,45010,13,{h})",
]
COLS = "BCDEFGHIJKLM"


def main() -> None:
    rows = [("none", ""), ("number 45001", "45001"), ("number 45003", "45003")]
    for day, forms in FORMS.items():
        rows += [(f"{day} {f}", f) for f in forms]
    sheet: dict[str, object] = {}
    for r, (label, h) in enumerate(rows, start=1):
        sheet[f"A{r}"] = label
        for c, func in enumerate(FUNCS):
            call = func.format(h=h) if h else func.replace(",{h})", ")")
            sheet[f"{COLS[c]}{r}"] = "=" + call
            sheet[f"{COLS[c + 6]}{r}"] = f"=ERROR.TYPE({call})"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4408: WORKDAY, WORKDAY.INTL (weekend 1, 13), NETWORKDAYS and NETWORKDAYS.INTL "
            "(weekend 1, 13) from 45000 (Wed 2023-03-15) with one holiday given as text that a "
            "formula returns directly in the argument: TEXT(d,\"0\"), TEXT(d,\"m/d/yyyy\"), "
            "d&\"\" and IF(TRUE,\"date text\") for 45001 (Thu) and 45003 (Sat); rows 1:3 are "
            "controls (none, numbers). Columns B:G bare results, H:M ERROR.TYPE. Label in A."
        ),
        "tags": ["M6", "probe", "date", "workday", "networkdays", "T-4408"],
        "sheets": [{"name": "Sheet1", "cells": sheet}],
    }
    path = CASES / "t4408-holiday-formula-text.case.json"
    path.write_text(json.dumps(case, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
