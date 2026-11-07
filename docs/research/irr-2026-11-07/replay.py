"""Replays T-4844's counts (financial.md "IRR's second stage ignores the guess and reads only NPV's values").

Runs the engine's IRR secant in d exactly (worksheet NPV's running product with Excel's twice-rounded operators,
tools' `fitlab` exact arithmetic) without its 50-step cap, over the three t4844 probe cases, and prints:
- the guess ladder's split: guess cells equal to the default call against the cells where the replica fails or
  converges, the replica's bits where it converges, and the fixed points;
- each padded annuity's predicted underflow threshold, from the smallest 1 + r of the replica's run from 0.1;
- each annuity's |NPV(X)| and the 2^k at which it crosses the 1e-7 stop.

    cd tools && uv run python -I ../docs/research/irr-2026-11-07/replay.py
"""

from __future__ import annotations

import json
import math
from collections import Counter
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


def secant(values: list[float], guess: float, cap: int = 400) -> tuple[float | None, list[float]]:
    """The run's result (None when it fails: flat, d >= 1, NPV out of range, cap) and its iterates in d."""
    npv = lambda d: _npv_v_running(values, 1.0 - d)  # noqa: E731 - the one evaluation the loop repeats
    try:
        d0 = X87.div(guess, 1 + guess)
        f0 = npv(d0)
        d1 = d0 - (1e-3 if f0 > 0 else -1e-3)
        f1 = npv(d1)
        path = [d0, d1]
        for _ in range(cap):
            if f1 == f0:
                break
            d2 = d1 - X87.div(f1 * (d1 - d0), f1 - f0)
            if d2 >= 1:
                break
            f2 = npv(d2)
            path.append(d2)
            if abs(d2 - d1) < 1e-7 and abs(f2) < 1e-7:
                return X87.div(1.0, 1.0 - d2) - 1, path
            d0, f0, d1, f1 = d1, f1, d2, f2
    except NoResultError:
        pass
    return None, path


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
            result, _ = secant(values, guess)
            if result is None:
                split["replica fails", "Excel = default" if excel == default else "Excel other"] += 1
            else:
                split["replica converges", "Excel = replica" if excel == result else "Excel other"] += 1
    print("guess ladder:", dict(split), f"fixed points equal to the default: {fixed} of {len(cells) // BLOCK}")
    cells, oracle = load("t4844-irr-stage-invariance")
    for block in range(4):
        top = 1 + 15 * block
        values = flows(cells[f"A{top}"])
        x = oracle[f"A{top}"]["v"]
        residual = _npv_v_running(values, 1.0 - X87.div(x, 1 + x))
        _, path = secant(values, 0.1)
        smallest = min(1 / (1 - d) for d in path)
        threshold = 1022 * math.log(2) / math.log(1 / smallest)
        print(
            f"annuity {block}: |NPV(X)| {abs(residual):.3g}, crosses 1e-7 at 2^{math.log2(1e-7 / abs(residual)):.2f};"
            f" smallest 1 + r of the run from 0.1 {smallest:.6f}, underflow threshold N {threshold:.1f}"
        )


if __name__ == "__main__":
    main()
