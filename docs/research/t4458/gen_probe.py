"""T-4458: generate probes/t4458-bessely-tiny-x-cuts.

One column per BESSELY order 1..7, x descending, bracketing the engine's predicted #NUM! cut
(the first x at which a stored double overflows: 1/x for order 1, the upward recurrence's
product for orders 2..7) at decade, 1-digit, 3-digit and 4-digit resolution. Order 1's cut
(1/x overflows below 1/DBL_MAX, a subnormal) is left out: see bisect() and fill1/.
Usage: python3 gen_probe.py OUT.case.json, or --bisect OUT.case.json for the second fill
"""

import json
import math
import sys

# The engine's cuts (bisected on a double model of excel_bessel_y; 3 significant digits suffice
# to centre the grid, the probe brackets Excel's own cut).
CUTS = {1: 5.5627e-309, 2: 8.416e-155, 3: 3.048e-103, 4: 2.030e-77, 5: 6.710e-62,
        6: 1.545e-51, 7: 3.998e-44}
COLS = "ABCDEFG"


MIN_NORMAL = 2.2250738585072014e-308
BASE = "2.5E-308"


def arg(x: str) -> str:
    """The BESSELY argument for literal x. Excel reads a subnormal literal as 0
    (numbers.md "Subnormal flush edges") and Excel for the web refuses a workbook holding one,
    so a subnormal x is the exact difference of two normal doubles (`3E-308-2.5E-308` is a kept
    subnormal, arrays.md); the difference is exact at this scale (both are multiples of 2^-1074)."""
    v = float(x)
    if v >= MIN_NORMAL:
        return x
    return f"{float(BASE) + v!r}-{BASE}".replace("e", "E")


def lit(m: float, e: int) -> str:
    """A decimal literal m*10^e, shortest form."""
    s = f"{m:.6f}".rstrip("0").rstrip(".")
    return f"{s}E{e}"


def xs_for(c: float) -> list[str]:
    e = math.floor(math.log10(c))
    m = c / 10**e
    out: dict[str, float] = {}

    def add(mm: float, ee: int) -> None:
        mm = round(mm, 6)
        if mm >= 10:
            mm, ee = round(mm / 10, 6), ee + 1
        if mm < 1:
            mm, ee = round(mm * 10, 6), ee - 1
        if mm <= 0:
            return
        out[lit(mm, ee)] = mm * 10.0**ee if ee > -300 else float(lit(mm, ee))

    for k in range(-3, 4):
        add(1, e + k)
    for ee in (e - 1, e, e + 1):
        for d in range(1, 10):
            add(d, ee)
    m2 = round(m, 2)
    for i in range(-10, 11):
        add(m2 + i * 0.01, e)
    m3 = round(m, 3)
    for i in range(-10, 11):
        add(m3 + i * 0.001, e)
    return sorted(out, key=lambda s: -float(s))


def main() -> None:
    cells: dict[str, str] = {}
    desc = []
    for n, col in zip(range(1, 8), COLS):
        # No subnormal x: in the first fill (docs/research/t4458/fill1) the same subnormal x
        # gave #NUM! in one cell and 0 in another, and the workbook's other cells stopped
        # reproducing (BESSELY(3E-43,7) #NUM! there, a number in the bisection fill).
        xs = [x for x in xs_for(CUTS[n]) if float(x) >= MIN_NORMAL]
        for row, x in enumerate(xs, 1):
            cells[f"{col}{row}"] = f"=BESSELY({arg(x)},{n})"
        desc.append(f"{col} order {n}")
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4458: BESSELY at a tiny x, each column brackets one order's #NUM! cut "
        "(x descending). Columns: " + "; ".join(desc) + ".",
        "tags": ["M9", "probe", "engineering", "T-4458", "bessel"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    with open(sys.argv[1], "w", encoding="utf-8") as f:
        json.dump(case, f, indent=1)
        f.write("\n")
    print(len(cells), "cells")




def bisect(path: str) -> None:
    """Second fill: the first fill put order 7's cut between 4E-43 and 3E-43, a decade above the
    recurrence's overflow (3.998E-44), so A steps it at 3 digits; B..D map orders 8..10 over the
    decades from their recurrence-overflow cut (1.48E-38, 3.21E-34, 9.59E-31) up four more."""
    cells: dict[str, str] = {}
    for row, i in enumerate(range(400, 299, -1), 1):
        cells[f"A{row}"] = f"=BESSELY({lit(i / 100, -43)},7)"
    for col, n, e0 in (("B", 8, -39), ("C", 9, -35), ("D", 10, -32)):
        xs = [(d, e) for e in range(e0 + 5, e0 - 1, -1) for d in range(9, 0, -1)]
        for row, (d, e) in enumerate(xs, 1):
            cells[f"{col}{row}"] = f"=BESSELY({lit(d, e)},{n})"
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4458 bisection: A BESSELY order 7 from 4E-43 down to 3E-43 in 1E-45 steps; "
        "B, C, D orders 8, 9, 10 over mantissas 9..1 of six decades from the recurrence-overflow "
        "cut's decade up (x descending).",
        "tags": ["M9", "probe", "engineering", "T-4458", "bessel"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    with open(path, "w", encoding="utf-8") as f:
        json.dump(case, f, indent=1)
        f.write("\n")
    print(len(cells), "cells")


if __name__ == "__main__":
    if sys.argv[1:2] == ["--bisect"]:
        bisect(sys.argv[2])
    else:
        main()
