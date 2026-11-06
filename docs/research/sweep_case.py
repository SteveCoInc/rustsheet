"""Shared pieces of the fresh-sweep generators under docs/research/ (T-3795, T-3930): the 15-digit literal
and the one-sheet probe case they write."""

import json


def lit(v):
    """`v` as a 15-significant-digit worksheet literal."""
    return f"{v:.15g}".replace("e+", "E+").replace("e-", "E-")


def write_case(path, description, task, cells):
    """Write a one-sheet ("S") M9 engineering probe case with `cells` and no expected values."""
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "engineering", "probe", task],
        "sheets": [{"name": "S", "cells": cells}],
        "expected": {},
    }
    with open(path, "w") as f:
        json.dump(case, f, indent=1)
        f.write("\n")
    print(len(cells))


def write_checked_case(path, description, tags, cells):
    """Write a one-sheet ("Calc") probe case that checks every cell in row-major order, with no expected
    values (T-4290, T-4291)."""
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": tags,
        "expected": {},
        "check": [f"Calc!{r}" for r in sorted(cells, key=lambda r: (int(r[1:]), r[0]))],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    path.write_text(json.dumps(case, indent=1) + "\n")
