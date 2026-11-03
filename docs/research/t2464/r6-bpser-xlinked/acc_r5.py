"""On r5's probe: compare GL53's stored s and E with their correctly rounded exact values (s: the exact series
1/a + sum c_n/(a+n) at the double a, b, x; E: exp of the model's own extended z), split by exact/miss.
Says which piece the model gets wrong, if either, on the cells it misses.
usage: acc_r5.py"""
from collections import Counter, defaultdict

import mpmath

import common as cm

mpmath.mp.dps = 50


def s_exact(a, b, x):
    a, b, x = mpmath.mpf(a), mpmath.mpf(b), mpmath.mpf(x)
    s, c, n = 1 / a, mpmath.mpf(1), 0
    while True:
        n += 1
        c *= (1 - b / n) * x
        t = c / (a + n)
        s += t
        if abs(t) < mpmath.mpf(10) ** -45:
            return s


def ulp_off(v, ex):
    r = float(ex)
    return round((v - r) / cm.math.ulp(r))


def main() -> None:
    tab = defaultdict(Counter)
    for blk, x, a, b, e in cm.r5_rows():
        p = cm.pieces(a, b, x)
        ds = ulp_off(p["s"], s_exact(a, b, x))
        de = ulp_off(p["e"], mpmath.exp(p["z"]))
        tab[(blk, "exact" if p["i"] == e else "miss")][f"s{ds:+d} E{de:+d}"] += 1
    for k in sorted(tab):
        print(k, dict(tab[k].most_common()))


if __name__ == "__main__":
    main()
