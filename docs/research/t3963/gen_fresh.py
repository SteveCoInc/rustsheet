"""T-3963: draw probes/t3963-rate-keep-r0-fresh, the fresh sweep of the frozen keep-r0 model.

The model was frozen before this draw (a secant step whose new residual equals the last one keeps the older
iterate r0, financial.md "RATE keeps its older iterate when a step leaves the residual unchanged"). For each
source series (the args of every oracle-backed literal `=RATE(` cell of the probes and probes-numeric suites),
new guesses are drawn (seeded, log-uniform in +-1e-4..10). A call is kept when the merged model (RS_V=0) and the
frozen model (RS_V=2304) give different values, at most 3 calls a series, until 240 cells. Both runs use the
temporary switch in trace.diff and the example binary t3963_probe, never merged.

Run: python3 -I docs/research/t3963/gen_fresh.py <cells.tsv> <probe-binary> <out.case.json>
(cells.tsv: case, cell, formula, oracle value; one oracle-backed literal RATE cell a line)
"""

import json
import math
import os
import random
import re
import subprocess
import sys


def run(binary: str, forms: list[str], variant: str) -> list[str]:
    p = subprocess.run(
        [binary], input="\n".join(forms) + "\n", capture_output=True, text=True,
        env={**os.environ, "RS_V": variant}, check=True,
    )
    return p.stdout.split("\n")[: len(forms)]


def main() -> None:
    cells_tsv, binary, out = sys.argv[1:4]
    series: list[str] = []
    seen = set()
    for line in open(cells_tsv):
        f = line.split("\t")[2]
        m = re.match(r"=RATE\((.*)\)$", f)
        if m is None:  # extract.py writes only literal RATE calls; skip anything else
            continue
        args = m.group(1).split(",")
        key = ",".join(args[:5] + [""] * (5 - len(args[:5])))
        if len(args) >= 4 and key not in seen:
            seen.add(key)
            series.append(key)
    rng = random.Random("T-3963 fresh keep-r0 sweep")
    rng.shuffle(series)
    forms: list[str] = []
    for key in series:
        for _ in range(40):
            g = rng.choice((-1.0, 1.0)) * math.exp(rng.uniform(math.log(1e-4), math.log(10)))
            forms.append(f"=RATE({key.rstrip(',')}{',' * (4 - key.rstrip(',').count(','))},{g:.6g})"
                         if key.count(",") < 4 else f"=RATE({key},{g:.6g})")
    a = run(binary, forms, "0")
    b = run(binary, forms, "2304")
    kept: list[str] = []
    per: dict[str, int] = {}
    for f, x, y in zip(forms, a, b):
        k = f.rsplit(",", 1)[0]
        if x != y and per.get(k, 0) < 3:
            per[k] = per.get(k, 0) + 1
            kept.append(f)
        if len(kept) == 240:
            break
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-3963 fresh sweep, drawn after the keep-r0 model was frozen: RATE calls on the args of the "
        "existing RATE probe series with new guesses, kept where the merged model and the frozen keep-r0 model "
        "differ (at most 3 a series). Drawn by docs/research/t3963/gen_fresh.py before the oracle fill.",
        "tags": ["M9", "financial", "probe"],
        "expected": {},
        "hypotheses": {f"Calc!A{i}": "rate/keep-r0 fresh" for i in range(1, len(kept) + 1)},
        "sheets": [{"name": "Calc", "cells": {f"A{i}": f for i, f in enumerate(kept, start=1)}}],
    }
    with open(out, "w") as fh:
        json.dump(case, fh, indent=1)
        fh.write("\n")
    print(len(kept), "cells from", len(per), "series")


main()
