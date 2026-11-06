"""T-2555 attempt 2: write excel_cdf_probe.case.json, a probe of Excel's BETA.DIST at AS 109's iterates.

Run with mpmath: cd tools && uv run python ../docs/research/t2555/gen_iterates_probe.py. For each of the 8 tiny-quantile
cells of probes-numeric/t1538-f-inv-huge-df it writes BETA.DIST(x, a, b, TRUE) at the AS 109 start x0 and at the
iterates x1 to x3 of the replay (as109_replay.py, exact ln B, log1p), so that the search can be replayed on Excel's
own CDF (iterates.py). Row r: cell r of `as109_replay.cells`, columns A to D the iterates 0 to 3.
"""

import importlib.util
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "as109_replay", os.path.join(HERE, "as109_replay.py")
)
assert _spec and _spec.loader
replay = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(replay)

# Kept here, not in probes-numeric: its cells are evidence for T-2555's BETA.INV cells, not cells to score (the
# engine's BETA.DIST is a few ulps off on 29 of them), and decision 0267 freezes new probes-numeric cells. The fill
# was made with `rustsheet-oracle fill` on a copy in eval/suites/probes-numeric (excel-graph, 2026-11-05); its
# sidecar is excel_cdf_probe.oracle.json.
OUT = os.path.join(HERE, "excel_cdf_probe.case.json")


def main() -> None:
    cells: dict[str, str] = {}
    for r, (_, p, a, b, _) in enumerate(replay.cells, 1):
        _, xs = replay.as109(p, a, b, 1e-300, True, l1p=True)
        for col, x in zip("ABCD", xs[:4]):
            cells[f"{col}{r}"] = f"=BETA.DIST({x!r},{a!r},{b:.0E},TRUE)".replace(
                "E+0", "E+"
            )
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-2555: Excel's BETA.DIST at AS 109's start and first iterates for the 8 tiny-quantile "
        "BETA.INV cells of t1538-f-inv-huge-df (rows 1-8 in that order; columns A-D iterates 0-3), to replay the "
        "search on Excel's own CDF.",
        "tags": ["M9", "probe", "statistics", "T-2555"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    with open(OUT, "w") as fh:
        json.dump(case, fh, indent=1)
        fh.write("\n")
    print(len(cells), "cells")


if __name__ == "__main__":
    main()
