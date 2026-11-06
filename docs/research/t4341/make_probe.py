"""T-4341: write probes/t4341-elapsed-second-forms (run once; the case is committed).

Rows are values, columns the `[s]` shapes T-4295 left unprobed (see the task file).
"""

import json
from pathlib import Path

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4341-elapsed-second-forms.case.json"
VALUES = ("1.7/24+0.3/1440", "45000.123", "0.25+7/86400")
FORMATS = ("[ss] m", "[ss] h", "[ss] mm", "[s] m m", "[s] h m", "[s] m h", "d [s] m", "mmm [s] h", "[h] [s] m",
           "[m] [s] h", "[s].0 m", "[s].0 h", "s [s] m", "s [s] h")


def main() -> None:
    cols = "ABCDEFGHIJKLMN"[: len(FORMATS)]
    cells = {}
    for col, fmt in zip(cols, FORMATS):
        for row, v in enumerate(VALUES, start=1):
            cells[f"{col}{row}"] = f'=TEXT({v},"{fmt}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4341: the [s] forms T-4295 left unprobed. Columns "
            + ", ".join(f"{c} {f}" for c, f in zip(cols, FORMATS))
            + f"; rows 1:{len(VALUES)} the values " + ", ".join(VALUES) + "."
        ),
        "tags": ["M10", "probe", "dates", "T-4341"],
        "expected": {},
        "check": [f"Calc!{r}" for r in sorted(cells, key=lambda r: (int(r[1:]), r[0]))],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    OUT.write_text(json.dumps(case, indent=1) + "\n")


if __name__ == "__main__":
    main()
