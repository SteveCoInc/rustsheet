"""T-4345: POISSON.DIST's cdf below the 1.3a cut (x / (k + 1) in [1, 1.3)), candidate models scored against Excel.

Run from the repository root with the tools venv (mpmath), plus scipy for the last candidate (skipped without it;
scored with scipy 1.17.1):
    tools/.venv/bin/python docs/research/t4345/score.py

Splits (decision 0188 R4), fixed before any candidate was scored: train is every fit cell of T-4322 below 1.3a
(probes/t4291-poisson-far-above-mean A1:H1, probes/t4302-poisson-cdf-beside-pmf D19); the hold-out is every cell
of the fresh sweep probes-numeric/t4322-poisson-window-r4-fresh below 1.3a (A8:C10). No candidate was taken, so
no fresh sweep was drawn. Part 2 re-scores T-4322's window form (x/a 1.3 to 1.5) with Cephes' continued fraction
evaluated in doubles in place of the exact fraction, to test whether that explains the form's few-ulp residue.
"""

import importlib.util
import math
import sys
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "t4302"))
import score as t4302  # noqa: E402  (its oracle reader, exact Q and Stirling error)

_spec = importlib.util.spec_from_file_location("t4322_score", HERE.parent / "t4322" / "score.py")
assert _spec and _spec.loader
t4322 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4322)

mp.mp.dps = 50
M = mp.mpf
CUT = 1.3
MACHEP = 1.11022302462515654042e-16
BIG = 4.503599627370496e15


def exact_q(a, x):
    return t4302.exact(a, x)[0]


def cephes_cf(a, x):
    """Cephes igamc's continued fraction for Q / (x^a e^-x / Gamma(a)), in doubles."""
    y = 1.0 - a
    z = x + y + 1.0
    c = 0.0
    pkm2, qkm2, pkm1, qkm1 = 1.0, x, x + 1.0, (x + y + 1.0) * x
    ans = pkm1 / qkm1
    while True:
        c += 1.0
        y += 1.0
        z += 2.0
        yc = y * c
        pk = pkm1 * z - pkm2 * yc
        qk = qkm1 * z - qkm2 * yc
        t = 1.0
        if qk != 0:
            r = pk / qk
            t = abs((ans - r) / r)
            ans = r
        pkm2, pkm1, qkm2, qkm1 = pkm1, pk, qkm1, qk
        if abs(pk) > BIG:
            pkm2, pkm1, qkm2, qkm1 = (v / BIG for v in (pkm2, pkm1, qkm2, qkm1))
        if t <= MACHEP:
            return ans


def stirling_pref(b):
    return lambda a, x: math.sqrt(a / (2 * math.pi)) * math.exp(-b(a, x) - t4302.stirlerr(a))


def direct_b(a, x):
    return a * math.log(a / x) + (x - a)


def cr_b(a, x):
    return float(M(a) * mp.log(M(a) / M(x)) + M(x) - M(a))


def temme(b, z_of, rounded_erfc):
    """Temme's Q = erfc(z) / 2 + R with z = sqrt(B) from a double B; R exact, so only the leading term is tested."""
    def f(a, x):
        zx = mp.sqrt(M(a) * mp.log(M(a) / M(x)) + M(x) - M(a))
        r = exact_q(a, x) - mp.erfc(zx) / 2
        z = z_of(a, b(a, x))
        lead = float(mp.erfc(M(z))) if rounded_erfc else math.erfc(z)
        return 0.5 * lead + float(r)
    return f


def pmf(k, m):
    return float(mp.exp(-M(m)) * M(m) ** k / mp.factorial(k))


def pmf_sum_down(k, m):
    t = s = pmf(k, m)
    for i in range(k, 0, -1):
        t = t * i / m
        s += t
    return s


def pmf_times_bracket(k, m):
    t = s = 1.0
    for i in range(k, 0, -1):
        t = t * i / m
        s += t
    return pmf(k, m) * s


def pmf_sum_up(k, m):
    t = s = math.exp(-m)
    for i in range(1, k + 1):
        t = t * m / i
        s += t
    return s


def log1pmx_b(a, x):
    s = (x - a) / a
    return -a * (math.log1p(s) - s)


SQRT_B = lambda a, b: math.sqrt(b)  # noqa: E731
ETA_FORM = lambda a, b: math.sqrt(2 * b / a) * math.sqrt(a / 2)  # noqa: E731

CANDIDATES = {
    "correctly rounded Q": lambda k, a, x: float(exact_q(a, x)),
    "window form x exact F (T-4322)": lambda k, a, x: stirling_pref(t4322.window_b)(a, x) * float(t4302.exact(a, x)[2]),
    "window form x Cephes CF": lambda k, a, x: stirling_pref(t4322.window_b)(a, x) * cephes_cf(a, x),
    "direct-B prefactor x Cephes CF": lambda k, a, x: stirling_pref(direct_b)(a, x) * cephes_cf(a, x),
    "rounded prefactor x Cephes CF": lambda k, a, x: float(t4302.exact(a, x)[1]) * cephes_cf(a, x),
    "Cephes exp(a ln x - x - lgam a) x CF": lambda k, a, x: math.exp(a * math.log(x) - x - math.lgamma(a)) * cephes_cf(a, x),
    "Temme erfc(sqrt(rounded B))": lambda k, a, x: temme(cr_b, SQRT_B, True)(a, x),
    "Temme erfc(sqrt(window B))": lambda k, a, x: temme(t4322.window_b, SQRT_B, True)(a, x),
    "Temme erfc(sqrt(direct B))": lambda k, a, x: temme(direct_b, SQRT_B, True)(a, x),
    "Temme erfc(eta sqrt(a/2)), log1pmx B": lambda k, a, x: temme(log1pmx_b, ETA_FORM, True)(a, x),
    "pmf sum, k down to 0": lambda k, a, x: pmf_sum_down(k, x),
    "pmf(k) x bracket in doubles": lambda k, a, x: pmf_times_bracket(k, x),
    "pmf sum, 0 up to k": lambda k, a, x: pmf_sum_up(k, x),
}
try:
    from scipy.special import gammaincc  # Cephes igamc: the Temme series below SMALLRATIO 0.3

    CANDIDATES["scipy gammaincc (Cephes igamc)"] = lambda k, a, x: float(gammaincc(a, x))
except ImportError:
    pass


def splits():
    train = [p for p in t4322.fit_points() if p[2] / (p[1] + 1) < CUT]
    hold = [p for p in t4322.fresh_points() if p[2] / (p[1] + 1) < CUT]
    above_fit = [p for p in t4322.fit_points() if p[2] / (p[1] + 1) >= CUT]
    above_fresh = [p for p in t4322.fresh_points() if p[2] / (p[1] + 1) >= CUT]
    return {"train": train, "hold-out": hold}, {"fit": above_fit, "fresh": above_fresh}


def ulps(v, ref):
    return (v - ref) / math.ulp(ref)


def tally(f, pts):
    u = [ulps(f(k, k + 1.0, m), cdf) for _, k, m, cdf in pts]
    a = [abs(x) for x in u]
    return f"{sum(x <= 0.5 for x in a):2}/{sum(x <= 4 for x in a):2} of {len(a):2}, max {max(a):5.0f}", u


def main():
    below, above = splits()
    print(f"1. Below {CUT}a: bit-exact / within 4 ulp, max |ulp| from Excel")
    for name, f in CANDIDATES.items():
        cells = []
        line = f"   {name:38}"
        for split, pts in below.items():
            s, u = tally(f, pts)
            line += f"  {split} {s}"
            cells += [f"{ref}:{x:.0f}" for (ref, *_), x in zip(pts, u)]
        print(line)
        print("      " + " ".join(cells))
    print(f"2. {CUT}a to 1.5a (T-4322's window form): the fraction exact or Cephes' CF in doubles")
    for name in ("window form x exact F (T-4322)", "window form x Cephes CF"):
        print(f"   {name:38}" + "".join(f"  {s} {tally(CANDIDATES[name], p)[0]}" for s, p in above.items()))


if __name__ == "__main__":
    main()
