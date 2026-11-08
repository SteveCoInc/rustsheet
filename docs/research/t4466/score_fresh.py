"""Score fresh_predictions.json (written before the fill; `rounded`, the correctly rounded candidate, is computed) against probes/t4466-count-fresh: python3 score_fresh.py."""

import json
import os
import re

from count_models import COUNT, TEXT, rounded, serial

HERE = os.path.dirname(os.path.abspath(__file__))
CASES = os.path.join(HERE, "../../../eval/suites/probes/cases/")
oracle = json.load(open(CASES + "t4466-count-fresh.oracle.json"))["sheets"]["Sheet1"]
cells = json.load(open(CASES + "t4466-count-fresh.case.json"))["sheets"][0]["cells"]
preds = json.load(open(os.path.join(HERE, "fresh_predictions.json")))
right = {"register": 0, "max": 0, "rounded": 0}
total = 0
for cell, pred in sorted(preds.items(), key=lambda kv: int(kv[0][1:])):
    if cell not in oracle:
        continue
    total += 1
    count = int(re.match(r"\d+", oracle[cell]["v"]).group())
    m = TEXT.match(cells[cell])
    f = COUNT.match(m.group(2))
    pred = dict(pred, rounded=rounded(serial(m.group(1)), {"h": 3600, "m": 60, "s": 1}[f.group(1)], 10 ** len(f.group(2))))
    for name in right:
        right[name] += pred[name] == count
    if pred["register"] != pred["max"] or pred["register"] != count:
        print(
            f"{cell:5} {cells[cell]:48} Excel {oracle[cell]['v']:22} register {pred['register']} max {pred['max']}"
        )
print(total, "cells:", ", ".join(f"{k} {v}" for k, v in right.items()))
