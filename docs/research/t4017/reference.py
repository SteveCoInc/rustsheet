"""T-4017 references: high-precision mpmath F upper tail at the double arguments Excel sees.

F.DIST.RT(x, d1, d2) = betainc(d1/2, d2/2, u, 1) with u = fl(d1 x / (d1 x + d2)), the
same small-side argument as T-2528 (the large side's double rounding is amplified by d2/2). Writes references.json (50 digits) and
prints per-df1 counts: correctly rounded, within 4 ulp, within 1E-15 relative, largest
relative error, over the F.DIST.RT cells of probes-numeric/t4017-f-huge-df2-above-mean-*.

Run: python3 -I docs/research/t4017/reference.py  (needs mpmath)
"""

import collections
import json
import math
import re
from pathlib import Path

import mpmath as mp

mp.mp.dps = 120


def _upper_tail_at(d1: float, d2: float, u: float, dps: int) -> "mp.mpf":
    with mp.workdps(dps):
        a, b, uu = mp.mpf(d1) / 2, mp.mpf(d2) / 2, mp.mpf(u)
        lead = mp.exp(a * mp.log(uu) + mp.loggamma(a + b) - mp.loggamma(a) - mp.loggamma(b)) / a
        return +(1 - lead * mp.hyp2f1(a, 1 - b, a + 1, uu))


def upper_tail(d1: float, d2: float, u: float) -> "mp.mpf":
    """The upper tail 1 - I_u(d1/2, d2/2) via the 2F1 series for the lower one. The series
    alternates with terms up to e^(d1 x / 2) and the tail is as small as e^-(d1 x / 2), so the
    working precision grows with d1 x; the value is kept only if two precisions agree to 55
    digits (mp.betainc returns 0 or garbage for the tails below 1E-130). Else returns None."""
    size = d1 * d2 * u / (d2 * (1 - u))  # d1 x
    base = 300 + int(2.0 * size)
    lo, hi = _upper_tail_at(d1, d2, u, base), _upper_tail_at(d1, d2, u, base + 200)
    if hi > 0 and abs(lo / hi - 1) < mp.mpf(10) ** -55:
        return hi
    return None


ROOT = Path(__file__).resolve().parents[3]
CASES = ROOT / "eval/suites/probes-numeric/cases"
refs = {}
errors = []
zeros = []
unreferenced = []
stats = collections.defaultdict(lambda: [0, 0, 0, 0, 0.0])
for case in sorted(CASES.glob("t4017-f-huge-df2-above-mean-*.case.json")):
    cells = json.loads(case.read_text())["sheets"][0]["cells"]
    excel = json.loads(case.with_suffix("").with_suffix(".oracle.json").read_text())["sheets"]["Calc"]
    for addr, formula in cells.items():
        m = re.fullmatch(r"=F\.DIST\.RT\(([\d.]+),(\d+),([\dE+]+)\)", formula)
        if not m:
            continue
        x, d1, d2 = float(m[1]), float(m[2]), float(m[3])
        scaled = d1 * x
        u = scaled / (scaled + d2)
        ref = upper_tail(d1, d2, u)
        e = excel[addr]
        if e["t"] == "n" and float(e["v"]) == 0:
            # Excel's exact zeros: the chi-square limit Q(d1/2, d1 x/2) (d2 -> infinity) estimates
            # the true size to about d1 x / d2 relative, enough to show it is above or below the minimum.
            with mp.workdps(60):
                est = mp.gammainc(mp.mpf(d1) / 2, mp.mpf(d1 * x) / 2, mp.inf, regularized=True)
            zeros.append((case.name[28:], addr, formula, mp.nstr(est, 3)))
        if ref is None:
            unreferenced.append((case.name, addr, formula))
            continue
        refs[f"{case.name}:{addr}"] = {"f": formula, "v": mp.nstr(ref, 50)}
        s = stats[int(d1)]
        s[0] += 1
        if e["t"] != "n":
            errors.append((case.name, addr, formula, e["v"]))
            continue
        got = float(e["v"])
        if ref == 0:
            print("zero reference", case.name, addr, formula, got)
            continue
        r = float(ref)
        err = abs(got - r)
        if err <= 0.5 * math.ulp(r):
            s[1] += 1
        if err <= 4 * math.ulp(r):
            s[2] += 1
        rel = float(abs(mp.mpf(got) / ref - 1))
        if rel > 1e-12:
            print("far", case.name[28:], addr, formula, got, mp.nstr(ref, 8))
        if rel < 1e-15:
            s[3] += 1
        s[4] = max(s[4], rel)
(ROOT / "docs/research/t4017/references.json").write_text(json.dumps(refs, indent=1) + "\n")
for z in zeros:
    print("excel zero", *z)
for cell in unreferenced:
    print("no stable reference", *cell)
for err in errors:
    print("error cell", *err)
print("d1 cells correctly_rounded within_4ulp within_1e-15 max_rel_err")
for d1 in sorted(stats):
    print(d1, *stats[d1])
