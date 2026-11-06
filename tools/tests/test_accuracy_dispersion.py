"""Gate A definitions of the dispersion I functions (T-4089) against Microsoft's examples and statistics.md."""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.dispersion import DEVSQ, STDEV, STDEV_S, STDEVA, VAR, VAR_S, VARA, _draw
from rustsheet_tools.accuracy.model import Definition, DomainError

mpmath.mp.dps = 50
# Microsoft's STDEV/VAR example: tool breaking strength of ten tools.
STRENGTH = [1345, 1301, 1368, 1322, 1310, 1370, 1318, 1350, 1303, 1299]


def mp(xs: Sequence[float]) -> list[mpmath.mpf]:
    return [mpmath.mpf(x) for x in xs]


@pytest.mark.parametrize("definition", [VAR, VAR_S, VARA])
def test_variance_is_the_exact_sample_variance(definition: Definition) -> None:
    # Microsoft: VAR of the strength column is 754.2666667 (6788.4 / 9).
    assert definition.evaluate(mp(STRENGTH)) == mpmath.mpf(67884) / 90
    # statistics.md: VAR.S({1,2,4}) is 2.333333333333334 in Excel (one-pass); the math is 7/3.
    assert definition.evaluate(mp([1, 2, 4])) == mpmath.mpf(7) / 3


@pytest.mark.parametrize("definition", [STDEV, STDEV_S, STDEVA])
def test_stdev_is_the_root_of_the_variance(definition: Definition) -> None:
    # Microsoft: STDEV of the strength column is 27.46391572.
    value = definition.evaluate(mp(STRENGTH))
    assert abs(value - mpmath.mpf("27.4639157198435")) < mpmath.mpf("1e-12")


def test_devsq() -> None:
    # Microsoft: DEVSQ(4,5,8,7,11,4,3) is 48.
    assert DEVSQ.evaluate(mp([4, 5, 8, 7, 11, 4, 3])) == 48
    # statistics.md: DEVSQ(1E15,1E15+1) is 0.5, and one number has no deviation.
    assert DEVSQ.evaluate(mp([1e15, 1e15 + 1])) == mpmath.mpf("0.5")
    assert DEVSQ.evaluate(mp([3.5])) == 0


def test_domain() -> None:
    with pytest.raises(DomainError):
        VAR_S.evaluate(mp([2.0]))
    assert VAR_S.domain is not None and DEVSQ.domain is not None
    assert VAR_S.domain(mp([2.0])) is not None
    # statistics.md: VAR.P of 1E+200 twice is #NUM! (the one-pass sum of squares overflows).
    assert VAR_S.domain(mp([1e200, 1e200])) is not None
    # DEVSQ's two-pass form does not overflow where VAR's one-pass sum of squares does
    assert DEVSQ.domain(mp([1e200, 1e200])) is None
    assert DEVSQ.domain(mp([1e200, -1e200])) is not None  # (x - m)^2 = 1E+400
    assert VAR_S.domain(mp([1e150] * 255)) is None


@pytest.mark.parametrize("stratum", ["typical", "tail", "edge", "extreme", "log-uniform"])
def test_draws_stay_inside_the_domain(stratum: str) -> None:
    rng = random.Random(4089)
    for _ in range(200):
        args = _draw(stratum, rng)
        assert 1 <= len(args) <= 255
        assert VAR_S.domain is not None and DEVSQ.domain is not None
        numbers = mp([float(a) for a in args])  # type: ignore[arg-type]
        assert DEVSQ.domain(numbers) is None
        assert (VAR_S.domain(numbers) is None) == (len(args) >= 2)
