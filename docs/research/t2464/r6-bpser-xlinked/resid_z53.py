"""GL53 with z stored (z53) on r5's probe: per (b0, center) the misses at each x, the a0 where both x miss against
the count expected if independent, signed offsets, and which of s / E / z53 moved by |k| <= 2 ulps fixes each miss
(E re-evaluated as EXP(z53 + k ulps) for the z column).
usage: resid_z53.py"""
import math
from collections import Counter, defaultdict

import common as cm

Z53 = cm.lib.V(lg="GL53", z="z53")


def main() -> None:
    miss = defaultdict(dict)
    offs, fix = defaultdict(Counter), defaultdict(Counter)
    for blk, x, a, b, e in cm.r5_rows():
        p = cm.pieces(a, b, x)
        e53 = cm.EXP(p["z53"])
        m = cm.sx("*", e53, p["s"])
        assert m == cm.lib.kernel(a, b, x, 1.0 - x, Z53)[0]
        b0, c, xs = blk.split(":")
        miss[(b0, c)][(a, xs)] = m != e
        if m == e:
            continue
        offs[(b0, c)][round((m - e) / math.ulp(e))] += 1
        fx = []
        for k in (-2, -1, 1, 2):
            if cm.sx("*", e53, cm.step(p["s"], k)) == e:
                fx.append(f"s{k:+d}")
            if cm.sx("*", cm.step(e53, k), p["s"]) == e:
                fx.append(f"E{k:+d}")
            if cm.sx("*", cm.EXP(cm.step(p["z53"], k)), p["s"]) == e:
                fx.append(f"z{k:+d}")
        fix[(b0, c)][" ".join(fx) or "none"] += 1
    for key in sorted(miss):
        d = miss[key]
        xs = sorted({k[1] for k in d})
        a0s = sorted({k[0] for k in d})
        m1 = sum(d[(a, xs[0])] for a in a0s)
        m2 = sum(d[(a, xs[1])] for a in a0s)
        both = sum(d[(a, xs[0])] and d[(a, xs[1])] for a in a0s)
        print(f"{':'.join(key):9} miss {m1:>3} {m2:>3} both {both:>3} indep {m1 * m2 / len(a0s):5.1f}"
              f"  offsets {dict(sorted(offs[key].items()))}")
        print("    fixes", dict(fix[key].most_common(6)))


if __name__ == "__main__":
    main()
