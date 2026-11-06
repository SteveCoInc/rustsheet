"""MEDIAN, PERCENTILE, PERCENTILE.INC, PERCENTILE.EXC, QUARTILE, QUARTILE.INC and QUARTILE.EXC (Gate A order
statistics, T-4092, decision 0271).

The math, on the literals' doubles sorted as a[1] <= ... <= a[n] (ties kept), uses AGGREGATE 16-19's kernels
(`defs/aggregate_array.py`, T-3693):
- MEDIAN is a[(n + 1)/2] for an odd n and the exact (a[n/2] + a[n/2 + 1]) / 2 for an even n. Excel computes the
  even case as `a + (b - a)/2` in doubles (statistics.md "MEDIAN of an even count is a + (b - a)/2"), which moves
  the last digit or two; the math is the exact midpoint either way.
- PERCENTILE and PERCENTILE.INC: k in [0, 1], the exact interpolation at the one-based rank 1 + k(n - 1)
  (statistics.md "PERCENTILE and PERCENTILE.INC interpolate from a one-based rank 1 + k(n-1)"). Excel forms that
  rank in doubles, so a k below an ulp of the rank is lost there (`PERCENTILE.INC(...,1E-20)` is the smallest
  value); the math keeps it, and the edge stratum draws such a k on purpose.
- PERCENTILE.EXC: k in (0, 1) and the rank k(n + 1) in [1, n]; the exact interpolation there.
- QUARTILE and QUARTILE.INC: quart = floor(quart) in 0-4, read as PERCENTILE.INC at quart/4; QUARTILE.EXC: quart
  in 1-3, read as PERCENTILE.EXC at quart/4. The fraction is floored, not truncated: probes/stat-order-floor-batch
  A85, A88 and A89 (excel-graph) give `QUARTILE(K1:K8,0.9)` the minimum, 4.9999 the maximum and -0.5 `#NUM!`
  (statistics.md "PERCENTILE.EXC's k and rank domain; QUARTILE floors a fractional quart").

Domain (an Excel error region, never a math value; statistics.md, the entry above and "PERCENTILE and
PERCENTILE.INC interpolate from a one-based rank"): k outside those intervals, a PERCENTILE.EXC rank outside [1, n]
and a quart outside its range are `#NUM!`. The data is a one-row array literal of numbers only, so no data rule
(text, booleans, empty data, error order) applies to a point.
MEDIAN's even case when `b - a` overflows a double is unprobed (statistics.md: two 9E+307 values give 9E+307), so
such a point gets no value; the draws never reach it.

Draws: PERCENTILE*/QUARTILE* reuse AGGREGATE 16-19's samplers (`valid_point`, `near_bound_point`) with each
function's own edge shapes (Microsoft's examples, the exact-double bounds and the error neighbours). MEDIAN reuses
SUM's variadic draw (T-3549: 1 to 255 numbers per stratum, near-cancelling decimals in the tail).
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, domain_of, one_of, scalars
from rustsheet_tools.accuracy.defs.aggregate import draw_numbers
from rustsheet_tools.accuracy.defs.aggregate_array import (
    DOMAIN_SOURCE,
    EVIDENCE,
    PERCENTILE_EXC,
    PERCENTILE_INC,
    PERCENTILE_SOURCE,
    QUARTILE_EXC,
    QUARTILE_INC,
    array_of,
    min_count,
    near_bound_point,
    percentile_exc,
    percentile_inc,
    quart_k,
    sorted_values,
    valid_point,
)
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, MathArg, NoValueError, Param, register

MEDIAN_SOURCE = "statistics.md#median-of-an-even-count-is-a--b---a2"

# Microsoft's examples (support.microsoft.com, each function's page) and probes/aggregate-grid's data.
GRID = (3.0, 1.0, 4.0, 1.0, 5.0, 9.0)
PERCENTILE_EXAMPLE = (1.0, 3.0, 2.0, 4.0)  # PERCENTILE(.INC)(..., 0.3) is 1.9
PERCENTILE_EXC_EXAMPLE = (1.0, 2.0, 3.0, 6.0, 6.0, 6.0, 7.0, 8.0, 9.0)  # 0.25: 2.5; 0, 0.01 and 2: #NUM!
QUARTILE_EXAMPLE = (1.0, 2.0, 4.0, 7.0, 8.0, 9.0, 10.0, 12.0)  # QUARTILE(.INC)(..., 1) is 3.5
QUARTILE_EXC_EXAMPLE = (6.0, 7.0, 15.0, 36.0, 39.0, 40.0, 41.0, 42.0, 43.0, 47.0, 49.0)  # 1: 15, 3: 43


def _median(args: Sequence[MathArg]) -> mpmath.mpf:
    xs = sorted(scalars(args))
    n = len(xs)
    if n % 2:
        return xs[n // 2]
    a, b = xs[n // 2 - 1], xs[n // 2]
    if not math.isfinite(float(b) - float(a)):
        msg = "MEDIAN's b - a beyond the double range (unprobed)"
        raise NoValueError(msg)
    return (a + b) / 2


def _data(args: Sequence[MathArg]) -> tuple[tuple[mpmath.mpf, ...], mpmath.mpf]:
    array, k = args
    if not isinstance(k, mpmath.mpf):
        msg = "k must be a number"
        raise TypeError(msg)
    return sorted_values(array), k


def _percentile_inc(args: Sequence[MathArg]) -> mpmath.mpf:
    xs, k = _data(args)
    return percentile_inc(xs, k)


def _percentile_exc(args: Sequence[MathArg]) -> mpmath.mpf:
    xs, k = _data(args)
    return percentile_exc(xs, k)


def _quartile_inc(args: Sequence[MathArg]) -> mpmath.mpf:
    xs, k = _data(args)
    return percentile_inc(xs, quart_k(k, 0, 4))


def _quartile_exc(args: Sequence[MathArg]) -> mpmath.mpf:
    xs, k = _data(args)
    return percentile_exc(xs, quart_k(k, 1, 3))


# -- draws -------------------------------------------------------------------------------------

Shapes = Callable[[random.Random, tuple[float, ...]], list[Callable[[], list[Arg]]]]


def _percentile_inc_edges(rng: random.Random, array: tuple[float, ...]) -> list[Callable[[], list[Arg]]]:
    return [
        lambda: [array, rng.choice((0.0, 1.0))],  # the extremes
        lambda: [array, rng.choice((-0.1, 1.1, -1e-10, 1.0000000001))],  # #NUM!
        lambda: [(round15(rng.uniform(-9, 9)),), round15(rng.uniform(0, 1))],  # one value
        lambda: [(round15(rng.uniform(-9, 9)),) * 5, round15(rng.uniform(0, 1))],  # ties only
        lambda: [GRID, 0.3],  # probes/aggregate-grid H43
        lambda: [PERCENTILE_EXAMPLE, 0.3],  # Microsoft: 1.9
        # a k lost in Excel's double rank 1 + k(n - 1): Excel gives the smallest value (statistics.md)
        lambda: [(0.0, *(abs(x) for x in array)), rng.choice((1e-20, 1e-200))],
        lambda: [(-0.0001, 7.0), 1e-10],  # statistics.md: Excel -9.999929998994208E-05
    ]


def _percentile_exc_edges(rng: random.Random, array: tuple[float, ...]) -> list[Callable[[], list[Arg]]]:
    three = (1.0, 3.0, 2.0)  # n + 1 = 4: the bounds 0.25 and 0.75 are exact doubles
    seven = (5.0, 2.0, 8.0, 1.0, 9.0, 3.0, 7.0)  # n + 1 = 8: 0.125 and 0.875 exact
    return [
        lambda: [array, rng.choice((0.0, 1.0, -0.5, 1.5))],  # #NUM!
        lambda: [(round15(rng.uniform(1, 9)),), 0.5],  # one value: rank 1 inside [1, 1]
        lambda: [array, round15(0.5 / (len(array) + 1))],  # rank 0.5 below 1: #NUM!
        lambda: [three, rng.choice((0.25, 0.75, 0.2, 0.8))],  # the exact bounds and outside
        lambda: [seven, rng.choice((0.125, 0.875, 0.1, 0.9))],
        lambda: [(round15(rng.uniform(-9, 9)),) * 5, 0.5],  # ties only
        lambda: [GRID, 0.3],  # probes/aggregate-grid H45
        lambda: [PERCENTILE_EXC_EXAMPLE, rng.choice((0.25, 0.0, 0.01, 2.0))],  # Microsoft: 2.5, then #NUM!
    ]


def _quartile_inc_edges(rng: random.Random, array: tuple[float, ...]) -> list[Callable[[], list[Arg]]]:
    return [
        lambda: [array, rng.choice((0.0, 4.0, 4.5))],  # the extremes (4.5 reads as quart 4)
        lambda: [array, rng.choice((5.0, -1.0, 4.9999))],  # quart 5 and -1: #NUM!; 4.9999 is quart 4
        lambda: [(round15(rng.uniform(-9, 9)),), float(rng.randint(0, 4))],  # one value
        lambda: [(round15(rng.uniform(-9, 9)),) * 5, 2.0],  # ties only
        lambda: [GRID, 1.0],  # probes/aggregate-grid H44
        lambda: [QUARTILE_EXAMPLE, float(rng.randint(0, 4))],  # Microsoft: quart 1 is 3.5
    ]


def _quartile_exc_edges(rng: random.Random, array: tuple[float, ...]) -> list[Callable[[], list[Arg]]]:
    return [
        lambda: [array, rng.choice((0.0, 0.5, 4.0))],  # quart 0 and 4: #NUM!
        lambda: [
            (1.0, 2.0),
            rng.choice((1.0, 2.0, 3.0)),
        ],  # n = 2: quart 2's rank 1.5; 1 and 3 outside [1, 2]
        lambda: [(round15(rng.uniform(1, 9)),), rng.choice((1.0, 2.0))],  # one value: quart 2 only
        lambda: [(1.0, 3.0, 2.0), rng.choice((1.0, 2.0, 3.0))],  # n = 3: ranks 1, 2 and 3 exactly
        lambda: [(round15(rng.uniform(-9, 9)),) * 5, 2.0],  # ties only
        lambda: [GRID, 1.0],  # probes/aggregate-grid H46
        lambda: [QUARTILE_EXC_EXAMPLE, rng.choice((1.0, 3.0, 3.5))],  # Microsoft: 15 and 43
    ]


def _draw(code: int, edges: Shapes) -> Callable[[str, random.Random], list[Arg]]:
    """The draw of the function AGGREGATE numbers `code` (16-19): AGGREGATE's samplers, this function's edges."""

    def point(lo: float, hi: float, sizes: tuple[int, int]) -> Callable[[random.Random], list[Arg]]:
        return lambda rng: list(valid_point(rng, code, lo, hi, sizes))

    def tail(rng: random.Random) -> list[Arg]:
        return list(near_bound_point(rng, code, rng.randint(max(2, min_count(code)), 12)))

    def edge(rng: random.Random) -> list[Arg]:
        return one_of(rng, edges(rng, array_of(rng, rng.randint(1, 8), 1e-3, 1e6)))

    return by_stratum(
        {
            "typical": point(1e-3, 1e6, (1, 12)),
            "tail": tail,
            "edge": edge,
            "extreme": point(1e-100, 1e100, (20, 150)),
            "log-uniform": point(1e-150, 1e150, (1, 8)),
        },
        "An array literal and a valid k by stratum (AGGREGATE 16-19's samplers); the edge stratum adds the domain "
        "errors, the exact-double bounds and Microsoft's examples.",
    )


def _order(
    function: str,
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    code: int,
    edges: Shapes,
    *,
    k_name: str,
    sources: tuple[str, ...],
) -> Definition:
    return register(
        Definition(
            function=function,
            params=(Param("array", "array", source=sources[0]), Param(k_name, "real", source=sources[0])),
            value=value,
            domain=domain_of(value),
            draw=_draw(code, edges),
            sources=(*sources, EVIDENCE, "decision 0271"),
        )
    )


MEDIAN = register(
    Definition(
        function="MEDIAN",
        params=(Param("number", "real", source=MEDIAN_SOURCE),),
        value=_median,
        draw=draw_numbers,
        variadic=True,
        sources=(MEDIAN_SOURCE, "decision 0271"),
    )
)
PERCENTILE = _order(
    "PERCENTILE",
    _percentile_inc,
    PERCENTILE_INC,
    _percentile_inc_edges,
    k_name="k",
    sources=(PERCENTILE_SOURCE,),
)
PERCENTILE_INC_DEF = _order(
    "PERCENTILE.INC",
    _percentile_inc,
    PERCENTILE_INC,
    _percentile_inc_edges,
    k_name="k",
    sources=(PERCENTILE_SOURCE,),
)
PERCENTILE_EXC_DEF = _order(
    "PERCENTILE.EXC",
    _percentile_exc,
    PERCENTILE_EXC,
    _percentile_exc_edges,
    k_name="k",
    sources=(PERCENTILE_SOURCE, DOMAIN_SOURCE),
)
QUARTILE = _order(
    "QUARTILE",
    _quartile_inc,
    QUARTILE_INC,
    _quartile_inc_edges,
    k_name="quart",
    sources=(DOMAIN_SOURCE, PERCENTILE_SOURCE),
)
QUARTILE_INC_DEF = _order(
    "QUARTILE.INC",
    _quartile_inc,
    QUARTILE_INC,
    _quartile_inc_edges,
    k_name="quart",
    sources=(DOMAIN_SOURCE, PERCENTILE_SOURCE),
)
QUARTILE_EXC_DEF = _order(
    "QUARTILE.EXC",
    _quartile_exc,
    QUARTILE_EXC,
    _quartile_exc_edges,
    k_name="quart",
    sources=(DOMAIN_SOURCE, PERCENTILE_SOURCE),
)
