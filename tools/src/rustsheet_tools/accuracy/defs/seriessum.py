"""SERIESSUM (Gate A slice 4i, T-3676, decision 0228).

SERIESSUM(x, n, m, coefficients) is the sum of c_i * x^(n + i*m), i = 0, 1, ... over the
coefficients, read in order. The math is that exact sum of the written literals' doubles, with the
rules of numbers.md that depend only on those values:
- "SERIESSUM with x = 0": with x = 0, a term whose power is zero or negative makes the call `#NUM!`
  whatever its coefficient; a positive power is a term of 0.
- "SERIESSUM: a negative x to a non-integer power is a term of 0 (T-2380)": that term counts as 0,
  odd roots included.
- "SERIESSUM: blank coefficient cells are dropped; an overflowing term beats a text coefficient
  (T-2163)" and "Overflow in `+ - * /` is #NUM! where it happens": a power or a term beyond the
  double range is `#NUM!` (also a negative power whose reciprocal underflows,
  `SERIESSUM(1E-10,-33.5,1,1)`), and so is a running sum beyond it.
The coefficients are an array literal (the `array` kind, T-3586); the T-1802 argument-mode rules
("QUOTIENT, FACTDOUBLE and SERIESSUM argument modes") do not arise for scalar x, n and m.

The draws keep every power an exact sum (n and m are integers or multiples of 1/4, so `n + i*m` is
the double Excel forms) and keep every nonzero x^p normal: an underflowing power is not claimed.
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, one_of
from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.exact import double_round

X_ZERO_SOURCE = "numbers.md#seriessum-with-x--0"
NEGATIVE_X_SOURCE = "numbers.md#seriessum-a-negative-x-to-a-non-integer-power-is-a-term-of-0-t-2380"
OVERFLOW_SOURCE = "numbers.md#seriessum-blank-coefficient-cells-are-dropped-an-overflowing-term-beats-a-text-coefficient-t-2163"
MAX_DOUBLE = sys.float_info.max


def _parts(args: Sequence[MathArg]) -> tuple[mpmath.mpf, mpmath.mpf, mpmath.mpf, tuple[mpmath.mpf, ...]]:
    x, n, m, coefficients = args
    assert not isinstance(x, (bool, tuple)) and not isinstance(n, (bool, tuple))
    assert not isinstance(m, (bool, tuple)) and isinstance(coefficients, tuple)
    return x, n, m, coefficients


def terms(args: Sequence[MathArg]) -> list[mpmath.mpf]:
    """The exact terms c_i * x^(n + i*m); DomainError for an x = 0 non-positive power, or a power or
    term beyond the double range."""
    x, n, m, coefficients = _parts(args)
    out: list[mpmath.mpf] = []
    for i, c in enumerate(coefficients):
        p = n + i * m
        if x == 0:
            if p <= 0:
                raise DomainError("x = 0 with a power at or below 0")
            out.append(mpmath.mpf(0))
            continue
        if x < 0 and p != mpmath.floor(p):
            out.append(mpmath.mpf(0))  # T-2380: a negative x to a non-integer power is a term of 0
            continue
        power = mpmath.power(x, p)
        if abs(power) > MAX_DOUBLE or abs(c * power) > MAX_DOUBLE:
            raise DomainError("a power or a term is beyond the double range")
        out.append(c * power)
    return out


def _running_overflow(values: Sequence[mpmath.mpf]) -> bool:
    total = 0.0
    for v in values:
        try:
            total = double_round(Fraction(total) + Fraction(float(v)))
        except OverflowError:
            return True
    return False


def _domain(args: Sequence[MathArg]) -> str | None:
    try:
        values = terms(args)
    except DomainError as why:
        return str(why)
    return "a running sum is beyond the double range" if _running_overflow(values) else None


def _value(args: Sequence[MathArg]) -> mpmath.mpf:
    return mpmath.fsum(terms(args))


# -- draws -------------------------------------------------------------------------------------


def _quarter(rng: random.Random, lo: int, hi: int) -> float:
    """An integer in [lo, hi], or (one time in three) a multiple of 1/4 there: exact powers."""
    value = rng.randint(lo, hi)
    return float(value) if rng.random() < 2 / 3 else value + rng.choice((0.25, 0.5, 0.75))


def _coefficients(rng: random.Random, count: int, lo: float = 1e-3, hi: float = 1e3) -> tuple[float, ...]:
    return tuple(round15(log_uniform(rng, lo, hi) * (-1 if rng.random() < 0.3 else 1)) for _ in range(count))


def _normal_powers(x: float, n: float, m: float, count: int) -> bool:
    """Whether every nonzero x^(n + i*m) is a normal double or overflows (underflow is not claimed)."""
    if x == 0:
        return True
    for i in range(count):
        p = n + i * m
        if x < 0 and p != math.floor(p):
            continue
        log2 = p * math.log2(abs(x))
        if log2 < -1021:
            return False
    return True


def _point(
    rng: random.Random,
    x_lo: float,
    x_hi: float,
    *,
    n: tuple[int, int],
    m: tuple[int, int],
    count: tuple[int, int],
    negative: float = 0.3,
) -> list[Arg]:
    while True:
        x = round15(log_uniform(rng, x_lo, x_hi) * (-1 if rng.random() < negative else 1))
        nn, mm = _quarter(rng, *n), _quarter(rng, *m)
        k = rng.randint(*count)
        if _normal_powers(x, nn, mm, k):
            return [x, nn, mm, _coefficients(rng, k)]


def _taylor(rng: random.Random) -> list[Arg]:
    """A Taylor series near 1 (exp's 1/k!, alternating for a cancelling sum): many terms of falling size."""
    x = round15(rng.uniform(0.05, 2.0) * (-1 if rng.random() < 0.4 else 1))
    k = rng.randint(5, 30)
    return [x, 0.0, 1.0, tuple(round15(1 / math.factorial(i)) for i in range(k))]


def _edge(rng: random.Random) -> list[Arg]:
    return one_of(
        rng,
        [
            lambda: [
                0.0,
                float(rng.randint(-2, 0)),
                1.0,
                _coefficients(rng, rng.randint(1, 4)),
            ],  # x = 0: #NUM!
            lambda: [0.0, float(rng.randint(1, 3)), 1.0, _coefficients(rng, rng.randint(1, 4))],  # x = 0: 0
            lambda: [
                round15(-log_uniform(rng, 0.5, 4)),
                0.5,
                1.0,
                _coefficients(rng, rng.randint(2, 6)),
            ],  # zero terms
            lambda: [
                round15(log_uniform(rng, 1e100, 1e150)),
                3.0,
                1.0,
                _coefficients(rng, 2),
            ],  # an overflowing power
            lambda: [
                round15(log_uniform(rng, 0.5, 4)),
                _quarter(rng, -3, 3),
                0.0,
                _coefficients(rng, 3),
            ],  # m = 0
            lambda: [
                round15(log_uniform(rng, 0.5, 4)),
                _quarter(rng, -3, 3),
                1.0,
                _coefficients(rng, 1),
            ],  # one term
            lambda: [MAX_LITERAL, 1.0, 0.0, (1.0, 1.0)],  # a running sum that overflows
        ],
    )


DRAW = by_stratum(
    {
        "typical": lambda rng: _point(rng, 0.1, 10, n=(-3, 5), m=(1, 3), count=(1, 8)),
        "tail": _taylor,
        "edge": _edge,
        "extreme": lambda rng: _point(rng, 1e-6, 1e6, n=(-20, 20), m=(1, 10), count=(1, 12)),
        "log-uniform": lambda rng: _point(rng, 1e-10, 1e10, n=(-10, 10), m=(0, 5), count=(1, 6)),
    },
    "SERIESSUM draws: x, n, m and the coefficients by stratum; exact powers, no underflowing power.",
)

SERIESSUM = register(
    Definition(
        function="SERIESSUM",
        params=(
            Param("x", "real", source=X_ZERO_SOURCE),
            Param("n", "real", source=X_ZERO_SOURCE),
            Param("m", "real", source=X_ZERO_SOURCE),
            Param("coefficients", "array", source=OVERFLOW_SOURCE),
        ),
        value=_value,
        domain=_domain,
        draw=DRAW,
        sources=(X_ZERO_SOURCE, NEGATIVE_X_SOURCE, OVERFLOW_SOURCE, "decision 0228"),
    )
)
