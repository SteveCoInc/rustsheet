"""FORECAST, FORECAST.LINEAR, INTERCEPT, SLOPE, RSQ, PEARSON, CORREL, COVAR, COVARIANCE.P, COVARIANCE.S and STEYX
(Gate A paired regression, T-4094 and T-4095, decision 0271).

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

COVAR, COVARIANCE.P, COVARIANCE.S and STEYX (T-4095) use the same moments and draws:
- COVAR and COVARIANCE.P = Sxy / n, COVARIANCE.S = Sxy / (n - 1); STEYX = sqrt((Syy - Sxy^2 / Sxx) / (n - 2))
  (statistics.md "Paired regression functions", the order Excel evaluates in).
- Domain: unequal lengths `#N/A`; one pair is 0 for COVAR and COVARIANCE.P ("Array arguments given a scalar, a single
  cell or an error": COVAR of one pair is 0; probes/t2436/computed-scalar A29 `COVARIANCE.P(4,5)`) and `#DIV/0!` for
  COVARIANCE.S (A30); a constant series gives the covariances 0 (pairs-b L6, L7). STEYX: x's overflow `#NUM!` before
  its fewer-than-three-pairs `#DIV/0!` (Z69), a constant x `#DIV/0!`, y's sum of squares overflowing `#NUM!` with four
  or more pairs (T-4047 H4), a constant y 0, and `#NUM!` where Excel's double Syy - Sxy*Sxy/Sxx is below zero ("STEYX on
  an exact fit"; with Sxy*Sxy overflowing too, T-4047 J4).
- No math value (unprobed): for the covariances a sum of the values or Sxy beyond the double range (their sums about
  one mean overflowing is evidenced finite: "COVARIANCE.P is finite"); for STEYX y's overflow with two or three pairs
  or with a constant x, y's own sum or Sxy beyond the double range, and a subnormal Sxy^2, explained part or residual.
- STEYX draws at least three pairs outside the edge stratum, keeps its tail lines at noise 1E-6 or more (nearer, the
  residual is only Excel's rounding), and adds exact decimal lines (probes/steyx-exact-fit) and data near 1E+160 a
  relative 1E-12 to 1E-9 apart (Sxy^2 overflows) to the edges.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from fractions import Fraction
from functools import partial

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    StratumDraws,
    below_normal,
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
COVARIANCE_SOURCE = "statistics.md#covariance-a-boolean-scalar-is-one-value-t-2436"
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
    x_sum_overflows: bool
    y_sum_overflows: bool
    doubles: tuple[
        float, float, float
    ]  # Excel's double sxx, syy and sxy (STEYX's residual decides its #NUM!)


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
        if below_normal(dx * dx, dy * dy, dx * dy):
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
    if below_normal(*xs, *ys):
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
        x_sum_overflows=not math.isfinite(sum_x),
        y_sum_overflows=not math.isfinite(sum_y),
        doubles=(sxx_d, syy_d, sxy_d),
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


def _covariance(args: Sequence[MathArg], *, sample: bool) -> mpmath.mpf:
    """COVAR and COVARIANCE.P (Sxy / n) or COVARIANCE.S (Sxy / (n - 1)), with their errors."""
    m = moments(args)
    if m.x_sum_overflows or m.y_sum_overflows or m.sxy_overflows:
        msg = "a sum of the values or of the deviation products beyond the double range (unprobed)"
        raise NoValueError(msg)
    if sample and m.n < 2:
        msg = "#DIV/0!: COVARIANCE.S of one pair"
        raise DomainError(msg)
    for series in _series(args):
        # called for its NoValueError only: a constant whose double mean is not its value has no evidenced result,
        # while an exact constant needs no special case (its Sxy is 0)
        constant_exact(series)
    return fraction_to_mpf(m.sxy / (m.n - 1 if sample else m.n))


def _steyx(args: Sequence[MathArg]) -> mpmath.mpf:
    """STEYX = sqrt((Syy - Sxy^2 / Sxx) / (n - 2)), with its errors in Excel's order (see the module doc)."""
    m = moments(args)
    if m.x_overflows:
        msg = "#NUM!: the sum about x's mean is beyond the double range"
        raise DomainError(msg)
    if m.y_overflows and (m.n <= 3 or m.y_sum_overflows):
        msg = "y's sums beyond the double range with two or three pairs, or y's sum itself (unprobed)"
        raise NoValueError(msg)
    if m.n < 3:
        msg = "#DIV/0!: fewer than three pairs"
        raise DomainError(msg)
    if _x_constant(args, m):
        if m.y_overflows:
            msg = "a constant x with y's sum about its mean beyond the double range (unprobed order)"
            raise NoValueError(msg)
        msg = "#DIV/0!: a constant x"
        raise DomainError(msg)
    if m.y_overflows:
        msg = "#NUM!: the sum about y's mean is beyond the double range (T-4047)"
        raise DomainError(msg)
    if m.sxy_overflows:
        msg = "the sum of the deviation products beyond the double range (unprobed)"
        raise NoValueError(msg)
    if _y_constant(args):
        return mpmath.mpf(0)
    sxx, syy, sxy = m.doubles
    square = sxy * sxy
    explained = square / sxx
    residual = (syy - explained) / (m.n - 2)
    if residual < 0:  # also -inf where Sxy^2 overflows (T-4047, near 1E+160)
        msg = "#NUM!: Excel's double Syy - Sxy^2/Sxx is below zero"
        raise DomainError(msg)
    if below_normal(square, explained, residual):
        msg = "a subnormal intermediate of the residual (unprobed flush)"
        raise NoValueError(msg)
    return mpmath.sqrt(fraction_to_mpf((m.syy - m.sxy * m.sxy / m.sxx) / (m.n - 2)))


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


def _typical(rng: random.Random, low: int = 2) -> list[Arg]:
    n = rng.randint(low, 10)
    if rng.random() < 0.5:
        return _pairs(rng, n, 1e-3, 1e6)
    return list(_noisy_line(rng, n, log_uniform(rng, 1e-3, 1.0)))


def offset_series(
    rng: random.Random,
    n: int,
    offsets: tuple[float, float] = (1.0, 1e10),
    spreads: tuple[float, float] = (1e-9, 1e-1),
) -> tuple[float, ...]:
    """A large offset plus a small relative spread, written to 15 digits (the paired entry's I1:I6, 10000000001..)."""
    offset = signed_log_uniform(rng, *offsets, 0.5)
    spread = abs(offset) * log_uniform(rng, *spreads)
    return tuple(round15(offset + spread * rng.uniform(-1, 1)) for _ in range(n))


def _tail(rng: random.Random, noise: float = 1e-12) -> list[Arg]:
    n = rng.randint(3, 12)
    shape = rng.randrange(3)
    if shape == 0:  # both offset
        return [offset_series(rng, n), offset_series(rng, n)]
    if shape == 1:  # one offset, against a plain series
        plain = array_of(rng, n, 1e-3, 1e6)
        return [offset_series(rng, n), plain] if rng.random() < 0.5 else [plain, offset_series(rng, n)]
    return list(_noisy_line(rng, n, log_uniform(rng, noise, 1e-4)))  # near-exact lines: r near +-1


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


def _log_uniform(rng: random.Random, low: int = 2) -> list[Arg]:
    return _pairs(rng, rng.randint(low, 6), 1e-150, 1e150)


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


def _decimal_line(rng: random.Random) -> list[Arg]:
    """y = a + d*i over x = 1..n with one-decimal a and d (probes/steyx-exact-fit): an exact fit whose double residual
    is below zero (`#NUM!`), above it (a tiny number) or 0."""
    n = rng.randint(3, 11)
    a, d = rng.randint(-30, 30) / 10, rng.choice([i for i in range(-5, 6) if i != 0]) / 10
    return [tuple(round15(a + d * i) for i in range(1, n + 1)), tuple(float(i) for i in range(1, n + 1))]


def _near_1e160(rng: random.Random, n: int) -> tuple[float, ...]:
    """Values near 1E+160 a relative 1E-12 to 1E-9 apart: the sums about the means stay finite and Sxy^2 overflows."""
    return offset_series(rng, n, (1e159, 1e161), (1e-12, 1e-9))


def _steyx_edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(3, 8)
    return one_of(
        rng,
        [
            lambda: _edge(rng),  # the paired edges (one pair, unequal lengths, constants, overflows)
            lambda: _pairs(rng, 2, 1e-3, 1e6),  # two pairs: #DIV/0!
            lambda: _decimal_line(rng),
            lambda: [_near_1e160(rng, n), _near_1e160(rng, n)],  # Sxy^2 overflows: #NUM! (T-4047 J4)
        ],
    )


_STEYX_DRAWS: StratumDraws = {
    "typical": lambda rng: _typical(rng, 3),
    "tail": lambda rng: _tail(rng, 1e-6),  # a nearer line leaves only Excel's rounding in the residual
    "edge": _steyx_edge,
    "extreme": _extreme,
    "log-uniform": lambda rng: _log_uniform(rng, 3),
}


_PAIR_DRAW = by_stratum(
    _DRAWS, "Paired draws: two equal-length arrays (y, x) by stratum; edge shapes from statistics.md."
)
_FORECAST_DRAW = _forecast_draws()
_STEYX_DRAW = by_stratum(
    _STEYX_DRAWS,
    "STEYX draws: the paired draws with at least three pairs, exact decimal lines and Sxy^2 overflow.",
)
SOURCES = (PAIRED_SOURCE, SIZES_SOURCE, SINGLE_SOURCE, BLANK_SOURCE, FLUSH_SOURCE, "decision 0271")


def _definition(
    function: str,
    names: tuple[str, str],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    *,
    forecast: bool = False,
    draw: Callable[[str, random.Random], list[Arg]] = _PAIR_DRAW,
    sources: tuple[str, ...] = (),
) -> Definition:
    arrays = tuple(Param(name, "array", source=PAIRED_SOURCE) for name in names)
    params = (Param("x", "real", source=LIFT_SOURCE), *arrays) if forecast else arrays
    return register(
        Definition(
            function=function,
            params=params,
            value=value,
            domain=domain_of(value),
            draw=_FORECAST_DRAW if forecast else draw,
            sources=(*SOURCES, LIFT_SOURCE) if forecast else (*SOURCES, *sources),
        )
    )


KNOWN = ("known_y's", "known_x's")
SLOPE = _definition("SLOPE", KNOWN, _slope_value)
INTERCEPT = _definition("INTERCEPT", KNOWN, _intercept)
FORECAST = _definition("FORECAST", KNOWN, _forecast, forecast=True)
FORECAST_LINEAR = _definition("FORECAST.LINEAR", KNOWN, _forecast, forecast=True)
RSQ = _definition("RSQ", KNOWN, _rsq)
_ARRAYS = ("array1", "array2")  # CORREL's, PEARSON's and the covariances' parameter names
CORREL = _definition("CORREL", _ARRAYS, _correl)
PEARSON = _definition("PEARSON", _ARRAYS, _correl)
COVAR = _definition("COVAR", _ARRAYS, partial(_covariance, sample=False), sources=(COVARIANCE_SOURCE,))
COVARIANCE_P = _definition(
    "COVARIANCE.P", _ARRAYS, partial(_covariance, sample=False), sources=(COVARIANCE_SOURCE,)
)
COVARIANCE_S = _definition(
    "COVARIANCE.S", _ARRAYS, partial(_covariance, sample=True), sources=(COVARIANCE_SOURCE,)
)
STEYX = _definition("STEYX", KNOWN, _steyx, draw=_STEYX_DRAW)
