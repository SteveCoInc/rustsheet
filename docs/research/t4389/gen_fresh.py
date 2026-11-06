"""T-4389 fresh sweep: python3 gen_fresh.py CASE_PATH META_PATH (seed 43891).

Generated after the hold-out chose the days variant and it was frozen (common.py `days_digits`: 2^-72 added to the
serial after the `.000` half step, before the cut). 120 cells of `TEXT(k/86400+n*2^(e),fmt)`: `.000` ties with k
log-uniform from 1E-4 to 60 s and n uniform in -64..4. The first 60 are cells where the frozen model and T-4370
disagree; the other 60 are drawn without that filter.
"""
import json
import math
import random
import sys

from common import base_digits, days_digits

case_path, meta_path = sys.argv[1:3]
random.seed(43891)
FMTS = ("[h]:mm:ss.000", "s.000", "mm:ss.000", "h:mm:ss.000")
cells, meta = {}, {}


def draw():
    while True:
        sec = 10 ** random.uniform(-4, math.log10(60))
        k = float("%.7g" % ((math.floor(sec * 1000) + 0.5) / 1000))
        x = k / 86400
        e = math.frexp(math.ulp(x))[1] - 1
        n = random.randint(-64, 4)
        y = x + n * 2.0 ** e
        if (y - x) == n * 2.0 ** e and repr(k) == f"{k}":
            return k, n, e, y


def add(k, n, e, y):
    r = len(cells) + 1
    f = random.choice(FMTS)
    off = "" if n == 0 else f"+({n})*2^({e})"
    cells[f"A{r}"] = f'=TEXT({k}/86400{off},"{f}")'
    meta[f"A{r}"] = [f, 1000, k, n, y.hex()]


while len(cells) < 60:
    k, n, e, y = draw()
    if days_digits(y, 1000) != base_digits(y, 1000):
        add(k, n, e, y)
while len(cells) < 120:
    add(*draw())
json.dump({"format": "rustsheet-case/v1",
           "description": "T-4389 fresh sweep: TEXT of .000 ties from 1E-4 to 60 s moved -64..4 ulp, drawn after "
                          "the days-offset model was frozen (the first 60 separate it from T-4370's).",
           "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}},
          open(case_path, "w"), indent=1)
json.dump(meta, open(meta_path, "w"))
print(len(cells), "cells")
