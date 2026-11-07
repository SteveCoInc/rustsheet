"""Generate probes/t4874-numbervalue-empty-sep (T-4874): NUMBERVALUE with a written-empty separator.

Each probe is paired with the same call where the empty separator is written as the en-US default (decimal ".",
group ","), or left out when trailing; the engine's value of that call is H_omitted, the prediction committed before
the fill (docs/research/t4874/predictions.tsv). H_value is the engine before T-4874 (`#VALUE!`). Controls: a blank
cell (Z1) and "" as a separator stay `#VALUE!` in both.
Run: python3 docs/research/t4874/gen.py [pairs] > eval/suites/probes/cases/t4874-numbervalue-empty-sep.case.json
"""

import json
import sys

TEXTS = ["1.5", "1,234.5", "1.234,5", "2,5", "50%", "1:30"]


def pairs() -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for t in TEXTS:
        q = f'"{t}"'
        out.append((f'NUMBERVALUE({q},,",")', f'NUMBERVALUE({q},".",",")'))
        out.append((f'NUMBERVALUE({q},".",)', f'NUMBERVALUE({q},".")'))
        out.append((f"NUMBERVALUE({q},,)", f"NUMBERVALUE({q})"))
    out += [
        ('NUMBERVALUE("1,5",,".")', 'NUMBERVALUE("1,5",".",".")'),
        ('NUMBERVALUE("1.234,5",",",)', 'NUMBERVALUE("1.234,5",",")'),
        ('LET(e,,NUMBERVALUE("1.5",e))', 'NUMBERVALUE("1.5")'),
        ('LAMBDA(a,g,NUMBERVALUE(a,".",g))("1,234.5",)', 'NUMBERVALUE("1,234.5",".")'),
        ('NUMBERVALUE("1.5",Z1)', 'NUMBERVALUE("1.5",Z1)'),
        ('NUMBERVALUE("1.5","")', 'NUMBERVALUE("1.5","")'),
        ('NUMBERVALUE("1.5",".",Z1)', 'NUMBERVALUE("1.5",".",Z1)'),
    ]
    return out


def main() -> None:
    rows = pairs()
    if sys.argv[1:] == ["pairs"]:
        for i, (probe, omitted) in enumerate(rows, start=1):
            print(f"A{i}\t={probe}\t={omitted}")
        return
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "NUMBERVALUE with a written-empty decimal or group separator (T-4874), on six texts, plus the LET and "
            "LAMBDA paths and the blank-cell and empty-text controls (Z1 blank). Predictions in "
            "docs/research/t4874/predictions.tsv committed before the excel-graph fill: H_omitted (the en-US default "
            "separator) against H_value (#VALUE!)."
        ),
        "tags": ["M5", "text", "probe"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": {f"A{i}": f"={p}" for i, (p, _) in enumerate(rows, start=1)}}],
    }
    sys.stdout.write(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
