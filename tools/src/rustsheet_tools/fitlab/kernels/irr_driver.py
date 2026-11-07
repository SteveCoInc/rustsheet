"""The `irr-type` fit: IRR's outcome type (a number, `#NUM!` or `#DIV/0!`) and bits on every IRR probe case.

Phase 0 of docs/research/irr-2026-11-07/plan.md (T-4834). feasibility.py there scored the candidates' types in plain
doubles. This kernel runs them on the exact replica: NPV is worksheet NPV's running product with every operator
rounded twice (`irr_secant_d._npv_v_running`, T-3205). The fit scores error cells too (`Fit.errors`), and the
report's `typed` count is the type score. Every candidate exposes `trace` (the rates its runs walk through) and
`event` (how each run ended, and at which step), which the JSON lists per cell.

Candidates, all on the fitted secant in d = 1 - v with the retry from 0.1 (financial.md "IRR is a secant in d"):
- `base`: the replica's dead ends (flat secant, iterate at d >= 1, step cap) are failures; no fallback.
- `vb`: the failure handlers of Microsoft.VisualBasic's Financial.IRR (plan.md appendix A): on a flat secant move the
  older iterate one offset away and retry the step once; past the pole, halve the distance to it in r ("r-half") or
  in d ("d-half").
- `rate`: RATE's fitted driver (financial.md "RATE holds on a flat secant and runs on below -1"): hold at d1 on a flat
  secant and return it when |f1| < 1e-4 and 1 + r > 1e-4; run on past d = 1; at the cap return the last iterate
  under the same tests.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.kernels.irr_secant_d import _npv_v_running
from rustsheet_tools.fitlab.kernels.solvers import NoResultError, floats
from rustsheet_tools.fitlab.score import Candidate, ExcelError
from rustsheet_tools.fitlab.sweep import Arg, Cell

# The IRR probe cases of plan.md section 6 (feasibility.py's list).
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
HELD = 1e-4  # RATE's acceptance tests for a held or capped iterate

# How a run ends (the event tracer, plan.md Phase 0 item 2).
CONVERGED = "converged"
STALLED = "zero-step stop"
FLAT = "flat"
FLAT_ZERO_STEP = "flat after a zero step"
HELD_FLAT = "held on flat"
POLE = "iterate at d >= 1"
AT_POLE = "iterate at d = 1"
NOT_FINITE = "step not finite"
OVERFLOW = "NPV out of range"
CAP = "step cap"
HELD_CAP = "held at the cap"
FAR = "past the far-root bound"
REJECTED = "held iterate rejected"


@dataclass(frozen=True)
class Options:
    """One candidate's handlers (the module docstring)."""

    cap: int = 50
    nudge: bool = False
    clamp: str | None = None  # "r-half" or "d-half"
    hold: bool = False
    run_on: bool = False


@dataclass
class Run:
    """One secant run from one guess: its result (a rate, or None), the error it ends the call with (None: retry
    from 0.1), the event that ended it, the step it ended on, and the d iterates it walked through."""

    rate: float | None = None
    error: str | None = None
    event: str = ""
    step: int = 0
    iterates: list[float] = field(default_factory=list)


@dataclass
class Outcome:
    """A whole IRR call: the value (a rate or an error code) and its runs (the guess's, then the retry's)."""

    value: float | str
    runs: list[Run]

    @property
    def event(self) -> str:
        return " then ".join(f"{r.event}@{r.step}" for r in self.runs)

    @property
    def trace(self) -> list[float]:
        out: list[float] = []
        for r in self.runs:
            out.extend(rate for d in r.iterates if (rate := rate_of(d)) is not None)
        return out


class _DivZeroError(Exception):
    """NPV cannot be evaluated at this d: the call is #DIV/0!."""


def npv_at(flows: list[float], d: float) -> float:
    """NPV at v = 1 - d, worksheet NPV's running product with twice-rounded operators; #DIV/0! where it has no
    finite value."""
    v = 1.0 - d
    if v == 0:
        raise _DivZeroError
    try:
        total = _npv_v_running(flows, v)
    except (NoResultError, ZeroDivisionError, OverflowError) as e:
        raise _DivZeroError from e
    if not math.isfinite(total):
        raise _DivZeroError
    return total


def rate_of(d: float) -> float | None:
    """1/(1 - d) - 1, or None at or past the pole and past the far-root bound."""
    if d >= 1:
        return None
    x = X87.div(1.0, 1.0 - d)
    return x - 1 if x < FAR_ROOT else None


class _EndedError(Exception):
    """A run ended on `event`; with `error`, the call ends with that error instead of retrying."""

    def __init__(self, event: str, error: str | None = None) -> None:
        super().__init__(event)
        self.event = event
        self.error = error


def secant(flows: list[float], guess: float, opts: Options) -> Run:
    """One secant run from `guess` under `opts`."""
    run = Run()
    try:
        _secant(flows, guess, opts, run)
    except _EndedError as e:
        run.event, run.error = e.event, e.error
    except _DivZeroError:
        run.error, run.event = "#DIV/0!", OVERFLOW
    except (ZeroDivisionError, OverflowError):
        run.event = NOT_FINITE
    return run


def _accept(run: Run, d: float, event: str) -> None:
    rate = rate_of(d)
    run.event = event if rate is not None else FAR
    run.rate = rate


def _held(run: Run, d: float, f: float, event: str) -> None:
    rate = rate_of(d)
    if rate is None or not (1 + rate > HELD and abs(f) < HELD):
        raise _EndedError(REJECTED)
    run.event, run.rate = event, rate


def _next_iterate(d0: float, d1: float, f0: float, f1: float, opts: Options) -> float:
    """The secant step from (d0, f0), (d1, f1), clamped or refused at the pole as `opts` says."""
    d2 = d1 - X87.div(f1 * (d1 - d0), f1 - f0)
    if not math.isfinite(d2):
        raise _EndedError(NOT_FINITE)
    if d2 >= 1 and opts.clamp == "r-half":
        return 2 * d1 - 1
    if d2 >= 1 and opts.clamp == "d-half":
        return (d1 + 1) * 0.5
    if d2 == 1:
        raise _EndedError(AT_POLE, "#DIV/0!")
    if d2 > 1 and not opts.run_on:
        raise _EndedError(POLE)
    return d2


def _secant(flows: list[float], guess: float, opts: Options, run: Run) -> None:
    peak = max(abs(c) for c in flows)
    d0 = X87.div(guess, 1 + guess)
    f0 = npv_at(flows, d0)
    d1 = d0 - (STEP if f0 > 0 else -STEP)
    f1 = npv_at(flows, d1)
    run.iterates.extend([d0, d1])
    for step in range(opts.cap):
        run.step = step
        if f1 == f0:
            if opts.hold and f1 != 0:
                _held(run, d1, f1, HELD_FLAT)
                return
            if not opts.nudge:
                raise _EndedError(FLAT_ZERO_STEP if d1 == d0 else FLAT)
            d0 = d0 - STEP if d1 > d0 else d0 + STEP
            f0 = npv_at(flows, d0)
            if f1 == f0:
                raise _EndedError(FLAT)
        d2 = _next_iterate(d0, d1, f0, f1, opts)
        f2 = npv_at(flows, d2)
        run.iterates.append(d2)
        stalled = d2 == d1 and abs(f2) < 1e-7 * peak and step < 50
        if abs(d2 - d1) < 1e-7 and (abs(f2) < 1e-7 or stalled):
            _accept(run, d2, CONVERGED if abs(f2) < 1e-7 else STALLED)
            return
        d0, f0, d1, f1 = d1, f1, d2, f2
    run.step = opts.cap
    if not (opts.hold and abs(d1 - d0) < HELD):
        raise _EndedError(CAP)
    _held(run, d1, f1, HELD_CAP)


def irr(flows: list[float], guess: float, opts: Options) -> Outcome:
    """IRR with leading zeros dropped, the no-sign-change #NUM!, and the retry from 0.1."""
    flows = flows[next((i for i, c in enumerate(flows) if c != 0), 0) :]
    if all(c >= 0 for c in flows) or all(c <= 0 for c in flows):
        return Outcome("#NUM!", [Run(event="no sign change")])
    runs: list[Run] = []
    for g in [guess, 0.1] if guess != 0.1 else [0.1]:
        run = secant(flows, g, opts)
        runs.append(run)
        if run.rate is not None:
            return Outcome(run.rate, runs)
        if run.error is not None:
            return Outcome(run.error, runs)
    return Outcome("#NUM!", runs)


def _call(args: tuple[Arg, ...], opts: Options) -> Outcome:
    flows = floats(args[0])
    guess = args[1] if len(args) > 1 else 0.1
    if not isinstance(guess, float):
        msg = f"guess is not a number: {guess!r}"
        raise TypeError(msg)
    return irr(flows, guess, opts)


def irr_candidate(name: str, opts: Options, family: str) -> Candidate:
    """A candidate scoring `opts`: its value (an error code raises ExcelError), its trace and its events. The
    score asks for all three on one cell in a row, so the last cell's outcome is kept and the call runs once."""
    last: list[tuple[Cell, Outcome]] = []

    def outcome(cell: Cell) -> Outcome:
        if not last or last[0][0] is not cell:
            last[:] = [(cell, _call(cell.args, opts))]
        return last[0][1]

    def evaluate(cell: Cell) -> float:
        value = outcome(cell).value
        if isinstance(value, str):
            raise ExcelError(code=value)
        return value

    return Candidate(
        name,
        evaluate,
        family,
        trace=lambda cell: outcome(cell).trace,
        event=lambda cell: outcome(cell).event,
    )


CANDIDATES: dict[str, tuple[Options, str]] = {
    "base, cap 50": (Options(cap=50), "base"),
    "base, cap 200": (Options(cap=200), "base"),
    "vb: r-half clamp, cap 200": (Options(cap=200, clamp="r-half"), "vb"),
    "vb: d-half clamp, cap 200": (Options(cap=200, clamp="d-half"), "vb"),
    "vb: flat nudge + r-half clamp, cap 200": (Options(cap=200, nudge=True, clamp="r-half"), "vb"),
    "rate: hold on flat, cap 50": (Options(cap=50, hold=True), "rate"),
    "rate: hold on flat + run on past d = 1, cap 50": (Options(cap=50, hold=True, run_on=True), "rate"),
    "rate: hold on flat + run on past d = 1, cap 200": (Options(cap=200, hold=True, run_on=True), "rate"),
}


def candidates() -> list[Candidate]:
    return [irr_candidate(name, opts, family) for name, (opts, family) in CANDIDATES.items()]


def _numeric_irr(cell: Cell) -> bool:
    """An IRR cell whose flows are all numbers (an array literal or a range of constants) and whose guess, if
    any, is a number: feasibility.py's cell set."""
    if not cell.args:
        return False
    try:
        floats(cell.args[0])
    except TypeError:
        return False
    return all(isinstance(a, float) for a in cell.args[1:])


FITS = [
    Fit(
        name="irr-type",
        description="IRR's outcome type and bits on every IRR probe case: the secant in d with the base, VB and RATE failure handlers (T-4834)",
        cases=CASES,
        pattern=r"^=IRR\((.*)\)$",
        candidates=candidates,
        where=_numeric_irr,
        errors=True,
    ),
]
