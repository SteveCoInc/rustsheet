"""T-4437 second fresh sweep, written after the second freeze (freeze2.md): L140 bup+bgrat BETA.DIST
is 0 at n >= 3 bup terms when bup's sum W < 2^-970. Untouched shapes, walked in ln of the true tail
by 0.05 nats from ln 2^-970 + 0.525 (row 1) to ln 2^-970 - 0.525 (row 22): the rule predicts a
number through row 11 and 0 from row 12. Columns I and J are n = 2 families, evidence for T-4564
only (out of the rule's domain).
Run: cd tools && uv run python ../docs/research/t4437/gen_fresh2.py
"""

import json
import pathlib

import mpmath as mp

mp.mp.dps = 60
EDGE = -970 * mp.log(2)
FAMILIES = [("A", 1_500_000, 3.25), ("B", 6_000_000, 3.75), ("C", 2_500_000, 4.25), ("D", 7_000_000, 5.0),
            ("E", 1_200_000, 6.5), ("F", 9_000_000, 7.75), ("G", 3_500_000, 7.25), ("H", 4_500_000, 3.5),
            ("I", 3_000_000, 2.5), ("J", 7_000_000, 2.75)]
ROWS = 22


def ln_tail(x: mp.mpf, a: int, b: float) -> mp.mpf:
    return mp.log(mp.betainc(a, mp.mpf(b), 0, x, regularized=True))


def main() -> None:
    cells: dict[str, str] = {}
    lns: dict[str, str] = {}
    for col, a, b in FAMILIES:
        for row in range(1, ROWS + 1):
            target = EDGE + mp.mpf("0.525") - mp.mpf("0.05") * (row - 1)
            x = mp.findroot(lambda v: ln_tail(v, a, b) - target, (1 - mp.mpf(800) / a, 1 - mp.mpf(500) / a),
                            solver="illinois")
            xs = mp.nstr(x, 15, strip_zeros=True, min_fixed=-30, max_fixed=30)
            cells[f"{col}{row}"] = f"=BETA.DIST({xs},{a},{b:g},TRUE)"
            lns[f"{col}{row}"] = mp.nstr(ln_tail(mp.mpf(xs), a, b) - EDGE, 6)
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4437 second fresh sweep after the second freeze (docs/research/t4437/freeze2.md): BETA.DIST(x,a,b,TRUE) "
            "on L140 bup+bgrat at untouched shapes A (1.5E6, 3.25), B (6E6, 3.75), C (2.5E6, 4.25), D (7E6, 5), "
            "E (1.2E6, 6.5), F (9E6, 7.75), G (3.5E6, 7.25), H (4.5E6, 3.5), walked in ln of the true tail by -0.05 "
            "from ln 2^-970 + 0.525 (row 1) to ln 2^-970 - 0.525 (row 22); the rule predicts a number through row 11 "
            "and 0 from row 12. I (3E6, 2.5) and J (7E6, 2.75) are n = 2, evidence for T-4564. "
            "docs/research/t4437/gen_fresh2.py writes it."
        ),
        "tags": ["M9", "probe", "statistical", "T-4437"],
        "sheets": [{"name": "Sheet1", "cells": cells}],
        "expected": {},
    }
    root = pathlib.Path(__file__).resolve().parents[3]
    (root / "eval/suites/probes/cases/t4437-beta-zero-fresh2.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (pathlib.Path(__file__).parent / "fresh2_ln_minus_edge.json").write_text(
        json.dumps(lns, indent=1, sort_keys=True) + "\n"
    )


if __name__ == "__main__":
    main()
