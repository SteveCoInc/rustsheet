"""Generate T-4465's fresh sweep, written after the rule was frozen (decision 0188 R4.2).

Usage: python3 docs/research/t4465/gen_sweep.py <out_dir>

Writes t4465-extent-sweep (SEQUENCE rows and columns) and t4465-extent-sweep-randarray (RANDARRAY
rows and columns, volatile, read through ROWS/COLUMNS): at bases drawn log-uniformly from
[1, 1048576] (RANDARRAY's columns from [1, 16384]) with random.Random(44652), each base stepped
by -k*2^-30 three times and +k*2^-30 once, k uniform in [96, 320] (the window that straddles the
shared rule's 2^-22 + 2^-33 bound and a 2^-23 epsilon). Every argument is an exact double.
"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

BASES_PER_COLUMN = 16

VARIANTS = [
    ("seq_rows", "=ROWS(SEQUENCE({x}))", 1_048_576),
    ("seq_cols", "=COLUMNS(SEQUENCE(1,{x}))", 1_048_576),
]
RANDARRAY = [
    ("randarray", "=ROWS(RANDARRAY({x}))", 1_048_576),
    ("randarray_cols", "=COLUMNS(RANDARRAY(1,{x}))", 16_384),
]


def points(rng: random.Random, top: int) -> list[str]:
    out = []
    for _ in range(BASES_PER_COLUMN):
        base = max(1, min(top, round(math.exp(rng.uniform(0, math.log(top))))))
        for sign in ("-", "-", "-", "+"):
            out.append(f"{base}{sign}{rng.randint(96, 320)}*2^-30")
    return out


def build(variants, rng: random.Random, volatile: bool) -> dict:
    cells: dict[str, str] = {}
    meta = []
    for index, (key, template, top) in enumerate(variants):
        col = chr(ord("A") + index)
        for row, x in enumerate(points(rng, top), start=1):
            cells[f"{col}{row}"] = template.format(x=x)
        meta.append(f"{col}: {key}")
    case: dict = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4465 fresh sweep (decision 0188 R4.2), written after the rule was frozen "
            "(docs/research/t4465/frozen-rule.md): log-uniform bases to 1048576, offsets k*2^-30, "
            "k in [96, 320], seeded random.Random(44652). Generator: docs/research/t4465/gen_sweep.py. "
            "Columns: " + "; ".join(meta) + "."
        ),
        "tags": ["M7", "volatile", "probe"] if volatile else ["M7", "probe"],
        "sheets": [{"name": "A", "cells": cells}],
        "expected": {},
    }
    if volatile:
        case["invariant"] = [f"A!{ref}" for ref in cells]
    return case


def main() -> None:
    out = Path(sys.argv[1])
    rng = random.Random(44652)
    for name, variants, volatile in (
        ("t4465-extent-sweep", VARIANTS, False),
        ("t4465-extent-sweep-randarray", RANDARRAY, True),
    ):
        (out / f"{name}.case.json").write_text(json.dumps(build(variants, rng, volatile), indent=2) + "\n")


if __name__ == "__main__":
    main()
