"""T-3698 guess-carrying sweep (probes-numeric/t3698-rate-r4-fresh): Gate A's RATE draw (all strata) plus a
log-uniform guess, and a math.tsv of the 50-digit roots. Run: cd tools && uv run python -I
../docs/research/t3698/gen_fresh.py ../eval/suites/probes-numeric/cases/<name>.case.json 300"""

import io
import json
import random
import sys

from rustsheet_tools.accuracy import defs  # noqa: F401
from rustsheet_tools.accuracy.literal import round15
from rustsheet_tools.accuracy.model import REGISTRY
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.accuracy.tables import draw_points, formula

out = sys.argv[1]
d = REGISTRY["RATE"]
pts = draw_points(d, "r4-T-3698-fresh", count=int(sys.argv[2]), log=io.StringIO())
rng = random.Random("T-3698 fresh guesses")
cells = {}
hyp = {}
tsv = ["cell\tmath\tformula"]
for i, p in enumerate(pts, start=1):
    g = (
        rng.choice((-1.0, 1.0)) * log_uniform(rng, 1e-4, 0.9)
        if rng.random() < 0.8
        else log_uniform(rng, 0.9, 10)
    )
    f = formula("RATE", list(p.args) + [round15(g)])
    cells[f"A{i}"] = f
    hyp[f"Calc!A{i}"] = f"rate/{p.stratum}"
    tsv.append(f"Calc!A{i}\t{p.math.value if p.math.value is not None else '#NUM!'}\t{f}")
case = {
    "format": "rustsheet-case/v1",
    "description": "T-3698 R4 fresh sweep, drawn after the flat-secant restart rule was frozen (restart once from "
    "the older iterate when f1 = f0 on distinct iterates at steps 3..16 with 1.3e-7 <= |f1| < 1e-4): the points of "
    "Gate A's RATE draw (rustsheet-accuracy, batch seed r4-T-3698-fresh, every stratum), each with a guess drawn "
    "log-uniform in +-1e-4..0.9 (80%) or 0.9..10. math.tsv holds the 50-digit root (decision 0188 R4.2).",
    "tags": ["M9", "financial", "probe", "numeric"],
    "expected": {},
    "hypotheses": hyp,
    "sheets": [{"name": "Calc", "cells": cells}],
}
open(out, "w").write(json.dumps(case, indent=1) + "\n")
open(out.replace(".case.json", ".math.tsv"), "w").write("\n".join(tsv) + "\n")
print(len(cells))
