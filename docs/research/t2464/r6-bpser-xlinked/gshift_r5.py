"""Does gscan.out's a0 = 0.5 reading (internal lnGamma(a0) = GAMMALN(a0) + 1 ulp) hold for r5's a0 by single ulps
around 0.3, 0.5 and 0.7? Per (b0, center): cells exact with g = GAMMALN(a0) + k ulps, k = -1..2, and how many a0 are
best served (both x exact) by each k.
usage: gshift_r5.py"""
from collections import Counter, defaultdict

import common as cm

sx, mpf = cm.sx, cm.mpf


def main() -> None:
    rows = cm.r5_rows()
    tot = defaultdict(Counter)
    per_a = defaultdict(lambda: defaultdict(Counter))
    for blk, x, a, b, e in rows:
        b0, c, _ = blk.split(":")
        s = cm.MODEL.series_w3(a, b, x)
        alx = sx("*", a, cm.LN(x))
        d = sx("-", cm.GAMMALN(b), cm.GAMMALN(sx("+", a, b)))
        for k in (-1, 0, 1, 2):
            L = sx("+", cm.step(cm.GAMMALN(a), k), d)
            ok = sx("*", cm.EXPX(cm.fl64(mpf(alx) - mpf(L))), s) == e
            tot[(b0, c)][k] += ok
            per_a[(b0, c)][a][k] += ok
    for key in sorted(tot):
        both = Counter()
        for a, cnt in per_a[key].items():
            ks = tuple(k for k in (-1, 0, 1, 2) if cnt[k] == 2)
            both[ks if len(ks) < 4 else "all"] += 1
        print(":".join(key), "exact by k:", dict(tot[key]), " a0 with both x exact at k:",
              dict(sorted(both.items(), key=lambda kv: -kv[1])[:6]))


if __name__ == "__main__":
    main()
