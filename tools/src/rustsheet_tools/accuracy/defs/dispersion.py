"""STDEV, STDEV.S, STDEVA, VAR, VAR.S, VARA and DEVSQ (Gate A dispersion I, T-4089, decision 0271).

The math, on the written literals' doubles x[1..n] with the exact mean m = (x[1] + ... + x[n]) / n:
- DEVSQ is the exact sum of squared deviations S = sum((x[i] - m)^2) (statistics.md "DEVSQ is a two-pass sum": Excel
  computes it in doubles with two passes; the math is the exact S);
- VAR, VAR.S and VARA are the exact sample variance S / (n - 1), and STDEV, STDEV.S and STDEVA its square root
  (statistics.md "VAR/STDEV switch between a one-pass and a two-pass formula": Excel's choice of formula moves at most
  a few last digits, so the math is the exact variance either way).
STDEVA and VARA over numbers are STDEV.S and VAR.S: they differ only in how text and booleans in a range count
(statistics.md "VARA family: single-cell reference arguments"), and the sampler draws numbers only.

Domain (an Excel error region, never a math value):
- VAR and STDEV of one number is `#DIV/0!` (n - 1 = 0).
- VAR and STDEV: a one-pass sum of squares beyond the double range is `#NUM!` (statistics.md: "`VAR.P` of 1E+200
  twice is `#NUM!`").
- DEVSQ: an overflow in its two-pass form (the sum of the numbers, a squared deviation or their sum) is `#NUM!`,
  inferred from statistics.md "DEVSQ is a two-pass sum" and not probed.
The draws keep |x| at or below 1E+150, so no drawn point reaches either overflow; the rules keep an eval cell over
larger literals from getting a value.
Squares that are subnormal are flushed to 0 by Excel (statistics.md, `STDEV.S({0,2.2E-154})` is 0); the draws keep
every number at or above 1E-60 in magnitude, so a deviation small enough to flush would need a number within 1E-154
of the mean, which a draw does not aim at.

The arity is drawn per stratum: 2 to 10 numbers (typical), offset data near the one-pass/two-pass switch (tail),
small shapes such as equal numbers, one or two numbers and the argument limit (edge; the limit is 254 for VAR.S
and STDEV.S), 50 to 255 numbers of mixed magnitudes (extreme) and a log-uniform count of log-uniform numbers (log-uniform).
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import scalars, signed_log_uniform
from rustsheet_tools.accuracy.defs.aggregate import MAX_ARGS, running_total
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

DEVSQ_SOURCE = "statistics.md#devsq-is-a-two-pass-sum-t-1749"
SWITCH_SOURCE = "statistics.md#varstdev-switch-between-a-one-pass-and-a-two-pass-formula"
VARA_SOURCE = "statistics.md#vara-family-single-cell-reference-arguments"
MAX_MAGNITUDE = 1e150  # every square and the one-pass sum of 255 squares stay inside the double range
MIN_MAGNITUDE = 1e-60  # every nonzero deviation of a draw stays far above the subnormal squares Excel flushes


def squared_deviations(xs: Sequence[mpmath.mpf]) -> mpmath.mpf:
    """The exact sum of squared deviations from the exact mean."""
    m = mpmath.fsum(xs) / len(xs)
    return mpmath.fsum((x - m) ** 2 for x in xs)


def _devsq(args: Sequence[MathArg]) -> mpmath.mpf:
    return squared_deviations(scalars(args))


def _var(args: Sequence[MathArg]) -> mpmath.mpf:
    xs = scalars(args)
    if len(xs) < 2:
        msg = "#DIV/0!: the sample variance of one number"
        raise DomainError(msg)
    return squared_deviations(xs) / (len(xs) - 1)


def _stdev(args: Sequence[MathArg]) -> mpmath.mpf:
    return mpmath.sqrt(_var(args))


def squares_overflow(args: Sequence[MathArg]) -> str | None:
    """VAR's and STDEV's `#NUM!`: a square, or the one-pass running sum of squares, beyond the double range
    (statistics.md: `VAR.P` of 1E+200 twice is `#NUM!`)."""
    squares = [float(x) * float(x) for x in scalars(args)]
    if not all(math.isfinite(s) for s in squares):
        return "a square is beyond the double range"
    _, total = running_total(squares)
    return None if math.isfinite(total) else "the sum of squares is beyond the double range"


def deviations_overflow(args: Sequence[MathArg]) -> str | None:
    """DEVSQ's `#NUM!`, inferred from its two-pass form (statistics.md "DEVSQ is a two-pass sum": the double mean, then
    the squared deviations summed left to right) and not probed: the running sum of the numbers, a squared deviation
    or the running sum of squared deviations beyond the double range. The draws never reach it."""
    xs = [float(x) for x in scalars(args)]
    _, total = running_total(xs)
    if not math.isfinite(total):
        return "the sum of the numbers is beyond the double range"
    mean = total / len(xs)
    squares = [(x - mean) * (x - mean) for x in xs]
    if not all(math.isfinite(s) for s in squares):
        return "a squared deviation is beyond the double range"
    _, total = running_total(squares)
    return None if math.isfinite(total) else "the sum of squared deviations is beyond the double range"


def _var_domain(args: Sequence[MathArg]) -> str | None:
    if len(args) < 2:
        return "#DIV/0!: the sample variance of one number"
    return squares_overflow(args)


# -- draws -------------------------------------------------------------------------------------


def _number(rng: random.Random, lo: float = 1e-3, hi: float = 1e6) -> float:
    return signed_log_uniform(rng, lo, hi)


def _offset(rng: random.Random) -> list[float]:
    """Offset data, where Excel's one-pass sum cancels and its switch picks the two-pass form (statistics.md: `VAR.P`
    of {1000000003,1000000001,...}): a large offset plus a small spread, written to 15 digits."""
    offset = signed_log_uniform(rng, 1.0, 1e9)
    spread = abs(offset) * log_uniform(rng, 1e-9, 1e-1)
    return [round15(offset + spread * rng.uniform(-1, 1)) for _ in range(rng.randint(2, 12))]


def _edge(rng: random.Random, max_args: int) -> list[float]:
    shape = rng.randrange(6)
    if shape == 0:  # two numbers, the smallest sample
        return [_number(rng), _number(rng)]
    if shape == 1:  # equal numbers: S is exactly 0 (Excel's double mean of three or more may not be x)
        x = _number(rng)
        return [x] * rng.randint(2, 6)
    if shape == 5:  # one number, the invalid neighbour: VAR and STDEV are #DIV/0!, DEVSQ is 0
        return [_number(rng)]
    if shape == 2:  # small integers, where Excel keeps the one-pass form ({1,2,4}: 2.333333333333334)
        return [float(rng.randint(-20, 20)) for _ in range(rng.randint(2, 8))]
    if shape == 3:  # the argument limit
        return [_number(rng) for _ in range(max_args)]
    # shape 4: magnitudes near the draw bounds
    return [
        signed_log_uniform(rng, MAX_MAGNITUDE / 1e3, MAX_MAGNITUDE, 0.5) for _ in range(rng.randint(2, 6))
    ]


def _draws(max_args: int) -> Callable[[str, random.Random], list[Arg]]:
    """The draw of a function whose calls take at most `max_args` numbers."""

    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        if stratum == "tail":
            return list(_offset(rng))
        if stratum == "edge":
            return list(_edge(rng, max_args))
        if stratum == "extreme":
            return [_number(rng, 1e-20, 1e20) for _ in range(rng.randint(50, max_args))]
        if stratum == "log-uniform":
            count = max(2, round(log_uniform(rng, 2, max_args)))
            return [_number(rng, MIN_MAGNITUDE, MAX_MAGNITUDE) for _ in range(count)]
        return [_number(rng) for _ in range(rng.randint(2, 10))]

    return draw


_draw = _draws(MAX_ARGS)
# A file holding a newer (`_xlfn.`) function with 255 arguments does not open in Excel; 254 does (syntax.md
# "The repeating maxima", T-4089), so VAR.S and STDEV.S points take at most 254 numbers.
_draw_xlfn = _draws(MAX_ARGS - 1)


def _definition(
    function: str,
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    domain: Callable[[Sequence[MathArg]], str | None],
    sources: tuple[str, ...],
    *,
    draw: Callable[[str, random.Random], list[Arg]] = _draw,
    param: str = "number",
) -> Definition:
    return register(
        Definition(
            function=function,
            params=(Param(param, "real", source=sources[0]),),
            value=value,
            domain=domain,
            draw=draw,
            variadic=True,
            sources=(*sources, "decision 0271"),
        )
    )


VAR = _definition("VAR", _var, _var_domain, (SWITCH_SOURCE,))
VAR_S = _definition("VAR.S", _var, _var_domain, (SWITCH_SOURCE,), draw=_draw_xlfn)
VARA = _definition("VARA", _var, _var_domain, (SWITCH_SOURCE, VARA_SOURCE), param="value")
STDEV = _definition("STDEV", _stdev, _var_domain, (SWITCH_SOURCE,))
STDEV_S = _definition("STDEV.S", _stdev, _var_domain, (SWITCH_SOURCE,), draw=_draw_xlfn)
STDEVA = _definition("STDEVA", _stdev, _var_domain, (SWITCH_SOURCE, VARA_SOURCE), param="value")
DEVSQ = _definition("DEVSQ", _devsq, deviations_overflow, (DEVSQ_SOURCE,))
