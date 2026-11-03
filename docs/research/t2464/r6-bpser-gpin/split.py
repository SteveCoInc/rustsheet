"""T-3771: split GL53's misses on t3570-r5-bpser-lg (r5's model, the best single form, lforms.out) between
  LN       Excel's =LN(x) differs from the model's (probe t3771-r6-bpser-gpin-ln; positive control first)
  EXP arg  z = fl(a0 LN x) - L is not a double (T-3769: EXP of a 54-55-bit argument, unsolved, T-3873)
  s        z is a double, and moving w3's series s by one ulp makes the cell exact (E is then Excel's worksheet EXP)
  rest     z is a double and no one-ulp move of s fixes it (L, or more than one piece)
per center, tune/hold-out (pin.hold).
usage: split.py"""
from __future__ import annotations

import json
import math
from collections import Counter, defaultdict
from pathlib import Path

import gpin

pin, lib, sx, mpf = gpin.pin, gpin.lib, gpin.sx, gpin.mpf
CASE = Path(__file__).resolve().parents[4] / "eval/suites/probes-numeric/cases/t3771-r6-bpser-gpin-ln.oracle.json"


def step(u, k):
    for _ in range(abs(k)):
        u = math.nextafter(u, math.inf if k > 0 else -math.inf)
    return u


def main() -> None:
    out = json.loads(CASE.read_text())["sheets"]["Calc"]
    ln = {x: float(out[f"A{i}"]["v"]) for i, x in json.load(open(gpin.HERE / "t3771-r6-bpser-gpin-ln.meta.json"))}
    print(f"LN control: model == Excel on {sum(pin.LN(x) == v for x, v in ln.items())} / {len(ln)}")
    tab = defaultdict(Counter)
    for p in pin.load():
        a0, b0 = p["a0"], p["b0"]
        L = sx("+", p["gl"], sx("-", p["glb"], p["glab"]))
        key = (f"{a0:.2f}", "hold" if pin.hold(a0) else "tune")
        for x, e in p["cells"]:
            if e is None:
                continue
            z = mpf(sx("*", a0, pin.LN(x))) - mpf(L)
            s = lib.Model.series_w3(a0, b0, x)
            E = lib.EXPX(lib.fl64(z))
            t = tab[key]
            t["cells"] += 1
            if sx("*", E, s) == e:
                continue
            if pin.LN(x) != ln[x]:
                t["LN"] += 1
            elif mpf(float(z)) != z:
                t["EXP arg"] += 1
            elif any(sx("*", E, step(s, k)) == e for k in (-1, 1)):
                t["s"] += 1
            else:
                t["rest"] += 1
    cols = ["cells", "LN", "EXP arg", "s", "rest"]
    print(f"{'center':6} {'split':5} " + " ".join(f"{c:>8}" for c in cols))
    tot = defaultdict(Counter)
    for key in sorted(tab):
        tot[key[1]].update(tab[key])
        print(f"{key[0]:6} {key[1]:5} " + " ".join(f"{tab[key][c]:>8}" for c in cols))
    for s, t in sorted(tot.items()):
        print(f"{'all':6} {s:5} " + " ".join(f"{t[c]:>8}" for c in cols))


if __name__ == "__main__":
    main()
