"""Score compositions of bpser's prefactor E from Excel's measured pieces (GL(a0), GL(b0), GL(a0+b0), alx =
fl(a0 LN x)) on t3769-r6-bpser-beta, all with w3's stored series s and I = fl(E s). Bit-exact per shape; tune is
b0 2.5 and 6.25, hold-out b0 4.75 (fixed before the fill). "53" = every op stored, "64" = kept extended.
usage: zcomp.py [--hold]"""
import sys
from collections import defaultdict

import common as cm

sx, fl64, fl53, mpf, EXP, EXPX = cm.sx, cm.fl64, cm.fl53, cm.mpf, cm.EXP, cm.EXPX


def e64(v):
    return EXPX(fl64(v))


def forms(a, b, x):
    g, gb, gab = (cm.GAMMALN(v) for v in (a, b, sx("+", a, b)))
    G, GB, GAB = mpf(g), mpf(gb), mpf(gab)
    alx = sx("*", a, cm.LN(x))
    A = mpf(alx)
    L53 = mpf(sx("+", g, sx("-", gb, gab)))
    L64 = fl64(G + fl64(GB - GAB))
    Lp53 = mpf(sx("-", sx("+", g, gb), gab))
    Lp64 = fl64(fl64(G + GB) - GAB)
    out = {
        "GL53 (z64)": e64(A - L53),
        "z53": EXP(fl53(A - L53)),
        "L64": e64(A - L64),
        "L64 z53": EXP(fl53(A - L64)),
        "(a+b)-ab 53": e64(A - Lp53),
        "(a+b)-ab 64": e64(A - Lp64),
        "seq64 ((alx-g)-gb)+gab": e64(fl64(fl64(A - G) - GB) + GAB),
        "seq53 ((alx-g)-gb)+gab": EXP(sx("+", sx("-", sx("-", alx, g), gb), gab)),
        "(alx-g)-(gb-gab) 64": e64(fl64(A - G) - fl64(GB - GAB)),
        "(alx-g)-(gb-gab) 53": EXP(sx("-", sx("-", alx, g), sx("-", gb, gab))),
        "(alx-g) z64, (gb-gab) 53": e64(fl64(A - G) - mpf(sx("-", gb, gab))),
        "(alx - fl(gb-gab)) - g 64": e64(fl64(A - mpf(sx("-", gb, gab))) - G),
        "EXP(alx-L)?: EXP(alx)/EXP(L)": sx("/", EXP(alx), EXP(float(L53))),
        "EXP(alx)*EXP(-L)": sx("*", EXP(alx), EXP(-float(L53))),
    }
    return out


def main() -> None:
    rows = cm.beta_rows()
    score = defaultdict(lambda: defaultdict(int))
    for part, b, a, x, e in rows:
        s = cm.MODEL.series_w3(a, b, x)
        key = f"{part[0]}{b}:{a}"
        for nm, E in forms(a, b, x).items():
            ok = sx("*", E, s) == e
            score[nm][key] += ok
            score[nm][part] += ok
    keys = sorted({k for d in score.values() for k in d}, key=lambda k: (len(k) > 4, k))
    print(f"{'form':30}" + "".join(f"{k:>9}" for k in keys))
    for nm, d in score.items():
        print(f"{nm:30}" + "".join(f"{d[k]:>9}" for k in keys))


if __name__ == "__main__":
    main()
