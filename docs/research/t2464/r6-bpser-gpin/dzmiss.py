"""Where the double-z rule breaks (gpin.out: a0 with no consistent g, mostly near 0.4-0.6): at the g with the most
exact double-z cells per a0, the double-z misses by center, b0 and x, their signed ulp offsets, and whether moving s
(the series) or E (Excel's EXP of the double z, the worksheet value) by one ulp fixes them.
usage: dzmiss.py"""
from __future__ import annotations

import math
from collections import Counter, defaultdict

import gpin

pin, lib, sx, mpf = gpin.pin, gpin.lib, gpin.sx, gpin.mpf


def step(u, k):
    for _ in range(abs(k)):
        u = math.nextafter(u, math.inf if k > 0 else -math.inf)
    return u


def main() -> None:
    by = defaultdict(Counter)
    fix = defaultdict(Counter)
    for p in pin.load():
        best = max(range(-gpin.R, gpin.R + 1), key=lambda k: (gpin.judge(p, step(p["gl"], k))[1], -abs(k)))
        g = step(p["gl"], best)
        L = sx("+", g, sx("-", p["glb"], p["glab"]))
        c = f"{p['a0']:.2f}"
        for x, e in p["cells"]:
            if e is None:
                continue
            z = mpf(sx("*", p["a0"], pin.LN(x))) - mpf(L)
            if mpf(float(z)) != z:
                continue
            E = lib.EXP(float(z))
            s = lib.Model.series_w3(p["a0"], p["b0"], x)
            m = sx("*", E, s)
            by[(c, p["b0"])]["cells"] += 1
            if m == e:
                continue
            by[(c, p["b0"])]["miss"] += 1
            by[(c, p["b0"])][f"x={x}"] += 1
            f = [f"s{k:+d}" for k in (-1, 1) if sx("*", E, step(s, k)) == e]
            f += [f"E{k:+d}" for k in (-1, 1) if sx("*", step(E, k), s) == e]
            fix[c][" ".join(f) or "none"] += 1
            fix[c][f"off {round((m - e) / math.ulp(e)):+d}"] += 1
    for k in sorted(by):
        d = by[k]
        xs = {kk: v for kk, v in d.items() if kk.startswith("x=")}
        print(f"center {k[0]} b0={k[1]}: double-z misses {d['miss']} / {d['cells']}  by x {dict(sorted(xs.items()))}")
    for c in sorted(fix):
        print(f"center {c}: {dict(fix[c].most_common())}")


if __name__ == "__main__":
    main()
