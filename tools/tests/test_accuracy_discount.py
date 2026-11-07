"""Gate A definitions of DISC, INTRATE, RECEIVED, PRICEDISC, YIELDDISC, TBILLEQ and TBILLPRICE (T-4112) against
Microsoft's published examples, the semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import contextlib
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.discount import (
    DISC,
    INTRATE,
    PRICEDISC,
    RECEIVED,
    TBILLEQ,
    TBILLPRICE,
    YIELDDISC,
)
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf
DEFINITIONS = [DISC, INTRATE, RECEIVED, PRICEDISC, YIELDDISC, TBILLEQ, TBILLPRICE]


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float) -> float:
    return float(definition.evaluate([mp(a) for a in args]))


def test_microsofts_examples() -> None:
    # DISC(2018-01-25,2018-06-15,97.975,100,1) 0.052420213; INTRATE(2008-02-15,2008-05-15,1000000,1014420,2)
    # 0.05768; RECEIVED(2008-02-15,2008-05-15,1000000,0.0575,2) 1014584.654; PRICEDISC(2008-02-16,2008-03-01,
    # 0.0525,100,2) 99.79583; YIELDDISC(2008-02-16,2008-03-01,99.795,100,2) 0.052823; TBILLPRICE(2008-03-31,
    # 2008-06-01,0.09) 98.45; TBILLEQ(2008-03-31,2008-06-01,0.0914) 0.094151494
    assert value(DISC, 43125, 43266, 97.975, 100, 1) == pytest.approx(0.052420213, abs=5e-10)
    assert value(INTRATE, 39493, 39583, 1000000, 1014420, 2) == pytest.approx(0.05768, abs=5e-6)
    assert value(RECEIVED, 39493, 39583, 1000000, 0.0575, 2) == pytest.approx(1014584.654, abs=5e-4)
    assert value(PRICEDISC, 39494, 39508, 0.0525, 100, 2) == pytest.approx(99.79583, abs=5e-6)
    assert value(YIELDDISC, 39494, 39508, 99.795, 100, 2) == pytest.approx(0.052823, abs=5e-7)
    assert value(TBILLPRICE, 39538, 39600, 0.09) == pytest.approx(98.45, abs=5e-3)
    assert value(TBILLEQ, 39538, 39600, 0.0914) == pytest.approx(0.094151494, abs=5e-10)


def test_excel_values_of_the_semantics_entries() -> None:
    # financial.md "Discount-security functions: settlement 0, basis and argument modes"
    assert value(INTRATE, 0, 43466, 1000, 1100) == pytest.approx(0.0008403165192222404, rel=1e-15)
    assert value(PRICEDISC, 0, 43466, 0.05, 100) == pytest.approx(-495.0138888888889, rel=1e-15)
    assert value(YIELDDISC, 0, 43466, 95, 100) == pytest.approx(0.00044227185222223174, rel=1e-15)
    # probes/discount-serial-zero A1, E1: DISC(0,366,95,100,0) and basis 4
    assert value(DISC, 0, 366, 95, 100, 0) == pytest.approx(0.04986149584487539, rel=1e-14)
    assert value(DISC, 0, 366, 95, 100, 4) == pytest.approx(0.05, rel=1e-14)
    # "Discount securities: day count and year length": INTRATE.xlsx F2 is 0.08, and 30 days of basis 0 1.2
    assert value(INTRATE, 36526, 36616, 2000000, 2040000, 0) == pytest.approx(0.08, rel=1e-15)
    assert value(INTRATE, 43159, 43189, 1000, 1100, 0) == pytest.approx(1.2, rel=1e-15)
    assert value(INTRATE, 43159, 43189, 1000, 1100, 4) == pytest.approx(1.125, rel=1e-15)
    # "Discount securities check order": RECEIVED(39448,43831,1,0.05,0) is 2.5
    assert value(RECEIVED, 39448, 43831, 1, 0.05, 0) == pytest.approx(2.5, rel=1e-15)
    # "TBILLEQ and TBILLYIELD", "T-bill settlement below serial 1"
    assert value(TBILLEQ, 2040, 2223, 0.8) == pytest.approx(1.364497682211261, rel=1e-14)
    assert value(TBILLEQ, 0, 366, 0.1) == pytest.approx(0.11013984535998977, rel=1e-15)
    # A price of exactly 0 (4·90 is 360): TBILLEQ 0 up to 182 days, like TBILLEQ(1000,1100,3.6), TBILLPRICE 0
    assert value(TBILLEQ, 1000, 1090, 4) == 0
    assert value(TBILLPRICE, 1000, 1090, 4) == 0
    for args in [(0, 366, 0.1), (1, 367, 0.1), (60, 426, 0.1), (45351, 45717, 0.1)]:
        assert value(TBILLPRICE, *args) == pytest.approx(89.83333333333333, rel=1e-15)
    for args in [(0, 1, 0.1), (0.5, 1.9, 0.1), (39000.7, 39001.2, 0.1)]:
        assert value(TBILLPRICE, *args) == pytest.approx(99.97222222222221, rel=1e-15)
    assert value(TBILLPRICE, 2958100, 2958465, 0.05) == pytest.approx(94.93055555555556, rel=1e-15)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (DISC, (43831, 43466, 95, 100)),  # settlement after maturity
        (DISC, (43466, 43466, 95, 100)),
        (DISC, (43466, 43831, 95, 100, 5)),  # basis outside 0..4
        (INTRATE, (43466, 43831, 1000, 1100, -1)),
        (DISC, (-1, 43831, 95, 100)),  # off the calendar
        (DISC, (43466, 2958466, 95, 100)),
        (INTRATE, (43466, 43831, 0, 1100)),  # an amount at most 0
        (YIELDDISC, (43466, 43831, 95, -100)),
        (PRICEDISC, (43466, 43831, 0, 100)),
        (RECEIVED, (43466, 43831, -1000, 0.05)),
        (RECEIVED, (0, 43466, 1000, 0.05)),  # settlement 0
        (RECEIVED, (39448, 39448 + 360, 1000, 1, 2)),  # 1 - discount*D/B = 0 (RECEIVED(...,1000,1))
        (TBILLPRICE, (0, 367, 0.1)),  # more than a year
        (TBILLPRICE, (61, 427, 0.1)),
        (TBILLPRICE, (0.5, 0.9, 0.1)),  # equal days
        (TBILLPRICE, (2958101, 2958110, 0.05)),  # settlement in 9999
        (TBILLPRICE, (1000, 1100, 4)),  # a negative price
        (TBILLEQ, (1000, 1100, 4)),
        (TBILLEQ, (1000, 1183, 2)),
        (TBILLEQ, (1000, 1360, 1)),
        (TBILLEQ, (-1, 39100, 0.05)),
        (TBILLEQ, (39427, 39448, -0.5)),  # a negative discount (gen-m9 31474c20fc1dafd6 A241)
        (RECEIVED, (36767, 46418, 285642, 0.05)),  # 1 - discount*D/B below 0 (gen-m9 01523d247594952e A152)
        (RECEIVED, (38015, 45886, 14389, 0)),  # discount 0 (gen-m9 30e20ba3302fea8b A49)
        (DISC, (43831, 45494, 0, 104, 0)),  # price 0 (gen-m9 1dce4774f5c0e736 A227)
        (DISC, (43466, 43831, 95, 0, 5)),  # redemption 0 is unprobed, but the basis is read first
        (DISC, (43831, 43466, 95, 0)),  # and so are the dates
    ],
)
def test_domain_errors_follow_excel(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


def test_unprobed_points_have_no_value() -> None:
    for definition, args in [
        (DISC, (43466, 43831, 95, 0)),  # DISC at redemption 0
        (TBILLPRICE, (45000, 45100, 0)),  # a T-bill discount of 0
        (DISC, (43130, 43131, 95, 100, 0)),  # 2018-01-30 to 01-31: D = 0 on basis 0
        (INTRATE, (43130, 43131, 95, 100, 4)),
        (DISC, (43466, 43831, 100 - 1e-9, 100)),  # 1 - pr/redemption cancels
        (TBILLEQ, (1000, 1100, 3.6)),  # 3.6·100 is 360 in doubles but above it exactly: Excel's 0, a hair off
        (TBILLPRICE, (1000, 1100, 3.6)),
        (TBILLEQ, (1000, 1200, 1.8)),
    ]:
        with pytest.raises(NoValueError):
            definition.evaluate([mp(a) for a in args])


def test_tbilleq_branches_meet_at_183_days() -> None:
    # Both forms are the bond-equivalent yield; at 182 and 183 days they differ only through the compounding term
    short = value(TBILLEQ, 45000, 45182, 0.05)
    long = value(TBILLEQ, 45000, 45183, 0.05)
    assert short == pytest.approx(365 * 0.05 / (360 - 0.05 * 182), rel=1e-15)
    assert long == pytest.approx(short, rel=1e-3)


@pytest.mark.parametrize("definition", DEFINITIONS)
def test_draws_are_reproducible_and_evaluate(definition: Definition) -> None:
    assert definition.draw is not None
    for stratum in STRATA:
        a = definition.draw(stratum, random.Random(7))
        assert a == definition.draw(stratum, random.Random(7))
        rng = random.Random(11)
        for _ in range(40):
            args = definition.draw(stratum, rng)
            with contextlib.suppress(DomainError, NoValueError):
                definition.evaluate([mp(x) for x in args if not isinstance(x, tuple)])
