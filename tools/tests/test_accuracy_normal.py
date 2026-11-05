"""Gate A definitions of NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI and GAUSS (T-3783) against published
values, the semantics entries' Excel values and the excluded cancellation regions."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.normal import (
    GAUSS,
    NORM_DIST,
    NORM_INV,
    NORM_S_DIST,
    NORM_S_INV,
    PHI,
    _draw_gauss,
    _draw_norm_inv,
    gauss_cancels,
    inv_cancels,
)
from rustsheet_tools.accuracy.model import DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def test_published_values() -> None:
    # Abramowitz and Stegun table 26.1: Phi(1) = 0.8413447460685429, phi(1) = 0.2419707245191434
    assert mpmath.almosteq(
        NORM_S_DIST.evaluate([mp(1), True]), mp("0.841344746068542948585232545632"), mp("1e-30")
    )
    assert mpmath.almosteq(PHI.evaluate([mp(1)]), mp("0.241970724519143349797830192936"), mp("1e-30"))
    assert NORM_S_DIST.evaluate([mp(0), True]) == mp(1) / 2
    # Microsoft's NORM.DIST page: NORM.DIST(42,40,1.5,TRUE) = 0.9087888, FALSE = 0.10934
    assert float(NORM_DIST.evaluate([mp(42), mp(40), mp(1.5), True])) == pytest.approx(0.9087888, abs=1e-7)
    assert float(NORM_DIST.evaluate([mp(42), mp(40), mp(1.5), False])) == pytest.approx(0.10934, abs=1e-5)
    # Microsoft's NORM.INV page: NORM.INV(0.908789,40,1.5) = 42.000002
    assert float(NORM_INV.evaluate([mp(0.908789), mp(40), mp(1.5)])) == pytest.approx(42.000002, abs=1e-6)
    # Microsoft's NORM.S.INV page: NORM.S.INV(0.908789) = 1.3333347; GAUSS(2) = 0.47724987
    assert float(NORM_S_INV.evaluate([mp(0.908789)])) == pytest.approx(1.3333347, abs=1e-7)
    assert float(GAUSS.evaluate([mp(2)])) == pytest.approx(0.47724987, abs=1e-8)


def test_inverse_round_trips_the_cdf() -> None:
    for p in (mp("1e-300"), mp("0.075"), mp("0.5"), mp("0.925"), mp("0.999999")):
        z = NORM_S_INV.evaluate([p])
        assert mpmath.almosteq(NORM_S_DIST.evaluate([z, True]), p, rel_eps=mp("1e-30"))


def test_excel_values_of_the_semantics_entries_round_from_the_math() -> None:
    # statistics.md T-2255: GAUSS(1E-300) is 3.989422804014327E-301 (the leading term, one ulp above the math)
    assert float(GAUSS.evaluate([mp(1e-300)])) == pytest.approx(3.989422804014327e-301, rel=2e-16)
    # statistics.md T-1041: z = -37.5 gives 1.7282337322841054E-306
    assert float(PHI.evaluate([mp(-37.5)])) == pytest.approx(1.7282337322841054e-306, rel=1e-14)


def test_density_overflow_is_the_error_domain() -> None:
    # statistics.md T-2435: NORM.S.DIST(2^512,FALSE), PHI(1E+200) and NORM.DIST(3,1,2^-512,FALSE) are #NUM!,
    # NORM.S.DIST(1.3E+154,FALSE) and the cdf at any magnitude are numbers
    with pytest.raises(DomainError):
        NORM_S_DIST.evaluate([mp(2.0**512), False])
    with pytest.raises(DomainError):
        PHI.evaluate([mp(1e200)])
    with pytest.raises(DomainError):
        NORM_DIST.evaluate([mp(3), mp(1), mp(2.0**-512), False])
    assert NORM_S_DIST.evaluate([mp(1.3e154), False]) >= 0
    assert NORM_S_DIST.evaluate([mp(-1e300), True]) >= 0
    with pytest.raises(DomainError):
        NORM_DIST.evaluate([mp(1), mp(0), mp(0), True])  # standard_dev 0 is #NUM!


def test_cumulative_past_the_double_range_is_redrawn() -> None:
    with pytest.raises(NoValueError):
        NORM_DIST.evaluate([mp(1e300), mp(-1e300), mp(1e-300), True])


def test_draws_avoid_the_cancellation_regions() -> None:
    assert gauss_cancels(4e-15)
    assert not gauss_cancels(1e-15)
    assert not gauss_cancels(1e-8)
    assert inv_cancels(0.3, 0.524400512708041, 1.0)  # NORM.S.INV(0.3) = -0.5244005127080407
    assert not inv_cancels(0.3, 1.0, 2.0)
    rng = random.Random(7)
    for stratum in STRATA:
        for _ in range(40):
            (z,) = _draw_gauss(stratum, rng)
            assert isinstance(z, float)
            assert not gauss_cancels(z)
            p, mean, sd = _draw_norm_inv(stratum, rng)
            assert isinstance(p, float) and isinstance(mean, float) and isinstance(sd, float)
            assert 0 < p < 1 and sd > 0
            assert not inv_cancels(p, mean, sd)
