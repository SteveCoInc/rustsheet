"""Generate the T-4397 probe cases (holiday text forms, DAYS with two errors).

Run from the repository root: python3 docs/research/t4397/gen_probe.py
"""

import json
from pathlib import Path

CASES = Path("eval/suites/probes/cases")

# 45000 is Wednesday 2023-03-15; 45001 a Thursday, 45003 a Saturday, 45005 a Monday.
TEXTS = {
    45001: ["45001", "3/16/2023", "March 16, 2023", "$45,001", "4500100%"],
    45003: ["45003", "3/18/2023", "March 18, 2023", "$45,003", "4500300%"],
}
RANGE_NUMBER = 45005
FUNCS = [
    "WORKDAY(45000,5,{h})",
    "WORKDAY.INTL(45000,5,1,{h})",
    "WORKDAY.INTL(45000,5,13,{h})",
    "NETWORKDAYS(45000,45010,{h})",
    "NETWORKDAYS.INTL(45000,45010,1,{h})",
    "NETWORKDAYS.INTL(45000,45010,13,{h})",
]
COLS = "BCDEFGHIJKLM"  # six bare columns, then six ERROR.TYPE columns


def q(text: str) -> str:
    return '"' + text + '"'


def holidays_case() -> dict:
    data: dict[str, object] = {}
    rows: list[tuple[str, str]] = [("none", "")]
    row = 1
    for day, texts in TEXTS.items():
        rows.append((f"number {day}", str(day)))
        for text in texts:
            data[f"A{row}"] = text
            data[f"B{row}"] = RANGE_NUMBER
            rows.append((f"literal {text}", q(text)))
            rows.append((f"cell {text}", f"Data!A{row}"))
            rows.append((f"array {text}", "{" + q(text) + "}"))
            rows.append((f"range {text} + {RANGE_NUMBER}", f"Data!A{row}:B{row}"))
            row += 1
    sheet: dict[str, object] = {}
    for r, (label, h) in enumerate(rows, start=1):
        sheet[f"A{r}"] = label
        for c, func in enumerate(FUNCS):
            call = func.format(h=h) if h else func.replace(",{h})", ")")
            sheet[f"{COLS[c]}{r}"] = "=" + call
            sheet[f"{COLS[c + 6]}{r}"] = f"=ERROR.TYPE({call})"
    return {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4397: WORKDAY, WORKDAY.INTL (weekend 1, 13), NETWORKDAYS and NETWORKDAYS.INTL "
            "(weekend 1, 13) from 45000 (Wed 2023-03-15) with one holiday given as a number "
            "(control), a text literal, a cell holding the text, a one-element array of the "
            "text, or a two-cell range of the text cell and 45005 (Mon). Texts: number, "
            "m/d/yyyy, long date, currency and percent text of 45001 (Thu) and 45003 (Sat). "
            "Columns B:G bare results, H:M ERROR.TYPE of the same calls. Row label in A."
        ),
        "tags": ["M6", "probe", "date", "workday", "networkdays", "T-4397"],
        "sheets": [
            {"name": "Sheet1", "cells": sheet},
            {"name": "Data", "cells": data},
        ],
    }


def days_case() -> dict:
    data = {"A1": "=NA()", "A2": "=1/0", "A3": "=#REF!", "A4": "=#NAME?", "A5": "abc"}
    # One "first second" pair per line: literals, error calls, references to error cells, mixed,
    # and an error beside text as a control.
    pairs = [
        tuple(line.split())
        for line in """
NA() 1/0
1/0 NA()
#REF! #NAME?
#NAME? #REF!
#NULL! #NUM!
#NUM! #NULL!
Data!A1 Data!A2
Data!A2 Data!A1
Data!A3 Data!A4
Data!A4 Data!A3
Data!A1 1/0
1/0 Data!A1
NA() "abc"
"abc" NA()
Data!A1 Data!A5
Data!A5 Data!A1
""".strip().splitlines()
    ]
    sheet: dict[str, object] = {}
    for r, (a, b) in enumerate(pairs, start=1):
        sheet[f"A{r}"] = f"=DAYS({a},{b})"
        sheet[f"B{r}"] = f"=ERROR.TYPE(DAYS({a},{b}))"
    return {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4397: DAYS with an error in both arguments (literals, error-returning calls, "
            "one-cell references to error cells, mixed), and an error beside text as a control. "
            "A: bare result, B: ERROR.TYPE of it."
        ),
        "tags": ["M6", "probe", "date", "days", "T-4397"],
        "sheets": [
            {"name": "Sheet1", "cells": sheet},
            {"name": "Data", "cells": data},
        ],
    }


def main() -> None:
    for name, case in [
        ("t4397-holiday-text-forms", holidays_case()),
        ("t4397-days-two-errors", days_case()),
    ]:
        path = CASES / f"{name}.case.json"
        path.write_text(json.dumps(case, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
