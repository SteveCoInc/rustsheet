"""Generate probes/t4870-empty-date-args (T-4870): written-empty arguments of the ToolPak date functions.

Each row is one call with an empty argument, tagged "req" (a required argument is empty), "opt" (only an optional one
is) or "ctl" (a non-ToolPak control). Predictions (docs/research/t4870/predictions.tsv), committed before the fill:
H_zero, the engine before T-4870 (an empty argument read as 0 or as the engine reads it today); H_na_req, `#N/A`
where a required argument is empty and H_zero elsewhere (the rule MROUND, RANDBETWEEN, IMSUB and IMDIV show);
H_na_all, `#N/A` for any empty argument of a ToolPak function. Controls keep H_zero in every hypothesis.
Run: python3 docs/research/t4870/gen.py > eval/suites/probes/cases/t4870-empty-date-args.case.json
"""

import json
import sys

ROWS = [
    ("EDATE(,1)", "req"), ("EDATE(1,)", "req"), ("EDATE(,)", "req"),
    ("EOMONTH(,1)", "req"), ("EOMONTH(40000,)", "req"), ("EOMONTH(,)", "req"),
    ("WORKDAY(,1)", "req"), ("WORKDAY(1,)", "req"), ("WORKDAY(,)", "req"), ("WORKDAY(1,5,)", "opt"),
    ("NETWORKDAYS(,1)", "req"), ("NETWORKDAYS(1,)", "req"), ("NETWORKDAYS(,)", "req"), ("NETWORKDAYS(1,10,)", "opt"),
    ("WEEKNUM(,1)", "req"), ("WEEKNUM(45000,)", "opt"), ("WEEKNUM(,)", "req"),
    ("YEARFRAC(,1)", "req"), ("YEARFRAC(1,)", "req"), ("YEARFRAC(,)", "req"), ("YEARFRAC(13816,16816,)", "opt"),
    ("WORKDAY.INTL(,5,1)", "req"), ("WORKDAY.INTL(1,,1)", "req"), ("WORKDAY.INTL(1,5,1,)", "opt"),
    ("NETWORKDAYS.INTL(,10,1)", "req"), ("NETWORKDAYS.INTL(1,,1)", "req"), ("NETWORKDAYS.INTL(1,10,1,)", "opt"),
    ("LAMBDA(a,b,EDATE(a,b))(1,)", "req"), ("LET(e,,EDATE(1,e))", "req"),
    ("WEEKDAY(45000,)", "ctl"), ("DAYS(10,)", "ctl"),
]


def main() -> None:
    cells = {f"A{i}": f"={f}" for i, (f, _) in enumerate(ROWS, start=1)}
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "Written-empty arguments of the ToolPak date functions (T-4870): EDATE, EOMONTH, WORKDAY, NETWORKDAYS, "
            "WEEKNUM, YEARFRAC and the .INTL pair, one empty argument at a time and all empty, the LAMBDA and LET "
            "omitted-variable paths, and WEEKDAY and DAYS as controls. Predictions in "
            "docs/research/t4870/predictions.tsv committed before the excel-graph fill (H_zero, H_na_req, H_na_all)."
        ),
        "tags": ["M6", "dates", "probe"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    sys.stdout.write(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
