"""Gate A definitions of FV, PV, PMT, IPMT, PPMT, ISPMT and PDURATION (T-4109) against Microsoft's published
examples, the semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import contextlib
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.annuity_values import FV, IPMT, ISPMT, PDURATION, PMT, PPMT, PV
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float) -> float:
    return float(definition.evaluate([mp(a) for a in args]))


def test_microsofts_examples() -> None:
    # Microsoft's examples: FV(0.06/12,10,-200,-500,1) $2,581.40; PV(0.08/12,12*20,500) ($59,777.15);
    # PMT(0.08/12,10,10000) ($1,037.03); IPMT(0.1/12,1,3*12,8000) ($66.67); PPMT(0.1/12,1,2*12,2000) ($75.62);
    # ISPMT(0.1/12,1,3*12,8000000) -64814.8148 (per 1, rate 0.1/12, nper 36); PDURATION(0.025,2000,2200) 3.86
    cents = 0.005
    assert value(FV, 0.06 / 12, 10, -200, -500, 1) == pytest.approx(2581.40, abs=cents)
    assert value(PV, 0.08 / 12, 240, 500, 0, 0) == pytest.approx(-59777.15, abs=cents)
    assert value(PMT, 0.08 / 12, 10, 10000) == pytest.approx(-1037.03, abs=cents)
    assert value(IPMT, 0.1 / 12, 1, 36, 8000) == pytest.approx(-66.67, abs=cents)
    assert value(PPMT, 0.1 / 12, 1, 24, 2000) == pytest.approx(-75.62, abs=cents)
    assert value(ISPMT, 0.1 / 12, 1, 36, 8000000) == pytest.approx(-64814.8148, abs=5e-5)
    # probes/fin-argument-modes H231 (excel-graph): 3.859866162622655
    assert value(PDURATION, 0.025, 2000, 2200) == pytest.approx(3.859866162622655, rel=1e-15)


def test_excel_values_of_the_semantics_entries() -> None:
    # financial.md "PMT is a discounted payment": PMT(0.05,1,10000) is -10499.999999999998, exact -10500
    assert value(PMT, 0.05, 1, 10000) == -10500
    # "PMT when (1+rate)^nper overflows or underflows": past the overflow PMT is -pv*rate (-pv*rate/(1+rate) for
    # type 1); the fv term vanishes
    assert value(PMT, 100, 184, 439103) == pytest.approx(-43910300, rel=1e-15)
    assert value(PMT, 100, 184, 439103, 0, 1) == pytest.approx(-434755.44554455444, rel=1e-15)
    assert value(PMT, 1, 2000, 1, 5) == pytest.approx(-1, rel=1e-15)
    assert value(IPMT, 100, 1, 184, 439103) == pytest.approx(-43910300, rel=1e-15)
    # "PMT, IPMT and PPMT are the exact payment": PPMT(0.2,1,360,10000) is -6.248581209907205E-26
    assert value(PPMT, 0.2, 1, 360, 10000) == pytest.approx(-6.248581209907205e-26, rel=1e-14)
    # IPMT(0.2,180,360,10000,0,1) is -1666.6666666666588 (A83)
    assert value(IPMT, 0.2, 180, 360, 10000, 0, 1) == pytest.approx(-1666.6666666666588, rel=1e-14)
    # "ISPMT's operation order": ISPMT(0.2,1,3,100) is -13.333333333333334
    assert value(ISPMT, 0.2, 1, 3, 100) == pytest.approx(-13.333333333333334, rel=1e-15)
    # "PV, PMT and IRR near a rate of -1": FV at rate -1 is 421
    assert value(FV, -1, 5, -421) == 421
    # "Annuity and DB argument domains at their limits": PV(-0.5,1022,0,1) (P = 2^-1022, normal) is a value
    assert value(PV, -0.5, 1022, 0, 1) == pytest.approx(-4.49423283715579e307, rel=1e-14)


def test_ipmt_split_matches_the_exact_balance() -> None:
    # "PMT, IPMT and PPMT are the exact payment": with B the balance after e = per-2 periods, type-1 IPMT is
    # -rate*(B + PMT); the split form here must agree, and IPMT + PPMT is PMT.
    for r, per, n, pv, fv in [(0.05, 7, 30, 1000, 250), (0.2, 2, 10, -500, 0), (0.01, 12.5, 36, 8000, 1234)]:
        e = per - 2
        growth = [(1 + mp(r)) ** x for x in (e, n - e, n)]
        balance = (pv * growth[0] * (growth[1] - 1) - fv * (growth[0] - 1)) / (growth[2] - 1)
        payment = PMT.evaluate([mp(r), mp(n), mp(pv), mp(fv), mp(1)])
        ipmt = IPMT.evaluate([mp(r), mp(per), mp(n), mp(pv), mp(fv), mp(1)])
        ppmt = PPMT.evaluate([mp(r), mp(per), mp(n), mp(pv), mp(fv), mp(1)])
        assert mpmath.almosteq(ipmt, -r * (balance + payment), mp("1e-30"))
        assert mpmath.almosteq(ipmt + ppmt, payment, mp("1e-30"))


def test_rate_zero_and_period_one_of_type_one() -> None:
    assert value(FV, 0, 10, -100, -1000) == 2000
    assert value(PV, 0, 10, -100, -1000) == 2000
    assert value(PMT, 0, 10, 1000, 500) == -150
    assert value(IPMT, 0, 3, 10, 1000) == 0
    assert value(PPMT, 0, 3, 10, 1000, 500) == -150
    assert value(IPMT, 0.1, 1, 10, 1000, 0, 1) == 0
    assert value(PPMT, 0.1, 1, 10, 1000, 0, 1) == value(PMT, 0.1, 10, 1000, 0, 1)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (PV, (-1, 0, -713)),  # 0^0: #NUM!
        (PV, (-1, 5, -421)),  # P = 0: #DIV/0!
        (PV, (-0.5, 1023, -1, 1)),  # P below the normal range: #DIV/0!
        (PV, (100, 184, 1)),  # P past the double range: #NUM!
        (PV, (1e300, 2, -421)),
        (FV, (100, 184, 1)),
        (FV, (1e300, 2, -421)),
        (PMT, (-1, 5, -421)),  # rate -1: #NUM!
        (PMT, (-0.5, 1070, 1)),  # P below the normal range: #NUM!
        (PMT, (1, -2000, 1)),
        (IPMT, (0.5, 12, 0, 347880)),  # period past nper: #NUM!
        (IPMT, (1e300, 0.2, 161, 210756, 227)),  # period below 1
        (PPMT, (1, 0.5, 323, 401723)),
        (PPMT, (0.1, 9, 1.5, 404118, 62, 1)),
        (IPMT, (-0.5, 1, 2000, 1)),  # 1/P overflows: #NUM!
        (PPMT, (-0.5, 1, 2000, 1)),
        (PPMT, (-0.5, 1, 2000, 1, 0, 1)),  # PMT's #NUM! in period 1 of type 1
        (PDURATION, (0, 7968, 32308)),  # rate 0: #NUM!
        (PDURATION, (100, 0, 49370)),  # pv 0: #NUM!
        (PDURATION, (1e-300, 4938, 46197)),  # 1 + rate is 1: #DIV/0!
    ],
)
def test_domain_errors_follow_excel(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


def test_unprobed_points_have_no_value() -> None:
    for definition, args in [
        (PMT, (0.1, 0, 1000)),  # nper 0
        (ISPMT, (0.1, 1, 0, 1000)),
        (FV, (-1.5, 3, -1, 1)),  # rate below -1
        (IPMT, (-1, 1, 5, 100)),
        (PDURATION, (1e-300, 4938, 4938)),  # 1 + rate is 1 and fv = pv: 0/0, unprobed
    ]:
        with pytest.raises(NoValueError):
            definition.evaluate([mp(a) for a in args])
    # accuracy/PPMT/s1 A118: PPMT(0.0798622533510215,139.5,139,-115683.067689867,0,0) is 8890.724785738455
    assert value(PPMT, 0.0798622533510215, 139.5, 139, -115683.067689867, 0, 0) == pytest.approx(
        8890.724785738455, rel=1e-14
    )
    # IPMT in period 1 of type 1 is 0 even where 1/P overflows ("PMT when (1+rate)^nper overflows or underflows")
    assert value(IPMT, -0.5, 1, 2000, 1, 0, 1) == 0


def test_ispmt_has_no_period_domain() -> None:
    # gen-m9 d912671baa1364aa A215: ISPMT(1E-10,361,124,371658) is 7.103463387096774E-05
    assert value(ISPMT, 1e-10, 361, 124, 371658) == pytest.approx(7.103463387096774e-05, rel=1e-15)


@pytest.mark.parametrize("definition", [FV, PV, PMT, IPMT, PPMT, ISPMT, PDURATION])
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
