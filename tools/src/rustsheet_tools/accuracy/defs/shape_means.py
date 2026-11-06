"""Gate A shape and means (T-4091, decision 0271): KURT, SKEW, SKEW.P, GEOMEAN, HARMEAN, AVERAGEA and TRIMMEAN.

The math, on the written literals' doubles x[1..n], with the exact mean m and the exact sums
S_p = sum((x[i] - m)^p) (`deviation_power_sum`, the dispersion module's DEVSQ kernel):
- SKEW is n / ((n - 1)(n - 2)) * S_3 / s^3 with s^2 = S_2 / (n - 1); SKEW.P is (S_3 / n) / (S_2 / n)^(3/2); KURT is
  n(n + 1) / ((n - 1)(n - 2)(n - 3)) * S_4 / s^4 - 3(n - 1)^2 / ((n - 2)(n - 3)) (Microsoft's SKEW, SKEW.P and KURT
  pages). Excel forms them in doubles from a double mean; the math is the exact value.
- GEOMEAN is the exact n-th root of the exact product, (x[1] ... x[n])^(1/n). Excel's root is within an ulp or so of
  it in range and in its over/underflow path (numbers.md "GEOMEAN in-range root", "GEOMEAN root rounding").
- HARMEAN is the exact n / sum(1 / x[i]). Excel inverts the average of the twice-rounded reciprocals (statistics.md
  "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP"), which moves the last digits.
- AVERAGEA over numbers is AVERAGE (aggregate.py, T-3549): the exact sum, or 0 where Excel's final addition is
  snapped, over n. AVERAGEA snaps its sum as AVERAGE does (numbers.md "Near-zero snapping in addition and
  subtraction": `AVERAGEA(0.1,0.2,-0.3)` is 0, probes/t3066-average-snap-forms D2). It differs from AVERAGE only in
  how text and booleans count (statistics.md "AVERAGEA, MAXA and MINA over an array"), and the sampler draws numbers.
- TRIMMEAN(array, percent) drops k = floor(n * percent / 2) values from each end of the sorted data and is the
  exact mean of the n - 2k values kept (Microsoft's TRIMMEAN page: the count excluded is rounded down to a multiple
  of 2).

Domain (an Excel error region, never a math value):
- SKEW and SKEW.P of fewer than three numbers, and KURT of fewer than four, are `#DIV/0!` (statistics.md "SKEW.P of
  two numbers is #DIV/0!"; "SKEW, SKEW.P, KURT, HARMEAN ... edge pools": `KURT(A1:A3)` is `#DIV/0!`, `KURT(A1:A4)` a
  number, probes/stats-evidence-floor-t1623 Q29, Q28). The count is checked before an overflowing sum of the
  numbers; that order is unprobed.
- The three moments of equal numbers are `#DIV/0!` (`SKEW.P(5,5,5)`, `SKEW(N1:N3)` over 2, 2, 2, and
  `KURT({0,0,0},{0,0,0},{0,0,0})`, gen-m9/f34f66fb7f81e673 A391: zero variance).
  That is evidenced where Excel's double sum of the numbers is exact (so its double mean is the number); where it
  is not, Excel's deviations from its mean are not all 0 and its result is unprobed, so such a point gets no value.
- The three moments are `#DIV/0!` where one of Excel's double squared deviations overflows on its own, every
  deviation finite (statistics.md "Overflowing squares (T-2432)": `{1E+200,1E+200,-1E+200}`). Unprobed, and given no
  value: an overflowing sum of the numbers, a deviation x - mean that itself overflows, and finite squares whose sum
  overflows. The cubes and fourth powers in S_3 and S_4 may pass the double range with no error: Excel gives finite
  values there (statistics.md "KURT, SKEW, SKEW.P, GEOMEAN, HARMEAN and TRIMMEAN domain cells of the Gate A tables").
- GEOMEAN of a number at or below 0 is `#NUM!` (`GEOMEAN(4,)`, coercion.md "AVERAGE, MEDIAN, GEOMEAN, HARMEAN, SKEW,
  KURT, STDEV/VAR, COUNT value arguments"; ironcalc-calc/STATISTICAL/GEOMEAN.xlsx D11 and D16 over -4, 3 and 2).
  HARMEAN is `#NUM!` with a 0 among the numbers or with only negative numbers (accuracy/HARMEAN/s1 A71; `HARMEAN(0)`
  and `HARMEAN(-1,-2)`, probes/stats-evidence-floor-t1623 Q51, Q52); negative and positive numbers with no 0
  (`HARMEAN(1,-2)`) are unprobed and get no value.
- GEOMEAN's other `#NUM!`, a product of the numbers' [1, 2) significands beyond the double range, and its exponent
  wrap below the double range (numbers.md "GEOMEAN is #NUM! when the product of significands overflows") both need
  more than 1000 numbers, beyond the 255 arguments of a call, so they are not part of this definition.
- HARMEAN: a reciprocal or the sum of the reciprocals beyond the double range is unprobed and gets no value.
- AVERAGEA: a running total beyond the double range is unprobed for AVERAGEA (numbers.md's overflow entry lists
  AVERAGE, not AVERAGEA), so such a point gets no value.
- TRIMMEAN: a percent outside [0, 1) is `#NUM!` (statistics.md "TRIMMEAN of a typed scalar (T-3781)":
  `TRIMMEAN("2.5",2)`, `(..,-1)` and `(..,TRUE)`). Where Excel's double product n * percent and the exact one floor
  to different trim counts, Excel's choice is unprobed and the point gets no value. A sum of the kept values beyond
  the double range is unprobed too.

The table cells showing each rule are listed in statistics.md "KURT, SKEW, SKEW.P, GEOMEAN, HARMEAN and TRIMMEAN
domain cells of the Gate A tables (T-4091)" (`TABLES_SOURCE`), which every definition here cites.

Draws: the moments reuse the dispersion draws (T-4089: offset data where Excel's one-pass forms cancel, equal
numbers, one and two numbers and the argument limit; SKEW.P, a newer `_xlfn` function, takes at most 254 numbers).
GEOMEAN and HARMEAN take the magnitudes of the same draws, with edge shapes of their own (Microsoft's examples, a 0
or a negative number, products beyond the double range). AVERAGEA reuses SUM's draw (T-3549). TRIMMEAN reuses PERCENTILE.INC's
samplers (`valid_point`, `near_bound_point`, T-4092), its k in [0, 1] read as the percent.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    any_edge,
    by_stratum,
    by_stratum_with_edge,
    domain_of,
    scalars,
)
from rustsheet_tools.accuracy.defs.aggregate import (
    MAX_ARGS,
    SNAP_SOURCE,
    average_value,
    draw_numbers,
    overflow,
    running_total,
)
from rustsheet_tools.accuracy.defs.aggregate_array import (
    PERCENTILE_INC,
    array_of,
    near_bound_point,
    sorted_values,
    valid_point,
)
from rustsheet_tools.accuracy.defs.dispersion import (
    deviation_power_sum,
    variadic_definition,
    variance_draws,
)
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

SKEW_P_SOURCE = "statistics.md#skewp-of-two-numbers-is-div0"
POOLS_SOURCE = "statistics.md#skew-skewp-kurt-harmean-standardize-sumx-and-mdeterm-edge-pools-t-1623"
SQUARES_SOURCE = "statistics.md#overflowing-squares-t-2432"
HARMEAN_SOURCE = (
    "statistics.md#fisher-fisherinv-expondist-and-harmean-are-excel-operators-around-ln-and-exp-t-2437"
)
VALUE_ARGS_SOURCE = "coercion.md#average-median-geomean-harmean-skew-kurt-stdevvar-count-value-arguments"
GEOMEAN_ROOT_SOURCE = "numbers.md#geomean-in-range-root"
AVERAGEA_SOURCE = "statistics.md#averagea-maxa-and-mina-over-an-array"
TRIMMEAN_SOURCE = "statistics.md#trimmean-of-a-typed-scalar-t-3781"
TABLES_SOURCE = (
    "statistics.md#kurt-skew-skewp-geomean-harmean-and-trimmean-domain-cells-of-the-gate-a-tables-t-4091"
)
GEOMEAN_EVIDENCE = "ironcalc-calc/STATISTICAL/GEOMEAN.xlsx D11, D16"
POOLS_EVIDENCE = "probes/stats-evidence-floor-t1623"


# -- the moments ---------------------------------------------------------------------------------


def _moment_domain(least: int) -> Callable[[Sequence[MathArg]], str | None]:
    """The `#DIV/0!` rules of a moment of at least `least` numbers: too few numbers (checked first, before an
    overflowing sum of the numbers: that order is unprobed), equal numbers whose double sum is exact, or a squared
    deviation that overflows on its own (with the numbers' sum and every deviation in range)."""

    def domain(args: Sequence[MathArg]) -> str | None:
        xs = scalars(args)
        if len(xs) < least:
            return f"#DIV/0!: fewer than {least} numbers"
        if overflow(args) is not None:
            return None  # unprobed: `_moment_inputs` gives it no value
        if all(x == xs[0] for x in xs) and _exact_double_sum(xs):
            return "#DIV/0!: equal numbers (zero variance)"
        if _squares_overflow(xs) == EACH_SQUARE:
            return "#DIV/0!: a squared deviation is beyond the double range"
        return None

    return domain


EACH_SQUARE = "a squared deviation overflows on its own"
UNPROBED_SQUARES = "a deviation, or only the sum of the squared deviations, overflows"


def _squares_overflow(xs: Sequence[mpmath.mpf]) -> str | None:
    """How Excel's double squared deviations from its double mean overflow, for numbers whose sum is in range:
    `EACH_SQUARE` when a square overflows on its own with every deviation finite (the evidenced `#DIV/0!`,
    statistics.md "Overflowing squares (T-2432)": `{1E+200,1E+200,-1E+200}`), `UNPROBED_SQUARES` when a deviation
    itself overflows or every square is finite but their sum is not, else None."""
    doubles = [float(x) for x in xs]
    mean = running_total(doubles)[1] / len(doubles)
    deviations = [x - mean for x in doubles]
    if not all(math.isfinite(d) for d in deviations):
        return UNPROBED_SQUARES
    squares = [d * d for d in deviations]
    if not all(math.isfinite(q) for q in squares):
        return EACH_SQUARE
    return None if math.isfinite(running_total(squares)[1]) else UNPROBED_SQUARES


def _exact_double_sum(xs: Sequence[mpmath.mpf]) -> bool:
    """Whether Excel's left-to-right double sum of `xs` is the exact sum (then its double mean of equal numbers is the
    number, and every deviation is 0)."""
    _, total = running_total([float(x) for x in xs])
    return math.isfinite(total) and Fraction(total) == sum((Fraction(float(x)) for x in xs), Fraction(0))


def _moment_inputs(args: Sequence[MathArg]) -> tuple[int, Callable[[int], mpmath.mpf]]:
    """(n, p -> S_p) for a point the domain accepted; NoValueError for the unprobed regions."""
    xs = scalars(args)
    if overflow(args) is not None:
        msg = "the sum of the numbers is beyond the double range (unprobed for the moments)"
        raise NoValueError(msg)
    if all(x == xs[0] for x in xs):
        msg = "equal numbers whose double sum is inexact: Excel's deviations are not all 0 (unprobed)"
        raise NoValueError(msg)
    if _squares_overflow(xs) == UNPROBED_SQUARES:
        msg = f"{UNPROBED_SQUARES} (unprobed for the moments)"
        raise NoValueError(msg)
    mean = mpmath.fsum(xs) / len(xs)
    return len(xs), lambda p: deviation_power_sum(xs, p, mean)


def _skew(args: Sequence[MathArg]) -> mpmath.mpf:
    n, s = _moment_inputs(args)
    sd = mpmath.sqrt(s(2) / (n - 1))
    return mpmath.mpf(n) / ((n - 1) * (n - 2)) * s(3) / sd**3


def _skew_p(args: Sequence[MathArg]) -> mpmath.mpf:
    n, s = _moment_inputs(args)
    return (s(3) / n) / (s(2) / n) ** mpmath.mpf(1.5)


def _kurt(args: Sequence[MathArg]) -> mpmath.mpf:
    n, s = _moment_inputs(args)
    variance = s(2) / (n - 1)
    lead = mpmath.mpf(n * (n + 1)) / ((n - 1) * (n - 2) * (n - 3))
    return lead * s(4) / variance**2 - mpmath.mpf(3 * (n - 1) ** 2) / ((n - 2) * (n - 3))


# -- GEOMEAN and HARMEAN -------------------------------------------------------------------------


def _positive(xs: Sequence[mpmath.mpf]) -> None:
    if any(x <= 0 for x in xs):
        msg = "#NUM!: a number at or below 0"
        raise DomainError(msg)


def _harmean_positive(xs: Sequence[mpmath.mpf]) -> None:
    """HARMEAN's `#NUM!`, as far as it is evidenced: a 0 among the numbers (accuracy/HARMEAN/s1 A71, `HARMEAN(0)`) or
    numbers that are all negative (`HARMEAN(-1,-2)`). Negative and positive numbers with no 0 (`HARMEAN(1,-2)`) are
    unprobed: NoValueError."""
    if any(x == 0 for x in xs) or all(x < 0 for x in xs):
        msg = "#NUM!: a 0, or only negative numbers"
        raise DomainError(msg)
    if any(x < 0 for x in xs):
        msg = "negative and positive numbers with no 0 (unprobed for HARMEAN)"
        raise NoValueError(msg)


def _geomean(args: Sequence[MathArg]) -> mpmath.mpf:
    xs = scalars(args)
    _positive(xs)
    return mpmath.exp(mpmath.fsum(mpmath.log(x) for x in xs) / len(xs))


def _harmean(args: Sequence[MathArg]) -> mpmath.mpf:
    xs = scalars(args)
    _harmean_positive(xs)
    reciprocals = [1.0 / float(x) for x in xs]  # inf where it overflows
    if not math.isfinite(running_total(reciprocals)[1]):
        msg = "a reciprocal or the sum of the reciprocals is beyond the double range (unprobed)"
        raise NoValueError(msg)
    return len(xs) / mpmath.fsum(1 / x for x in xs)


# -- AVERAGEA and TRIMMEAN -----------------------------------------------------------------------


def _averagea(args: Sequence[MathArg]) -> mpmath.mpf:
    if overflow(args) is not None:
        msg = "a running total is beyond the double range (unprobed for AVERAGEA)"
        raise NoValueError(msg)
    return average_value(args)


def trim_count(n: int, percent: mpmath.mpf) -> int:
    """TRIMMEAN's k = floor(n * percent / 2); NoValueError where the double product n * percent floors to another k
    (Excel's choice there is unprobed)."""
    k = int(mpmath.floor(n * percent / 2))
    if math.floor(float(n) * float(percent) / 2) != k:
        msg = "the double n * percent floors to another trim count (unprobed)"
        raise NoValueError(msg)
    return k


def _trimmean(args: Sequence[MathArg]) -> mpmath.mpf:
    array, percent = args
    if not isinstance(percent, mpmath.mpf):
        msg = "percent must be a number"
        raise TypeError(msg)
    xs = sorted_values(array)
    if percent < 0 or percent >= 1:
        msg = "#NUM!: percent outside [0, 1)"
        raise DomainError(msg)
    n = len(xs)
    k = trim_count(n, percent)
    kept = xs[k : n - k]
    if not math.isfinite(running_total([float(x) for x in kept])[1]):
        msg = "the sum of the kept values is beyond the double range (unprobed)"
        raise NoValueError(msg)
    return mpmath.fsum(kept) / len(kept)


# -- draws ---------------------------------------------------------------------------------------

MS_DATA = (3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 5.0, 6.0, 4.0, 7.0)  # Microsoft's SKEW, SKEW.P and KURT example
MS_MEANS = (4.0, 5.0, 8.0, 7.0, 11.0, 4.0, 3.0)  # Microsoft's GEOMEAN and HARMEAN example
EXACT_ROOTS: tuple[tuple[float, ...], ...] = ((3.0, 3.0), (1.0, 9.0), (2.0, 8.0, 4.0))
MS_TRIMMEAN = (4.0, 5.0, 6.0, 7.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0)  # TRIMMEAN(.., 0.2) is 3.777777778


def _magnitudes(make: Callable[[str, random.Random], list[Arg]]) -> Callable[[str, random.Random], list[Arg]]:
    """`make`'s points with every number's magnitude: positive data for GEOMEAN and HARMEAN."""

    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        return [abs(x) for x in make(stratum, rng) if isinstance(x, float)]

    return draw


def _positive_edge(rng: random.Random) -> list[Arg]:
    def some(lo: float = 1e-3, hi: float = 1e6) -> list[Arg]:
        return [round15(log_uniform(rng, lo, hi)) for _ in range(rng.randint(2, 8))]

    def bad() -> list[Arg]:  # a 0 or a negative number among positive ones: #NUM!
        xs = some()
        xs[rng.randrange(len(xs))] = rng.choice((0.0, -round15(log_uniform(rng, 1e-3, 1e3))))
        return xs

    def huge() -> list[Arg]:  # a product beyond the double range: Excel's split path
        return [round15(log_uniform(rng, 1e100, 1e150)) for _ in range(rng.randint(3, 12))]

    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [round15(log_uniform(rng, 1e-3, 1e6))],  # one number: itself
        lambda: [round15(log_uniform(rng, 1e-3, 1e6))] * rng.randint(2, 8),  # equal numbers
        lambda: list(MS_MEANS),  # Microsoft: GEOMEAN 5.476986969, HARMEAN 5.028375962
        lambda: list(rng.choice(EXACT_ROOTS)),  # exact roots (numbers.md "GEOMEAN in-range root")
        bad,
        huge,
        lambda: [round15(log_uniform(rng, 1e-3, 1e6)) for _ in range(MAX_ARGS)],  # the argument limit
    ]
    return rng.choice(shapes)()


def _moment_draw(max_args: int) -> Callable[[str, random.Random], list[Arg]]:
    """The dispersion draws (T-4089), with Microsoft's example data added to the edge stratum."""
    base = variance_draws(max_args)
    return by_stratum_with_edge(
        base,
        any_edge([lambda rng: base("edge", rng), lambda _rng: list(MS_DATA)]),
        "The dispersion draws by stratum; the edge stratum adds Microsoft's SKEW/KURT example.",
    )


def _trimmean_edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(1, 12)
    array = array_of(rng, n, 1e-3, 1e6)
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [MS_TRIMMEAN, 0.2],  # Microsoft: 3.777777778
        lambda: [array, 0.0],  # nothing trimmed
        lambda: [array, rng.choice((0.999, 0.9999999))],  # all but the middle one or two
        lambda: [array, rng.choice((1.0, 1.5, -0.1, -1.0, 2.0))],  # #NUM!
        lambda: [(round15(rng.uniform(-9, 9)),), round15(rng.uniform(0, 0.99))],  # one value
        lambda: [(round15(rng.uniform(-9, 9)),) * 5, 0.4],  # ties only
        # a percent just above or below a trim-count step 2j/n (the floor moves k by one)
        lambda: [
            array,
            round15(min(0.99, 2 * rng.randint(1, max(1, n // 2)) / n * rng.choice((0.999, 1.001)))),
        ],
    ]
    return rng.choice(shapes)()


TRIMMEAN_DRAW = by_stratum(
    {
        "typical": lambda rng: list(valid_point(rng, PERCENTILE_INC, 1e-3, 1e6, (1, 30))),
        "tail": lambda rng: list(near_bound_point(rng, PERCENTILE_INC, rng.randint(2, 30))),
        "edge": _trimmean_edge,
        "extreme": lambda rng: list(valid_point(rng, PERCENTILE_INC, 1e-100, 1e100, (20, 150))),
        "log-uniform": lambda rng: list(valid_point(rng, PERCENTILE_INC, 1e-150, 1e150, (1, 30))),
    },
    "An array literal and a percent by stratum (PERCENTILE.INC's samplers, T-4092); the edge stratum adds "
    "Microsoft's example, the domain errors and percents beside a trim-count step.",
)


# -- definitions ---------------------------------------------------------------------------------


def _variadic(
    function: str,
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    domain: Callable[[Sequence[MathArg]], str | None] | None,
    draw: Callable[[str, random.Random], list[Arg]],
    sources: tuple[str, ...],
    *,
    param: str = "number",
) -> Definition:
    return variadic_definition(function, value, domain, (*sources, TABLES_SOURCE), draw=draw, param=param)


_positive_draw = by_stratum_with_edge(
    _magnitudes(variance_draws(MAX_ARGS)),
    _positive_edge,
    "The dispersion draws' magnitudes by stratum; the edge stratum adds Microsoft's example, a 0 or a negative "
    "number and products beyond the double range.",
)

KURT = _variadic(
    "KURT",
    _kurt,
    _moment_domain(4),
    _moment_draw(MAX_ARGS),
    (POOLS_SOURCE, SQUARES_SOURCE, VALUE_ARGS_SOURCE),
)
SKEW = _variadic(
    "SKEW",
    _skew,
    _moment_domain(3),
    _moment_draw(MAX_ARGS),
    (SKEW_P_SOURCE, POOLS_SOURCE, SQUARES_SOURCE, VALUE_ARGS_SOURCE),
)
# A file holding a newer (`_xlfn.`) function with 255 arguments does not open in Excel (syntax.md "The repeating
# maxima", T-4089), so SKEW.P points take at most 254 numbers.
SKEW_P = _variadic(
    "SKEW.P",
    _skew_p,
    _moment_domain(3),
    _moment_draw(MAX_ARGS - 1),
    (SKEW_P_SOURCE, POOLS_SOURCE, SQUARES_SOURCE, VALUE_ARGS_SOURCE),
)
GEOMEAN = _variadic(
    "GEOMEAN",
    _geomean,
    domain_of(_geomean),
    _positive_draw,
    (GEOMEAN_ROOT_SOURCE, VALUE_ARGS_SOURCE, GEOMEAN_EVIDENCE),
)
HARMEAN = _variadic(
    "HARMEAN",
    _harmean,
    domain_of(_harmean),
    _positive_draw,
    (HARMEAN_SOURCE, POOLS_SOURCE, VALUE_ARGS_SOURCE, POOLS_EVIDENCE),
)
AVERAGEA = _variadic("AVERAGEA", _averagea, None, draw_numbers, (SNAP_SOURCE, AVERAGEA_SOURCE), param="value")
TRIMMEAN = register(
    Definition(
        function="TRIMMEAN",
        params=(
            Param("array", "array", source=TRIMMEAN_SOURCE),
            Param("percent", "real", source=TRIMMEAN_SOURCE),
        ),
        value=_trimmean,
        domain=domain_of(_trimmean),
        draw=TRIMMEAN_DRAW,
        sources=(TRIMMEAN_SOURCE, TABLES_SOURCE, "decision 0271"),
    )
)
