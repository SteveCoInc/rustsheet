"""PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC (Gate A, T-4093, decision 0271).

The rank, exact, on the literals' doubles sorted as a[1] <= ... <= a[n] (ties kept), with c the count of values
below x (statistics.md "PERCENTRANK ranks x by the count of values below it (T-4093)", fitted to Microsoft's examples
and VERIFIED with this task's table cells, ties and a constant array included):
- x equal to a value: the position p = c; x strictly between a[c] and a[c + 1]: p = c - 1 + (x - a[c]) / (a[c + 1] -
  a[c]), the linear interpolation between the two neighbours.
- PERCENTRANK and PERCENTRANK.INC: p / (n - 1); PERCENTRANK.EXC: (p + 1) / (n + 1).

The result is that rank at `significance` digits (3 when omitted), by Excel's own decimal rule, as decisions 0223
and 0244 read the ROUND and CEILING families: the rank times 10^significance is cut to 15 significant digits (a
16th-digit tie away from zero, `ceiling_floor.cut15`), floored, and divided back (statistics.md "PERCENTRANK: a huge
significance is #DIV/0!; truncation does not lose a digit at a cut", the fit of probes/t2473-percentrank-significance
D1-D24). Without the cut the math of `PERCENTRANK.EXC({1E+15,1,-1E+15},0.3)`, whose exact rank is
0.4999999999999998..., would be 0.499 where Excel gives 0.5, a false 0.2% gap. Where the scaled rank lies within
rounding noise of a point where that digit changes, the digit depends on how Excel's doubles round, so such a point
has no math value (`NoValueError`); see `_scaled_rank`.

Domain (an Excel error region, never a math value): x below the smallest value or above the largest is `#N/A`
(x at either end is in range, EXC too), checked before significance (statistics.md "Order statistics: the scalar
argument's error before the no-number result; PERCENTRANK's #N/A before significance"); then a significance below 1
is `#NUM!` and one of 309 or more (10^significance overflows) `#DIV/0!` (statistics.md, the huge-significance entry
above). One value equal to x ranks 1 for all three functions, whatever numeric significance (statistics.md
"Scalar distributions at extreme arguments, part 2": probes/percentrank-single-value B2-B20, EXC too); with a
significance of 309 or more there it is unprobed (the entry's UNVERIFIED line), so such a point has no value. The data
is a one-row array literal of numbers only, so no data rule (text, booleans, empty data, error order) applies.
A neighbour gap a[c + 1] - a[c] beyond the double range is unprobed, so such a point has no value; the draws never
reach it. A fractional significance is not drawn (`int`): an eval cell passing one gets no math value.

Draws: the data reuses AGGREGATE 14-19's `array_of` (T-3693); x is a data value or a 15-digit value between two
neighbours, and the edge stratum adds the error regions, ties, a single value, Microsoft's examples and the t2473
cells.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, domain_of, fraction_to_mpf, one_of, scalars
from rustsheet_tools.accuracy.defs.aggregate_array import array_of, sorted_values
from rustsheet_tools.accuracy.defs.ceiling_floor import cut15
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    MathArg,
    NoValueError,
    Param,
    register,
)

CUT_SOURCE = (
    "statistics.md#percentrank-a-huge-significance-is-div0-truncation-does-not-lose-a-digit-at-a-cut-t-2256"
)
ORDER_SOURCE = (
    "statistics.md#order-statistics-the-scalar-arguments-error-before-the-no-number-result-percentranks-na-"
    "before-significance-t-2256"
)
SINGLE_SOURCE = "statistics.md#scalar-distributions-at-extreme-arguments-part-2-t-2256"
RANK_SOURCE = "statistics.md#percentrank-ranks-x-by-the-count-of-values-below-it-t-4093"
MS_DOCS = "support.microsoft.com PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC examples"
EVIDENCE = "probes/t2473-percentrank-significance"
DEFAULT_SIGNIFICANCE = 3
OVERFLOW_SIGNIFICANCE = 309  # 10^309 overflows a double: `#DIV/0!` from here on (t2473 A7, B7, C7)
ULP = 2.0**-53  # a double's unit roundoff
DIGIT_NOISE = Fraction(
    1, 10**12
)  # a digit change smaller than this, relative, is the 15-digit cut's last digit

# Microsoft's examples: PERCENTRANK(.INC)(INC_EXAMPLE, x) is 0.333 for 2, 0.555 for 4, 0.666 for 8, 0.583 for 5;
# PERCENTRANK.EXC(EXC_EXAMPLE, x) is 0.7 for 7, 0.381 for 5.43 and 0.3 for 5.43 at 1 digit.
INC_EXAMPLE = (13.0, 12.0, 11.0, 8.0, 4.0, 3.0, 2.0, 1.0, 1.0, 1.0)
EXC_EXAMPLE = (1.0, 2.0, 3.0, 6.0, 6.0, 6.0, 7.0, 8.0, 9.0)


def _exact(x: mpmath.mpf) -> Fraction:
    """The exact rational of an argument's double."""
    return Fraction(float(x))


def _position(xs: Sequence[Fraction], x: Fraction) -> tuple[Fraction, int, bool]:
    """x's zero-based position among the sorted `xs` (inside [xs[0], xs[-1]]), the count c of values below it,
    and whether x equals a value."""
    c = sum(1 for v in xs if v < x)
    if xs[c] == x:
        return Fraction(c), c, True
    below, above = xs[c - 1], xs[c]
    if math.isinf(float(above) - float(below)):
        msg = "the neighbour gap overflows a double (unprobed)"
        raise NoValueError(msg)
    return c - 1 + (x - below) / (above - below), c, False


def _double_position(xs: Sequence[float], x: float, c: int, hit: bool) -> float:
    """The position in doubles, Excel's operation order as the engine fits it (`percentrank_inc_impl`)."""
    if hit:
        return float(c)
    below, above = xs[c - 1], xs[c]
    return c - 1.0 + (x - below) / (above - below)


def _truncated(scaled: Fraction, factor: int) -> Fraction:
    """Excel's digit rule on the scaled rank: the 15-digit cut, floored, divided back."""
    cut = cut15(scaled)
    return Fraction(cut.numerator // cut.denominator, factor)


def _scaled_rank(rank: Fraction, rank_double: float, significance: int) -> Fraction:
    """`rank` at `significance` digits by Excel's rule. NoValueError where the result changes inside the noise of
    Excel's double scaled rank: twice the gap between the exact scaled rank and the double one Excel's operation
    order gives, plus two ulps (so an exact boundary such as 0.5 or 0.29, which the cut absorbs, keeps its value).
    A change below `DIGIT_NOISE` relative is not one: from about 15 digits on the cut decides the last digit of a
    15-digit result (2/3 at 15 digits is 0.666666666666667), which is far inside Gate A's ε either way."""
    factor = 10**significance
    scaled = rank * factor
    double = Fraction(rank_double * float(factor))
    noise = 2 * abs(double - scaled) + scaled * Fraction(2 * ULP)
    result = _truncated(scaled, factor)
    candidates = (
        _truncated(scaled - noise, factor),
        _truncated(scaled + noise, factor),
        _truncated(double, factor),
    )
    if max(abs(c - result) for c in candidates) > DIGIT_NOISE * max(abs(c) for c in (result, *candidates)):
        msg = "the scaled rank is within rounding noise of a digit boundary"
        raise NoValueError(msg)
    return result


def _rank(args: Sequence[MathArg], exclusive: bool) -> mpmath.mpf:
    x_arg, *rest = scalars(args[1:])
    values = sorted_values(args[0])
    xs = [_exact(v) for v in values]
    x = _exact(x_arg)
    significance = int(rest[0]) if rest else DEFAULT_SIGNIFICANCE
    if x < xs[0] or x > xs[-1]:
        raise DomainError("x outside the data (#N/A)")
    n = len(xs)
    if n == 1:
        if significance >= OVERFLOW_SIGNIFICANCE:
            msg = "one value with a significance of 309 or more (unprobed)"
            raise NoValueError(msg)
        return mpmath.mpf(1)
    if significance < 1:
        raise DomainError("significance below 1 (#NUM!)")
    if significance >= OVERFLOW_SIGNIFICANCE:
        raise DomainError("10^significance overflows a double (#DIV/0!)")
    position, c, hit = _position(xs, x)
    doubles = [float(v) for v in values]
    pos_double = _double_position(doubles, float(x_arg), c, hit)
    if exclusive:
        rank = (position + 1) / (n + 1)
        rank_double = (pos_double + 1.0) / (n + 1)
    else:
        rank = position / (n - 1)
        rank_double = pos_double / (n - 1)
    return fraction_to_mpf(_scaled_rank(rank, rank_double, significance))


def _inc(args: Sequence[MathArg]) -> mpmath.mpf:
    return _rank(args, exclusive=False)


def _exc(args: Sequence[MathArg]) -> mpmath.mpf:
    return _rank(args, exclusive=True)


# -- draws -------------------------------------------------------------------------------------


def _x_in(rng: random.Random, array: tuple[float, ...]) -> float:
    """A data value (40%) or a 15-digit value between two distinct neighbours."""
    xs = sorted(array)
    if len(set(xs)) == 1 or rng.random() < 0.4:
        return rng.choice(xs)
    i = rng.choice([j for j in range(len(xs) - 1) if xs[j] < xs[j + 1]])
    return min(xs[i + 1], max(xs[i], round15(rng.uniform(xs[i], xs[i + 1]))))


def _significance(rng: random.Random, hi: int = 6) -> list[Arg]:
    """No significance (the default 3) half the time, else 1 to `hi` digits."""
    return [] if rng.random() < 0.5 else [rng.randint(1, hi)]


def _point(lo: float, hi: float, sizes: tuple[int, int]) -> Callable[[random.Random], list[Arg]]:
    def draw(rng: random.Random) -> list[Arg]:
        array = array_of(rng, rng.randint(*sizes), lo, hi)
        return [array, _x_in(rng, array), *_significance(rng)]

    return draw


def _tail(rng: random.Random) -> list[Arg]:
    """Values straddling 0, x a hair beside a value (a position near a whole number) and up to 15 digits."""
    array = array_of(rng, rng.randint(2, 12), 1e-3, 1e3, negative=0.5)
    xs = sorted(array)
    i = rng.randrange(len(xs) - 1)
    if xs[i] == xs[i + 1]:
        return [array, xs[i], *_significance(rng, 15)]
    gap = xs[i + 1] - xs[i]
    hair = gap * 10.0 ** rng.uniform(-12, -3)
    x = round15(xs[i] + hair if rng.random() < 0.5 else xs[i + 1] - hair)
    return [array, min(xs[i + 1], max(xs[i], x)), *_significance(rng, 15)]


def _edges(rng: random.Random, array: tuple[float, ...]) -> list[Callable[[], list[Arg]]]:
    xs = sorted(array)
    lo, hi = xs[0], xs[-1]
    span = max(abs(lo), abs(hi), 1.0)
    one = round15(rng.uniform(-9, 9))
    return [
        lambda: [array, rng.choice((lo, hi))],  # the ends are in range (rank 0 or 1, EXC 1/(n+1) or n/(n+1))
        lambda: [array, round15(rng.choice((lo - span * 0.01, hi + span * 0.01)))],  # #N/A
        lambda: [array, _x_in(rng, array), rng.choice((0, -1))],  # #NUM! (x in range)
        lambda: [array, _x_in(rng, array), rng.choice((309, 400, 2147483648))],  # #DIV/0!
        lambda: [array, _x_in(rng, array), rng.choice((15, 16, 20, 100, 308))],  # many digits
        lambda: [(one,), one, *rng.choice(([], [0], [-1], [3]))],  # one value: 1
        lambda: [(one,), round15(one + rng.choice((-1.0, 1.0)))],  # one value, x beside it: #N/A
        lambda: [(1.0, 2.0, 2.0, 2.0, 3.0), rng.choice((1.0, 2.0, 3.0, 1.5, 2.5))],  # ties
        lambda: [(one,) * 5, one],  # ties only
        lambda: [INC_EXAMPLE, rng.choice((2.0, 4.0, 8.0, 5.0, 1.0, 13.0))],  # Microsoft (INC)
        lambda: [
            EXC_EXAMPLE,
            *rng.choice(([7.0], [5.43], [5.43, 1], [6.0], [1.0], [9.0])),
        ],  # Microsoft (EXC)
        lambda: [(1e15, 1.0, -1e15), 0.3, *rng.choice(([], [15], [16], [20]))],  # t2473 D1-D4: 0.5
        lambda: [(0.0, 1.0), rng.choice((0.36, 0.29, 0.57, 0.58)), 2],  # t2473 D5-D10
        lambda: [(1.0, 2.0, 3.0, 4.0), 2.5, rng.choice((15, 100, 308))],  # t2473 A1-A6, B1-B6: 0.5
        lambda: [(0.0, 3.0), rng.choice((1.0, 2.0)), rng.choice((15, 16, 17))],  # t2473 D19-D22
    ]


DRAW = by_stratum(
    {
        "typical": _point(1e-3, 1e6, (2, 12)),
        "tail": _tail,
        "edge": lambda rng: one_of(rng, _edges(rng, array_of(rng, rng.randint(2, 8), 1e-3, 1e6))),
        "extreme": _point(1e-100, 1e100, (20, 150)),
        "log-uniform": _point(1e-150, 1e150, (2, 8)),
    },
    "An array literal, an x inside it (a value or between two neighbours) and an optional significance by stratum; "
    "the edge stratum adds the error regions, ties, a single value, Microsoft's examples and the t2473 cells.",
)


def _definition(function: str, value: Callable[[Sequence[MathArg]], mpmath.mpf]) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("array", "array", source=RANK_SOURCE),
                Param("x", "real", source=ORDER_SOURCE),
                Param("significance", "int", source=CUT_SOURCE, optional=True),
            ),
            value=value,
            domain=domain_of(value),
            draw=DRAW,
            sources=(
                RANK_SOURCE,
                MS_DOCS,
                CUT_SOURCE,
                ORDER_SOURCE,
                SINGLE_SOURCE,
                EVIDENCE,
                "decision 0271",
            ),
        )
    )


PERCENTRANK = _definition("PERCENTRANK", _inc)
PERCENTRANK_INC = _definition("PERCENTRANK.INC", _inc)
PERCENTRANK_EXC = _definition("PERCENTRANK.EXC", _exc)
