"""Score predictions.json (written before the fill) against probes/t4466-count-hypotheses: python3 score.py [-v]."""

import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CASES = os.path.join(HERE, "../../../eval/suites/probes/cases/")
pred = json.load(open(os.path.join(HERE, "predictions.json")))
oracle = json.load(open(CASES + "t4466-count-hypotheses.oracle.json"))["sheets"]["Sheet1"]
case = json.load(open(CASES + "t4466-count-hypotheses.case.json"))["sheets"][0]["cells"]


def group(row):
    return "train" if row <= 120 else "hold-out"


tally = {}
misses = {}
for cell, models in sorted(pred.items()):
    if cell not in oracle:
        continue
    count = int(re.match(r"-?\d+", oracle[cell]["v"]).group())
    g = group(int(re.sub(r"^[A-Z]+", "", cell)))
    for name, got in models.items():
        t = tally.setdefault((name, g), [0, 0])
        t[1] += 1
        if got == count:
            t[0] += 1
        else:
            misses.setdefault((name, g), []).append(cell)
for (name, g), (ok, n) in sorted(tally.items()):
    print(f"{name:7} {g:8} {ok}/{n}  misses: {' '.join(misses.get((name, g), [])[:20])}")
if "-v" in sys.argv:
    for cell in sorted(case, key=lambda c: (int(re.sub(r"^[A-Z]+", "", c)), c)):
        if cell in oracle:
            p = pred.get(cell, {})
            print(cell, case[cell], oracle[cell]["v"], p)
