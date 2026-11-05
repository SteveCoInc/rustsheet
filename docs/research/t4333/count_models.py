"""T-4333: score candidate models of TEXT's elapsed [h]/[m]/[s] count before a fraction of a second.

Run from this directory with plain `python3 count_models.py`. It reads every elapsed-count cell with fraction digits of
the seven t4323 cases and probes/t4332-fraction-fresh (534 cells), plus probes/t4333-count-ulp-scan (56 cells, built by
`ulp_scan.py`, mapped by `meta7.json`), and prints how many counts each candidate gets, with its first misses.
`x` is the serial, `s` the fraction scale (10, 100, 1000), `h = 0.5/(86400*s)` the half step, `y = x + h`.
"""
import json
import math
import os
import re
from decimal import Decimal, getcontext
from fractions import Fraction as F

getcontext().prec = 80
fl = math.floor
HERE = os.path.dirname(os.path.abspath(__file__))
CASES = os.path.join(HERE, "../../../eval/suites/probes/cases/")
NAMES = ["t4323-fraction-ties", "t4323-fraction-sweep", "t4323-fraction-sweep-2", "t4323-fraction-sweep-3",
         "t4323-fraction-holdout", "t4323-carry-ties", "t4323-carry-sweep", "t4332-fraction-fresh"]
FORMULA = re.compile(r'^=TEXT\((?:(\d+)\+)?([0-9.]+)/86400,"([^"]+)"\)$')
MULT = {1: 86400, 60: 1440, 3600: 24}


def rows():
    """(case, cell, unit, scale, x, Excel's count) of every elapsed-count cell with fraction digits."""
    out = []
    for name in NAMES:
        case = json.load(open(CASES + name + ".case.json"))
        oracle = json.load(open(CASES + name + ".oracle.json"))["sheets"]["Sheet1"]
        for cell, formula in case["sheets"][0]["cells"].items():
            m = FORMULA.match(formula)
            if not m or cell not in oracle or not m.group(3).startswith("["):
                continue
            digits = re.search(r"\.(0+)$", m.group(3))
            if not digits:
                continue
            d = int(m.group(1) or 0)
            k = float(m.group(2))
            x = (d + k / 86400) if d else k / 86400
            unit = {"h": 3600, "m": 60, "s": 1}[m.group(3)[1]]
            out.append((name, cell, unit, 10 ** len(digits.group(1)), x, int(re.match(r"\d+", oracle[cell]["v"]).group())))
    scan = json.load(open(CASES + "t4333-count-ulp-scan.oracle.json"))["sheets"]["Sheet1"]
    for cell, (d, k, off, e) in json.load(open(os.path.join(HERE, "meta7.json"))).items():
        x = ((d + float(k) / 86400) if d else float(k) / 86400) + off * 2.0 ** e
        out.append(("t4333-count-ulp-scan", cell, 1, 10, x, int(re.match(r"\d+", scan[cell]["v"]).group())))
    return out


def field_seconds(v):
    """Whole days and whole seconds within the day of `v`, cut field by field (24, 60, 60) as T-4332's clock."""
    days = fl(v)
    rest, secs = v - days, 0
    for f in (24, 60, 60):
        rest *= f
        w = fl(rest)
        secs = secs * f + w
        rest -= w
    return days, secs, rest


def snap(q, n):
    d = Decimal(q.numerator) / Decimal(q.denominator)
    return F(d.quantize(Decimal(1).scaleb(d.adjusted() - n + 1)))


def candidates(x, s, u):
    """Each candidate's count of `u` seconds."""
    h = 0.5 / (86400 * s)
    y = x + h
    dx, fx = fl(x), x - fl(x)
    dy, fy = fl(y), y - fl(y)
    out = {
        "register floor(y*86400/u) (engine)": fl(y * 86400 / u),
        "floor(y*mult[u])": fl(y * MULT[u]),
        "value rounded to the digit": fl(x * 86400 * s + 0.5) // s // u,
        "floor(x*86400 + 0.5/s)": fl(x * 86400 + 0.5 / s) // u,
        "days*86400 + floor(frac(x)*86400 + 0.5/s)": (dx * 86400 + fl(fx * 86400 + 0.5 / s)) // u,
        "days*86400 + floor(frac(y)*86400)": (dy * 86400 + fl(fy * 86400)) // u,
        "floor(days*86400 + (frac(x)*86400 + 0.5/s))": fl(dx * 86400 + (fx * 86400 + 0.5 / s)) // u,
        "floor(days*86400 + (frac(x)+h)*86400)": fl(dx * 86400 + (fx + h) * 86400) // u,
        "days*86400 + field-cut clock of y": (lambda d, c, _: (d * 86400 + c) // u)(*field_seconds(y)),
        "floor(y*S)//s": fl(y * 86400 * s) // s // u,
    }
    for n in (13, 15, 17):
        out[f"exact (x snapped to {n} digits + h)*86400"] = int((snap(F(x), n) + F(1, 2 * 86400 * s)) * 86400) // u
        out[f"y*86400 snapped to {n} digits"] = int(snap(F(y) * 86400, n)) // u
    out["exact (x + h)*86400"] = int((F(x) + F(1, 2 * 86400 * s)) * 86400) // u
    return out


if __name__ == "__main__":
    data = rows()
    scored = {}
    for case, cell, u, s, x, count in data:
        for name, got in candidates(x, s, u).items():
            hit, miss = scored.setdefault(name, [0, []])
            if got == count:
                scored[name][0] += 1
            else:
                miss.append(f"{case[6:]} {cell}")
    for name, (hit, miss) in sorted(scored.items(), key=lambda kv: -kv[1][0]):
        print(f"{hit:4}/{len(data)}  {name:48} {', '.join(miss[:6])}")
