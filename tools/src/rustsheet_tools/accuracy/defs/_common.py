"""Helpers shared by the Gate A definitions of several families (T-3748).

Not a function family: the leading underscore keeps `defs/__init__.py` from importing it as one.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import DomainError, Draw, MathArg, NoValueError, spend
from rustsheet_tools.accuracy.sampler import log_uniform

DF_CUT = 1e10  # the cumulative functions' last valid (truncated) df (statistics.md Student-t df limits)
CONDITION_LIMIT = 1e-10  # an ill-conditioned point (kappa * 2^-52 above this) is redrawn
EPS = 2.0**-52
OVERFLOW_POINT = mpmath.mpf(2) ** 1024 * (
    1 - mpmath.mpf(2) ** -54
)  # the smallest value that rounds past the largest double


def scalars(args: Sequence[MathArg]) -> list[mpmath.mpf]:
    """The numeric arguments of a definition that takes no logical or array (its sampler draws numbers only)."""
    out: list[mpmath.mpf] = []
    for a in args:
        assert not isinstance(a, (bool, tuple))
        out.append(a)
    return out


def finite_double(value: int | mpmath.mpf, what: str) -> mpmath.mpf:
    """`value` as an mpf; DomainError (`#NUM!`) where it rounds past the largest double (FACT, COMBIN, GAMMA,
    PERMUT)."""
    result = mpmath.mpf(value)
    if abs(result) >= OVERFLOW_POINT:
        msg = f"{what} overflows a double (#NUM!)"
        raise DomainError(msg)
    return result


def ill_conditioned(kappa: float) -> None:
    """NoValueError (redraw) when a relative condition number `kappa` puts one rounding past `CONDITION_LIMIT`."""
    if kappa * EPS > CONDITION_LIMIT:
        msg = f"ill-conditioned (kappa {kappa:.3g})"
        raise NoValueError(msg)


def zero_pole(args: Sequence[MathArg]) -> str | None:
    """One-argument functions with a pole at 0 (COT, CSC, COTH, CSCH): Excel's `#DIV/0!`."""
    (x,) = scalars(args)
    return "a pole at 0" if x == 0 else None


StratumDraws = dict[str, Callable[[random.Random], list[Arg]]]
Shape = Callable[[random.Random], list[Arg]]
"""One edge shape (or any one-stratum draw): rng → a whole point."""


def domain_of(value: Callable[[Sequence[MathArg]], mpmath.mpf]) -> Callable[[Sequence[MathArg]], str | None]:
    """A definition's `domain` from its `value`: why `value` raises `DomainError` at the point, or None."""

    def domain(args: Sequence[MathArg]) -> str | None:
        try:
            value(args)
        except DomainError as why:
            return str(why)
        return None

    return domain


def by_stratum(draws: StratumDraws, doc: str) -> Draw:
    """A `draw` dispatching on the stratum (`log-uniform` is the fallback)."""

    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        return draws.get(stratum, draws["log-uniform"])(rng)

    draw.__doc__ = doc
    return draw


def by_stratum_with_edge(make: Draw, edge: Shape, doc: str) -> Draw:
    """A `draw` taking the edge stratum from `edge` and every other stratum from `make(stratum, rng)` (an unknown
    stratum is `make`'s log-uniform, as in `by_stratum`)."""

    def one(stratum: str) -> Shape:
        def draw(rng: random.Random) -> list[Arg]:
            return make(stratum, rng)

        return draw

    draws: StratumDraws = {s: one(s) for s in ("typical", "tail", "extreme", "log-uniform")}
    return by_stratum({**draws, "edge": edge}, doc)


def any_edge(edges: Sequence[Shape]) -> Shape:
    """An edge draw picking one of `edges` uniformly, then drawing it."""

    def edge(rng: random.Random) -> list[Arg]:
        return rng.choice(edges)(rng)

    return edge


def one_of(rng: random.Random, shapes: Sequence[Callable[[], list[Arg]]]) -> list[Arg]:
    """One of the edge `shapes`, drawn uniformly."""
    return rng.choice(shapes)()


def fraction_to_mpf(r: Fraction | int) -> mpmath.mpf:
    """An exact rational as an mpf at the working precision."""
    r = Fraction(r)
    return mpmath.mpf(r.numerator) / r.denominator


def mpf_to_fraction(v: mpmath.mpf) -> Fraction:
    """An mpf's exact binary value as a rational (the inverse of `fraction_to_mpf`); `man_exp` gives |v|."""
    man, exp = v.man_exp
    return (-1 if v < 0 else 1) * Fraction(int(man)) * Fraction(2) ** int(exp)


def trunc(x: mpmath.mpf) -> int:
    """`x` truncated toward zero."""
    return int(mpmath.floor(x)) if x >= 0 else -int(mpmath.floor(-x))


def square_overflows(x: float) -> bool:
    """Whether x*x overflows a double (from |x| = 2^512): the `#NUM!` cut of ASINH and ACOSH and of the normal
    densities (statistics.md T-2435)."""
    return math.isinf(x * x)


def signed(rng: random.Random, x: float) -> float:
    """`x` or `-x`, each with probability 1/2."""
    return -x if rng.random() < 0.5 else x


def signed_log_uniform(rng: random.Random, lo: float, hi: float, negative: float = 0.3) -> float:
    """A log-uniform magnitude in [lo, hi], negated with probability `negative` (the aggregate families' draws)."""
    x = log_uniform(rng, lo, hi)
    return -x if rng.random() < negative else x


def sum_away_from_mode(
    term: mpmath.mpf, j: mpmath.mpf, end: mpmath.mpf, step: int, ratio: Callable[[mpmath.mpf], mpmath.mpf]
) -> mpmath.mpf:
    """The sum of a log-concave pmf from count `j` (whose pmf is `term`) to `end`, moving away from the mode by
    `step` (+1 or -1); `ratio(j)` is pmf(j + step) / pmf(j). The ratio falls monotonically away from the mode, so once
    it is below 1 the rest of the sum is at most term * r / (1 - r) (a geometric bound) and the walk stops there.
    Every term is charged to the work budget (BINOM.DIST.RANGE, T-3741; HYPGEOM.DIST, T-4105)."""
    eps = mpmath.eps * 2**-10
    total = term
    while (j < end) if step > 0 else (j > end):
        spend()
        r = ratio(j)
        term *= r
        total += term
        j += step
        if r < 1 and term * r / (1 - r) <= eps * total:
            break
    return total
