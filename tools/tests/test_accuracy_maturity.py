"""Gate A definitions of ACCRINT, ACCRINTM, PRICEMAT, YIELDMAT and TBILLYIELD (T-4114) against Microsoft's published
examples, the semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import contextlib
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.accrint import ACCRINT
from rustsheet_tools.accuracy.defs.discount import ACCRINTM, PRICEMAT, TBILLYIELD, YIELDMAT
from rustsheet_tools.accuracy.defs.yearfrac import serial_of
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf
DEFINITIONS = [ACCRINT, ACCRINTM, PRICEMAT, YIELDMAT, TBILLYIELD]


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def math_args(*args: float | bool) -> list[MathArg]:
    return [a if isinstance(a, bool) else mp(a) for a in args]


def value(definition: Definition, *args: float | bool) -> float:
    return float(definition.evaluate(math_args(*args)))


def test_microsofts_examples() -> None:
    # ACCRINT(2008-03-01,2008-08-31,2008-05-01,0.1,1000,2,0) 16.666666667, issue 2008-03-05 with calc_method FALSE
    # 15.555555556, issue 2008-04-05 TRUE 7.2222222; ACCRINTM(2008-04-01,2008-06-15,0.1,1000,3) 20.54794521;
    # PRICEMAT(2008-02-15,2008-04-13,2007-11-11,0.061,0.061,0) 99.98449888; YIELDMAT(2008-03-15,2008-11-03,
    # 2007-11-08,0.0625,100.0123,0) 0.060954334; TBILLYIELD(2008-03-31,2008-06-01,98.45) 0.0914
    assert value(ACCRINT, 39508, 39691, 39569, 0.1, 1000, 2, 0) == pytest.approx(16.666666667, abs=5e-10)
    assert value(ACCRINT, 39512, 39691, 39569, 0.1, 1000, 2, 0, False) == pytest.approx(
        15.555555556, abs=5e-10
    )
    assert value(ACCRINT, 39543, 39691, 39569, 0.1, 1000, 2, 0, True) == pytest.approx(7.2222222, abs=5e-8)
    assert value(ACCRINTM, 39539, 39614, 0.1, 1000, 3) == pytest.approx(20.54794521, abs=5e-9)
    assert value(PRICEMAT, 39493, 39551, 39397, 0.061, 0.061, 0) == pytest.approx(99.98449888, abs=5e-9)
    assert value(YIELDMAT, 39522, 39755, 39394, 0.0625, 100.0123, 0) == pytest.approx(0.060954334, abs=5e-10)
    assert value(TBILLYIELD, 39538, 39600, 98.45) == pytest.approx(0.0914, abs=5e-5)


def test_excel_values_of_the_semantics_entries() -> None:
    # financial.md "ACCRINT at the calendar's start" and "ACCRINT with calc_method TRUE: a next coupon past the
    # calendar is #NUM!" (settlement 9999-07-01: 274.86)
    accrint = value(ACCRINT, 92, 182, serial_of(9990, 6, 30), 0.05, 1000, 4, 0, True)
    assert accrint == pytest.approx(404512.3611111111, rel=1e-15)
    assert value(ACCRINT, 2956276, 2956456, 2958282, 0.05, 1000, 2, 0, True) == pytest.approx(
        274.86, abs=5e-3
    )
    # dates.md "YEARFRAC accepts serial 0": ACCRINTM keeps a negative 30/360 day count; probes/discount-serial-zero
    assert value(ACCRINTM, 59, 60, 0.1, 1000, 0) == pytest.approx(-0.2777777777777778, rel=1e-15)
    assert value(ACCRINTM, 0, 366, 0.1, 1000, 0) == pytest.approx(100.27777777777777, rel=1e-15)
    assert value(ACCRINTM, 0, 366, 0.1, 1000, 4) == pytest.approx(100.0, rel=1e-15)
    # "Discount securities: day count and year length" and "Discount-security functions: settlement 0, ..."
    assert value(YIELDMAT, 43101, 43466, 42736, 0.06, 98, 0) == pytest.approx(0.07692307692307698, rel=1e-14)
    assert value(PRICEMAT, 43101, 43466, 42736, 0, 0.05) == pytest.approx(95.23809523809524, rel=1e-15)
    assert value(YIELDMAT, 43101, 43466, 42736, 0, 98) == pytest.approx(0.02040816326530614, rel=1e-14)
    # probes/matdisc-serial-0-60 A1: PRICEMAT(59,400,0,0.05,0.06,0)
    assert value(PRICEMAT, 59, 400, 0, 0.05, 0.06, 0) == pytest.approx(99.07605454704901, rel=1e-14)
    # "TBILLEQ and TBILLYIELD", probes/tbill-settlement-below-one D1, D2
    assert value(TBILLYIELD, 45366, 45731, 2000) == pytest.approx(-0.9369863013698629, rel=1e-15)
    assert value(TBILLYIELD, 0, 1, 98) == pytest.approx(7.346938775510203, rel=1e-15)
    assert value(TBILLYIELD, 0, 366, 98) == pytest.approx(0.020073603211776512, rel=1e-15)
    # A yld of 0 computes (gen-m9 9a1fa5366bc50918 A180)
    assert value(PRICEMAT, 36890, 44902, 33776, 0.1, 0, 3) == pytest.approx(319.5068493150685, rel=1e-14)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (ACCRINT, (61, 182, 122, 0.05, 1000, 4, 0, True)),  # the issue's period starts 1899-12-31
        (ACCRINT, (2956276, 2956456, 2958283, 0.05, 1000, 2, 0, True)),  # next coupon 10000-01-01
        (ACCRINT, (2956276, 2956456, 2958283, 0.05, 1000, 2, 1, True)),
        (ACCRINT, (2956276, 2956456, 2958467, 0.05, 1000, 2, 0, False)),  # a settlement past the calendar
        (ACCRINT, (39508, 39691, 39508, 0.1, 1000, 2, 0)),  # issue on settlement
        (ACCRINT, (39508, 39691, 39569, 0, 1000, 2, 0)),  # rate 0 (gen-m9 f81bd3b6dc373e5c A180)
        (ACCRINT, (34667, 45535, 45351, -0.5, 2421, 2)),  # gen-m9 290180acf017709c A177
        (ACCRINT, (34052, 36345, 39703, 1e300, -1000, 4)),  # gen-m9 290180acf017709c A147
        (ACCRINT, (39508, 39691, 39569, 0.1, 1000, 2, -1)),  # a negative basis
        (ACCRINT, (-1, 39691, 39569, 0.1, 1000, 2, 0)),
        (ACCRINTM, (39083, 39448, 0, 1000)),  # rate 0 (gen-m9 5c6a23c4f6597f1c A118)
        (ACCRINTM, (33676, 37345, -0.5, 3341)),  # gen-m9 0a957119475760c5 A92
        (ACCRINTM, (33054, 47664, 0.2, -1000)),  # gen-m9 1cd3096e3e87c965 A123
        (ACCRINTM, (39448, 39083, 0.1, 1000)),
        (ACCRINTM, (39083, 39448, 0.1, 1000, 5)),
        (ACCRINTM, (-1, 39448, 0.1, 1000, 0)),
        (PRICEMAT, (43101, 43466, 42736, -0.06, 0.05)),  # probes/discount-security-modes A107
        (PRICEMAT, (37717, 44053, 34809, 0.1, -1, 4)),  # gen-m9 1cd3096e3e87c965 A140
        (PRICEMAT, (39448, 43831, 39448, 0.06, 0.05)),  # issue on settlement
        (PRICEMAT, (59, 400, 59, 0.05, 0.06, 0)),  # probes/matdisc-serial-0-60
        (YIELDMAT, (43101, 43466, 42736, 0.06, -98)),  # probes/discount-security-modes A130
        (YIELDMAT, (38011, 44323, 34864, -0.05, 112)),  # gen-m9 67a4c03afd63f988 A179
        (YIELDMAT, (43466, 43101, 42736, 0.06, 98)),
        (TBILLYIELD, (45366, 45366, 98)),  # probes/tbill-equal-dates A1
        (TBILLYIELD, (0, 367, 98)),  # probes/tbill-settlement-below-one D3
    ],
)
def test_domain_errors_follow_excel(definition: Definition, args: tuple[float | bool, ...]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate(math_args(*args))


def test_unprobed_points_have_no_value() -> None:
    for definition, args in [
        (TBILLYIELD, (45000, 45100, 0)),  # a price of 0
        (TBILLYIELD, (45000, 45100, -5)),
        (TBILLYIELD, (45000, 45100, 100 - 1e-12)),  # 100 - pr cancels
        (YIELDMAT, (43101, 43466, 42736, 0.06, 0)),  # a price of 0
        (YIELDMAT, (43130, 43131, 43100, 0.06, 98, 0)),  # issue 2017-12-31: DSM = 0 on basis 0
        (ACCRINTM, (39083, 39448, 0.1, 0)),  # a par of 0
        (ACCRINT, (39508, 39691, 39569, 0.1, 0, 2, 0)),
        (ACCRINT, (39508, 39691, 39569, 0.1, 1000, 3, 0)),  # frequency 3
        (ACCRINT, (49116, 49633, 49117, 0.00183790807306151, 21.6508879408564, 2, 2)),  # cancels to exactly 0
        (ACCRINT, (39508, 39691, 39569, 0.1, 1000, 2, 5)),  # basis 5
        (ACCRINT, (33853, 1, 37554, 1, 9888, 2, 1)),  # serials below 61 are not modelled
        (ACCRINT, (2956276, 2956456, 2958283, 0.05, 1000, 2, 2, True)),  # next coupon past 9999, basis 2
    ]:
        with pytest.raises(NoValueError):
            definition.evaluate(math_args(*args))


def test_accrint_over_the_schedule_matches_excel_cells() -> None:
    # Settlement years after first_interest, calc_method TRUE and FALSE, every basis: eval cells' Excel values
    for args, excel in [
        ((33853, 35000, 37554, 0.05, 1000, 2, 0), 506.80555555555554),  # probes/accrint-boolean-args A29
        ((2956276, 2956456, 2958464, 0.05, 1000, 2, 0, False), 299.72222222222223),  # -settlement-bound A21
        ((34075, 36190, 38049, 10, 1082, 1, 3), 117745.31506849313),  # gen-m9 f81bd3b6dc373e5c A10
        ((34716, 35336, 38900, 1e-10, 3511, 4, 4), 4.023020833333334e-06),  # f81bd3b6dc373e5c A30
        (
            (39508, 39691, 39569, 0.1, 1000, 2, 0, False),
            16.666666666666664,
        ),  # probes/t2492-vdb-ddb-arguments A48
    ]:
        assert value(ACCRINT, *args) == pytest.approx(excel, rel=1e-14)
    # 30/360 basis 0 does not move a 31st after February's last day: 2009-02-28 to 2009-08-31 is 181 days
    assert value(ACCRINT, 39508, 39691, 40056, 0.1, 1000, 2, 0, True) == pytest.approx(
        150 + 50 / 180, rel=1e-15
    )
    assert value(ACCRINT, 39508, 39691, 40056, 0.1, 1000, 2, 0, False) == pytest.approx(150, rel=1e-15)


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
                definition.evaluate(
                    [x if isinstance(x, bool) else mp(x) for x in args if not isinstance(x, tuple)]
                )
