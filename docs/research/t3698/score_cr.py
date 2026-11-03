"""Correctly rounded root vs a candidate of score_grid.py on the fit set's cells with a 50-digit math sidecar,
per hash side. Run: cd tools && uv run python -I ../docs/research/t3698/score_cr.py <out-dir> <name>"""

import json
import math
import sys
from pathlib import Path

from rustsheet_tools.fitlab.holdout import is_holdout
from rustsheet_tools.fitlab.score import ulp_distance, within_fifteen_digits
from rustsheet_tools.fitlab.sweep import Cell

ROOT = Path(__file__).resolve().parents[3]
G = Path(sys.argv[1])
rule = json.load(open(G / f"per-{sys.argv[2]}.json"))
res = {s: {"cells": 0, "cr": [0, 0, 0], "rule": [0, 0, 0]} for s in ("train", "test")}
for k, ok in rule.items():
    cid, key = k.split(" ")
    sheet, a1 = key.split("!")
    su, _, nm = cid.partition("/")
    base = ROOT / "eval/suites" / su / "cases" / (nm + ".case.json")
    mj = Path(str(base).replace(".case.json", ".math.json"))
    if not mj.exists():
        continue
    m = json.load(open(mj))["sheets"].get(sheet, {}).get(a1)
    o = json.load(open(str(base).replace(".case.json", ".oracle.json")))["sheets"].get(sheet, {}).get(a1)
    if not m or not o:
        continue
    side = res["test" if is_holdout(Cell(cid, sheet, a1, "", (), math.nan)) else "train"]
    side["cells"] += 1
    side["rule"] = [a + b for a, b in zip(side["rule"], ok)]
    if m["t"] == "n" and o["t"] == "n":
        cr, ex = float(m["v"]), float(o["v"])
        side["cr"][0] += cr == ex
        side["cr"][1] += ulp_distance(cr, ex) <= 4
        side["cr"][2] += within_fifteen_digits(ex, cr)
    elif m["t"] == "e" and o["t"] == "e":
        side["cr"] = [x + 1 for x in side["cr"]]
print(res)
