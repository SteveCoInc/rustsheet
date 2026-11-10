"""Build probes/t4466-register-fresh, the fresh sweep for the `General` register beside fraction digits, drawn after
the field-cut register (elapsed.rs `date_section_register`) was frozen (2026-11-08, decision 0188 R4.2):
python3 gen_register_fresh.py CASE_PATH.

120 cells drawn with random.Random(44662): a day from a log-spread list, a second of the day (a decimal tie k.f5 at
the shown digit one time in two, else a random decimal with four places), the double nearest it or one ulp either
side, under one of six date-section formats with `.0`, `.00` or `.000` followed by ` General`. The engine's strings
were recorded before the fill in register_fresh_predictions.json (`rustsheet-eval run` on the case with placeholder
expectations, the `actual` of each cell); score_register_fresh.py scores them and the serial cut at once.
"""

import json
import math
import random
import sys

DAYS = [0, 0, 0, 1, 2, 9, 31, 366, 1000, 9999, 45001, 60000, 2958000]
FORMATS = [
    "[s].0 General",
    "[s].00 General",
    "[s].000 General",
    "s.0 General",
    "mm:ss.00 General",
    "[h]:mm:ss.000 General",
]
rng = random.Random(44662)
cells = {}
for i in range(1, 121):
    d = rng.choice(DAYS)
    f = rng.choice(FORMATS)
    digits = len(f.split(".")[1].split(" ")[0])
    k = rng.randrange(0, 86399)
    if rng.random() < 0.5:
        secs = f"{k}.{rng.randrange(0, 10 ** digits):0{digits}d}5"
    else:
        secs = f"{k}.{rng.randrange(0, 10000):04d}"
    base = (d + float(secs) / 86400) if d else float(secs) / 86400
    e = round(math.log2(math.ulp(base)))
    off = rng.choice([0, 0, -1, 1])
    expr = (f"{d}+{secs}/86400" if d else f"{secs}/86400") + ("" if off == 0 else f"{off:+d}*2^{e}")
    cells[f"A{i}"] = f'=TEXT({expr},"{f}")'
case = {
    "format": "rustsheet-case/v1",
    "description": "T-4333: fresh sweep of the General register beside fraction digits in a date section, drawn "
    "after the field-cut register was frozen (docs/research/t4466/gen_register_fresh.py).",
    "tags": ["M10", "text", "probe"],
    "sheets": [{"name": "Sheet1", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as fh:
    fh.write(json.dumps(case, indent=2) + "\n")
print(len(cells), "cells")
