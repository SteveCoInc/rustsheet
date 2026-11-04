"""Gate A definitions of BESSELI, BESSELJ, BESSELK and BESSELY (T-4083) against published values, the semantics
entries' Excel values and their domains."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.bessel import (
    BESSELI,
    BESSELJ,
    BESSELK,
    BESSELY,
    I_CUT,
    ORDER_CAP,
    PHASE_DRAW,
)
from rustsheet_tools.accuracy.model import DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def test_published_values() -> None:
    # Abramowitz and Stegun tables 9.1 and 9.8: J0(1), Y0(1), I0(1), K0(1)
    assert mpmath.almosteq(
        BESSELJ.evaluate([mp(1), mp(0)]), mp("0.765197686557966551449717526103"), mp("1e-28")
    )
    assert mpmath.almosteq(
        BESSELY.evaluate([mp(1), mp(0)]), mp("0.0882569642156769579829267660236"), mp("1e-28")
    )
    assert mpmath.almosteq(
        BESSELI.evaluate([mp(1), mp(0)]), mp("1.26606587775200833559824462521"), mp("1e-28")
    )
    assert mpmath.almosteq(
        BESSELK.evaluate([mp(1), mp(0)]), mp("0.421024438240708333335627379213"), mp("1e-28")
    )


def test_excel_values_of_the_semantics_entries_are_close_to_the_math() -> None:
    # engineering.md T-0900: Excel's BESSELJ(8,1) 0.2346283865070744 is 3.4E-5 off the true 0.2346363468539146,
    # inside R3's G = 1e-4; BESSELY(8,0) 0.22352148924968093 is about 6E-9 off.
    j = float(BESSELJ.evaluate([mp(8), mp(1)]))
    assert j == pytest.approx(0.2346363468539146, rel=1e-15)
    assert abs(0.2346283865070744 - j) / j < 1e-4
    assert float(BESSELY.evaluate([mp(8), mp(0)])) == pytest.approx(0.22352148924968093, rel=1e-8)
    # T-2613: BESSELI(709.7,0) is 2.4788083687166675E+306 in Excel, 1.4E-7 below the math (inside G = 1e-4)
    i = float(BESSELI.evaluate([mp(I_CUT), mp(0)]))
    assert i == pytest.approx(2.4788087273156452e306, rel=1e-15)
    assert abs(2.4788083687166675e306 - i) / i < 1e-6
    # T-2421: BESSELK(705,0) is 3.135297088468052E-308 in Excel
    assert float(BESSELK.evaluate([mp(705), mp(0)])) == pytest.approx(3.135297088468052e-308, rel=1e-12)


def test_order_truncates_and_parity() -> None:
    assert BESSELJ.evaluate([mp(8), mp("9.99")]) == BESSELJ.evaluate([mp(8), mp(9)])
    assert BESSELY.evaluate([mp(30), mp("1.99")]) == BESSELY.evaluate([mp(30), mp(1)])
    assert mpmath.almosteq(BESSELJ.evaluate([mp(-3), mp(1)]), -BESSELJ.evaluate([mp(3), mp(1)]), mp("1e-35"))
    assert mpmath.almosteq(BESSELI.evaluate([mp(-3), mp(2)]), BESSELI.evaluate([mp(3), mp(2)]), mp("1e-35"))
    assert BESSELJ.evaluate([mp(0), mp(2)]) == 0


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (BESSELJ, [mp(1), mp(-1)]),
        (BESSELY, [mp(0), mp(0)]),
        (BESSELY, [mp(-1), mp(0)]),
        (BESSELK, [mp(0), mp(1)]),
        (BESSELI, [mp("709.78"), mp(0)]),
        (BESSELI, [mp(-710), mp(1)]),
        (BESSELJ, [mp("1e10"), mp(0)]),
        (BESSELY, [mp("1e14"), mp(1)]),
    ],
)
def test_domain_errors(definition: object, args: list[mpmath.mpf]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate(args)  # type: ignore[attr-defined]


def test_draws_stay_inside_the_scored_region() -> None:
    rng = random.Random(7)
    for definition in (BESSELI, BESSELJ, BESSELK, BESSELY):
        assert definition.draw is not None
        for stratum in STRATA:
            for _ in range(200):
                x, n = definition.draw(stratum, rng)
                assert isinstance(x, float)
                assert 0 <= float(n) <= ORDER_CAP
                if stratum != "edge" and definition in (BESSELJ, BESSELY):
                    assert abs(x) <= PHASE_DRAW
                if definition in (BESSELY, BESSELK):
                    assert x >= 0
