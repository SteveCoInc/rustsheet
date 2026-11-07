"""Build probes/t4333-count-ulp-scan: python3 ulp_scan.py CASE_PATH.

Each row steps one tie serial of the t4323 probes by -3..+3 of its own ulp under `[s].0`, to locate in ulps of x where
Excel's elapsed-seconds count flips. Seeds: four ties whose count Excel rounds up against the half-step register
(t4323-fraction-ties A90, -sweep-2 A1, A84, -carry-ties A31) and four it truncates (-sweep-2 A9, A19, -carry-sweep A49,
-carry-ties A15). `meta7.json` maps each cell to (day, seconds, ulp offset, ulp exponent).
"""
import json
import math
import sys

SEEDS = [(45000, "0.95"), (3, "5.95"), (1, "217.95"), (45000, "59.95"),
         (45000, "2740.95"), (0, "3733.95"), (60000, "82799.95"), (1, "59.95")]
OFFSETS = range(-3, 4)
cells, meta = {}, {}
for row, (d, k) in enumerate(SEEDS, start=1):
    x = (d + float(k) / 86400) if d else float(k) / 86400
    e = round(math.log2(math.ulp(x)))
    base = f"{d}+{k}/86400" if d else f"{k}/86400"
    for col, off in enumerate(OFFSETS):
        cell = f"{'ABCDEFG'[col]}{row}"
        step = "" if off == 0 else f"{off:+d}*2^{e}"
        cells[cell] = f'=TEXT({base}{step},"[s].0")'
        meta[cell] = [d, k, off, e]
case = {"format": "rustsheet-case/v1",
        "description": "T-4333: [s].0 on tie serials of the t4323 probes stepped -3..+3 ulp, to locate where Excel's "
                       "elapsed-seconds count flips (docs/research/t4333/ulp_scan.py).",
        "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}}
open(sys.argv[1], "w").write(json.dumps(case, indent=2) + "\n")
json.dump(meta, open("meta7.json", "w"), indent=1)
