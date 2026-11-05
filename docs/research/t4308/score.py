"""T-4308: read Excel's large-shape gamma window exponent B as a double, then score candidate forms.

Usage: tools/.venv/bin/python docs/research/t4308/score.py  (needs mpmath; reads the cases and their oracle sidecars)

Part 1, the read-out. Inside the window (|x - a| <= 0.2 (x + a)) the density is the T-4296 chain
exp(-c(a) - B) / sqrt(2 pi a) * (a / x), c(a) the exact Stirling error. For each probed cell this
searches the doubles within 128 ulp of the true B = a ln(a/x) + x - a for the ones whose chain, in
doubles, is Excel's value bit for bit. One B is found per cell, or none: Excel's exponent is then
known exactly, where T-4255 and T-4290 could only read it as a per-unit error of +-0.5 ulp(ln u).
Part 2, the families, scored on that B (the split and the margin were fixed before scoring, see
the task file): the count of cells where a candidate's B is Excel's exactly, then the rms of its
distance in ulp(B). Fit = t4290 columns A:C, hold-out = column D.
"""

import importlib.util
import math
import sys
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "t4255"))
# t4290's scorer is also named score.py, so it is loaded by path under its own name.
_SPEC = importlib.util.spec_from_file_location("t4290_score", HERE.parent / "t4290" / "score.py")
t4290 = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(t4290)
from score_forms import atanh_series, bd0, toms_rlog  # importable once the paths above are set

mp.mp.dps = 50
M = mp.mpf
TWO_PI = 2 * math.pi
SEARCH = 128


def corr(a):
    return float(t4290.corr(a))


def chain(b, a, x, c):
    """The T-4296 window chain in doubles."""
    return math.exp(-c - b) / math.sqrt(TWO_PI * a) * (a / x)


def true_b(x, a):
    X, A = M(x), M(a)
    return A * mp.log(A / X) + X - A


def excel_b(x, a, excel):
    """The doubles B within SEARCH ulp of the true B whose chain is `excel` bit for bit."""
    c, b0 = corr(a), float(true_b(x, a))
    found, b = [], b0
    for _ in range(SEARCH):
        b = math.nextafter(b, -math.inf)
    for _ in range(2 * SEARCH + 1):
        if chain(b, a, x, c) == excel:
            found.append(b)
        b = math.nextafter(b, math.inf)
    return found


def series_w(w):
    """u - 1 - ln u = w - log1p(w) by its Taylor series in w = u - 1, summed until it stops moving."""
    term, acc, k = -w, 0.0, 2
    while k < 2000:
        term = -term * w
        nxt = acc + term / k
        if nxt == acc:
            break
        acc, k = nxt, k + 1
    return acc


def two_sum(p, q):
    s = p + q
    bp = s - p
    return s, (p - (s - bp)) + (q - bp)


def split(p):
    t = p * 134217729.0
    hi = t - (t - p)
    return hi, p - hi


def two_prod(p, q):
    """Dekker's exact product p q = hi + lo (no fma in this Python)."""
    hi = p * q
    ph, pl = split(p)
    qh, ql = split(q)
    return hi, ((ph * qh - hi) + ph * ql + pl * qh) + pl * ql


def fma_form(x, a):
    """fl(-a ln u + (x - a)) with the product kept exact (one rounding, as an fma would give)."""
    hi, lo = two_prod(-a, math.log(x / a))
    s, e = two_sum(hi, x - a)
    return s + (e + lo)


def candidates(x, a):
    """Each candidate B (density = exp(-c - B) / sqrt(2 pi a) (a / x)), in doubles."""
    u, w, v = x / a, (x - a) / a, (x - a) / (x + a)
    return {
        "correctly rounded B": float(true_b(x, a)),
        "a ln(a/x) + (x - a) (Loader's direct)": a * math.log(a / x) + (x - a),
        "-a ln(x/a) + (x - a)": -a * math.log(u) + (x - a),
        "fl(-a ln u + (x - a)), one rounding": fma_form(x, a),
        "-a log1p(w) + (x - a)": -a * math.log1p(w) + (x - a),
        "a((u - 1) - ln u)": a * ((u - 1) - math.log(u)),
        "a(w - log1p w)": a * (w - math.log1p(w)),
        "a fl(u - 1 - ln u) (B/a rounded)": a * float(M(u) - 1 - mp.log(u)),
        "a series_w(w)": a * series_w(w),
        "a TOMS 654 rlog(u)": a * toms_rlog(u),
        "bd0(a, x) (Loader's series)": bd0(a, x),
        "-2a atanh(v) + (x - a), series": -2 * a * atanh_series(v) + (x - a),
        "-2a atanh(v) + (x - a), libm": -2 * a * math.atanh(v) + (x - a),
        "(x - a) v - 2a(atanh(v) - v)": (x - a) * v - 2 * a * (math.atanh(v) - v),
        "a ln(a/x) - (a - x)": a * math.log(a / x) - (a - x),
        "(a ln(a/x) + x) - a": (a * math.log(a / x) + x) - a,
    }


def points():
    """(part, ref, x, a, Excel's B) for the t4290 window cells, and the refs with no B."""
    pts, missing = [], []
    for r in range(1, 227):
        for col in "ABCD":
            x, a, xl = t4290.cell(f"{col}{r}")
            if not (0.6668 < x / a < 1.4999) or abs(x / a - 1) < 0.02:
                continue
            found = excel_b(x, a, float(xl))
            if len(found) == 1:
                pts.append(("hold-out" if col == "D" else "fit", f"{col}{r}", x, a, found[0]))
            else:
                missing.append((f"{col}{r}", len(found)))
    return pts, missing


def part1(pts, missing):
    print("Part 1: Excel's B read out as a double on the T-4296 chain")
    for part in ("fit", "hold-out"):
        print(f"  {part}: {sum(p[0] == part for p in pts)} cells with exactly one B")
    print(f"  cells with no B or more than one: {missing}")
    print("  distance of Excel's B from the true B, in ulp(B), and in a ulp(ln u) (x/a, every 8th fit cell):")
    for _, ref, x, a, b in [p for p in pts if p[0] == "fit"][::8]:
        t = true_b(x, a)
        print(
            f"    {ref:5} {x / a:.6f} {float((b - t) / math.ulp(b)):+8.2f}"
            f" {float((b - t) / (a * math.ulp(math.log(x / a)))):+.3f}"
        )


def part2(pts):
    print("\nPart 2: families, cells with Excel's B exactly and rms ulp(B) (fit A:C | hold-out D)")
    names = list(candidates(30.0, 50.0))
    rows = []
    for name in names:
        out = []
        for part in ("fit", "hold-out"):
            ps = [p for p in pts if p[0] == part]
            errs = [(candidates(x, a)[name] - b) / math.ulp(b) for _, _, x, a, b in ps]
            exact = sum(e == 0 for e in errs)
            out.append((exact, len(ps), math.sqrt(sum(e * e for e in errs) / len(errs))))
        rows.append((name, out))
    for name, out in rows:
        cols = " | ".join(f"{e:3}/{n:3} ({100 * e / n:4.1f}%) {r:6.2f}" for e, n, r in out)
        print(f"  {name:40} {cols}")


if __name__ == "__main__":
    PTS, MISSING = points()
    part1(PTS, MISSING)
    part2(PTS)
