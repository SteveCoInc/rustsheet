"""Gate A definitions of LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER and FISHERINV (T-3785) against
Microsoft's published examples, the semantics entries' values, the evidenced error domain and the excluded regions."""

from __future__ import annotations

import math
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.continuous import (
    EXPON_DIST,
    FISHER,
    FISHERINV,
    LOGNORM_DIST,
    LOGNORM_INV,
    WEIBULL_DIST,
    fisher_cancels,
)
from rustsheet_tools.accuracy.defs.normal import NORM_DIST
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float | bool) -> float:
    return float(definition.evaluate([a if isinstance(a, bool) else mp(a) for a in args]))


def test_microsoft_examples() -> None:
    # Microsoft's LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER and FISHERINV pages
    assert value(LOGNORM_DIST, 4, 3.5, 1.2, True) == pytest.approx(0.0390836, abs=1e-7)
    assert value(LOGNORM_DIST, 4, 3.5, 1.2, False) == pytest.approx(0.0176176, abs=1e-7)
    assert value(LOGNORM_INV, 0.039084, 3.5, 1.2) == pytest.approx(4.0000252, abs=1e-7)
    assert value(EXPON_DIST, 0.2, 10, True) == pytest.approx(0.86466472, abs=1e-8)
    assert value(EXPON_DIST, 0.2, 10, False) == pytest.approx(1.35335283, abs=1e-8)
    assert value(WEIBULL_DIST, 105, 20, 100, True) == pytest.approx(0.929581, abs=1e-6)
    assert value(WEIBULL_DIST, 105, 20, 100, False) == pytest.approx(0.035589, abs=1e-6)
    assert value(FISHER, 0.75) == pytest.approx(0.9729551, abs=1e-7)
    assert value(FISHERINV, 0.972955) == pytest.approx(0.75, abs=1e-6)


def test_lognorm_dist_is_the_normal_of_ln_x() -> None:
    # statistics.md T-1778: LOGNORM.DIST(x,m,s,TRUE) = NORM.DIST(LN(x),m,s,TRUE)
    x, m, s = mp(2.5), mp(0.3), mp(1.7)
    assert mpmath.almosteq(
        LOGNORM_DIST.evaluate([x, m, s, True]), NORM_DIST.evaluate([mpmath.log(x), m, s, True]), mp("1e-35")
    )
    # T-1040's deep lower tail: the true values beside Excel's 2E-13-off ones
    assert value(LOGNORM_DIST, 2, 1, 0.01, True) == pytest.approx(4.473044928316143e-207, rel=1e-15)
    assert value(LOGNORM_DIST, 0.1, 0, 0.1, True) == pytest.approx(1.2841756306435298e-117, rel=1e-15)


def test_lognorm_inv_inverts_the_cdf() -> None:
    # T-0925: LOGNORM.INV(1E-20,0,1) is Excel's 9.493291347224372E-05, near the math
    assert value(LOGNORM_INV, 1e-20, 0, 1) == pytest.approx(9.493291347224372e-05, rel=1e-13)
    for p in (1e-300, 0.075, 0.5, 0.999):
        x = LOGNORM_INV.evaluate([mp(p), mp(1.5), mp(0.8)])
        assert mpmath.almosteq(LOGNORM_DIST.evaluate([x, mp(1.5), mp(0.8), True]), mp(p), rel_eps=mp("1e-30"))


def test_lognorm_domain_and_left_out_points() -> None:
    for args in ((0, 0, 1, True), (-1, 0, 1, False), (1, 0, 0, True)):
        with pytest.raises(DomainError):
            LOGNORM_DIST.evaluate([a if isinstance(a, bool) else mp(a) for a in args])
    for inv_args in ((0, 0, 1), (1, 0, 1), (0.5, 0, 0)):
        with pytest.raises(DomainError):
            LOGNORM_INV.evaluate([mp(a) for a in inv_args])
    with pytest.raises(NoValueError):  # the overflow of exp(mean + sd*z) is unprobed
        LOGNORM_INV.evaluate([mp(0.5), mp(710), mp(1)])
    with pytest.raises(NoValueError):  # ln x - mean cancels against a tiny sd
        LOGNORM_DIST.evaluate([mp(math.e), mp(1), mp(1e-12), True])
    with pytest.raises(NoValueError):  # exp(-z^2/2) underflows before the division by x*sd
        LOGNORM_DIST.evaluate([mp(3.63830575608042e-201), mp(12.7611551233304), mp(10.2414046542683), False])


def test_expon_dist() -> None:
    assert value(EXPON_DIST, 0, 2, True) == 0
    assert value(EXPON_DIST, 0, 2, False) == 2
    assert value(EXPON_DIST, 1e-300, 1, True) == pytest.approx(1e-300, rel=1e-15)  # expm1 keeps the digits
    for args in ((-1e-300, 1, True), (1, 0, True), (1, -1, False)):
        with pytest.raises(DomainError):
            EXPON_DIST.evaluate([a if isinstance(a, bool) else mp(a) for a in args])
    with pytest.raises(NoValueError):
        EXPON_DIST.evaluate([mp(1e200), mp(1e200), True])


def test_weibull_semantics_values() -> None:
    # statistics.md T-1046
    assert value(WEIBULL_DIST, 1e-10, 1, 1, True) == pytest.approx(9.999999999499998e-11, rel=1e-15)
    assert value(WEIBULL_DIST, 1e-10, 0.5, 1, True) == pytest.approx(9.999950000166663e-06, rel=1e-15)
    assert value(WEIBULL_DIST, 1e-10, 0.5, 1, False) == pytest.approx(49999.5000025, rel=1e-12)
    assert value(WEIBULL_DIST, 1000, 0.5, 1, True) == pytest.approx(0.9999999999999816, rel=1e-15)
    # x = 0 is 0 for every shape, the density too (t1048-weibull A16, A17; T-3312's (0,308,10,FALSE))
    assert value(WEIBULL_DIST, 0, 0.5, 1, False) == 0
    assert value(WEIBULL_DIST, 0, 308, 10, False) == 0


@pytest.mark.parametrize(
    "args",
    [
        (0.1, 10000000000.5, 0.2, False),  # beta^alpha underflows: #DIV/0!
        (0.3, 99999, 99999, False),  # beta^alpha overflows: #NUM!
        (100, 1e10, 0.01, True),  # t overflows: #NUM!
        (0, 309, 10, False),  # T-3312: #NUM!
        (0, 400, 0.1, False),  # T-3312: #DIV/0!
        (-1e-300, 1, 1, True),
        (1, 0, 1, True),
        (1, 1, 0, False),
    ],
)
def test_weibull_error_domain(args: tuple[float, float, float, bool]) -> None:
    with pytest.raises(DomainError):
        WEIBULL_DIST.evaluate([mp(args[0]), mp(args[1]), mp(args[2]), args[3]])


def test_weibull_cdf_takes_x_over_beta() -> None:
    # T-2239: WEIBULL(1E+15,42,10000000001,TRUE) is a number though 10000000001^42 overflows
    assert 0 < value(WEIBULL_DIST, 1e15, 42, 10000000001, True) <= 1


def test_fisher() -> None:
    assert value(FISHER, 0) == 0
    assert value(FISHERINV, 0) == 0
    assert value(FISHERINV, 1000) == 1
    assert value(FISHERINV, -1e300) == -1
    for x in (1, -1, 1.00000000000001):
        with pytest.raises(DomainError):
            FISHER.evaluate([mp(x)])
    assert fisher_cancels(1e-7) and fisher_cancels(-1e-16)
    assert not fisher_cancels(0) and not fisher_cancels(1e-6)


@pytest.mark.parametrize("definition", [FISHER, FISHERINV])
def test_fisher_draws_avoid_the_cancellation_region(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(3785))
    for stratum in STRATA:
        for i in range(200):
            (x,) = sampler.draw(stratum, i)
            assert isinstance(x, float)
            assert not fisher_cancels(x)
