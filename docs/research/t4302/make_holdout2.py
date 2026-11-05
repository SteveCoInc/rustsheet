"""T-4302: write probes/t4302-poisson-cdf-holdout-2 (run once, review round 1).

A clean R4.1 hold-out (decision 0188) for the frozen far-above-the-mean form. The first hold-out was seen while
the deviance B was ranked. These 48 points come from seed 43021: k log-uniform on [30, 300] (rounded to an
integer), x / (k + 1) uniform on (1.5, 2.99], and the mean cut to 6 significant digits, with no selection. The
pass margin was written in the task (the Outcome's "Hold-out 2" line) and pushed before this case was filled.
Rows 1:6 hold the cdf POISSON.DIST(k, mean, TRUE), in columns A:H, column-major.
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4302-poisson-cdf-holdout-2.case.json"


def points():
    rng = random.Random(43021)
    pts = []
    while len(pts) < 48:
        k = round(math.exp(rng.uniform(math.log(30), math.log(300))))
        mean = float(f"{(k + 1) * rng.uniform(1.5, 2.99):.6g}")
        if 1.5 < mean / (k + 1) <= 2.99:
            pts.append((k, mean))
    return pts


def main() -> None:
    cells = {}
    for i, (k, mean) in enumerate(points()):
        cells[f"{'ABCDEFGH'[i // 6]}{1 + i % 6}"] = f"=POISSON.DIST({k},{mean:g},TRUE)"
    write_checked_case(
        OUT,
        (
            "T-4302: a second hold-out for POISSON.DIST's cdf far above the mean, fixed with its margin before "
            "scoring (decision 0188 R4.1): 48 points from seed 43021, k 30 to 300 log-uniform, mean/(k + 1) "
            "uniform in (1.5, 2.99]."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4302"],
        cells,
    )


if __name__ == "__main__":
    main()
