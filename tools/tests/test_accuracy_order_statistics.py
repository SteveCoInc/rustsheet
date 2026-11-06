"""Gate A definitions of the order statistics (T-4092) against Microsoft's examples and statistics.md."""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.order_statistics import (
    MEDIAN,
    PERCENTILE,
    PERCENTILE_EXC_DEF,
    PERCENTILE_INC_DEF,
    QUARTILE,
    QUARTILE_EXC_DEF,
    QUARTILE_INC_DEF,
)
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg

mp = mpmath.mpf
GRID = (3, 1, 4, 1, 5, 9)
ORDER = [PERCENTILE, PERCENTILE_INC_DEF, PERCENTILE_EXC_DEF, QUARTILE, QUARTILE_INC_DEF, QUARTILE_EXC_DEF]


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def at(definition: Definition, data: Sequence[float], k: float) -> mpmath.mpf:
    return definition.evaluate([tuple(mp(x) for x in data), mp(k)])


def median(*xs: float) -> mpmath.mpf:
    return MEDIAN.evaluate([mp(x) for x in xs])


def test_median() -> None:
    # Microsoft: MEDIAN(1,2,3,4,5) is 3 and MEDIAN(1,2,3,4,5,6) is 3.5.
    assert median(1, 2, 3, 4, 5) == 3
    assert median(1, 2, 3, 4, 5, 6) == mp("3.5")
    # statistics.md: Excel's a + (b - a)/2 of -0.5 and 0.3 is -0.09999999999999998; the math is the exact midpoint
    assert median(0.3, -0.5) == (mp(0.3) + mp(-0.5)) / 2
    assert median(9e307, 9e307) == mp(9e307)  # probes/aggregate-overflow A48
    with pytest.raises(NoValueError):  # b - a beyond the double range: unprobed
        median(-1.7e308, 1.7e308)


@pytest.mark.parametrize("definition", [PERCENTILE, PERCENTILE_INC_DEF])
def test_percentile_inc(definition: Definition) -> None:
    assert abs(at(definition, (1, 3, 2, 4), 0.3) - mp("1.9")) < mp(1e-15)  # Microsoft: 1.9
    assert abs(at(definition, GRID, 0.3) - 2) < mp(1e-15)  # aggregate-grid H43: 2
    assert at(definition, (0, 5, 7), 1e-20) == mp(1e-20) * 2 * 5  # Excel 0: the rank loses k (statistics.md)
    assert at(definition, (7,), 0.4) == 7
    for k in (-0.1, 1.1, -1e-10):
        with pytest.raises(DomainError):
            at(definition, GRID, k)


def test_percentile_exc() -> None:
    data = (1, 2, 3, 6, 6, 6, 7, 8, 9)
    assert at(PERCENTILE_EXC_DEF, data, 0.25) == mp("2.5")  # Microsoft: 2.5
    for k in (0, 0.01, 2):  # Microsoft: #NUM!
        with pytest.raises(DomainError):
            at(PERCENTILE_EXC_DEF, data, k)
    assert at(PERCENTILE_EXC_DEF, (1, 3, 2), 0.25) == 1  # rank exactly 1
    assert at(PERCENTILE_EXC_DEF, (1, 3, 2), 0.75) == 3  # rank exactly n
    assert at(PERCENTILE_EXC_DEF, (7,), 0.5) == 7
    h45 = at(PERCENTILE_EXC_DEF, GRID, 0.3)  # aggregate-grid H45: 1.2000000000000002
    assert abs(h45 - mp(1.2000000000000002)) < mp(1e-15)


@pytest.mark.parametrize("definition", [QUARTILE, QUARTILE_INC_DEF])
def test_quartile_inc(definition: Definition) -> None:
    data = (1, 2, 4, 7, 8, 9, 10, 12)
    assert at(definition, data, 1) == mp("3.5")  # Microsoft: 3.5
    assert at(definition, GRID, 1) == mp("1.5")  # aggregate-grid H44
    assert at(definition, GRID, 4.5) == 9  # quart = floor(4.5) = 4
    assert at(definition, GRID, 0) == 1
    for quart in (5, -1):
        with pytest.raises(DomainError):
            at(definition, GRID, quart)


def test_quartile_exc() -> None:
    data = (6, 7, 15, 36, 39, 40, 41, 42, 43, 47, 49)
    assert at(QUARTILE_EXC_DEF, data, 1) == 15  # Microsoft: 15
    assert at(QUARTILE_EXC_DEF, data, 3) == 43  # Microsoft: 43
    assert at(QUARTILE_EXC_DEF, GRID, 1) == 1  # aggregate-grid H46
    assert at(QUARTILE_EXC_DEF, (5,), 2) == 5  # statistics.md T-1748: QUARTILE.EXC(TRUE,2) is 1
    for data_, quart in ((GRID, 0), (GRID, 4), ((5,), 1), ((1, 2), 1)):  # QUARTILE.EXC(TRUE,1) is #NUM!
        with pytest.raises(DomainError):
            at(QUARTILE_EXC_DEF, data_, quart)


@pytest.mark.parametrize("definition", ORDER)
def test_draws_are_valid_outside_the_edge_stratum(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(4092))
    for stratum in STRATA:
        for index in range(40):
            args = sampler.draw(stratum, index)
            assert len(args) == 2
            array = args[0]
            assert isinstance(array, tuple) and 1 <= len(array) <= 150
            margs = [math_arg(p, a) for p, a in zip(definition.params, args, strict=True)]
            if stratum != "edge":
                definition.evaluate(margs)
