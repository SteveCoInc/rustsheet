"""Is r5's a0-linked residual the same effect as the x-linked one? On r5's probe (a0 by single ulps), split GL53's
cells by whether z = alx - L is a double, per (b0, center): misses / cells in each part, and, among a0 where both x
miss, how many have a non-double z at both x.
usage: repr_r5.py"""
from collections import Counter, defaultdict

import common as cm

mpf = cm.mpf


def main() -> None:
    tab = defaultdict(Counter)
    per_a = defaultdict(dict)
    for blk, x, a, b, e in cm.r5_rows():
        b0, c, _ = blk.split(":")
        p = cm.pieces(a, b, x)
        rep = mpf(float(p["z"])) == p["z"]
        miss = p["i"] != e
        k = (b0, c)
        tab[k][("double z" if rep else "non-double z", "miss" if miss else "exact")] += 1
        per_a[k].setdefault(a, []).append((miss, rep))
    for k in sorted(tab):
        t = tab[k]
        both = [v for v in per_a[k].values() if all(m for m, _ in v)]
        nn = sum(all(not r for _, r in v) for v in both)
        print(f"{':'.join(k):9} double z: {t[('double z', 'miss')]:>3} miss / {t[('double z', 'miss')] + t[('double z', 'exact')]:>3};"
              f"  non-double z: {t[('non-double z', 'miss')]:>3} miss / {t[('non-double z', 'miss')] + t[('non-double z', 'exact')]:>3};"
              f"  a0 with both x missing: {len(both)}, of them both z non-double: {nn}")


if __name__ == "__main__":
    main()
