"""IRR research plan (2026-11-07): feasibility checks behind docs/research/irr-2026-11-07/plan.md.

Scores the outcome TYPE (a number, `#NUM!` or `#DIV/0!`) of candidate IRR solvers against Excel on every literal
IRR cell of the IRR probe cases, beside the engine on main. Types, not bits: NPV is the running product at
x = 1/v in plain doubles (the engine rounds its operators twice), so the exact counts are only indicative.

Candidates, all on the fitted secant in d = 1 - v (financial.md "IRR is a secant in d"), retry from 0.1:
- `base`: the secant with the replica's three dead ends (flat secant, iterate at d >= 1, step cap) as failures and
  no fallback.
- `vb`: the failure handlers of Microsoft.VisualBasic's Financial.IRR (dotnet/runtime, Financial.vb): on a flat
  secant move the older iterate one step away and retry the step once; on an iterate past the pole, halve the
  distance to it ("r-half": 1 + r halves, the VB rule (r1 - 1) * 0.5 translated; "d-half": (d1 + 1) / 2).
- `rate`: RATE's fitted driver (financial.md "RATE holds on a flat secant and runs on below -1", "RATE is a secant
  on the FV residual"): hold at d1 on a flat secant and return it when |f1| < 1e-4 and 1 + r > 1e-4; run on past
  d = 1; at the cap return the last iterate under the same 1e-4 tests.

It also lists the cells no candidate gets, by how the replica's run from 0.1 ends. Needs the tools venv and the
release harness binary (the engine column):

    cd tools && uv run python -I ../docs/research/irr-2026-11-07/feasibility.py
"""

from __future__ import annotations

import collections
import math
from collections.abc import Callable

from rustsheet_tools.fitlab.fit import engine_candidate
from rustsheet_tools.fitlab.kernels.solvers import floats
from rustsheet_tools.fitlab.score import ExcelError
from rustsheet_tools.fitlab.sweep import Cell, load_sweep

CASES = [
    *(
        f"probes-numeric/{name}"
        for name in (
            "m9-fin-irr-rate",
            "t2927-irr-fallback-sweep",
            "t3347-irr-conventional-holdout",
            "t3347-irr-conventional-sweep",
            "t3347-irr-r4-fresh",
            "t3408-irr-r4-fresh",
            "t3411-irr-r4-fresh",
            "t3411-irr-r4-fresh-2",
            "t3411-irr-r4-fresh-3",
            "t3411-irr-r4-fresh-4",
            "t3411-irr-residual-scale",
            "t3478-irr-r4-wide",
            "t4213-irr-r4-fresh",
        )
    ),
    *(
        f"probes/{name}"
        for name in (
            "fin-irr-error-array",
            "irr-fallback-roots",
            "irr-guess-edges",
            "irr-guess-edges-2",
            "solver-iteration-traces",
            "t2496-fin-subexpressions",
            "t2565-irr-guess-sweeps",
            "t2657-pv-irr-div0",
            "t3347-irr-conventional-num",
            "t3406-irr-num-boundary",
            "t3410-irr-zero-root-imcos",
            "t4027-irr-annuity-scale-guess",
            "xirr-irr-empty-guess",
        )
    ),
]
STEP = 1e-3  # the fitted start offset in d
FAR_ROOT = 1e7  # IRR_FAR_ROOT_BOUND


class FailedError(Exception):
    """The run gives no root: the call retries from 0.1, then is #NUM!."""


class DivZeroError(Exception):
    """NPV cannot be evaluated (v = 0, a discount or sum out of range): the call is #DIV/0!."""


def npv_at(flows: list[float], d: float) -> float:
    """NPV at v = 1 - d as worksheet NPV's running product of x = 1/v, in plain doubles."""
    v = 1.0 - d
    if v == 0:
        raise DivZeroError
    x, total, discount = 1.0 / v, 0.0, 1.0
    for i, c in enumerate(flows):
        if i:
            discount *= x
            if discount == 0 or not math.isfinite(discount):
                raise DivZeroError
        total += c / discount
    if not math.isfinite(total):
        raise DivZeroError
    return total


def rate_of(d: float) -> float | None:
    """1/(1 - d) - 1, or None past the far-root bound."""
    x = 1.0 / (1.0 - d)
    return x - 1 if x < FAR_ROOT else None


def secant(
    flows: list[float],
    guess: float,
    *,
    cap: int,
    nudge: bool = False,
    clamp: str | None = None,
    hold: bool = False,
    run_on: bool = False,
) -> float:
    """One secant run from `guess`; raises FailedError or DivZeroError. The options are the handlers of the docstring."""
    peak = max(abs(c) for c in flows)
    d0 = guess / (1 + guess)
    f0 = npv_at(flows, d0)
    d1 = d0 - (STEP if f0 > 0 else -STEP)
    f1 = npv_at(flows, d1)

    def held(d: float, f: float) -> float:
        r = rate_of(d) if d < 1 else None
        if r is None or not (1 + r > 1e-4 and abs(f) < 1e-4):
            raise FailedError
        return r

    for step in range(cap):
        if f1 == f0:
            if hold and f1 != 0:
                return held(d1, f1)
            if not nudge:
                raise FailedError
            d0 = d0 - STEP if d1 > d0 else d0 + STEP
            f0 = npv_at(flows, d0)
            if f1 == f0:
                raise FailedError
        d2 = d1 - f1 * (d1 - d0) / (f1 - f0)
        if not math.isfinite(d2):
            raise FailedError
        if d2 >= 1 and clamp == "r-half":
            d2 = 2 * d1 - 1
        elif d2 >= 1 and clamp == "d-half":
            d2 = (d1 + 1) * 0.5
        elif d2 == 1:
            raise DivZeroError
        elif d2 > 1 and not run_on:
            raise FailedError
        f2 = npv_at(flows, d2)
        stalled = d2 == d1 and abs(f2) < 1e-7 * peak and step < 50
        if abs(d2 - d1) < 1e-7 and (abs(f2) < 1e-7 or stalled):
            r = rate_of(d2) if d2 < 1 else None
            if r is None:
                raise FailedError
            return r
        d0, f0, d1, f1 = d1, f1, d2, f2
    if hold and abs(d1 - d0) < 1e-4:
        return held(d1, f1)
    raise FailedError


def irr(flows: list[float], guess: float, **options: object) -> float | str:
    """IRR with leading zeros dropped, the no-sign-change #NUM!, and the retry from 0.1."""
    flows = flows[next((i for i, c in enumerate(flows) if c != 0), 0) :]
    if all(c >= 0 for c in flows) or all(c <= 0 for c in flows):
        return "#NUM!"
    for g in [guess, 0.1] if guess != 0.1 else [0.1]:
        try:
            return secant(flows, g, **options)  # type: ignore[arg-type]
        except DivZeroError:
            return "#DIV/0!"
        except (FailedError, ZeroDivisionError, OverflowError):
            continue
    return "#NUM!"


def ending(flows: list[float], guess: float, cap: int = 200) -> str:
    """How the base replica's run from `guess` ends: converged, flat, flat after a zero step, d >= 1, cap or overflow."""
    flows = flows[next((i for i, c in enumerate(flows) if c != 0), 0) :]
    try:
        d0 = guess / (1 + guess)
        f0 = npv_at(flows, d0)
        d1 = d0 - (STEP if f0 > 0 else -STEP)
        f1 = npv_at(flows, d1)
        for _ in range(cap):
            if f1 == f0:
                return "flat after a zero step" if d1 == d0 else "flat"
            d2 = d1 - f1 * (d1 - d0) / (f1 - f0)
            if not math.isfinite(d2) or d2 >= 1:
                return "iterate at d >= 1"
            f2 = npv_at(flows, d2)
            if abs(d2 - d1) < 1e-7 and abs(f2) < 1e-7:
                return "converged"
            d0, f0, d1, f1 = d1, f1, d2, f2
    except DivZeroError:
        return "NPV out of range"
    return "step cap"


def kind(value: float | str) -> str:
    return value if isinstance(value, str) else "number"


def main() -> None:
    sweep = load_sweep(CASES, r"^=IRR\((.*)\)$", None, True, [])
    cells: list[tuple[Cell, list[float], float]] = []
    for cell in sweep.cells:
        try:
            flows = floats(cell.args[0])
        except TypeError:
            continue
        if all(isinstance(a, float) for a in cell.args[1:]):
            cells.append((cell, flows, float(cell.args[1]) if len(cell.args) > 1 else 0.1))
    engine = engine_candidate(sorted({c.case for c, _, _ in cells}))

    def engine_value(cell: Cell) -> float | str:
        try:
            return engine.evaluate(cell)  # type: ignore[return-value]
        except ExcelError as e:
            return e.code

    want = {id(c): c.error or "number" for c, _, _ in cells}
    engine_ok = {id(c): kind(engine_value(c)) == want[id(c)] for c, _, _ in cells}
    print(
        f"{len(cells)} literal IRR cells; the engine (main) gives Excel's type on {sum(engine_ok.values())}"
    )
    candidates: dict[str, Callable[[list[float], float], float | str]] = {
        "base, cap 50": lambda f, g: irr(f, g, cap=50),
        "base, cap 200": lambda f, g: irr(f, g, cap=200),
        "vb: r-half clamp, cap 200": lambda f, g: irr(f, g, cap=200, clamp="r-half"),
        "vb: d-half clamp, cap 200": lambda f, g: irr(f, g, cap=200, clamp="d-half"),
        "vb: flat nudge + r-half clamp, cap 200": lambda f, g: irr(f, g, cap=200, nudge=True, clamp="r-half"),
        "rate: hold on flat, cap 50": lambda f, g: irr(f, g, cap=50, hold=True),
        "rate: hold on flat + run on past d = 1, cap 50": lambda f, g: irr(
            f, g, cap=50, hold=True, run_on=True
        ),
        "rate: hold on flat + run on past d = 1, cap 200": lambda f, g: irr(
            f, g, cap=200, hold=True, run_on=True
        ),
    }
    right: dict[str, set[int]] = {}
    for name, fn in candidates.items():
        right[name] = {id(c) for c, f, g in cells if kind(fn(f, g)) == want[id(c)]}
        gained = sum(1 for i in right[name] if not engine_ok[i])
        lost = sum(1 for i, ok in engine_ok.items() if ok and i not in right[name])
        print(f"{len(right[name]):5d}  (+{gained} the engine misses, -{lost} it gets)  {name}")
    anyone = set().union(*right.values()) | {i for i, ok in engine_ok.items() if ok}
    residual = collections.Counter(
        (want[id(c)], ending(f, 0.1), "21+ flows" if len(f) > 20 else "2-20 flows")
        for c, f, g in cells
        if id(c) not in anyone
    )
    print(
        f"\n{sum(residual.values())} cells no candidate (nor the engine) types right, by Excel / replica end / length:"
    )
    for key, n in residual.most_common():
        print(f"  {n:4d}  {key}")


if __name__ == "__main__":
    main()
