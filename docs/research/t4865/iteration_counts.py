"""T-4865: replay AS 109 (as revised by AS R19; the loop of docs/research/t2555/as109_replay.py, here
with the published start clamp to [1E-4, 0.9999] as an option) on the class 1 and 2 cells of
accuracy/BETA.INV/s1 and BETAINV/s1, counting Newton iterations, to test whether an iteration cap
separates Excel's end values from its #NUM!s.

Run: cd tools && uv run python ../docs/research/t4865/iteration_counts.py
"""

import json
import math
import pathlib
import sys

import mpmath

mpmath.mp.dps = 60
ROOT = pathlib.Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "docs/research/t2555"))
import as109_replay  # noqa: E402

END_HI = 1 - 2**-53
END_LO = 2**-53


def cdf(x, a, b):
    return float(mpmath.betainc(a, b, 0, x, regularized=True))


def lbeta(a, b):
    return float(mpmath.log(mpmath.beta(a, b)))


def as109(alpha, p, q, clamp=True, fpu=1e-300, iters=100000, trace=None, cdf=cdf):
    """Returns (value or None, iterations, inner g-divisions, the most divisions in one step)."""
    beta = lbeta(p, q)
    a, pp, qq, swap = alpha, p, q, False
    if alpha > 0.5:
        a, pp, qq, swap = 1 - alpha, q, p, True
    try:
        x = as109_replay.start(a, pp, qq, beta)
    except OverflowError:
        x = 0.0
    if clamp:
        x = min(max(x, 1e-4), 0.9999)
    if not 0 < x < 1:
        return "start", 0, 0, 0
    r = 1 - pp
    t = 1 - qq
    yprev = 0.0
    adj = 1.0
    acu = max(fpu, 10 ** (-13 - 2.5 / (pp * pp) - 0.5 / (a * a)))
    divs = 0
    kmax = 0
    prev = 1.0
    for it in range(1, iters + 1):
        yy = cdf(x, pp, qq)
        if trace is not None:
            trace.append((x, pp, qq, yy))
        yy = (yy - a) * math.exp(beta + r * math.log(x) + t * math.log1p(-x))
        if yy * yprev <= 0:
            prev = max(abs(adj), fpu)
        g = 1.0
        k = 0
        while True:
            adj = g * yy
            if abs(adj) < prev:
                tx = x - adj
                if 0 <= tx <= 1:
                    if prev <= acu or abs(yy) <= acu:
                        return (1 - x if swap else x), it, divs, max(kmax, k)
                    if tx != 0 and tx != 1:
                        break
            g /= 3
            divs += 1
            k += 1
        kmax = max(kmax, k)
        if tx == x:
            return (1 - x if swap else x), it, divs, kmax
        x = tx
        yprev = yy
    return None, iters, divs, kmax


def cells():
    for fn in ("BETA.INV", "BETAINV"):
        base = ROOT / f"eval/accuracy/tables/{fn}/s1"
        case = json.loads(base.with_suffix(".case.json").read_text())["sheets"][0]["cells"]
        oracle = json.loads(base.with_suffix(".oracle.json").read_text())["sheets"]["Calc"]
        for ref, formula in case.items():
            try:
                p, a, b = (float(v) for v in formula.split("(")[1].rstrip(")").split(","))
            except ValueError:
                continue
            if not (0 < p < 1 and 1e-300 < a < 1e5 and 1e-300 < b < 1e5):
                continue
            ex = oracle.get(ref, {}).get("v")
            if ex in ("#NUM!", END_HI, END_LO) and (min(a, b) < 1e-2):
                yield fn, ref, p, a, b, ex


if __name__ == "__main__":
    for clamp in (True, False):
        print("clamp", clamp)
        for fn, ref, p, a, b, ex in cells():
            v, it, divs, kmax = as109(p, a, b, clamp)
            print(f"  {fn:8} {ref:5} p={p:.3g} a={a:.3g} b={b:.3g} excel={ex!s:22} replay={v!s:22} iters={it} divs={divs} kmax={kmax}")
