"""T-3771: score candidate forms of bpser's L on t3570-r5-bpser-lg by the double-z rule (gpin.py's docstring): per
form and a0, the double-z cells and their misses; a form is consistent at an a0 when it has double-z cells and misses
none. Forms (d = fl(GL(b0) - GL(fl(a0 + b0))), all GL Excel's measured GAMMALN; A = GAMMALN(fl(1 + a0)) from
t3570-r5-bpser-lg-aux column A):
  GL53     fl(GAMMALN(a0) + d)                          (r5's model)
  g1ln53   fl(fl(gamln1(a0) - LN a0) + d)               (TOMS)
  Ag53     fl(fl(A - LN a0) + d)                        (g from GAMMALN(1 + a0), stored)
  Ag64     fl(fl64(A - LN a0) + d)                      (g kept extended, L stored)
  AL64     fl64(fl64(A - LN a0) + d)                    (g and L kept extended)
  GL64     fl64(GAMMALN(a0) + d)                        (L kept extended)
Also "Lscan": does ANY double L within +-24 ulps of GL53's L make every double-z cell exact? (an upper bound for a
stored L). Tune/hold-out: r5-bpser-lg's md5 split of a0 (pin.hold), fixed before this round.
usage: lforms.py"""
from __future__ import annotations

import json
import math
from collections import Counter, defaultdict

import gpin

pin, lib, sx, mpf = gpin.pin, gpin.lib, gpin.sx, gpin.mpf
fl64 = lib.fl64


def aux_a() -> dict:
    out = json.loads((lib.CASES / "t3570-r5-bpser-lg-aux.oracle.json").read_text())["sheets"]["Calc"]
    got = {}
    for row, a0, _, _ in json.load(open(pin.HERE / "t3570-r5-bpser-lg.meta.json")):
        c = out.get(f"A{row}")
        if c and c.get("t") == "n":
            got[a0] = float(c["v"])
    return got


def judge_L(p, L):
    """(double-z cells, double-z misses, all exact) for an L (mpf, double or extended)."""
    dz = miss = ok_all = 0
    for x, e in p["cells"]:
        if e is None:
            continue
        z = fl64(mpf(sx("*", p["a0"], pin.LN(x))) - L)
        ok = sx("*", lib.EXPX(z), lib.Model.series_w3(p["a0"], p["b0"], x)) == e
        ok_all += ok
        if mpf(float(z)) == z:
            dz += 1
            miss += not ok
    return dz, miss, ok_all


def forms(p, A):
    a0 = p["a0"]
    d = sx("-", p["glb"], p["glab"])
    lna = pin.LN(a0)
    g64 = fl64(mpf(A) - mpf(lna))
    return {
        "GL53": mpf(sx("+", p["gl"], d)),
        "g1ln53": mpf(sx("+", sx("-", pin.K.gamln1(a0), lna), d)),
        "Ag53": mpf(sx("+", sx("-", A, lna), d)),
        "Ag64": mpf(float(fl64(g64 + mpf(d)))),
        "AL64": fl64(g64 + mpf(d)),
        "GL64": fl64(mpf(p["gl"]) + mpf(d)),
    }


def main() -> None:
    aa = aux_a()
    tally = defaultdict(Counter)
    for p in pin.load():
        A = aa.get(p["a0"])
        if A is None:
            continue
        key = (f"{p['a0']:.2f}", "hold" if pin.hold(p["a0"]) else "tune")
        t = tally[key]
        t["a0"] += 1
        fs = forms(p, A)
        for nm, L in fs.items():
            dz, miss, ok = judge_L(p, L)
            t[nm] += dz > 0 and miss == 0
            t[nm + ":ex"] += ok
        base = float(fs["GL53"])
        L = base
        for _ in range(24):
            L = math.nextafter(L, -math.inf)
        anyok = False
        for _ in range(49):
            dz, miss, _ = judge_L(p, mpf(L))
            anyok |= dz > 0 and miss == 0
            L = math.nextafter(L, math.inf)
        t["Lscan"] += anyok
    names = ["GL53", "g1ln53", "Ag53", "Ag64", "AL64", "GL64"]
    print("a0 consistent (double-z cells all exact) per form; then bit-exact cells per form (8 per a0)")
    print(f"{'center':6} {'split':5} {'a0':>4} {'Lscan':>6} " + " ".join(f"{n:>7}" for n in names) + " | "
          + " ".join(f"{n:>7}" for n in names))
    tot = defaultdict(Counter)
    for key in sorted(tally):
        t = tally[key]
        tot[key[1]].update(t)
        print(f"{key[0]:6} {key[1]:5} {t['a0']:>4} {t['Lscan']:>6} " + " ".join(f"{t[n]:>7}" for n in names) + " | "
              + " ".join(f"{t[n + ':ex']:>7}" for n in names))
    for s, t in sorted(tot.items()):
        print(f"{'all':6} {s:5} {t['a0']:>4} {t['Lscan']:>6} " + " ".join(f"{t[n]:>7}" for n in names) + " | "
              + " ".join(f"{t[n + ':ex']:>7}" for n in names))


if __name__ == "__main__":
    main()
