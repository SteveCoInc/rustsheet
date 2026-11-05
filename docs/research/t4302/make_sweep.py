"""T-4302: write probes-numeric/t4302-poisson-cdf-r4-fresh (run once, after the model was frozen in commit 3fd8a6c90).

The R4.2 fresh sweep (decision 0188) of the far-above-the-mean form: 48 points from seed 43020, k
log-uniform on [30, 300] (rounded to an integer), x / (k + 1) uniform on (1.5, 3], the mean cut to 6
significant digits, with no selection. Rows 1:6 hold the cdf POISSON.DIST(k, mean, TRUE) and rows 7:12
the gamma density GAMMA.DIST(mean, k + 1, 1, FALSE) at the same points (columns A:H, column-major).
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t4302-poisson-cdf-r4-fresh.case.json"


def points():
    rng = random.Random(43020)
    pts = []
    while len(pts) < 48:
        k = round(math.exp(rng.uniform(math.log(30), math.log(300))))
        mean = float(f"{(k + 1) * rng.uniform(1.5, 3.0):.6g}")
        if 1.5 < mean / (k + 1) <= 3.0:
            pts.append((k, mean))
    return pts


def main() -> None:
    cells = {}
    for i, (k, mean) in enumerate(points()):
        col, row = "ABCDEFGH"[i // 6], 1 + i % 6
        cells[f"{col}{row}"] = f"=POISSON.DIST({k},{mean:g},TRUE)"
        cells[f"{col}{row + 6}"] = f"=GAMMA.DIST({mean:g},{k + 1},1,FALSE)"
    write_checked_case(
        OUT,
        (
            "T-4302: fresh sweep after the freeze (decision 0188 R4.2) of POISSON.DIST's cdf far above the "
            "mean: 48 points from seed 43020, k 30 to 300 log-uniform, mean/(k + 1) uniform in (1.5, 3]; "
            "the cdf in rows 1:6 and the gamma density GAMMA.DIST(mean, k + 1, 1, FALSE) in rows 7:12."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4302"],
        cells,
    )


if __name__ == "__main__":
    main()
