"""T-4146: draw a probe of FV and PV calls where the plain and twice-rounded forms (forms.py) differ.

Each call has a cancelling sum: pmt is drawn so that pmt·A is close to -pv·P (FV) or -fv (PV), with a relative
offset log-uniform in 1e-9..1e-1, so the sum loses digits and the two forms' rounding shows. rate is log-uniform in
1e-7..0.5 with either sign (above -1), nper an integer 1..400, type 0 or 1, |pv| and |fv| log-uniform in 1e2..1e13.
All literals are 15 significant digits, so the parsed double is the one scored. A call is kept when the two forms
give different values; FV and PV alternate until `count` cells. The seed names the draw, so a fresh sweep is a new
seed drawn after the form is frozen. With a fifth argument `nonint` (T-4244), nper is an integer 1..399 plus a
uniform fraction, written with 15 significant digits, so the forms are compared at non-integer nper.

Run (from tools/): uv run python ../docs/research/t4146/gen_probe.py <seed> <count> <out.case.json> <description> [nonint]
"""

import json
import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import forms  # noqa: E402

from rustsheet_tools.fitlab.kernels.solvers import caret_pow  # noqa: E402
from rustsheet_tools.fitlab.exact import X87  # noqa: E402


def lit(x: float) -> float:
    return float(f"{x:.15g}")


def loguni(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def draw(rng: random.Random, func: str, nonint: bool = False) -> tuple[str, list[float]] | None:
    r = lit(rng.choice((-1.0, 1.0)) * loguni(rng, 1e-7, 0.5))
    n = lit(rng.randint(1, 399) + rng.random()) if nonint else float(rng.randint(1, 400))
    t = float(rng.randint(0, 1))
    big = lit(rng.choice((-1.0, 1.0)) * loguni(rng, 1e2, 1e13))
    try:
        p = caret_pow(X87.add(1.0, r), n)
    except Exception:
        return None
    if not (math.isfinite(p) and p > 0):
        return None
    a = forms._factor("plain", r, p, t)
    if not math.isfinite(a) or a == 0:
        return None
    target = -big * p if func == "FV" else -big
    pmt = lit(target / a * (1 + rng.choice((-1.0, 1.0)) * loguni(rng, 1e-9, 1e-1)))
    if not math.isfinite(pmt) or pmt == 0:
        return None
    if func == "FV":
        return "FV", [r, n, pmt, big, t]
    return "PV", [r, n, pmt, big, t]


def main() -> None:
    seed, count, out, desc = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
    nonint = sys.argv[5:] == ["nonint"]
    rng = random.Random(seed)
    cells: list[str] = []
    while len(cells) < count:
        func = ("FV", "PV")[len(cells) % 2]
        d = draw(rng, func, nonint)
        if d is None:
            continue
        name, args = d
        ev = forms.fv if name == "FV" else forms.pv
        try:
            a, b = ev("plain", *args), ev("twice", *args)
        except Exception:
            continue
        if not (math.isfinite(a) and math.isfinite(b)) or a == b:
            continue
        r, n, pmt, x, t = args
        cells.append(f"={name}({r:.15g},{n:.15g},{pmt:.15g},{x:.15g},{t:.0f})")
    case = {
        "format": "rustsheet-case/v1",
        "description": desc,
        "tags": ["M9", "financial", "probe"],
        "expected": {},
        "hypotheses": {f"Calc!A{i}": "annuity/twice-rounded" for i in range(1, len(cells) + 1)},
        "sheets": [{"name": "Calc", "cells": {f"A{i}": f for i, f in enumerate(cells, start=1)}}],
    }
    with open(out, "w") as fh:
        json.dump(case, fh, indent=1)
        fh.write("\n")
    print(len(cells), "cells")


main()
