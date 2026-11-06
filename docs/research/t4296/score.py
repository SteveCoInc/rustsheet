"""T-4296: read probes/t4296-gamma-density-at-mode (Excel's large-shape gamma density at x = a).

Usage: tools/.venv/bin/python docs/research/t4296/score.py  (needs mpmath; reads the case and its oracle sidecar)

At x = a the exponent is 0, so each candidate is a prefactor chain alone: an order of the factors
sqrt(a / 2 pi), 1 / x and exp(-c(a)), with c(a) = ln Gamma(a) - Stirling taken several ways. Each
chain is evaluated in doubles and scored as ulps from Excel: the count at 0 ulp, then within 1 ulp,
on the fit (column A) and the hold-out (column B, fixed before scoring). Column C (scale 3, x = 3a)
is compared with column A / 3.

Part 2 takes every probed large-shape GAMMA.DIST density inside the window (|x - a| < 0.2 (x + a)) in
probes and probes-numeric and counts, per case, the cells bit-exact and within 15 digits of Excel for
the old chain sqrt(a/2pi) exp(-B - c5(a)) / x and the window chain exp(-c(a) - B) / sqrt(2pi a) (a/x),
both on the correctly rounded B = a ln(a/y) + y - a, so only the chain differs (c5: the 5-term series).
"""

import glob
import json
import math
import re
from decimal import ROUND_DOWN, Decimal
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
M = mp.mpf
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases"
STEM = "t4296-gamma-density-at-mode"
CELLS = json.loads((CASES / f"{STEM}.case.json").read_text())["sheets"][0]["cells"]
ORACLE = json.loads((CASES / f"{STEM}.oracle.json").read_text())["sheets"]["Calc"]
TWO_PI = 2 * math.pi
INV_SQRT_2PI = float(1 / mp.sqrt(2 * mp.pi))
LN_SQRT_2PI = float(mp.log(2 * mp.pi) / 2)
# Stirling-series coefficients 1/12, 1/360, 1/1260, 1/1680, 1/1188 (Loader's S0..S4).
S = [1 / 12, 1 / 360, 1 / 1260, 1 / 1680, 1 / 1188]


def corr_exact(a):
    A = M(a)
    return float(mp.loggamma(A) - ((A - 0.5) * mp.log(A) - A + mp.log(2 * mp.pi) / 2))


def corr_series(a, terms):
    n2 = a * a
    acc = 0.0
    for k in reversed(range(terms)):
        acc = S[k] - acc / n2 if k < terms - 1 else S[k]
    return acc / a


def corr_loader(a):
    """Loader's stirlerr cut points: 15, 35, 80, 500 (R's stirlerr for n > 15)."""
    terms = 2 if a > 500 else 3 if a > 80 else 4 if a > 35 else 5
    return corr_series(a, terms)


def corr_lgamma(a):
    return math.lgamma(a) - ((a - 0.5) * math.log(a) - a + LN_SQRT_2PI)


CORRS = {
    "exact": corr_exact,
    "loader": corr_loader,
    "series5": lambda a: corr_series(a, 5),
    "series3": lambda a: corr_series(a, 3),
    "lgamma": corr_lgamma,
}

CHAINS = {
    "sqrt(a/2pi) * exp(-c) / x": lambda a, x, c: math.sqrt(a / TWO_PI) * math.exp(-c) / x,
    "sqrt(a/2pi) / x * exp(-c)": lambda a, x, c: math.sqrt(a / TWO_PI) / x * math.exp(-c),
    "exp(-c) * sqrt(a/2pi) / x": lambda a, x, c: math.exp(-c) * math.sqrt(a / TWO_PI) / x,
    "exp(-c) / x * sqrt(a/2pi)": lambda a, x, c: math.exp(-c) / x * math.sqrt(a / TWO_PI),
    "sqrt(a/2pi) * (exp(-c) / x)": lambda a, x, c: math.sqrt(a / TWO_PI) * (math.exp(-c) / x),
    "exp(-c) / sqrt(2pi a) * (a/x)": lambda a, x, c: math.exp(-c) / math.sqrt(TWO_PI * a) * (a / x),
    "exp(-c) / sqrt(2pi a)": lambda a, x, c: math.exp(-c) / math.sqrt(TWO_PI * a),
    "exp(-c) / sqrt(2pi) / sqrt(a)": lambda a, x, c: math.exp(-c) / math.sqrt(TWO_PI) / math.sqrt(a),
    "exp(-c) / (sqrt(2pi) sqrt(a))": lambda a, x, c: math.exp(-c) / (math.sqrt(TWO_PI) * math.sqrt(a)),
    "INV_SQRT_2PI / sqrt(a) * exp(-c)": lambda a, x, c: INV_SQRT_2PI / math.sqrt(a) * math.exp(-c),
    "INV_SQRT_2PI * exp(-c) / sqrt(a)": lambda a, x, c: INV_SQRT_2PI * math.exp(-c) / math.sqrt(a),
    "sqrt(a) * INV_SQRT_2PI * exp(-c) / x": lambda a, x, c: math.sqrt(a) * INV_SQRT_2PI * math.exp(-c) / x,
    "sqrt(a) / x * INV_SQRT_2PI * exp(-c)": lambda a, x, c: math.sqrt(a) / x * INV_SQRT_2PI * math.exp(-c),
    "sqrt(1/(2pi a)) * exp(-c)": lambda a, x, c: math.sqrt(1 / (TWO_PI * a)) * math.exp(-c),
    "1 / (sqrt(2pi a) exp(c))": lambda a, x, c: 1 / (math.sqrt(TWO_PI * a) * math.exp(c)),
    "exp(-c) / sqrt(2pi a) / exp(0)": lambda a, x, c: math.exp(-c) / math.sqrt(TWO_PI * a),
    "exp(-c - ln sqrt(2pi a))": lambda a, x, c: math.exp(-c - 0.5 * math.log(TWO_PI * a)),
    "exp(-c - LN_SQRT_2PI - ln(a)/2)": lambda a, x, c: math.exp(-c - LN_SQRT_2PI - 0.5 * math.log(a)),
}


def cells(col):
    out = []
    for r in range(1, 41):
        m = re.fullmatch(r"=GAMMA\.DIST\(([^,]+),([^,]+),([^,]+),FALSE\)", CELLS[f"{col}{r}"])
        out.append((float(m[1]), float(m[2]), float(m[3]), float(ORACLE[f"{col}{r}"]["v"])))
    return out


def true_density(a):
    A = M(a)
    return mp.exp((A - 1) * mp.log(A) - A - mp.loggamma(A))


def ulps(v, ex):
    return round((v - ex) / math.ulp(ex), 2)


def main():
    fit, hold = cells("A"), cells("B")
    print("Excel against the 50-digit density, in ulps (fit A, then hold-out B):")
    for name, pts in (("A", fit), ("B", hold)):
        print(f"  {name}: {[ulps(float(true_density(a)), ex) for _, a, _, ex in pts]}")
    print("\nChains: fit exact / within 1 of 40, hold-out exact / within 1 of 40")
    rows = []
    for cname, corr in CORRS.items():
        for chain, f in CHAINS.items():
            score = []
            for pts in (fit, hold):
                errs = [ulps(f(a, x, corr(a)), ex) for x, a, _, ex in pts]
                score.append((sum(e == 0 for e in errs), sum(abs(e) <= 1 for e in errs), errs))
            rows.append((score[0][0] + score[0][1], cname, chain, score))
    rows.sort(key=lambda t: -t[0])
    for _, cname, chain, score in rows:
        print(f"  {cname:8} {chain:38} {score[0][0]:2}/{score[0][1]:2} | {score[1][0]:2}/{score[1][1]:2}")
    best = rows[0]
    print(f"\nBest chain's misses: fit {[(fit[i][1], e) for i, e in enumerate(best[3][0][2]) if e]}")
    print(f"  hold-out {[(hold[i][1], e) for i, e in enumerate(best[3][1][2]) if e]}")
    print("\nColumn C (scale 3) against column A / 3, in ulps:")
    print(f"  {[ulps(c[3], a[3] / 3) for a, c in zip(fit, cells('C'), strict=True)]}")


def literal(text):
    """The double Excel reads for a formula literal: 15 significant digits, cut (numbers.md)."""
    d = Decimal(text)
    return float(d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_DOWN))


def exact_b(a, y):
    A, Y = M(a), M(y)
    return float(A * mp.log(A / Y) + Y - A)


def old_chain(a, x, y):
    return math.sqrt(a / TWO_PI) * math.exp(-exact_b(a, y) - corr_series(a, 5)) / x


def window_chain(a, x, y):
    return math.exp(-corr_exact(a) - exact_b(a, y)) / math.sqrt(TWO_PI * a) * (a / x)


def part2():
    print("\nPart 2: window densities, bit-exact / within 15 digits (old chain, then the window chain)")
    num = r"(-?[0-9.]+(?:E[-+]?[0-9]+)?)"
    pat = re.compile(rf"=(?:GAMMA\.DIST|GAMMADIST)\({num},{num},{num},(?:FALSE|0)\)")
    root = CASES.parents[1]
    total = [0] * 5
    for path in sorted(glob.glob(str(root / "probes*/cases/*.case.json"))):
        sidecar = Path(path.replace(".case.json", ".oracle.json"))
        if not sidecar.exists():
            continue
        oracle = json.loads(sidecar.read_text())["sheets"]
        counts = [0] * 5
        for sheet in json.loads(Path(path).read_text())["sheets"]:
            values = oracle.get(sheet["name"], {})
            for ref, formula in sheet.get("cells", {}).items():
                m = pat.fullmatch(formula) if isinstance(formula, str) else None
                if not m or values.get(ref, {}).get("t") != "n":
                    continue
                x, a, b = literal(m[1]), float(m[2]), float(m[3])
                y = x / b
                if a < 20 or not abs(y - a) < 0.2 * (y + a):
                    continue
                excel = values[ref]["v"]
                counts[0] += 1
                for i, chain in ((1, old_chain), (3, window_chain)):
                    v = chain(a, x, y)
                    counts[i] += v == excel
                    counts[i + 1] += f"{v:.15g}" == f"{excel:.15g}"
        if counts[0]:
            print(f"  {Path(path).name[:-10]:40} {counts[0]:4}  {counts[1]:4}/{counts[2]:4}  {counts[3]:4}/{counts[4]:4}")
            total = [t + c for t, c in zip(total, counts, strict=True)]
    print(f"  {'total':40} {total[0]:4}  {total[1]:4}/{total[2]:4}  {total[3]:4}/{total[4]:4}")


if __name__ == "__main__":
    main()
    part2()
