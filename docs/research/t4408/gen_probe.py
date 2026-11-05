"""Generate the T-4408 probe case (holiday text returned by a formula).

Run from the repository root: python3 docs/research/t4408/gen_probe.py
"""

import json
from pathlib import Path

CASES = Path("eval/suites/probes/cases")

# 45000 is Wednesday 2023-03-15; 45001 a Thursday, 45003 a Saturday.
DAYS = {45001: "3/16/2023", 45003: "3/18/2023"}
FUNCS = [
    "WORKDAY(45000,5,{h})",
    "WORKDAY.INTL(45000,5,1,{h})",
    "WORKDAY.INTL(45000,5,13,{h})",
    "NETWORKDAYS(45000,45010,{h})",
    "NETWORKDAYS.INTL(45000,45010,1,{h})",
    "NETWORKDAYS.INTL(45000,45010,13,{h})",
]
COLS = "BCDEFGHIJKLM"  # six bare columns, then six ERROR.TYPE columns


def case() -> dict:
    rows: list[tuple[str, str]] = []
    for d, txt in DAYS.items():
        rows.append((f"number {d}", str(d)))
        rows.append((f"TEXT {d} 0", f'TEXT({d},"0")'))
        rows.append((f"TEXT {d} m/d/yyyy", f'TEXT({d},"m/d/yyyy")'))
        rows.append((f"concat {d}", f'{d}&""'))
        rows.append((f"IF text {txt}", f'IF(TRUE,"{txt}")'))
    sheet: dict[str, object] = {}
    for r, (label, h) in enumerate(rows, start=1):
        sheet[f"A{r}"] = label
        for c, func in enumerate(FUNCS):
            call = func.format(h=h)
            sheet[f"{COLS[c]}{r}"] = "=" + call
            sheet[f"{COLS[c + 6]}{r}"] = f"=ERROR.TYPE({call})"
    return {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4408: WORKDAY, WORKDAY.INTL (weekend 1, 13), NETWORKDAYS and NETWORKDAYS.INTL "
            "(weekend 1, 13) from 45000 (Wed 2023-03-15) with the holiday given as text "
            "returned directly by a formula: TEXT(d,\"0\"), TEXT(d,\"m/d/yyyy\"), d&\"\" and "
            "IF(TRUE,\"m/d/yyyy\") for 45001 (Thu) and 45003 (Sat); a plain number is the "
            "control. Columns B:G bare results, H:M ERROR.TYPE of the same calls. Label in A."
        ),
        "tags": ["M6", "probe", "date", "workday", "networkdays", "T-4408"],
        "sheets": [{"name": "Sheet1", "cells": sheet}],
    }


if __name__ == "__main__":
    path = CASES / "t4408-holiday-formula-text.case.json"
    path.write_text(json.dumps(case(), indent=1) + "\n", encoding="utf-8")
