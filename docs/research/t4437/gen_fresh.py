"""T-4437 fresh sweep, written after the rule froze (L140 bup+bgrat BETA.DIST is 0 when its tail
w < 2^-970 = DBL_MIN / 2^-52). Untouched shapes, walked in ln of the true tail by 0.25 nats around
ln 2^-970 = -672.36 (rows 1-17: +2.125 to -1.875 from it; the rule predicts a number through
row 9, at +0.125, and 0 from row 10, at -0.125).
Run: cd tools && uv run python ../docs/research/t4437/gen_fresh.py
"""

import json
import pathlib

import mpmath as mp

mp.mp.dps = 60
EDGE = -970 * mp.log(2)
FAMILIES = [("A", 3_000_000, 3.5), ("B", 5_000_000, 6.5), ("C", 8_000_000, 4.5), ("D", 2_000_000, 2.5),
            ("E", 1_000_000, 5.0), ("F", 4_000_000, 7.5)]


def ln_tail(x: mp.mpf, a: int, b: float) -> mp.mpf:
    return mp.log(mp.betainc(a, mp.mpf(b), 0, x, regularized=True))


def sweep(
    families: list[tuple[str, int, float]], rows: int, start: object, step: object
) -> tuple[dict[str, str], dict[str, str]]:
    """BETA.DIST cells walked in ln of the true tail by -step from ln 2^-970 + start (row 1), and each
    cell's ln tail minus ln 2^-970 at the double x the cell holds. gen_fresh2.py shares it."""
    cells: dict[str, str] = {}
    lns: dict[str, str] = {}
    for col, a, b in families:
        for row in range(1, rows + 1):
            target = EDGE + start - step * (row - 1)
            x = mp.findroot(lambda v: ln_tail(v, a, b) - target, (1 - mp.mpf(800) / a, 1 - mp.mpf(500) / a),
                            solver="illinois")
            xs = mp.nstr(x, 15, strip_zeros=True, min_fixed=-30, max_fixed=30)
            cells[f"{col}{row}"] = f"=BETA.DIST({xs},{a},{b:g},TRUE)"
            lns[f"{col}{row}"] = mp.nstr(ln_tail(mp.mpf(xs), a, b) - EDGE, 6)
    return cells, lns


def write(case_name: str, description: str, cells: dict[str, str], lns: dict[str, str], ln_file: str) -> None:
    """Writes the probe case and its ln-minus-edge record."""
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe", "statistical", "T-4437"],
        "sheets": [{"name": "Sheet1", "cells": cells}],
        "expected": {},
    }
    root = pathlib.Path(__file__).resolve().parents[3]
    (root / f"eval/suites/probes/cases/{case_name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (pathlib.Path(__file__).parent / ln_file).write_text(json.dumps(lns, indent=1, sort_keys=True) + "\n")


def main() -> None:
    cells, lns = sweep(FAMILIES, 17, 2.125, 0.25)
    write(
        "t4437-beta-zero-fresh",
        "T-4437 fresh sweep after the rule froze: BETA.DIST(x,a,b,TRUE) on L140 bup+bgrat at untouched shapes "
        "A (3E6, 3.5), B (5E6, 6.5), C (8E6, 4.5), D (2E6, 2.5), E (1E6, 5), F (4E6, 7.5), walked in ln of the "
        "true tail by -0.25 from ln 2^-970 + 2.125 (row 1) to ln 2^-970 - 1.875 (row 17); the rule predicts a "
        "number through row 9 and 0 from row 10. docs/research/t4437/gen_fresh.py writes it.",
        cells,
        lns,
        "fresh_ln_minus_edge.json",
    )

if __name__ == "__main__":
    main()
