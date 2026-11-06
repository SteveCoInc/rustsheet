"""T-4437: bracket where Excel's exact 0 starts for BETA.DIST at n = 3 bup terms and for T.DIST.RT.

Writes eval/suites/probes/cases/t4437-beta-t-zero-bracket.case.json. BETA.DIST families are walked
in ln of bup's prefactor ret = x0^a0 y0^c / (c B(c, a0)) (c = b - n, x0 = x, y0 = 1 - x; bup.rs
`l140_bup_prefactor_underflows`), T.DIST.RT families in ln of the true tail, by 0.5 nats.
Run: cd tools && uv run python ../docs/research/t4437/gen_bracket.py
"""

import json
import pathlib

import mpmath as mp

mp.mp.dps = 60


def ln_ret(x: mp.mpf, a: int, b: float) -> mp.mpf:
    n = int(b)
    c = mp.mpf(b) - n
    if c == 0:
        c = mp.mpf(1)
    return a * mp.log(x) + c * mp.log(1 - x) - mp.log(c * mp.beta(c, a))


def ln_t_tail(t: mp.mpf, df: int) -> mp.mpf:
    z = mp.mpf(df) / (df + t * t)
    return mp.log(mp.betainc(mp.mpf(df) / 2, mp.mpf(1) / 2, 0, z, regularized=True) / 2)


def solve(f, target: float, lo: float, hi: float) -> mp.mpf:
    return mp.findroot(lambda v: f(v) - target, (mp.mpf(lo), mp.mpf(hi)), solver="illinois")


def fmt(v: mp.mpf) -> str:
    return mp.nstr(v, 15, strip_zeros=True, min_fixed=-30, max_fixed=30)


BETA = [("A", 5_000_000, 3.5, 0.9997, 0.99995), ("B", 5_000_000, 4.0, 0.9997, 0.99995),
        ("C", 1_000_000, 3.5, 0.998, 0.9998), ("D", 5_000_000, 2.5, 0.9997, 0.99995),
        ("E", 5_000_000, 5.5, 0.9997, 0.99995), ("F", 5_000_000, 3.0, 0.9997, 0.99995)]
TDIST = [("G", 10_000_000), ("H", 100_000), ("I", 1_000_000)]
STEPS = [-680 - 0.5 * i for i in range(37)]  # -680 to -698 by 0.5


def main() -> None:
    cells: dict[str, str] = {}
    lns: dict[str, str] = {}
    for col, a, b, lo, hi in BETA:
        for row, target in enumerate(STEPS, 1):
            x = solve(lambda v: ln_ret(v, a, b), target, lo, hi)
            cells[f"{col}{row}"] = f"=BETA.DIST({fmt(x)},{a},{b:g},TRUE)"
            lns[f"{col}{row}"] = mp.nstr(ln_ret(mp.mpf(fmt(x)), a, b), 8)
    for col, df in TDIST:
        for row, target in enumerate(STEPS, 1):
            t = solve(lambda v: ln_t_tail(v, df), target, 30, 45)
            cells[f"{col}{row}"] = f"=T.DIST.RT({fmt(t)},{df})"
            lns[f"{col}{row}"] = mp.nstr(ln_t_tail(mp.mpf(fmt(t)), df), 8)
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4437: bracket Excel's exact 0 to 0.5 nats. Rows 1-37 step ln by -0.5 from -680 to -698. "
            "A:F BETA.DIST(x,a,b,TRUE) on L140 bup+bgrat walked in ln of bup's prefactor "
            "ret = x^a (1-x)^c / (c B(c, a)), c = b - n: A (5E6, 3.5), B (5E6, 4), C (1E6, 3.5), D (5E6, 2.5), "
            "E (5E6, 5.5), F (5E6, 3). G:I T.DIST.RT walked in ln of the true tail at df 1E7, 1E5 and 1E6. "
            "docs/research/t4437/gen_bracket.py writes it; ln_targets.json holds each cell's ln."
        ),
        "tags": ["M9", "probe", "statistical", "T-4437"],
        "sheets": [{"name": "Sheet1", "cells": cells}],
        "expected": {},
    }
    root = pathlib.Path(__file__).resolve().parents[3]
    out = root / "eval/suites/probes/cases/t4437-beta-t-zero-bracket.case.json"
    out.write_text(json.dumps(case, indent=1) + "\n")
    (pathlib.Path(__file__).parent / "ln_targets.json").write_text(json.dumps(lns, indent=1, sort_keys=True) + "\n")
    print(f"wrote {len(cells)} cells to {out}")


if __name__ == "__main__":
    main()
