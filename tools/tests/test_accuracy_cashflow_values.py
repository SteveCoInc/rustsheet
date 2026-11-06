"""Gate A definitions of NPV, XNPV, MIRR and FVSCHEDULE (T-4116) against Microsoft's published examples, the
semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import contextlib
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.cashflow_values import FVSCHEDULE, MIRR, NPV, XNPV
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, math_arg

mp = mpmath.mpf
EXAMPLE_FLOWS = (-10000, 2750, 4250, 3250, 2750)
EXAMPLE_DATES = (39448, 39508, 39751, 39859, 39904)  # 2008-01-01, 03-01, 10-30, 2009-02-15, 04-01


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def row(*xs: float) -> tuple[mpmath.mpf, ...]:
    return tuple(mp(x) for x in xs)


def npv(rate: float, *flows: float) -> float:
    return float(NPV.evaluate([mp(rate), row(*flows)]))


def xnpv(rate: float, flows: tuple[float, ...], dates: tuple[float, ...]) -> float:
    return float(XNPV.evaluate([mp(rate), row(*flows), row(*dates)]))


def mirr(flows: tuple[float, ...], fr: float, rr: float) -> float:
    return float(MIRR.evaluate([row(*flows), mp(fr), mp(rr)]))


def fvschedule(principal: float, *schedule: float) -> float:
    return float(FVSCHEDULE.evaluate([mp(principal), row(*schedule)]))


def test_microsofts_examples() -> None:
    # NPV(0.1,-10000,3000,4200,6800) $1,188.44; NPV(0.08,8000,9200,10000,12000,14500)-40000 $1,922.06
    assert npv(0.1, -10000, 3000, 4200, 6800) == pytest.approx(1188.44, abs=0.005)
    assert npv(0.08, 8000, 9200, 10000, 12000, 14500) - 40000 == pytest.approx(1922.06, abs=0.005)
    # XNPV(0.09,...) 2086.65
    assert xnpv(0.09, EXAMPLE_FLOWS, EXAMPLE_DATES) == pytest.approx(2086.65, abs=0.005)
    # MIRR: 12.61% over five years, -4.80% over three, 13.48% at a reinvest rate of 14%
    flows = (-120000, 39000, 30000, 21000, 37000, 46000)
    assert mirr(flows, 0.1, 0.12) == pytest.approx(0.1261, abs=5e-5)
    assert mirr(flows[:4], 0.1, 0.12) == pytest.approx(-0.0480, abs=5e-5)
    assert mirr(flows, 0.1, 0.14) == pytest.approx(0.1348, abs=5e-5)
    # FVSCHEDULE(1,{0.09,0.11,0.1}) 1.33089
    assert fvschedule(1, 0.09, 0.11, 0.1) == pytest.approx(1.33089, rel=1e-15)


def test_excel_values_of_the_semantics_entries() -> None:
    # "NPV and FVSCHEDULE arguments": NPV(0.1,{-10000,2750,4250,3250,2750}) is 302.233453999042
    assert npv(0.1, *EXAMPLE_FLOWS) == pytest.approx(302.233453999042, rel=1e-14)
    # "NPV's discount past the double range": the last power is never formed; a rate below -1 is a value
    assert npv(1e300, 1, 1e300) == pytest.approx(2e-300, rel=1e-14)
    assert npv(1e154, 1, 1, 1) == pytest.approx(1e-154, rel=1e-14)
    assert npv(-2, 1, 1) == 0
    assert npv(-3, 1, 1, 1) == -0.375
    assert npv(1, *range(1, 1025)) == pytest.approx(1.9999999999999998, rel=1e-15)
    assert npv(-0.5, *([0] * 1023)) == 0
    # "MIRR error and edge cases" (probes/fin-mirr-edges)
    assert mirr((-1, -2, -3), 0.1, 0.1) == -1  # A2
    assert mirr((-1, -2, -3), 0.1, -2) == -1  # A27
    assert mirr((-100, 50, 80), -1, 0.1) == pytest.approx(0.1618950038622251, rel=1e-15)  # A6
    assert mirr((-100, 50, 80), -2, 0.1) == pytest.approx(0.1618950038622251, rel=1e-15)  # A9
    assert mirr((-100, 50, 80), 0.1, -2) == pytest.approx(-0.4522774424948339, rel=1e-15)  # A10
    assert mirr((-100, 50, 80), -1.5, -1.5) == pytest.approx(-0.2583801512904337, rel=1e-15)  # A11
    assert mirr((-100, 50, 80), 0.1, 50) == pytest.approx(4.128352561983234, rel=1e-15)  # A18
    assert mirr((-50, 100), -1, 0.1) == 1  # A22
    # "MIRR's evaluation order": MIRR({-100,120},fr,0.03) is 0.19999999999999973, exactly 0.2 in the math
    assert mirr((-100, 120), 0.07, 0.03) == pytest.approx(0.2, rel=1e-15)
    # "FVSCHEDULE with one empty schedule cell is the principal" (an empty schedule is the product's 1)
    assert fvschedule(6527, 0) == 6527


def test_xnpv_truncates_its_dates_and_takes_equal_and_unsorted_later_dates() -> None:
    # "XNPV reads its values and dates element by element (T-3815)", rows 1-3, 10 and 12
    whole = xnpv(0.1, (-100, 30, 80), (39448, 39600, 39700))
    assert xnpv(0.1, (-100, 30, 80), (39448.5, 39600.7, 39700.2)) == whole
    assert xnpv(0.1, (-100, 30), (39448.7, 39448.2)) == -70
    assert xnpv(0.1, (-100, 80, 30), (39448, 39700, 39600)) == whole
    assert xnpv(0.1, (30,), (39448,)) == 30


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (NPV, (-1, (1, 2))),  # 1 + rate is 0: #DIV/0!
        (NPV, (-1, (0,))),
        (NPV, (1e200, (1, 1, 1))),  # (1 + rate)^2 past the double range: #DIV/0!
        (NPV, (1e300, (0, 0, 0))),
        (NPV, (9e307, (1, 1, 1))),
        (NPV, (1, tuple(range(1, 1026)))),  # 2^1024
        (XNPV, (0, (-100, 30), (39448, 39600))),  # a rate at or below 0: #NUM!
        (XNPV, (-0.5, (-100, 30), (39448, 39600))),
        (XNPV, (-1, (-100, 30), (39448, 39600))),
        (XNPV, (0.1, (-100, 30, 40), (39448, 39600))),  # sizes differ: #NUM!
        (XNPV, (0.1, (-100, 30), (39448, 39400))),  # a date before the first: #NUM!
        (XNPV, (0.1, (-100, 30), (39448, 0))),  # date 0
        (XNPV, (0.1, (-100, 30), (39448, -1))),
        (XNPV, (0.1, (-100, 30), (39448, 2958466))),
        (XNPV, (0.1, (-100, 30), (39448, 2958465.5))),  # bounded before it is truncated
        (MIRR, ((1, 2, 3), 0.1, 0.1)),  # no negative flow: #DIV/0! (A1)
        (MIRR, ((0, 0), 0.1, 0.1)),  # A5
        (MIRR, ((-100,), 0.1, 0.1)),  # one flow (A3)
        (MIRR, ((-100, 50, 80), 0.1, -1)),  # reinvest -1 (A7)
        (MIRR, ((-1, -2, -3), 0.1, -1)),  # A25
        (MIRR, ((50, -100, 80), -1, 0.1)),  # finance -1 and a later negative flow (A12)
        (MIRR, ((-1, -2, -3), -1, 0.1)),  # A30
        (MIRR, ((50, -100, 80), -2, 0.1)),  # a negative ratio under the square root: #NUM! (A23)
        (MIRR, ((-100, 50, 80), 1e200, 0.1)),  # power overflow (A15, A16, A21, A24, A28, A29)
        (MIRR, ((-100, 50, 80), 0.1, 1e200)),
        (MIRR, ((-100, 50, 80), -1e200, 0.1)),
        (MIRR, ((-1, -2, -3), 0.1, 1e200)),
    ],
)
def test_domain_errors_follow_excel(definition: Definition, args: tuple[object, ...]) -> None:
    math: list[MathArg] = [row(*a) if isinstance(a, tuple) else mp(a) for a in args]  # type: ignore[arg-type]
    with pytest.raises(DomainError):
        definition.evaluate(math)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (NPV, (-0.5, (0,) * 1050)),  # a subnormal power: unprobed between 2^-1073 and 2^-1022
        (XNPV, (0.1, (-100, 30), (0.5, 39448))),  # a date that truncates to 0
        (XNPV, (1e10, (-100, 30), (39448, 60000))),  # a discount past the double range
        (MIRR, ((-100, 50, 80, 0), -1, 0.1)),  # finance -1 with a later zero flow
        (MIRR, ((50, -100, 80, 20), -2, 0.1)),  # a negative ratio under a cube root
        (FVSCHEDULE, (1e300, (1e10,))),  # a product past the double range (OPEN, T-4250)
        (FVSCHEDULE, (5916, (-1e300, 1e300))),
    ],
)
def test_unprobed_points_have_no_value(definition: Definition, args: tuple[object, ...]) -> None:
    math: list[MathArg] = [row(*a) if isinstance(a, tuple) else mp(a) for a in args]  # type: ignore[arg-type]
    with pytest.raises(NoValueError):
        definition.evaluate(math)


def test_fvschedule_has_no_domain_on_its_rates() -> None:
    assert fvschedule(1000, 0.1, -1, 0.2) == 0
    assert fvschedule(100, -2, -3) == 200
    assert fvschedule(-100, 0.5) == -150


@pytest.mark.parametrize("definition", [NPV, XNPV, MIRR, FVSCHEDULE])
def test_draws_are_reproducible_and_evaluate(definition: Definition) -> None:
    assert definition.draw is not None
    for stratum in STRATA:
        a = definition.draw(stratum, random.Random(7))
        assert a == definition.draw(stratum, random.Random(7))
        rng = random.Random(11)
        for _ in range(40):
            args = definition.draw(stratum, rng)
            params = definition.params_for(len(args))
            with contextlib.suppress(DomainError, NoValueError):
                definition.evaluate([math_arg(p, x) for p, x in zip(params, args, strict=True)])
