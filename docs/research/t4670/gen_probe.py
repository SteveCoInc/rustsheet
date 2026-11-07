"""Generate the T-4670 probe case (CONVERT distance/area near the overflow path).

Run from the repository root: python3 docs/research/t4670/gen_probe.py
"""

import json
import random
from pathlib import Path

CASES = Path("eval/suites/probes-numeric/cases")
# Seeded 15-digit mantissas; exponents run from the first value*1E10 overflow (~1.8E298) to the top.
PAIRS = [("m", "m"), ("m", "ft"), ("mi", "km"), ("in", "cm"), ("m2", "m2"), ("m2", "ft2")]
EXPS = [297, 298, 299, 300, 302, 304, 306, 307]


def main() -> None:
    rng = random.Random(4670)
    cells: dict[str, str] = {}
    n = 1
    for a, b in PAIRS:
        for e in EXPS:
            mant = f"{rng.uniform(1.0, 1.7):.14f}".rstrip("0")
            cells[f"A{n}"] = f'=CONVERT({mant}E{e},"{a}","{b}")'
            n += 1
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4670: CONVERT of seeded non-round values (E297 to E307) for m to m, m to ft, "
            "mi to km, in to cm, m2 to m2 and m2 to ft2, where value*F_from (angstrom factors, "
            "T-4630) overflows or nearly does. The pre-prefix conversion overflowing gives #NUM!."
        ),
        "tags": ["M9", "probe", "engineering", "T-4670"],
        "sheets": [{"name": "Sheet1", "cells": cells}],
    }
    (CASES / "t4670-convert-distance-overflow.case.json").write_text(
        json.dumps(case, indent=1) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
