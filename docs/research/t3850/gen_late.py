"""T-3850: draw probes/t3850-rate-same-state, a blind probe of RATE flat secants where the model's state at the flat
step is the same and only the step differs.

For each source series (the args of every oracle-backed `=RATE(` cell of probes and probes-numeric whose first flat
secant on distinct iterates comes at step 3 or later and where the hold and an unconditional restart differ), 400 new
guesses are drawn (seeded, log-uniform in +-1e-4..10). Each new call is run with the engine's trace (trace.diff: a
temporary, uncommitted switch in `rate_secant` and the example `t3850_probe`, which prints each flat secant's step,
r0, r1 and f1). Calls are grouped by (args, r0, r1, f1) of their first flat secant on distinct iterates. Groups with
at least three distinct steps are kept, up to six calls a group, with steps spread over the run, until 240 cells.

Prediction, fixed before the oracle fill: if Excel's value were a function of the model's visible state at the flat
step (r0, r1, f1 and the step), each group would split at most once along the step (a step bound). The finding to
test (from t3698-rate-flat-restart) is that Excel's value is not: groups get several Excel values in no step order.

Run: python3 -I docs/research/t3850/gen_late.py <cells.tsv> <probe-binary> <out.case.json>
(cells.tsv: case, cell, formula, oracle type, oracle value; one oracle-backed RATE cell a line)
"""

import json
import os
import random
import subprocess
import sys
from collections import defaultdict


def trace(binary: str, forms: list[str], env: dict[str, str]) -> list[dict]:
    p = subprocess.run(
        [binary], input="\n".join(forms) + "\n", capture_output=True, text=True,
        env={**os.environ, "RS_TRACE": "1", **env}, check=True,
    )
    res: list[dict] = []
    cur: dict = {}
    for line in p.stderr.splitlines():
        if line.startswith("### "):
            cur = {"flats": []}
        elif line.startswith("FLAT"):
            cur["flats"].append(dict(kv.split("=") for kv in line.split()[1:]))
        elif line.startswith("=== "):
            cur["v"] = line[4:]
            res.append(cur)
    assert len(res) == len(forms)
    return res


def first_flat(t: dict) -> dict | None:
    for d in t["flats"]:
        if d["restart"] == "true" and d["r0"] != d["r1"] and int(d["step"]) >= 3:
            return d
    return None


HOLD = {"RS_OFF": "1"}
ANY = {"RS_K": "100000", "RS_FLO": "0", "RS_FHI": "1e300"}


def main() -> None:
    cells_tsv, binary, out = sys.argv[1:4]
    rows = [line.rstrip("\n").split("\t") for line in open(cells_tsv)]
    forms = [r[2] for r in rows]
    hold, anyr = trace(binary, forms, HOLD), trace(binary, forms, ANY)
    sources = sorted({
        ",".join(f[6:-1].split(",")[:5])
        for f, h, a in zip(forms, hold, anyr)
        if first_flat(a) and h["v"] != a["v"] and len(f[6:-1].split(",")) >= 5
    })
    rng = random.Random("T-3850 same-state guesses")
    cand = []
    for s in sources:
        for _ in range(400):
            g = rng.choice((-1.0, 1.0)) * 10 ** rng.uniform(-4, 1)
            cand.append(f"=RATE({s},{float(f'{g:.6g}')!r})")
    cand = sorted(set(cand))
    hold, anyr = trace(binary, cand, HOLD), trace(binary, cand, ANY)
    groups = defaultdict(dict)
    for f, h, a in zip(cand, hold, anyr):
        d = first_flat(a)
        if d is None or h["v"] == a["v"]:
            continue
        key = (f[6:].rsplit(",", 1)[0], d["r0"], d["r1"], d["f1"])
        groups[key].setdefault(int(d["step"]), f)
    picked = []
    for key in sorted(groups, key=lambda k: (-len(groups[k]), k)):
        steps = sorted(groups[key])
        if len(steps) < 3:
            continue
        if len(steps) > 6:
            steps = [steps[round(i * (len(steps) - 1) / 5)] for i in range(6)]
        picked.extend((key, s, groups[key][s]) for s in steps)
        if len(picked) >= 240:
            break
    picked = picked[:240]
    cells, hyp = {}, {}
    for i, (key, s, f) in enumerate(picked, start=1):
        cells[f"A{i}"] = f
        hyp[f"Calc!A{i}"] = f"rate/same-state step {s}"
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-3850 blind probe: RATE calls in groups whose model state at the first flat secant on "
        "distinct iterates (args, r0, r1, f1) is the same and only the step differs (steps 3..200), to test "
        "whether any rule on that state and the step separates Excel's restarts from its holds. Drawn by "
        "docs/research/t3850/gen_late.py before the oracle fill.",
        "tags": ["M9", "financial", "probe"],
        "expected": {},
        "hypotheses": hyp,
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    with open(out, "w") as fh:
        fh.write(json.dumps(case, indent=1) + "\n")
    print(len(cells), "cells,", len({k for k, _, _ in picked}), "groups,", len(sources), "sources")


main()
