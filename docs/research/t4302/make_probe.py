"""T-4302: write probes/t4302-poisson-cdf-beside-pmf (run once; the case is committed).

Next to each far-above-the-mean POISSON.DIST cdf, the same point's own chains: the pmf
POISSON.DIST(k, mean, FALSE), the gamma density GAMMA.DIST(mean, k + 1, 1, FALSE) (the same value
mathematically) and the chi-square composition CHISQ.DIST.RT(2 mean, 2k + 2) (the same value as the cdf).
Excel's cdf over Excel's pmf, against the exact ratio sum_j k!/((k - j)! mean^j), says whether the cdf is
the pmf times a series, so whether the prefactor chain is the pmf's (score.py).

- Fit, rows 1:18: the points of probes/t4291-poisson-far-above-mean (k 30 to 300 in columns A:H, mean/k
  1.3 to 3 in rows): the pmf (rows 1:6), the gamma density (7:12), CHISQ.DIST.RT (13:18). The cdfs are
  t4291's own cells.
- Hold-out, rows 19:42, fixed before any scoring (decision 0188 R4): 48 points from seed 4302, k
  log-uniform on [30, 300] (rounded to an integer), mean/k uniform on [1.3, 3], the mean cut to 6
  significant digits. The cdf (rows 19:24), the pmf (25:30), the gamma density (31:36) and CHISQ.DIST.RT
  (37:42).
Every literal has at most 15 significant digits (numbers.md).
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import lit, write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4302-poisson-cdf-beside-pmf.case.json"
KS = (30, 40, 60, 80, 120, 150, 250, 300)
RATIOS = (1.3, 1.5, 1.75, 2.0, 2.5, 3.0)
COLS = "ABCDEFGH"


def holdout_points():
    """The 48 hold-out points (k, mean), column-major: 6 per column."""
    rng = random.Random(4302)
    pts = []
    for _ in range(48):
        k = round(math.exp(rng.uniform(math.log(30), math.log(300))))
        mean = float(f"{k * rng.uniform(1.3, 3.0):.6g}")
        pts.append((k, mean))
    return pts


def chains(k, mean):
    return (
        f"=POISSON.DIST({k},{mean:g},TRUE)",
        f"=POISSON.DIST({k},{mean:g},FALSE)",
        f"=GAMMA.DIST({mean:g},{k + 1},1,FALSE)",
        f"=CHISQ.DIST.RT({lit(2 * mean)},{2 * k + 2})",
    )


def main() -> None:
    cells = {}
    for col, k in zip(COLS, KS):
        for row, r in enumerate(RATIOS, start=1):
            mean = round(k * r, 6)
            _, pmf, dens, chi = chains(k, mean)
            cells[f"{col}{row}"] = pmf
            cells[f"{col}{row + 6}"] = dens
            cells[f"{col}{row + 12}"] = chi
    for i, (k, mean) in enumerate(holdout_points()):
        col, row = COLS[i // 6], 19 + i % 6
        for j, f in enumerate(chains(k, mean)):
            cells[f"{col}{row + 6 * j}"] = f
    write_checked_case(
        OUT,
        (
            "T-4302: POISSON.DIST's cdf far above the mean beside the same point's pmf, gamma density and "
            "CHISQ.DIST.RT. Rows 1:18 are the points of t4291-poisson-far-above-mean (pmf, density, "
            "CHISQ.DIST.RT); rows 19:42 are 48 hold-out points from seed 4302, fixed before scoring "
            "(cdf, pmf, density, CHISQ.DIST.RT)."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4302"],
        cells,
    )


if __name__ == "__main__":
    main()
