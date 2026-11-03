"""Gate A definitions of SUMPRODUCT and the SUMX2MY2 family (T-3677) against
probes/t3677-sumx-running-overflow-snap and the values numbers.md cites."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.sumx import SUMPRODUCT, SUMX2MY2, SUMX2PY2, SUMXMY2
from rustsheet_tools.accuracy.model import DomainError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def arr(*xs: float) -> tuple[mpmath.mpf, ...]:
    return tuple(mp(x) for x in xs)


def test_sumproduct_overflows_at_a_running_sum() -> None:
    # t3677 A1 #NUM!, A2 9E+307, A3 #NUM! (one array)
    with pytest.raises(DomainError):
        SUMPRODUCT.evaluate([arr(9e307, 9e307, -9e307), arr(1, 1, 1)])
    assert SUMPRODUCT.evaluate([arr(9e307, -9e307, 9e307), arr(1, 1, 1)]) == mp(9e307)
    with pytest.raises(DomainError):
        SUMPRODUCT.evaluate([arr(9e307, 9e307, -9e307)])
    with pytest.raises(DomainError):  # a product past the double range
        SUMPRODUCT.evaluate([arr(1e200), arr(1e200)])


def test_sumproduct_is_exact_and_never_snaps() -> None:
    # numbers.md "Summing functions" (T-3048): no snap; the math is the exact sum of the doubles' products
    assert SUMPRODUCT.evaluate([arr(0.1, 0.2, -0.3), arr(1, 1, 1)]) == mp(0.1) + mp(0.2) - mp(0.3)
    assert SUMPRODUCT.evaluate([arr(1, 2, 3), arr(4, 5, 6), arr(1, 1, 2)]) == 4 + 10 + 36
    with pytest.raises(DomainError):  # #VALUE!: the lengths differ
        SUMPRODUCT.evaluate([arr(1, 2, 3), arr(1, 2)])


def test_sumproduct_flushes_each_running_product() -> None:
    # numbers.md "Subnormal flush edges" (T-1007): {1E-200,1}·{1E-110,1E-300} is exactly 1E-300; three arrays flush
    # after each factor, left to right
    assert SUMPRODUCT.evaluate([arr(1e-200, 1), arr(1e-110, 1e-300)]) == mp(1e-300)
    assert SUMPRODUCT.evaluate([arr(1e-200), arr(1e-110), arr(1e300)]) == 0
    assert SUMPRODUCT.evaluate([arr(1e-200), arr(1e300), arr(1e-110)]) == mp(1e-200) * mp(1e300) * mp(1e-110)


def test_sumx2my2_adds_one_term_per_pair() -> None:
    # t3677 A4 #NUM! (the running sum overflows), A5 1.44E+308, A6 0; T-3808: a square past the range is #NUM!
    big = 1.2e154
    with pytest.raises(DomainError):
        SUMX2MY2.evaluate([arr(big, big, 0), arr(0, 0, big)])
    assert SUMX2MY2.evaluate([arr(big, 0, big), arr(0, big, 0)]) == mp(big) ** 2
    assert SUMX2MY2.evaluate([arr(big, big), arr(big, big)]) == 0
    with pytest.raises(DomainError):
        SUMX2MY2.evaluate([arr(1.5e154), arr(1.4e154)])
    assert float(SUMX2MY2.evaluate([arr(9e153), arr(0)])) == 8.1e307  # numbers.md: a finite near-limit result


def test_sumx2my2_never_snaps() -> None:
    # t3677 A8 and A10: 1 - (1+2^-52)^2 is -4.44E-16, 2 ULP of 1, not 0
    assert float(SUMX2MY2.evaluate([arr(1, 0), arr(0, 1.0000000000000002)])) == -4.440892098500626e-16
    assert float(SUMX2MY2.evaluate([arr(0, 1.0000000000000002), arr(1, 0)])) == 4.440892098500626e-16


def test_sumx2py2_and_sumxmy2_overflow_on_a_term() -> None:
    # t3677 A20 and A19; numbers.md: SUMXMY2({1.5E+154},{1.4E+154}) is 1.0000000000000033E+306 and of ({1E+155},{0}) #NUM!
    with pytest.raises(DomainError):
        SUMX2PY2.evaluate([arr(1.2e154), arr(1.2e154)])
    with pytest.raises(DomainError):
        SUMXMY2.evaluate([arr(1e154, -1e154), arr(-1e154, 1e154)])
    assert float(SUMXMY2.evaluate([arr(1.5e154), arr(1.4e154)])) == 1.0000000000000033e306
    with pytest.raises(DomainError):
        SUMXMY2.evaluate([arr(1e155), arr(0)])
    with pytest.raises(DomainError):  # t3677 A7: the running sum overflows
        SUMX2PY2.evaluate([arr(1.2e154, 0, 1.2e154), arr(0, 1.2e154, 0)])


def test_sumx_values_and_lengths() -> None:
    # coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing": SUMX2PY2({1,"a"},{3,4}) is 10 (the numeric pair 1, 3);
    # unequal lengths are #N/A
    assert SUMX2PY2.evaluate([arr(1), arr(3)]) == 10
    assert SUMXMY2.evaluate([arr(2), arr(4)]) == 4
    assert SUMX2MY2.evaluate([arr(1, 0), arr(0, 1)]) == 0  # t3677 A18
    for definition in (SUMX2MY2, SUMX2PY2, SUMXMY2):
        with pytest.raises(DomainError):
            definition.evaluate([arr(1, 2, 3), arr(1, 2)])


def test_sumx_flushes_a_subnormal_square() -> None:
    # t3677 A12:A15: the 1E-310 square is dropped, as SUMSQ's is (T-1057)
    tiny = arr(1e-155, 1e-150)
    assert SUMX2PY2.evaluate([tiny, arr(0, 0)]) == mp(1e-150) ** 2
    assert SUMX2MY2.evaluate([tiny, arr(0, 0)]) == mp(1e-150) ** 2
    assert SUMXMY2.evaluate([tiny, arr(0, 0)]) == mp(1e-150) ** 2
    assert SUMX2MY2.evaluate([arr(0, 0), tiny]) == -(mp(1e-150) ** 2)


def test_draws_fit_excel_limits() -> None:
    for definition in (SUMPRODUCT, SUMX2MY2, SUMX2PY2, SUMXMY2):
        sampler = Sampler(definition, random.Random(7))
        for stratum in STRATA:
            for index in range(20):
                args = sampler.draw(stratum, index)
                assert 1 <= len(args) <= 255  # Excel's argument limit
                assert all(isinstance(a, tuple) and 1 <= len(a) <= 255 for a in args)
                assert sum(len(a) for a in args if isinstance(a, tuple)) <= 300  # the 8192-character formula
