"""T-4994 fresh probe (decision 0188 R4.2): BETA.INV at a shape of exactly one, after the rule was frozen
(seed 4994_2026_11_11).

The rule, found on the shape-one cells of probes/t4974-beta-inv-tiny-shape-num-boundary, t4713-beta-inv-beta-above-1-cut
and t3351-beta-inv-p-near-1 (124 cells): the closed form 1 - (1-p)^(1/beta) or p^(1/alpha) is Excel's value only where
it is a double strictly inside (0, 1) in AS 109's orientation (the flip at p > 1/2); elsewhere Excel returns its
search's own stop.

Strata, none chosen by Excel's answer; every argument is a literal of at most 15 significant digits:
- OneTiny: alpha = 1, beta from 1E-20 to 0.3; p log-uniform below 1/2 and 1 - q for q log-uniform below 1/2.
- TinyOne: the same with the shapes exchanged.
- OneLarge: one shape 1, the other from 3 to 8E4, both ways round.
- Edge: the other shape chosen so that the oriented closed form is 1 - 2^-k for k from 50 to 56 and beyond, where
  it stops being an interior double.
- Legacy: BETAINV on rows of the sheets above.

usage: python3 -I fresh.py CASE.json
"""

import json
import math
import random
import sys

rng = random.Random(4994_2026_11_11)


def r15(v: float) -> float:
    return float(f"{v:.15g}")


def lit(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def log_uniform(lo: float, hi: float) -> float:
    return r15(math.exp(rng.uniform(math.log(lo), math.log(hi))))


def p_draw() -> float:
    q = log_uniform(1e-12, 0.5)
    return q if rng.random() < 0.5 else r15(1 - q)


def one_tiny(count):
    return [(p_draw(), 1.0, log_uniform(1e-20, 0.3)) for _ in range(count)]


def tiny_one(count):
    return [(p_draw(), log_uniform(1e-20, 0.3), 1.0) for _ in range(count)]


def one_large(count):
    rows = []
    for i in range(count):
        other = log_uniform(3, 8e4)
        rows.append((p_draw(), 1.0, other) if i % 2 else (p_draw(), other, 1.0))
    return rows


def edge():
    """The other shape for which (1 - p)^(1/shape) is 2^-k: the oriented closed form 1 - 2^-k is an interior
    double up to k = 53 and rounds to 1 from k = 54."""
    rows = []
    for k in (40, 48, 50, 51, 52, 53, 54, 55, 56, 60, 80, 200, 1000):
        for p in (0.1, 0.5, 0.001):
            shape = r15(math.log1p(-p) / (-k * math.log(2)))
            rows.append((p, 1.0, shape))  # no flip: x itself nears 1
            rows.append((r15(1 - p), shape, 1.0))  # the flip: 1 - x nears 1
    return rows


strata = {
    "OneTiny": [("BETA.INV", *row) for row in one_tiny(60)],
    "TinyOne": [("BETA.INV", *row) for row in tiny_one(60)],
    "OneLarge": [("BETA.INV", *row) for row in one_large(40)],
    "Edge": [("BETA.INV", *row) for row in edge()],
}
strata["Legacy"] = [("BETAINV", *row[1:]) for name in ("OneTiny", "TinyOne", "Edge") for row in strata[name][:5]]

sheets = []
for name, rows in strata.items():
    cells = {}
    for i, (function, p, alpha, beta) in enumerate(rows, 1):
        assert all(float(lit(v)) == v for v in (p, alpha, beta)), (name, i)
        assert 0 < p < 1 and (alpha == 1.0 or beta == 1.0), (name, i)
        cells[f"A{i}"] = f"={function}({lit(p)},{lit(alpha)},{lit(beta)})"
    sheets.append({"name": name, "cells": cells})
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "BETA.INV at a shape of exactly one after its rule was frozen (T-4994, decision 0188 R4.2): the closed form "
        "where it is a double strictly inside (0, 1) in AS 109's orientation, the search's own stop elsewhere. "
        "Predictions in docs/research/t4994/fresh-engine.tsv, committed before the fill. OneTiny and TinyOne: the "
        "other shape from 1E-20 to 0.3, p on both sides of 1/2. OneLarge: the other shape from 3 to 8E4. Edge: the "
        "oriented closed form at 1 - 2^-k around k = 53. Legacy: BETAINV."
    ),
    "tags": ["M9", "statistical", "probe", "T-4994"],
    "sheets": sheets,
    "expected": {},
}
with open(sys.argv[1], "w") as case_file:
    json.dump(case, case_file, indent=1)
print({s["name"]: len(s["cells"]) for s in sheets}, sum(len(s["cells"]) for s in sheets))
