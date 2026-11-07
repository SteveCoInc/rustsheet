"""Generate probes/t4530-ipmt-band (T-4530): IPMT and PPMT side by side in the band nper < period < nper + 1.

Rows: rate in {0, 1E-10, 0.05, -0.5} x type in {0, 1} x nper in {12, 12.5}; pv -1234.56789, fv 98.7654321.
Columns A..E IPMT, F..J PPMT, at periods nper, nper + 1E-9, nper + 0.5, nper + 0.999 and the double just
below nper + 1 (written as (nper + 1) - 2^-49, since 13 and 13.5 share the binade [8, 16)).
Run: python3 docs/research/t4530/gen.py > eval/suites/probes/cases/t4530-ipmt-band.case.json
"""

import json

RATES = ["0", "1E-10", "0.05", "-0.5"]
TYPES = ["0", "1"]
NPERS = [("12", "13"), ("12.5", "13.5")]
PV, FV = "-1234.56789", "98.7654321"


def periods(nper: str, top: str) -> list[str]:
    frac = {"12": ("12.000000001", "12.5", "12.999"), "12.5": ("12.500000001", "13", "13.499")}[nper]
    return [nper, frac[0], frac[1], frac[2], f"({top}-2^-49)"]


def main() -> None:
    cells: dict[str, str] = {}
    row = 0
    for rate in RATES:
        for typ in TYPES:
            for nper, top in NPERS:
                row += 1
                for i, per in enumerate(periods(nper, top)):
                    args = f"{rate},{per},{nper},{PV},{FV},{typ}"
                    cells[f"{'ABCDE'[i]}{row}"] = f"=IPMT({args})"
                    cells[f"{'FGHIJ'[i]}{row}"] = f"=PPMT({args})"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "IPMT and PPMT in the band nper < period < nper + 1 (T-4530), predictions in docs/research/t4530 "
            "committed before the fill. Rows 1-16: rate 0, 1E-10, 0.05, -0.5 x type 0, 1 x nper 12, 12.5; pv "
            "-1234.56789, fv 98.7654321. Columns A-E IPMT, F-J PPMT, at periods nper, nper + 1E-9, nper + 0.5, "
            "nper + 0.999 and the double below nper + 1. H_value: IPMT is a value in the band, as PPMT is; "
            "H_nper: IPMT is #NUM! once period > nper."
        ),
        "tags": ["M9", "financial", "ipmt", "probe"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    print(json.dumps(case, indent=2))


main()
