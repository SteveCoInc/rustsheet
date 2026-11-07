"""Generate probes/t4870-r3-order-and-lift (T-4870 round 3): where the written-empty `#N/A` sits among argument errors,
and how it lifts.

Asked for by the fidelity review of #3925: the first implementation put the empty-argument `#N/A` before every other
argument's error and before lifting (H_first). The verified MROUND and GCD rules read arguments in order and spread an
empty argument over an array (arrays.md "MROUND keeps its scalar rules", coercion.md), which predicts H_order: each
function's own read order (left to right; YEARFRAC reads basis first, NETWORKDAYS.INTL its weekend first) with the
empty argument's `#N/A` at its position, and one result per element of a lifted call. Predictions are written here,
committed before the excel-graph fill.
Run: python3 docs/research/t4870/gen_r3.py [predictions] > eval/suites/probes/cases/t4870-r3-order-and-lift.case.json
"""

import json
import sys

# formula | H_order | H_first, one row per line
TABLE = """
EDATE(1/0,) | #DIV/0! | #N/A
EDATE("x",) | #VALUE! | #N/A
EDATE(,1/0) | #N/A | #N/A
EDATE(,"x") | #N/A | #N/A
EOMONTH(-1,) | #NUM! | #N/A
WORKDAY(1/0,) | #DIV/0! | #N/A
WORKDAY(,1/0) | #N/A | #N/A
WORKDAY(1,,1/0) | #N/A | #N/A
NETWORKDAYS(1/0,) | #DIV/0! | #N/A
NETWORKDAYS(-1,) | #NUM! | #N/A
NETWORKDAYS.INTL(1,,99) | #NUM! | #N/A
YEARFRAC(1/0,) | #DIV/0! | #N/A
YEARFRAC(,1,9) | #NUM! | #N/A
YEARFRAC(1,,9) | #NUM! | #N/A
LAMBDA(a,b,EDATE(a,b))(1/0,) | #DIV/0! | #N/A
ROWS(EDATE({1;2},)) | 2 | #N/A
INDEX(EDATE({1;2},),2) | #N/A | #N/A
INDEX(EDATE({1;"x"},),2) | #N/A | #N/A
ROWS(EOMONTH(,{1;2})) | 2 | #N/A
ROWS(WEEKNUM(,{1;2})) | 2 | #N/A
ROWS(YEARFRAC({1;2},)) | 2 | #N/A
INDEX(YEARFRAC({1;2},),1) | #N/A | #N/A
ROWS(WORKDAY({1;2},)) | 2 | #N/A
ROWS(NETWORKDAYS(,{1;2})) | 2 | #N/A
"""
ROWS = [tuple(part.strip() for part in line.split(" | ")) for line in TABLE.strip().splitlines()]


def main() -> None:
    if sys.argv[1:] == ["predictions"]:
        print("cell\tformula\tH_order\tH_first")
        for i, (f, order, first) in enumerate(ROWS, start=1):
            print(f"A{i}\t={f}\t{order}\t{first}")
        return
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4870 round 3: the written-empty #N/A of the ToolPak date functions among other argument errors and "
            "over a lifted array. Predictions in docs/research/t4870/predictions-r3.tsv committed before the "
            "excel-graph fill: H_order (each function's read order, the #N/A at the empty argument's position, one "
            "result per element) against H_first (the #N/A first, one scalar)."
        ),
        "tags": ["M6", "dates", "probe"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": {f"A{i}": f"={f}" for i, (f, _, _) in enumerate(ROWS, start=1)}}],
    }
    sys.stdout.write(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
