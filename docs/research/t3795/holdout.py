"""T-3795 R4.1 hold-out: the chosen models against the correctly rounded value, per function.

Population: every IMSQRT, IMLOG10, IMLOG2 and IMSECH cell of the eval suites whose argument is a
literal or COMPLEX(x, y) of literals and whose oracle value is text. Split fixed before scoring:
hold-out = sha1("<case>!<cell>") first byte % 10 < 3 (about 30%); margin 5 points of exact rate.
"Exact" is the 15-digit text (the only precision a text result carries). The engine's misses come
from `rustsheet-eval show` (one "<case>!<sheet>!<cell>" per line in FAILS).

Run (from tools/, mpmath): uv run python ../docs/research/t3795/holdout.py FAILS
"""
import glob
import hashlib
import json
import re
import sys

from mpmath import atan2, cos, cosh, log, mp, mpc, mpf, sech, sin, sqrt

mp.dps = 50
ROOT = "../eval/suites"
fails = set(line.strip() for line in open(sys.argv[1]))


def parse(t):
    t = t.strip()
    if not t.endswith(("i", "j")):
        return float(t), 0.0
    b = t[:-1]
    for i in range(len(b) - 1, 0, -1):
        if b[i] in "+-" and b[i - 1] not in "Ee":
            im = b[i:]
            return float(b[:i]), float(im + "1" if im in "+-" else im)
    im = b
    return 0.0, float(im + "1" if im in ("", "+", "-") else im)


def text(v):
    return float(f"{float(v):.15g}")


def truth(fn, x, y):
    x, y = mpf(x), mpf(y)
    if fn == "IMSQRT":
        r = sqrt(mpc(x, y))
        return r.real, r.imag
    if fn == "IMSECH":
        r = sech(mpc(x, y))
        return r.real, r.imag
    base = {"IMLOG10": log(10), "IMLOG2": log(2)}[fn]
    return log(sqrt(x * x + y * y)) / base, atan2(y, x) / base


pat = re.compile(r'^=(IMSQRT|IMLOG10|IMLOG2|IMSECH)\((?:COMPLEX\(([^,()]+),([^,()]+)\)|"([^"]*)")\)$')
stats = {}
for cf in sorted(glob.glob(ROOT + "/**/*.case.json", recursive=True)):
    of = cf.replace(".case.json", ".oracle.json")
    try:
        case, oracle = json.load(open(cf)), json.load(open(of))
    except (OSError, ValueError):
        continue
    cid = cf.split("suites/")[1].replace("/cases/", "/")
    for sheet in case.get("sheets", []):
        osh = oracle.get("sheets", {}).get(sheet["name"], {})
        for cell, formula in sheet.get("cells", {}).items():
            m = pat.match(formula) if isinstance(formula, str) else None
            if not m or osh.get(cell, {}).get("t") != "s":
                continue
            fn = m.group(1)
            try:
                if m.group(4) is not None:
                    x, y = (float(f"{v:.15g}") for v in parse(m.group(4)))
                else:
                    x, y = float(m.group(2)), float(m.group(3))
                er, ei = parse(osh[cell]["v"])
                tr, ti = truth(fn, x, y)
            except (ValueError, ZeroDivisionError):
                continue
            key = f"{cid}!{sheet['name']}!{cell}"
            part = "hold" if hashlib.sha1(f"{cid}!{cell}".encode()).digest()[0] % 10 < 3 else "train"
            s = stats.setdefault((fn, part), [0, 0, 0])
            s[0] += 1
            s[1] += key not in fails
            s[2] += text(tr) == er and text(ti) == ei
for (fn, part), (n, chosen, rounded) in sorted(stats.items()):
    print(f"{fn:8} {part:5} n={n:4}  chosen {chosen:4} ({chosen / n:.1%})  correctly rounded {rounded:4} ({rounded / n:.1%})")
