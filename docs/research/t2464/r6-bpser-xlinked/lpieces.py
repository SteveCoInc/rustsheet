"""Where does the implied L at a0 = 0.5 (lcover_r6.out: ~0.8-1.0 units of 2^-53 above GL53's) come from? For each
shape print, in units of 2^-53 from the exact value, Excel's GAMMALN at a0, b0, fl(a0 + b0), GL53's L and the best
constant L from lcover_r6 (recomputed here).
usage: lpieces.py"""
from collections import defaultdict

import mpmath

import common as cm
import lcover_r6 as lc

mpmath.mp.dps = 40
U = mpmath.mpf(2) ** -53


def main() -> None:
    shapes = defaultdict(list)
    for part, b, a, x, e in cm.beta_rows():
        shapes[(part, b, a)].append((x, e))
    for (part, b, a), cells in sorted(shapes.items(), key=lambda kv: (kv[0][2], kv[0][1])):
        ab = cm.sx("+", a, b)
        offs = []
        for v in (a, b, ab):
            ex = mpmath.loggamma(mpmath.mpf(v))
            offs.append(f"{float((cm.mpf(cm.GAMMALN(v)) - ex) / U):+.3f}")
        ivs = [iv for iv in (lc.lscan.l_interval(a, b, x, e) for x, e in cells) if iv]
        _, bl = lc.best_cover(ivs)
        lex = mpmath.log(mpmath.beta(mpmath.mpf(a), mpmath.mpf(b)))
        # the exact ln B at the double a0 + b0 actually used: lnG(a0) + lnG(b0) - lnG(fl(a0 + b0))
        lfl = mpmath.loggamma(a) + mpmath.loggamma(b) - mpmath.loggamma(mpmath.mpf(ab))
        print(f"{part} b0={b} a0={a}: GL err a0/b0/a0+b0 = {'/'.join(offs)};  L: GL53 {float((cm.MODEL.lnb_w3(a, b) - lex) / U):+.3f}"
              f"  best {float(((bl[0] + bl[1]) / 2 - lex) / U):+.3f}  exact-at-fl(a0+b0) {float((lfl - lex) / U):+.3f}"
              f"  ulp(L) {cm.math.ulp(float(lex)) / 2**-53:.3f}")


if __name__ == "__main__":
    main()
