"""Gate A definitions of FORECAST, FORECAST.LINEAR, INTERCEPT, SLOPE, RSQ, PEARSON and CORREL (T-4094) against
Microsoft's published examples, the semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.paired_regression import (
    CORREL,
    FORECAST,
    FORECAST_LINEAR,
    INTERCEPT,
    PEARSON,
    RSQ,
    SLOPE,
)
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg

mp = mpmath.mpf
PAIRED = [SLOPE, INTERCEPT, RSQ, CORREL, PEARSON]
EVERY = [*PAIRED, FORECAST, FORECAST_LINEAR]
# probes/regression-pairs-a (excel-graph): A1:A6, B1:B6 and the large offset I1:I6
A = (2, 4, 5, 4, 5, 7)
B = (1, 2, 3, 4, 5, 6)
OFFSET = (10000000001, 10000000002, 10000000004, 10000000003, 10000000005, 10000000007)


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def arrays(*data: Sequence[float]) -> list[tuple[mpmath.mpf, ...]]:
    return [tuple(mp(x) for x in a) for a in data]


def pair(definition: Definition, y: Sequence[float], x: Sequence[float]) -> mpmath.mpf:
    return definition.evaluate(arrays(y, x))


def forecast(definition: Definition, at: float, y: Sequence[float], x: Sequence[float]) -> mpmath.mpf:
    return definition.evaluate([mp(at), *arrays(y, x)])


def test_microsofts_examples() -> None:
    y, x = (2, 3, 9, 1, 8, 7, 5), (6, 5, 11, 7, 5, 4, 4)
    assert float(pair(SLOPE, y, x)) == pytest.approx(0.305556, abs=5e-7)
    assert float(pair(RSQ, y, x)) == pytest.approx(0.05795, abs=5e-6)
    assert float(pair(INTERCEPT, (2, 3, 9, 1, 8), (6, 5, 11, 7, 5))) == pytest.approx(0.0483871, abs=5e-8)
    assert float(pair(CORREL, (3, 2, 4, 5, 6), (9, 7, 12, 15, 17))) == pytest.approx(0.997054, abs=5e-7)
    assert float(pair(PEARSON, (9, 7, 5, 3, 1), (10, 6, 1, 5, 3))) == pytest.approx(0.699379, abs=5e-7)
    for definition in (FORECAST, FORECAST_LINEAR):
        value = forecast(definition, 30, (6, 7, 9, 15, 21), (20, 28, 31, 38, 40))
        assert float(value) == pytest.approx(10.607253, abs=5e-7)


def test_excel_values_of_the_semantics_entries() -> None:
    # statistics.md "Paired regression functions": the large offset keeps its variance (pairs-a L21, O22;
    # pairs-b N22; forecast L34). Excel's two-pass doubles are within 1e-13 of the exact values.
    assert pair(CORREL, OFFSET, B) == pytest.approx(0.9402561526801988, rel=1e-13)
    assert pair(SLOPE, A, OFFSET) == pytest.approx(0.7285714285713528, rel=2e-13)
    assert pair(INTERCEPT, A, OFFSET) == pytest.approx(-7285714283.884956, rel=2e-13)
    assert forecast(FORECAST, 7, A, OFFSET) == pytest.approx(-7285714278.784956, rel=2e-13)
    # FORECAST(0,A1:A6,B1:B6) is 1.7999999999999998, FORECAST.LINEAR(7,..) 7.2: exactly 1.8 and 7.2
    assert forecast(FORECAST, 0, A, B) == mp(9) / 5
    assert forecast(FORECAST_LINEAR, 7, A, B) == mp(36) / 5
    # an exact line: r is exactly 1 (Excel's CORREL of {1.5,..,6.5} against 1..6 is 1, RSQ 1.0000000000000004)
    assert pair(CORREL, (1.5, 2.5, 4.5, 5.5, 6.5), (1, 2, 4, 5, 6)) == 1
    assert pair(RSQ, (1.5, 2.5, 4.5, 5.5, 6.5), (1, 2, 4, 5, 6)) == 1
    assert pair(CORREL, (3, 2, 1), (1, 2, 3)) == -1


def test_zero_variance_in_y() -> None:
    # pairs-a O7: SLOPE of a constant y is 0; INTERCEPT and FORECAST are the mean of y
    assert pair(SLOPE, (3, 3, 3, 3), B[:4]) == 0
    assert pair(INTERCEPT, (3, 3, 3, 3), B[:4]) == 3
    assert forecast(FORECAST, 99, (3, 3, 3, 3), B[:4]) == 3
    for definition in (RSQ, CORREL, PEARSON):  # pairs-a L7: #DIV/0!
        with pytest.raises(DomainError):
            pair(definition, (3, 3, 3, 3), B[:4])


@pytest.mark.parametrize("definition", PAIRED)
def test_domain_errors_follow_excel(definition: Definition) -> None:
    cases: list[tuple[Sequence[float], Sequence[float]]] = [
        ((2, 4, 5), (1, 2)),  # #N/A: CORREL({2,4,5},{1,2}), pairs-a L17
        ((2,), (1,)),  # one pair: #DIV/0! (pairs-a L3)
        (A, (4, 4, 4, 4, 4, 4)),  # x constant: #DIV/0! (pairs-a L6, O6)
        ((0.3, 0.3, 0.3), (1, 2, 3))
        if definition in (RSQ, CORREL, PEARSON)
        else ((1, 2, 3), (0.3, 0.3, 0.3)),
        ((1, 2, 3, 4), (1e200, 2e200, 3e200, 5e200)),  # x's sum about the mean overflows: #NUM! (T-4047)
        ((1, 2, 3, 4), (1.5e308, 1.6e308, 1.4e308, 1.55e308)),  # the sum of x overflows: #NUM! (T-4047)
    ]
    for y, x in cases:
        with pytest.raises(DomainError):
            pair(definition, y, x)


def test_overflow_in_y_alone() -> None:
    # T-4047: y = 1E200, 2E200, 3E200, 5E200 on x = 1..4: CORREL, PEARSON, RSQ #NUM!; SLOPE 1.3E200, INTERCEPT -5E199,
    # FORECAST(2,..) 2.1E200
    y, x = (1e200, 2e200, 3e200, 5e200), (1, 2, 3, 4)
    for definition in (RSQ, CORREL, PEARSON):
        with pytest.raises(DomainError):
            pair(definition, y, x)
    assert pair(SLOPE, y, x) == pytest.approx(1.3e200, rel=1e-15)
    assert pair(INTERCEPT, y, x) == pytest.approx(-5e199, rel=1e-15)
    assert forecast(FORECAST, 2, y, x) == pytest.approx(2.1e200, rel=1e-15)


def test_unprobed_points_have_no_value() -> None:
    # "Inexact constants": CORREL({0.1,0.1,0.1},{1,2,3}) is 0 in Excel, where the exact variance is 0
    for definition in EVERY:
        args = (
            arrays((1, 2, 3), (0.1, 0.1, 0.1))
            if definition in PAIRED
            else [mp(1), *arrays((1, 2, 3), (0.1,) * 3)]
        )
        with pytest.raises(NoValueError):
            definition.evaluate(args)
    with pytest.raises(NoValueError):  # a subnormal literal
        pair(SLOPE, (1, 2), (1e-310, 1))
    with pytest.raises(NoValueError):  # a deviation square below the normal range
        pair(CORREL, (1, 2), (1e-160, 2e-160))


@pytest.mark.parametrize("definition", EVERY)
def test_draws_are_reproducible_and_evaluate(definition: Definition) -> None:
    for stratum in STRATA:
        points = []
        for seed in range(2):
            sampler = Sampler(definition, random.Random(seed))
            points.append([sampler.draw(stratum, i) for i in range(12)])
        assert points[0] != points[1] or stratum == "edge"
        again = Sampler(definition, random.Random(0))
        assert [again.draw(stratum, i) for i in range(12)] == points[0]
        for args in points[0]:
            params = definition.params_for(len(args))
            math_args = [math_arg(p, a) for p, a in zip(params, args, strict=True)]
            try:
                definition.evaluate(math_args)
            except DomainError:
                assert stratum == "edge"  # only the edge stratum aims at Excel's errors
            except NoValueError:
                pass  # the generator redraws
