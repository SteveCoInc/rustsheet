"""Build probes/t4466-general-fixed-or-e: TEXT(v,"General") on values in [1E-09, 1E-04) where General's old fixed-or-E
rule (compare the digits left after rounding trims zeros) and T-4333's rule (credit each form with the digits it has
room for, capped by the value's 15 digits) disagree, plus controls where they agree: python3 gen_general_fresh.py
CASE_PATH. Writes general_predictions.json (old, new) before the fill. random.Random(44664).
"""

import json
import math
import os
import random
import sys
from decimal import ROUND_HALF_UP, Decimal

HERE = os.path.dirname(os.path.abspath(__file__))


def sig15(v):
    d = Decimal(v)
    return d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_HALF_UP)


def round_half_away(v, decimals):
    """The engine's round_decimal(HalfAwayFromZero): the 15-digit decimal rounded half away from zero."""
    q = sig15(v).quantize(Decimal(1).scaleb(-decimals), rounding=ROUND_HALF_UP)
    return float(q)


def sig_digits(text):
    return len(text.replace("-", "").replace(".", "").lstrip("0"))


def scientific(v):
    e = math.floor(math.log10(abs(v)))
    dec = 4 if abs(e) >= 99 else 5
    m = f"{v / 10.0**e:.{dec}f}"
    if m.lstrip("-").startswith("10."):
        e += 1
        dec = 4 if abs(e) >= 99 else 5
        m = f"{math.copysign(1.0, v):.{dec}f}"
    return m.rstrip("0").rstrip("."), dec + 1, e


def general(v, rule):
    mant, room, e = scientific(v)
    sci = f"{mant}E{'-' if e < 0 else '+'}{abs(e):02d}"
    decimals = 9
    r = round_half_away(v, decimals)
    untrimmed = f"{r:.{decimals}f}"
    fixed = untrimmed.rstrip("0").rstrip(".")
    if rule == "old":
        ok = sig_digits(fixed) >= sig_digits(mant)
    else:
        digits = len(str(sig15(v)).replace(".", "").split("E")[0].lstrip("0").rstrip("0")) or 1
        ok = min(sig_digits(untrimmed), digits) >= min(room, digits)
    return fixed if ok else sci


rng = random.Random(44664)
disagree, agree = [], []
while len(disagree) < 80 or len(agree) < 40:
    v = 10 ** rng.uniform(-9, -4)
    v = float(f"{v:.{rng.randint(1, 15)}g}")
    if not 1e-9 <= v < 1e-4:
        continue
    old, new = general(v, "old"), general(v, "new")
    (disagree if old != new else agree).append((v, old, new))
picked = disagree[:80] + agree[:40]
cells, preds = {}, {}
for i, (v, old, new) in enumerate(picked, 1):
    cells[f"A{i}"] = f'=TEXT({v!r},"General")'
    preds[f"A{i}"] = {"old": old, "new": new}
case = {
    "format": "rustsheet-case/v1",
    "description": "T-4333: TEXT(v,\"General\") in [1E-09, 1E-04): A1:A80 where General's old fixed-or-E rule and the "
    "room-based rule disagree, A81:A120 controls where they agree (docs/research/t4466/gen_general_fresh.py; "
    "predictions in general_predictions.json, committed before the fill).",
    "tags": ["M10", "text", "probe"],
    "sheets": [{"name": "Sheet1", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as fh:
    fh.write(json.dumps(case, indent=2) + "\n")
with open(os.path.join(HERE, "general_predictions.json"), "w") as fh:
    json.dump(preds, fh, indent=0, sort_keys=True)
print(len(cells), "cells")
