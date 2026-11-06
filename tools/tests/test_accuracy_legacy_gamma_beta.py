"""Gate A definitions of the legacy CHIDIST, CHIINV, GAMMADIST, GAMMAINV, BETADIST and BETAINV (T-4104): each is its
modern function under the legacy name, checked against Microsoft's published examples and the paired modern value."""

from __future__ import annotations

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import legacy_gamma_beta
from rustsheet_tools.accuracy.model import REGISTRY

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(function: str, *args: float | bool) -> mpmath.mpf:
    return REGISTRY[function].evaluate([a if isinstance(a, bool) else mp(a) for a in args])


def near(got: mpmath.mpf, want: float, digits: int) -> bool:
    return bool(abs(got - mp(want)) <= mp(10) ** -digits * max(1, abs(mp(want))))


def test_published_examples() -> None:
    # Microsoft's pages: CHIDIST(18.307,10) 0.050001, CHIINV(0.050001,10) 18.30697, GAMMADIST(10.00001131,9,2,FALSE)
    # 0.032639, GAMMADIST(..,TRUE) 0.068094, GAMMAINV(0.068094,9,2) 10.0000112, BETADIST(2,8,10,1,3) 0.6854706,
    # BETAINV(0.685470581,8,10,1,3) 2 (each to the digits the page shows).
    assert near(value("CHIDIST", 18.307, 10), 0.050001, 6)
    assert near(value("CHIINV", 0.050001, 10), 18.30697, 6)
    assert near(value("GAMMADIST", 10.00001131, 9, 2, False), 0.032639, 6)
    assert near(value("GAMMADIST", 10.00001131, 9, 2, True), 0.068094, 6)
    assert near(value("GAMMAINV", 0.068094, 9, 2), 10.0000112, 6)
    assert near(value("BETADIST", 2, 8, 10, 1, 3), 0.6854706, 7)
    assert near(value("BETAINV", 0.685470581, 8, 10, 1, 3), 2, 8)


@pytest.mark.parametrize(
    ("legacy", "modern", "args", "extra"),
    [
        ("CHIDIST", "CHISQ.DIST.RT", (3.5, 4), ()),
        ("CHIINV", "CHISQ.INV.RT", (0.2, 7), ()),
        ("GAMMADIST", "GAMMA.DIST", (2.5, 3, 1.5, True), ()),
        ("GAMMAINV", "GAMMA.INV", (0.3, 2, 4), ()),
        ("BETAINV", "BETA.INV", (0.4, 2, 5), ()),
        ("BETADIST", "BETA.DIST", (0.3, 2, 5), (True,)),
    ],
)
def test_equals_the_modern_function(
    legacy: str, modern: str, args: tuple[float | bool, ...], extra: tuple[bool, ...]
) -> None:
    # statistics.md "Legacy distribution names sweep (T-1047)" and "(T-1048)": the same value on every paired cell.
    assert value(legacy, *args) == value(modern, *args, *extra)


def test_betadist_takes_bounds_after_the_shapes() -> None:
    # BETADIST(x,a,b,A,B) is BETA.DIST(x,a,b,TRUE,A,B).
    assert value("BETADIST", 2, 8, 10, 1, 3) == value("BETA.DIST", 2, 8, 10, True, 1, 3)
    assert [p.name for p in legacy_gamma_beta.BETADIST.params] == ["x", "alpha", "beta"]
