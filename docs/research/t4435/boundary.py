"""T-4435: where the large-shape expansion (`large_shape_beta.rs`) is accurate, as a function of b/a, against 60-digit
math. This sets the expansion's lower bound on a from the math alone; no Excel value is read.

    tools/.venv/bin/python -I docs/research/t4435/boundary.py make          # boundary-references.json
    tools/.venv/bin/python -I docs/research/t4435/boundary.py case DIR      # scratch eval root
    target/release/rustsheet-eval show --all --root DIR t4435/grid.case.json > show.txt   (a build that forces the expansion)
    tools/.venv/bin/python -I docs/research/t4435/boundary.py score show.txt

Points: `BETA.DIST(x,a,b,TRUE)` = I_x(a, b) with a in 100 to 2E4, b in 1.5 to 200 (b/a at most 0.5), and 1 - x =
r b/a for r in 0.5, 0.8, 1, 1.25, 2 (the mean is at r = 1), 1 - x kept below 0.5. x is rounded to 12 significant
digits; the math is at that double.
"""

from __future__ import annotations

import json
import math
import re
import sys
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import grid  # noqa: E402

mp.mp.dps = 60
REFS = HERE / "boundary-references.json"
A = (100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0)
B = (1.5, 2.7, 10.5, 25.3, 39.5, 60.2, 100.5, 150.7, 200.0)
R = (0.5, 0.8, 1.0, 1.25, 2.0)


def points() -> list[tuple[float, float, float]]:
    out = []
    for a in A:
        for b in B:
            if b / a > 0.5:
                continue
            for r in R:
                y = r * b / a
                if y < 0.5:
                    out.append((grid.round_sig(1.0 - y, 12), a, b))
    return out


def make() -> None:
    rows = []
    for x, a, b in points():
        v = grid.math_value("BETA.DIST", x, a, b)
        rows.append(["b", grid.formula("BETA.DIST", x, a, b), mp.nstr(v, 40, min_fixed=1, max_fixed=0)])
    REFS.write_text("[\n" + ",\n".join(json.dumps(r) for r in rows) + "\n]\n")
    print(f"wrote {REFS.name}: {len(rows)} points")


def score(show: Path) -> None:
    rows = [(g, f, mp.mpf(v)) for g, f, v in json.loads(REFS.read_text())]
    got = grid.engine(show)
    worst: dict[tuple[float, float], float] = {}
    for i, (_, f, v) in enumerate(rows, start=1):
        m = re.match(r"=BETA\.DIST\(([^,]+),([^,]+),([^,]+),", f)
        a, b = float(m[2]), float(m[3])
        e = float(abs(mp.mpf(got[i]) - v) / v) if math.isfinite(got[i]) else math.inf
        worst[(a, b)] = max(worst.get((a, b), 0.0), e)
    print("Worst relative error of the expansion per (a, b) over r; columns b, rows a")
    print("| a \\ b | " + " | ".join(f"{b:g}" for b in B) + " |")
    print("|---|" + "---:|" * len(B))
    for a in A:
        cells = [f"{worst[(a, b)]:.1e}" if (a, b) in worst else "" for b in B]
        print(f"| {a:g} | " + " | ".join(cells) + " |")
    print("\nWorst relative error by b/a band")
    bands = [(0, 1e-3), (1e-3, 4e-3), (4e-3, 1e-2), (1e-2, 2e-2), (2e-2, 5e-2), (5e-2, 0.1), (0.1, 0.5)]
    for lo, hi in bands:
        es = [e for (a, b), e in worst.items() if lo < b / a <= hi]
        if es:
            print(f"- b/a in ({lo:g}, {hi:g}]: {len(es)} shape pairs, worst {max(es):.1e}")


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "make":
        make()
    elif cmd == "case":
        saved = grid.REFS
        grid.REFS = REFS
        grid.case(Path(sys.argv[2]))
        grid.REFS = saved
    elif cmd == "score":
        score(Path(sys.argv[2]))
