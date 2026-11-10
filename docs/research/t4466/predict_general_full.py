"""Write general_full_predictions.json from the frozen engine's answers on the unfilled t4466-general-full case:
python3 predict_general_full.py SHOW_OUTPUT, where SHOW_OUTPUT is `rustsheet-eval show --all` run against a
placeholder sidecar (deleted before the fill). Also records the trimmed-digit rule's answer (gen_general_fresh.py's
`general(v, "old")`) in [1E-09, 1E-04), the only band where the two rules can differ.
"""

import json
import os
import re
import sys

from caseio import HERE

src = open(os.path.join(HERE, "gen_general_fresh.py")).read().split("rng = random.Random")[0]
ns: dict = {}
exec(src, ns)
preds = {}
text = open(sys.argv[1]).read()
for cell, value, actual in re.findall(r'Sheet1!(A\d+).*?\n\s+formula : =TEXT\(([^,]+),"General"\)\n.*?\n\s+actual  : "([^"]*)"', text):
    v = float(value)
    # The two rules differ only in [1E-09, 1E-04): outside it both are fixed-only or E-only.
    trimmed = ns["general"](v, "old") if 1e-9 <= abs(v) < 1e-4 else None
    preds[cell] = {"frozen": actual, "trimmed": trimmed}
with open(os.path.join(HERE, "general_full_predictions.json"), "w") as fh:
    json.dump(preds, fh, indent=0, sort_keys=True)
print(len(preds), "cells;", sum(p["trimmed"] not in (None, p["frozen"]) for p in preds.values()), "where the rules differ")
