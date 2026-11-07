"""Replays T-4844's counts and builds T-4888's pole probe (financial.md "IRR's second stage ignores the guess and reads
only NPV's values", "IRR's secant runs on past the pole").

Runs the engine's IRR secant in d exactly (worksheet NPV's running product with Excel's twice-rounded operators,
tools' `fitlab` exact arithmetic) without its 50-step cap, over the three t4844 probe cases, and prints:
- the guess ladder's split: guess cells equal to the default call against the cells where the replica fails or
  converges, the replica's bits where it converges, and the fixed points;
- each padded annuity's predicted underflow threshold, from the smallest 1 + r of the replica's run from 0.1;
- each annuity's |NPV(X)| and the 2^k at which it crosses the 1e-7 stop.

    cd tools && uv run python -I ../docs/research/irr-2026-11-07/replay.py          # the T-4844 counts
    cd tools && uv run python -I ../docs/research/irr-2026-11-07/replay.py pole     # T-4888's case: write or check

`pole` draws short series (3 to 6 flows, 6 significant digits, seed "T-4838/pole") whose secant from 0.1 crosses the
pole (an iterate at d >= 1), and keeps them by how two models answer: the secant run on past the pole (`secant(...,
run_on=True)`) and the engine's fallback before T-4888, Newton on P(v) = sum c_i v^i from v = 0 (plain doubles,
50 steps). It writes probes/t4838-irr-pole-runon and its predictions file, which were committed before the fill.

The secant here is exact (Excel's twice-rounded operators); docs/research/irr-2026-11-07/feasibility.py's is in plain
doubles for speed, so the two are kept apart.
"""

from __future__ import annotations

import json
import math
import random
import sys
from collections import Counter, defaultdict
from pathlib import Path

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.kernels.irr_secant_d import _npv_v_running
from rustsheet_tools.fitlab.kernels.solvers import NoResultError

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases"
GUESSES = 37  # -0.9..0.9 by 0.05
BLOCK = GUESSES + 2


def load(name: str) -> tuple[dict[str, str], dict[str, dict[str, object]]]:
    cells = json.loads((CASES / f"{name}.case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["P"]
    return cells, oracle


def flows(formula: str) -> list[float]:
    return [float(x) for x in formula[formula.index("{") + 1 : formula.index("}")].split(",")]


def secant(
    values: list[float], guess: float, cap: int = 400, *, run_on: bool = False
) -> tuple[float | None, list[float], str]:
    """The run's result (None when it fails), its iterates in d and how it ended: converged, flat, pole (an iterate at
    d >= 1 without `run_on`; with it the run goes on there), npv (out of range) or cap. A run-on result past the pole
    (v <= 0) is no result."""

    def npv(d: float) -> float:
        return _npv_v_running(values, 1.0 - d)

    path: list[float] = []
    try:
        d0 = X87.div(guess, 1 + guess)
        f0 = npv(d0)
        d1 = d0 - (1e-3 if f0 > 0 else -1e-3)
        f1 = npv(d1)
        path = [d0, d1]
        for _ in range(cap):
            if f1 == f0:
                return None, path, "flat"
            d2 = d1 - X87.div(f1 * (d1 - d0), f1 - f0)
            if d2 == 1 or (d2 > 1 and not run_on):
                return None, path, "pole"
            f2 = npv(d2)
            path.append(d2)
            if abs(d2 - d1) < 1e-7 and abs(f2) < 1e-7:
                return (X87.div(1.0, 1.0 - d2) - 1 if d2 < 1 else None), path, "converged"
            d0, f0, d1, f1 = d1, f1, d2, f2
    except NoResultError:
        return None, path, "npv"
    return None, path, "cap"


def newton_from_zero(values: list[float], cap: int = 50) -> float | None:
    """The engine's fallback before T-4888: Newton on P(v) by Horner from v = 0, plain doubles, stop at a step under
    1e-7 max(|v|, 1); the rate, or None."""
    v = 0.0
    for _ in range(cap):
        p = dp = 0.0
        for c in reversed(values):
            dp = dp * v + p
            p = p * v + c
        if dp == 0:
            return None
        nv = v - p / dp
        if not math.isfinite(nv):
            return None
        if abs(nv - v) < 1e-7 * max(abs(nv), 1):
            return 1 / nv - 1 if nv > 0 else None
        v = nv
    return None


POLE_CASE = "t4838-irr-pole-runon"
POLE_QUOTA = {"both, digits differ": 70, "run-on only": 45, "Newton only": 30, "both, same digits": 15}


def run_on(values: list[float]) -> float | None:
    result, _, _ = secant(values, 0.1, cap=200, run_on=True)
    return result if result is not None and result > -1 else None


def pole_rows() -> list[tuple[list[float], float | None, float | None, str]]:
    rng = random.Random("T-4838/pole")
    kept: defaultdict[str, list[tuple[list[float], float | None, float | None]]] = defaultdict(list)
    while any(len(kept[k]) < q for k, q in POLE_QUOTA.items()):
        n = rng.choice([3, 3, 4, 4, 5, 6])
        first = float(f"{rng.choice([1, -1]) * 10 ** rng.uniform(-1, 3):.6g}")
        values = [first] + [
            float(f"{rng.choice([1, -1]) * 10 ** rng.uniform(-1, 2):.6g}") for _ in range(n - 1)
        ]
        if all(c > 0 for c in values) or all(c < 0 for c in values):
            continue
        try:
            ending = secant(values, 0.1, cap=200)[2]
        except (OverflowError, ZeroDivisionError):
            continue
        if ending != "pole":
            continue
        ours, theirs = run_on(values), newton_from_zero(values)
        if ours is not None and theirs is not None:
            kind = "both, same digits" if ours == theirs else "both, digits differ"
        elif ours is not None:
            kind = "run-on only"
        elif theirs is not None:
            kind = "Newton only"
        else:
            continue
        if len(kept[kind]) < POLE_QUOTA[kind]:
            kept[kind].append((values, ours, theirs))
    return [(v, a, b, kind) for kind in POLE_QUOTA for v, a, b in kept[kind]]


def pole_case() -> int:
    rows = pole_rows()
    sheets = [
        {
            "name": "P",
            "cells": {
                f"A{i}": "=IRR({" + ",".join(map(repr, v)) + "})" for i, (v, _, _, _) in enumerate(rows, 1)
            },
        }
    ]
    path = CASES / f"{POLE_CASE}.case.json"
    if path.exists():
        same = json.loads(path.read_text())["sheets"] == sheets
        print(f"{POLE_CASE}: {'up to date' if same else 'differs from the generator'}")
        return 0 if same else 1
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4838: IRR on 160 fresh short series (3 to 6 flows, seed 'T-4838/pole') whose secant from 0.1 crosses the "
            "pole (an iterate at d >= 1). Stratified by whether the run-on secant (continued past d = 1) and the "
            "engine's Newton fallback from v = 0 return, and whether their digits agree: A1:A70 both return, digits "
            "differ; A71:A115 run-on only; A116:A145 Newton only; A146:A160 both, same digits. Predictions in "
            f"{POLE_CASE}.predictions.tsv. docs/research/irr-2026-11-07/replay.py."
        ),
        "tags": ["M9", "probe", "financial", "irr"],
        "sheets": sheets,
        "expected": {},
    }
    path.write_text(json.dumps(case, indent=2) + "\n")
    with (CASES / f"{POLE_CASE}.predictions.tsv").open("w") as out:
        out.write("cell\tclass\trun-on secant\tnewton from v=0 (engine fallback model)\n")
        for i, (_, a, b, kind) in enumerate(rows, 1):
            out.write(f"A{i}\t{kind}\t{a!r}\t{b!r}\n")
    print(f"wrote {path.name} and its predictions")
    return 0


def main() -> None:
    cells, oracle = load("t4844-irr-guess-ladder")
    split: Counter[tuple[str, str]] = Counter()
    fixed = 0
    for block in range(len(cells) // BLOCK):
        top = 1 + BLOCK * block
        values = flows(cells[f"A{top}"])
        default = oracle[f"A{top}"]["v"]
        fixed += oracle[f"A{top + 1}"]["v"] == default
        for k in range(GUESSES):
            guess = round(-0.9 + 0.05 * k, 2)
            excel = oracle[f"A{top + 2 + k}"]["v"]
            result, _, _ = secant(values, guess)
            if result is None:
                split["replica fails", "Excel = default" if excel == default else "Excel other"] += 1
            else:
                split["replica converges", "Excel = replica" if excel == result else "Excel other"] += 1
    print(
        "guess ladder:", dict(split), f"fixed points equal to the default: {fixed} of {len(cells) // BLOCK}"
    )
    cells, oracle = load("t4844-irr-stage-invariance")
    for block in range(4):
        top = 1 + 15 * block
        values = flows(cells[f"A{top}"])
        x = oracle[f"A{top}"]["v"]
        residual = _npv_v_running(values, 1.0 - X87.div(x, 1 + x))
        _, path, _ = secant(values, 0.1)
        smallest = min(1 / (1 - d) for d in path)
        threshold = 1022 * math.log(2) / math.log(1 / smallest)
        print(
            f"annuity {block}: |NPV(X)| {abs(residual):.3g}, crosses 1e-7 at 2^{math.log2(1e-7 / abs(residual)):.2f};"
            f" smallest 1 + r of the run from 0.1 {smallest:.6f}, underflow threshold N {threshold:.1f}"
        )


if __name__ == "__main__":
    if sys.argv[1:] == ["pole"]:
        sys.exit(pole_case())
    main()
