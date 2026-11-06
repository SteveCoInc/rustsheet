"""T-4255: score candidate large-shape gamma prefactors against Excel inside the T-3252 window.

Usage: python3 score_forms.py  (needs mpmath; reads the probe cases and their oracle sidecars)

Part 1 scores each candidate exponent on the T-3252 GAMMA.DIST densities (scale 1) with x/a in
[0.668, 0.99] or [1.05, 1.45]. The split was fixed before scoring: fit = window + holdout,
hold-out = holdout-2 + sweep. A cell counts when the candidate is within 2 ulp of Excel.

Part 2 reads probes/t4255-gamma-window-exact-ratio, where x/a = k/64 is exact and the shape is 64
(a power of two, so a*g(x/a) and a - x are exact): the per-unit exponent error
delta = (E_excel - E_true) / a, in ulps of ln(x/a), against each candidate's own delta.
"""

import json
import math
import re
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
M = mp.mpf
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases"


def stirling_correction(a):
    """ln Gamma(a) - Stirling's series, to 50 digits, rounded once (the engine's other factors)."""
    A = M(a)
    return float(mp.loggamma(A) - ((A - 0.5) * mp.log(A) - A + mp.log(2 * mp.pi) / 2))


def density(x, a, exponent):
    """sqrt(a / 2 pi) exp(E - correction(a)) / x: Excel's ratio-form density outside the window."""
    return math.sqrt(a / (2 * math.pi)) * math.exp(exponent - stirling_correction(a)) / x


def toms_rlog(u):
    """TOMS 654's rlog(u) = u - 1 - ln u, its rational on [0.61, 1.57]."""
    if u < 0.61 or u > 1.57:
        return ((u - 0.5) - 0.5) - math.log(u)
    if u < 0.82:
        t, w1 = (u - 0.7) / 0.7, 0.566749439387324e-01
        w1 -= t * 0.3
    elif u > 1.18:
        t = 0.75 * u - 1.0
        w1 = 0.456512608815524e-01 + t / 3.0
    else:
        t, w1 = (u - 0.5) - 0.5, 0.0
    r = t / (t + 2.0)
    s = r * r
    w = ((0.620886815375787e-02 * s - 0.224696413112536) * s + 0.333333333333333) / (
        (0.354508718369557 * s - 0.127408923933623e01) * s + 1.0
    )
    return 2.0 * s * (1.0 / (1.0 - r) - r * w) + w1


def bd0(k, lam):
    """Loader's bd0(k, lam) = k ln(k/lam) + lam - k by its series in v = (k - lam)/(k + lam)."""
    v = (k - lam) / (k + lam)
    s, ej, v2 = (k - lam) * v, 2 * k * v, v * v
    for j in range(1, 1000):
        ej *= v2
        s1 = s + ej / (2 * j + 1)
        if s1 == s:
            break
        s = s1
    return s


def atanh_series(v):
    """atanh(v) by Horner in v^2 (20 terms)."""
    v2, p = v * v, 0.0
    for k in range(20, 0, -1):
        p = p * v2 + 1.0 / (2 * k + 1)
    return v + v * (v2 * p)


def exact_exponent(x, a):
    X, A = M(x), M(a)
    return A * mp.log(X / A) + A - X


def candidates(x, a):
    """Each candidate exponent E (density = sqrt(a/2pi) exp(E - corr(a)) / x), in double."""
    u, w = x / a, (x - a) / a
    return {
        "ratio form (T-2638)": (-a * math.log(a / x) if x < a else a * math.log(x / a)) + (a - x),
        "rcomp, accurate": float(exact_exponent(x, a)),
        "a ln(x/a) + (a - x)": a * math.log(u) + (a - x),
        "a log1p(w) + (a - x)": a * math.log1p(w) + (a - x),
        "-a((u - 1) - ln u)": -a * (((u - 0.5) - 0.5) - math.log(u)),
        "-a(w - log1p w)": -a * (w - math.log1p(w)),
        "a(ln u + 1 - u)": a * (math.log(u) + 1 - u),
        "TOMS 654 -a rlog(u)": -a * toms_rlog(u),
        "-bd0(a, x)": -bd0(a, x),
        "a 2atanh(v) + (a - x)": a * 2 * atanh_series((x - a) / (x + a)) + (a - x),
        "(a ln x - x) - (a ln a - a)": (a * math.log(x) - x) - (a * math.log(a) - a),
    }


def true_density(x, a):
    X, A = M(x), M(a)
    return mp.exp((A - 1) * mp.log(X) - X - mp.loggamma(A))


def load(stem):
    cells = json.loads((CASES / f"{stem}.case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads((CASES / f"{stem}.oracle.json").read_text())["sheets"]["Calc"]
    return cells, oracle


def part1():
    split = {"fit": ("window", "holdout"), "hold-out": ("holdout-2", "sweep")}
    rows = []
    for part, stems in split.items():
        for s in stems:
            cells, oracle = load(f"t3252-gamma-prefactor-{s}")
            for ref, f in cells.items():
                m = re.fullmatch(r"=GAMMA\.DIST\(([^,]+),([^,]+),1,FALSE\)", f)
                if not m:
                    continue
                x, a = float(m[1]), float(m[2])
                q = x / a
                if 0.6675 <= q <= 0.991 or 1.049 <= q <= 1.4501:
                    rows.append((part, x, a, oracle[ref]["v"]))
    names = list(candidates(30.0, 50.0))
    print("Part 1: within 2 ulp of Excel / rms ulp from Excel")
    print(f"{'form':30} " + "  ".join(f"{p:>16}" for p in split))
    for n in names:
        cols = []
        for p in split:
            g = [r for r in rows if r[0] == p]
            d = []
            for _, x, a, xl in g:
                t = true_density(x, a)
                d.append(float((M(density(x, a, candidates(x, a)[n])) - M(xl)) / math.ulp(xl)))
            cols.append(
                f"{sum(abs(v) <= 2 for v in d):3}/{len(g)} / {math.sqrt(sum(v * v for v in d) / len(d)):6.1f}"
            )
        print(f"{n:30} " + "  ".join(f"{c:>16}" for c in cols))
    ex = [float((M(xl) - true_density(x, a)) / math.ulp(xl)) for _, x, a, xl in rows]
    print(
        f"Excel against the 50-digit density: rms {math.sqrt(sum(v * v for v in ex) / len(ex)):.1f} ulp, {len(ex)} cells"
    )


def part2():
    cells, oracle = load("t4255-gamma-window-exact-ratio")
    a = 64.0
    names = list(candidates(30.0, 50.0))
    sq = {n: 0.0 for n in names}
    n_cells = 0
    print(
        "\nPart 2: per-unit exponent error at shape 64, rms ulp(ln u) from Excel's (u in (0.667, 0.94) or (1.06, 1.49))"
    )
    for r in range(1, 59):
        u = (r + 38) / 64
        if not (0.667 < u < 0.94 or 1.06 < u < 1.49):
            continue
        x = a * u
        xl = oracle[f"A{r}"]["v"]
        t = true_density(x, a)
        e_true = exact_exponent(x, a)
        # Excel's exponent error: its density against the same chain on the exact exponent.
        base = M(density(x, a, float(e_true)))
        d_excel = (mp.log(M(xl) / t) - mp.log(base / t)) / a
        unit = math.ulp(math.log(u))
        n_cells += 1
        for n, e in candidates(x, a).items():
            sq[n] += float((((M(e) - e_true) / a) - d_excel) / unit) ** 2
    for n in names:
        print(f"{n:30} {math.sqrt(sq[n] / n_cells):7.2f}")
    print(f"({n_cells} cells; a delta uniform in +-0.5 ulp has rms 0.29)")


if __name__ == "__main__":
    part1()
    part2()
