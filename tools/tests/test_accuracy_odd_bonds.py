"""Gate A definitions of ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD (T-4115) against Microsoft's published
examples, the semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import datetime
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.odd_bonds import ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD
from rustsheet_tools.accuracy.defs.yearfrac import us_30_360_both
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
    # ODDFPRICE(2008-11-11, 2021-03-01, 2008-10-15, 2009-03-01, 7.85%, 6.25%, 100, 2, 1) $113.60;
    # ODDFYIELD(2008-11-11, 2021-03-01, 2008-10-15, 2009-03-01, 5.75%, 84.50, 100, 2, 0) 7.72%;
    # ODDLYIELD(2008-04-20, 2008-06-15, 2007-12-24, 3.75%, 99.875, 100, 2, 0) 4.52%
    first = (serial(2008, 11, 11), serial(2021, 3, 1), serial(2008, 10, 15), serial(2009, 3, 1))
    assert value(ODDFPRICE, *first, 0.0785, 0.0625, 100, 2, 1) == pytest.approx(113.597717, abs=5e-6)
    assert value(ODDFYIELD, *first, 0.0575, 84.5, 100, 2, 0) == pytest.approx(0.0772455, abs=5e-7)
    last = (serial(2008, 4, 20), serial(2008, 6, 15), serial(2007, 12, 24))
    assert value(ODDLYIELD, *last, 0.0375, 99.875, 100, 2, 0) == pytest.approx(0.0451922, abs=5e-7)


def test_excel_values_of_the_semantics_entries() -> None:
    # "PRICE and ODDFPRICE discount with Excel's `^`"
    assert value(ODDFPRICE, 39448, 43831, 39083, 39630, 0.08, 0.09, 100, 2, 0) == pytest.approx(
        92.4077632093273, rel=1e-14
    )
    # "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order"
    assert value(ODDLPRICE, 42918, 43200, 42917, 0.0575, 0.065, 100, 2, 0) == pytest.approx(
        99.44775142683748, rel=1e-14
    )
    # "ODDFPRICE and ODDLPRICE at a huge yield"
    assert value(ODDFPRICE, 43831, 47484, 43800, 44013, 0.05, 5.12154e15, 100, 2, 0) == pytest.approx(
        -0.41666666666666546, rel=1e-13
    )
    assert value(ODDLPRICE, 47300, 47484, 47119, 0.05, 9.99999999999999e307, 100, 2, 0) == -2.5
    # "ODDFYIELD and ODDLYIELD at a price far from par": Excel within 3e-12 of the root
    bond = (43831, 47484, 43800, 44013, 0.05)
    assert value(ODDFYIELD, *bond, 1, 100, 2, 0) == pytest.approx(3.9188814666622176, rel=1e-11)
    assert value(ODDFYIELD, *bond, 1000, 100, 2, 0) == pytest.approx(-0.19969492032817104, rel=1e-11)
    assert abs(value(ODDFYIELD, *bond, 150, 100, 2, 0)) < 1e-15


def test_oddfyield_roots_of_t3031() -> None:
    # "ODDFYIELD stops short of ODDFPRICE's root": the 50-digit roots of probes-numeric/t3031-r4-oddfyield-long-first
    # S!A61 and S!A70 (28 February first coupons) are 0.077480666073101446 and 0.12192508630779288; Excel is
    # 0.07748066607022848 and 0.12192508631378622
    a61 = (
        serial(2010, 1, 7),
        serial(2037, 2, 28),
        serial(2009, 8, 12),
        serial(2010, 2, 28),
        0.0085,
        21.426,
        100,
        2,
        0,
    )
    a70 = (
        serial(2017, 12, 3),
        serial(2038, 2, 28),
        serial(2017, 3, 2),
        serial(2019, 2, 28),
        0.0179,
        23.169,
        134.68,
        1,
        2,
    )
    assert value(ODDFYIELD, *a61) == pytest.approx(0.077480666073101446, rel=1e-15)
    assert value(ODDFYIELD, *a70) == pytest.approx(0.12192508630779288, rel=1e-15)


def test_yields_invert_the_prices() -> None:
    first = (serial(2023, 3, 16), serial(2030, 6, 15), serial(2022, 12, 15), serial(2024, 6, 15), 0.05)
    for basis in range(5):  # a long first period
        price = value(ODDFPRICE, *first, 0.07, 100, 2, basis)
        assert value(ODDFYIELD, *first, price, 100, 2, basis) == pytest.approx(0.07, rel=1e-13)
    last = (serial(2024, 2, 1), serial(2025, 3, 31), serial(2023, 8, 30), 0.04)
    for basis in range(5):  # a long last period over a February (the drifting walk)
        price = value(ODDLPRICE, *last, 0.03, 105, 4, basis)
        assert value(ODDLYIELD, *last, price, 105, 4, basis) == pytest.approx(0.03, rel=1e-13)


def test_us_30_360_both() -> None:
    # both dates modified: a 31st end is 30 whatever the start; February's last day is 30
    assert us_30_360_both((2023, 1, 15), (2023, 1, 31)) == 15
    assert us_30_360_both((2023, 2, 28), (2023, 3, 31)) == 30
    assert us_30_360_both((2024, 1, 31), (2024, 2, 29)) == 30


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        # "ODDFPRICE and ODDFYIELD need issue < settlement < first_coupon"
        (
            ODDFPRICE,
            (
                serial(2024, 3, 15),
                serial(2030, 6, 15),
                serial(2024, 3, 15),
                serial(2024, 6, 15),
                0.05,
                0.06,
                100,
                2,
                0,
            ),
        ),
        (
            ODDFPRICE,
            (
                serial(2027, 1, 1),
                serial(2030, 6, 15),
                serial(2024, 3, 15),
                serial(2024, 6, 15),
                0.05,
                0.06,
                100,
                2,
                0,
            ),
        ),
        # "ODDFPRICE and ODDFYIELD need the first coupon on the maturity's schedule"
        (
            ODDFPRICE,
            (
                serial(2024, 2, 1),
                serial(2025, 7, 2),
                serial(2024, 1, 1),
                serial(2024, 7, 1),
                0.05,
                0.06,
                100,
                2,
                0,
            ),
        ),
        (
            ODDFYIELD,
            (
                serial(2024, 2, 1),
                serial(2025, 7, 1),
                serial(2024, 1, 1),
                serial(2024, 7, 1),
                0.05,
                99,
                100,
                2,
                0,
            ),
        ),  # DFC = E
        (
            ODDFPRICE,
            (
                serial(2023, 8, 1),
                serial(2024, 6, 30),
                serial(2023, 7, 1),
                serial(2023, 12, 31),
                0.05,
                0.06,
                100,
                2,
                0,
            ),
        ),  # 180 days 30/360
        # "ODDFPRICE and ODDLPRICE at a huge yield"
        (ODDFPRICE, (43831, 47484, 43800, 44013, 0.05, 5.22501e15, 100, 2, 0)),
        # "ODDLPRICE and ODDLYIELD: a quasi-coupon date past the calendar is #NUM!"
        (ODDLPRICE, (serial(9999, 8, 1), serial(9999, 12, 31), serial(9999, 5, 15), 0.05, 0.06, 100, 2, 0)),
        (ODDLYIELD, (serial(9999, 8, 1), serial(9999, 12, 31), serial(9999, 5, 15), 0.05, 99, 100, 2, 1)),
        # "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order"
        (ODDLPRICE, (42917, 43200, 42917, 0.0575, 0.065, 100, 2, 0)),
        (ODDLPRICE, (42918, 43200, 42917, -0.05, 0.065, 100, 2, 0)),
        (ODDLPRICE, (42918, 43200, 42917, 0.05, -0.065, 100, 2, 0)),
        (ODDLYIELD, (42918, 43200, 42917, 0.05, 0, 100, 2, 0)),
        (ODDLPRICE, (42918, 43200, 42917, 0.05, 0.065, 0, 2, 0)),
        (ODDLPRICE, (42918, 43200, 42917, 0.05, 0.065, 100, 3, 0)),
        (ODDLPRICE, (42918, 43200, 42917, 0.05, 0.065, 100, 2, 5)),
        (ODDLPRICE, (42918, 2958466, 42917, 0.05, 0.065, 100, 2, 0)),
    ],
)
def test_evidenced_num_errors(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


def test_no_value_where_unevidenced() -> None:
    # a date before 1901-03-01, and a quasi-coupon date past the calendar on basis 2
    with pytest.raises(NoValueError):
        ODDLPRICE.evaluate([mp(a) for a in (200, 300, 100, 0.05, 0.06, 100, 2, 0)])
    with pytest.raises(NoValueError):
        ODDLPRICE.evaluate(
            [
                mp(a)
                for a in (
                    serial(9999, 8, 1),
                    serial(9999, 12, 31),
                    serial(9999, 5, 15),
                    0.05,
                    0.06,
                    100,
                    2,
                    2,
                )
            ]
        )
    # DFC = E where it is unprobed: basis 3 annual (365 days) and basis 2 quarterly (90 days)
    for args in (
        (
            serial(2024, 9, 1),
            serial(2027, 6, 30),
            serial(2024, 6, 30),
            serial(2025, 6, 30),
            0.05,
            0.06,
            100,
            1,
            3,
        ),
        (
            serial(2024, 5, 1),
            serial(2025, 6, 30),
            serial(2024, 4, 1),
            serial(2024, 6, 30),
            0.05,
            0.06,
            100,
            4,
            2,
        ),
    ):
        with pytest.raises(NoValueError):
            ODDFPRICE.evaluate([mp(a) for a in args])
    # ODDLYIELD with no days from settlement to the maturity (the 30th to the 31st on 30/360)
    with pytest.raises(NoValueError):
        ODDLYIELD.evaluate(
            [
                mp(a)
                for a in (serial(2024, 1, 30), serial(2024, 1, 31), serial(2023, 12, 15), 0.05, 99, 100, 2, 0)
            ]
        )


def test_draws_are_valid_points() -> None:
    rng = random.Random(4115)
    for definition in (ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD):
        assert definition.draw is not None
        for stratum in STRATA:
            for _ in range(20):
                args = definition.draw(stratum, rng)
                assert len(definition.params_for(len(args))) == len(args)
                if stratum != "edge":
                    assert float(args[0]) < float(args[1])
