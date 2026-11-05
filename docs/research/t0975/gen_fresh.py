"""T-0975 fresh sweep (decision 0188 R4.2), generated after the normal-density model was frozen.

Log-uniform points over the double range the densities reach: NORM.S.DIST(z,FALSE) and PHI(z) at
|z| in [1e-3, 38], NORM.DIST(x,m,s,FALSE) with s log-uniform in [1e-3, 1e3], m uniform in
[-100, 100] and z = (x-m)/s log-uniform in [1e-3, 38] (either sign); every argument is a double
written with 15 significant digits (Excel reads a typed literal to 15, T-4077). Seeded, so rerunning rewrites the same case.
Usage: python3 gen_fresh.py OUT.case.json
"""

import json
import math
import random
import sys


def lu(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def write_case(path: str, description: str, tags: list[str], cells: dict[str, str]) -> None:
    """Writes a pending probes case (one `Calc` sheet), as the probe fresh sweeps share (T-1749 reuses it)."""
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": tags,
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    with open(path, "w", encoding="utf-8") as out:
        json.dump(case, out, indent=1)
        out.write("\n")


def g(v: float) -> str:
    return repr(float(f"{v:.15g}"))


def main() -> None:
    rng = random.Random(975)
    cells: dict[str, str] = {}
    row = 1
    for _ in range(80):
        z = lu(rng, 1e-3, 38) * rng.choice((-1, 1))
        cells[f"A{row}"] = f"=NORM.S.DIST({g(z)},FALSE)"
        cells[f"B{row}"] = f"=PHI({g(z)})"
        row += 1
    for _ in range(160):
        s = lu(rng, 1e-3, 1e3)
        m = rng.uniform(-100, 100)
        z = lu(rng, 1e-3, 38) * rng.choice((-1, 1))
        x = float(g(m + z * s))
        cells[f"A{row}"] = f"=NORM.DIST({g(x)},{g(m)},{g(s)},FALSE)"
        cells[f"B{row}"] = f"=NORMDIST({g(x)},{g(m)},{g(s)},FALSE)"
        row += 1
    write_case(
        sys.argv[1],
        "T-0975 fresh sweep (R4.2, after the freeze): normal densities at log-uniform points.",
        ["M9", "probe", "statistics", "T-0975", "sweep", "r4-fresh"],
        cells,
    )

if __name__ == "__main__":
    main()
