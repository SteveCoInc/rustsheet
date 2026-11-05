"""T-1749 fresh sweep (decision 0188 R4.2), generated after DEVSQ's two-pass form was frozen.

200 DEVSQ calls on 2 to 12 literal numbers: an offset log-uniform in [1e-3, 1e15] of either sign plus a spread
log-uniform in [1e-6, 1e3] times a uniform draw, so cancellation (spread far below the offset) and plain data both
occur. Literals have 15 significant digits (Excel reads a typed literal to 15, T-4077). Seeded.
Usage: python3 gen_fresh.py OUT.case.json
"""

import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "t0975"))
from gen_fresh import lu, write_case  # noqa: E402  (T-0975's sweep helpers)


def main() -> None:
    rng = random.Random(1749)
    cells: dict[str, str] = {}
    for row in range(1, 201):
        offset = lu(rng, 1e-3, 1e15) * rng.choice((-1, 1))
        spread = lu(rng, 1e-6, 1e3)
        n = rng.randint(2, 12)
        xs = [f"{offset + spread * rng.uniform(-1, 1):.15g}" for _ in range(n)]
        cells[f"A{row}"] = f"=DEVSQ({','.join(xs)})"
    write_case(
        sys.argv[1],
        "T-1749 fresh sweep (R4.2, after the freeze): DEVSQ of literal data, with and without cancellation.",
        ["M9", "probe", "statistics", "T-1749", "sweep", "r4-fresh"],
        cells,
    )

if __name__ == "__main__":
    main()
