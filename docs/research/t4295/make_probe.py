"""T-4295: write probes/t4295-elapsed-with-month (run once; the case is committed).

Rows: values; columns: formats that put an elapsed token next to a month-or-minute `m`.
"""

import json
from pathlib import Path

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4295-elapsed-with-month.case.json"
VALUES = ("1.7/24+0.3/1440", "1/86400", "1/1440", "1/24", "0.5", "1.25", "2+5/86400", "45000.123", "33")
FORMATS = ("[s] m", "[s] mm", "[s]:m", "[s] m s", "m [m]", "m [s]", "m [h]", "mm [m]", "d [m]", "[s] d",
           "[s] h", "[m] s", "[s] mmm", "[h] [s]")


VALUES_B = ("45000", "45000.5", "45000+7/86400", "366", "367", "400", "1000", "10000", "20000", "30000", "40000",
            "44000", "46000", "60", "61", "0.25+7/86400", "45000.75+3/86400", "2958465")
FORMATS_B = ("m [h]", "d [h]", "yy [h]", "mmm [h]")
OUT_B = OUT.with_name("t4295-elapsed-with-month-b.case.json")


def write(out: Path, values: tuple[str, ...], formats: tuple[str, ...], what: str) -> None:
    cols = "ABCDEFGHIJKLMN"[: len(formats)]
    cells = {}
    for col, fmt in zip(cols, formats):
        for row, v in enumerate(values, start=1):
            cells[f"{col}{row}"] = f'=TEXT({v},"{fmt}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"T-4295: {what}. Columns {cols[0]}:{cols[-1]} are formats "
            + ", ".join(f"{c} {f}" for c, f in zip(cols, formats))
            + f"; rows 1:{len(values)} the values " + ", ".join(values) + "."
        ),
        "tags": ["M10", "probe", "dates", "T-4295"],
        "expected": {},
        "check": [f"Calc!{r}" for r in sorted(cells, key=lambda r: (int(r[1:]), r[0]))],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    out.write_text(json.dumps(case, indent=1) + "\n")


def main() -> None:
    write(OUT, VALUES, FORMATS, "an elapsed token next to m/mm in TEXT")
    write(OUT_B, VALUES_B, FORMATS_B, "what an elapsed token after a date token counts, by year, month, day and time")


def _unused() -> None:
    cells = {}
    for col, fmt in zip("ABCDEFGHIJKLMN", FORMATS):
        for row, v in enumerate(VALUES, start=1):
            cells[f"{col}{row}"] = f'=TEXT({v},"{fmt}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4295: an elapsed token next to m/mm in TEXT. Columns A:N are formats "
            + ", ".join(f"{c} {f}" for c, f in zip("ABCDEFGHIJKLMN", FORMATS))
            + "; rows 1:9 the values " + ", ".join(VALUES) + "."
        ),
        "tags": ["M10", "probe", "dates", "T-4295"],
        "expected": {},
        "check": [f"Calc!{r}" for r in sorted(cells, key=lambda r: (int(r[1:]), r[0]))],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    OUT.write_text(json.dumps(case, indent=1) + "\n")


if __name__ == "__main__":
    main()
