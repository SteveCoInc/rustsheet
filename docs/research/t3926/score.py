"""T-3926: score the engine against Excel on the two subnormal-Y F density cases, and count the power-of-two cells
where Excel's F*x/2^-1074 is an integer (Y/x exact in double, so Y is Excel's stored subnormal).

Run: cd tools && uv run python -I ../docs/research/t3926/score.py <repo>"""

from __future__ import annotations

import json
import re
import sys
from fractions import Fraction
from pathlib import Path

from rustsheet_tools.fitlab.score import ulp_distance, within_fifteen_digits
from rustsheet_tools.snapshot import show_cells

REPO = Path(sys.argv[1])
CASES = REPO / "eval" / "suites" / "probes-numeric" / "cases"
for suffix in ("", "-hold"):
    name = f"t3926-f-density-subnormal-y{suffix}"
    case = json.loads((CASES / f"{name}.case.json").read_text())
    oracle = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["Calc"]
    shown = show_cells(f"probes-numeric/{name}.case.json")
    tot: dict[str, int] = {}

    def bump(k: str, ok: bool = True) -> None:
        tot[k] = tot.get(k, 0) + int(ok)

    for key, hyp in case["hypotheses"].items():
        kind = hyp.split("/")[4]
        ex = oracle[key.split("!")[1]]["v"]
        a = float(shown[key]["actual"])
        for k in (kind, "all"):
            bump(f"{k} cells")
            bump(f"{k} bit-exact", a == ex)
            bump(f"{k} <=4ulp", ulp_distance(a, ex) <= 4)
            bump(f"{k} <=15d", within_fifteen_digits(ex, a))
        if kind == "pow2":
            k = int(re.search(r"x=2\^-(\d+)", hyp).group(1))
            y_units = Fraction(ex) * Fraction(1, 2**k) * 2**1074
            bump("pow2 Excel F*x integer in 2^-1074 units", y_units.denominator == 1)
    print(name, json.dumps(tot, indent=1))
