"""T-4465: probes of the whole number each M7 array constructor reads from an extent just below or above an integer.

Run from the repo root: python3 -I docs/research/t4465/gen_probe.py [main|holdout|fresh]. It writes the case and
docs/research/t4465/predictions-<set>.tsv, the extent each candidate rule reads (committed before the fill):
- shared: near_integer_whole (floor, but within 2^-22 + 2^-33 below an integer counts as it; half that below 0),
- eps: floor(x + 2^-23) (sequence.rs's EXTENT_EPSILON),
- f32: floor of x rounded to the nearest float32,
- floor: plain floor (RANDARRAY), trunc: toward zero (TAKE, DROP, EXPAND, WRAPROWS/WRAPCOLS today).

Sheet P: column A holds the extent x as a formula (`=1000-(2^-22+2^-33)`), columns B:J read one constructor each as
`IFERROR(ROWS(f)&"x"&COLUMNS(f),"E"&ERROR.TYPE(f))`. Sheet D holds nothing (TAKE/DROP read D!A1:A100003's blanks).
The main set steps 2^-24, 2^-23, 1.5*2^-23, 2^-22, 2^-22+2^-33, 2^-22+2^-32 and 2^-21 below and above 1, 2, 1000 and
100000, and below 0. The hold-out (split fixed before any fill; seed 4465) draws 36 rows: a base from 3, 7, 500,
30000, an offset 2^u with u uniform in [-24.5, -20.5], a side (below, above, below 0). The fresh sweep (seed 44651,
generated only after the rule was frozen) draws 36 more from bases 4, 12, 999, 65536 and u in [-26, -19].
"""

import json
import math
import random
import struct
import sys

COLS = {
    "B": ("SEQUENCE rows", "SEQUENCE({x})"),
    "C": ("SEQUENCE cols", "SEQUENCE(1,{x})"),
    "D": ("RANDARRAY rows", "RANDARRAY({x})"),
    "E": ("MAKEARRAY rows", "MAKEARRAY({x},1,LAMBDA(r,c,0))"),
    "F": ("TAKE rows", "TAKE(D!A1:A100003,{x})"),
    "G": ("DROP rows", "DROP(D!A1:A100003,{x})"),
    "H": ("EXPAND rows", "EXPAND(D!A1,{x})"),
    "I": ("WRAPROWS wrap", "WRAPROWS(SEQUENCE(1,3),{x})"),
    "J": ("WRAPCOLS wrap", "WRAPCOLS(SEQUENCE(3),{x})"),
}
# Columns past 16384 are an error whatever the rule: those cells are left out.
WIDE = {"C", "I"}

TOL = 2.0**-22 + 2.0**-33


def shared(x: float) -> float:
    tol = TOL / 2 if x < 0 else TOL
    w = math.floor(x)
    return w + (1 if x - w >= 1 - tol else 0)


def eps(x: float) -> float:
    return math.floor(x + 2.0**-23)


def f32(x: float) -> float:
    return math.floor(struct.unpack("f", struct.pack("f", x))[0])


def main_rows() -> list[tuple[str, float]]:
    offsets = [
        ("2^-24", 2.0**-24),
        ("2^-23", 2.0**-23),
        ("1.5*2^-23", 1.5 * 2.0**-23),
        ("2^-22", 2.0**-22),
        ("(2^-22+2^-33)", 2.0**-22 + 2.0**-33),
        ("(2^-22+2^-32)", 2.0**-22 + 2.0**-32),
        ("2^-21", 2.0**-21),
    ]
    rows = []
    for base in (1, 2, 1000, 100000):
        for sign in ("-", "+"):
            for text, d in offsets:
                rows.append((f"{base}{sign}{text}", base - d if sign == "-" else base + d))
    for text, d in offsets:
        rows.append((f"0-{text}", -d))
    return rows


def drawn_rows(seed: int, bases: tuple[int, ...], lo: float, hi: float) -> list[tuple[str, float]]:
    rng = random.Random(seed)
    rows = []
    for _ in range(36):
        base = rng.choice(bases)
        u = rng.uniform(lo, hi)
        side = rng.choice(("-", "+", "0"))
        # The offset written as a decimal (`0-d` below 0: Excel's unary minus binds before `^`) of 17 significant digits, which reads back as the same double.
        text = f"{2.0**u:.17G}"
        d = float(text)
        if side == "0":
            rows.append((f"0-{text}", -d))
        else:
            rows.append((f"{base}{side}{text}", base - d if side == "-" else base + d))
    return rows


def write(name: str, rows: list[tuple[str, float]], description: str) -> None:
    cells: dict[str, object] = {}
    invariant = []
    for i, (text, x) in enumerate(rows, start=1):
        cells[f"A{i}"] = f"={text}"
        for col, (_, form) in COLS.items():
            if col in WIDE and x > 16384:
                continue
            f = form.format(x=f"$A{i}")
            cells[f"{col}{i}"] = f'=IFERROR(ROWS({f})&"x"&COLUMNS({f}),"E"&ERROR.TYPE({f}))'
            if col == "D":
                invariant.append(f"P!{col}{i}")
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M7", "probe"],
        "sheets": [{"name": "P", "cells": cells}, {"name": "D", "cells": {}}],
        "expected": {},
        "invariant": invariant,
    }
    with open(f"eval/suites/probes/cases/t4465-extent-near-integer-{name}.case.json", "w") as fh:
        json.dump(case, fh, indent=2)
        fh.write("\n")
    with open(f"docs/research/t4465/predictions-{name}.tsv", "w") as fh:
        fh.write("row\tx\tshared\teps\tf32\tfloor\ttrunc\n")
        for i, (text, x) in enumerate(rows, start=1):
            fh.write(f"{i}\t{text}\t{shared(x):g}\t{eps(x):g}\t{f32(x):g}\t{math.floor(x):g}\t{math.trunc(x):g}\n")


DESC = (
    "T-4465: the whole number SEQUENCE (rows, columns), RANDARRAY, MAKEARRAY, TAKE, DROP, EXPAND, WRAPROWS and "
    "WRAPCOLS read from an extent just below or above an integer. P!A holds the extent; B:J read one constructor "
    "each as ROWS&\"x\"&COLUMNS, or E and its ERROR.TYPE (B SEQUENCE rows, C SEQUENCE cols, D RANDARRAY, E MAKEARRAY, "
    "F TAKE, G DROP, H EXPAND, I WRAPROWS, J WRAPCOLS). Generated by docs/research/t4465/gen_probe.py ({which}); the "
    "candidate rules' predictions are in docs/research/t4465/predictions-{name}.tsv."
)

if __name__ == "__main__":
    which = sys.argv[1] if len(sys.argv) > 1 else "main"
    if which == "main":
        write("main", main_rows(), DESC.format(which="main set", name="main"))
    elif which == "holdout":
        write("holdout", drawn_rows(4465, (3, 7, 500, 30000), -24.5, -20.5),
              DESC.format(which="hold-out, seed 4465, fixed before any fill", name="holdout"))
    elif which == "fresh":
        write("fresh", drawn_rows(44651, (4, 12, 999, 65536), -26.0, -19.0),
              DESC.format(which="fresh sweep, seed 44651, generated after the rule was frozen", name="fresh"))
