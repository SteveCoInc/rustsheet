"""Score probes/t4466-register-fresh (gen_register_fresh.py): python3 score_register_fresh.py.

`engine` is register_fresh_predictions.json, the engine's strings written before the fill (field-cut register).
`at once` is the earlier register, the half-stepped serial cut at once, `frac(y)*86400*s mod s`; Excel's `General`
prints about ten significant digits, so it is scored as within 1E-9 relative of Excel's printed register, plus 1E-13 absolute. The date part of each string is scored as part of `engine` only.
"""

import json
import math
import os

from count_models import TEXT, serial

HERE = os.path.dirname(os.path.abspath(__file__))
CASES = os.path.join(HERE, "../../../eval/suites/probes/cases/")
oracle = json.load(open(CASES + "t4466-register-fresh.oracle.json"))["sheets"]["Sheet1"]
cells = json.load(open(CASES + "t4466-register-fresh.case.json"))["sheets"][0]["cells"]
preds = json.load(open(os.path.join(HERE, "register_fresh_predictions.json")))


def at_once(x, s):
    y = x + 0.5 / (86400 * s)
    return ((y - math.floor(y)) * 86400 * s) % s


def close(a, b):
    return abs(a - b) <= 1e-9 * abs(b) + 1e-13


engine = once = total = 0
for cell in sorted(cells, key=lambda c: int(c[1:])):
    if cell not in oracle:
        continue
    total += 1
    excel = oracle[cell]["v"]
    m = TEXT.match(cells[cell])
    s = 10 ** len(m.group(2).split(".")[1].split(" ")[0])
    shown = float(excel.split(" ")[-1])
    engine += preds[cell] == excel
    once += close(at_once(serial(m.group(1)), s), shown)
    if preds[cell] != excel:
        print(f"{cell:5} {cells[cell]:58} Excel {excel:30} engine {preds[cell]}")
print(total, "cells: engine", engine, "at once", once)
