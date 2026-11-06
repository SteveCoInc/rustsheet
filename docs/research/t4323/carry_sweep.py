"""Fresh carry sweep of T-4323 round 4: python3 carry_sweep.py SEED COUNT CASE_PATH DESCRIPTION.

Elapsed-count formats with fraction digits at a tie, half the cells just short of a whole minute or
hour (where the count could carry), half at a random tie. Generated after the register count
(`floor(y*86400/unit)`, y the half-step register) was frozen; seed 43235, 120 cells.
"""
import json
import math
import random
import sys

seed, count = (int(a) for a in sys.argv[1:3])
case_path, description = sys.argv[3:5]
random.seed(seed)
FMTS = [("[h]:mm:ss.0", 10), ("[h]:mm:ss.00", 100), ("[h]:mm:ss.000", 1000), ("[h]:mm.0", 10),
        ("[m]:ss.0", 10), ("[m]:ss.00", 100), ("[s].0", 10), ("[s].00", 100), ("[s].000", 1000)]
DAYS = [0, 1, 2, 3, 5, 7, 30, 365, 1000, 20000, 45000, 60000, 2958000]
cells = {}
for r in range(1, count + 1):
    f, s = random.choice(FMTS)
    d = random.choice(DAYS)
    if r % 2:
        whole = random.choice([60, 3600]) * random.randint(1, 23)
        sec = whole - 0.5 / s
    else:
        sec = 10 ** random.uniform(-2, math.log10(86399))
        sec = (math.floor(sec * s) + 0.5) / s
    sec = float("%.7g" % sec)
    dpart = f"{d}+" if d else ""
    cells[f"A{r}"] = f'=TEXT({dpart}{sec:.7g}/86400,"{f}")'
case = {"format": "rustsheet-case/v1", "description": description, "tags": ["M10", "text", "probe"],
        "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}}
open(case_path, "w").write(json.dumps(case, indent=2) + "\n")
