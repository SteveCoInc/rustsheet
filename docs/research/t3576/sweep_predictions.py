"""T-3576 fresh sweep generator (decision 0188 R4.2 bracket): writes probes/t3576-r2-ratio-overflow-sweep and prints
each cell's prediction (#NUM! where ln R passes ln DBL_MAX, else a value), frozen before the excel-graph fill.
Run from the repository root: python3 docs/research/t3576/sweep_predictions.py
"""
import json, math
LNMAX = math.log(1.7976931348623157e308)
def lnR(a0, b0):
    n = int(b0); c = b0 - n
    if c == 0: n -= 1; c = 1.0
    apb, ap1 = c + a0, c + 1
    return sum(math.log((apb+l)/(ap1+l)) for l in range(n-1))
def solve(b0):
    lo, hi = 1e3, 1e300
    for _ in range(200):
        mid = math.sqrt(lo*hi)
        if lnR(mid, b0) < LNMAX: lo = mid
        else: hi = mid
    return lo
cells = {}; i = 1; pred = {}
def add(f, p):
    global i
    cells[f"A{i}"] = f; pred[f"A{i}"] = p; i += 1
for f in [19, 23, 31, 35, 38]:
    s = solve(f + 1)
    for p in [0.25, 0.55, 0.95]:
        for k in [0.97, 1.03]:
            sv = float(f"{s*k:.4g}")
            add(f"=NEGBINOM.DIST({f},{sv:.4G},{p},TRUE)".replace("E+","E+"), "#NUM!" if lnR(sv, f+1) > LNMAX else "value")
for b in [21.5, 30.25, 37.75]:
    s = solve(b)
    for x in [0.3, 0.9]:
        for k in [0.97, 1.03]:
            sv = float(f"{s*k:.4g}")
            add(f"=BETA.DIST({x},{sv:.4G},{b},TRUE)", "#NUM!" if lnR(sv, b) > LNMAX else "value")
            add(f"=BETA.DIST({1-x:.1f},{b},{sv:.4G},TRUE)", "#NUM!" if lnR(sv, b) > LNMAX else "value")
case = {"format": "rustsheet-case/v1",
 "description": "T-3576 fresh sweep (attempt 2, frozen before fill): Excel's incomplete beta is #NUM! on bratio's L140 branch exactly where bup's running ratio product prod (apb + l)/(ap1 + l) passes the largest double. Points 3% either side of the predicted cut, at failures 19 to 38, p either side of 0.7 (bup + bpser, bup + bgrat), non-integer second shapes through BETA.DIST and the swapped orientation.",
 "tags": ["M9", "probe", "statistics", "T-3576"],
 "sheets": [{"name": "Calc", "cells": cells}]}
json.dump(case, open("eval/suites/probes/cases/t3576-r2-ratio-overflow-sweep.case.json", "w"), indent=1)
open("eval/suites/probes/cases/t3576-r2-ratio-overflow-sweep.case.json", "a").write("\n")
for k, v in cells.items(): print(k, pred[k], v)
