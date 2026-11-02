"""T-3582 r6-bgrat-q pnoise: the shape of Excel's P residual on each run of probe t3582-r6-gamma-runs: Excel - CR in
ulps of P (histogram), lag autocorrelation and FFT peak of the relative residual against the exact P."""
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
from common import CASES, num  # noqa: E402  (r3-bgrat's readers, on the path above)

import math  # noqa: E402

NAME = "t3582-r6-gamma-runs"
mpmath.mp.dps = 40
sh = json.load(open(CASES / f"{NAME}.oracle.json"))["sheets"]["Calc"]
runs = defaultdict(list)
for k, b, z, run in json.load(open(HERE / f"{NAME}.meta.json")):
    e = num(sh.get(f"A{k}"))
    if e is not None:
        runs[run].append((z, b, e))
for run, pts in runs.items():
    du, rel = [], []
    for z, b, e in pts:
        ex = mpmath.gammainc(b, 0, z, regularized=True)
        cr = float(ex)
        du.append(round((e - cr) / math.ulp(cr)))
        rel.append(float((mpmath.mpf(e) - ex) / ex) / 2 ** -53)
    r = np.array(rel) - np.mean(rel)
    ac = [np.corrcoef(r[:-lag], r[lag:])[0, 1] for lag in (1, 2, 8)]
    F = np.abs(np.fft.rfft(r))
    print(run, f"P ~ {pts[0][2]:.4f}, ulp(P)/P = {math.ulp(pts[0][2]) / pts[0][2] / 2 ** -53:.2f} units;",
          "Excel - CR in ulps:", dict(sorted(Counter(du).items())),
          "| lag1/2/8 autocorr", " ".join(f"{a:+.2f}" for a in ac), f"| FFT max {F[1:].max() / F[1:].mean():.1f}x mean")
