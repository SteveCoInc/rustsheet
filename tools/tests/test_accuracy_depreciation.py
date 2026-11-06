"""Gate A definitions of SLN, SYD, DB, DDB, VDB, AMORLINC and AMORDEGRC (T-4111) against Microsoft's published
examples, the semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import contextlib
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.amortization import AMORDEGRC, AMORLINC
from rustsheet_tools.accuracy.defs.depreciation import DB, DDB, SLN, SYD, VDB
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float | bool) -> float:
    return float(definition.evaluate([a if isinstance(a, bool) else mp(a) for a in args]))


def test_microsofts_examples() -> None:
    cents = 0.005
    assert value(SLN, 30000, 7500, 10) == 2250
    assert value(SYD, 30000, 7500, 10, 1) == pytest.approx(4090.91, abs=cents)
    assert value(SYD, 30000, 7500, 10, 10) == pytest.approx(409.09, abs=cents)
    for period, expected in enumerate(
        (186083.33, 259639.42, 176814.44, 120410.64, 81999.64, 55841.76, 15845.10), start=1
    ):
        assert value(DB, 1000000, 100000, 6, period, 7) == pytest.approx(expected, abs=cents)
    assert value(DDB, 2400, 300, 3650, 1) == pytest.approx(1.32, abs=cents)
    assert value(DDB, 2400, 300, 120, 1, 2) == pytest.approx(40.00, abs=cents)
    assert value(DDB, 2400, 300, 10, 1, 2) == pytest.approx(480.00, abs=cents)
    assert value(DDB, 2400, 300, 10, 2, 1.5) == pytest.approx(306.00, abs=cents)
    assert value(DDB, 2400, 300, 10, 10) == pytest.approx(22.12, abs=cents)
    assert value(VDB, 2400, 300, 3650, 0, 1) == pytest.approx(1.32, abs=cents)
    assert value(VDB, 2400, 300, 120, 0, 1) == pytest.approx(40.00, abs=cents)
    assert value(VDB, 2400, 300, 10, 0, 1) == pytest.approx(480.00, abs=cents)
    assert value(VDB, 2400, 300, 120, 6, 18) == pytest.approx(396.31, abs=cents)
    assert value(VDB, 2400, 300, 120, 6, 18, 1.5) == pytest.approx(311.81, abs=cents)
    assert value(VDB, 2400, 300, 10, 0, 0.875, 1.5) == pytest.approx(315.00, abs=cents)
    assert value(AMORLINC, 2400, 39679, 39813, 300, 1, 0.15, 1) == 360
    assert value(AMORDEGRC, 2400, 39679, 39813, 300, 1, 0.15, 1) == 776


def test_excel_values_of_the_semantics_entries() -> None:
    # "DB and DDB operation order"
    assert value(DB, 100, 10, 4, 1, 2) == pytest.approx(7.3, rel=1e-15)
    assert value(DB, 23, 10, 4, 5, 2) == pytest.approx(1.8687313963962309, rel=1e-14)
    assert value(DB, 10000, 1000, 6, 2, 7) == pytest.approx(2596.3941666666665, rel=1e-15)
    assert value(DDB, 10000, 1000, 6, 2, 2) == pytest.approx(2222.222222222222, rel=1e-15)
    assert value(DDB, 5832, 692, 25, 1) == pytest.approx(466.56, rel=1e-15)
    assert value(DDB, 9695, 282, 20, 8, 1) == pytest.approx(338.5190042814453, rel=1e-14)
    # "DB and SYD argument domains"
    assert value(DB, 100, 0, 4294967296, 0.5, 8) == pytest.approx(11.111111111111114, rel=1e-14)
    assert value(DB, 100, 0, 1e10, 0.5, 6) == 25
    assert value(DB, 100, 10, 4, 2147483648) == pytest.approx(43.8, rel=1e-15)
    assert value(DB, 0.01, 488, 32, 1e200, 10) == pytest.approx(-0.0033416666666666673, rel=1e-14)
    assert value(DB, 1, 0, 1e15, 1e15) == 1
    assert value(DB, 0, 0, 4, 1) == 0
    assert value(DB, 0.01, 488, 32, 33.5, 10) == value(DB, 0.01, 488, 32, 33, 10)
    assert value(SYD, 100, 10, 1.3e154, 1) == pytest.approx(1.3846153846153848e-152, rel=1e-14)
    # "VDB charge order" and "T-1625 probes"
    assert value(VDB, 2400, 300, 10, 7, 10, 2, True) == pytest.approx(203.31648, rel=1e-14)
    assert value(VDB, 2400, 300, 10, 0, 10, 3, True) == pytest.approx(2100, rel=1e-14)
    assert value(VDB, 1234.56, 78.9, 12, 4, 9, 1.75) == pytest.approx(379.8854487417499, rel=1e-14)
    assert value(VDB, 2400, 100, 10, 6.5, 7.5) == pytest.approx(133.20886857142858, rel=1e-14)
    assert value(VDB, 2400, 100, 10, 8.5, 9.5) == pytest.approx(133.20886857142858, rel=1e-14)
    assert value(VDB, 2400, 100, 10, 6.3, 7.3) == pytest.approx(132.80996324324326, rel=1e-14)
    assert value(VDB, 2400, 300, 10, 8.3, 9.9, 1.5) == pytest.approx(244.0757706382978, rel=1e-14)
    assert value(VDB, 2400, 300, 10, 9.25, 10) == pytest.approx(4.512314880000062, rel=1e-12)
    assert value(VDB, 1000, 5, 10, 9.25, 10, 4) == pytest.approx(4.069926399999998, rel=1e-12)
    assert value(VDB, 2400, 3000, 10, 0, 1) == -600
    assert value(VDB, 2400, 3000, 10, 0, 1, 2, True) == 0
    assert value(VDB, 2400, 3000, 10, 0.5, 1.5) == -600
    assert value(VDB, 1000, 100, 2, 0, 1, 3, True) == 900
    assert value(VDB, 1000, 100, 10, 0, 1, 0) == 90
    assert value(VDB, 1000, 100, 10, 0, 1, 0, True) == 0
    assert value(VDB, 1000, 900, 5, 0.5, 1) == 25
    assert value(VDB, 1000, 950, 4, 1.5, 3) == 0
    assert value(VDB, 2400, 300, 10, 9.5, 10, 2, False) == 0
    assert value(VDB, 1000, 100, 10, 0, 0) == 0
    # "AMORLINC and AMORDEGRC over many periods", "check order", "life bands"
    assert value(AMORLINC, 2400, 39679, 39813, 300, 0.5, 0.15, 1) == 360
    assert value(AMORLINC, 2400, 1, 1, 300, 1, 0.15, 1) == 360
    assert value(AMORLINC, 2400, 39679, 39813, 300, 5.5, 0.15, 1) == value(
        AMORLINC, 2400, 39679, 39813, 300, 5, 0.15, 1
    )
    assert value(AMORDEGRC, 1e15, 39679, 39813, 1, 1, 0.000001, 1) == 2499997712
    assert value(AMORDEGRC, 1e15, 39679, 39813, 1, 1000, 0.000001, 1) == 2493761750
    assert value(AMORDEGRC, 2400, 39679, 39813, 300, 2, 0.15, 1) == 485
    assert value(AMORDEGRC, 2400, 0, 366, 300, 1, 0.15, 1) == 562
    assert value(AMORDEGRC, 2400, 0, 1, 300, 1, 0.15, 1) == 899
    assert value(AMORDEGRC, 2400, 39679, 39813, 300, 0, 1 / 3, 1) == 439
    assert value(AMORDEGRC, 2400, 39679, 39813, 300, 0, 1 / 4.5, 1) == 391
    assert value(AMORDEGRC, 2400, 39679, 39813, 300, 0, 1 / 2.01, 1) == 656
    assert value(AMORDEGRC, 1e300, 39679, 39813, 300, 1, 1e-7, 1) == pytest.approx(
        2.49999977117486e293, rel=1e-15
    )


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (SLN, (100, 10, 0)),  # #DIV/0!
        (SYD, (100, -1e-300, 10, 1)),  # negative salvage: #NUM!
        (SYD, (100, 10, 1.4e154, 1)),  # life·(life + 1) overflows: #DIV/0!
        (SYD, (1e300, 1, 1e200, 1)),
        (SYD, (9e307, 0, 10, 1)),  # numerator overflows: #NUM!
        (SYD, (100, -1, 1e200, 1)),  # the salvage's #NUM! first
        (DB, (100, 10, 2147483648, 1)),  # a life past 32 bits: #NUM!
        (DB, (100, 10, 2147483648, 0.5)),
        (DB, (100, 10, 4294967294, 0.5, 8)),
        (DB, (100, 10, 4294967296, 0.5, 12)),
        (DB, (100, 10, 4294967296, 1, 8)),
        (DB, (0, 10, 1e10, 1)),  # before the cost-0 shortcut
        (DB, (100, 10, 4, 2147483647)),  # past life + 1
        (DB, (0, -1, 4, 1)),  # negative salvage
        (DB, (100, 10, 4.5, 6, 6)),
        (DB, (100, 10, 4.5, 5, 12)),  # month 12 past the life
        (AMORLINC, (2400, -1, 39813, 300, 1, 0.15, 1)),  # date off the calendar
        (AMORLINC, (2400, 39679, 2958466, 300, 1, 0.15, 1)),
        (AMORLINC, (-1, 39679, 39813, 300, 1, 0.15, 1)),
        (AMORLINC, (2400, 39679, 39813, 2401, 1, 0.15, 1)),  # salvage above the cost
        (AMORLINC, (2400, 39679, 39813, 300, -1, 0.15, 1)),
        (AMORLINC, (2400, 39679, 39813, 300, 1, 0, 1)),
        (AMORLINC, (2400, 39814, 39813, 300, 1, 0.15, 1)),
        (AMORLINC, (2400, 39679, 39813, 300, 1, 0.15, 2)),
        (AMORLINC, (2400, 39679, 39813, 300, 1, 0.15, -1)),
        (AMORDEGRC, (2400, 39679, 39813, 300, 0, 0.5, 1)),  # an asset life of 2: #NUM!
        # a salvage equal to the cost still meets the asset-life check and the basis, and a cost of 0 is #NUM!
        # (probes/amor-salvage-equal-cost E1, E4, E5, E11, E21, E24)
        (AMORDEGRC, (2400, 39679, 39813, 2400, 1, 0.5, 1)),
        (AMORDEGRC, (2400, 39679, 39813, 2400, 0.5, 0.5, 1)),
        (AMORDEGRC, (2400, 39679, 39813, 2400, 1, 0.15, 2)),
        (AMORDEGRC, (0, 39679, 39813, 0, 1, 0.15, 1)),
        (AMORLINC, (0, 39679, 39813, 0, 0, 0.15, 1)),
        (AMORDEGRC, (2400, 39679, 39813, 300, 0, 1 / 1.99, 1)),
    ],
)
def test_domain_errors_follow_excel(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


def test_points_without_a_value() -> None:
    for definition, args in [
        (AMORDEGRC, (2400, 39679, 39813, 300, 0.5, 0.15, 1)),  # workbook state
        (DB, (100, 10, 200000, 150000)),  # the engine's period cap
        (AMORLINC, (2400, 39679, 39813, 300, 875000, 0.000001, 1)),  # Excel's walk carries 875,000 roundings
        (AMORLINC, (2400, 39679, 39813, 300, 875, 0.001, 1)),  # 875 roundings of 2100 in a remainder of 1.52
        # 46117 in Excel, the cost less the salvage, but the 30/360 count of serial 0 is not probed where it shows
        (AMORLINC, (46557, 0, 39881, 440, 0, 0.1)),
        (AMORDEGRC, (2400, 1, 366, 300, 1, 0.15, 1)),  # 563 in Excel: a charge of 562.5 and a few 1e-16 more
        (VDB, (2400, 300, 10, 10, 10)),  # switching with life, start and end equal (unprobed)
    ]:
        with pytest.raises(NoValueError):
            definition.evaluate([mp(a) for a in args])
    # a salvage equal to the cost is 0 at every period, fractions and period 0 included
    # (probes/amor-salvage-equal-cost A1-C18)
    for period in (0, 0.0001, 0.5, 1, 1.5, 5, 6, 8):
        assert value(AMORLINC, 2400, 39679, 39813, 2400, period, 0.15, 1) == 0
        assert value(AMORDEGRC, 2400, 39679, 39813, 2400, period, 0.15, 1) == 0
    # past the asset life a period charges 0
    assert value(AMORLINC, 2400, 39679, 39813, 300, 8, 0.15, 1) == 0
    assert value(AMORDEGRC, 2400, 39679, 39813, 300, 20, 0.15, 1) == 0


def test_rules_found_by_the_tables() -> None:
    # financial.md "AMORLINC and AMORDEGRC check order": a salvage equal to the cost is 0 (accuracy/AMORLINC/s1 A125,
    # accuracy/AMORDEGRC/s1 A118)
    assert value(AMORLINC, 99977.707698474, 42751, 42993, 99977.707698474, 2, 0.213533204746625) == 0
    assert value(AMORDEGRC, 496430.992883828, 51857, 52179, 496430.992883828, 4, 0.0599516348138364) == 0
    # "VDB charge order": straight line from period 0 covers a fractional start (accuracy/VDB/s1 A188, A187)
    assert value(
        VDB, 121503138647.778, 3864354372.11506, 4, 0.5, 4, 0.15833054798759, False
    ) == pytest.approx(102933936241.20506, rel=1e-15)
    assert value(
        VDB, 147.345559004139, 0.00344462972942079, 21, 12.5, 18.5, 0.295807274286059
    ) == pytest.approx(42.09774696411702, rel=1e-15)


@pytest.mark.parametrize("definition", [SLN, SYD, DB, DDB, VDB, AMORLINC, AMORDEGRC])
def test_draws_are_reproducible_and_evaluate(definition: Definition) -> None:
    assert definition.draw is not None
    for stratum in STRATA:
        a = definition.draw(stratum, random.Random(7))
        assert a == definition.draw(stratum, random.Random(7))
        rng = random.Random(11)
        for _ in range(40):
            args = definition.draw(stratum, rng)
            with contextlib.suppress(DomainError, NoValueError):
                definition.evaluate([x if isinstance(x, bool) else mp(x) for x in args])
