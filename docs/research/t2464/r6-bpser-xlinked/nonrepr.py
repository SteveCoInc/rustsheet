"""On the cells where z = alx - L is not a double (a0 0.3 and 0.7, where GL53's L fits): which of E = EXP(extended
z) (z64), EXP(z rounded down) and EXP(z rounded up) gives Excel's I, per shape; and whether E64 lies strictly between
the two double-argument values (so Excel's E is one of the neighbours' or something else); then the same by
z's position above the double below it, in ulps of that double.
usage: nonrepr.py"""
import math
from collections import Counter, defaultdict

import common as cm

sx, mpf = cm.sx, cm.mpf


def main() -> None:
    tab = defaultdict(Counter)
    for part, b, a, x, e in cm.beta_rows():
        if a == 0.5:
            continue
        s = cm.MODEL.series_w3(a, b, x)
        alx = sx("*", a, cm.LN(x))
        L = cm.MODEL.lnb_w3(a, b)
        z = mpf(alx) - mpf(L)
        if mpf(float(z)) == z:
            continue
        n = float(z)
        lo = n if mpf(n) < z else math.nextafter(n, -math.inf)
        hi = math.nextafter(lo, math.inf)
        e64, elo, ehi = cm.EXPX(cm.fl64(z)), cm.EXP(lo), cm.EXP(hi)
        ok = "".join(t for t, E in (("X", e64), ("d", elo), ("u", ehi)) if sx("*", E, s) == e) or "none"
        rel = "E64=lo" if e64 == elo else "E64=hi" if e64 == ehi else "E64 between" if elo < e64 < ehi else "E64 out"
        q = float((z - mpf(lo)) / mpf(math.ulp(lo)))
        tab[f"{part} b0={b} a0={a}"][f"{rel}: {ok}"] += 1
        tab[f"{part} b0={b} a0={a} by z position"][f"{q:.3g}: {ok}"] += 1
    for k in sorted(tab):
        print(k, dict(sorted(tab[k].items())))


if __name__ == "__main__":
    main()
