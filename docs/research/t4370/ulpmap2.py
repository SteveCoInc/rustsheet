"""T-4370 ulp map 2: python3 ulpmap2.py CASE_PATH META_PATH (seed 43702).

Three blocks of TEXT of a serial moved n units in the last place from a fraction-of-a-second tie below serial 1
(`k/86400+n*2^(e)`, the addition exact): (a) ties under 0.01 s at n = -64..0 in steps of 4, where ulpmap.py saw no flip
down to n = -8; (b) one serial each from the 1.45 and 2.05 ties at n = -3..1 under several formats, to see whether the
flip depends on the format; (c) 30 random ties (log-uniform 1E-3 to 60 s) at n = -6..3.
"""
import json
import math
import random
import sys

FMT = {10: "[s].0", 100: "s.00", 1000: "[h]:mm:ss.000"}
case_path, meta_path = sys.argv[1:3]
random.seed(43702)
cells, meta = {}, {}


def add(k, s, n, f):
    x = k / 86400
    e = math.frexp(math.ulp(x))[1] - 1
    y = x + n * 2.0 ** e
    if (y - x) != n * 2.0 ** e:
        return
    r = len(cells) + 1
    off = "" if n == 0 else f"+({n})*2^({e})"
    cells[f"A{r}"] = f'=TEXT({k}/86400{off},"{f}")'
    meta[f"A{r}"] = [f, s, k, n, y.hex()]


for k, s in [(0.0005, 1000), (0.0015, 1000), (0.0075, 1000), (0.0025, 1000), (0.0045, 1000), (0.005, 100)]:
    for n in range(-64, 1, 4):
        add(k, s, n, FMT[s])
for k in (1.45, 2.05):
    for f in ("[s].0", "h:mm:ss.0", "s.0", "mm:ss.0", "[h]:mm:ss.0", "[m]:ss.0"):
        for n in range(-3, 2):
            add(k, 10, n, f)
for _ in range(30):
    s = random.choice((10, 100, 1000))
    sec = 10 ** random.uniform(-3, math.log10(59))
    k = float("%.7g" % ((math.floor(sec * s) + 0.5) / s))
    for n in range(-6, 4):
        add(k, s, n, FMT[s])
json.dump({"format": "rustsheet-case/v1",
           "description": "T-4370 ulp map 2: TEXT of fraction-of-a-second ties below serial 1 moved n units in the "
                          "last place: wide offsets under 0.01 s, the same serial under several formats, and 30 "
                          "random ties.",
           "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}},
          open(case_path, "w"), indent=1)
json.dump(meta, open(meta_path, "w"))
print(len(cells), "cells")
