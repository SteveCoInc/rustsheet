"""Predictions for probes/t4466-count-hypotheses, written before the fill (R4): python3 predict.py.

Writes predictions.json: for every cell whose format is an elapsed count before a fraction, the count each model gives.
- engine: the half-step register `floor((x + 0.5/(86400*s))*86400/u)` (FractionalSeconds, formatter/elapsed.rs);
- h1: T-4333's first hypothesis. At the double nearest the decimal tie d + k/86400 the count is the decimal value
  rounded half up to the shown digit, truncated to the unit; elsewhere the register;
- h2: T-4333's second hypothesis. The count is the bare `[s]` token's whole-second count, `round(x*86400)`, less one
  when the shown fraction digits (T-4332's field cut) are at least half a second, divided down to the unit.
"""

import json
import math
import os
import re
from decimal import ROUND_HALF_UP, Decimal
from fractions import Fraction

HERE = os.path.dirname(os.path.abspath(__file__))
CASE = os.path.join(HERE, "../../../eval/suites/probes/cases/t4466-count-hypotheses.case.json")


def cut_fraction(x, s):
    """T-4332's field-by-field cut of the half-step serial: the shown fraction digits (before T-4370's 15-digit
    rounding, which only matters below serial 1)."""
    v = x + 0.5 / (86400 * s)
    v -= math.floor(v)
    for f in (24, 60, 60, s):
        v *= f
        w = math.floor(v)
        v -= w
    return w


def models(d, k, off, e, s, u):
    x = (d + float(k) / 86400) if d else float(k) / 86400
    x = x + off * 2.0**e if off else x
    tie = Fraction(d) + Fraction(k) / 86400
    engine = math.floor((x + 0.5 / (86400 * s)) * 86400 / u)
    if x == float(tie):
        secs = (Decimal(d) * 86400 + Decimal(k)).quantize(Decimal(1) / s, rounding=ROUND_HALF_UP)
        h1 = int(secs // u)
    else:
        h1 = engine
    whole = math.floor(x * 86400 + 0.5)
    h2 = (whole - (1 if cut_fraction(x, s) * 2 >= s else 0)) // u
    return {"engine": engine, "h1": h1, "h2": h2}


if __name__ == "__main__":
    meta = json.load(open(os.path.join(HERE, "meta.json")))
    cells = json.load(open(CASE))["sheets"][0]["cells"]
    out = {}
    for cell, formula in cells.items():
        f = re.search(r',"([^"]+)"\)$', formula).group(1)
        m = re.match(r"\[(h|m|s)\][^ ]*?\.(0+)", f)
        if not m:
            continue
        u = {"h": 3600, "m": 60, "s": 1}[m.group(1)]
        d, k, off, e = meta[re.sub(r"^[A-Z]+", "", cell)]
        out[cell] = models(d, k, off, e, 10 ** len(m.group(2)), u)
    json.dump(out, open(os.path.join(HERE, "predictions.json"), "w"), indent=0, sort_keys=True)
    print(len(out), "predicted cells")
