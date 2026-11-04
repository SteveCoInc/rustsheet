"""Build probes-numeric/t3411-irr-residual-scale: the cells that bracket T-3411's IRR_ROOT_RESIDUAL, every flow
scaled by 10^k (exact decimal shifts), to tell an absolute residual cut from one relative to the flows' size.

Run: python3 -I docs/research/t3411/gen_scale.py <repo>
"""
import json, re, sys
from decimal import Decimal
repo = sys.argv[1]
base = [("t3411-irr-r4-fresh", "A37"), ("t3411-irr-r4-fresh", "A140"), ("t3478-irr-r4-wide", "A79"),
        ("t3478-irr-r4-wide", "A292"), ("t3478-irr-r4-wide", "A90"), ("t3411-irr-r4-fresh-3", "A49"),
        ("t3411-irr-r4-fresh-4", "A49"), ("t3478-irr-r4-wide", "A69")]
scales = [-6, -4, -2, -1, 0, 1, 2, 3, 4]
def fmt(d):
    s = format(d.normalize(), "G")
    return s
cells, hyp, row = {}, {}, 1
for case, addr in base:
    f = json.load(open(f"{repo}/eval/suites/probes-numeric/cases/{case}.case.json"))["sheets"][0]["cells"][addr]
    m = re.fullmatch(r"=IRR\(\{([^}]*)\}(,[^)]*)?\)", f)
    flows, guess = m.group(1).split(","), m.group(2) or ""
    for k in scales:
        sc = [fmt(Decimal(x).scaleb(k)) for x in flows]
        cells[f"A{row}"] = "=IRR({" + ",".join(sc) + "}" + guess + ")"
        hyp[f"Calc!A{row}"] = f"irr-scale/{case} {addr} x1E{k:+d}"
        row += 1
case = {"format": "rustsheet-case/v1",
        "description": "T-3411 scale family (fidelity review of PR #3063): the cells that bracket IRR_ROOT_RESIDUAL "
        "(fresh A37, A140 returns; wide A90, fresh-3 A49 #NUM!) and neighbours, with every flow scaled by 10^k, "
        "k = -6..4, guess unchanged. Tells an absolute residual cut from one relative to the flows' size.",
        "tags": ["M9", "probe", "financial", "T-3411", "irr"], "expected": {}, "hypotheses": hyp,
        "sheets": [{"name": "Calc", "cells": cells}]}
out = f"{repo}/eval/suites/probes-numeric/cases/t3411-irr-residual-scale.case.json"
open(out, "w").write(json.dumps(case, indent=1) + "\n")
print(out, row - 1)
