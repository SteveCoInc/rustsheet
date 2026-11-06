"""Generate T-4465's probe cases: array-constructor extents just below and above an integer.

Usage: python3 docs/research/t4465/gen_probe.py <out_dir>

Writes five rustsheet-case/v1 files:
- t4465-extent-near-integer: SEQUENCE (rows, columns), MAKEARRAY, TAKE, DROP, EXPAND, WRAPROWS and
  WRAPCOLS at the fixed offsets of the task, below and above 0, 1, 2, 1000 and 100000 (and -1 for
  TAKE and DROP, which accept negative counts);
- t4465-extent-near-integer-randarray: the same for RANDARRAY (volatile, read through ROWS);
- t4465-extent-near-integer-holdout and -holdout-randarray: seeded offsets k*2^-30 (k from
  random.Random(4465)), scored only after the rule is frozen (decision 0188 R4);
- t4465-extent-near-integer-randarray-cols: RANDARRAY's columns at the fixed offsets, added after
  the freeze as a fresh sweep.
Every argument is an exact double: base +/- (powers of two), computed by Excel's own arithmetic.
"""

from __future__ import annotations

import json
import random
import sys
from pathlib import Path

# Offsets as formula text, from the task's acceptance criteria plus 2^-23 + 2^-34 and 2^-23 + 2^-33
# (either side of the shared rule's negative-side tolerance).
OFFSETS = [
    "2^-24",
    "2^-23",
    "(2^-23+2^-34)",
    "(2^-23+2^-33)",
    "1.5*2^-23",
    "2^-22",
    "(2^-22+2^-33)",
    "(2^-22+2^-32)",
    "2^-21",
]

N_LONG = 100001  # source length for TAKE and DROP, longer than the largest count
N_WRAP = 3  # source length for WRAPCOLS (rows = wrap count)

# (column key, formula template with {x}, bases)
VARIANTS = [
    ("seq_rows", "=ROWS(SEQUENCE({x}))", [0, 1, 2, 1000, 100000]),
    ("seq_cols", "=COLUMNS(SEQUENCE(1,{x}))", [0, 1, 2, 1000]),
    ("makearray", "=ROWS(MAKEARRAY({x},1,LAMBDA(r,c,0)))", [0, 1, 2, 1000, 100000]),
    ("take", f"=ROWS(TAKE(SEQUENCE({N_LONG}),{{x}}))", [-1, 0, 1, 2, 1000, 100000]),
    ("drop", f"=ROWS(DROP(SEQUENCE({N_LONG}),{{x}}))", [-1, 0, 1, 2, 1000, 100000]),
    ("expand", "=ROWS(EXPAND(1,{x}))", [0, 1, 2, 1000, 100000]),
    ("wraprows", "=COLUMNS(WRAPROWS(SEQUENCE(1,3),{x}))", [0, 1, 2, 1000]),
    ("wrapcols", f"=ROWS(WRAPCOLS(SEQUENCE({N_WRAP}),{{x}}))", [0, 1, 2, 1000, 100000]),
]
RANDARRAY = [("randarray", "=ROWS(RANDARRAY({x}))", [0, 1, 2, 1000, 100000])]
# Added after the rule was frozen (docs/research/t4465/frozen-rule.md): a fresh sweep of the
# columns argument, which the fixed and hold-out cases did not step.
RANDARRAY_COLUMNS = [("randarray_cols", "=COLUMNS(RANDARRAY(1,{x}))", [0, 1, 2, 1000])]


def arg(base: int, sign: str, offset: str) -> str:
    """The argument text: base, then sign and offset (base 0 gives just the signed offset)."""
    if not offset:
        return str(base)
    if base == 0:
        return f"{sign}{offset}" if sign == "-" else offset
    return f"{base}{sign}{offset}"


def fixed_points(bases: list[int]) -> list[tuple[int, str, str]]:
    points = []
    for base in bases:
        points.append((base, "", ""))
        for sign in ("-", "+"):
            points.extend((base, sign, off) for off in OFFSETS)
    return points


def holdout_points(bases: list[int], rng: random.Random) -> list[tuple[int, str, str]]:
    points = []
    for base in bases:
        for sign, count in (("-", 4), ("+", 2)):
            for _ in range(count):
                points.append((base, sign, f"{rng.randint(96, 320)}*2^-30"))
    return points


def build(variants, point_fn, description: str, volatile: bool) -> dict:
    cells: dict[str, str] = {}
    meta: list[str] = []
    for col_index, (key, template, bases) in enumerate(variants):
        col = chr(ord("A") + col_index)
        for row, (base, sign, off) in enumerate(point_fn(bases), start=1):
            x = arg(base, sign, off)
            cells[f"{col}{row}"] = template.format(x=x)
        meta.append(f"{col}: {key}")
    case: dict = {
        "format": "rustsheet-case/v1",
        "description": description + " Columns: " + "; ".join(meta) + ".",
        "tags": ["M7", "volatile", "probe"] if volatile else ["M7", "probe"],
        "sheets": [{"name": "A", "cells": cells}],
        "expected": {},
    }
    if volatile:
        case["invariant"] = [f"A!{ref}" for ref in cells]
    return case


def main() -> None:
    out = Path(sys.argv[1])
    head = (
        "T-4465: the whole number an M7 array constructor reads from a rows, columns, count or wrap "
        "argument just below and above an integer, read through ROWS/COLUMNS. Generator: "
        "docs/research/t4465/gen_probe.py."
    )
    fixed = head + " Each column steps its argument by 0, then -o and +o for o in " + ", ".join(OFFSETS) + " around each base in turn."
    hold = head + " Hold-out (decision 0188 R4): offsets k*2^-30, k seeded from random.Random(4465) in [96, 320], four below and two above each base; scored only after the rule was frozen on the fixed case."
    rng = random.Random(4465)
    files = {
        "t4465-extent-near-integer": build(VARIANTS, fixed_points, fixed, False),
        "t4465-extent-near-integer-randarray": build(RANDARRAY, fixed_points, fixed + " RANDARRAY is volatile; ROWS of it does not depend on the draw.", True),
        "t4465-extent-near-integer-holdout": build(VARIANTS, lambda b: holdout_points(b, rng), hold, False),
        "t4465-extent-near-integer-holdout-randarray": build(RANDARRAY, lambda b: holdout_points(b, rng), hold + " RANDARRAY is volatile; ROWS of it does not depend on the draw.", True),
    }
    files["t4465-extent-near-integer-randarray-cols"] = build(
        RANDARRAY_COLUMNS,
        fixed_points,
        fixed + " Fresh sweep of RANDARRAY's columns, written after the rule was frozen. RANDARRAY is volatile; COLUMNS of it does not depend on the draw.",
        True,
    )
    for name, case in files.items():
        (out / f"{name}.case.json").write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
