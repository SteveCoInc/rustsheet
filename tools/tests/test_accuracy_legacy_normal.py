"""Gate A definitions of NORMDIST, NORMSDIST, NORMINV, NORMSINV, LOGNORMDIST, LOGINV and STANDARDIZE (T-4100): the
legacy names share their modern Definitions (statistics.md T-1048) and STANDARDIZE is (x - mean)/sd, checked against
Microsoft's examples and the semantics entries' Excel cells."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.continuous import LOGINV, LOGNORM_DIST, LOGNORM_INV, LOGNORMDIST
from rustsheet_tools.accuracy.defs.normal import (
    NORM_DIST,
    NORM_INV,
    NORM_S_DIST,
    NORM_S_INV,
    NORMDIST,
    NORMINV,
    NORMSDIST,
    NORMSINV,
    STANDARDIZE,
)
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float | bool) -> float:
    return float(definition.evaluate([a if isinstance(a, bool) else mp(a) for a in args]))


def test_legacy_names_equal_their_modern_definitions() -> None:
    # statistics.md T-1048: every paired cell is identical, value and error
    for z in (mp(-37), mp("0.5"), mp(8)):
        assert NORMSDIST.evaluate([z]) == NORM_S_DIST.evaluate([z, True])
    for args in ([mp(42), mp(40), mp("1.5")], [mp("3.7"), mp(3), mp("0.1")]):
        for cumulative in (True, False):
            assert NORMDIST.evaluate([*args, cumulative]) == NORM_DIST.evaluate([*args, cumulative])
    for p in (mp("1e-300"), mp("0.3"), mp("0.9999999")):
        assert NORMSINV.evaluate([p]) == NORM_S_INV.evaluate([p])
        assert NORMINV.evaluate([p, mp(10), mp(2)]) == NORM_INV.evaluate([p, mp(10), mp(2)])
        assert LOGINV.evaluate([p, mp(1), mp("0.5")]) == LOGNORM_INV.evaluate([p, mp(1), mp("0.5")])
    for x in (mp("0.01"), mp(2), mp(50)):
        assert LOGNORMDIST.evaluate([x, mp(1), mp("1.5")]) == LOGNORM_DIST.evaluate(
            [x, mp(1), mp("1.5"), True]
        )


def test_excel_cells_of_the_legacy_sweep() -> None:
    # probes-numeric/t1048-* column A (excel-graph), within the few ulp of T-1040 and T-1746
    assert value(NORMDIST, 42, 40, 1.5, True) == pytest.approx(0.9087887802741321, rel=1e-14)
    assert value(NORMDIST, 42, 40, 1.5, False) == pytest.approx(0.10934004978399575, rel=1e-14)
    assert value(NORMSDIST, -37) == pytest.approx(5.7255712225239266e-300, rel=1e-12)
    assert value(NORMSDIST, -6) == pytest.approx(9.865876450376944e-10, rel=1e-14)
    assert value(NORMINV, 0.001, 10, 2) == pytest.approx(3.8195353876643736, rel=1e-14)
    assert value(NORMSINV, 1e-300) == pytest.approx(-37.0470962993612, rel=1e-14)
    assert value(NORMSINV, 0.3) == pytest.approx(-0.5244005127080409, rel=1e-14)
    assert value(LOGNORMDIST, 0.01, -1, 1) == pytest.approx(0.00015597421740995494, rel=1e-14)
    assert value(LOGNORMDIST, 1, 0, 1) == 0.5
    assert value(LOGINV, 0.9, 2, 1) == pytest.approx(26.617038758334957, rel=1e-14)
    # Microsoft's LOGNORMDIST, LOGINV and NORMSDIST pages
    assert value(LOGNORMDIST, 4, 3.5, 1.2) == pytest.approx(0.0390836, abs=1e-7)
    assert value(LOGINV, 0.039084, 3.5, 1.2) == pytest.approx(4.0000252, abs=1e-7)
    assert value(NORMSDIST, 1.333333) == pytest.approx(0.908789, abs=1e-6)


def test_legacy_error_domains_are_the_modern_ones() -> None:
    # statistics.md T-1048 (t1048-* A9:A17): sd 0 or below, x <= 0, p outside (0, 1) are #NUM!
    for definition, args in (
        (NORMDIST, (0, 0, 0, True)),
        (NORMDIST, (1, 0, -1, True)),
        (NORMINV, (0, 0, 1)),
        (NORMINV, (1, 0, 1)),
        (NORMINV, (0.5, 0, 0)),
        (NORMSINV, (0,)),
        (NORMSINV, (1,)),
        (LOGNORMDIST, (0, 0, 1)),
        (LOGNORMDIST, (-1, 0, 1)),
        (LOGNORMDIST, (1, 0, 0)),
        (LOGINV, (0, 0, 1)),
        (LOGINV, (1, 0, 1)),
        (LOGINV, (0.5, 0, -1)),
    ):
        with pytest.raises(DomainError):
            value(definition, *args)
    # T-2435: NORMDIST's density shares NORM.DIST's overflow domain; the cdf never errs
    with pytest.raises(DomainError):
        value(NORMDIST, 1e155, 0, 1, False)
    assert value(NORMSDIST, 1e300) == 1.0
    assert value(NORMSDIST, -1e300) == 0.0


def test_standardize_is_the_quotient() -> None:
    # Microsoft's STANDARDIZE page: STANDARDIZE(42,40,1.5) = 1.33333333
    assert value(STANDARDIZE, 42, 40, 1.5) == pytest.approx(1.3333333333333333, rel=1e-15)
    # statistics.md T-1623 and T-2255: the quotient of the doubles
    assert value(STANDARDIZE, 5, 3, 1e-300) == pytest.approx(1.9999999999999998e300, rel=1e-15)
    assert value(STANDARDIZE, -1e-15, 1e-15, 1) == pytest.approx(-2e-15, rel=1e-15)
    assert value(STANDARDIZE, 3, 3, 2) == 0
    # T-2255 and T-2436: standard_dev -1 and 0 are #NUM!
    for sd in (-1, 0):
        with pytest.raises(DomainError):
            value(STANDARDIZE, 3, -1, sd)
    # past the double range, or a subnormal quotient: unprobed, redrawn
    for args in ((1.7e308, -1.7e308, 1e10), (1e300, 0, 1e-10), (1e-300, 0, 1e100)):
        with pytest.raises(NoValueError):
            value(STANDARDIZE, *args)


def test_lognormdist_draws_drop_the_flag() -> None:
    definition = LOGNORMDIST
    assert definition.draw is not None
    rng = random.Random(3)
    for stratum in STRATA:
        for _ in range(20):
            assert len(definition.draw(stratum, rng)) == 3
