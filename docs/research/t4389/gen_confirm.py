"""T-4389 confirmation batch: python3 gen_confirm.py CASE_PATH META_PATH (seed 43892).

Generated after the excel-fidelity review of #3440. Two questions, and the prediction fixed before generation: the
ungated days model (common.py `days_digits` with no gate, the model scored on train, hold-out and fresh sweep).
(a) Up to 40 `.000` ties with a half-stepped serial in [2^-19, 2^-18) days (0.165 to 0.33 s) at n = -18..-10 ulp,
    keeping the cells where the ungated model and the 2^-19 gate disagree.
(b) Up to 40 ties after whole seconds or minutes (a new seed) where the days and seconds candidates disagree.
"""
import json
import math
import random
import sys

from common import days_digits, gated_digits, seconds_digits

case_path, meta_path = sys.argv[1:3]
random.seed(43892)
cells, meta = {}, {}


def add(k, n, f):
    x = k / 86400
    e = math.frexp(math.ulp(x))[1] - 1
    y = x + n * 2.0 ** e
    if (y - x) != n * 2.0 ** e or repr(k) != f"{k}":
        return None
    r = len(cells) + 1
    off = "" if n == 0 else f"+({n})*2^({e})"
    cells[f"A{r}"] = f'=TEXT({k}/86400{off},"{f}")'
    meta[f"A{r}"] = [f, 1000, k, n, y.hex()]
    return y


def scan(pick_k, differ, limit, f, ns):
    found = 0
    for _ in range(20000):
        if found >= limit:
            return found
        k = pick_k()
        x = k / 86400
        e = math.frexp(math.ulp(x))[1] - 1
        for n in ns:
            y = x + n * 2.0 ** e
            if found < limit and differ(y) and add(k, n, f) is not None:
                found += 1
    return found


a = scan(lambda: float(f"{random.randint(166, 330) / 1000 - 0.0005:.4f}"),
         lambda y: days_digits(y, 1000) != gated_digits(y, 1000), 40, "s.000", range(-18, -9))
b = scan(lambda: float(f"{random.choice([random.randint(1, 59), 60 * random.randint(1, 59) + random.randint(0, 59)]) + random.randint(1, 99) / 1000 - 0.0005:.4f}"),
         lambda y: days_digits(y, 1000) != seconds_digits(y, 1000), 40, "[h]:mm:ss.000", range(-6, 2))
json.dump({"format": "rustsheet-case/v1",
           "description": "T-4389 confirmation: .000 ties between 0.165 and 0.33 s that separate a 2^-19-day gate "
                          "from none, and ties after whole seconds that separate an offset in days from one in "
                          "seconds (generated after the review of #3440).",
           "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}},
          open(case_path, "w"), indent=1)
json.dump(meta, open(meta_path, "w"))
print(len(cells), "cells:", a, "band,", b, "seconds-vs-days")
