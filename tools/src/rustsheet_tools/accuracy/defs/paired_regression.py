"""FORECAST, FORECAST.LINEAR, INTERCEPT, SLOPE, RSQ, PEARSON and CORREL (Gate A paired regression, T-4094, decision
0271).

The math, on the one-row array literals' doubles x[1..n] (known_x's, CORREL's and PEARSON's array2) and y[1..n]
(known_y's, array1), with the exact means and the exact sums about them Sxx = sum((x - mx)^2), Syy = sum((y - my)^2)
and Sxy = sum((x - mx)(y - my)) (exact rationals, so no cancellation enters the math):
- SLOPE = Sxy / Sxx, INTERCEPT = my - SLOPE * mx;
- FORECAST and FORECAST.LINEAR at x = my + SLOPE * (x - mx) (the line through the means);
- CORREL and PEARSON r = Sxy / sqrt(Sxx * Syy), RSQ = Sxy^2 / (Sxx * Syy).
Excel computes these from the same two-pass sums in doubles, dividing by n (CORREL, PEARSON) or n - 1 (RSQ) before
the square roots, and FORECAST as INTERCEPT + SLOPE * x (statistics.md "Paired regression functions: single points,
zero variance, text and large offsets" and "CORREL, PEARSON and RSQ over arrays of different sizes, and their last
digit"); that moves the last digits (RSQ is not clamped to 1, `CORREL` of an array with itself is 1.0000000000000002),
and the math is the exact value. FORECAST's x is a scalar here: an array x lifts (statistics.md "RANK, FORECAST, PROB,
Z.TEST and T.TEST: lifted scalar arguments"), which an eval cell's fit leaves out.

Domain (an Excel error region, never a math value), each from those entries:
- arrays of different lengths are `#N/A` (all seven: "CORREL, PEARSON and RSQ over arrays of different sizes";
  "Paired functions pair two ranges of equal count whatever their shapes");
- one pair is `#DIV/0!` ("Array arguments given a scalar, a single cell or an error": `INTERCEPT(1.005,2.5)`,
  `SLOPE(123456789.123,<number>)`, `FORECAST.LINEAR(x,7,137.5)`, `RSQ` and the correlations);
- zero variance: x constant is `#DIV/0!` for all seven, y constant for CORREL, PEARSON and RSQ only (SLOPE is then 0,
  INTERCEPT and FORECAST the mean of y);
- overflow, checked before the zero variance but after the pair count: Excel's double sum about the mean of x
  (or the sum of x) beyond the double range is `#NUM!` for all seven, and that of y for CORREL, PEARSON and RSQ
  (T-4047: with y of 1E+200 scale SLOPE, INTERCEPT and FORECAST are finite). The sums are the two-pass double sums
  the entry fits to Excel's bits (and the engine's `PairedMoments`).
Given no math value (`NoValueError`), as no evidence names Excel's result:
- a constant series whose double mean is not its value (Excel's deviations are then a nonzero constant:
  `CORREL({0.1,0.1,0.1},{1,2,3})` is 0 where the math is `#DIV/0!`; "Inexact constants");
- a subnormal literal, or a nonzero deviation square or product below the normal range (Excel flushes subnormal
  results, numbers.md "Subnormal results become 0"; unprobed for these functions);
- for SLOPE, INTERCEPT and FORECAST: the sum of y or the double sum Sxy beyond the double range (unprobed; T-4047
  covers y's sum of squares only).
The draws keep magnitudes at or below 1E+150 but for the edge stratum's evidenced overflow shapes.

Draws (two equal-length arrays; FORECAST's x first): typical, 2 to 10 pairs, half of them near a line with noise;
tail, large offsets with a small spread (statistics.md: `CORREL(I1:I6,B1:B6)` over 10000000001.. is 0.9402561526801988)
and near-exact lines (RSQ's and CORREL's last digit); edge, Microsoft's examples, one pair, unequal lengths, constant
series with exact means, exact lines, the overflow shapes; extreme, 10 to 120 pairs of mixed magnitudes; log-uniform,
2 to 6 pairs from 1E-150 to 1E+150. The arrays reuse AGGREGATE's `array_of` (T-3693).
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    StratumDraws,
    by_stratum,
    domain_of,
    fraction_to_mpf,
    one_of,
    scalars,
    signed_log_uniform,
)
from rustsheet_tools.accuracy.defs.aggregate import running_total
from rustsheet_tools.accuracy.defs.aggregate_array import array_of
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

PAIRED_SOURCE = "statistics.md#paired-regression-functions-single-points-zero-variance-text-and-large-offsets"
SIZES_SOURCE = "statistics.md#correl-pearson-and-rsq-over-arrays-of-different-sizes-and-their-last-digit"
SINGLE_SOURCE = "statistics.md#array-arguments-given-a-scalar-a-single-cell-or-an-error"
LIFT_SOURCE = "statistics.md#rank-forecast-prob-ztest-and-ttest-lifted-scalar-arguments"
BLANK_SOURCE = (
    "statistics.md#a-one-cell-reference-to-an-empty-cell-is-value-in-the-paired-functions-and-ftest-t-2255"
)
FLUSH_SOURCE = "numbers.md#subnormal-results-become-0"
MAX_PAIRS = 120  # two arrays of 120 literals stay well inside Excel's 8192-character formula
MICROSOFT_Y = (2.0, 3.0, 9.0, 1.0, 8.0, 7.0, 5.0)  # Microsoft's SLOPE, INTERCEPT and RSQ example
MICROSOFT_X = (6.0, 5.0, 11.0, 7.0, 5.0, 4.0, 4.0)


@dataclass(frozen=True)
class Moments:
    """The exact means and sums about them of the pairs, and the domain flags of Excel's double two-pass sums."""

    n: int
    mean_x: Fraction
    mean_y: Fraction
    sxx: Fraction
    syy: Fraction
    sxy: Fraction
    x_overflows: bool  # Excel's double sum of x, or about x's mean, is beyond the double range
    y_overflows: bool
    sxy_overflows: bool
    y_sum_overflows: bool


def _double_sums(xs: Sequence[float], ys: Sequence[float]) -> tuple[float, float, float, float, float]:
    """(sum of x, sum of y, sxx, syy, sxy) in doubles, two passes about the double means, as Excel sums them (the paired
    entry: this order fits Excel's bits; the sums of the values are Excel's left-to-right `running_total`); NoValueError
    where a nonzero deviation square or product is subnormal."""
    n = len(xs)
    sum_x, sum_y = running_total(xs)[1], running_total(ys)[1]
    mean_x, mean_y = sum_x / n, sum_y / n
    sxx = syy = sxy = 0.0
    for x, y in zip(xs, ys, strict=True):
        dx, dy = x - mean_x, y - mean_y
        for d in (dx * dx, dy * dy, dx * dy):
            if d != 0 and abs(d) < sys.float_info.min:
                msg = "a deviation square or product below the normal range (unprobed flush)"
                raise NoValueError(msg)
        sxx += dx * dx
        syy += dy * dy
        sxy += dx * dy
    return sum_x, sum_y, sxx, syy, sxy


def constant_exact(values: Sequence[float]) -> bool:
    """Whether a series is constant with Excel's double mean equal to its value (else NoValueError for a constant
    series: its double deviations are a nonzero constant, "Inexact constants")."""
    if len(set(values)) > 1:
        return False
    if running_total(values)[1] / len(values) != values[0]:
        msg = "a constant series whose double mean is not its value"
        raise NoValueError(msg)
    return True


def _series(args: Sequence[MathArg]) -> tuple[list[float], list[float]]:
    """(y, x): the last two arguments' doubles (known_y's then known_x's; CORREL's array1 then array2)."""
    out: list[list[float]] = []
    for a in args[-2:]:
        assert isinstance(a, tuple)
        out.append([float(v) for v in scalars(a)])
    ys, xs = out
    return ys, xs


def moments(args: Sequence[MathArg]) -> Moments:
    """The pairs' exact moments (y is the first array, x the second); DomainError (`#N/A`) for unequal lengths."""
    ys, xs = _series(args)
    if len(ys) != len(xs):
        msg = "#N/A: the arrays differ in length"
        raise DomainError(msg)
    if any(v != 0 and abs(v) < sys.float_info.min for v in xs + ys):
        msg = "a subnormal literal (Excel reads it as 0; unprobed here)"
        raise NoValueError(msg)
    sum_x, sum_y, sxx_d, syy_d, sxy_d = _double_sums(xs, ys)
    n = len(xs)
    fx, fy = [Fraction(v) for v in xs], [Fraction(v) for v in ys]
    mean_x, mean_y = sum(fx, Fraction(0)) / n, sum(fy, Fraction(0)) / n
    return Moments(
        n=n,
        mean_x=mean_x,
        mean_y=mean_y,
        sxx=sum(((v - mean_x) ** 2 for v in fx), Fraction(0)),
        syy=sum(((v - mean_y) ** 2 for v in fy), Fraction(0)),
        sxy=sum(((a - mean_x) * (b - mean_y) for a, b in zip(fx, fy, strict=True)), Fraction(0)),
        x_overflows=n >= 2 and not (math.isfinite(sum_x) and math.isfinite(sxx_d)),
        y_overflows=n >= 2 and not (math.isfinite(sum_y) and math.isfinite(syy_d)),
        sxy_overflows=not math.isfinite(sxy_d),
        y_sum_overflows=not math.isfinite(sum_y),
    )


def _x_constant(args: Sequence[MathArg], m: Moments) -> bool:
    return m.n < 2 or constant_exact(_series(args)[1])


def _y_constant(args: Sequence[MathArg]) -> bool:
    return constant_exact(_series(args)[0])


def _slope(args: Sequence[MathArg]) -> tuple[Moments, Fraction]:
    """The pairs' moments and the exact slope, with SLOPE's, INTERCEPT's and FORECAST's errors."""
    m = moments(args)
    if m.x_overflows:
        msg = "#NUM!: the sum about x's mean is beyond the double range"
        raise DomainError(msg)
    if _x_constant(args, m):
        msg = "#DIV/0!: one pair or a constant x"
        raise DomainError(msg)
    if m.y_sum_overflows or m.sxy_overflows:
        msg = "the sum of y or of the deviation products beyond the double range (unprobed)"
        raise NoValueError(msg)
    if _y_constant(args):
        return m, Fraction(0)
    return m, m.sxy / m.sxx


def _slope_value(args: Sequence[MathArg]) -> mpmath.mpf:
    return fraction_to_mpf(_slope(args)[1])


def _intercept(args: Sequence[MathArg]) -> mpmath.mpf:
    m, slope = _slope(args)
    return fraction_to_mpf(m.mean_y - slope * m.mean_x)


def _forecast(args: Sequence[MathArg]) -> mpmath.mpf:
    m, slope = _slope(args)
    (x,) = scalars(args[:1])
    return fraction_to_mpf(m.mean_y + slope * (Fraction(float(x)) - m.mean_x))


def _r_squared(args: Sequence[MathArg]) -> tuple[Moments, Fraction]:
    """The pairs' moments and the exact r^2, with CORREL's, PEARSON's and RSQ's errors."""
    m = moments(args)
    if m.x_overflows or m.y_overflows:
        msg = "#NUM!: a sum about a mean is beyond the double range"
        raise DomainError(msg)
    x_constant = _x_constant(args, m)
    if x_constant or _y_constant(args):
        msg = "#DIV/0!: one pair or a constant series"
        raise DomainError(msg)
    return m, m.sxy * m.sxy / (m.sxx * m.syy)


def _correl(args: Sequence[MathArg]) -> mpmath.mpf:
    m, r2 = _r_squared(args)
    magnitude = mpmath.sqrt(fraction_to_mpf(r2))
    return magnitude if m.sxy >= 0 else -magnitude


def _rsq(args: Sequence[MathArg]) -> mpmath.mpf:
    return fraction_to_mpf(_r_squared(args)[1])


# -- draws -------------------------------------------------------------------------------------


def _noisy_line(rng: random.Random, n: int, noise: float) -> tuple[tuple[float, ...], tuple[float, ...]]:
    """(y, x): x drawn as AGGREGATE's arrays, y = a + b*x plus Gaussian noise `noise` times y's spread."""
    xs = array_of(rng, n, 1e-3, 1e6)
    a, b = signed_log_uniform(rng, 1e-3, 1e4, 0.5), signed_log_uniform(rng, 1e-4, 1e4, 0.5)
    clean = [a + b * x for x in xs]
    spread = (max(clean) - min(clean)) or abs(a) or 1.0
    return tuple(round15(c + noise * spread * rng.gauss(0, 1)) for c in clean), xs


def _pairs(rng: random.Random, n: int, lo: float, hi: float) -> list[Arg]:
    return [array_of(rng, n, lo, hi), array_of(rng, n, lo, hi)]


def _typical(rng: random.Random) -> list[Arg]:
    n = rng.randint(2, 10)
    if rng.random() < 0.5:
        return _pairs(rng, n, 1e-3, 1e6)
    return list(_noisy_line(rng, n, log_uniform(rng, 1e-3, 1.0)))


def offset_series(rng: random.Random, n: int) -> tuple[float, ...]:
    """A large offset plus a small spread, written to 15 digits (the paired entry's I1:I6, 10000000001..)."""
    offset = signed_log_uniform(rng, 1.0, 1e10, 0.5)
    spread = abs(offset) * log_uniform(rng, 1e-9, 1e-1)
    return tuple(round15(offset + spread * rng.uniform(-1, 1)) for _ in range(n))


def _tail(rng: random.Random) -> list[Arg]:
    n = rng.randint(3, 12)
    shape = rng.randrange(3)
    if shape == 0:  # both offset
        return [offset_series(rng, n), offset_series(rng, n)]
    if shape == 1:  # one offset, against a plain series
        plain = array_of(rng, n, 1e-3, 1e6)
        return [offset_series(rng, n), plain] if rng.random() < 0.5 else [plain, offset_series(rng, n)]
    return list(_noisy_line(rng, n, log_uniform(rng, 1e-12, 1e-4)))  # near-exact lines: r near +-1


def _exact_line(rng: random.Random) -> list[Arg]:
    """Small integers on an exact line (r is exactly +-1; Excel's last digit may not be)."""
    n = rng.randint(2, 10)
    a, b = rng.randint(-20, 20), rng.choice([i for i in range(-5, 6) if i != 0])
    xs = tuple(float(rng.randint(-20, 20)) for _ in range(n))
    return [tuple(float(a + b * x) for x in xs), xs]


def constant_series(rng: random.Random, n: int) -> tuple[float, ...]:
    """A constant series with an exact double mean (a small integer or a multiple of 1/8)."""
    return (rng.randint(-160, 160) / 8,) * n


def _edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(2, 8)
    return one_of(
        rng,
        [
            lambda: [MICROSOFT_Y, MICROSOFT_X],  # Microsoft's example
            lambda: _pairs(rng, 1, 1e-3, 1e6),  # one pair: #DIV/0!
            lambda: [array_of(rng, n + 1, 1e-3, 1e6), array_of(rng, n, 1e-3, 1e6)],  # #N/A: lengths differ
            lambda: [array_of(rng, n, 1e-3, 1e6), constant_series(rng, n)],  # x constant: #DIV/0!
            lambda: [
                constant_series(rng, n),
                array_of(rng, n, 1e-3, 1e6),
            ],  # y constant: SLOPE 0, CORREL #DIV/0!
            lambda: _exact_line(rng),
            lambda: _pairs(rng, 2, 1e-3, 1e6),  # two pairs: r is +-1
            lambda: [array_of(rng, n, 1e-3, 1e6), array_of(rng, n, 1e199, 1e201)],  # x overflows: #NUM!
            lambda: [array_of(rng, n, 1e199, 1e201), array_of(rng, n, 1e-3, 1e6)],  # y overflows (T-4047)
            lambda: [array_of(rng, 2, 1e-3, 1e6), (_huge(rng), _huge(rng))],  # the sum of x overflows: #NUM!
        ],
    )


def _huge(rng: random.Random) -> float:
    """A value whose double sum with another such overflows (T-4047: x near 1.5E+308)."""
    return round15(rng.uniform(1.4e308, 1.7e308))


def _extreme(rng: random.Random) -> list[Arg]:
    return _pairs(rng, rng.randint(10, MAX_PAIRS), 1e-20, 1e20)


def _log_uniform(rng: random.Random) -> list[Arg]:
    return _pairs(rng, rng.randint(2, 6), 1e-150, 1e150)


_DRAWS: StratumDraws = {
    "typical": _typical,
    "tail": _tail,
    "edge": _edge,
    "extreme": _extreme,
    "log-uniform": _log_uniform,
}


def _forecast_x(rng: random.Random, xs: Sequence[float]) -> float:
    """FORECAST's x: inside the x data's range mostly, at a data point or the mean, or extrapolated."""
    lo, hi = min(xs), max(xs)
    width = (hi - lo) or abs(lo) or 1.0
    shape = rng.randrange(5)
    if shape == 0:
        return rng.choice(xs)
    if shape == 1:
        return round15(sum(xs) / len(xs))
    if shape == 2:  # extrapolated
        return round15(hi + width * log_uniform(rng, 1e-2, 1e3) * rng.choice((-1, 1)))
    return round15(rng.uniform(lo, hi))


def _forecast_draws() -> Callable[[str, random.Random], list[Arg]]:
    def with_x(draw: Callable[[random.Random], list[Arg]]) -> Callable[[random.Random], list[Arg]]:
        def drawn(rng: random.Random) -> list[Arg]:
            args = draw(rng)
            known_x = args[1]
            assert isinstance(known_x, tuple)
            xs = [v for v in known_x if isinstance(v, (int, float))]
            return [_forecast_x(rng, xs), *args]

        return drawn

    return by_stratum(
        {name: with_x(draw) for name, draw in _DRAWS.items()},
        "FORECAST draws: x near the known x's, then the paired draws (known_y's, known_x's).",
    )


_PAIR_DRAW = by_stratum(
    _DRAWS, "Paired draws: two equal-length arrays (y, x) by stratum; edge shapes from statistics.md."
)
_FORECAST_DRAW = _forecast_draws()
SOURCES = (PAIRED_SOURCE, SIZES_SOURCE, SINGLE_SOURCE, BLANK_SOURCE, FLUSH_SOURCE, "decision 0271")


def _definition(
    function: str,
    names: tuple[str, str],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    *,
    forecast: bool = False,
) -> Definition:
    arrays = tuple(Param(name, "array", source=PAIRED_SOURCE) for name in names)
    params = (Param("x", "real", source=LIFT_SOURCE), *arrays) if forecast else arrays
    return register(
        Definition(
            function=function,
            params=params,
            value=value,
            domain=domain_of(value),
            draw=_FORECAST_DRAW if forecast else _PAIR_DRAW,
            sources=(*SOURCES, LIFT_SOURCE) if forecast else SOURCES,
        )
    )


KNOWN = ("known_y's", "known_x's")
SLOPE = _definition("SLOPE", KNOWN, _slope_value)
INTERCEPT = _definition("INTERCEPT", KNOWN, _intercept)
FORECAST = _definition("FORECAST", KNOWN, _forecast, forecast=True)
FORECAST_LINEAR = _definition("FORECAST.LINEAR", KNOWN, _forecast, forecast=True)
RSQ = _definition("RSQ", KNOWN, _rsq)
CORREL = _definition("CORREL", ("array1", "array2"), _correl)
PEARSON = _definition("PEARSON", ("array1", "array2"), _correl)
