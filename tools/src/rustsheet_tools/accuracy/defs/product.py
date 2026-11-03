"""PRODUCT and SUMSQ (Gate A slice 4i, T-3676, decision 0228).

The math: the exact product, or the exact sum of the squares, of the doubles of the written literals.
Overflow at an intermediate point (numbers.md, "Overflow in `+ - * /` is #NUM! where it happens",
aggregate overflow, T-0192, T-0990): PRODUCT and SUMSQ overflow like SUM, so `PRODUCT(1E+200,1E+200)` is
`#NUM!` and stays `#NUM!` with a later factor of 0 or 1E-200, and `SUMSQ(1E+200)` is `#NUM!`. A point
whose running product, square or running sum of squares leaves the double range is therefore a
domain error. The running values follow Excel's x87 model (each operation rounded to a 64-bit
significand, then to a double; `fitlab.exact.double_round`), left to right, as SUM's do.

Underflow is not claimed: a product or square below the smallest normal double would meet
numbers.md "Subnormal results become 0" on some operations and not on others, and no oracle cell
pins it for these two functions, so the draws keep every running value normal (`_normal`).

The arity is drawn per stratum (`variadic`): 2 to 10 scalars (typical), 2 to 40 near 1 (tail), 1 to
6 or 255 (edge), 50 to 255 near 1 (extreme) and 1 to 6 wide-magnitude ones (log-uniform).
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, one_of, scalars
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.exact import double_round

OVERFLOW_SOURCE = "numbers.md#overflow-in------is-num-where-it-happens"
MAX_ARGS = 255  # Excel's argument limit


def running(xs: Sequence[float], step: Callable[[float, float], Fraction], start: float) -> list[float]:
    """Each running value of a left-to-right fold, x87-rounded; a value beyond the double range is
    `math.inf` (with its sign), and the fold stops there."""
    out: list[float] = []
    acc = start
    for x in xs:
        exact = step(acc, x)
        try:
            acc = double_round(exact)
        except OverflowError:
            out.append(math.copysign(math.inf, 1 if exact > 0 else -1))
            return out
        out.append(acc)
    return out


def running_product(xs: Sequence[float]) -> list[float]:
    """PRODUCT's running products."""
    return running(xs, lambda acc, x: Fraction(acc) * Fraction(x), 1.0)


def squares(xs: Sequence[float]) -> list[float]:
    """SUMSQ's squares, each x87-rounded (`math.inf` past the double range)."""
    out: list[float] = []
    for x in xs:
        try:
            out.append(double_round(Fraction(x) ** 2))
        except OverflowError:
            out.append(math.inf)
    return out


def running_sumsq(xs: Sequence[float]) -> list[float]:
    """SUMSQ's running squares and running sums of squares, in the order they are formed."""
    out: list[float] = []
    total = 0.0
    for square in squares(xs):
        out.append(square)
        if not math.isfinite(square):
            return out
        out.extend(running([square], lambda acc, x: Fraction(acc) + Fraction(x), total))
        total = out[-1]
        if not math.isfinite(total):
            return out
    return out


def _overflow(values: list[float]) -> str | None:
    return None if all(math.isfinite(v) for v in values) else "a running value is beyond the double range"


def _product_domain(args: Sequence[MathArg]) -> str | None:
    return _overflow(running_product([float(x) for x in scalars(args)]))


def _sumsq_domain(args: Sequence[MathArg]) -> str | None:
    return _overflow(running_sumsq([float(x) for x in scalars(args)]))


def _product(args: Sequence[MathArg]) -> mpmath.mpf:
    return mpmath.fprod(scalars(args))


def _sumsq(args: Sequence[MathArg]) -> mpmath.mpf:
    return mpmath.fsum(x * x for x in scalars(args))


# -- draws -------------------------------------------------------------------------------------


def _signed(rng: random.Random, lo: float, hi: float, negative: float = 0.3) -> float:
    x = log_uniform(rng, lo, hi)
    return -x if rng.random() < negative else x


def _normal(values: list[float]) -> bool:
    """Whether no running value is a nonzero below the smallest normal double (underflow is not claimed)."""
    return all(v == 0 or abs(v) >= sys.float_info.min for v in values)


def _near_one(rng: random.Random) -> float:
    return round15(rng.uniform(0.5, 2.0) * (-1 if rng.random() < 0.2 else 1))


def _redraw(
    rng: random.Random,
    draw: Callable[[random.Random], list[float]],
    check: Callable[[list[float]], list[float]],
) -> list[Arg]:
    """`draw` until its running values stay normal (or overflow: that point is a domain error)."""
    while True:
        values = [round15(v) for v in draw(rng)]
        if _normal(check(values)):
            return list(values)


def _product_edge(rng: random.Random) -> list[Arg]:
    big = round15(rng.uniform(1e155, 1e200))
    small = round15(1 / big)
    return one_of(
        rng,
        [
            lambda: [big, big, 0.0],  # overflow at the second factor; a later 0 stays #NUM! (T-0990)
            lambda: [big, big, small],  # the same with a later tiny factor
            lambda: [big, small, big],  # the same factors in an order that never overflows
            lambda: [_signed(rng, 1e-3, 1e6)],  # one argument
            lambda: [
                0.0 if rng.random() < 0.4 else _signed(rng, 1e-3, 1e3) for _ in range(rng.randint(2, 6))
            ],
            lambda: [_near_one(rng) for _ in range(MAX_ARGS)],  # the argument limit
        ],
    )


def _sumsq_edge(rng: random.Random) -> list[Arg]:
    big = round15(rng.uniform(1.4e154, 1e200))
    under = round15(rng.uniform(1e153, 1.3e154))
    return one_of(
        rng,
        [
            lambda: [big],  # the square overflows (SUMSQ(1E+200), T-0990)
            lambda: [
                under,
                round15(rng.uniform(1e153, 1.3e154)),
                under,
            ],  # finite squares; the running sum may overflow
            lambda: [-under],  # the largest finite squares
            lambda: [_signed(rng, 1e-3, 1e6)],  # one argument
            lambda: [
                0.0 if rng.random() < 0.4 else _signed(rng, 1e-3, 1e3) for _ in range(rng.randint(2, 6))
            ],
            lambda: [_signed(rng, 1e-3, 1e3, 0.5) for _ in range(MAX_ARGS)],  # the argument limit
        ],
    )


def _product_draw() -> Callable[[str, random.Random], list[Arg]]:
    return by_stratum(
        {
            "typical": lambda rng: _redraw(
                rng, lambda r: [_signed(r, 1e-3, 1e6) for _ in range(r.randint(2, 10))], running_product
            ),
            "tail": lambda rng: _redraw(
                rng, lambda r: [_near_one(r) for _ in range(r.randint(2, 40))], running_product
            ),
            "edge": _product_edge,
            "extreme": lambda rng: _redraw(
                rng, lambda r: [_near_one(r) for _ in range(r.randint(50, MAX_ARGS))], running_product
            ),
            "log-uniform": lambda rng: _redraw(
                rng, lambda r: [_signed(r, 1e-100, 1e100) for _ in range(r.randint(1, 6))], running_product
            ),
        },
        "PRODUCT draws: the arity and magnitudes by stratum; running products stay normal or overflow.",
    )


def _sumsq_draw() -> Callable[[str, random.Random], list[Arg]]:
    return by_stratum(
        {
            "typical": lambda rng: _redraw(
                rng, lambda r: [_signed(r, 1e-3, 1e6) for _ in range(r.randint(2, 10))], running_sumsq
            ),
            "tail": lambda rng: _redraw(
                rng, lambda r: [_signed(r, 1e-6, 1e6, 0.5) for _ in range(r.randint(2, 40))], running_sumsq
            ),
            "edge": _sumsq_edge,
            "extreme": lambda rng: _redraw(
                rng,
                lambda r: [_signed(r, 1e-20, 1e20, 0.5) for _ in range(r.randint(50, MAX_ARGS))],
                running_sumsq,
            ),
            "log-uniform": lambda rng: _redraw(
                rng, lambda r: [_signed(r, 1e-150, 1e150, 0.5) for _ in range(r.randint(1, 6))], running_sumsq
            ),
        },
        "SUMSQ draws: the arity and magnitudes by stratum; squares and running sums stay normal or overflow.",
    )


PRODUCT = register(
    Definition(
        function="PRODUCT",
        params=(Param("number", "real", source=OVERFLOW_SOURCE),),
        value=_product,
        domain=_product_domain,
        draw=_product_draw(),
        variadic=True,
        sources=(OVERFLOW_SOURCE, "decision 0228"),
    )
)

SUMSQ = register(
    Definition(
        function="SUMSQ",
        params=(Param("number", "real", source=OVERFLOW_SOURCE),),
        value=_sumsq,
        domain=_sumsq_domain,
        draw=_sumsq_draw(),
        variadic=True,
        sources=(OVERFLOW_SOURCE, "decision 0228"),
    )
)
