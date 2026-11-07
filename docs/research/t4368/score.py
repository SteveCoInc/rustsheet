"""T-4368: score Excel's gamma upper tail at integer and half-integer shapes against 50-digit math.

Run from the repository root with the tools venv: tools/.venv/bin/python docs/research/t4368/score.py

Splits, fixed before scoring (gen_probe.py): train rows 1:9, hold-out 10:18, fresh 19:42, edge rows 43:94
(m stepped across 1.0a). For each column it prints, per split, bit-exact / within 4 ulp / max |ulp| of Excel's
value from the correctly rounded truth, then whether POISSON.DIST equals CHISQ.DIST.RT(2m, 2a) bit for bit, and
the edge rows' signed ulp errors in order of m / a.
"""

import importlib.util
import json
import math
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
# The points, fixed before scoring, come from the generator itself.
_spec = importlib.util.spec_from_file_location("t4368_gen", HERE / "gen_probe.py")
assert _spec and _spec.loader
gen_probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen_probe)

mp.mp.dps = 50
STEM = ROOT / "eval/suites/probes/cases" / gen_probe.STEM
SPLITS = {"train": range(1, 10), "hold-out": range(10, 19), "fresh": range(19, 43), "edge": range(43, 95)}


def oracle():
    data = json.loads(STEM.with_suffix(".oracle.json").read_text())
    return {ref: v["v"] for ref, v in data["sheets"]["Calc"].items() if isinstance(v, dict) and v.get("t") == "n"}


def truths(k, m):
    """Exact values of columns A:E (gen_probe.py's docstring)."""
    a, x = mp.mpf(k + 1), mp.mpf(m)
    half = mp.mpf(1) / 2
    q = mp.gammainc(a, x, mp.inf, regularized=True)
    return {
        "A": q,
        "B": q,
        "C": mp.gammainc(a - half, x, mp.inf, regularized=True),
        "D": mp.gammainc(a + half, x, mp.inf, regularized=True),
        "E": mp.gammainc(a + half, 0, x, regularized=True),
    }


def ulps(value, truth):
    """Signed distance of `value` from `truth` in ulps of the correctly rounded truth."""
    rounded = float(truth)
    return float((mp.mpf(value) - truth) / mp.mpf(math.ulp(rounded)))


def main():
    excel = oracle()
    pts = gen_probe.points()
    errs = {}
    for row, (k, m) in enumerate(pts, start=1):
        t = truths(k, m)
        for col in gen_probe.COLUMNS:
            v = excel.get(f"{col}{row}")
            errs[(col, row)] = None if v is None else ulps(v, t[col])
    for col in gen_probe.COLUMNS:
        parts = []
        for name, rows in SPLITS.items():
            e = [errs[(col, r)] for r in rows if errs[(col, r)] is not None]
            exact = sum(abs(x) < 0.5 for x in e)
            near = sum(abs(x) <= 4 for x in e)
            parts.append(f"{name} {exact}/{near}/{max(abs(x) for x in e):.0f} of {len(e)}")
        print(f"{col}: " + "; ".join(parts))
    same = sum(excel.get(f"A{r}") == excel.get(f"B{r}") for r in range(1, len(pts) + 1))
    print(f"POISSON.DIST == CHISQ.DIST.RT(2m,2a) bit for bit: {same} of {len(pts)}")
    for k in gen_probe.EDGE_K:
        rows = [r for r, (kk, _) in enumerate(pts, start=1) if r >= 43 and kk == k]
        line = " ".join(f"{pts[r - 1][1] / (k + 1):.3f}:{errs[('A', r)]:+.0f}" for r in rows)
        print(f"edge k={k} A (m/a:ulp): {line}")
        line = " ".join(f"{errs[('D', r)]:+.0f}" for r in rows)
        print(f"edge k={k} D: {line}")


if __name__ == "__main__":
    main()
