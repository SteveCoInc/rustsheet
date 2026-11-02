"""T-3582 r6-bgrat-q preg: which stored double makes Excel's P residual periodic on the b = 0.7 runs of probe
t3582-r6-gamma-runs? Regress the relative residual against the exact P (units of 2^-53) on the rounding error of
candidate stored intermediates (units of 2^-53; relative or absolute as named), dreg.py's method (r5-bgrat-fdf1)."""
import json
import math
import sys
from collections import defaultdict
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
from common import CASES, num  # noqa: E402  (r3-bgrat's readers, on the path above)

NAME = "t3582-r6-gamma-runs"
U = 2.0 ** -53
mpmath.mp.dps = 40
sh = json.load(open(CASES / f"{NAME}.oracle.json"))["sheets"]["Calc"]
runs = defaultdict(list)
for k, b, z, run in json.load(open(HERE / f"{NAME}.meta.json")):
    e = num(sh.get(f"A{k}"))
    if e is not None:
        runs[run].append((z, b, e))


def ab(stored, exact):
    return float(mpmath.mpf(stored) - exact) / U


def rel(stored, exact):
    return float((mpmath.mpf(stored) - exact) / exact) / U


for run, pts in sorted(runs.items()):
    eps, rows = [], []
    for z, b, e in pts:
        Z, B = mpmath.mpf(z), mpmath.mpf(b)
        ex = mpmath.gammainc(B, 0, Z, regularized=True)
        eps.append(float((mpmath.mpf(e) - ex) / ex) / U)
        lz = math.log(z)
        blz = b * lz
        zb = math.exp(blz)
        rows.append({
            "ln z abs": ab(lz, mpmath.log(Z)),
            "b*ln z abs (stored ln z)": ab(blz, B * mpmath.mpf(lz)),
            "z^b=exp(b ln z) rel": rel(zb, mpmath.exp(B * mpmath.mpf(lz)) if False else Z ** B),
            "pow(z,b) rel": rel(z ** b, Z ** B),
            "b ln z - z abs": ab(blz - z, B * mpmath.log(Z) - Z),
            "e^-z rel": rel(math.exp(-z), mpmath.exp(-Z)),
            "ln z - z abs": ab(lz - z, mpmath.log(Z) - Z),
        })
    e = np.array(eps)
    k = np.arange(len(e)) / len(e)
    base = np.column_stack([np.ones_like(k), k])
    r0 = e - base @ np.linalg.lstsq(base, e, rcond=None)[0]
    print(f"\n{run}: eps mean {e.mean():+.2f} sd {e.std():.2f}; const + slope residual sd {r0.std():.3f}")
    for nm in rows[0]:
        col = np.array([r[nm] for r in rows])
        X = np.column_stack([base, col])
        coef, *_ = np.linalg.lstsq(X, e, rcond=None)
        print(f"  + {nm:26s} coef {coef[2]:+8.3f}  feat sd {col.std():6.3f}  residual sd {(e - X @ coef).std():.3f}")
