"""T-3771: pin bpser's lnGamma(a0) double g on r5-bpser-lg's probe (t3570-r5-bpser-lg, a0 by single ulps at seven
centers, three b0, 8 x per a0) by the double-z rule of T-3769 (docs/research/t2464/r6-bpser-xlinked): GL53 with the
right L is exact on every cell whose z = fl(a0 LN x) - L is a double, and its remaining misses sit only on non-double
z (EXP of an argument needing 54-55 bits, unsolved, T-3873). So a candidate g is judged on its double-z cells only.

For each a0 and each double g within +-R ulps of Excel's GAMMALN(a0): the cells whose z is a double under that g, and
whether all of them are exact ("consistent"). Reports, per center and tune/hold-out (r5-bpser-lg's md5 split of a0,
fixed there): how many a0 have exactly one consistent g with at least MIN double-z cells, and where the candidate
forms fall (gl = Excel's GAMMALN, g1ln53 = fl(gamln1(a0) - LN a0), acc = correctly rounded).
usage: gpin.py [--list]"""
from __future__ import annotations

import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r5-bpser-lg"))
import pin  # noqa: E402  (r5-bpser-lg's loader and model; its sys.path line brings r2-bpser-small's lib)

lib, sx, mpf = pin.lib, pin.sx, pin.mpf
R = 4
MIN = 1  # a g is judged only where it has at least this many double-z cells


def judge(p, g):
    """(double-z cells, double-z exact, non-double cells, non-double exact) for g."""
    a0, b0 = p["a0"], p["b0"]
    L = sx("+", g, sx("-", p["glb"], p["glab"]))
    out = [0, 0, 0, 0]
    for x, e in p["cells"]:
        if e is None:
            continue
        z = mpf(sx("*", a0, pin.LN(x))) - mpf(L)
        ok = sx("*", lib.EXPX(lib.fl64(z)), lib.Model.series_w3(a0, b0, x)) == e
        i = 0 if mpf(float(z)) == z else 2
        out[i] += 1
        out[i + 1] += ok
    return out


def main() -> None:
    pts = pin.load()
    tally = defaultdict(Counter)
    for p in pts:
        cs = pin.cands(p)
        offs = {nm: pin.off(v, p["gl"]) for nm, v in cs.items()}
        consistent, nd = [], {}
        for k in range(-R, R + 1):
            g = p["gl"]
            for _ in range(abs(k)):
                g = math.nextafter(g, math.inf if k > 0 else -math.inf)
            dz, dzok, nz, nzok = judge(p, g)
            nd[k] = (dz, dzok, nz, nzok)
            if dz >= MIN and dz == dzok:
                consistent.append(k)
        t = tally[(f"{p['a0']:.2f}", "hold" if pin.hold(p["a0"]) else "tune")]
        t["a0"] += 1
        t["one"] += len(consistent) == 1
        t["none"] += not consistent
        for nm, o in offs.items():
            t[nm] += o in consistent
            if len(consistent) == 1:
                t[nm + "=pin"] += consistent == [o]
        if "--list" in sys.argv:
            print(f"a0={p['a0']!r} b0={p['b0']} consistent={consistent} cands={offs} "
                  f"gl:{nd[offs['gl']] if offs['gl'] in nd else '-'}")
    cols = ["a0", "one", "none", "gl", "g1ln53", "acc", "gl=pin", "g1ln53=pin", "acc=pin"]
    print("a0 consistent with each form (all double-z cells exact); '=pin' where it is the only consistent g")
    print(f"{'center':6} {'split':5} " + " ".join(f"{c:>10}" for c in cols))
    tot = defaultdict(Counter)
    for key in sorted(tally):
        print(f"{key[0]:6} {key[1]:5} " + " ".join(f"{tally[key][c]:>10}" for c in cols))
        tot[key[1]].update(tally[key])
    for s in sorted(tot):
        print(f"{'all':6} {s:5} " + " ".join(f"{tot[s][c]:>10}" for c in cols))


if __name__ == "__main__":
    main()
