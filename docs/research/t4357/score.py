"""T-4357: POISSON.DIST's cdf companions below 1.3a (probes-numeric/t4357-poisson-cdf-companions), scored.

Run from the repository root with the tools venv (mpmath):
    tools/.venv/bin/python docs/research/t4357/score.py

Rows 1:9 are T-4345's train points and rows 10:18 its hold-out (split fixed before scoring, decision 0188 R4).
Part 1, per row, in ulp of the column: the cdf A against the correctly rounded Q; whether CHISQ.DIST.RT E is A bit
for bit; 1 - P (H) against A; P + Q - 1 in ulp of 1; the pmf B, GAMMA.DIST's P (C) and density (D) against the
truth; ERFC.PRECISE(SQRT(B))/2 (F) against the correctly rounded erfc term; GAMMALN.PRECISE (G) against the truth.
Part 2: Excel's pmf and density times the exact ratio to Q, bit-exact / within 4 ulp / max on train and hold-out.
Part 3: the x' within which Excel's cdf is Q(a, x'), in ulp of x, and Q's sensitivity to one ulp of x.
"""

import json
import math
import re
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
M = mp.mpf
CASE = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t4357-poisson-cdf-companions"


def ulps(v, ref):
    return (v - ref) / math.ulp(ref)


def rows():
    cells = json.loads(CASE.with_suffix(".case.json").read_text())["sheets"][0]["cells"]
    orc = json.loads(CASE.with_suffix(".oracle.json").read_text())["sheets"]["Calc"]
    for i in range(1, 19):
        k, x = re.match(r"=POISSON.DIST\((\d+),([\d.]+),TRUE\)", cells[f"A{i}"]).groups()
        yield i, int(k), float(x), {c: orc[f"{c}{i}"]["v"] for c in "ABCDEFGH"}


def truth(k, x):
    a = M(k + 1)
    q = mp.gammainc(a, M(x), mp.inf, regularized=True)
    pmf = mp.exp(-M(x)) * M(x) ** k / mp.factorial(k)
    dens = mp.exp(-M(x)) * M(x) ** k / mp.gamma(a)
    z = mp.sqrt(a * mp.log(a / M(x)) + M(x) - a)
    return q, pmf, dens, mp.erfc(z) / 2, mp.loggamma(a)


def main():
    print("1. row k x/a | A-crQ | E==A | H vs A | P+Q-1 | B pmf | C P | D dens | F erfc term | G")
    scores = {"Excel pmf x exact Q/pmf": [], "Excel density x exact Q/density": []}
    for i, k, x, v in rows():
        q, pmf, dens, half, lg = truth(k, x)
        print(f"   {i:2} {k:3} {x / (k + 1):.4f} | {ulps(float(q), v['A']):4.0f} | {v['A'] == v['E']!s:5} | "
              f"{ulps(v['H'], v['A']):7.0f} | {(v['A'] + v['C'] - 1) / 2**-53:4.1f} | {ulps(float(pmf), v['B']):4.0f} | "
              f"{ulps(float(1 - q), v['C']):3.0f} | {ulps(float(dens), v['D']):4.0f} | {ulps(float(half), v['F']):5.0f} | "
              f"{ulps(float(lg), v['G']):3.0f}")
        scores["Excel pmf x exact Q/pmf"].append(abs(ulps(v["B"] * float(q / pmf), v["A"])))
        scores["Excel density x exact Q/density"].append(abs(ulps(v["D"] * float(q / dens), v["A"])))
    print("2. bit-exact / within 4 ulp / max, train (rows 1:9) and hold-out (rows 10:18)")
    for name, u in scores.items():
        print("   " + f"{name:32}" + "".join(
            f"  {s} {sum(e <= 0.5 for e in p)}/{sum(e <= 4 for e in p)}/{max(p):.0f}"
            for s, p in (("train", u[:9]), ("hold-out", u[9:]))))
    print("3. implied x' - x in ulp of x, and ulp of Q per ulp of x")
    for i, k, x, v in rows():
        q, _, dens, _, _ = truth(k, x)
        dq = -dens
        print(f"   {i:2} k {k:3}  {float((M(v['A']) - q) / dq) / math.ulp(x):6.3f}  {float(dq * math.ulp(x)) / math.ulp(v['A']):7.1f}")


if __name__ == "__main__":
    main()
