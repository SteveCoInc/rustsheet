"""T-4237: write probes/t4237-unique-cycle-fresh, the probe made after T-4200 froze its UNIQUE merge model.

Three groups of rows, each with A = UNIQUE's kept lines, B = exactly_once's lines ("none" when it errs) and C = the
input order, read as probes/t4200-unique-cycle-permutations reads them (the code of each text's last character and
its length; Data is the t3211/t4200 Data sheet):
- 60 seeded permutations (seed "T-4237") of the six texts A3..A8, mixing the {A3,A4,A5} cycle with A6, A7, A8;
- 36 two-column rows where both columns hold a permutation of the cycle {A3,A4,A5} (every pair of the 6 orders);
- 24 two-column rows of a number, boolean or mixed-type first column beside a permutation of the cycle.

Run: python3 -I docs/research/t4237/gen_probe.py <out.case.json>
"""

import itertools
import json
import random
import sys

ONE = 'TEXTJOIN(",",,UNICODE(RIGHT(s))&"/"&LEN(s))'
TWO = (
    'TEXTJOIN(",",,UNICODE(RIGHT(CHOOSECOLS(s,1)))&"/"&LEN(CHOOSECOLS(s,1))&":"'
    '&UNICODE(RIGHT(CHOOSECOLS(s,2)))&"/"&LEN(CHOOSECOLS(s,2)))'
)
# A first column that is not text: numbers, booleans or both are read back with their own text.
TWO_MIXED = 'TEXTJOIN(",",,CHOOSECOLS(s,1)&":"&UNICODE(RIGHT(CHOOSECOLS(s,2)))&"/"&LEN(CHOOSECOLS(s,2)))'


def refs(order: tuple[int, ...]) -> str:
    return "VSTACK(" + ",".join(f"Data!A{i}" for i in order) + ")"


def row(arr: str, show: str) -> tuple[str, str]:
    a = f"=LET(s,UNIQUE({arr}),{show})"
    b = f'=IFERROR(LET(s,UNIQUE({arr},,TRUE),{show}),"none")'
    return a, b


def main() -> None:
    rng = random.Random("T-4237")
    rows: list[tuple[str, str, str]] = []
    perms = list(itertools.permutations((3, 4, 5, 6, 7, 8)))
    for order in rng.sample(perms, 60):
        rows.append((*row(refs(order), ONE), ",".join(map(str, order))))
    cyc = list(itertools.permutations((3, 4, 5)))
    for o1, o2 in itertools.product(cyc, cyc):
        rows.append((*row(f"HSTACK({refs(o1)},{refs(o2)})", TWO), f"{','.join(map(str, o1))} | {','.join(map(str, o2))}"))
    firsts = ["VSTACK(1,2,1)", "VSTACK(1,1,1)", "VSTACK(TRUE,FALSE,TRUE)", "VSTACK(TRUE,TRUE,TRUE)",
              'VSTACK(1,TRUE,1)', 'VSTACK(1,"1",1)', 'VSTACK(TRUE,"TRUE",TRUE)', 'VSTACK(0,FALSE,"")']
    for first in firsts:
        for o in rng.sample(cyc, 3):
            rows.append((*row(f"HSTACK({first},{refs(o)})", TWO_MIXED), f"{first} | {','.join(map(str, o))}"))
    cells: dict[str, str] = {}
    for i, (a, b, c) in enumerate(rows, start=1):
        cells[f"A{i}"], cells[f"B{i}"], cells[f"C{i}"] = a, b, c
    data = {
        "A1": '="a"', "A2": '="a"&REPT(UNICHAR(770),16)', "A3": '="a"&UNICHAR(834)',
        "A4": '="a"&UNICHAR(834)&UNICHAR(834)', "A5": '="a"&UNICHAR(807)', "A6": '="a"&UNICHAR(803)',
        "A7": '="a"&REPT(UNICHAR(803),3)', "A8": '="á"',
    }
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4237: UNIQUE (and exactly_once) on orders made after T-4200 froze its merge model: 60 "
        "seeded orders of the six texts A3..A8 of the t3211 Data sheet, two-column rows where both columns hold the "
        "{A3,A4,A5} cycle, and a number, boolean or mixed first column beside the cycle. A lists the kept rows, B "
        "the exactly_once rows, C the input order. Written by docs/research/t4237/gen_probe.py before the fill.",
        "tags": ["M7", "probe", "collation", "T-4237"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}, {"name": "Data", "cells": data}],
    }
    with open(sys.argv[1], "w") as fh:
        json.dump(case, fh, indent=1, ensure_ascii=False)
        fh.write("\n")
    print(len(rows), "rows")


main()
