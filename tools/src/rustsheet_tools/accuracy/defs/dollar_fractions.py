"""DOLLARDE and DOLLARFR (Gate A, T-4110, decision 0273): a price converted between fractional and decimal dollars.

The math, on the doubles of the written arguments, with F = fraction, D = 10^ceil(log10 F) (the smallest power of ten
at or above F) and x = i + f split into its integer part i (toward 0) and fraction f:
- DOLLARDE(x, F) = i + f·D/F (the digits after the point read as a count of 1/F, `DOLLARDE(1.02,16)` 1.125);
- DOLLARFR(x, F) = i + f·F/D, its inverse (`DOLLARFR(1.125,16)` 1.02).
No semantics entry ruled Excel's arithmetic exact, so these get a table (T-3797's default); the evidence before it was
the eval cells: probes/fin-argument-modes H109, H113 and probes/scalar-lift-open-a Z16-Z18 (`DOLLARDE(1.02,2)` 1.1,
`DOLLARDE(2.5,2)` 4.5), and the gen-m9 cells, such as `DOLLARDE(2.5,13)` 5.846153846153847 (D = 100 for F = 13) and
`DOLLARFR(0.2,12)` 0.024000000000000004 (a few ulps from the math). financial.md "Financial function argument modes"
and arrays.md list their argument lifting; the table's calls are scalar.
D is read off the double `log10` of the truncated fraction, as Excel does (probes/t4608-dollarde-dollarfr-fraction-edges,
T-4880): exact at 10^k + 1 up to k = 12, but 10^15 at `1E15+1` (the double log10 rounds to 15) and 10^300 at 1E300,
where the next power of ten above the double would be 10^301. A fraction below 0 is `#NUM!` and one from 0 up to but
not including 1 is `#DIV/0!` (the checks run on the untruncated fraction; the probe's A1:B6), both drawn by the edges;
fractions past 1E9 are drawn too, up to 1E300.
Negative dollars are drawn with the integer part toward 0, which the table confirms: financial.md "DOLLARDE and DOLLARFR scale
the fraction by the next power of ten (T-4110)" records it with the table's points.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    Shape,
    any_edge,
    by_stratum_with_edge,
    real_args,
    signed,
    trunc,
)
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import decimal_literal, log_uniform

MODES_SOURCE = "financial.md#financial-function-argument-modes"
DOLLAR_SOURCE = "financial.md#dollarde-and-dollarfr-scale-the-fraction-by-the-next-power-of-ten-t-4110"
LIFT_SOURCE = "arrays.md#scalar-arguments-lift-shapes-element-errors-and-the-analysis-toolpak-range-rule"
FRACTION_TOP = 10**9  # the log-uniform draws' largest fraction; the edges go to 1E300


def next_power_of_ten(n: int) -> int:
    """D: 10^ceil(log10 n) with Excel's double `log10` (1 for n <= 1): at or above `n` except where the double log10
    rounds a power of ten plus a few units down (`1E15+1`, T-4880)."""
    return 10 ** math.ceil(math.log10(float(n))) if n > 1 else 1


def _split(args: Sequence[MathArg]) -> tuple[int, mpmath.mpf, int, int]:
    """(i, f, F, D) of a call: the dollar's integer part toward 0 and its fraction, the fraction F and the power of ten
    D from the double log10 of it."""
    x, fraction = real_args(args)
    if fraction < 0:
        msg = "a fraction below 0: #NUM!"
        raise DomainError(msg)
    if fraction < 1:
        msg = "a fraction from 0 up to 1: #DIV/0!"
        raise DomainError(msg)
    whole = int(fraction)
    i = trunc(x)
    return i, x - i, whole, next_power_of_ten(whole)


def _dollarde(args: Sequence[MathArg]) -> mpmath.mpf:
    i, f, whole, digits = _split(args)
    return i + f * digits / whole


def _dollarfr(args: Sequence[MathArg]) -> mpmath.mpf:
    i, f, whole, digits = _split(args)
    return i + f * whole / digits


# -- draws -----------------------------------------------------------------------------------------


def _fraction(stratum: str, rng: random.Random) -> int:
    if stratum == "extreme":
        return round(log_uniform(rng, 1e3, FRACTION_TOP))
    if stratum == "log-uniform":
        return round(log_uniform(rng, 1, FRACTION_TOP))
    return rng.choice((2, 4, 8, 16, 32, 64, 3, 12, rng.randint(1, 100)))


def _dollar(stratum: str, rng: random.Random, fraction: int, *, decimal_form: bool) -> float:
    """A dollar amount: a whole part and, for DOLLARDE's typical draws, a count of 1/fraction written in D's digits
    (1.02 for 2/16); otherwise a decimal fraction of up to 15 significant digits."""
    if stratum == "extreme":
        return (
            signed(rng, log_uniform(rng, 1e6, 1e15)) if rng.random() < 0.5 else log_uniform(rng, 1e-15, 1e-3)
        )
    if stratum == "log-uniform":
        return log_uniform(rng, 1e-300, 1e15)
    whole = rng.randint(0, 10 ** rng.randint(0, 6))
    if decimal_form and stratum == "typical" and rng.random() < 0.6:
        return whole + rng.randrange(fraction) / next_power_of_ten(fraction)
    if stratum == "tail":  # a fraction near 0 or near 1
        tiny = log_uniform(rng, 1e-12, 1e-3)
        return whole + (tiny if rng.random() < 0.5 else 1 - tiny)
    return whole + decimal_literal(rng, rng.randint(1, 10), -1, 1.0)


def _make(decimal_form: bool) -> Callable[[str, random.Random], list[Arg]]:
    def make(stratum: str, rng: random.Random) -> list[Arg]:
        fraction = _fraction(stratum, rng)
        return [_dollar(stratum, rng, fraction, decimal_form=decimal_form), fraction]

    return make


EDGES: tuple[Shape, ...] = (
    lambda rng: [
        -(rng.randint(0, 999) + decimal_literal(rng, rng.randint(1, 6), -1, 1.0)),
        rng.choice((2, 8, 16, 32)),
    ],
    lambda rng: [
        rng.randint(0, 999) + decimal_literal(rng, 3, -1, 1.0),
        rng.choice((1, 10, 100, 1000, 10**6)),
    ],
    lambda rng: [float(rng.randint(0, 10**6)), rng.randint(1, 100)],  # a whole dollar: itself
    lambda rng: [  # a fraction below 1: #NUM! below 0, #DIV/0! from 0 to 1
        rng.randint(0, 99) + decimal_literal(rng, 3, -1, 1.0),
        rng.choice((-1, -0.5, 0, 0.3, 0.5, 0.999)),
    ],
    lambda rng: [  # a fraction past 1E9: D from the double log10
        rng.randint(0, 99) + decimal_literal(rng, 3, -1, 1.0),
        rng.choice((10**10 + 1, 10**12 + 1, 10**15 - 1, 1e15 + 1, 1e16, 1e300)),
    ],
    lambda rng: [
        rng.randint(0, 99) + decimal_literal(rng, 4, -1, 1.0),
        rng.choice((9, 11, 99, 101, 999, 1001)),
    ],
)


def _definition(
    function: str, value: Callable[[Sequence[MathArg]], mpmath.mpf], *, decimal_form: bool
) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param(
                    "fractional_dollar" if decimal_form else "decimal_dollar", "real", source=DOLLAR_SOURCE
                ),
                Param("fraction", "int", source=DOLLAR_SOURCE),
            ),
            value=value,
            draw=by_stratum_with_edge(
                _make(decimal_form), any_edge(EDGES), f"{function}: [dollar, fraction] by stratum."
            ),
            sources=(DOLLAR_SOURCE, MODES_SOURCE, LIFT_SOURCE),
        )
    )


DOLLARDE = _definition("DOLLARDE", _dollarde, decimal_form=True)
DOLLARFR = _definition("DOLLARFR", _dollarfr, decimal_form=False)
