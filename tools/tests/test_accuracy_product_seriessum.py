"""Gate A definitions of PRODUCT, SUMSQ, SERIESSUM and PERCENTOF (T-3676) against the values numbers.md cites."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.percentof import PERCENTOF
from rustsheet_tools.accuracy.defs.product import PRODUCT, SUMSQ, running_product
from rustsheet_tools.accuracy.defs.seriessum import SERIESSUM
from rustsheet_tools.accuracy.model import DomainError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def arr(*xs: float) -> tuple[mpmath.mpf, ...]:
    return tuple(mp(x) for x in xs)


def test_product_and_sumsq_overflow_at_an_intermediate_point() -> None:
    # numbers.md "Overflow in `+ - * /` is #NUM! where it happens" (T-0990)
    for args in ((1e200, 1e200), (1e200, 1e200, 0), (1e200, 1e200, 1e-200)):
        with pytest.raises(DomainError):
            PRODUCT.evaluate([mp(a) for a in args])
    assert PRODUCT.evaluate([mp(1e200), mp(1e-200), mp(1e200)]) == mp(1e200) * mp(1e-200) * mp(1e200)
    with pytest.raises(DomainError):
        SUMSQ.evaluate([mp(1e200)])
    assert running_product([2.0, 3.0, 0.5]) == [2.0, 6.0, 3.0]


def test_product_and_sumsq_are_exact_on_the_doubles() -> None:
    assert PRODUCT.evaluate([mp(0.1), mp(3)]) == mp(0.1) * 3
    assert SUMSQ.evaluate([mp(3), mp(4)]) == 25
    assert SUMSQ.evaluate([mp(-0.5)]) == mp(0.25)


def test_seriessum_x_zero() -> None:
    # numbers.md "SERIESSUM with x = 0": SERIESSUM(0,1,1,{1,2}) is 0; (0,0,2,{1,2,3}) and (0,1,-2,{1,2}) are #NUM!
    assert SERIESSUM.evaluate([mp(0), mp(1), mp(1), arr(1, 2)]) == 0
    for n, m, coefficients in ((0, 2, (1, 2, 3)), (1, -2, (1, 2)), (0, 1, (0,))):
        with pytest.raises(DomainError):
            SERIESSUM.evaluate([mp(0), mp(n), mp(m), arr(*coefficients)])


def test_seriessum_negative_x_non_integer_power_is_a_zero_term() -> None:
    # numbers.md T-2380: SERIESSUM(-2,0.5,1,{1,2}) is 0 and (-2,1,0.5,{1,1,1}) is -2 + 0 + 4 = 2
    assert SERIESSUM.evaluate([mp(-2), mp(0.5), mp(1), arr(1, 2)]) == 0
    assert SERIESSUM.evaluate([mp(-2), mp(1), mp(0.5), arr(1, 1, 1)]) == 2


def test_seriessum_values_and_overflow() -> None:
    # numbers.md T-1802: INDEX(SERIESSUM({1;2},0,1,{1,2}),2) = 5; T-2163: a term past the double range is #NUM!
    assert SERIESSUM.evaluate([mp(2), mp(0), mp(1), arr(1, 2)]) == 5
    assert (
        SERIESSUM.evaluate([mp(97), mp(0), mp(0.5), arr(42, 2.5, 2)])
        == 42 + mp(2.5) * mp(97) ** mp(0.5) + 2 * 97
    )
    with pytest.raises(DomainError):
        SERIESSUM.evaluate([mp(1e-10), mp(-33.5), mp(1), arr(1)])


def test_percentof_reads_its_sums_as_sum_does() -> None:
    # probes/percentof-snap-overflow-union (T-0416): D4 0, D2 #DIV/0!, D5, D6 #NUM!, D15 -0.5
    assert PERCENTOF.evaluate([arr(0.1, 0.2, -0.3), arr(1)]) == 0
    with pytest.raises(DomainError):
        PERCENTOF.evaluate([arr(0.1, 0.2), arr(0.1, 0.2, -0.3)])
    assert float(PERCENTOF.evaluate([arr(0.1), arr(0.3)])) == 0.33333333333333337
    with pytest.raises(DomainError):
        PERCENTOF.evaluate([arr(9e307, 9e307), arr(9e307, 9e307)])
    assert PERCENTOF.evaluate([arr(-1), arr(2)]) == mp(-0.5)


def test_draws_stay_inside_the_argument_limit() -> None:
    for definition in (PRODUCT, SUMSQ, SERIESSUM, PERCENTOF):
        sampler = Sampler(definition, random.Random(11))
        for stratum in STRATA:
            for index in range(20):
                args = sampler.draw(stratum, index)
                assert 1 <= len(args) <= 255  # Excel's argument limit
                # every array literal holds at most 255 values
                assert all(len(a) <= 255 for a in args if isinstance(a, tuple))


def test_subnormal_results_become_zero() -> None:
    # numbers.md "Subnormal results become 0": probes/subnormal-function-results, -flush-edges, -sumsq-sumproduct-edges
    for args in ((1e-308, 1), (3e-308, 0.5), (1e-200, 1e-110), (1e-308, 1e300), (1e-200, 1e-110, 1e300)):
        assert PRODUCT.evaluate([mp(a) for a in args]) == 0
    assert SUMSQ.evaluate([mp(1e-160)]) == 0
    for args in ((1e-155, 1e-150), (1e-150, 1e-155)):
        assert SUMSQ.evaluate([mp(a) for a in args]) == mp(1e-150) ** 2  # the 1E-310 square is dropped
    assert PERCENTOF.evaluate([arr(1e-300), arr(1e300)]) == 0  # probes/percentof-snap-overflow-union D14
    assert SERIESSUM.evaluate([mp(1e-200), mp(2), mp(1), arr(1)]) == 0  # the power 1E-400 is flushed
