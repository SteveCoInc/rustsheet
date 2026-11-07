"""T-4322: POISSON.DIST's cdf inside the gamma window (x / (k + 1) <= 1.5), scored against Excel.

Run from the repository root with the tools venv (mpmath), after `cargo build --release -p rustsheet_eval`:
    tools/.venv/bin/python docs/research/t4322/score.py

Splits (decision 0188 R4): the fit cells are every window point of T-4302's probes
(probes/t4291-poisson-far-above-mean rows 1:2, probes/t4302-poisson-cdf-beside-pmf rows 19:24), all of
them seen by T-4302; the hold-out is the fresh sweep probes-numeric/t4322-poisson-window-r4-fresh,
drawn after the model was frozen (docs/research/t4322/make_sweep.py, seed 43220).
Parts:
1. Each candidate prefactor sqrt(a / 2 pi) exp(-B - c(a)) times the exact fraction, by form of B, on the
   fit cells, in ulp from Excel, ordered by x / a (where the cut falls).
2. The engine's cdf (bit-exact / within 4 ulp / within 15 digits) and the correctly rounded Q, on the
   fit cells and the fresh sweep, with Gate A's envelope on the sweep.
"""

import math
import re
import subprocess
import sys
from pathlib import Path

import mpmath as mp

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "t4302"))
import score as t4302  # noqa: E402  (its oracle reader, exact Q and Stirling error)

mp.mp.dps = 50
M = mp.mpf
ROOT = Path(__file__).resolve().parents[3]
FRESH = "t4322-poisson-window-r4-fresh"


def atanh_rounded(v):
    return float(mp.atanh(M(v)))


def window_b(a, y):
    """T-4308's window exponent (y - a) v - (2a atanh(v) - 2a v), v = (y - a)/(y + a)."""
    d = y - a
    v = d / (y + a)
    return d * v - (2.0 * a * atanh_rounded(v) - 2.0 * a * v)


FORMS = {
    "window B (T-4308)": window_b,
    "direct a ln(a/x) + (x - a)": lambda a, y: a * math.log(a / y) + (y - a),
    "correctly rounded B": lambda a, y: float(M(a) * mp.log(M(a) / M(y)) + M(y) - M(a)),
}


def ulps(v, ref):
    return (v - ref) / math.ulp(ref)


def fit_points():
    return [(ref, k, m, cdf) for split, ref, k, m, cdf, _ in t4302.points()
            if split in ("train", "hold-out") and m / (k + 1) <= 1.5]


def fresh_points():
    cells, orc = t4302.oracle("probes-numeric", FRESH)
    out = []
    for ref, f in cells.items():
        if ref in orc and orc[ref].get("t") == "n":
            k, m = t4302.args(f)
            out.append((ref, int(k), m, orc[ref]["v"]))
    return out


def engine(stem, suite):
    text = subprocess.run([ROOT / "target/release/rustsheet-eval", "--root", ROOT, "show",
                           f"{suite}/{stem}.case.json", "--all"], capture_output=True, text=True).stdout
    vals, ref = {}, None
    for line in text.splitlines():
        if m := re.match(r"^Calc!(\S+)", line):
            ref = m[1]
        elif (m := re.match(r"^\s+actual\s*: ?(.*)", line)) and ref:
            vals[ref] = float(m[1])
    return vals


def main():
    fit = fit_points()
    print("1. Candidate prefactors times the exact fraction, ulp from Excel (fit cells by x/a)")
    print("   ref    k     x/a   " + "  ".join(f"{n[:20]:>20}" for n in FORMS))
    for ref, k, m, cdf in sorted(fit, key=lambda p: p[2] / (p[1] + 1)):
        a = k + 1.0
        fraction = t4302.exact(a, m)[2]
        c = t4302.stirlerr(a)
        errs = [ulps(float(M(math.sqrt(a / (2 * math.pi)) * math.exp(-f(a, m) - c)) * fraction), cdf)
                for f in FORMS.values()]
        print(f"   {ref:5} {k:4} {m / a:7.4f} " + "  ".join(f"{e:20.0f}" for e in errs))

    print("2. The cdf: bit-exact / within 4 ulp / within 15 digits of n")
    eng = {**{("fit", r): v for stem in ("t4291-poisson-far-above-mean", "t4302-poisson-cdf-beside-pmf")
              for r, v in engine(stem, "probes").items() if stem.startswith("t4291") or int(r[1:]) >= 19},
           **{("fresh", r): v for r, v in engine(FRESH, "probes-numeric").items()}}
    fresh = fresh_points()
    for split, pts in (("fit", fit), ("fresh", fresh)):
        for lo, hi, label in ((1.3, 1.5, "x/a 1.3..1.5"), (0.0, 1.3, "x/a < 1.3")):
            sel = [p for p in pts if lo <= p[2] / (p[1] + 1) <= hi and not (hi == 1.3 and p[2] / (p[1] + 1) >= 1.3)]
            if not sel:
                continue
            for name in ("engine", "correctly rounded"):
                ex = w4 = d15 = 0
                outside = []
                for ref, k, m, cdf in sel:
                    q = t4302.exact(k + 1.0, m)[0]
                    v = eng.get((split, ref), math.nan) if name == "engine" else float(q)
                    u = abs(ulps(v, cdf))
                    ex += u == 0
                    w4 += u <= 4
                    same15 = float(f"{v:.15g}") == float(f"{cdf:.15g}")
                    d15 += same15
                    if not (same15 or abs(M(v) / q - 1) <= M("1e-9")):
                        outside.append(ref)
                env = f"; Gate A envelope {len(sel) - len(outside)}/{len(sel)}" if split == "fresh" else ""
                print(f"   {split:6} {label:13} {name:17} {ex}/{w4}/{d15} of {len(sel)}{env}")
    print("3. Fresh sweep cells by x/a: engine and correctly rounded Q, ulp from Excel")
    for ref, k, m, cdf in sorted(fresh, key=lambda p: p[2] / (p[1] + 1)):
        q = float(t4302.exact(k + 1.0, m)[0])
        print(f"   {ref:4} {k:4} {m / (k + 1):7.4f} {ulps(eng.get(('fresh', ref), math.nan), cdf):6.0f} {ulps(q, cdf):6.0f}")


if __name__ == "__main__":
    main()
