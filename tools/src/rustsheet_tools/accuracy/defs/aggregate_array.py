"""AGGREGATE's array forms, function_num 14-19 (Gate A slice 4l, T-3693, decision 0228).

`AGGREGATE(function_num, options, array, k)` with function_num 14-19 is LARGE, SMALL, PERCENTILE.INC,
QUARTILE.INC, PERCENTILE.EXC or QUARTILE.EXC of the array, and k follows the named function (numbers.md
"AGGREGATE": `AGGREGATE(14,0,E1:E6,2.9)` = `LARGE(E1:E6,2.9)`). The array is a one-row array literal (T-3586), so it
holds no hidden rows, nested SUBTOTALs or errors, and options 0-7 do not change its numbers: the options are drawn
but do not enter the math. An eval cell gets a math value only where the array is a literal
(`literal_arrays`): over a range, hidden rows and the options that skip them are outside this math. Functions 1-13
over an array literal are `#VALUE!` (numbers.md "AGGREGATE"), so a function_num outside 14-19 is a domain error.

The math, on the literals' doubles a[1] <= ... <= a[n] (sorted, ties kept) and the double of k:
- 14 (LARGE): the ceil(k)-th largest; 15 (SMALL): the floor(k)-th smallest. k below 1, or ceil(k) above n, is a
  domain error (statistics.md "LARGE/SMALL: a fractional k rounds up for LARGE and down for SMALL").
- 16 (PERCENTILE.INC): k in [0, 1]; the exact interpolation at the one-based rank 1 + k(n - 1)
  (statistics.md "PERCENTILE and PERCENTILE.INC interpolate from a one-based rank"). Excel computes that rank in
  doubles, so a k below an ulp of the rank is lost there; the math keeps it and the draws keep k at or above 1e-6.
- 18 (PERCENTILE.EXC): k in (0, 1) and the rank k(n + 1) in [1, n]; the exact interpolation there.
- 17 (QUARTILE.INC) and 19 (QUARTILE.EXC): quart = floor(k), in 0-4 or 1-3, read as the percentile at quart/4.
  Both rules, PERCENTILE.EXC's domain and the floored quart, are statistics.md "PERCENTILE.EXC's k and rank domain;
  QUARTILE floors a fractional quart" (T-4092, VERIFIED with this table's cells).
Excel's own evidence: probes/aggregate-grid H41:H46 (`AGGREGATE(14..19,0,{3,1,4,1,5,9},k)`: 5, 1, 2, 1.5,
1.2000000000000002 and 1, excel-graph 2026-10-06, T-0993); the math there is 5, 1, 2, 1.5, 1.2 (to the double of
0.3: 1.19999999999999992...) and 1.
"""

from __future__ import annotations

import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, domain_of, is_real, one_of, signed_log_uniform
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, Interval, MathArg, Param, register

AGGREGATE_SOURCE = "numbers.md#aggregate"
K_FIRST_SOURCE = (
    "statistics.md#aggregate-14-19-k-is-read-before-the-array-a-referenced-error-k-is-value-t-2281"
)
LARGE_SMALL_SOURCE = "statistics.md#largesmall-a-fractional-k-rounds-up-for-large-and-down-for-small"
PERCENTILE_SOURCE = "statistics.md#percentile-and-percentileinc-interpolate-from-a-one-based-rank-1--kn-1-gap-triage-2026-10-25"
DOMAIN_SOURCE = "statistics.md#percentileexcs-k-and-rank-domain-quartile-floors-a-fractional-quart-t-4092"
EVIDENCE = "probes/aggregate-grid"

LARGE, SMALL, PERCENTILE_INC, QUARTILE_INC, PERCENTILE_EXC, QUARTILE_EXC = range(14, 20)


def _point_args(args: Sequence[MathArg]) -> tuple[int, tuple[mpmath.mpf, ...], mpmath.mpf]:
    function_num, _options, array, k = args
    assert isinstance(array, tuple) and is_real(function_num) and is_real(k)
    return int(function_num), sorted_values(array), k


def sorted_values(array: MathArg) -> tuple[mpmath.mpf, ...]:
    """An array argument's numbers sorted ascending, ties kept (the order statistics' a[1] <= ... <= a[n]).
    TypeError for a scalar, a logical or a matrix: the definitions take a one-row array of numbers only."""
    if not isinstance(array, tuple) or not all(isinstance(x, mpmath.mpf) for x in array):
        msg = "an order statistic's data must be a one-row array of numbers"
        raise TypeError(msg)
    return tuple(sorted(x for x in array if isinstance(x, mpmath.mpf)))


def _interpolate(xs: tuple[mpmath.mpf, ...], rank: mpmath.mpf) -> mpmath.mpf:
    """The exact value at the one-based fractional `rank` in [1, n] of the sorted `xs`."""
    j = int(mpmath.floor(rank))
    h = rank - j
    low = xs[j - 1]
    return low if h == 0 else low + h * (xs[j] - low)


def percentile_inc(xs: tuple[mpmath.mpf, ...], k: mpmath.mpf) -> mpmath.mpf:
    """PERCENTILE.INC of the sorted `xs`: the exact interpolation at the one-based rank 1 + k(n - 1)."""
    if k < 0 or k > 1:
        raise DomainError("PERCENTILE.INC: k outside [0, 1]")
    return _interpolate(xs, 1 + k * (len(xs) - 1))


def percentile_exc(xs: tuple[mpmath.mpf, ...], k: mpmath.mpf) -> mpmath.mpf:
    """PERCENTILE.EXC of the sorted `xs`: the exact interpolation at the rank k(n + 1), in [1, n]."""
    n = len(xs)
    rank = k * (n + 1)
    if k <= 0 or k >= 1 or rank < 1 or rank > n:
        raise DomainError("PERCENTILE.EXC: k outside (0, 1) or its rank k(n + 1) outside [1, n]")
    return _interpolate(xs, rank)


def quart_k(k: mpmath.mpf, lo: int, hi: int) -> mpmath.mpf:
    """QUARTILE's percentile quart/4 for quart = floor(k), which must lie in lo-hi."""
    quart = int(mpmath.floor(k))
    if not lo <= quart <= hi:
        msg = f"quart {quart} outside {lo}-{hi}"
        raise DomainError(msg)
    return mpmath.mpf(quart) / 4


def _value(args: Sequence[MathArg]) -> mpmath.mpf:
    function_num, xs, k = _point_args(args)
    n = len(xs)
    if function_num in (LARGE, SMALL):
        if k < 1 or int(mpmath.ceil(k)) > n:
            raise DomainError("k below 1 or ceil(k) above the count")
        return xs[n - int(mpmath.ceil(k))] if function_num == LARGE else xs[int(mpmath.floor(k)) - 1]
    if function_num == PERCENTILE_INC:
        return percentile_inc(xs, k)
    if function_num == QUARTILE_INC:
        return percentile_inc(xs, quart_k(k, 0, 4))
    if function_num == PERCENTILE_EXC:
        return percentile_exc(xs, k)
    if function_num == QUARTILE_EXC:
        return percentile_exc(xs, quart_k(k, 1, 3))
    # Unreachable through `evaluate` (the `Interval(14, 19)` check raises first); a direct call outside 14-19 is
    # Excel's `#VALUE!` region (numbers.md "AGGREGATE"), so it is a domain error too.
    msg = f"function_num {function_num} is not an array form"
    raise DomainError(msg)


# -- draws -------------------------------------------------------------------------------------


def array_of(
    rng: random.Random, count: int, lo: float, hi: float, negative: float = 0.3
) -> tuple[float, ...]:
    """`count` signed log-uniform magnitudes in [lo, hi], each written to 15 digits."""
    return tuple(round15(signed_log_uniform(rng, lo, hi, negative)) for _ in range(count))


def _decimals(rng: random.Random, lo: float, hi: float) -> float:
    """A k in [lo, hi] with 1 to 15 significant digits (round numbers such as 0.3 are common in sheets)."""
    return round(rng.uniform(lo, hi), rng.choice((1, 2, 3, 15)))


def valid_k(rng: random.Random, function_num: int, n: int) -> float:
    """A valid k for `function_num` over n values, away from the bounds Excel compares in doubles."""
    if function_num in (LARGE, SMALL):
        whole = rng.randint(1, n)
        if rng.random() < 0.7 or whole == n:
            return float(whole)
        return round15(rng.uniform(whole + 0.001, whole + 0.999))  # rounds to `whole + 1` or `whole`
    if function_num == PERCENTILE_INC:
        return min(1.0, max(0.0, _decimals(rng, 0.0, 1.0)))
    if function_num == QUARTILE_INC:
        return float(rng.randint(0, 4)) + (0.0 if rng.random() < 0.8 else 0.5)
    if function_num == QUARTILE_EXC:
        return float(rng.randint(1, 3)) + (0.0 if rng.random() < 0.8 else 0.5)
    lo, hi = 1 / (n + 1), n / (n + 1)
    margin = (hi - lo) * 1e-6
    return min(hi - margin, max(lo + margin, round15(rng.uniform(lo + margin, hi - margin))))


def min_count(function_num: int) -> int:
    """The smallest count every valid k needs: PERCENTILE.EXC's rank needs 2, QUARTILE.EXC's quart 1 needs 3."""
    return {PERCENTILE_EXC: 2, QUARTILE_EXC: 3}.get(function_num, 1)


def valid_point(
    rng: random.Random, function_num: int, lo: float, hi: float, sizes: tuple[int, int]
) -> tuple[tuple[float, ...], float]:
    """An array of `sizes[0]` to `sizes[1]` magnitudes in [lo, hi] (at least `min_count`) and a valid k for
    `function_num` (shared with the order-statistics definitions, T-4092)."""
    n = rng.randint(max(sizes[0], min_count(function_num)), max(sizes[1], min_count(function_num)))
    array = array_of(rng, n, lo, hi)
    return array, valid_k(rng, function_num, n)


def _point(rng: random.Random, lo: float, hi: float, sizes: tuple[int, int]) -> list[Arg]:
    function_num = rng.randint(LARGE, QUARTILE_EXC)
    array, k = valid_point(rng, function_num, lo, hi, sizes)
    return [function_num, rng.randint(0, 7), array, k]


def _near_bound(rng: random.Random) -> list[Arg]:
    """k near the bounds that are not compared in doubles, and arrays straddling 0 (cancelling interpolation)."""
    n = rng.randint(2, 12)
    function_num = rng.choice((LARGE, SMALL, PERCENTILE_INC, PERCENTILE_EXC))
    array, k = near_bound_point(rng, function_num, n)
    return [function_num, rng.randint(0, 7), array, k]


def near_bound_point(rng: random.Random, function_num: int, n: int) -> tuple[tuple[float, ...], float]:
    """`n` values straddling 0 (cancelling interpolation) and a k for `function_num` near the bounds that are not
    compared in doubles; QUARTILE.INC and QUARTILE.EXC (n >= 3) take a whole valid quart (T-4092)."""
    array = array_of(rng, n, 1e-3, 1e3, negative=0.5)
    if function_num in (LARGE, SMALL):
        whole = rng.randint(1, n - 1)
        k = whole + rng.choice((1e-10, 1e-6, 0.5, 1 - 1e-6))  # LARGE takes whole + 1, SMALL whole
    elif function_num == PERCENTILE_INC:
        tiny = 10.0 ** rng.uniform(-6, -2)
        k = rng.choice((tiny, 1 - tiny, 0.5 + tiny))
    elif function_num == QUARTILE_INC:
        k = float(rng.randint(0, 4))
    elif function_num == QUARTILE_EXC:
        k = float(rng.randint(1, 3))
    else:
        lo, hi = 1 / (n + 1), n / (n + 1)
        k = rng.choice((lo + (hi - lo) * 1e-5, hi - (hi - lo) * 1e-5, rng.uniform(lo, hi)))
    return array, round15(k)


def _edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(1, 8)
    array = array_of(rng, n, 1e-3, 1e6)
    o = rng.randint(0, 7)
    three = (1.0, 3.0, 2.0)  # n + 1 = 4: the EXC bounds 0.25 and 0.75 are exact doubles
    seven = tuple(float(x) for x in (5, 2, 8, 1, 9, 3, 7))  # n + 1 = 8: 0.125 and 0.875 exact
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [rng.choice((LARGE, SMALL)), o, array, rng.choice((0.0, 0.5, -1.0))],  # k below 1: #NUM!
        lambda: [
            rng.choice((LARGE, SMALL)),
            o,
            array,
            float(n) + rng.choice((0.5, 1.0)),
        ],  # ceil(k) > n: #NUM!
        lambda: [rng.choice((LARGE, SMALL)), o, array, float(n)],  # the extremes
        lambda: [PERCENTILE_INC, o, array, rng.choice((0.0, 1.0))],  # the extremes
        lambda: [PERCENTILE_INC, o, array, rng.choice((-0.1, 1.1, -1e-10))],  # #NUM!
        lambda: [PERCENTILE_EXC, o, array, rng.choice((0.0, 1.0, -0.5, 1.5))],  # #NUM!
        lambda: [PERCENTILE_EXC, o, (round15(rng.uniform(1, 9)),), 0.5],  # one value: rank 1 inside [1, 1]
        lambda: [PERCENTILE_EXC, o, array, round15(0.5 / (n + 1))],  # rank 0.5 below 1: #NUM!
        lambda: [rng.choice((PERCENTILE_EXC, QUARTILE_EXC)), o, three, rng.choice((0.25, 0.75, 1.0, 3.0))],
        lambda: [PERCENTILE_EXC, o, seven, rng.choice((0.125, 0.875, 0.1, 0.9))],  # exact bounds and outside
        lambda: [QUARTILE_INC, o, array, rng.choice((0.0, 4.0, 4.5, 5.0, -1.0))],  # quart 0, 4 and #NUM!
        lambda: [QUARTILE_EXC, o, array, rng.choice((0.0, 0.5, 4.0, 2.0))],  # quart 0 and 4: #NUM!
        lambda: [QUARTILE_EXC, o, (1.0, 2.0), 2.0],  # n = 2: rank 0.75 below 1, #NUM!
        lambda: [rng.randint(LARGE, QUARTILE_EXC), o, (round15(rng.uniform(-9, 9)),) * 5, 2.0],  # ties only
        lambda: [
            rng.choice((PERCENTILE_INC, PERCENTILE_EXC)),
            o,
            (3.0, 1.0, 4.0, 1.0, 5.0, 9.0),
            0.3,
        ],  # H43, H45
    ]
    return one_of(rng, shapes)


DRAW = by_stratum(
    {
        "typical": lambda rng: _point(rng, 1e-3, 1e6, (1, 12)),
        "tail": _near_bound,
        "edge": _edge,
        "extreme": lambda rng: _point(rng, 1e-100, 1e100, (20, 150)),
        "log-uniform": lambda rng: _point(rng, 1e-150, 1e150, (1, 8)),
    },
    "AGGREGATE 14-19 draws: function_num, options, an array and a valid k by stratum; the edge stratum adds the "
    "domain errors and the bounds that are exact doubles.",
)

AGGREGATE = register(
    Definition(
        function="AGGREGATE",
        params=(
            Param(
                "function_num", "int", Interval(14, 19, lo_open=False, hi_open=False), source=AGGREGATE_SOURCE
            ),
            Param("options", "int", Interval(0, 7, lo_open=False, hi_open=False), source=AGGREGATE_SOURCE),
            Param("array", "array", source=AGGREGATE_SOURCE),
            Param("k", "real", source=K_FIRST_SOURCE),
        ),
        value=_value,
        domain=domain_of(_value),
        draw=DRAW,
        literal_arrays=True,
        sources=(
            AGGREGATE_SOURCE,
            K_FIRST_SOURCE,
            LARGE_SMALL_SOURCE,
            PERCENTILE_SOURCE,
            DOMAIN_SOURCE,
            EVIDENCE,
            "decision 0228",
        ),
    )
)
