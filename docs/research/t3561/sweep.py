"""T-3561 sweep: BETA.DIST's cdf on bratio's L140 bup+bgrat branch with n >= 2 bup terms against 50-digit math.

Points land on that branch by bratio's own dispatch (min(a, b) > 1, lambda >= 0 after the swap, x0 > 0.7,
a0 > 15) with 2 <= b0 < 40: a0 log-uniform in (15, 1E4], b0 uniform in [2, 39.9] (half of them whole), y0 = 1 - x0
log-uniform in [1E-4, 0.3], half the points written in the swapped orientation, and only points whose cdf
rounds strictly between 0 and 1. The engine's value comes from
`rustsheet-eval show --all` on a scratch suite (the 50-digit value as its expected sidecar); the script prints the
counts within 1, 4 and 16 ulp of the correctly rounded value, and the worst points.

    cd tools && uv run python ../docs/research/t3561/sweep.py ../target/release/rustsheet-eval [count [main binary]]
"""

from __future__ import annotations

import contextlib
import json
import math
import random
import re
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

import mpmath

mpmath.mp.dps = 50
SEED = 35610


def on_branch(a: float, b: float, x: float) -> bool:
    """bratio's dispatch lands on L140 bup+bgrat with b0 >= 2 (bratio/mod.rs `dispatch`)."""
    y = 1.0 - x
    if min(a, b) <= 1.0 or not 0.0 < x < 1.0:
        return False
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    a0, b0, x0 = (b, a, y) if lam < 0 else (a, b, x)
    return 2.0 <= b0 < 40.0 and b0 * x0 > 0.7 and x0 > 0.7 and a0 > 15.0


def point(rng: random.Random) -> tuple[float, float, float] | None:
    a0 = round(math.exp(rng.uniform(math.log(15.0), math.log(1e4))), 3)
    b0 = float(rng.randint(2, 39)) if rng.random() < 0.5 else round(rng.uniform(2.0, 39.9), 3)
    y0 = math.exp(rng.uniform(math.log(1e-4), math.log(0.3)))
    x0 = 1.0 - y0
    a, b, x = (a0, b0, x0) if rng.random() < 0.5 else (b0, a0, y0)
    # Formula literals keep 15 significant digits, as Excel's do: take the math at that double.
    x = float(f"{x:.15g}")
    return (x, a, b) if on_branch(a, b, x) else None


def ordinal(v: float) -> int:
    """A double's position on the number line, so ulp distances are differences."""
    (i,) = struct.unpack("<q", struct.pack("<d", v))
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def main() -> None:
    binary = Path(sys.argv[1]).resolve()
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 1000
    rng = random.Random(SEED)
    points: list[tuple[float, float, float]] = []
    exact: list[float] = []
    while len(points) < count:
        p = point(rng)
        if p is None or p in points:
            continue
        x, a, b = p
        v = float(mpmath.betainc(a, b, 0, x, regularized=True))
        # A cdf that rounds to 0 or 1 says nothing about the kernel's last digits.
        if 0.0 < v < 1.0:
            points.append(p)
            exact.append(v)
    dist = distances(binary, points, exact)
    n = len(points)
    print(f"points {n} (seed {SEED})")
    for limit in (0, 1, 4, 16):
        print(f"within {limit} ulp: {sum(1 for d in dist if d <= limit)}")
    for i in sorted(range(n), key=lambda i: -dist[i])[:10]:
        x, a, b = points[i]
        print(f"  {dist[i]} ulp  BETA.DIST({x!r},{a!r},{b!r},TRUE) exact {exact[i]!r}")
    if len(sys.argv) > 3:
        base = distances(Path(sys.argv[3]).resolve(), points, exact)
        closer = sum(1 for d, e in zip(dist, base, strict=True) if d < e)
        farther = [i for i in range(n) if dist[i] > base[i]]
        print(
            f"against {sys.argv[3]}: closer {closer}, farther {len(farther)}, same {n - closer - len(farther)}"
        )
        for i in sorted(farther, key=lambda i: base[i] - dist[i])[:10]:
            x, a, b = points[i]
            print(f"  farther {base[i]} -> {dist[i]} ulp  BETA.DIST({x!r},{a!r},{b!r},TRUE)")


def distances(binary: Path, points: list[tuple[float, float, float]], exact: list[float]) -> list[int]:
    """The engine's ulp distance from the correctly rounded value at each point."""
    with tempfile.TemporaryDirectory() as root:
        cases = Path(root) / "eval/suites/scratch/cases"
        cases.mkdir(parents=True)
        (cases.parent / "suite.toml").write_text(
            'name = "scratch"\ndescription = "T-3561 sweep"\nkind = "spec"\nroot = "cases"\n'
            'include = ["**/*.case.json"]\nlicense = "Apache-2.0"\norigin = "generated"\ngate = false\n'
            'baseline = "cells"\nmilestone = ["M9"]\n'
        )
        cells = {f"A{i + 1}": f"=BETA.DIST({x!r},{a!r},{b!r},TRUE)" for i, (x, a, b) in enumerate(points)}
        case = {
            "format": "rustsheet-case/v1",
            "description": "T-3561 sweep",
            "tags": ["probe"],
            "sheets": [{"name": "Calc", "cells": cells}],
            "check": [f"Calc!{k}" for k in cells],
            "expected": {},
        }
        (cases / "sweep.case.json").write_text(json.dumps(case))
        oracle = {
            "format": "rustsheet-oracle/v1",
            "source": {"application": "mpmath betainc, 50 digits", "kind": "libreoffice", "version": None},
            "sheets": {"Calc": {f"A{i + 1}": {"t": "n", "v": v} for i, v in enumerate(exact)}},
        }
        (cases / "sweep.oracle.json").write_text(json.dumps(oracle))
        run = subprocess.run(
            [str(binary), "--root", root, "show", "--all", "scratch/sweep.case.json"],
            capture_output=True,
            text=True,
            check=False,
        )
    if run.returncode != 0:
        sys.exit(f"rustsheet-eval show failed: {run.stderr.strip()}")
    out = run.stdout
    actual: dict[int, float] = {}
    for m in re.finditer(r"Calc!A(\d+)\s.*?\n(?:.*\n)*?\s+actual\s*: (\S+)", out):
        with contextlib.suppress(ValueError):
            actual[int(m.group(1)) - 1] = float(m.group(2))
    dist = []
    for i, v in enumerate(exact):
        got = actual.get(i)
        dist.append(abs(ordinal(got) - ordinal(v)) if got is not None and math.isfinite(got) else 1 << 62)
    return dist


if __name__ == "__main__":
    main()
