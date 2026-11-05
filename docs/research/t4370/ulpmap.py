"""T-4370 ulp map: python3 ulpmap.py CASE_PATH META_PATH.

For each chosen rounding tie of the shown fraction of a second at a serial below 1, the cells print TEXT of the serial
`k/86400` moved by n units in the last place (n = -8..8, written `k/86400+n*2^(e)` with the addition exact), so the
oracle shows where between neighbouring doubles Excel's rounding flips. The ties mix ones Excel rounds up that
T-4332's chain truncates and ones Excel truncates (docs/research/t4332).
"""
import json
import math
import sys

FMT = {10: "[s].0", 100: "s.00", 1000: "[h]:mm:ss.000"}
TIES = [(1.45, 10), (3.55, 10), (1.15, 10), (0.0005, 1000), (0.0015, 1000), (0.0075, 1000), (4.305, 100),
        (0.055, 100), (1.115, 100), (20.65, 10), (7.55, 10), (52.85, 10), (2.6565, 1000), (9.35, 10), (2.05, 10),
        (0.05, 10), (0.75, 10), (12.75, 10), (0.0025, 1000), (3.045, 100)]
case_path, meta_path = sys.argv[1:3]
cells, meta, r = {}, {}, 1
for k, s in TIES:
    x = k / 86400
    e = math.frexp(math.ulp(x))[1] - 1
    for n in range(-8, 9):
        y = x + n * 2.0 ** e
        if (y - x) != n * 2.0 ** e:
            continue
        off = "" if n == 0 else f"+({n})*2^({e})"
        cells[f"A{r}"] = f'=TEXT({k}/86400{off},"{FMT[s]}")'
        meta[f"A{r}"] = [FMT[s], s, k, n, y.hex()]
        r += 1
json.dump({"format": "rustsheet-case/v1",
           "description": "T-4370 ulp map: TEXT of a fraction-of-a-second rounding tie at a serial below 1, the serial "
                          "moved -8..8 units in the last place, to place Excel's rounding flip between doubles.",
           "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}},
          open(case_path, "w"), indent=1)
json.dump(meta, open(meta_path, "w"))
print(r - 1, "cells")
