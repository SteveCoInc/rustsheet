"""T-4357: score models of POISSON.DIST's cdf below 1.3a built from Excel's own companion values at the same point.

Run from the repository root with the tools venv: tools/.venv/bin/python docs/research/t4357/score.py

Splits, fixed before scoring (gen_probe.py): train rows 1:9, hold-out rows 10:18, fresh rows 19:42. Each model
takes one Excel companion cell of the row as given and the rest in 50-digit math, so a model that is Excel's
says which step carries Excel's error. Excel's cdf is column A.
"""

import json
import math
import sys
from pathlib import Path

import mpmath as mp

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_probe  # noqa: E402

mp.mp.dps = 50
STEM = ROOT / "eval/suites/probes-numeric/cases" / gen_probe.STEM
SPLITS = {"train": range(1, 10), "hold-out": range(10, 19), "fresh": range(19, 43)}


def oracle():
    data = json.loads(STEM.with_suffix(".oracle.json").read_text())
    out = {}
    for ref, v in data["sheets"]["Calc"].items():
        if isinstance(v, dict) and v.get("t") == "n":
            out[ref] = v["v"]
    return out


def exact(k, m):
    a, x = mp.mpf(k + 1), mp.mpf(m)
    q = mp.gammainc(a, x, mp.inf, regularized=True)
    density = mp.exp((a - 1) * mp.log(x) - x - mp.loggamma(a))
    pmf = mp.exp(k * mp.log(x) - x - mp.loggamma(k + 1))
    z = mp.sqrt(a * (x / a - 1 - mp.log(x / a)))
    return q, density, pmf, z


def models(k, m, row):
    q, density, pmf, z = exact(k, m)
    c = lambda col: row[col]  # noqa: E731
    return {
        "correctly rounded Q": lambda: float(q),
        "Excel 1-GAMMA.DIST (C)": lambda: c("C"),
        "1 - Excel GAMMA.DIST (B) in doubles": lambda: 1.0 - c("B"),
        "Excel density (D) times exact Q/density": lambda: float(mp.mpf(c("D")) * (q / density)),
        "Excel pmf (E) times exact Q/pmf": lambda: float(mp.mpf(c("E")) * (q / pmf)),
        "Excel erfc (F)/2 plus exact remainder": lambda: float(mp.mpf(c("F")) / 2 + (q - mp.erfc(z) / 2)),
    }


def main():
    orc = oracle()
    pts = gen_probe.points()
    tallies = {}
    for split, rows in SPLITS.items():
        for r in rows:
            k, m = pts[r - 1]
            row = {col: orc.get(f"{col}{r}") for col in gen_probe.COLUMNS}
            excel = row["A"]
            if excel is None or any(v is None for v in row.values()):
                continue
            for name, f in models(k, m, row).items():
                u = abs(f() - excel) / math.ulp(excel)
                t = tallies.setdefault(name, {s: [0, 0, 0, 0.0] for s in SPLITS})[split]
                t[0] += u <= 0.5
                t[1] += u <= 4
                t[2] += 1
                t[3] = max(t[3], u)
    print("bit-exact / within 4 ulp / cells, max ulp from Excel's cdf")
    for name, by in tallies.items():
        cols = "  ".join(f"{s} {e:2}/{w:2}/{n:2} max {mx:6.0f}" for s, (e, w, n, mx) in by.items())
        print(f"  {name:42} {cols}")


if __name__ == "__main__":
    main()
