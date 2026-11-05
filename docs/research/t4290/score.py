"""T-4290: read probes/t4290-gamma-window-large-pow2 (Excel's large-shape gamma density in the window).

Usage: tools/.venv/bin/python docs/research/t4290/score.py  (needs mpmath; reads the case and its oracle sidecar)

Part 1, the chain constant: below the window (x/a < 0.6667) Excel's density, read back through
sqrt(a/2pi) exp(E - c(a)) / x with c(a) = ln Gamma(a) - Stirling to 50 digits, is the ratio-form
exponent E to the bit once one log offset per shape is allowed. The offset is found from the
fractional parts of the implied exponent (in ulps of E), modulo the ulp of the largest binade, and
the ratio-form cells then sit at 0 ulp.
Part 2, the per-unit error delta = ln(Excel / true density) / a in ulps of ln(x/a) inside the
window, at shapes 4096, 1024 and 256 (columns A, B, C), side by side.
Part 3, families: each candidate exponent's own delta against Excel's, rms and the count within
1/8 ulp(ln u). The split was fixed before scoring: fit = A, B, C; hold-out = D (shape 2048, x/a on
the k + 2 grid). Cells within 0.02 of the mode are left out (the exponent there is tiny).
Part 4, the ladders across x/a = 2/3 (shape 3072, column E) and 3/2 (shape 2048, column F): Excel
reads a formula literal at 15 significant digits, cut (numbers.md), so x is taken that way.
"""

import json
import math
import re
import sys
from decimal import ROUND_DOWN, Decimal
from pathlib import Path

import mpmath as mp

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "t4255"))
from score_forms import candidates  # noqa: E402

mp.mp.dps = 50
M = mp.mpf
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases"
STEM = "t4290-gamma-window-large-pow2"
CELLS = json.loads((CASES / f"{STEM}.case.json").read_text())["sheets"][0]["cells"]
ORACLE = json.loads((CASES / f"{STEM}.oracle.json").read_text())["sheets"]["Calc"]


def literal(text):
    """The double Excel reads for a formula literal: 15 significant digits, cut."""
    d = Decimal(text)
    return float(d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_DOWN))


def cell(ref):
    m = re.fullmatch(r"=GAMMA\.DIST\(([^,]+),([^,]+),1,FALSE\)", CELLS[ref])
    return literal(m[1]), float(m[2]), M(ORACLE[ref]["v"])


def corr(a):
    A = M(a)
    return mp.loggamma(A) - ((A - 0.5) * mp.log(A) - A + mp.log(2 * mp.pi) / 2)


def true_density(x, a):
    X, A = M(x), M(a)
    return mp.exp((A - 1) * mp.log(X) - X - mp.loggamma(A))


def exact_exponent(x, a):
    X, A = M(x), M(a)
    return A * mp.log(X / A) + A - X


def ratio_form(x, a):
    return (-a * math.log(a / x) if x < a else a * math.log(x / a)) + (a - x)


def implied_exponent(x, a, xl):
    return mp.log(xl * x / mp.sqrt(M(a) / (2 * mp.pi))) + corr(a)


def part1():
    print("Part 1: below-window cells against the ratio form, after one log offset per shape")
    for col in "ABCD":
        rows = [cell(f"{col}{r}") for r in range(1, 227)]
        below = [(x, a, xl) for x, a, xl in rows if x / a < 0.6667]
        a = below[0][1]
        imp = [(implied_exponent(x, a, xl), ratio_form(x, a)) for x, a, xl in below]
        # Excel's E is a double: the offset makes every implied E land on its own grid.
        best = min(
            (sum(abs(float((i - c * M(1e-17) - r) / math.ulp(r))) for i, r in imp), c)
            for c in range(-10000, 10001)
        )
        off = best[1] * 1e-17
        ulps = [float((i - off - r) / math.ulp(r)) for i, r in imp]
        print(f"  shape {a:6.0f}: offset {off:+.3e}, ulps from the ratio form {[round(u, 2) for u in ulps]}")


def part2():
    print("\nPart 2: per-unit error delta in ulp(ln u) (x/a, then shapes 4096, 1024, 256)")
    for r in range(1, 227, 8):
        out = []
        for col in "ABC":
            x, a, xl = cell(f"{col}{r}")
            u = x / a
            out.append(float(mp.log(xl / true_density(x, a)) / a / math.ulp(math.log(u))) if u != 1 else 0.0)
        print(f"  {u:.6f} " + " ".join(f"{d:+.3f}" for d in out))


def part3():
    print("\nPart 3: families, rms delta difference in ulp(ln u) and count within 1/8 (fit A-C, hold-out D)")
    pts = {"fit": [], "hold-out": []}
    for r in range(1, 227):
        for col in "ABCD":
            x, a, xl = cell(f"{col}{r}")
            if 0.6668 < x / a < 1.4999 and abs(x / a - 1) >= 0.02:
                d = mp.log(xl / true_density(x, a)) / a
                pts["hold-out" if col == "D" else "fit"].append((x, a, d))
    for name in candidates(30.0, 50.0):
        cols = []
        for part, ps in pts.items():
            errs = []
            for x, a, d in ps:
                e = candidates(x, a)[name]
                errs.append(float(((M(e) - exact_exponent(x, a)) / a - d) / math.ulp(math.log(x / a))))
            rms = math.sqrt(sum(e * e for e in errs) / len(errs))
            cols.append(f"{part} {rms:5.2f} {sum(abs(e) < 0.125 for e in errs):3}/{len(errs)}")
        print(f"  {name:30} " + " | ".join(cols))


def part4():
    print("\nPart 4: ladders (implied exponent minus the ratio form, absolute)")
    for col in "EF":
        for r in range(1, 14):
            x, a, xl = cell(f"{col}{r}")
            v = (M(x) - a) / (M(x) + a)
            print(
                f"  {col}{r:<3} x {x!r:>18} v {float(v):+.17f} {float(implied_exponent(x, a, xl) - ratio_form(x, a)):+.3e}"
            )


if __name__ == "__main__":
    part1()
    part2()
    part3()
    part4()
