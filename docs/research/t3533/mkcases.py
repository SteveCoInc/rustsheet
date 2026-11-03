"""T-3533: one probe workbook per (aggregate, element kind) over a lifted OFFSET, formula in E1, on the t2816 data
(probes/t2816-lift-*) plus A8 = -3.

Element kinds (A4 "5" text, A5 blank, A6 TRUE, A7 7, A8 -3):
- variance family (VAR.P, STDEV.P, VARA, VARPA, STDEVA, STDEVPA, VAR): three elements, so text or a blank read as 0,
  skipped, or read as its value all give different results: num {A1;A2;A3}, multi {A1:B1;A2:B2} (top-left or
  #VALUE!), text {A4;A7;A8}, blank {A5;A7;A8}, bool {A6;A7;A8}.
- MAXA / MINA: two elements, the other one -3 (A8) and 7 (A7), so that the extremum tells a skipped element
  from one read as 0 (MAXA with -3, MINA with 7).
Usage (repo root): python3 docs/research/t3533/mkcases.py"""

from __future__ import annotations

import json
from pathlib import Path

DIR = Path("eval/suites/probes/cases")
DATA = {"A1": 1, "A2": 100, "A3": 10000, "B1": 2, "B2": 200, "B3": 20000, "A4": "5", "A6": True, "A7": 7, "A8": -3}
VAR3 = {
    "num": "OFFSET(A1,{0;1;2},0)",
    "multi": "OFFSET(A1,{0;1},0,1,2)",
    "text": "OFFSET(A4,{0;3;4},0)",
    "blank": "OFFSET(A5,{0;2;3},0)",
    "bool": "OFFSET(A6,{0;1;2},0)",
}
EXT = {
    "num": "OFFSET(A1,{0;1},0)",
    "multi": "OFFSET(A1,{0;1},0,1,2)",
    "text-neg": "OFFSET(A4,{0;4},0)",
    "text-pos": "OFFSET(A4,{0;3},0)",
    "blank-neg": "OFFSET(A5,{0;3},0)",
    "blank-pos": "OFFSET(A5,{0;2},0)",
    "bool-neg": "OFFSET(A6,{0;2},0)",
    "bool-pos": "OFFSET(A6,{0;1},0)",
}
FUNCS = {f: VAR3 for f in ("VAR.P", "STDEV.P", "VARA", "VARPA", "STDEVA", "STDEVPA", "VAR")} | {"MAXA": EXT, "MINA": EXT}

n = 0
for fn, kinds in FUNCS.items():
    for kind, arg in kinds.items():
        case = {
            "format": "rustsheet-case/v1",
            "description": (
                f"T-3533: {fn} over a lifted OFFSET ({kind} elements), alone in its workbook on the t2816 data plus "
                "A8 = -3 (docs/research/t3533/mkcases.py)."
            ),
            "tags": ["M6", "probe", "offset"],
            "sheets": [{"name": "Sheet1", "cells": {**DATA, "E1": f"={fn}({arg})"}}],
            "expected": {},
        }
        slug = fn.lower().replace(".", "")
        (DIR / f"t3533-lift-{slug}-{kind}.case.json").write_text(json.dumps(case, indent=1) + "\n")
        n += 1
print(n, "cases")
