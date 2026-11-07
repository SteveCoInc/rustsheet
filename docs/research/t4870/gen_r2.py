"""Generate probes/t4870-r2-empty-optional (T-4870 round 2): an empty optional argument of the ToolPak date functions.

Round 1 (probes/t4870-empty-date-args) showed a written-empty optional argument (holidays, return_type, basis) acting
as omitted, which no round-1 hypothesis predicted. Fresh calls at other arguments test that rule; each probe is paired
with the same call with the argument left out, whose engine value is the prediction H_omitted
(docs/research/t4870/predictions-r2.tsv, committed before the fill).
Run: python3 docs/research/t4870/gen_r2.py [pairs] > eval/suites/probes/cases/t4870-r2-empty-optional.case.json
"""

import json
import sys

# Each probe ends in a written-empty optional argument; its prediction is the call without it. The two variable rows
# name their omitted call explicitly.
PROBES = """
WORKDAY(45000,10,) WORKDAY(45000,-7,) NETWORKDAYS(45000,45030,) NETWORKDAYS(45030,45000,) WEEKNUM(45010,) WEEKNUM(44927,)
YEARFRAC(45000,45200,) YEARFRAC(44985,45350,) WORKDAY.INTL(45000,10,11,) WORKDAY.INTL(45000,10,"0000011",)
NETWORKDAYS.INTL(45000,45030,11,) NETWORKDAYS.INTL(45000,45030,"1000001",)
""".split()
VARIABLE_ROWS = [("LET(h,,WORKDAY(45000,10,h))", "WORKDAY(45000,10)"), ("LAMBDA(a,r,WEEKNUM(a,r))(45010,)", "WEEKNUM(45010)")]
PAIRS = [(p, p[: -len(",)")] + ")") for p in PROBES] + VARIABLE_ROWS


def main() -> None:
    if sys.argv[1:] == ["pairs"]:
        for i, (probe, omitted) in enumerate(PAIRS, start=1):
            print(f"A{i}\t={probe}\t={omitted}")
        return
    cells = {f"A{i}": f"={probe}" for i, (probe, _) in enumerate(PAIRS, start=1)}
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4870 round 2: a written-empty optional argument of WORKDAY, NETWORKDAYS, WEEKNUM, YEARFRAC and the "
            ".INTL pair at fresh arguments, and a LET or LAMBDA parameter bound to nothing there. Predictions "
            "(H_omitted: the call with the argument left out) in docs/research/t4870/predictions-r2.tsv, committed "
            "before the excel-graph fill."
        ),
        "tags": ["M6", "dates", "probe"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    sys.stdout.write(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
