"""QUOTIENT and MOD (Gate A slice 4h, T-3675, decision 0228).

- QUOTIENT: the integer part of `numerator/denominator`. The engine truncates the double quotient
  (`fn_quotient`), and the math takes it from that double too, as MROUND's does (decision 0246): an
  exact-ratio reading would put `QUOTIENT(1,0.1)` at 9 (the double 0.1 is a little above 1/10)
  where the double quotient is 10, a whole unit from any implementation that divides in binary. The
  tail stratum draws quotients a few ulps from a whole number, so the fill shows which reading Excel
  takes: the double quotient (numbers.md "QUOTIENT truncates the double quotient"). A zero
  denominator is `#DIV/0!`; a quotient that overflows a double has no evidenced value.
- MOD (numbers.md "MOD is an exact remainder", "MOD is #NUM! for a huge quotient", "MOD of a small
  number by a huge opposite-sign divisor is 0", "MOD near the smallest normal"): f = the exact
  fmod(n, d), plus d when f is nonzero and its sign differs from d's. The shift is defined on doubles:
  where `f + d` rounds back to `d` Excel gives 0, so the math does too (a value at the double, as the
  entry states it); otherwise the math is the exact `f + d`. `#NUM!` where `|n/d|` (the double
  quotient) is at least 1.1259E+12, for a subnormal remainder or shift by the entry's rules, and
  `#DIV/0!` for d = 0.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy import model
from rustsheet_tools.accuracy.defs._common import by_stratum, fraction_to_mpf, one_of, scalars
from rustsheet_tools.accuracy.literal import MAX_LITERAL, MIN_LITERAL, Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import decimal_literal, log_uniform, random_sign

MOD_EXACT_SOURCE = "numbers.md#mod-is-an-exact-remainder"
MOD_HUGE_SOURCE = "numbers.md#mod-is-num-for-a-huge-quotient"
MOD_SHIFT_SOURCE = "numbers.md#mod-of-a-small-number-by-a-huge-opposite-sign-divisor-is-0-t-2654"
MOD_SUBNORMAL_SOURCE = "numbers.md#mod-near-the-smallest-normal-t-2714"
QUOTIENT_SOURCE = "numbers.md#quotient-truncates-the-double-quotient-t-3675"
DECISION = "decision 0228"
MOD_QUOTIENT_LIMIT = 1.1259e12  # MOD is #NUM! from this |n/d| up (the decimal, not a power of two)
SMALLEST_NORMAL = 2.0**-1022


def quotient_value(numerator: float, denominator: float) -> int:
    """QUOTIENT: the double quotient truncated toward zero. DomainError for a zero denominator,
    NoValueError where the quotient overflows."""
    if denominator == 0:
        msg = "a zero denominator (#DIV/0!)"
        raise model.DomainError(msg)
    q = numerator / denominator
    if not math.isfinite(q):
        msg = "the double quotient overflows: no Excel evidence"
        raise model.NoValueError(msg)
    return math.trunc(q)


def mod_value(n: float, d: float) -> Fraction:
    """MOD's math (see the module docstring); DomainError where Excel gives an error."""
    if d == 0:
        msg = "a zero divisor (#DIV/0!)"
        raise model.DomainError(msg)
    if abs(n / d) >= MOD_QUOTIENT_LIMIT:
        msg = "|number/divisor| at or above 1.1259E+12 (#NUM!)"
        raise model.DomainError(msg)
    f = math.fmod(n, d)  # exact for doubles
    if f == 0:
        return Fraction(0)

    def subnormal(x: float) -> bool:
        return abs(x) < SMALLEST_NORMAL

    if (f < 0) == (d < 0):
        if subnormal(f):
            msg = "a subnormal remainder (#NUM!)"
            raise model.DomainError(msg)
        return Fraction(f)
    shifted = f + d  # the double sum decides the subnormal and back-to-d rules
    if subnormal(f) and subnormal(shifted) and abs(d) == SMALLEST_NORMAL:
        return Fraction(d) - Fraction(f)
    if subnormal(f) or subnormal(shifted):
        msg = "a subnormal remainder or shift (#NUM!)"
        raise model.DomainError(msg)
    if shifted == d:
        return Fraction(0)
    return Fraction(f) + Fraction(d)


def _quotient(args: Sequence[MathArg]) -> mpmath.mpf:
    n, d = (float(a) for a in scalars(args))
    return mpmath.mpf(quotient_value(n, d))


def _mod(args: Sequence[MathArg]) -> mpmath.mpf:
    n, d = (float(a) for a in scalars(args))
    return fraction_to_mpf(mod_value(n, d))


def _pair(rng: random.Random, n_exp: tuple[int, int], gap: tuple[int, int]) -> list[Arg]:
    """A numerator with exponent in `n_exp` and a denominator `gap` decades below it, signs drawn
    apart."""
    n = decimal_literal(rng, rng.randint(1, 15), rng.randint(*n_exp), random_sign(rng))
    exp = math.floor(math.log10(abs(n))) - rng.randint(*gap)
    exp = max(-307, min(307, exp))
    return [n, decimal_literal(rng, rng.randint(1, 6), exp, random_sign(rng))]


def _near_multiple(rng: random.Random, *, mod: bool) -> list[Arg]:
    """A numerator a whole multiple of the denominator, or one to three units of the 15th digit
    beside it (`0.3` over `0.1`, `QUOTIENT(1,0.1)`), so the fill shows how each side is read."""
    d = decimal_literal(rng, rng.randint(1, 3), rng.randint(-4, 3), random_sign(rng))
    steps = rng.randint(1, 10 ** rng.randint(1, 9 if mod else 12))
    exact = Fraction(str(d)) * steps
    nudge = rng.choice((0, 0, 1, -1, 2, -3))
    digits = len(str(abs(exact.numerator // exact.denominator))) if abs(exact) >= 1 else 0
    unit = Fraction(10) ** (digits - 15) if digits else Fraction(1, 10**15)
    value = float(exact + nudge * unit)
    return [value * random_sign(rng) if mod else value, d]


def _quotient_edge(rng: random.Random) -> list[Arg]:
    n = decimal_literal(rng, 6, rng.randint(-3, 6), random_sign(rng))
    return one_of(
        rng,
        (
            lambda: [0.0, decimal_literal(rng, 3, rng.randint(-3, 3), random_sign(rng))],
            lambda: [n, n * rng.choice((1.0, -1.0, 2.0, 1.5, 1e6))],
            lambda: [
                random_sign(rng) * rng.choice((MAX_LITERAL, 1e300)),
                rng.choice((1.0, 3.0, 1e300, -7.0)),
            ],
            lambda: [random_sign(rng) * rng.choice((MIN_LITERAL, 1e-300)), rng.choice((1.0, 1e-300, 3e-8))],
            lambda: [
                rng.choice((9007199254740992.0, 9007199254740993.0, 1e15 + 1)),
                rng.choice((1.0, 2.0, 3.0)),
            ],
        ),
    )


def _log_uniform_pair(rng: random.Random, keep: Callable[[float], bool]) -> list[Arg]:
    """Both arguments log-uniform over the literal range, signs drawn apart, redrawn until `keep`
    accepts the double quotient."""
    while True:
        n = random_sign(rng) * log_uniform(rng, MIN_LITERAL, MAX_LITERAL)
        d = random_sign(rng) * log_uniform(rng, MIN_LITERAL, MAX_LITERAL)
        if keep(n / d):
            return [n, d]


_quotient_draw = by_stratum(
    {
        "typical": lambda rng: _pair(rng, (-4, 8), (0, 6)),
        "tail": lambda rng: _near_multiple(rng, mod=False),
        "edge": _quotient_edge,
        "extreme": lambda rng: _pair(rng, (-300, 306), (-5, 300)),
        "log-uniform": lambda rng: _log_uniform_pair(rng, math.isfinite),
    },
    "[numerator, denominator]: typical quotients 1 to 1E6, tails beside whole quotients, edges (zero numerator, a "
    "denominator at or above the numerator, the literal range's ends, 2^53), extreme magnitudes with quotients up "
    "to 1E+300, and both log-uniform over the literal range (an overflowing quotient is redrawn)",
)


def _mod_tail(rng: random.Random) -> list[Arg]:
    if rng.random() < 0.6:
        return _near_multiple(rng, mod=True)
    d = decimal_literal(rng, rng.randint(1, 3), rng.randint(-3, 3), random_sign(rng))
    q = rng.choice((1125899999999, 1125900000000, 1125899999999.5, 1e12, 1.2e12, 562949999999.5))
    return [float(Fraction(str(d)) * Fraction(q)) * random_sign(rng), d]


def _mod_edge(rng: random.Random) -> list[Arg]:
    n = decimal_literal(rng, 6, rng.randint(-3, 6), random_sign(rng))
    return one_of(
        rng,
        (
            lambda: [0.0, decimal_literal(rng, 3, rng.randint(-3, 3), random_sign(rng))],
            lambda: [
                rng.choice((1.0, 0.5, 0.75, 1.005)),
                -rng.choice((2.0**52, 2.0**53, 2.0**54, 1e16, 1e17, 1e30)),
            ],
            lambda: [
                random_sign(rng) * rng.choice((1e-307, 1e-300)),
                random_sign(rng) * rng.choice((1e-307, 1e-300)),
            ],
            lambda: [n, n * rng.choice((1.0, -1.0, 2.0, -2.0, 1e6))],
            lambda: [rng.choice((1.0, -1.0, 5.0)), rng.choice((1e-10, -1e-10, 1e-12, 0.1))],
        ),
    )


_mod_draw = by_stratum(
    {
        "typical": lambda rng: _pair(rng, (-4, 8), (0, 6)),
        "tail": _mod_tail,
        "edge": _mod_edge,
        "extreme": lambda rng: _pair(rng, (-290, 306), (-5, 11)),
        "log-uniform": lambda rng: _log_uniform_pair(rng, lambda q: abs(q) < 1e11),
    },
    "[number, divisor]: typical quotients up to 1E6, tails beside whole multiples and about the 1.1259E+12 cut, "
    "edges (zeros, the opposite-sign huge divisor, the smallest normal), extreme magnitudes with quotients up to "
    "1E+11, and both log-uniform with the quotient under the cut",
)


QUOTIENT = register(
    Definition(
        function="QUOTIENT",
        params=(
            Param("numerator", "real", Interval(), source=QUOTIENT_SOURCE),
            Param("denominator", "real", Interval(), source=QUOTIENT_SOURCE),
        ),
        value=_quotient,
        draw=_quotient_draw,
        sources=(DECISION, QUOTIENT_SOURCE),
    )
)
MOD = register(
    Definition(
        function="MOD",
        params=(
            Param("number", "real", Interval(), source=MOD_EXACT_SOURCE),
            Param("divisor", "real", Interval(), source=MOD_EXACT_SOURCE),
        ),
        value=_mod,
        draw=_mod_draw,
        sources=(DECISION, MOD_EXACT_SOURCE, MOD_HUGE_SOURCE, MOD_SHIFT_SOURCE, MOD_SUBNORMAL_SOURCE),
    )
)
