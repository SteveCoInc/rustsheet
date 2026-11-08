"""Shared pieces of the t4466 sweep generators: the tie expression and the case-plus-predictions writer."""

import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
FORMATS = {
    ".95": ["[s].0", "[m]:ss.0", "[h]:mm:ss.0"],
    ".995": ["[s].00", "[m]:ss.00", "[h]:mm:ss.00"],
    ".9995": ["[s].000", "[m]:ss.000", "[h]:mm:ss.000"],
}
UNIT = {"s": 1, "m": 60, "h": 3600}


def draw_tie(rng, days: list[int]) -> tuple[int, str, str]:
    """Draw a day, a unit and a whole second just below that unit's carry; return the day, the tie seconds and the
    unit's format with the tie's digits."""
    d = rng.choice(days)
    unit = rng.choice("sssmh")
    if unit == "s":
        k = rng.randrange(0, 86399)
    elif unit == "m":
        k = rng.randrange(0, 1440) * 60 + 59
    else:
        k = rng.randrange(0, 24) * 3600 + 3599
    if d == 0 and k == 86399:
        k = 86398
    tail = rng.choice(list(FORMATS))
    return d, f"{k}{tail}", FORMATS[tail][{"s": 0, "m": 1, "h": 2}[unit]]


def tie_expr(d: int, secs: str, off: int) -> tuple[str, float]:
    """The TEXT argument for day d plus secs seconds, nudged off ulps, and the double it evaluates to."""
    base = (d + float(secs) / 86400) if d else float(secs) / 86400
    e = round(math.log2(math.ulp(base)))
    expr = (f"{d}+{secs}/86400" if d else f"{secs}/86400") + ("" if off == 0 else f"{off:+d}*2^{e}")
    return expr, (base + off * 2.0**e if off else base)


def write_case(path: str, description: str, cells: dict[str, str], preds: dict, preds_name: str) -> None:
    """Write the probe case to path and its predictions beside this file, committed before the fill."""
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M10", "text", "probe"],
        "sheets": [{"name": "Sheet1", "cells": cells}],
        "expected": {},
    }
    with open(path, "w") as fh:
        fh.write(json.dumps(case, indent=2) + "\n")
    with open(os.path.join(HERE, preds_name), "w") as fh:
        json.dump(preds, fh, indent=0, sort_keys=True)
