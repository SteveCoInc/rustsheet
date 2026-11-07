"""Generate probes/t4928-iseven-isodd-omitted (T-4928): ISEVEN and ISODD of a LET or LAMBDA omission.

Rows are tagged "rule" (the ToolPak rule predicts `#N/A`), "lift" (an operator or an IF over the omission) and "ctl"
(non-ToolPak IS functions and TYPE, and a blank cell). Predictions (docs/research/t4928/predictions.tsv), committed
before the fill: H_na, `#N/A` for the rule rows and the engine's value elsewhere; H_zero, the engine as it stands.
Run: python3 docs/research/t4928/gen.py > eval/suites/probes/cases/t4928-iseven-isodd-omitted.case.json
"""

import json
import sys

ROWS = """
rule | LET(e,,ISEVEN(e))
rule | LET(e,,ISODD(e))
rule | LAMBDA(a,b,ISEVEN(b))(1,)
rule | LAMBDA(a,b,ISODD(b))(1,)
lift | LET(e,,ISEVEN(e+0))
lift | LET(e,,ISODD(e+0))
lift | ROWS(LET(e,,ISEVEN(IF({1;1},e))))
ctl | LET(e,,ISNUMBER(e))
ctl | LET(e,,ISBLANK(e))
ctl | LET(e,,TYPE(e))
ctl | ISEVEN(Z1)
ctl | ISODD(Z1)
"""
TAGGED = [tuple(part.strip() for part in line.split(" | ")) for line in ROWS.strip().splitlines()]


def main() -> None:
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "ISEVEN and ISODD of a LET value or LAMBDA parameter bound to nothing (T-4928), with operator and lifted "
            "forms and non-ToolPak controls (Z1 blank). Predictions in docs/research/t4928/predictions.tsv committed "
            "before the excel-graph fill: H_na (#N/A for the rule rows) against H_zero (the engine before T-4928)."
        ),
        "tags": ["M5", "information", "probe"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": {f"A{i}": f"={f}" for i, (_, f) in enumerate(TAGGED, start=1)}}],
    }
    sys.stdout.write(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
