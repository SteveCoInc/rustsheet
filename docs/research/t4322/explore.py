"""T-4322 exploration: candidate models of POISSON.DIST's cdf inside the gamma window (x/a <= 1.5)."""
import math, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "t4302"))
import score as s  # noqa: E402
import mpmath as mp  # noqa: E402

M = mp.mpf
L = math.log


def atanh_cr(v):
    return float(mp.atanh(M(v)))


def bw(a, y):
    d = y - a
    v = d / (y + a)
    return d * v - (2.0 * a * atanh_cr(v) - 2.0 * a * v)


def bd(a, y):
    return a * L(a / y) + (y - a)


def c_exact(a):
    return s.stirlerr(a)


def ulps(v, ref):
    return (v - ref) / math.ulp(ref)


def main():
    pts = [p for p in s.points() if p[3] / (p[2] + 1) <= 1.5]
    rows = []
    for split, ref, k, m, cdf, dens in pts:
        a, x = k + 1.0, m
        q, pref, F = s.exact(a, x)
        c = c_exact(a)
        B_true = M(a) * mp.log(M(a) / M(x)) + M(x) - M(a)
        cand = {}
        for name, B in (("win", bw(a, x)), ("dir", bd(a, x))):
            p1 = math.sqrt(a / (2 * math.pi)) * math.exp(-B - c)
            cand[f"sqrt*exp {name}"] = float(M(p1) * F)
            ch = math.exp(-B - c) / math.sqrt(2 * math.pi * a) * (a / x)
            cand[f"chain*x {name}"] = float(M(ch) * M(x) * F)
        cand["xlDens*x*F"] = float(M(dens) * M(x) * F)
        cand["true"] = float(q)
        # implied exponent offset of Excel: ln(cdf/q) in ulps of B_true
        implied = float(mp.log(M(cdf) / q)) / math.ulp(float(B_true))
        dimp = float(mp.log(M(dens) / (pref / M(x)))) / math.ulp(float(B_true))
        rows.append((split, ref, k, m, {n: ulps(v, cdf) for n, v in cand.items()}, implied, dimp,
                     ulps(bw(a, x), float(B_true)) if True else 0))
    names = list(rows[0][4])
    print(f"{'split':8} {'ref':4} {'k':>4} {'x/a':>6} " + " ".join(f"{n[:13]:>13}" for n in names) + "  impl(cdf) impl(dens)")
    for split, ref, k, m, e, imp, dimp, _ in rows:
        print(f"{split:8} {ref:4} {k:4} {m/(k+1):6.4f} " + " ".join(f"{e[n]:13.1f}" for n in names) + f"  {-imp:8.2f} {-dimp:8.2f}")


if __name__ == "__main__":
    main()
