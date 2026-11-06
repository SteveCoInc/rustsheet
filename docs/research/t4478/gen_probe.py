"""Writes probes/t4478-reduce-scan-reference-accumulator (T-4478): whether a REDUCE or SCAN accumulator
that a LAMBDA returned as a reference is still a reference on the next step and in the result.

Sheet Calc: data A1:A4 = 1, 2, 3, "x", B1:B2 = 10, 20; Sheet2!A1 = 5, Sheet3!A1 = 7. Each expression E in
column D is read in E:H as ISREF(E), its shape ROWS&"x"&COLUMNS, ERROR.TYPE of its first cell (0 when not
an error) and ARRAYTOTEXT(E,1). Columns J, L, N and P hold four bare formulas over empty cells below them,
so their spill extent is scored.
"""

import json
import sys
from pathlib import Path

EXPRESSIONS = [
    # The accumulator after a step that returned a value, and after one that returned `v`.
    "REDUCE(A1,A2:A3,LAMBDA(a,v,IF(ISREF(a),1,0)+IF(ISREF(v),10,0)))",
    'REDUCE(A1,A2:A3,LAMBDA(a,v,IF(ISREF(a),v,"no")))',
    "REDUCE(A1,A2:A3,LAMBDA(a,v,v))",
    "REDUCE(A1,A2:A3,LAMBDA(a,v,IF(ISREF(a),a,0)+0*ISREF(v)))",
    # Ranges built across steps.
    "REDUCE(A1,A2:A3,LAMBDA(a,v,a:v))",
    "SUM(REDUCE(A1,A2:A3,LAMBDA(a,v,a:v)))",
    "AREAS(REDUCE(A1,A2:A3,LAMBDA(a,v,(a,v))))",
    "REDUCE(A1,{1,2},LAMBDA(a,v,OFFSET(a,1,0)))",
    "REDUCE(A1,A2:A3,LAMBDA(a,v,ROWS(a:v)))",
    # The omitted initial value: the first element seeds the accumulator.
    "REDUCE(,A1:A3,LAMBDA(a,v,IF(ISREF(a),1,0)))",
    "REDUCE(,A1:A3,LAMBDA(a,v,v))",
    # SCAN.
    "SCAN(A1,A2:A3,LAMBDA(a,v,v))",
    "SCAN(A1,A2:A4,LAMBDA(a,v,IF(ISREF(a),v,0)))",
    "SCAN(A1,A2:A3,LAMBDA(a,v,ROWS(a:v)))",
    "SCAN(A1,A2:A3,LAMBDA(a,v,a:v))",
    # A union and a 3-D span as the initial value.
    "AREAS(REDUCE((A1,A3),{1},LAMBDA(a,v,a)))",
    "REDUCE((A1,A3),{1},LAMBDA(a,v,ISREF(a)))",
    "REDUCE(Sheet2:Sheet3!A1,{1},LAMBDA(a,v,ISREF(a)))",
    "REDUCE(Sheet2:Sheet3!A1,{1},LAMBDA(a,v,a))",
    # The range forms again with parameter names that cannot read as columns (`a:v` is also the column
    # range A:V), so the accumulator itself is the range's first corner.
    "REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,p_a:p_v))",
    "SUM(REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,p_a:p_v)))",
    "AREAS(REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,(p_a,p_v))))",
    "REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,ROWS(p_a:p_v)))",
    "SCAN(A1,A2:A3,LAMBDA(p_a,p_v,ROWS(p_a:p_v)))",
    "SCAN(A1,A2:A3,LAMBDA(p_a,p_v,p_a:p_v))",
    "SCAN(A1,A2:A3,LAMBDA(p_a,p_v,SUM(p_a:p_v)))",
    "REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,SUM(p_a:p_v)))",
]

BARE = {
    "J1": "=REDUCE(A1,A2:A3,LAMBDA(a,v,a:v))",
    "L1": "=REDUCE(A1,A2:A3,LAMBDA(a,v,v))",
    "N1": "=SCAN(A1,A2:A3,LAMBDA(a,v,v))",
    "P1": "=REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,p_a:p_v))",
}


def main(out: Path) -> None:
    cells: dict[str, object] = {"A1": 1, "A2": 2, "A3": 3, "A4": "x", "B1": 10, "B2": 20}
    for row, expr in enumerate(EXPRESSIONS, start=1):
        cells[f"D{row}"] = expr
        cells[f"E{row}"] = f"=ISREF({expr})"
        cells[f"F{row}"] = f'=IFERROR(ROWS({expr})&"x"&COLUMNS({expr}),"ERR")'
        cells[f"G{row}"] = f"=IFERROR(ERROR.TYPE(INDEX({expr},1,1)),0)"
        cells[f"H{row}"] = f"=_xlfn.ARRAYTOTEXT({expr},1)"
    cells.update(BARE)
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4478: REDUCE and SCAN accumulators a LAMBDA returns as references. Each expression in "
        "Calc!D is read as ISREF (E), ROWS&\"x\"&COLUMNS (F), ERROR.TYPE of its first cell (G) and ARRAYTOTEXT (H); "
        "J1, L1, N1 and P1 are bare formulas whose spill is scored. A1:A4 = 1, 2, 3, \"x\"; B1:B2 = 10, 20; "
        "Sheet2!A1 = 5, Sheet3!A1 = 7.",
        "tags": ["M7", "probe", "dynamic-arrays", "T-4478"],
        "sheets": [
            {"name": "Calc", "cells": cells},
            {"name": "Sheet2", "cells": {"A1": 5}},
            {"name": "Sheet3", "cells": {"A1": 7}},
        ],
        "expected": {},
    }
    out.write_text(json.dumps(case, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
