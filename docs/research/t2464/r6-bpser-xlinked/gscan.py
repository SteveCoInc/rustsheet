"""Implied internal lnGamma(a0) per shape: L = fl(g + d), d = fl(GL(b0) - GL(fl(a0 + b0))) (measured GAMMALN), z =
alx - L extended. For each double g within +-4 ulps of Excel's GAMMALN(a0), count the cells exact. Then name the
candidates: GAMMALN (measured), the correctly rounded lnGamma(a0), TOMS's fl(gamln1(a0) - LN(a0)).
usage: gscan.py"""
from collections import defaultdict

import mpmath

import common as cm

sx, mpf = cm.sx, cm.mpf


def main() -> None:
    shapes = defaultdict(list)
    for part, b, a, x, e in cm.beta_rows():
        shapes[(part, b, a)].append((x, e))
    for (part, b, a), cells in sorted(shapes.items(), key=lambda kv: (kv[0][2], kv[0][1])):
        gl = cm.GAMMALN(a)
        d = sx("-", cm.GAMMALN(b), cm.GAMMALN(sx("+", a, b)))
        pre = [(cm.MODEL.series_w3(a, b, x), mpf(sx("*", a, cm.LN(x))), e) for x, e in cells]
        named = {
            "GL": gl,
            "cr": float(mpmath.loggamma(mpmath.mpf(a))),
            "toms": sx("-", cm.MODEL.gamln1(a), cm.LN(a)),
        }
        row = []
        for k in range(-4, 5):
            g = cm.step(gl, k)
            L = mpf(sx("+", g, d))
            n = sum(sx("*", cm.EXPX(cm.fl64(A - L)), s) == e for s, A, e in pre)
            tag = ",".join(nm for nm, v in named.items() if v == g)
            row.append(f"{k:+d}{'[' + tag + ']' if tag else ''}:{n}")
        print(f"{part} b0={b} a0={a}  " + "  ".join(row))


if __name__ == "__main__":
    main()
