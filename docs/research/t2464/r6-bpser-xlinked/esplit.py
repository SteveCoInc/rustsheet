"""Separate E from s with measured EXP (t3769-r6-bpser-exp). First the positive control: the EXP model against
=EXP(z) at every probed z. Then, on the cells where z = alx - L is exactly a double (so the worksheet EXP sees the
same argument bpser does, with L GL53's), does fl(E_measured * s) reproduce Excel's
BETA.DIST? If it misses where the model misses, E is not the culprit: s or the product is.
usage: esplit.py"""
import json
from collections import Counter, defaultdict

import common as cm

sx, mpf = cm.sx, cm.mpf
G_SHIFT: dict = {}  # GL53 (repr_r6.out: GAMMALN + 1 ulp at a0 = 0.5 is not Excel's L)


def measured_exp() -> dict:
    out = cm.sidecar("t3769-r6-bpser-exp")
    got = {}
    for i, _, z, _, _ in json.load(open(cm.HERE / "t3769-r6-bpser-exp.meta.json")):
        c = out.get(f"A{i}")
        if c and c.get("t") == "n":
            got[z] = float(c["v"])
    return got


def main() -> None:
    ex = measured_exp()
    print(f"EXP control: model == Excel on {sum(cm.EXP(z) == v for z, v in ex.items())} / {len(ex)}")
    tab = defaultdict(Counter)
    for part, b, a, x, e in cm.beta_rows():
        g = cm.step(cm.GAMMALN(a), G_SHIFT.get(a, 0))
        L = sx("+", g, sx("-", cm.GAMMALN(b), cm.GAMMALN(sx("+", a, b))))
        alx = sx("*", a, cm.LN(x))
        z = mpf(alx) - mpf(L)
        if mpf(float(z)) != z or float(z) not in ex:
            continue
        s = cm.MODEL.series_w3(a, b, x)
        model = sx("*", cm.EXPX(cm.fl64(z)), s)
        meas = sx("*", ex[float(z)], s)
        key = f"{part} b0={b} a0={a}"
        tab[key]["n"] += 1
        tab[key]["model exact"] += model == e
        tab[key]["E measured exact"] += meas == e
        tab[key]["E model == measured"] += cm.EXPX(cm.fl64(z)) == ex[float(z)]
        if meas != e:
            f = [f"s{j:+d}" for j in (-2, -1, 1, 2) if sx("*", ex[float(z)], cm.step(s, j)) == e]
            tab[key]["miss fixed by " + (" ".join(f) or "no s step")] += 1
    for k in sorted(tab, key=lambda k: (k.split("a0=")[1], k)):
        print(k, dict(tab[k]))


if __name__ == "__main__":
    main()
