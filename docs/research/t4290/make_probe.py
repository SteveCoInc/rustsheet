"""T-4290: write probes/t4290-gamma-window-large-pow2 (run once; the case is committed).

Shapes 4096, 1024 and 256 (columns A, B, C) at x/a = k/1024, k = 656..1556 step 4: a power-of-two
shape makes x, a - x and a*g(x/a) exact, and at shape 4096 one ulp of the exponent spans about 256
density ulps, so Excel's exponent reads out as an integer number of its ulps. Column D, the
hold-out fixed before any scoring: shape 2048 at k + 2. Columns E and F: ladders across the
mathematical switch points |x - a| = 0.2 (x + a): shape 3072 around x = 2048 (x/a = 2/3) and
shape 2048 around x = 3072 (x/a = 3/2), both exact.
"""

import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4290-gamma-window-large-pow2.case.json"


def dens(x: float, a: float) -> str:
    return f"=GAMMA.DIST({x!r},{a!r},1,FALSE)"


def ladder(center: float) -> list[float]:
    out = []
    for j in (10, 20, 30, 40):
        out.append(center * (1 - 2.0**-j))
    out += [math.nextafter(math.nextafter(center, 0), 0), math.nextafter(center, 0), center]
    out += [math.nextafter(center, math.inf), math.nextafter(math.nextafter(center, math.inf), math.inf)]
    for j in (40, 30, 20, 10):
        out.append(center * (1 + 2.0**-j))
    return out


def main() -> None:
    cells: dict[str, str] = {}
    for row, k in enumerate(range(656, 1557, 4), start=1):
        for col, a, kk in (("A", 4096.0, k), ("B", 1024.0, k), ("C", 256.0, k), ("D", 2048.0, k + 2)):
            cells[f"{col}{row}"] = dens(a * kk / 1024, a)
    for col, a, c in (("E", 3072.0, 2048.0), ("F", 2048.0, 3072.0)):
        for row, x in enumerate(ladder(c), start=1):
            cells[f"{col}{row}"] = dens(x, a)
    write_checked_case(
        OUT,
        (
            "T-4290: Excel's large-shape gamma density inside the T-3252 window at power-of-two shapes "
            "4096, 1024, 256 (A, B, C; x/a = k/1024 exact, k 656 to 1556 step 4), the hold-out shape 2048 "
            "at k + 2 (D), and ladders across x/a = 2/3 at shape 3072 (E) and x/a = 3/2 at shape 2048 (F)."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4290"],
        cells,
    )


if __name__ == "__main__":
    main()
