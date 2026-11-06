"""T-4332's frozen model of the clock and fraction digits of a second (frozen 2026-11-05, before the fresh sweep).

The half step is added to the serial as in T-4323 (`y = x + 0.5/(86400*s)`), but the units within the day are cut
from `frac(y)` one field at a time, hours, then minutes, then seconds, then the fraction: each stage multiplies the
remainder (24, 60, 60, s), floors it and keeps what is left, in plain doubles. Run from this directory:
`python3 chain.py` scores it against T-4323's register (`fu`) on the five t4323 cases.
"""
import json
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "../t4323"))
from frozen_fmt import fmt  # noqa: E402

FIELDS = (24, 60, 60)
CASES = [("../t4323/meta1.json", "t4323-fraction-ties"), ("../t4323/meta2.json", "t4323-fraction-sweep"),
         ("../t4323/meta3.json", "t4323-fraction-sweep-2"), ("../t4323/meta4.json", "t4323-fraction-sweep-3"),
         ("../t4323/meta5.json", "t4323-fraction-holdout")]


def load(pairs):
    """(case, cell, format, scale, day, seconds, Excel's text) of every cell; meta paths are relative to this file."""
    here = os.path.dirname(os.path.abspath(__file__))
    rows = []
    for meta_path, case in pairs:
        meta = json.load(open(os.path.join(here, meta_path)))
        oracle = json.load(open(os.path.join(here, f"../../../eval/suites/probes/cases/{case}.oracle.json")))
        cells = oracle["sheets"]["Sheet1"]
        for c, (f, s, d, k) in meta.items():
            rows.append((case, c, f, s, d, k, cells[c]["v"]))
    return rows


def rnd(v):
    """f64::round on a non-negative double: half away from zero."""
    r = math.floor(v)
    return r + 1 if v - r >= 0.5 else r


def chain_units(x, s):
    """Units of 1/s second within the day of `x + 0.5/(86400*s)`, cut field by field."""
    v = x + 0.5 / (86400 * s)
    v -= math.floor(v)
    units = 0
    for f in FIELDS + (s,):
        v *= f
        w = math.floor(v)
        units = units * f + w
        v -= w
    return units


def register_units(x, s):
    """T-4323's register: floor(frac(y)*86400*s)."""
    y = x + 0.5 / (86400 * s)
    return math.floor((y - math.floor(y)) * 86400 * s)


def as_fields(units_fn):
    def m(x, s):
        u = units_fn(x, s)
        return u % s, u // s, rnd(x * 86400 * s) // s
    return m


def score(rows, units_fn):
    out = {"d0": [0, 0], "d>=1": [0, 0]}
    for (_, _, f, s, d, k, e) in rows:
        x = (d + k / 86400) if d else k / 86400
        g = "d0" if d == 0 else "d>=1"
        out[g][1] += 1
        out[g][0] += fmt(f, s, as_fields(units_fn)(x, s)) == e
    return out


if __name__ == "__main__":
    pairs = [tuple(a.split(":")) for a in sys.argv[1:]] or CASES
    rows = load(pairs)
    for name, fn in (("chain", chain_units), ("register (T-4323)", register_units)):
        print(f"{name:18} {score(rows, fn)}")
