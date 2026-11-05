"""T-4291: write probes/t4291-poisson-far-above-mean (run once; the case is committed).

POISSON.DIST(k, mean, TRUE) = Q(k + 1, mean) far above the mean: k 30, 40, 60, 80, 120, 150, 250 and 300
(column A by row block) at mean/k 1.3, 1.5, 1.75, 2, 2.5 and 3. Every literal has at most 15
significant digits (formula literals are cut to 15, numbers.md). This whole case is the hold-out
of decision 0188 R4, fixed before any scoring: the forms are compared on probes/t3252-gamma-prefactor-window
A97:A140 first.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4291-poisson-far-above-mean.case.json"
KS = (30, 40, 60, 80, 120, 150, 250, 300)
RATIOS = (1.3, 1.5, 1.75, 2.0, 2.5, 3.0)


def main() -> None:
    cells = {}
    for col, k in zip("ABCDEFGH", KS):
        for row, r in enumerate(RATIOS, start=1):
            mean = round(k * r, 6)
            cells[f"{col}{row}"] = f"=POISSON.DIST({k},{mean:g},TRUE)"
    write_checked_case(
        OUT,
        (
            "T-4291: POISSON.DIST's cdf far above the mean, k 30 to 300 (columns A:H) at mean/k 1.3 to 3 "
            "(rows 1:6): which large-shape prefactor Excel takes for Q(k + 1, mean) there."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4291"],
        cells,
    )


if __name__ == "__main__":
    main()
