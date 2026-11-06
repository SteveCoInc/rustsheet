"""Gate A definitions of CONFIDENCE.NORM and CONFIDENCE (T-4101) against Microsoft's published example, the semantics
entries' values, the evidenced error domain and the excluded cancellation region."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs._common import NORMAL_MIN, confidence_margin
from rustsheet_tools.accuracy.defs.confidence import (
    CONFIDENCE,
    CONFIDENCE_NORM,
    confidence_cancels,
    p_rounds_to_one,
)
from rustsheet_tools.accuracy.defs.normal import NORM_S_INV
from rustsheet_tools.accuracy.eval_math import fit_int
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

mp = mpmath.mpf
BOTH = [CONFIDENCE, CONFIDENCE_NORM]


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float) -> float:
    return float(definition.evaluate([mp(a) for a in args]))


@pytest.mark.parametrize("definition", BOTH)
def test_microsoft_example(definition: Definition) -> None:
    # Microsoft's CONFIDENCE.NORM and CONFIDENCE pages: alpha 0.05, sd 2.5, size 50
    assert value(definition, 0.05, 2.5, 50) == pytest.approx(0.692952, abs=1e-6)


@pytest.mark.parametrize("definition", BOTH)
def test_semantics_values(definition: Definition) -> None:
    # probes-numeric/m9-legacy-confidence (statistics.md T-1047): Excel's values, within a few ulp of the math here
    assert value(definition, 0.05, 1, 100) == pytest.approx(0.19599639845400538, rel=1e-15)
    assert value(definition, 0.05, 2.5, 30) == pytest.approx(0.8945970718585784, rel=1e-15)
    assert value(definition, 0.01, 10, 1000) == pytest.approx(0.8145487463019762, rel=1e-15)
    assert value(definition, 0.5, 1, 2) == pytest.approx(0.47693627620447, rel=1e-14)
    assert value(definition, 0.999, 3, 7) == pytest.approx(0.0014211250243260142, rel=1e-12)


def test_the_margin_is_the_normal_quantile() -> None:
    alpha, sd, n = mp("0.2"), mp(2), 4
    z = NORM_S_INV.evaluate([1 - alpha / 2])
    assert mpmath.almosteq(CONFIDENCE_NORM.evaluate([alpha, sd, mp(n)]), z * sd / mpmath.sqrt(n), mp("1e-35"))


@pytest.mark.parametrize("definition", BOTH)
@pytest.mark.parametrize(
    "args",
    [
        (0.0, 1, 100),
        (1.0, 1, 100),
        (-0.1, 1, 100),
        (1.5, 0.5, 6),
        (0.05, 0, 100),
        (0.05, -1, 100),
        (0.05, 1, 0),
        (0.05, 1, -5),
        # 1 - alpha/2 rounds to 1 in doubles (statistics.md T-4101)
        (1e-16, 0.5, 6),
        (2.0**-53, 1, 5),
        (1e-100, 2, 5),
        (1e-300, 99999, 9),
    ],
)
def test_error_domain(definition: Definition, args: tuple[float, float, float]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


def test_p_rounds_to_one_at_two_to_the_minus_53() -> None:
    assert p_rounds_to_one(1e-16)
    assert p_rounds_to_one(2.0**-53)
    assert not p_rounds_to_one(1.11022302462516e-16)  # just above 2^-53: a number in Excel too
    assert not p_rounds_to_one(1e-15)  # `CONFIDENCE(1E-15,0.01,9)` is a number
    assert value(CONFIDENCE, 1e-15, 0.01, 9) == pytest.approx(0.026756196275115137, rel=1e-15)


def test_size_truncates_for_confidence_only() -> None:
    # `CONFIDENCE(0.05,1,100.9)` is size 100 and `CONFIDENCE(0.05,1,0.9)` #NUM! (T-1047); unprobed for CONFIDENCE.NORM
    assert fit_int(CONFIDENCE.params[2], 100.9) == 100
    assert fit_int(CONFIDENCE.params[2], 0.9) == 0
    assert isinstance(fit_int(CONFIDENCE_NORM.params[2], 100.9), str)
    with pytest.raises(DomainError):
        CONFIDENCE.evaluate([mp(0.05), mp(1), 0])


def test_out_of_range_intermediates_have_no_value() -> None:
    with pytest.raises(NoValueError):
        CONFIDENCE.evaluate([mp(0.001), mp(1e300), mp(4)])
    with pytest.raises(NoValueError):
        CONFIDENCE.evaluate([mp(0.05), mp(1e-306), mp(1e15)])


def test_cancellation_region() -> None:
    # Excel's double 1 - alpha/2: 1.6E-3 off at 1E-15, 2.2E-9 at 1E-9, 1.1E-10 at 1 - 1E-6, all of it at 1 - 1E-16
    for alpha in (1e-15, 1e-9, 1e-8, 0.999999, 1 - 1e-10, 1 - 1e-16):
        assert confidence_cancels(alpha)
    for alpha in (5e-8, 1e-3, 0.05, 0.5, 0.99, 1 - 2e-6, 0.0, 1.0, 1e-16):
        assert not confidence_cancels(alpha)


@pytest.mark.parametrize("definition", BOTH)
def test_draws_avoid_the_cancellation_region(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(4101))
    for stratum in STRATA:
        for i in range(100):
            alpha, sd, size = sampler.draw(stratum, i)
            assert isinstance(alpha, float)
            assert isinstance(size, int)
            assert isinstance(sd, float)
            assert not confidence_cancels(alpha)


def test_the_shared_margin_and_its_range_check() -> None:
    # `_common.confidence_margin`, shared with CONFIDENCE.T (T-4581): quantile 2, sd 3, n 4 is 2*3/2 exactly
    assert confidence_margin(mp(2), mp(3), mp(4)) == 3
    with pytest.raises(NoValueError):  # sd*quantile = 2e300 overflows MARGIN_RANGE
        confidence_margin(mp(2), mp("1e300"), mp(1))
    with pytest.raises(NoValueError):  # sd below the smallest normal double
        confidence_margin(mp(2), mp(NORMAL_MIN) / 2, mp(1))
