"""T-4435: where the large-shape expansion (`large_shape_beta.rs`) is accurate, as a function of a and b, against
60-digit math, and where the kernels it replaces are. This sets the expansion's lower bound on a from the math
alone; no Excel value is read.

    tools/.venv/bin/python -I docs/research/t4435/boundary.py make       # boundary-references.json, boundary-points.tsv
    # The expansion forced at every point (an ignored unit test, so the table is reproducible from the commit):
    cargo test -p rustsheet_engine --lib boundary_table -- --ignored --nocapture > forced.txt
    # The kernels dispatch picks, for a build of main (the old kernels) or of the branch:
    tools/.venv/bin/python -I docs/research/t4435/boundary.py case DIR
    target/release/rustsheet-eval show --all --root DIR t4435/grid.case.json > dispatch.txt
    tools/.venv/bin/python -I docs/research/t4435/boundary.py score forced.txt OLD_DISPATCH.txt

Points: `BETA.DIST(x,a,b,TRUE)` = I_x(a, b), x below 1 on the side of the huge first shape a:
- a in 523.71, 1033.37, 1517.93, 2049.9, 2537.31, 3571.13, 5138.657, 7919.27, 12345.67, 49999.37, 123456.7 and
  987654.3. None is a multiple of 1/2, so nu = a + (b - 1)/2 is not exact in doubles: the shapes of the
  t3568-r5-bup probes that T-4435's first bound mishandled (fidelity review).
- b in 1.5, 2.7, 10.5, 19.293, 25.3, 33.175, 39.5, 60.2, 100.5, 150.7 and 200, with b/a at most 0.5.
- 1 - x = r b/a for r in 0.5, 1, 2 (around the mean, at r = 1), 5, 10, 30 and 100 (deep lower tails, where z = nu |ln x|
  reaches several hundred), kept while 1 - x < 0.6, r b <= 600 (z about r b) and the tail is above 1E-290.
x is rounded to 12 significant digits; the math is at that double.
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
POINTS = HERE / "boundary-points.tsv"
A = (
    523.71,
    1033.37,
    1517.93,
    2049.9,
    2537.31,
    3571.13,
    5138.657,
    7919.27,
    12345.67,
    49999.37,
    123456.7,
    987654.3,
)
B = (1.5, 2.7, 10.5, 19.293, 25.3, 33.175, 39.5, 60.2, 100.5, 150.7, 200.0)
R = (0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 100.0)


def points() -> list[tuple[float, float, float]]:
    out = []
    for a in A:
        for b in B:
            if b / a > 0.5:
                continue
            for r in R:
                y = r * b / a
                # z = nu |ln x| is about r b; past 600 the tail is below the double range's floor.
                if y < 0.6 and r * b <= 600:
                    out.append((grid.round_sig(1.0 - y, 12), a, b))
    return out


def _math(p: tuple[float, float, float]) -> mp.mpf:
    x, a, b = p
    return grid.math_value("BETA.DIST", x, a, b)


def make() -> None:
    from concurrent.futures import ProcessPoolExecutor

    pts = points()
    with ProcessPoolExecutor() as pool:
        values = list(pool.map(_math, pts))
    rows, tsv = [], []
    for (x, a, b), v in zip(pts, values):
        if v < mp.mpf("1e-290"):
            continue
        rows.append(["b", grid.formula("BETA.DIST", x, a, b), mp.nstr(v, 40, min_fixed=1, max_fixed=0)])
        tsv.append(f"{x!r}\t{a!r}\t{b!r}")
    REFS.write_text("[\n" + ",\n".join(json.dumps(r) for r in rows) + "\n]\n")
    POINTS.write_text("\n".join(tsv) + "\n")
    print(f"wrote {REFS.name} and {POINTS.name}: {len(rows)} points")


def load() -> list[tuple[float, float, float, mp.mpf]]:
    out = []
    for _, f, v in json.loads(REFS.read_text()):
        m = re.match(r"=BETA\.DIST\(([^,]+),([^,]+),([^,]+),", f)
        out.append((float(m[1]), float(m[2]), float(m[3]), mp.mpf(v)))
    return out


def forced(path: Path) -> dict[int, float]:
    out = {}
    for line in path.read_text().splitlines():
        m = re.fullmatch(r"(\d+)\t(\S+)", line.strip())
        if m:
            out[int(m[1])] = float(m[2])
    return out


def worst_by_shape(rows, got) -> dict[tuple[float, float], float]:
    worst: dict[tuple[float, float], float] = {}
    for i, (_, a, b, v) in enumerate(rows, start=1):
        g = got.get(i, math.nan)
        e = float(abs(mp.mpf(g) - v) / v) if math.isfinite(g) else math.inf
        worst[(a, b)] = max(worst.get((a, b), 0.0), e)
    return worst


def table(title: str, worst: dict[tuple[float, float], float]) -> None:
    print(f"\n{title}")
    print("| a \\ b | " + " | ".join(f"{b:g}" for b in B) + " |")
    print("|---|" + "---:|" * len(B))
    for a in A:
        print(f"| {a:g} | " + " | ".join(f"{worst[(a, b)]:.1e}" if (a, b) in worst else "" for b in B) + " |")


def score(forced_path: Path, dispatch_path: Path) -> None:
    rows = load()
    exp = worst_by_shape(rows, forced(forced_path))
    old = worst_by_shape(rows, grid.engine(dispatch_path))
    table("The expansion, forced (worst relative error over r)", exp)
    table("The kernels dispatch picks in the given build (worst relative error over r)", old)
    print(
        "\nPer a: is the expansion at least as accurate as those kernels at every b (to 1.5 times, or within 2E-16)?"
    )
    for a in A:
        bad = [b for b in B if (a, b) in exp and exp[(a, b)] > max(1.5 * old[(a, b)], 2e-16)]
        print(f"- a = {a:g}: " + ("yes" if not bad else "no, at b = " + ", ".join(f"{b:g}" for b in bad)))


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
        score(Path(sys.argv[2]), Path(sys.argv[3]))
