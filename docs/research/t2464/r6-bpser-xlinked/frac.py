"""Where the unrounded product sits inside I's ulp, on the a0 = 0.3 shapes (L fits). For each cell: f = (E s - lo)/ulp,
with E the model's stored EXP and the product exact, lo the double below. Histogram of f in tenths for exact cells and
misses, split by I's binade. Near-ties (f ~ 0.5) missed both ways would mean noise in E or s; misses at every f
would mean a systematic shift.
usage: frac.py"""
import math
from collections import Counter, defaultdict

import mpmath

import common as cm

mpmath.mp.prec = 200


def main() -> None:
    hist = defaultdict(Counter)
    for part, b, a, x, e in cm.beta_rows():
        if a != 0.3:
            continue
        p = cm.pieces(a, b, x)
        t = mpmath.mpf(p["e"]) * mpmath.mpf(p["s"])
        lo = math.nextafter(float(t), 0.0) if mpmath.mpf(float(t)) > t else float(t)
        f = float((t - lo) / math.ulp(lo))
        key = (math.frexp(p["i"])[1], "miss" if p["i"] != e else "exact")
        hist[key][min(int(f * 10), 9)] += 1
    for k in sorted(hist):
        print(k, " ".join(f"{hist[k][i]:4d}" for i in range(10)))


if __name__ == "__main__":
    main()
