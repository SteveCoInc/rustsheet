"""T-3886: rank 96 arctangent/arcsine variants of BETA.DIST at a = b = 1/2 with x87 (64-bit) or double (53-bit)
intermediates against Excel's three sweeps, plus 11 double-only closed forms.

Each step is computed exactly (mpmath, 200 bits) and rounded to p bits, p in {53, 64} per step: the ratio t
(sqrt(x)/sqrt(y), sqrt(x/y), or sqrt(x) for asin), the angle a (atan(t) or asin), the result's arithmetic
(2a/pi, a/(pi/2), a*(2/pi), a*(1/pi)*2) and pi itself; y = fl(1 - x); the result is stored to a double.
Usage (repo root): tools/.venv/bin/python docs/research/t3886/score_half_x87.py
"""
import itertools
import json
import math
from pathlib import Path

import mpmath
from mpmath import mp, mpf

mp.prec = 200
BASE = "eval/suites/probes-numeric/cases/t3192-beta-half-half"


def load(path):
    path = Path(path)
    case = json.loads(path.with_suffix(".case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads(path.with_suffix(".oracle.json").read_text())["sheets"]["Calc"]
    rows, i = [], 1
    while f"A{i}" in case:
        rows.append((case[f"A{i}"], oracle.get(f"B{i}", {}).get("v"), oracle.get(f"D{i}", {}).get("v")))
        i += 1
    return rows


def rnd(v, p):
    return mpf(mpmath.libmp.mpf_pos(mpf(v)._mpf_, p, "n"))


def variant(x, p_t, p_a, p_r, p_pi, tform, rform):
    big_x = mpf(x)
    y = rnd(1 - big_x, 53)
    if tform == "sqrt(x)/sqrt(y)":
        a = rnd(mp.atan(rnd(rnd(mp.sqrt(big_x), p_t) / rnd(mp.sqrt(y), p_t), p_t)), p_a)
    elif tform == "sqrt(x/y)":
        a = rnd(mp.atan(rnd(mp.sqrt(rnd(big_x / y, p_t)), p_t)), p_a)
    else:  # asin(sqrt(x))
        a = rnd(mp.asin(rnd(mp.sqrt(big_x), p_t)), p_a)
    pi = rnd(mp.pi, p_pi)
    r = {
        "2a/pi": lambda: rnd(rnd(2 * a, p_r) / pi, p_r),
        "a/(pi/2)": lambda: rnd(a / rnd(pi / 2, p_r), p_r),
        "a*(2/pi)": lambda: rnd(a * rnd(2 / pi, p_r), p_r),
        "a*(1/pi)*2": lambda: rnd(2 * rnd(a * rnd(1 / pi, p_r), p_r), p_r),
    }[rform]()
    return float(rnd(r, 53))


sq = math.sqrt
DOUBLE_FORMS = {
    "direct 2*atan(sqrt x/sqrt y)/pi (ported)": lambda x: 2 * math.atan(sq(x) / sq(1 - x)) / math.pi,
    "2*atan(sqrt(x/y))/pi": lambda x: 2 * math.atan(sq(x / (1 - x))) / math.pi,
    "2*atan2(sqrt x, sqrt y)/pi": lambda x: 2 * math.atan2(sq(x), sq(1 - x)) / math.pi,
    "atan(..)/(pi/2)": lambda x: math.atan(sq(x) / sq(1 - x)) / (math.pi / 2),
    "atan(..)*(2/pi)": lambda x: math.atan(sq(x) / sq(1 - x)) * (2 / math.pi),
    "2*atan(..)*(1/pi)": lambda x: 2 * math.atan(sq(x) / sq(1 - x)) * (1 / math.pi),
    "2*asin(sqrt x)/pi": lambda x: 2 * math.asin(sq(x)) / math.pi,
    "1-2*acos(sqrt x)/pi": lambda x: 1 - 2 * math.acos(sq(x)) / math.pi,
    "acos(1-2x)/pi": lambda x: math.acos(1 - 2 * x) / math.pi,
    "0.5+asin(2x-1)/pi": lambda x: 0.5 + math.asin(2 * x - 1) / math.pi,
}


def main():
    sets = [load(BASE + s) for s in ("", "-2", "-3")]
    direct = DOUBLE_FORMS["direct 2*atan(sqrt x/sqrt y)/pi (ported)"]
    print(f"sweep 1: Excel's worksheet D column equals the direct form in doubles on "
          f"{sum(direct(x) == d for x, _, d in sets[0])} of {len(sets[0])} rows")
    print("double forms, bit-exact on sweeps 1 / 2 / 3:")
    for name, f in DOUBLE_FORMS.items():
        print(f"  {name:42s} " + " / ".join(str(sum(f(x) == b for x, b, _ in rows)) for rows in sets))
    results = []
    for p_t, p_a, p_r, p_pi in itertools.product((53, 64), repeat=4):
        for tform in ("sqrt(x)/sqrt(y)", "sqrt(x/y)", "asin(sqrt x)"):
            for rform in ("2a/pi", "a/(pi/2)", "a*(2/pi)", "a*(1/pi)*2"):
                scores = [sum(variant(x, p_t, p_a, p_r, p_pi, tform, rform) == b for x, b, _ in rows) for rows in sets]
                results.append((sum(scores), scores, (p_t, p_a, p_r, p_pi), tform, rform))
    results.sort(key=lambda r: -r[0])
    print(f"{len(results)} precision variants; the best 10 (total, per sweep, bits of t/a/result/pi):")
    for total, scores, bits, tform, rform in results[:10]:
        print(f"  {total} {scores} {bits} {tform} {rform}")
    half = [r for r in results if variant(0.5, *r[2], r[3], r[4]) != 0.5]
    print(f"variants giving 0.5000000000000001 at x = 0.5: {len(half)}; the best 3:")
    for total, scores, bits, tform, rform in half[:3]:
        print(f"  {total} {scores} {bits} {tform} {rform}")


main()
