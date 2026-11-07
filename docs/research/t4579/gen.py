"""T-4579 probe generator: paired T.TEST, predictions per hypothesis (seed 4579_2026_11_07)."""
import json, math, random, sys
from fractions import Fraction as F
import mpmath as mp

mp.mp.dps = 60
rng = random.Random(4579_2026_11_07)


def r15(v: float) -> float:
    """The double Excel reads for a literal: 15 significant digits."""
    return float("%.15g" % v)


def lit(v: float) -> str:
    return "%.15g" % v


def arr(vs) -> str:
    return "{" + ",".join(lit(v) for v in vs) + "}"


def p_two(md, vd, n):
    """Two-tailed paired p from mean and variance of the differences, in 60 digits."""
    if vd <= 0:
        return "#DIV/0!"
    t2 = mp.mpf(md) ** 2 / (mp.mpf(vd) / n)
    df = n - 1
    return float(mp.betainc(df / 2, 0.5, 0, df / (df + t2), regularized=True))


def one_pass_var(d):
    n = len(d)
    s = 0.0
    q = 0.0
    for v in d:
        s += v
        q += v * v
    return (q - s * s / n) / (n - 1)


def two_pass_var(d):
    n = len(d)
    m = sum(d) / n
    return sum((v - m) ** 2 for v in d) / (n - 1)


def cand_double(x, y):
    d = [a - b for a, b in zip(x, y)]
    return p_two(sum(d) / len(d), two_pass_var(d), len(d))


def cand_exact(x, y):
    d = [F(a) - F(b) for a, b in zip(x, y)]
    n = len(d)
    m = sum(d) / n
    v = sum((e - m) ** 2 for e in d) / (n - 1)
    return p_two(mp.mpf(m.numerator) / m.denominator, mp.mpf(v.numerator) / v.denominator, n)


def cand_deviations(x, y):
    n = len(x)
    mx, my = sum(x) / n, sum(y) / n
    dev = [(a - mx) - (b - my) for a, b in zip(x, y)]
    return p_two(mx - my, sum(e * e for e in dev) / (n - 1), n)


rows_div, rows_off, preds = [], [], []
# Div: near-constant differences; classify by the one-pass variance of the double differences.
want = {"neg": 8, "zero": 8, "pos": 8, "const": 3}
got = {k: 0 for k in want}
while any(got[k] < want[k] for k in want):
    n = rng.choice([2, 3, 4])
    c = rng.uniform(1, 50)
    x = [r15(rng.uniform(-600, 600)) for _ in range(n)]
    y = [r15(xi - c) for xi in x]
    d = [a - b for a, b in zip(x, y)]
    if len(set(d)) == 1:
        kind = "const"
    else:
        v1 = one_pass_var(d)
        kind = "neg" if v1 < 0 else "zero" if v1 == 0 else "pos"
    if got[kind] < want[kind]:
        got[kind] += 1
        rows_div.append((x, y, kind, one_pass_var(d), two_pass_var(d)))
# Off: a shared large spread with a tiny paired effect, both samples offset.
for _ in range(24):
    n = rng.choice([4, 6, 8, 10])
    a = rng.choice([-1, 1]) * 10 ** rng.uniform(3, 6)
    b = rng.choice([-1, 1]) * 10 ** rng.uniform(3, 6)
    spread = 10 ** rng.uniform(0, 3)
    eps = 10 ** rng.uniform(-9, -6)
    shift = eps * rng.uniform(0.5, 3)
    u = [rng.uniform(-spread, spread) for _ in range(n)]
    x = [r15(a + ui) for ui in u]
    y = [r15(b + ui + shift + eps * rng.gauss(0, 1)) for ui in u]
    rows_off.append((x, y))

div_cells, off_cells = {}, {}
out = ["sheet\trow\tkind\tone_pass_var\ttwo_pass_var\tH1_onepass_le0\tH2_twopass_le0\tp_double\tp_exact\tp_deviations"]
for i, (x, y, kind, v1, v2) in enumerate(rows_div, 1):
    div_cells[f"A{i}"] = f"=T.TEST({arr(x)},{arr(y)},2,1)"
    out.append(f"Div\t{i}\t{kind}\t{v1!r}\t{v2!r}\t{'#DIV/0!' if v1 <= 0 else 'number'}\t{'#DIV/0!' if v2 <= 0 else 'number'}\t\t\t")
for i, (x, y) in enumerate(rows_off, 1):
    off_cells[f"A{i}"] = f"=T.TEST({arr(x)},{arr(y)},2,1)"
    out.append(f"Off\t{i}\t\t\t\t\t\t{cand_double(x, y)!r}\t{cand_exact(x, y)!r}\t{cand_deviations(x, y)!r}")
rev = {
    "A1": "=T.TEST({5},{3,3,3,3},2,3)",
    "A2": "=T.TEST({1E+200,1E+200,-1E+200},{1,2,4},1,2)",
    "A3": "=F.TEST({1E+200,1E+200,-1E+200},{1,2,4})",
}
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "Paired T.TEST (T-4579), predictions in docs/research/t4579/predictions.tsv, committed before the fill. "
        "Div: near-constant differences whose one-pass variance (Σd² - (Σd)²/n)/(n-1) of the double differences is "
        "negative, zero or tiny positive (and exactly constant controls): H1 #DIV/0! iff that one-pass variance <= 0, "
        "H2 iff the two-pass variance <= 0. Off: a shared spread with a tiny paired effect, both samples offset: "
        "double differences, exact differences and per-sample deviations give different p. Rev: the reversed "
        "argument orders of two T-4097 rules."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [
        {"name": "Div", "cells": div_cells},
        {"name": "Off", "cells": off_cells},
        {"name": "Rev", "cells": rev},
    ],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
open(sys.argv[2], "w").write("\n".join(out) + "\n")
