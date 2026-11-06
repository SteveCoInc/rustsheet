"""Gate A definitions of PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC (T-4093) against Microsoft's examples and
the statistics.md cells (probes/t2473-percentrank-significance, probes/percentrank-single-value)."""

from __future__ import annotations

import contextlib
import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.percentrank import (
    PERCENTRANK,
    PERCENTRANK_EXC,
    PERCENTRANK_INC,
    _significance_of,
)
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg

mp = mpmath.mpf
INC = [PERCENTRANK, PERCENTRANK_INC]
ALL = [*INC, PERCENTRANK_EXC]


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def at(definition: Definition, data: Sequence[float], x: float, *significance: int) -> mpmath.mpf:
    return definition.evaluate([tuple(mp(v) for v in data), mp(x), *(mp(s) for s in significance)])


@pytest.mark.parametrize("definition", INC)
def test_microsoft_inc(definition: Definition) -> None:
    data = (13, 12, 11, 8, 4, 3, 2, 1, 1, 1)
    for x, expected in ((2, "0.333"), (4, "0.555"), (8, "0.666"), (5, "0.583")):
        assert at(definition, data, x) == mp(expected)


def test_microsoft_exc() -> None:
    data = (1, 2, 3, 6, 6, 6, 7, 8, 9)
    assert at(PERCENTRANK_EXC, data, 7) == mp("0.7")
    assert at(PERCENTRANK_EXC, data, 5.43) == mp("0.381")
    assert at(PERCENTRANK_EXC, data, 5.43, 1) == mp("0.3")


@pytest.mark.parametrize("significance", [15, 16, 17, 20, 100, 308])
def test_t2473_many_digits(significance: int) -> None:
    # t2473 A1-A6, B1-B6, C1-C6: 0.5
    assert at(PERCENTRANK, (1, 2, 3, 4), 2.5, significance) == mp("0.5")
    assert at(PERCENTRANK_EXC, (1, 2, 3, 4), 2.5, significance) == mp("0.5")
    assert at(PERCENTRANK, (1, 2, 3), 2, significance) == mp("0.5")


@pytest.mark.parametrize("definition", ALL)
def test_huge_significance_is_a_domain_error(definition: Definition) -> None:
    for significance in (309, 400, 2147483648):  # t2473 A7-A15: #DIV/0!
        with pytest.raises(DomainError):
            at(definition, (1, 2, 3, 4), 2.5, significance)


def test_the_cut_keeps_a_digit_within_rounding() -> None:
    # t2473 D1-D4: the exact rank 0.4999999999999998... is 0.5 in Excel
    for significance in ((), (15,), (16,), (20,)):
        assert at(PERCENTRANK_EXC, (1e15, 1, -1e15), 0.3, *significance) == mp("0.5")
    # t2473 D5-D18: Excel's values at the written decimals
    for x, significance, expected in (
        (0.36, 3, "0.36"),
        (0.36, 2, "0.36"),
        (0.29, 2, "0.29"),
        (0.57, 2, "0.57"),
        (0.58, 2, "0.58"),
        (0.1 + 0.2, 1, "0.3"),
        (0.1 + 0.2, 17, "0.3"),
        (0.999999999999999, 3, "0.999"),
        (0.499999999999999, 3, "0.499"),
    ):
        assert at(PERCENTRANK, (0, 1), x, significance) == mp(expected)
    # t2473 D19-D23: from 15 digits on the cut rounds the last digit
    assert at(PERCENTRANK, (0, 3), 1, 15) == mp("0.333333333333333")
    assert at(PERCENTRANK, (0, 3), 2, 15) == mp("0.666666666666667")
    assert at(PERCENTRANK, (0, 7), 1, 20) == mp("0.142857142857143")


def test_a_cut_inside_the_noise_has_no_value() -> None:
    # The exact scaled rank 49.99999999999995004 sits at the cut's half unit below 50: the digit is the doubles' call.
    with pytest.raises(NoValueError):
        at(PERCENTRANK, (0.0, 1.0), 0.4999999999999995, 2)


@pytest.mark.parametrize("definition", ALL)
def test_domain(definition: Definition) -> None:
    for x in (0.5, 4.5):  # outside the data: #N/A
        with pytest.raises(DomainError):
            at(definition, (1, 2, 3, 4), x)
    for significance in (0, -1):  # #NUM!
        with pytest.raises(DomainError):
            at(definition, (1, 2, 3, 4), 2.5, significance)
    # x at either end is in range
    low, high = at(definition, (1, 2, 3, 4), 1), at(definition, (1, 2, 3, 4), 4)
    assert (low, high) == ((mp("0.2"), mp("0.8")) if definition is PERCENTRANK_EXC else (0, 1))


@pytest.mark.parametrize("definition", ALL)
def test_single_value(definition: Definition) -> None:
    # probes/percentrank-single-value: one value equal to x is 1 whatever numeric significance; x beside it #N/A
    for significance in ((), (0,), (-1,), (3,)):
        assert at(definition, (5,), 5, *significance) == 1
    with pytest.raises(DomainError):
        at(definition, (5,), 4)
    with pytest.raises(NoValueError):  # unprobed
        at(definition, (5,), 5, 309)


@pytest.mark.parametrize("definition", ALL)
def test_draws_are_valid_outside_the_edge_stratum(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(4093))
    for stratum in STRATA:
        for index in range(40):
            args = sampler.draw(stratum, index)
            assert len(args) in (2, 3)
            array = args[0]
            assert isinstance(array, tuple) and 1 <= len(array) <= 150
            margs = [math_arg(p, a) for p, a in zip(definition.params_for(len(args)), args, strict=True)]
            if stratum != "edge":
                with contextlib.suppress(NoValueError):  # a cut inside the noise is redrawn
                    definition.evaluate(margs)


@pytest.mark.parametrize("definition", ALL)
def test_a_tied_lower_neighbour_interpolates_from_its_last_copy(definition: Definition) -> None:
    # probes/t4093-percentrank-tied-neighbour (excel-graph): A1:C8 at 15 digits
    exclusive = definition is PERCENTRANK_EXC
    for data, x, inc, exc in (
        ((1, 2, 2, 2, 3), 2.5, "0.875", "0.75"),
        ((1, 1, 1, 3), 2, "0.833333333333333", "0.7"),
        ((1, 2, 2, 3), 2.25, "0.75", "0.65"),
        ((1, 1, 2, 2, 5), 1.5, "0.375", "0.416666666666667"),
    ):
        assert at(definition, data, x, 15) == mp(exc if exclusive else inc)


def test_a_fractional_or_non_finite_significance_has_no_value() -> None:
    with pytest.raises(NoValueError):
        PERCENTRANK.evaluate([(mp(1), mp(2), mp(3)), mp(2), mp(2.5)])
    for significance in (mpmath.inf, mpmath.nan):  # `evaluate` refuses these first; the helper must too
        with pytest.raises(NoValueError):
            _significance_of(significance)
