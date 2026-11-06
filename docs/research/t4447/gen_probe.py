"""Writes probes/t4447-sort-fractional-keys (T-4447): SORT's fractional and near-integer
sort_order and sort_index, each result read as TEXTJOIN or, on an error, ERROR.TYPE."""

import json
import sys
from pathlib import Path

ORDERS = ["-0.0001", "-0.5", "-0.9999", "-1.5", "0.0001", "0.5", "0.9999", "1.5", "1.9999",
          "1-2^-21", "1-2^-22", "1-2^-23", "1-2^-24", "-1-2^-22", "-1-2^-23", "-1-2^-24",
          "2-2^-22", '"-0.5"', "-1+2^-23", "-1+2^-21"]
INDEXES = ["0.9999", "1-2^-23", "1-2^-21", "1.5", "2.5", "2-2^-23", "2+2^-22", "-0.5", "1-2^-24",
           "3-2^-23"]


def read(call: str) -> str:
    return f'=IFERROR(TEXTJOIN(",",,{call}),ERROR.TYPE({call}))'


def main(out: Path) -> None:
    cells: dict[str, object] = {"A1": 3, "B1": "c", "A2": 1, "B2": "a", "A3": 2, "B3": "b"}
    row = 1
    for order in ORDERS:
        cells[f"D{row}"] = order
        cells[f"E{row}"] = read(f"SORT(A1:B3,1,{order})")
        cells[f"F{row}"] = read(f"SORT(A1:B3,1,IF({{1}},{order}))")
        row += 1
    cells[f"D{row}"] = "{-0.5,1} keys {1,2}"
    cells[f"E{row}"] = read("SORT(A1:B3,{1,2},{-0.5,1})")
    row += 1
    cells[f"D{row}"] = "{1-2^-23,-1-2^-23} keys {1,2}"
    cells[f"E{row}"] = read("SORT(A1:B3,{1,2},HSTACK(1-2^-23,-1-2^-23))")
    row = 1
    for index in INDEXES:
        cells[f"H{row}"] = index
        cells[f"I{row}"] = read(f"SORT(A1:B3,{index},-1)")
        cells[f"J{row}"] = read(f"SORT(TRANSPOSE(A1:B3),{index},-1,TRUE)")
        row += 1
    cells[f"H{row}"] = "{1.5,2.0000001}"
    cells[f"I{row}"] = read("SORT(A1:B3,{1.5,2.0000001},-1)")
    cells[f"J{row}"] = read("SORT(TRANSPOSE(A1:B3),{1.5,2.0000001},-1,TRUE)")
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4447: SORT's sort_order (column E scalar, F a one-element array IF({1},x)) and sort_index "
        "(column I, and J by_col over TRANSPOSE) at fractions and 2^-21..2^-24 from an integer, over "
        "A1:B3 = (3,c),(1,a),(2,b); each read as TEXTJOIN, or ERROR.TYPE on an error.",
        "tags": ["M7", "probe", "lookup", "T-4447"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    out.write_text(json.dumps(case, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
