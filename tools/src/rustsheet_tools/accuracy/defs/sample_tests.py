"""T.TEST, TTEST, F.TEST and FTEST (Gate A two-sample tests, T-4097, decision 0271).

The math, on the array literals' doubles x[1..n1] (array1) and y[1..n2] (array2), with the exact means and the exact
sums of squared deviations (dispersion's `squared_deviations`), v1 and v2 the sample variances S / (n - 1):
- T.TEST type 1 (paired): the differences d[i] = x[i] - y[i] (exact differences of the doubles), t^2 = n md^2 / vd
  with md and vd the differences' mean and sample variance, df = n - 1;
- type 2 (equal variance): the pooled sp^2 = ((n1 - 1) v1 + (n2 - 1) v2) / (n1 + n2 - 2),
  t^2 = (m1 - m2)^2 / (sp^2 (1/n1 + 1/n2)), df = n1 + n2 - 2;
- type 3 (Welch): a = v1/n1, b = v2/n2, t^2 = (m1 - m2)^2 / (a + b) and the fractional
  df = (a + b)^2 / (a^2/(n1 - 1) + b^2/(n2 - 1)), not truncated (statistics.md "T.TEST rounds y = df/(df+t^2) to a
  double before the tail": Welch's df is fractional, and `T.TEST({1,1,1},{2,3,4},2,3)` has df exactly 2);
- the p-value is tails x P(T > |t|) at df, the Student t tail of `student_t.lower_tail` (I_y(df/2, 1/2)/2 at
  y = df/(df + t^2)). Excel rounds y to a double first, which moves its p by up to tens of ulps (that entry): the
  math is the exact tail.
- F.TEST: f = v1/v2 and p = 2 min(P(F <= f), P(F > f)) at (n1 - 1, n2 - 1), F.DIST's tails (`fisher.lower_tail`,
  `fisher.upper_tail`); statistics.md "F.TEST / FTEST: larger variance on top" (symmetric in its arguments, never
  above 1).
TTEST is T.TEST and FTEST is F.TEST on every paired cell (statistics.md "Legacy distribution names sweep, second
family": "TTEST = T.TEST"; probes-numeric t1048-ttest, m9-legacy-ftest).

Domain (an Excel error region, never a math value), in Excel's check order, each from these statistics.md entries:
- tails outside {1, 2} or type outside {1, 2, 3} is `#NUM!` (the parameters' intervals; "Thin distribution
  functions": `T.TEST(..,3,2)`, `(..,0,2)`, `(..,2,0)`, `(..,2,4)`, t1049-ttest A22:A25, t1048-ttest A21:A24), before
  the paired size check (`T.TEST(B11,A8:B9,0.1,0.3)` is `#NUM!`, "Array arguments given a scalar"). Excel truncates
  both (`T.TEST(..,1.9,2.9)` is tails 1, type 2), so an eval cell's non-integer is truncated (`Param.truncates`);
- a paired test (type 1) of arrays of different lengths is `#N/A` (t1049-ttest A18, A19; t1048-ttest A25);
- too few numbers is `#DIV/0!`: a paired test of one pair (t1049-ttest A41), an equal-variance test with fewer than
  two numbers on either side (`T.TEST(B1:B1,C1:C1,2,2)` A38, `T.TEST(B1:B5,3,2,2)` A50), F.TEST with fewer than two on
  either side (t1049-ftest A15, A16, A29:A31; m9-legacy-ftest A24:A26);
- zero variance is `#DIV/0!`: of the differences for type 1 (t1049-ttest A32, t1048-ttest A7, A10), of both samples
  for types 2 and 3 (t1049-ttest A45:A47), of either sample for F.TEST (t1049-ftest A17:A19, m9-legacy-ftest A21:A23).
  Type 3 with one constant sample is a number (A48), and one number after a constant sample is `#DIV/0!`
  (`T.TEST({3,3,3,3},{5},2,3)`, gen-m9 607e85cfee83a689 A132; the single number second, as evidenced). Type 2 with one constant sample has a positive pooled
  variance, so the math is a number there (the table's edge stratum draws it);
- a variance that overflows is `#NUM!` for an equal-variance T.TEST and for F.TEST ("Overflowing squares (T-2432)":
  `T.TEST(Data!B6:D12,{1E+200,1E+200,-1E+200},1,2)`; "F.TEST: an error argument is that error; overflowing squares
  are #NUM!": `F.TEST(Data!A3:A10,{1E+200,1E+200,-1E+200})`; `F.TEST({100000000.1,100000000.2,100000000.3},
  {1E+200,1E+200,-1E+200})`, gen-m9 3e98b26cf6713e77 A339). These cells overflow Excel's sum of squares in either form
  (one pass, `dispersion.squares_overflow`, and two passes, `dispersion.deviations_overflow`), the overflowing
  sample is second and the first is ordinary; only that shape, in that order, is a domain rule.
Given no math value (`NoValueError`), as no evidence names Excel's result:
- a Welch test with a sample of one number beside a varying sample or another single number, or before a constant
  sample (the reversed order of A132; unprobed);
- an overflow in one form only, the overflowing sample first (the reversed order of the evidence), an overflow beside another sample's error, and any overflow of a paired or Welch test;
- a constant sample (or constant differences) whose double mean is not its value (Excel's deviations are then a
  nonzero constant; paired_regression's `constant_exact`), and differences that are constant in doubles but not
  exactly (Excel's variance is then 0 where the math's is not);
- a subnormal literal (Excel flushes it, numbers.md "Subnormal results become 0"; unprobed here), and a double t^2,
  Welch intermediate or F ratio outside the normal range (unprobed).
The draws keep magnitudes at or below 1E+70 (so every square of a variance stays a normal double) but for the edge
stratum's evidenced overflow shape.

Draws: typical, 2 to 10 numbers per sample, half of them two noisy samples about nearby means; tail, far-apart means
(p down to about 1E-30), near-equal means (p near 1) and large offsets with a small spread (paired_regression's
`offset_series`); edge, Microsoft's examples, the evidenced error shapes, two numbers per sample, identical samples
(t = 0, p = 1) and a constant sample; extreme, 10 to 120 numbers of mixed magnitudes; log-uniform, 2 to 6 numbers
from 1E-70 to 1E+70. The arrays reuse AGGREGATE's `array_of` (T-3693).
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy import model
from rustsheet_tools.accuracy.defs import fisher, student_t
from rustsheet_tools.accuracy.defs._common import (
    StratumDraws,
    below_normal,
    by_stratum,
    domain_of,
    is_real,
    one_of,
    real_args,
)
from rustsheet_tools.accuracy.defs.aggregate_array import array_of
from rustsheet_tools.accuracy.defs.dispersion import deviations_overflow, squared_deviations, squares_overflow
from rustsheet_tools.accuracy.defs.paired_regression import (
    MAX_PAIRS,
    constant_exact,
    constant_series,
    offset_series,
)
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

TTEST_Y_SOURCE = "statistics.md#ttest-rounds-y--dfdft2-to-a-double-before-the-tail-t-0969"
THIN_SOURCE = "statistics.md#thin-distribution-functions-argument-coercion-truncation-and-error-edges-t-1049"
LEGACY_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"
SINGLE_SOURCE = "statistics.md#array-arguments-given-a-scalar-a-single-cell-or-an-error"
OVERFLOW_SOURCE = "statistics.md#overflowing-squares-t-2432"
FTEST_SOURCE = "statistics.md#ftest--ftest-larger-variance-on-top-excels-rounded-powers-an-accurate-remainder-t-1927-t-2361"
FTEST_ERROR_SOURCE = (
    "statistics.md#ftest-an-error-argument-is-that-error-overflowing-squares-are-num-kurts-div0-t-2255"
)
FTEST_SINGLE_SOURCE = "statistics.md#ftest-of-a-single-value-t-2455"
ORDER_SOURCE = (
    "statistics.md#ttest-an-error-sample-first-then-tails-and-type-errors-then-their-range-check-t-2256"
)
MAX_MAGNITUDE = 1e70  # a variance's square (Welch's df) stays a normal double
MIN_MAGNITUDE = 1e-70
OVERFLOW_SAMPLE = (1e200, 1e200, -1e200)  # the evidenced overflow shape (T-2432, T-2255)
# Microsoft's T.TEST example (Data 1, Data 2; T.TEST(A2:A10,B2:B10,2,1) = 0.196016) and F.TEST example (0.648318)
MICROSOFT_T = (
    (3.0, 4.0, 5.0, 8.0, 9.0, 1.0, 2.0, 4.0, 5.0),
    (6.0, 19.0, 3.0, 2.0, 14.0, 4.0, 5.0, 17.0, 1.0),
)
MICROSOFT_F = ((6.0, 7.0, 9.0, 15.0, 21.0), (20.0, 28.0, 31.0, 38.0, 40.0))
PAIRED, EQUAL, WELCH = 1, 2, 3


def _samples(args: Sequence[MathArg]) -> tuple[list[float], list[float]]:
    """The two arrays' doubles; NoValueError for a subnormal literal."""
    out: list[list[float]] = []
    for a in args[:2]:
        assert isinstance(a, tuple)
        out.append([float(v) for v in real_args(a)])
    xs, ys = out
    if below_normal(*xs, *ys):
        msg = "a subnormal literal (Excel reads it as 0; unprobed here)"
        raise NoValueError(msg)
    return xs, ys


def _overflows(values: Sequence[float]) -> int:
    """How many of Excel's two forms of the sum of squares overflow (one pass, two passes): 0, 1 or 2."""
    xs = [mpmath.mpf(v) for v in values]
    return int(squares_overflow(xs) is not None) + int(deviations_overflow(xs) is not None)


def _variance(values: Sequence[float]) -> mpmath.mpf:
    """The exact sample variance (two or more numbers)."""
    return squared_deviations([mpmath.mpf(v) for v in values]) / (len(values) - 1)


def _mean(values: Sequence[float]) -> mpmath.mpf:
    return mpmath.fsum(mpmath.mpf(v) for v in values) / len(values)


def _overflow_rule(samples: Sequence[Sequence[float]], evidenced: bool) -> None:
    """`#NUM!` where the second sample overflows Excel's sum of squares in both forms, the first is ordinary (two or
    more numbers, not constant) and the test is an evidenced one; NoValueError for any other overflow."""
    counts = [_overflows(s) for s in samples]
    if not any(counts):
        return
    first = samples[0]
    # Every evidencing cell has the overflowing sample second (T-2432, T-2255, gen-m9 3e98b26cf6713e77 A339); the
    # reversed order is unprobed.
    if evidenced and counts == [0, 2] and len(first) >= 2 and len(set(first)) > 1:
        msg = "#NUM!: a sample's sum of squares is beyond the double range"
        raise DomainError(msg)
    msg = "an overflowing sum of squares in an unprobed shape"
    raise NoValueError(msg)


def _check(t2: mpmath.mpf, *intermediates: mpmath.mpf) -> None:
    """NoValueError where Excel's double t^2 or an intermediate leaves the normal range (unprobed)."""
    for value in (t2, *intermediates):
        double = float(value)
        if not math.isfinite(double) or below_normal(double) or (value != 0 and double == 0):
            msg = "a double intermediate outside the normal range (unprobed)"
            raise NoValueError(msg)


def _paired(xs: Sequence[float], ys: Sequence[float]) -> tuple[mpmath.mpf, mpmath.mpf]:
    """(t^2, df) of the paired test."""
    if len(xs) < 2:
        msg = "#DIV/0!: one pair"
        raise DomainError(msg)
    if _overflows(xs) or _overflows(ys):
        msg = "an overflowing sum of squares in a paired test (unprobed)"
        raise NoValueError(msg)
    exact = [Fraction(x) - Fraction(y) for x, y in zip(xs, ys, strict=True)]
    doubles = [x - y for x, y in zip(xs, ys, strict=True)]
    if len(set(doubles)) == 1:
        if len(set(exact)) > 1:
            msg = "differences constant in doubles only (Excel's variance is 0)"
            raise NoValueError(msg)
        if constant_exact(doubles):
            msg = "#DIV/0!: the differences have zero variance"
            raise DomainError(msg)
    n = len(exact)
    diffs = [mpmath.mpf(d.numerator) / d.denominator for d in exact]
    md = mpmath.fsum(diffs) / n
    vd = squared_deviations(diffs) / (n - 1)
    t2 = n * md * md / vd
    _check(t2, vd)
    return t2, mpmath.mpf(n - 1)


def _equal_variance(xs: Sequence[float], ys: Sequence[float]) -> tuple[mpmath.mpf, mpmath.mpf]:
    """(t^2, df) of the equal-variance test."""
    if len(xs) < 2 or len(ys) < 2:
        msg = "#DIV/0!: fewer than two numbers in a sample"
        raise DomainError(msg)
    _overflow_rule((xs, ys), evidenced=True)
    if constant_exact(xs) and constant_exact(ys):
        msg = "#DIV/0!: both samples have zero variance"
        raise DomainError(msg)
    n1, n2 = len(xs), len(ys)
    df = mpmath.mpf(n1 + n2 - 2)
    pooled = ((n1 - 1) * _variance(xs) + (n2 - 1) * _variance(ys)) / df
    diff = _mean(xs) - _mean(ys)
    t2 = diff * diff / (pooled * (mpmath.mpf(1) / n1 + mpmath.mpf(1) / n2))
    _check(t2, pooled)
    return t2, df


def _welch(xs: Sequence[float], ys: Sequence[float]) -> tuple[mpmath.mpf, mpmath.mpf]:
    """(t^2, df) of the Welch test, df fractional."""
    # `T.TEST({3,3,3,3},{5},2,3)` is #DIV/0! (gen-m9 607e85cfee83a689 A132): one number after a constant sample. The
    # reversed order is unprobed (NoValueError below).
    if len(ys) < 2 and len(xs) >= 2 and constant_exact(xs):
        msg = "#DIV/0!: one number after a constant sample"
        raise DomainError(msg)
    if len(xs) < 2 or len(ys) < 2:
        msg = "a Welch test with one number in a sample (its variance is 0/0; unprobed but after a constant sample)"
        raise NoValueError(msg)
    _overflow_rule((xs, ys), evidenced=False)
    if constant_exact(xs) and constant_exact(ys):
        msg = "#DIV/0!: both samples have zero variance"
        raise DomainError(msg)
    n1, n2 = len(xs), len(ys)
    a, b = _variance(xs) / n1, _variance(ys) / n2
    diff = _mean(xs) - _mean(ys)
    t2 = diff * diff / (a + b)
    df = (a + b) ** 2 / (a * a / (n1 - 1) + b * b / (n2 - 1))
    _check(t2, a, b, a * a, b * b, (a + b) ** 2)
    return t2, df


_TESTS: dict[int, Callable[[Sequence[float], Sequence[float]], tuple[mpmath.mpf, mpmath.mpf]]] = {
    PAIRED: _paired,
    EQUAL: _equal_variance,
    WELCH: _welch,
}


def _t_test(args: Sequence[MathArg]) -> mpmath.mpf:
    tails, test_type = args[2], args[3]
    assert is_real(tails) and is_real(test_type)
    if int(tails) not in (1, 2) or int(test_type) not in _TESTS:
        msg = "#NUM!: tails or type out of range"
        raise DomainError(msg)
    xs, ys = _samples(args)
    if int(test_type) == PAIRED and len(xs) != len(ys):
        msg = "#N/A: a paired test of arrays of different lengths"
        raise DomainError(msg)
    t2, df = _TESTS[int(test_type)](xs, ys)
    return int(tails) * student_t.lower_tail(-mpmath.sqrt(t2), df)


def _f_test(args: Sequence[MathArg]) -> mpmath.mpf:
    xs, ys = _samples(args)
    if len(xs) < 2 or len(ys) < 2:
        msg = "#DIV/0!: fewer than two numbers in a sample"
        raise DomainError(msg)
    _overflow_rule((xs, ys), evidenced=True)
    if constant_exact(xs) or constant_exact(ys):
        msg = "#DIV/0!: a sample has zero variance"
        raise DomainError(msg)
    v1, v2 = _variance(xs), _variance(ys)
    f = v1 / v2
    _check(f, v1, v2)
    d1, d2 = mpmath.mpf(len(xs) - 1), mpmath.mpf(len(ys) - 1)
    return 2 * min(fisher.lower_tail(f, d1, d2), fisher.upper_tail(f, d1, d2))


# -- draws -------------------------------------------------------------------------------------


def _noisy(rng: random.Random, n: int, centre: float, spread: float) -> tuple[float, ...]:
    return tuple(round15(centre + spread * rng.gauss(0, 1)) for _ in range(n))


def _two_noisy(rng: random.Random, n1: int, n2: int, separation: float) -> list[Arg]:
    """Two noisy samples whose means are `separation` spreads apart (a p from about 1 down to tiny)."""
    centre = log_uniform(rng, 1e-2, 1e4) * rng.choice((-1, 1))
    spread = abs(centre) * log_uniform(rng, 1e-3, 1.0)
    return [_noisy(rng, n1, centre, spread), _noisy(rng, n2, centre + separation * spread, spread)]


def _sizes(rng: random.Random, test_type: int, lo: int, hi: int) -> tuple[int, int]:
    n1 = rng.randint(lo, hi)
    return (n1, n1) if test_type == PAIRED else (n1, rng.randint(lo, hi))


def _typical_samples(rng: random.Random, test_type: int) -> list[Arg]:
    n1, n2 = _sizes(rng, test_type, 2, 10)
    if rng.random() < 0.5:
        return [array_of(rng, n1, 1e-3, 1e6), array_of(rng, n2, 1e-3, 1e6)]
    return _two_noisy(rng, n1, n2, log_uniform(rng, 1e-2, 3.0))


def _tail_samples(rng: random.Random, test_type: int) -> list[Arg]:
    shape = rng.randrange(3)
    n1, n2 = _sizes(rng, test_type, 3, 12)
    if shape == 0:  # far-apart means: a p in the deep tail
        return _two_noisy(rng, n1, n2, log_uniform(rng, 3.0, 100.0))
    if shape == 1:  # near-equal means: p near 1
        return _two_noisy(rng, n1, n2, log_uniform(rng, 1e-8, 1e-2))
    return [offset_series(rng, n1), offset_series(rng, n2)]


def _extreme_samples(rng: random.Random, test_type: int) -> list[Arg]:
    n1, n2 = _sizes(rng, test_type, 10, MAX_PAIRS)
    return [array_of(rng, n1, 1e-20, 1e20), array_of(rng, n2, 1e-20, 1e20)]


def _log_uniform_samples(rng: random.Random, test_type: int) -> list[Arg]:
    n1, n2 = _sizes(rng, test_type, 2, 6)
    return [array_of(rng, n1, MIN_MAGNITUDE, MAX_MAGNITUDE), array_of(rng, n2, MIN_MAGNITUDE, MAX_MAGNITUDE)]


def _edge_samples(rng: random.Random, test_type: int) -> list[Arg]:
    """The shapes at Excel's errors and the evidenced special values, for the test `test_type` (0 for F.TEST)."""
    n = rng.randint(2, 8)
    sample = array_of(rng, n, 1e-3, 1e6)
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [array_of(rng, 2, 1e-3, 1e6), array_of(rng, 2, 1e-3, 1e6)],  # two numbers each
        lambda: [sample, sample],  # identical samples: t = 0 (p = 1), f = 1
        lambda: [constant_series(rng, n), constant_series(rng, n)],  # both constant: #DIV/0!
        lambda: [array_of(rng, 1, 1e-3, 1e6), array_of(rng, 1, 1e-3, 1e6)],  # one number each: #DIV/0!
    ]
    if test_type == PAIRED:
        shift = constant_series(rng, 1)[0]
        shapes += [
            lambda: [
                sample,
                tuple(round15(v + shift) for v in sample),
            ],  # constant differences (often #DIV/0!)
            lambda: [array_of(rng, n + 1, 1e-3, 1e6), sample],  # #N/A: lengths differ
        ]
    else:
        shapes += [
            lambda: [array_of(rng, n + rng.randint(1, 5), 1e-3, 1e6), sample],  # unequal lengths are fine
            lambda: [
                sample,
                constant_series(rng, n),
            ],  # one constant: Welch a number, type 2 a number, F #DIV/0!
        ]
    if test_type in (0, EQUAL):
        shapes += [
            lambda: [sample, array_of(rng, 1, 1e-3, 1e6)],  # one number on one side: #DIV/0!
            lambda: [sample, OVERFLOW_SAMPLE],  # the evidenced overflow: #NUM!
        ]
    return one_of(rng, shapes)


def _with_tails_type(
    make: Callable[[random.Random, int], list[Arg]], edge: bool
) -> Callable[[random.Random], list[Arg]]:
    def draw(rng: random.Random) -> list[Arg]:
        if edge and rng.random() < 0.2:  # tails or type out of range: #NUM!
            test_type = rng.choice((PAIRED, EQUAL, WELCH))
            bad = rng.choice(((0, test_type), (3, test_type), (2, 0), (1, 4)))
            return [*_typical_samples(rng, test_type), *bad]
        test_type = rng.choice((PAIRED, EQUAL, WELCH))
        return [*make(rng, test_type), rng.choice((1, 2)), test_type]

    return draw


def _without_tails_type(
    make: Callable[[random.Random, int], list[Arg]],
) -> Callable[[random.Random], list[Arg]]:
    def draw(rng: random.Random) -> list[Arg]:
        return make(rng, 0)

    return draw


_MAKERS: dict[str, Callable[[random.Random, int], list[Arg]]] = {
    "typical": _typical_samples,
    "tail": _tail_samples,
    "edge": _edge_samples,
    "extreme": _extreme_samples,
    "log-uniform": _log_uniform_samples,
}
_T_DRAWS: StratumDraws = {name: _with_tails_type(make, name == "edge") for name, make in _MAKERS.items()}
_F_DRAWS: StratumDraws = {name: _without_tails_type(make) for name, make in _MAKERS.items()}
_T_DRAW = by_stratum(
    _T_DRAWS, "T.TEST draws: two arrays (equal lengths for type 1), tails and type by stratum."
)
_F_DRAW = by_stratum(
    _F_DRAWS, "F.TEST draws: two arrays of two or more numbers by stratum; edge shapes from statistics.md."
)

T_SOURCES = (
    TTEST_Y_SOURCE,
    THIN_SOURCE,
    LEGACY_SOURCE,
    SINGLE_SOURCE,
    OVERFLOW_SOURCE,
    ORDER_SOURCE,
    "Microsoft's T.TEST page",
    "decision 0271",
)
F_SOURCES = (
    FTEST_SOURCE,
    THIN_SOURCE,
    FTEST_ERROR_SOURCE,
    FTEST_SINGLE_SOURCE,
    "Microsoft's F.TEST page",
    "Abramowitz and Stegun 26.6.2",
    "decision 0271",
)


def _t_definition(function: str) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("array1", "array", source=THIN_SOURCE),
                Param("array2", "array", source=THIN_SOURCE),
                Param(
                    "tails",
                    "int",
                    model.Interval(1, 2, lo_open=False, hi_open=False),
                    source=THIN_SOURCE,
                    truncates=THIN_SOURCE,
                ),
                Param(
                    "type",
                    "int",
                    model.Interval(1, 3, lo_open=False, hi_open=False),
                    source=THIN_SOURCE,
                    truncates=THIN_SOURCE,
                ),
            ),
            value=_t_test,
            domain=domain_of(_t_test),
            draw=_T_DRAW,
            sources=T_SOURCES,
        )
    )


def _f_definition(function: str) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("array1", "array", source=FTEST_SOURCE),
                Param("array2", "array", source=FTEST_SOURCE),
            ),
            value=_f_test,
            domain=domain_of(_f_test),
            draw=_F_DRAW,
            sources=F_SOURCES,
        )
    )


T_TEST = _t_definition("T.TEST")
TTEST = _t_definition("TTEST")
F_TEST = _f_definition("F.TEST")
FTEST = _f_definition("FTEST")
