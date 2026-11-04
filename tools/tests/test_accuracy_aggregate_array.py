"""Gate A definition of AGGREGATE's array forms, function_num 14-19 (T-3693), against probes/aggregate-grid."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.aggregate_array import AGGREGATE
from rustsheet_tools.accuracy.model import DomainError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg

mp = mpmath.mpf
DATA = (3, 1, 4, 1, 5, 9)


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def value(function_num: int, data: tuple[float, ...], k: float, options: int = 0) -> mpmath.mpf:
    return AGGREGATE.evaluate([mp(function_num), mp(options), tuple(mp(x) for x in data), mp(k)])


def test_probes_aggregate_grid_h41_h46() -> None:
    # Excel (excel-graph, T-0993): 5, 1, 2, 1.5, 1.2000000000000002, 1
    assert value(14, DATA, 2) == 5
    assert value(15, DATA, 2) == 1
    # rank 1 + 0.3*5 = 2.5 between 1 and 3, to the double of 0.3 (a hair below 2)
    assert abs(value(16, DATA, 0.3) - 2) < mp(1e-15)
    assert value(17, DATA, 1) == 1.5
    h45 = value(18, DATA, 0.3)  # rank 0.3*7 = 2.1: 1 + 0.1*2
    assert h45 == 1 + (mp(0.3) * 7 - 2) * 2
    assert abs(h45 - mp(1.2000000000000002)) < mp(1e-15)
    assert value(19, DATA, 1) == 1  # rank 7/4 = 1.75 between 1 and 1


def test_options_do_not_change_an_array_literal() -> None:
    for options in range(8):
        assert value(14, DATA, 2.9, options) == 4  # the 3rd largest (statistics.md LARGE/SMALL)


def test_large_small_fractional_k() -> None:
    # statistics.md "LARGE/SMALL: a fractional k rounds up for LARGE and down for SMALL"
    assert value(14, DATA, 2.0000000001) == 4
    assert value(15, DATA, 2.9) == 1
    assert value(15, DATA, 3.5) == 3
    assert value(15, DATA, 5.9) == 5
    for function_num, k in ((14, 6.1), (15, 6.5), (14, 0.5), (15, 0), (14, -0.5)):
        with pytest.raises(DomainError):
            value(function_num, DATA, k)


def test_percentile_and_quartile_domains() -> None:
    for function_num, data, k in (
        (16, DATA, -0.1),
        (16, DATA, 1.1),
        (18, DATA, 0),
        (18, DATA, 1),
        (18, DATA, 0.1),  # rank 0.7 below 1
        (18, DATA, 0.9),  # rank 6.3 above 6
        (17, DATA, 5),
        (17, DATA, -1),
        (19, DATA, 0),
        (19, DATA, 4),
        (19, (1, 2), 1),  # rank 0.75 below 1
    ):
        with pytest.raises(DomainError):
            value(function_num, data, k)
    assert value(17, DATA, 4.5) == 9  # quart = floor(4.5) = 4: the largest
    assert value(17, DATA, 0) == 1
    assert value(18, (1, 3, 2), 0.25) == 1  # rank exactly 1
    assert value(18, (1, 3, 2), 0.75) == 3  # rank exactly n
    assert value(18, (7,), 0.5) == 7
    assert value(16, (7,), 0.4) == 7


def test_draws_are_valid_literals_and_stay_short() -> None:
    sampler = Sampler(AGGREGATE, random.Random(7))
    for stratum in STRATA:
        for index in range(40):
            args = sampler.draw(stratum, index)
            assert len(args) == 4
            assert 14 <= int(args[0]) <= 19 and 0 <= int(args[1]) <= 7  # type: ignore[arg-type]
            array = args[2]
            assert isinstance(array, tuple) and 1 <= len(array) <= 150
            margs = [math_arg(p, a) for p, a in zip(AGGREGATE.params, args, strict=True)]
            if stratum != "edge":
                AGGREGATE.evaluate(margs)  # every non-edge draw is valid
