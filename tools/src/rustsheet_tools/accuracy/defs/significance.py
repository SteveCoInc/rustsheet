"""CHISQ.TEST, CHITEST, Z.TEST, ZTEST and PROB (Gate A, T-4098, decision 0271).

The math, on the array literals' doubles:
- CHISQ.TEST and CHITEST: the exact statistic chi2 = sum((actual - expected)^2 / expected) over the paired cells (an
  exact rational), then the right tail Q(df / 2, chi2 / 2) of CHISQ.DIST.RT's definition (`gamma_dist`, T-3925),
  with df = (rows - 1)(columns - 1) for a table of at least two rows and two columns and cells - 1 for one row or one
  column (probes-numeric/t1049-chisqtest A12, A25, A26, A29:A31: `CHISQ.TEST({10,20,30},{15,15,30})` and
  `CHISQ.TEST({10;20;30},{15,15,30})` are 0.18887560283756183, Q(1, 5/3); `CHISQ.TEST({10,20;30,40},{15,15;30,40})`
  is 0.06788915486182905, Q(1/2, 5/3)). Excel takes the direct tail at every df (statistics.md "Incomplete gamma:
  accurate kernel", T-3058). CHITEST is the same function (statistics.md "Legacy distribution names sweep").
- Z.TEST and ZTEST: Phi(-z) with z = (mean - x) sqrt(n) / sigma, sigma given or the exact sample standard deviation
  (statistics.md "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, ...": Z.TEST is `NORM.S.DIST(-z,TRUE)`, not
  1 - NORM.S.DIST(z)); the mean and the deviations are exact rationals. Phi is NORM.S.DIST's (`normal.cdf`). Excel's
  deviations are two-pass: where the squares overflow and the deviations do not it is a number (probes/t4098 A47).
- PROB: the exact sum of the probabilities whose x lies in [lower_limit, upper_limit], or equals lower_limit when
  upper_limit is omitted (Microsoft's PROB page); `lower > upper` is 0 (statistics.md "RANK, FORECAST, PROB, Z.TEST
  and T.TEST: lifted scalar arguments"; t4098 A39). A probability outside [0, 1] is no error when the sum is 1
  (t4098 A32, A33: `PROB({1,2},{1.5,-0.5},1)` is 1.5).

Domain (an Excel error region, never a math value), each from the cited entries and cells:
- CHISQ.TEST: different cell counts are `#N/A` (t1049-chisqtest A16:A18; "Legacy distribution names sweep",
  `CHITEST(B1:D2,B4:D4)`); one cell is `#N/A` (A27, A28, A36: a scalar or a one-cell range; t4098 A42, A46: a 1x1
  array literal); an expected cell of exactly 0 is `#DIV/0!` (A33, and t4098 A43 beside an actual 0); a squared
  difference past the double range is `#NUM!` (t4098 A44, A45); a negative statistic is `#NUM!` (A32,
  `CHISQ.TEST({10,20,30},{15,15,-30})`; a negative expected cell with a positive statistic is a number, statistics.md
  "Paired functions pair two ranges of equal count").
- Z.TEST: one value is `#DIV/0!` with or without sigma ("Z.TEST: an error array first; one data value is #DIV/0!",
  t4098 A53); then a sigma of 0 or below `#NUM!` ("Legacy distribution names sweep, second family"; t4098 A56); with
  sigma omitted, constant data `#DIV/0!` ("Thin distribution functions: ... (T-1049)": `Z.TEST(D1:D3,5)`; t4098 A54)
  and a squared deviation past the double range `#NUM!` ("Overflowing squares (T-2432)"; t4098 A51); with sigma given,
  a z past the double range above 0 (mean above x) `#NUM!` (t4098 A48, one cell; sigma omitted is unprobed).
- PROB: different counts `#N/A` ("PROB: an error data argument first ..."); a probability sum off 1 `#NUM!`, evidenced
  from 1 - 1E-15 down and 1 + 1E-14 up (t4098 A3:A31; `SUM_NUM_BELOW`, `SUM_NUM_ABOVE`), while 1 + 1E-15 and the
  ten-0.1 sum are numbers (A1, A2; `SUM_OK_ABOVE`), as is a sum 5.55E-17 below 1 (A33; `SUM_OK_BELOW`).

Given no math value (`NoValueError`), as no evidence names Excel's result:
- CHISQ.TEST over arrays of equal counts and different shapes, one of them a table of two rows and two columns or
  more (whose df Excel takes is open, T-2650; a row against a column is df cells - 1, t1049-chisqtest A30); a difference
  past the double range, a finite-term statistic whose double sum overflows, an expected 0 beside an overflowing
  square, a double statistic whose sign is not the exact one; a subnormal literal.
- Z.TEST: a constant series whose double mean is not its value (`paired_regression.constant_exact`; Excel's
  `Z.TEST({0.1,0.1,0.1},0)` is 0, t4098 A50, where the math is `#DIV/0!`), a sum of the data past the double range,
  a subnormal literal or sigma, and a z past the double range above 0 with sigma omitted.
Excel kept as the target (decision 0300): a z past the double range below 0 keeps its math value 1, where Excel's
`Z.TEST({1,2,3},1E+300,1E-300)` is 0.5 (t4098 A49, sigma given, one cell).
- PROB: a probability sum off 1 between the evidenced bands (for two probabilities, below 1 by 5.6E-17 to 1E-15, above
  1 by 1.1E-15 to 1E-14; the `#NUM!` bands widen by 2^-53 for each further probability).
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    below_normal,
    by_stratum,
    domain_of,
    fraction_to_mpf,
    is_real,
    one_of,
    signed,
)
from rustsheet_tools.accuracy.defs.aggregate import running_total
from rustsheet_tools.accuracy.defs.aggregate_array import array_of
from rustsheet_tools.accuracy.defs.dispersion import deviations_overflow
from rustsheet_tools.accuracy.defs.gamma_dist import CHISQ_DIST_RT
from rustsheet_tools.accuracy.defs.normal import FLUSH, cdf
from rustsheet_tools.accuracy.defs.paired_regression import constant_exact
from rustsheet_tools.accuracy.literal import Arg, Matrix, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

CHISQ_SOURCE = "statistics.md#thin-distribution-functions-argument-coercion-truncation-and-error-edges-t-1049"
LEGACY_SOURCE = "statistics.md#legacy-distribution-names-sweep-t-1047"
SHAPES_SOURCE = "statistics.md#paired-functions-pair-two-ranges-of-equal-count-whatever-their-shapes-t-2255"
KERNEL_SOURCE = "statistics.md#incomplete-gamma-accurate-kernel-t-0971"
Z_SOURCE = (
    "statistics.md#ztest-an-error-array-first-one-data-value-is-div0-a-lone-non-number-cell-is-na-t-2256"
)
Z_FORM_SOURCE = (
    "statistics.md#one-incomplete-gamma-kernel-under-erf-erfc-norm-chisq-gammadist-and-poisson-t-1778"
)
Z_LEGACY_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"
SQUARES_SOURCE = "statistics.md#overflowing-squares-t-2432"
PROB_SOURCE = (
    "statistics.md#prob-an-error-data-argument-first-one-value-on-each-side-is-div0-unequal-counts-na-t-2256"
)
LIFT_SOURCE = "statistics.md#rank-forecast-prob-ztest-and-ttest-lifted-scalar-arguments"
PROBE = "probes/t4098-tests-prob-domains"
GATE_SOURCE = "statistics.md#chisqtest-ztest-and-prob-gate-a-domains-t-4098"

MAX_CELLS = 120  # two tables of 120 literals stay well inside Excel's 8192-character formula
# Microsoft's CHISQ.TEST example: rows Agree, Neutral, Disagree; columns Men, Women (0.0003082).
MICROSOFT_ACTUAL: Matrix = ((58.0, 35.0), (11.0, 25.0), (10.0, 23.0))
MICROSOFT_EXPECTED: Matrix = ((45.35, 47.65), (17.56, 18.44), (16.09, 16.91))
MICROSOFT_Z_DATA = (
    3.0,
    6.0,
    7.0,
    8.0,
    6.0,
    5.0,
    4.0,
    2.0,
    1.0,
    9.0,
)  # Z.TEST(A2:A11,4) 0.090574, ..6) 0.863043
MICROSOFT_PROB_X = (0.0, 1.0, 2.0, 3.0)  # PROB(x,p,2) 0.1, PROB(x,p,1,3) 0.8
MICROSOFT_PROB_P = (0.2, 0.3, 0.1, 0.4)


def _floats(a: MathArg) -> list[float]:
    """An array or matrix argument's doubles in reading order."""
    assert isinstance(a, tuple)
    out: list[float] = []
    for v in a:
        if isinstance(v, tuple):
            out.extend(float(e) for e in v)
        else:
            assert is_real(v)
            out.append(float(v))
    return out


def _shape(a: MathArg) -> tuple[int, int]:
    """A matrix argument's (rows, columns)."""
    assert isinstance(a, tuple) and a and isinstance(a[0], tuple)
    return len(a), len(a[0])


def _vector(a: MathArg) -> bool:
    """One row or one column: df is cells - 1 whichever (t1049-chisqtest A30, a column against a row)."""
    return 1 in _shape(a)


# -- CHISQ.TEST --------------------------------------------------------------------------------


def chi_squared(actual: Sequence[float], expected: Sequence[float]) -> Fraction:
    """The exact statistic, with CHISQ.TEST's `#DIV/0!` for an expected cell of 0 and `#NUM!` for a squared difference
    past the double range (t4098 A43:A45), and NoValueError where Excel's double sum (each term (a - e)^2 / e in
    doubles, summed left to right) leaves the double range otherwise or has another sign than the exact one."""
    diffs = [a - e for a, e in zip(actual, expected, strict=True)]
    if not all(math.isfinite(d) for d in diffs):
        msg = "a difference past the double range (unprobed)"
        raise NoValueError(msg)
    square_overflows = any(math.isinf(d * d) for d in diffs)
    zero_expected = any(e == 0 for e in expected)
    if square_overflows and zero_expected:
        msg = "an expected 0 beside an overflowing square: which error comes first is unprobed"
        raise NoValueError(msg)
    if zero_expected:
        msg = "#DIV/0!: an expected cell of 0"
        raise DomainError(msg)
    if square_overflows:
        msg = "#NUM!: a squared difference past the double range"
        raise DomainError(msg)
    terms = [d * d / e for d, e in zip(diffs, expected, strict=True)]
    double = running_total(terms)[1] if all(math.isfinite(t) for t in terms) else math.inf
    if not math.isfinite(double):
        msg = "Excel's double statistic leaves the double range (unprobed)"
        raise NoValueError(msg)
    exact = sum(
        ((Fraction(a) - Fraction(e)) ** 2 / Fraction(e) for a, e in zip(actual, expected, strict=True)),
        Fraction(0),
    )
    if (double < 0) != (exact < 0):
        msg = "Excel's double statistic and the exact one differ in sign"
        raise NoValueError(msg)
    return exact


def degrees_of_freedom(rows: int, columns: int) -> int:
    """(rows - 1)(columns - 1) for a table, cells - 1 for one row or one column (t1049-chisqtest A12, A29:A31)."""
    if rows > 1 and columns > 1:
        return (rows - 1) * (columns - 1)
    return rows * columns - 1


def _chisq_test(args: Sequence[MathArg]) -> mpmath.mpf:
    actual, expected = args
    xs, es = _floats(actual), _floats(expected)
    if len(xs) != len(es):
        msg = "#N/A: the arrays differ in cell count"
        raise DomainError(msg)
    if len(xs) < 2:
        msg = "#N/A: one cell"
        raise DomainError(msg)
    if _shape(actual) != _shape(expected) and not (_vector(actual) and _vector(expected)):
        msg = "equal counts of different shapes, one a table: Excel's df is unprobed (T-2650)"
        raise NoValueError(msg)
    if below_normal(*(xs + es)):
        msg = "a subnormal literal (Excel reads it as 0; unprobed here)"
        raise NoValueError(msg)
    chi2 = chi_squared(xs, es)
    if chi2 < 0:
        msg = "#NUM!: a negative statistic"
        raise DomainError(msg)
    df = degrees_of_freedom(*_shape(actual))
    return CHISQ_DIST_RT.value([fraction_to_mpf(chi2), mpmath.mpf(df)])


def _table(rng: random.Random, rows: int, columns: int, lo: float, hi: float) -> tuple[Matrix, Matrix]:
    """(actual, expected): expected cells log-uniform in [lo, hi], actual cells the expected moved by a relative noise
    drawn per table (a good fit to a poor one), each written to 15 digits."""
    noise = log_uniform(rng, 1e-4, 2.0)
    expected = tuple(tuple(round15(log_uniform(rng, lo, hi)) for _ in range(columns)) for _ in range(rows))
    actual = tuple(tuple(round15(abs(e * (1 + noise * rng.gauss(0, 1)))) for e in row) for row in expected)
    return actual, expected


def _counts(rng: random.Random, rows: int, columns: int) -> tuple[Matrix, Matrix]:
    """A contingency table of integer counts and its expected counts under independence (row total * column total /
    grand total, written to 15 digits)."""
    actual = tuple(tuple(float(rng.randint(0, 200)) for _ in range(columns)) for _ in range(rows))
    row_totals = [sum(r) for r in actual]
    column_totals = [sum(r[j] for r in actual) for j in range(columns)]
    total = sum(row_totals)
    if total == 0 or 0 in row_totals or 0 in column_totals:
        return _table(rng, rows, columns, 1.0, 200.0)
    expected = tuple(tuple(round15(rt * ct / total) for ct in column_totals) for rt in row_totals)
    return actual, expected


def _table_shape(rng: random.Random, hi: int) -> tuple[int, int]:
    """(rows, columns) of at least two cells and at most `hi` (one row and one column are common in sheets)."""
    while True:
        kind = rng.randrange(3)
        n = rng.randint(2, hi)
        if kind == 0:
            shape = (1, n)
        elif kind == 1:
            shape = (n, 1)
        else:
            shape = (rng.randint(2, hi), rng.randint(2, hi))
        if 2 <= shape[0] * shape[1] <= MAX_CELLS:
            return shape


def _chisq_typical(rng: random.Random) -> list[Arg]:
    rows, columns = _table_shape(rng, 6)
    pair = _counts(rng, rows, columns) if rng.random() < 0.5 else _table(rng, rows, columns, 1e-2, 1e4)
    return list(pair)


def _chisq_tail(rng: random.Random) -> list[Arg]:
    """A poor fit (a tiny right tail) or a very close one (a tail near 1)."""
    rows, columns = _table_shape(rng, 5)
    expected = tuple(tuple(round15(log_uniform(rng, 1.0, 1e4)) for _ in range(columns)) for _ in range(rows))
    if rng.random() < 0.5:
        scale = log_uniform(rng, 1.5, 30.0)
        actual = tuple(
            tuple(round15(e * scale if rng.random() < 0.5 else e / scale) for e in row) for row in expected
        )
    else:
        noise = log_uniform(rng, 1e-9, 1e-4)
        actual = tuple(tuple(round15(e * (1 + noise * rng.gauss(0, 1))) for e in row) for row in expected)
    return [actual, expected]


def _one_row(values: Sequence[float]) -> Matrix:
    return (tuple(values),)


def _chisq_edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(2, 6)

    def with_zero_expected() -> list[Arg]:
        actual, expected = _table(rng, 1, n, 1.0, 100.0)
        cells = list(expected[0])
        cells[rng.randrange(n)] = 0.0
        return [actual, _one_row(cells)]

    def negative_statistic() -> list[Arg]:
        # one large negative expected cell drives the sum below 0 (t1049-chisqtest A32)
        actual, expected = _table(rng, 1, n, 1.0, 100.0)
        cells = list(expected[0])
        i = rng.randrange(n)
        cells[i] = -round15(log_uniform(rng, 0.01, 0.5))
        row = list(actual[0])
        row[i] = round15(log_uniform(rng, 100.0, 1000.0))
        return [_one_row(row), _one_row(cells)]

    def negative_actual() -> list[Arg]:
        actual, expected = _table(rng, 1, n, 1.0, 100.0)
        row = list(actual[0])
        row[rng.randrange(n)] = -round15(log_uniform(rng, 0.1, 100.0))
        return [_one_row(row), expected]

    def unequal() -> list[Arg]:
        return [_table(rng, 1, n + 1, 1.0, 100.0)[0], _table(rng, 1, n, 1.0, 100.0)[1]]

    def one_cell() -> list[Arg]:
        actual, expected = _table(rng, 1, 1, 1.0, 100.0)
        return [actual, expected]

    def row_against_column() -> list[Arg]:
        actual, expected = _table(rng, 1, n, 1.0, 100.0)
        return [actual, tuple((e,) for e in expected[0])]  # t1049-chisqtest A30

    def exact_fit() -> list[Arg]:
        rows, columns = _table_shape(rng, 4)
        expected = _table(rng, rows, columns, 1.0, 1000.0)[1]
        return [expected, expected]

    return one_of(
        rng,
        [
            lambda: [MICROSOFT_ACTUAL, MICROSOFT_EXPECTED],
            lambda: [((10.0, 20.0, 30.0),), ((15.0, 15.0, 30.0),)],  # t1049-chisqtest A29
            lambda: [((10.0, 20.0), (30.0, 40.0)), ((15.0, 15.0), (30.0, 40.0))],  # A31
            exact_fit,
            with_zero_expected,
            negative_statistic,
            negative_actual,
            unequal,
            one_cell,
            lambda: list(_table(rng, 2, 2, 1.0, 100.0)),
            lambda: list(_table(rng, 1, 2, 1.0, 100.0)),
            lambda: list(_table(rng, 2, 1, 1.0, 100.0)),
            row_against_column,
        ],
    )


def _chisq_extreme(rng: random.Random) -> list[Arg]:
    """Large tables: up to 120 cells, many degrees of freedom."""
    while True:
        rows, columns = rng.randint(1, 12), rng.randint(1, 120)
        if 20 <= rows * columns <= MAX_CELLS:
            break
    return list(_table(rng, rows, columns, 1e-3, 1e6))


def _chisq_log_uniform(rng: random.Random) -> list[Arg]:
    rows, columns = _table_shape(rng, 4)
    return list(_table(rng, rows, columns, 1e-100, 1e100))


CHISQ_DRAW = by_stratum(
    {
        "typical": _chisq_typical,
        "tail": _chisq_tail,
        "edge": _chisq_edge,
        "extreme": _chisq_extreme,
        "log-uniform": _chisq_log_uniform,
    },
    "CHISQ.TEST draws: an actual and an expected table of one shape (counts under independence, or a noisy fit).",
)


def _chisq_test_definition(function: str) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("actual_range", "matrix", source=CHISQ_SOURCE),
                Param("expected_range", "matrix", source=CHISQ_SOURCE),
            ),
            value=_chisq_test,
            domain=domain_of(_chisq_test),
            draw=CHISQ_DRAW,
            sources=(
                CHISQ_SOURCE,
                LEGACY_SOURCE,
                SHAPES_SOURCE,
                KERNEL_SOURCE,
                GATE_SOURCE,
                PROBE,
                "decision 0271",
            ),
        )
    )


CHISQ_TEST = _chisq_test_definition("CHISQ.TEST")
CHITEST = _chisq_test_definition("CHITEST")


# -- Z.TEST ------------------------------------------------------------------------------------


def _z_test(args: Sequence[MathArg]) -> mpmath.mpf:
    data, x = args[0], args[1]
    assert is_real(x)
    xs = _floats(data)
    sigma = args[2] if len(args) > 2 else None
    assert sigma is None or is_real(sigma)
    if len(xs) < 2:
        msg = "#DIV/0!: one value"
        raise DomainError(msg)
    scalars_given = [float(x)] + ([float(sigma)] if sigma is not None else [])
    if below_normal(*(xs + scalars_given)):
        msg = "a subnormal literal (unprobed)"
        raise NoValueError(msg)
    if not math.isfinite(running_total(xs)[1]):
        msg = "the sum of the data beyond the double range (unprobed)"
        raise NoValueError(msg)
    n = len(xs)
    exact = [Fraction(v) for v in xs]
    mean = sum(exact, Fraction(0)) / n
    if sigma is not None:
        if sigma <= 0:
            msg = "#NUM!: sigma 0 or below"
            raise DomainError(msg)
        spread = mpmath.mpf(sigma)
    else:
        data_args: list[MathArg] = [mpmath.mpf(v) for v in xs]
        if (why := deviations_overflow(data_args)) is not None:
            msg = f"#NUM!: {why}"
            raise DomainError(msg)
        if constant_exact(xs):
            msg = "#DIV/0!: constant data with sigma omitted"
            raise DomainError(msg)
        spread = mpmath.sqrt(fraction_to_mpf(sum(((v - mean) ** 2 for v in exact), Fraction(0)) / (n - 1)))
    z = fraction_to_mpf(mean - Fraction(float(x))) * mpmath.sqrt(n) / spread
    _z_overflow(z, running_total(xs)[1] / n - float(x), n, spread, sigma is not None)
    if z >= FLUSH:
        # Phi(-z) is below the envelope's 1E-300 floor here, and Excel's lower tail is 0 (normal.py FLUSH, T-1040); a
        # z of 1E+200 would otherwise leave dps 60 and 80 apart in a value of e^-1E+400.
        return mpmath.mpf(0)
    if z <= -FLUSH:
        # 1 - Phi(-37.625) is below 1E-310: Phi(-z) is 1 at every working precision (t4098 A49's z is -1.7E+600)
        return mpmath.mpf(1)
    return cdf(-z)


def _z_overflow(z: mpmath.mpf, difference: float, n: int, spread: mpmath.mpf, sigma_given: bool) -> None:
    """Z.TEST's `#NUM!` where Excel's double z = (mean - x) sqrt(n) / sigma passes the double range above 0 with sigma
    given (t4098 A48, one cell); NoValueError for a z above 0 past the range otherwise (sigma omitted: unprobed). A z
    below 0 past the range keeps its math value, Phi(-z) = 1, although Excel gives 0.5 there (A49, sigma given; Excel
    kept as the target, decision 0300)."""
    double_z = difference * math.sqrt(n) / float(spread)
    if z < 0 or (abs(z) <= sys.float_info.max and math.isfinite(double_z)):
        return
    if sigma_given and double_z == math.inf:
        msg = "#NUM!: z past the double range above 0"
        raise DomainError(msg)
    msg = "z past the double range above 0 with sigma omitted (unprobed)"
    raise NoValueError(msg)


def _z_data(rng: random.Random, n: int, lo: float, hi: float) -> tuple[float, ...]:
    return array_of(rng, n, lo, hi)


def _z_x(rng: random.Random, xs: Sequence[float], spread: float) -> float:
    """x near the data: at the mean, a few standard errors off it, or far out (a tiny tail)."""
    mean = sum(xs) / len(xs)
    shape = rng.randrange(4)
    if shape == 0:
        return round15(mean)
    if shape == 1:
        return rng.choice(xs)
    width = spread / math.sqrt(len(xs)) or abs(mean) or 1.0
    return round15(mean + signed(rng, width * log_uniform(rng, 1e-2, 30.0)))


def _spread(xs: Sequence[float]) -> float:
    mean = sum(xs) / len(xs)
    return math.sqrt(sum((v - mean) ** 2 for v in xs) / (len(xs) - 1))


def _z_point(rng: random.Random, xs: tuple[float, ...]) -> list[Arg]:
    """The data, x and (two times in three) sigma: near the sample spread or log-uniform about it."""
    spread = _spread(xs) or 1.0
    x = _z_x(rng, xs, spread)
    if rng.random() < 1 / 3:
        return [xs, x]
    return [xs, x, round15(spread * log_uniform(rng, 0.1, 10.0))]


def _z_typical(rng: random.Random) -> list[Arg]:
    return _z_point(rng, _z_data(rng, rng.randint(2, 12), 1e-3, 1e6))


def _z_tail(rng: random.Random) -> list[Arg]:
    """x many standard errors from the mean (tails near 0 and 1), or data with a large offset."""
    n = rng.randint(3, 20)
    if rng.random() < 0.5:
        offset = signed(rng, log_uniform(rng, 1.0, 1e8))
        xs = tuple(round15(offset + abs(offset) * 1e-4 * rng.gauss(0, 1)) for _ in range(n))
    else:
        xs = _z_data(rng, n, 1e-2, 1e3)
    spread = _spread(xs) or 1.0
    mean = sum(xs) / n
    x = round15(mean + signed(rng, spread / math.sqrt(n) * log_uniform(rng, 3.0, 38.0)))
    return [xs, x] if rng.random() < 0.5 else [xs, x, round15(spread)]


def _z_edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(2, 8)
    constant = (rng.randint(-160, 160) / 8,) * n
    return one_of(
        rng,
        [
            lambda: [MICROSOFT_Z_DATA, 4.0],
            lambda: [MICROSOFT_Z_DATA, 6.0],
            lambda: [MICROSOFT_Z_DATA, 4.0, 1.0],
            lambda: [(round15(log_uniform(rng, 1e-3, 1e6)),), 1.0],  # one value: #DIV/0!
            lambda: [(round15(log_uniform(rng, 1e-3, 1e6)),), 1.0, 1.0],
            lambda: [_z_data(rng, n, 1e-3, 1e6), 1.0, 0.0],  # sigma 0: #NUM!
            lambda: [_z_data(rng, n, 1e-3, 1e6), 1.0, -round15(log_uniform(rng, 1e-3, 1e3))],
            lambda: [constant, round15(log_uniform(rng, 1e-3, 1e3))],  # constant: #DIV/0!
            lambda: [constant, constant[0] + 1, 1.0],  # constant with sigma: a number
            lambda: [_z_data(rng, n, 1e199, 1e201), 0.0],  # squared deviations overflow: #NUM!
            lambda: [_z_data(rng, n, 1e-3, 1e6), 0.0],
            lambda: [tuple(float(rng.randint(-9, 9)) for _ in range(n)), float(rng.randint(-9, 9))],
        ],
    )


def _z_extreme(rng: random.Random) -> list[Arg]:
    return _z_point(rng, _z_data(rng, rng.randint(20, MAX_CELLS), 1e-20, 1e20))


def _z_log_uniform(rng: random.Random) -> list[Arg]:
    return _z_point(rng, _z_data(rng, rng.randint(2, 6), 1e-150, 1e150))


Z_DRAW = by_stratum(
    {
        "typical": _z_typical,
        "tail": _z_tail,
        "edge": _z_edge,
        "extreme": _z_extreme,
        "log-uniform": _z_log_uniform,
    },
    "Z.TEST draws: the data, x near it and sigma (omitted one time in three).",
)


def _z_test_definition(function: str) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("array", "array", source=Z_SOURCE),
                Param("x", "real", source=LIFT_SOURCE),
                Param("sigma", "real", source=LIFT_SOURCE, optional=True),
            ),
            value=_z_test,
            domain=domain_of(_z_test),
            draw=Z_DRAW,
            sources=(
                Z_SOURCE,
                Z_FORM_SOURCE,
                Z_LEGACY_SOURCE,
                SQUARES_SOURCE,
                GATE_SOURCE,
                LIFT_SOURCE,
                PROBE,
                "decision 0271",
            ),
        )
    )


Z_TEST = _z_test_definition("Z.TEST")
ZTEST = _z_test_definition("ZTEST")


# -- PROB --------------------------------------------------------------------------------------


def _prob(args: Sequence[MathArg]) -> mpmath.mpf:
    xs, ps = _floats(args[0]), _floats(args[1])
    lower = args[2]
    upper = args[3] if len(args) > 3 else lower
    assert is_real(lower) and is_real(upper)
    if len(xs) != len(ps):
        msg = "#N/A: x_range and prob_range differ in count"
        raise DomainError(msg)
    if below_normal(*(xs + ps)):
        msg = "a subnormal literal (unprobed)"
        raise NoValueError(msg)
    sum_off(ps)
    lo, hi = Fraction(float(lower)), Fraction(float(upper))
    return fraction_to_mpf(
        sum((Fraction(p) for x, p in zip(xs, ps, strict=True) if lo <= x <= hi), Fraction(0))
    )


# The probability sum's evidenced bands (probes/t4098 A1:A31, the exact sum of the literals' doubles minus 1): 1E-15
# below 1 and 1E-14 above are `#NUM!`, 1E-15 above (1.0000000000000010214) and the ten-0.1 sum (5.6E-17 above) are
# numbers. Each bound keeps a margin of a few ulps of 1 for Excel's double sum against the exact one. The `#NUM!` bands
# are probed on two probabilities only (one addition): for n probabilities each further addition may move Excel's double
# sum by half an ulp of 1, so `sum_off` widens them by (n - 2) * 2^-53 (no table point is near them: the drawn `#NUM!`
# sums are off by 4.8E-9 or more). The number bands hold exact sums of the probed shapes (two, three and ten terms).
SUM_NUM_BELOW = Fraction(-99, 10**17)
SUM_NUM_ABOVE = Fraction(99, 10**16)
SUM_OK_ABOVE = Fraction(11, 10**16)
SUM_OK_BELOW = Fraction(
    -56, 10**18
)  # t4098 A33, `PROB({1,2,3},{1.2,-0.1,-0.1},1)`: 5.55E-17 below 1, a number


def sum_off(ps: Sequence[float]) -> None:
    """PROB's `#NUM!` for probabilities whose exact sum is off 1 by an evidenced amount (widened by the extra additions of
    more than two probabilities); NoValueError between the bands (for two probabilities, an exact sum below 1 by 5.6E-17
    to 1E-15, or above by 1.1E-15 to 1E-14)."""
    off = sum((Fraction(p) for p in ps), Fraction(0)) - 1
    margin = Fraction(max(len(ps) - 2, 0), 2**53)
    if off <= SUM_NUM_BELOW - margin or off >= SUM_NUM_ABOVE + margin:
        msg = "#NUM!: the probabilities do not sum to 1"
        raise DomainError(msg)
    if off < SUM_OK_BELOW or off > SUM_OK_ABOVE:
        msg = "a probability sum off 1 between the evidenced bands"
        raise NoValueError(msg)


def _dyadic_probabilities(rng: random.Random, n: int) -> tuple[float, ...]:
    """n probabilities k / 1024 summing to exactly 1 (each written exactly in 15 digits)."""
    cuts = sorted(rng.randint(0, 1024) for _ in range(n - 1))
    bounds = [0, *cuts, 1024]
    return tuple((bounds[i + 1] - bounds[i]) / 1024 for i in range(n))


def _decimal_probabilities(rng: random.Random, n: int) -> tuple[float, ...]:
    """n probabilities of two decimals summing to 1 in decimal (their doubles sum to 1 within a few ulps)."""
    cuts = sorted(rng.randint(0, 100) for _ in range(n - 1))
    bounds = [0, *cuts, 100]
    return tuple((bounds[i + 1] - bounds[i]) / 100 for i in range(n))


def _off_by(rng: random.Random, ps: tuple[float, ...]) -> tuple[float, ...]:
    """`ps` with its last probability moved by 1E-14 to 0.1 (above 1) or 1E-15 to 0.1 (below): a sum Excel refuses."""
    last = ps[-1]
    delta = (
        log_uniform(rng, 1e-14, 0.1) if rng.random() < 0.5 or last < 0.1 else -log_uniform(rng, 1e-15, 0.1)
    )
    return (*ps[:-1], round15(last + delta))


def _prob_xs(rng: random.Random, n: int, lo: float, hi: float) -> tuple[float, ...]:
    if rng.random() < 0.5:
        start = rng.randint(-5, 5)
        return tuple(float(start + i) for i in range(n))
    return array_of(rng, n, lo, hi)


def _limits(rng: random.Random, xs: Sequence[float]) -> list[Arg]:
    """lower_limit and (three times in four) upper_limit: at data points, between them, or reversed (0)."""
    pick = sorted(rng.sample(list(xs), min(2, len(xs))))
    lo, hi = pick[0], pick[-1]
    shape = rng.randrange(6)
    if shape == 0:
        return [rng.choice(xs)]
    if shape == 1:  # strictly between data points
        span = (max(xs) - min(xs)) or 1.0
        return [round15(lo - span * rng.random() * 0.1), round15(hi + span * rng.random() * 0.1)]
    if shape == 2 and lo != hi:  # reversed: 0
        return [hi, lo]
    return [lo, hi]


def _prob_point(rng: random.Random, n: int, lo: float, hi: float, dyadic: float = 0.5) -> list[Arg]:
    xs = _prob_xs(rng, n, lo, hi)
    ps = _dyadic_probabilities(rng, n) if rng.random() < dyadic else _decimal_probabilities(rng, n)
    return [xs, ps, *_limits(rng, xs)]


def _prob_typical(rng: random.Random) -> list[Arg]:
    return _prob_point(rng, rng.randint(2, 10), 1e-3, 1e6)


def _prob_tail(rng: random.Random) -> list[Arg]:
    """Many small probabilities: a selected sum of a few small terms."""
    return _prob_point(rng, rng.randint(10, 40), 1e-3, 1e6, dyadic=1.0)


def _prob_edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(2, 6)
    return one_of(
        rng,
        [
            lambda: [MICROSOFT_PROB_X, MICROSOFT_PROB_P, 2.0],
            lambda: [MICROSOFT_PROB_X, MICROSOFT_PROB_P, 1.0, 3.0],
            lambda: [(1.0,), (1.0,), 1.0],  # one pair: 1
            lambda: [_prob_xs(rng, n + 1, 1.0, 10.0), _dyadic_probabilities(rng, n), 1.0],  # #N/A
            lambda: [
                (1.0, 2.0),
                (1.5, -0.5),
                rng.choice((1.0, 2.0)),
            ],  # outside [0, 1], summing to 1 (t4098 A32)
            lambda: [_prob_xs(rng, n, 1.0, 10.0), _off_by(rng, _dyadic_probabilities(rng, n)), 1.0],  # #NUM!
            lambda: [
                _prob_xs(rng, n, 1.0, 10.0),
                tuple(p / 2 for p in _dyadic_probabilities(rng, n)),
                1.0,
            ],  # 0.5
            lambda: [_prob_xs(rng, n, 1.0, 10.0), tuple(p * 2 for p in _dyadic_probabilities(rng, n)), 1.0],
            lambda: [(1.0, 2.0), (1.0, 0.0), 1.0],
            lambda: [(5.0, 6.0, 7.0), (0.25, 0.25, 0.5), -1e300, 1e300],
        ],
    )


def _prob_extreme(rng: random.Random) -> list[Arg]:
    return _prob_point(rng, rng.randint(40, MAX_CELLS // 2), 1e-20, 1e20, dyadic=1.0)


def _prob_log_uniform(rng: random.Random) -> list[Arg]:
    return _prob_point(rng, rng.randint(2, 6), 1e-150, 1e150)


PROB_DRAW = by_stratum(
    {
        "typical": _prob_typical,
        "tail": _prob_tail,
        "edge": _prob_edge,
        "extreme": _prob_extreme,
        "log-uniform": _prob_log_uniform,
    },
    "PROB draws: x values, probabilities summing to 1 (k/1024 or two decimals) and limits at or between them.",
)

PROB = register(
    Definition(
        function="PROB",
        params=(
            Param("x_range", "array", source=PROB_SOURCE),
            Param("prob_range", "array", source=PROB_SOURCE),
            Param("lower_limit", "real", source=LIFT_SOURCE),
            Param("upper_limit", "real", source=LIFT_SOURCE, optional=True),
        ),
        value=_prob,
        domain=domain_of(_prob),
        draw=PROB_DRAW,
        sources=(PROB_SOURCE, LIFT_SOURCE, GATE_SOURCE, PROBE, "Microsoft PROB page", "decision 0271"),
    )
)
