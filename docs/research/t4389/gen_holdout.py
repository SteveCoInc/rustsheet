"""T-4389 hold-out probe: python3 gen_holdout.py CASE_PATH META_PATH (seed 43890).

Generated after the candidate in common.py was frozen and before it was scored on any cell of this case. Margin
(stated before scoring): the candidate's hold-out exact rate must be within 10 points of its train rate (the 538
`.000` cells it was fitted on, 100%) and above the correctly rounded value's and T-4370's.
Blocks, each cell `TEXT(k/86400+n*2^(e),fmt)` with the addition exact:
(a) 40 ties of `.000` below 1 s (k = j/1000 - 0.0005, j log-uniform in 1..99), n stepping the remainder from 0 to
    about 4.5E-17 s below the tie in 8 steps;
(b) ties at 1 to 59 whole seconds and at whole minutes, n = -6..1, keeping the cells where adding C in seconds and
    adding 2^-72 to the serial in days disagree (the seconds-or-days question), up to 60;
(c) 20 `.00` and `.0` ties below 0.06 s stepped the same way as (a): the scale control.
"""
import json
import math
import random
import sys

from common import base_digits, days_digits, seconds_digits

case_path, meta_path = sys.argv[1:3]
random.seed(43890)
cells, meta = {}, {}
FMTS = {1000: ("[h]:mm:ss.000", "s.000", "mm:ss.000"), 100: ("s.00", "[s].00"), 10: ("[s].0", "s.0")}


def add(k, s, n, f):
    x = k / 86400
    e = math.frexp(math.ulp(x))[1] - 1
    y = x + n * 2.0 ** e
    if (y - x) != n * 2.0 ** e or f"{k}" != repr(k):
        return False
    r = len(cells) + 1
    off = "" if n == 0 else f"+({n})*2^({e})"
    cells[f"A{r}"] = f'=TEXT({k}/86400{off},"{f}")'
    meta[f"A{r}"] = [f, s, k, n, y.hex()]
    return True


def stepped(k, s, f, steps=8, depth=4.5e-17):
    x = k / 86400
    ulp_s = math.ulp(x) * 86400
    top = max(1, round(depth / ulp_s))
    ns = sorted({-round(top * i / (steps - 1)) for i in range(steps)})
    for n in ns:
        add(k, s, n, f)


seen = set()
while len(seen) < 40:
    j = int(round(math.exp(random.uniform(0, math.log(99)))))
    if j in seen:
        continue
    seen.add(j)
    k = float(f"{j / 1000 - 0.0005:.4f}")
    stepped(k, 1000, random.choice(FMTS[1000]))

found = 0
for _ in range(4000):
    if found >= 60:
        break
    w = random.choice([random.randint(1, 59), 60 * random.randint(1, 59) + random.randint(0, 59)])
    j = random.randint(1, 99)
    k = float(f"{w + j / 1000 - 0.0005:.4f}")
    for n in range(-6, 2):
        x = k / 86400
        e = math.frexp(math.ulp(x))[1] - 1
        y = x + n * 2.0 ** e
        if days_digits(y, 1000) != seconds_digits(y, 1000) and found < 60:
            if add(k, 1000, n, "[h]:mm:ss.000"):
                found += 1

for _ in range(20):
    s = random.choice((100, 10))
    j = random.randint(1, 5 if s == 10 else 5)
    k = float(f"{j / s - 0.5 / s:.4f}")
    if k <= 0:
        continue
    stepped(k, s, random.choice(FMTS[s]), steps=4)

json.dump({"format": "rustsheet-case/v1",
           "description": "T-4389 hold-out: TEXT of .000 ties below 1 s stepped down to 4.5E-17 s below the tie, "
                          "ties after whole seconds that separate an offset in seconds from one in days, and .00/.0 "
                          "controls (generated after the candidate was frozen).",
           "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}},
          open(case_path, "w"), indent=1)
json.dump(meta, open(meta_path, "w"))
print(len(cells), "cells;", found, "seconds-vs-days cells")
