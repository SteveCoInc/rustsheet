"""Generate probes/t4923-sqrtpi-factdouble-omitted (T-4923): SQRTPI and FACTDOUBLE of a LET or LAMBDA omission.

Rows are tagged "rule" (the ToolPak rule predicts `#N/A`), "lift" (an operator or an IF over the omission) and "ctl"
(the variadic ToolPak controls). Predictions (docs/research/t4923/predictions.tsv), committed before the fill: H_na,
`#N/A` for the rule rows and the engine's value elsewhere; H_zero, the engine as it stands (the omission read as 0).
Run: python3 docs/research/t4923/gen.py > eval/suites/probes/cases/t4923-sqrtpi-factdouble-omitted.case.json
"""

import json
import sys

ROWS = """
rule | LET(e,,SQRTPI(e))
rule | LAMBDA(a,b,SQRTPI(b))(1,)
rule | LET(e,,FACTDOUBLE(e))
rule | LAMBDA(a,b,FACTDOUBLE(b))(1,)
lift | ROWS(LET(e,,SQRTPI(IF({1;1},e))))
lift | LET(e,,SQRTPI(e+0))
lift | LET(e,,FACTDOUBLE(e+0))
ctl | LET(e,,GCD(e,4))
ctl | LET(e,,GCD(4,e))
ctl | LAMBDA(a,b,LCM(a,b))(4,)
ctl | LET(e,,MULTINOMIAL(e,4))
ctl | SQRTPI(Z1)
ctl | FACTDOUBLE(Z1)
"""
TAGGED = [tuple(part.strip() for part in line.split(" | ")) for line in ROWS.strip().splitlines()]


def main() -> None:
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "SQRTPI and FACTDOUBLE of a LET value or LAMBDA parameter bound to nothing (T-4923), with lifted forms and "
            "the variadic ToolPak controls (Z1 blank). Predictions in docs/research/t4923/predictions.tsv committed "
            "before the excel-graph fill: H_na (#N/A for the rule rows) against H_zero (the omission read as 0)."
        ),
        "tags": ["M5", "math", "probe"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": {f"A{i}": f"={f}" for i, (_, f) in enumerate(TAGGED, start=1)}}],
    }
    sys.stdout.write(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
