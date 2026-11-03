"""esplit.out: GL53 is exact wherever z = alx - L is a double, so the x-linked misses are in how a non-double z
reaches EXP. Score z's rounding: extended (z64, r5's model), nearest double (z53), toward -inf (down), toward +inf (up),
toward zero (z < 0 here, so = up), with E = Excel's EXP of that double and I = fl(E s). Per shape; at a0 = 0.5 with
g = GAMMALN and g = GAMMALN + 1 ulp. Tune b0 2.5 and 6.25, hold-out b0 4.75 (fixed before the fill).
usage: zround.py"""
import math
from collections import defaultdict

import common as cm

sx, mpf = cm.sx, cm.mpf


def rounded(z, mode):
    if mode == "z64":
        return None
    n = float(z)
    if mpf(n) == z or mode == "z53":
        return n
    lo = n if mpf(n) < z else math.nextafter(n, -math.inf)
    return lo if mode == "down" else math.nextafter(lo, math.inf)


def main() -> None:
    modes = ("z64", "z53", "down", "up")
    score = defaultdict(lambda: defaultdict(int))
    for part, b, a, x, e in cm.beta_rows():
        s = cm.MODEL.series_w3(a, b, x)
        alx = sx("*", a, cm.LN(x))
        for gs in ((0, 1) if a == 0.5 else (0,)):
            g = cm.step(cm.GAMMALN(a), gs)
            L = sx("+", g, sx("-", cm.GAMMALN(b), cm.GAMMALN(sx("+", a, b))))
            z = mpf(alx) - mpf(L)
            for m in modes:
                zr = rounded(z, m)
                E = cm.EXPX(cm.fl64(z)) if zr is None else cm.EXP(zr)
                ok = sx("*", E, s) == e
                nm = m + (" g+1" if gs else "")
                score[nm][f"{part[0]}{b}:{a}"] += ok
                score[nm][part] += ok if gs == 0 or a != 0.5 else 0
    keys = sorted({k for d in score.values() for k in d}, key=lambda k: (":" in k, k.split(":")[-1], k))
    print(f"{'z':10}" + "".join(f"{k:>9}" for k in keys))
    for nm, d in score.items():
        print(f"{nm:10}" + "".join(f"{d.get(k, ''):>9}" for k in keys))
    print("(tune/hold totals use g = GAMMALN at a0 = 0.5)")


if __name__ == "__main__":
    main()
