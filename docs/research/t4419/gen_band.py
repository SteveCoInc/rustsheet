"""T-4419 dense band probe: python3 gen_band.py CASE_PATH META_PATH.

Every `.000` tie k = 0.1655, 0.1675, ..., 0.3295 s (83 ties, step 2 ms) whose half-stepped serial lies in
[2^-19, 2^-18) days, each stepped n = -19..-9 ulp of k/86400 below it (913 cells), under "s.000". No cell is
selected by any model: the probe maps, per tie, the ulp step where Excel's digit flips. The prediction fixed before the
fill is the engine's (T-4389's ungated days model, common.py `days_digits`), stored in the meta as the fifth item.
"""
import json
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "../t4389"))
from common import days_digits  # noqa: E402

case_path, meta_path = sys.argv[1:3]
cells, meta = {}, {}
row = 0
for i in range(83):
    k = float(f"{0.1655 + 0.002 * i:.4f}")
    x = k / 86400
    e = math.frexp(math.ulp(x))[1] - 1
    for n in range(-19, -8):
        y = x + n * 2.0 ** e
        assert y - x == n * 2.0 ** e
        row += 1
        c = f"A{row}"
        cells[c] = f'=TEXT({k}/86400+({n})*2^({e}),"s.000")'
        meta[c] = ["s.000", 1000, k, n, y.hex(), f"0.{days_digits(y, 1000):03d}"]
json.dump({"format": "rustsheet-case/v1",
           "description": "T-4419: .000 ties between 0.165 and 0.33 s (half-stepped serial in [2^-19, 2^-18) days), "
                          "every 2 ms, each stepped 19 to 9 ulp below the tie, to map where Excel's digit flips.",
           "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}},
          open(case_path, "w"), indent=1)
json.dump(meta, open(meta_path, "w"))
print(len(cells), "cells")
