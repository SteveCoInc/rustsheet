"""Gate A definitions of PRICE, YIELD, DURATION and MDURATION (T-4113) against Microsoft's published examples, the
semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import datetime
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs._common import OVERFLOW_POINT
from rustsheet_tools.accuracy.defs.bonds import DURATION, MDURATION, PRICE, YIELD
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def serial(year: int, month: int, day: int) -> int:
    return (datetime.date(year, month, day) - datetime.date(1899, 12, 30)).days


def value(definition: Definition, *args: float) -> float:
    return float(definition.evaluate([mp(a) for a in args]))


def test_microsofts_examples() -> None:
    # PRICE(2008-02-15, 2017-11-15, 5.75%, 6.5%, 100, 2, 0) $94.63; YIELD(2008-02-15, 2016-11-15, 5.75%, 95.04287, 100,
    # 2, 0) 6.5%; DURATION(2018-07-01, 2048-01-01, 8%, 9%, 2, 1) 10.9191453; MDURATION(2008-01-01, 2016-01-01, 8%, 9%,
    # 2, 1) 5.73567
    assert value(PRICE, serial(2008, 2, 15), serial(2017, 11, 15), 0.0575, 0.065, 100, 2, 0) == pytest.approx(
        94.63436, abs=5e-6
    )
    assert value(
        YIELD, serial(2008, 2, 15), serial(2016, 11, 15), 0.0575, 95.04287, 100, 2, 0
    ) == pytest.approx(0.065, abs=5e-8)
    assert value(DURATION, serial(2018, 7, 1), serial(2048, 1, 1), 0.08, 0.09, 2, 1) == pytest.approx(
        10.9191453, abs=5e-8
    )
    assert value(MDURATION, serial(2008, 1, 1), serial(2016, 1, 1), 0.08, 0.09, 2, 1) == pytest.approx(
        5.73567, abs=5e-6
    )


def test_excel_values_of_the_semantics_entries() -> None:
    # "Coupon dates keep the maturity's day": maturity 2027-11-29 quarterly
    assert value(PRICE, 37913, 46720, 0.05, 0.06, 100, 4, 0) == pytest.approx(87.29595935878451, rel=1e-14)
    assert value(DURATION, 37913, 46720, 0.05, 0.06, 4, 0) == pytest.approx(13.296825458940603, rel=1e-14)
    # "PRICE and ODDFPRICE discount with Excel's `^`": a whole-period bond and 240 coupons from serial 1
    assert value(PRICE, serial(2020, 3, 1), serial(2030, 3, 1), 0.06, 0.07, 130, 1, 0) == pytest.approx(
        108.22689722310892, rel=1e-15
    )
    assert value(PRICE, 1, 43831, 0.05, 0.06, 100, 2) == pytest.approx(83.34716628375435, rel=1e-14)
    # "PRICE, DURATION and MDURATION sum order ...": Excel is 32 ulp off the math here
    assert value(PRICE, 38136, 44737, 1e10, 7, 100, 1, 0) == pytest.approx(55709579072.88489, rel=1e-14)
    # "PRICE, DURATION and MDURATION at a huge yield or coupon"
    assert value(PRICE, 47300, 47484, 0.05, 1e300, 100, 2, 0) == pytest.approx(2.05e-298, rel=1e-15)
    assert value(DURATION, 47300, 47484, 0.05, 1e300, 2, 0) == 0.5
    assert value(MDURATION, 47300, 47484, 0.05, 1e300, 2, 0) == pytest.approx(1e-300, rel=1e-15)
    assert value(PRICE, 43831, 47484, 0.05, 5.12154e15, 100, 2, 0) == pytest.approx(
        9.76268856632966e-16, rel=1e-14
    )
    assert value(PRICE, 43922, 47484, 0.05, 1.27233e16, 100, 2, 0) == pytest.approx(
        -1.249999968655949, rel=1e-14
    )
    assert value(PRICE, 43831, 47484, 1e300, 0.05, 100, 2, 0) == pytest.approx(
        7.794581142823408e302, rel=1e-14
    )
    assert value(DURATION, 43831, 47484, 1e300, 0.05, 2, 0) == pytest.approx(4.841148506210218, rel=1e-14)
    assert value(PRICE, 43831, 47484, 1e300, 1e15, 100, 2, 0) == pytest.approx(1e287, rel=1e-14)
    assert value(DURATION, 43831, 47484, 0, 5.12154e15, 2, 0) == 10
    assert value(MDURATION, 43831, 47484, 0, 5.12154e15, 2, 0) == pytest.approx(
        3.905075426531862e-15, rel=1e-14
    )


def test_yield_is_prices_root_and_closed_form_with_one_period() -> None:
    # "YIELD stops short of PRICE's root": the root of YIELD(DATE(2096,3,1),DATE(2100,6,30),0.05,98,100,2,0) is
    # 0.0552358703070220, Excel 0.05523587030251456 (8e-11 below it)
    settle, maturity = serial(2096, 3, 1), serial(2100, 6, 30)
    root = value(YIELD, settle, maturity, 0.05, 98, 100, 2, 0)
    assert root == pytest.approx(0.0552358703070220, rel=1e-14)
    assert value(PRICE, settle, maturity, 0.05, root, 100, 2, 0) == pytest.approx(98, rel=1e-14)
    # "YIELD with one coupon period left is closed form": DSR 0 (the 30th to the 31st on basis 0) is 0
    assert value(YIELD, serial(2024, 1, 30), serial(2024, 1, 31), 0.05, 99, 100, 2, 0) == 0
    # the closed form, by hand: 2024-03-01 to 2024-08-31 on basis 0 is DSR 180, A 181 (from 2023-08-31), E 360
    f, a, e, dsr, rate, pr = 1, 181, 360, 180, 0.05, 99
    closed = ((1 + rate / f) - (pr / 100 + a / e * rate / f)) / (pr / 100 + a / e * rate / f) * (f * e / dsr)
    assert value(YIELD, serial(2024, 3, 1), serial(2024, 8, 31), rate, pr, 100, f, 0) == pytest.approx(closed)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (PRICE, (43831, 43831, 0.05, 0.06, 100, 2)),  # settlement not before maturity
        (PRICE, (0, 43831, 0.05, 0.06, 100, 2)),  # settlement below 1
        (PRICE, (1, 1000, 0.05, 0.06, 100, 2)),  # previous coupon before 1900-01-01
        (PRICE, (39448, 43831, -0.05, 0.06, 100, 2)),
        (PRICE, (39448, 43831, 0.05, -0.06, 100, 2)),
        (PRICE, (39448, 43831, 0.05, 0.06, 0, 2)),
        (PRICE, (39448, 43831, 0.05, 0.06, 100, 3)),
        (PRICE, (39448, 43831, 0.05, 0.06, 100, 2, 5)),
        (PRICE, (39448, 2958466, 0.05, 0.06, 100, 2)),
        (PRICE, (43831, 47484, 0.05, 5.22501e15, 100, 2, 0)),  # (1 + yld/f)^(s+N-1) overflows
        (PRICE, (43922, 47484, 0.05, 1.29803e16, 100, 2, 0)),
        (PRICE, (43831, 47484, 1e306, 0.05, 100, 2, 0)),  # the coupon sum overflows
        (DURATION, (43831, 47484, 9.99999999999999e307, 0.05, 2, 0)),
        (DURATION, (43831, 51136, 0.05, 1.02735e8, 2, 0)),
        (MDURATION, (43831, 47484, 0.05, 3.31587e16, 2, 0)),
        (YIELD, (39448, 43831, 0.05, 0, 100, 2)),  # a price of 0
        (YIELD, (39448, 43831, 0.05, 95, -1, 2)),
    ],
)
def test_evidenced_num_errors(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


def test_no_value_where_unevidenced() -> None:
    # a price at the yield whose power is at the double range's top (Excel's double may fall either side)
    with pytest.raises(NoValueError):
        cut = 2 * (OVERFLOW_POINT ** (mp(1) / 20) - 1)  # N = 20 coupons, settlement on a coupon date
        PRICE.evaluate([mp(a) for a in (43831, 47484, 0.05, float(cut * (1 + mp(1e-13))), 100, 2, 0)])
    # YIELD with one period left from a settlement before 1900-03-01
    with pytest.raises(NoValueError):
        YIELD.evaluate([mp(a) for a in (30, 100, 0.05, 99, 100, 4, 1)])


def test_draws_are_valid_points() -> None:
    rng = random.Random(4113)
    for definition in (PRICE, YIELD, DURATION, MDURATION):
        assert definition.draw is not None
        for stratum in STRATA:
            for _ in range(20):
                args = definition.draw(stratum, rng)
                assert len(definition.params_for(len(args))) == len(args)
                if stratum != "edge":
                    assert 1 <= float(args[0]) < float(args[1])
