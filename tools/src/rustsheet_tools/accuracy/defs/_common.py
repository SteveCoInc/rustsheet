"""Helpers shared by the Gate A definitions of several families (T-3748).

Not a function family: the leading underscore keeps `defs/__init__.py` from importing it as one.
"""

from __future__ import annotations

import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Draw, MathArg
from rustsheet_tools.accuracy.sampler import log_uniform


def scalars(args: Sequence[MathArg]) -> list[mpmath.mpf]:
    """The numeric arguments of a definition that takes no logical or array (its sampler draws numbers only)."""
    out: list[mpmath.mpf] = []
    for a in args:
        assert not isinstance(a, (bool, tuple))
        out.append(a)
    return out


def zero_pole(args: Sequence[MathArg]) -> str | None:
    """One-argument functions with a pole at 0 (COT, CSC, COTH, CSCH): Excel's `#DIV/0!`."""
    (x,) = scalars(args)
    return "a pole at 0" if x == 0 else None


StratumDraws = dict[str, Callable[[random.Random], list[Arg]]]


def by_stratum(draws: StratumDraws, doc: str) -> Draw:
    """A `draw` dispatching on the stratum (`log-uniform` is the fallback)."""

    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        return draws.get(stratum, draws["log-uniform"])(rng)

    draw.__doc__ = doc
    return draw


def one_of(rng: random.Random, shapes: Sequence[Callable[[], list[Arg]]]) -> list[Arg]:
    """One of the edge `shapes`, drawn uniformly."""
    return rng.choice(shapes)()


def fraction_to_mpf(r: Fraction | int) -> mpmath.mpf:
    """An exact rational as an mpf at the working precision."""
    r = Fraction(r)
    return mpmath.mpf(r.numerator) / r.denominator


def trunc(x: mpmath.mpf) -> int:
    """`x` truncated toward zero."""
    return int(mpmath.floor(x)) if x >= 0 else -int(mpmath.floor(-x))


def signed_log_uniform(rng: random.Random, lo: float, hi: float, negative: float = 0.3) -> float:
    """A log-uniform magnitude in [lo, hi], negated with probability `negative` (the aggregate families' draws)."""
    x = log_uniform(rng, lo, hi)
    return -x if rng.random() < negative else x
