"""Gate A definitions of KURT, SKEW, SKEW.P, GEOMEAN, HARMEAN, AVERAGEA and TRIMMEAN (T-4091) against Microsoft's
examples and the semantics entries."""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.shape_means import (
    AVERAGEA,
    GEOMEAN,
    HARMEAN,
    KURT,
    SKEW,
    SKEW_P,
    TRIMMEAN,
)
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError

mpmath.mp.dps = 50
MS_DATA = [3, 4, 5, 2, 3, 4, 5, 6, 4, 7]  # Microsoft's SKEW, SKEW.P and KURT example
MS_MEANS = [4, 5, 8, 7, 11, 4, 3]  # Microsoft's GEOMEAN and HARMEAN example
MS_TRIMMEAN = [4, 5, 6, 7, 2, 3, 4, 5, 1, 2, 3]
DEFINITIONS = [KURT, SKEW, SKEW_P, GEOMEAN, HARMEAN, AVERAGEA, TRIMMEAN]


def mp(xs: Sequence[float]) -> list[mpmath.mpf]:
    return [mpmath.mpf(x) for x in xs]


def close(value: mpmath.mpf, expected: str, tol: str = "1e-9") -> bool:
    return abs(value - mpmath.mpf(expected)) < mpmath.mpf(tol)


def test_microsoft_examples() -> None:
    assert close(KURT.evaluate(mp(MS_DATA)), "-0.151799637")
    assert close(SKEW.evaluate(mp(MS_DATA)), "0.359543071")
    assert close(SKEW_P.evaluate(mp(MS_DATA)), "0.303193339")
    assert close(GEOMEAN.evaluate(mp(MS_MEANS)), "5.476986969")
    assert close(HARMEAN.evaluate(mp(MS_MEANS)), "5.028375962")
    assert AVERAGEA.evaluate(mp([10, 7, 9, 2])) == 7
    assert TRIMMEAN.evaluate([tuple(mp(MS_TRIMMEAN)), mpmath.mpf(0.2)]) == mpmath.mpf(34) / 9


def test_semantics_values() -> None:
    # statistics.md "SKEW.P of two numbers is #DIV/0!": SKEW.P(1,2,3) is 0, SKEW.P(1,2,4) 0.3818017741606059.
    assert SKEW_P.evaluate(mp([1, 2, 3])) == 0
    assert close(SKEW_P.evaluate(mp([1, 2, 4])), "0.3818017741606059", "1e-15")
    # probes/stats-evidence-floor-t1623: KURT(1,1,1,2) is 4, KURT(1,2,3,4) -1.1999999999999975 (math -1.2),
    # KURT of 3,5,2,8,9,4,7,1 and TRUE -1.4331834319526635, HARMEAN(1,2,4) 1.7142857142857142.
    assert KURT.evaluate(mp([1, 1, 1, 2])) == 4
    assert close(KURT.evaluate(mp([1, 2, 3, 4])), "-1.2", "1e-40")
    assert close(KURT.evaluate(mp([3, 5, 2, 8, 9, 4, 7, 1, 1])), "-1.4331834319526635", "1e-14")
    assert HARMEAN.evaluate(mp([1, 2, 4])) == mpmath.mpf(12) / 7
    # numbers.md "GEOMEAN in-range root": GEOMEAN(3,3) and GEOMEAN(1,9) are 3.
    assert close(GEOMEAN.evaluate(mp([3, 3])), "3", "1e-40")
    assert close(GEOMEAN.evaluate(mp([1, 9])), "3", "1e-40")
    # numbers.md "Near-zero snapping": AVERAGEA(0.1,0.2,-0.3) is 0.
    assert AVERAGEA.evaluate(mp([0.1, 0.2, -0.3])) == 0


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (SKEW, [1.0, 2.0]),  # fewer than three numbers
        (SKEW_P, [1.0, 2.0]),
        (KURT, [3.0, 5.0, 2.0]),  # KURT(A1:A3), stats-evidence-floor-t1623 Q29
        (SKEW_P, [5.0, 5.0, 5.0]),  # zero variance
        (SKEW, [2.0, 2.0, 2.0]),  # SKEW(N1:N3), Q4
        (KURT, [1e200, 1e200, -1e200, 1.0]),  # overflowing squares (T-2432)
        (GEOMEAN, [4.0, 0.0]),  # GEOMEAN(4,)
        (GEOMEAN, [-4.0, 3.0, 2.0]),  # ironcalc-calc GEOMEAN D11
        (HARMEAN, [0.0]),  # Q51
        (HARMEAN, [-1.0, -2.0]),  # Q52
    ],
)
def test_domain_errors(definition: Definition, args: list[float]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate(mp(args))


@pytest.mark.parametrize("percent", [1.0, 1.5, -0.1, -1.0])
def test_trimmean_percent_domain(percent: float) -> None:
    with pytest.raises(DomainError):
        TRIMMEAN.evaluate([tuple(mp([1, 2, 3])), mpmath.mpf(percent)])


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (SKEW, [0.1, 0.1, 0.1]),  # equal numbers whose double sum is inexact: Excel's deviations are not 0
        (KURT, [9e307, 9e307, 1.0, 2.0]),  # the sum of the numbers overflows
        (HARMEAN, [1e-310, 1.0]),  # a reciprocal overflows
        (AVERAGEA, [9e307, 9e307, -9e307]),  # a running total overflows (unprobed for AVERAGEA)
    ],
)
def test_unprobed_regions_have_no_value(definition: Definition, args: list[float]) -> None:
    with pytest.raises(NoValueError):
        definition.evaluate(mp(args))


def test_trimmean_trim_count() -> None:
    # n = 10, percent = 0.6: the exact n * percent is just below 6 (k = 2), its double 6 (k = 3): unprobed.
    with pytest.raises(NoValueError):
        TRIMMEAN.evaluate([tuple(mp(range(10))), mpmath.mpf(0.6)])
    # n = 10, percent = 0.4: both give k = 2, so 2..7 are kept.
    assert TRIMMEAN.evaluate([tuple(mp(range(10))), mpmath.mpf(0.4)]) == mpmath.mpf(9) / 2
    assert TRIMMEAN.evaluate([(mpmath.mpf(5),), mpmath.mpf(0.99)]) == 5


@pytest.mark.parametrize("definition", DEFINITIONS)
@pytest.mark.parametrize("stratum", ["typical", "tail", "edge", "extreme", "log-uniform"])
def test_draws_evaluate(definition: Definition, stratum: str) -> None:
    assert definition.draw is not None
    rng = random.Random(4091)
    values = 0
    for _ in range(40):
        args: list[MathArg] = [
            tuple(mp(a)) if isinstance(a, tuple) else mpmath.mpf(a) for a in definition.draw(stratum, rng)
        ]
        try:
            definition.evaluate(args)
            values += 1
        except (DomainError, NoValueError):
            pass
    assert values > 0
