"""Gate A definitions of CUMIPMT, CUMPRINC, EFFECT, NOMINAL, RRI, DOLLARDE and DOLLARFR (T-4110) against Microsoft's
published examples, the semantics entries' Excel values and their domains."""

from __future__ import annotations

import math
import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.annuity_cumulative import CUMIPMT, CUMPRINC
from rustsheet_tools.accuracy.defs.dollar_fractions import DOLLARDE, DOLLARFR
from rustsheet_tools.accuracy.defs.interest_rates import EFFECT, NOMINAL, RRI
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import AIMED, STRATA, Sampler, math_arg

mp = mpmath.mpf
ALL = (CUMIPMT, CUMPRINC, EFFECT, NOMINAL, RRI, DOLLARDE, DOLLARFR)


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(definition: Definition, *args: float) -> float:
    return float(definition.evaluate([mp(a) for a in args]))


def test_microsoft_examples() -> None:
    # Microsoft's CUMIPMT, CUMPRINC, EFFECT, NOMINAL, RRI, DOLLARDE and DOLLARFR pages
    rate = 0.09 / 12
    assert value(CUMIPMT, rate, 360, 125000, 13, 24, 0) == pytest.approx(-11135.23213, abs=1e-5)
    assert value(CUMIPMT, rate, 360, 125000, 1, 1, 0) == pytest.approx(-937.5, rel=1e-15)
    assert value(CUMPRINC, rate, 360, 125000, 13, 24, 0) == pytest.approx(-934.1071234, abs=1e-7)
    assert value(CUMPRINC, rate, 360, 125000, 1, 1, 0) == pytest.approx(-68.27827118, abs=1e-8)
    assert value(EFFECT, 0.0525, 4) == pytest.approx(0.053542667, abs=1e-9)
    assert value(NOMINAL, 0.053543, 4) == pytest.approx(0.05250032, abs=1e-8)
    assert value(RRI, 96, 10000, 11000) == pytest.approx(0.0009933, abs=1e-7)
    assert value(DOLLARDE, 1.02, 16) == 1.125
    assert value(DOLLARDE, 1.1, 32) == pytest.approx(1.3125, rel=1e-15)  # 1.1 is a double above 1.1
    assert value(DOLLARFR, 1.125, 16) == pytest.approx(1.02, rel=1e-15)
    assert value(DOLLARFR, 1.125, 32) == pytest.approx(1.04, rel=1e-15)


def test_excel_values_of_the_semantics_entries() -> None:
    # financial.md "CUMIPMT and CUMPRINC over many periods" (probes/fin-cumipmt-period-bound, -overflow-and-forms)
    assert value(CUMIPMT, 1e-9, 1e9, 1, 1, 1e9, 0) == pytest.approx(-0.5819767073296636, rel=1e-14)
    assert value(CUMPRINC, 1e-9, 1e9, 1, 1, 1e9, 0) == pytest.approx(-1, rel=1e-15)
    assert value(CUMIPMT, 0.01, 1e6, 100000, 1, 1e6, 0) == pytest.approx(-999900000, rel=1e-15)
    assert value(CUMIPMT, 0.01, 100, 100000, 100, 100, 0) == pytest.approx(-15.708656559792416, rel=1e-15)
    assert value(CUMPRINC, 1e10, 100, 1, 1, 100, 0) == pytest.approx(-1, rel=1e-15)
    assert value(CUMIPMT, 1e10, 100, 1, 1, 100, 0) == pytest.approx(-999999999999, rel=1e-15)
    # the start rounds up and the end down: periods 2 to 1999 (A23)
    assert value(CUMIPMT, 0.001, 2000, 1000, 1.5, 1999.7, 0) == value(CUMIPMT, 0.001, 2000, 1000, 2, 1999, 0)
    assert value(CUMIPMT, 0.001, 2000, 1000, 2, 1999, 0) == pytest.approx(-1312.3961571024322, rel=1e-14)
    # "CUMIPMT of one period is not that period's IPMT": ulps from the exact value, type 1 included
    assert value(CUMIPMT, 0.1, 10, 345, 2, 2, 1) == pytest.approx(-29.3957126150485, rel=1e-14)
    assert value(CUMIPMT, 0.0083, 36, 8000, 3, 3, 0) == pytest.approx(-63.2064526131625, rel=1e-14)
    # "EFFECT and NOMINAL raise to their power with `^`"
    assert value(EFFECT, 0.5, 10) == pytest.approx(0.6288946267774416, rel=1e-15)
    assert value(NOMINAL, 0.5, 4) == pytest.approx(0.4267276788012868, rel=1e-15)
    assert value(EFFECT, 270, 30) == pytest.approx(1e30, rel=1e-15)
    assert value(NOMINAL, 1e300, 1) == pytest.approx(1e300, rel=1e-15)
    # "DURATION and RRI operation order"
    assert value(RRI, 1, 826, 40633) == pytest.approx(48.19249394673123, rel=1e-15)
    assert value(RRI, 10, -100, -200) == pytest.approx(0.07177346253629313, rel=1e-15)
    assert value(RRI, 0.5, 3, 4) == pytest.approx(0.7777777777777775, rel=1e-15)
    assert value(RRI, 10, 100, 0) == -1


def test_type_one_sums_the_whole_term_to_pv() -> None:
    # every principal of a loan pays it back: CUMPRINC over the whole term is -pv, type 1 too; each period's
    # interest and principal add to the payment, the last principal times 1 + rate
    for kind in (0, 1):
        assert value(CUMPRINC, 0.07, 36, 8000, 1, 36, kind) == pytest.approx(-8000, rel=1e-30)
        principal = value(CUMPRINC, 0.07, 36, 8000, 1, 1, kind)
        interest = value(CUMIPMT, 0.07, 36, 8000, 1, 1, kind)
        assert principal + interest == pytest.approx(
            value(CUMPRINC, 0.07, 36, 8000, 36, 36, kind) * 1.07, rel=1e-12
        )
    assert value(CUMIPMT, 0.07, 36, 8000, 1, 1, 1) == 0


def test_tiny_rate_interest_does_not_cancel() -> None:
    # the interest of a 1E-300 rate is about pv·rate per period, not lost to the payments' cancellation
    interest = value(CUMIPMT, 1e-300, 83, 156759, 5, 17, 0)
    assert interest == pytest.approx(-1.7923408554216868e-294, rel=1e-12)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (CUMIPMT, (0, 310, 54445, 6, 22, 0)),  # gen-m9 9d9ceba257f8001b A382
        (CUMPRINC, (0.05, 249, 329832, 0.3, 19, 0)),  # gen-m9 0a957119475760c5 A40
        (CUMIPMT, (0.01, 100.5, 1000, 1, 101, 0)),  # probes/fin-cumipmt-period-bound A21
        (CUMIPMT, (0.05, 272, 417600, 7, 2, 0)),  # gen-m9 67a4c03afd63f988 A271
        (EFFECT, (0, 4)),
        (EFFECT, (0.05, 0)),  # npery 0.5 floors to 0
        (EFFECT, (1e300, 2)),
        (EFFECT, (1e13, 365)),
        (NOMINAL, (0, 3)),
        (RRI, (0, 100, 200)),
        (RRI, (-5, 100, 200)),
        (RRI, (10, 0, 200)),
        (RRI, (1, -100, 200)),
        (RRI, (1e-5, 1, 2)),
    ],
)
def test_evidenced_domains(definition: Definition, args: Sequence[float]) -> None:
    with pytest.raises(DomainError):
        definition.evaluate([mp(a) for a in args])


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (CUMIPMT, (-0.5, 2000, 1, 1, 1, 0)),
        (CUMPRINC, (0.05, 36, -8000, 1, 1, 0)),
        (CUMPRINC, (0.05, 36, 8000, 1, 1, 2)),
        (CUMPRINC, (0.05, 36, 8000, 1.5, 1.7, 0)),
        (EFFECT, (-0.05, 4)),
        (NOMINAL, (0.05, -4)),
        (RRI, (10, 0, 0)),
        (DOLLARDE, (1.02, 0)),
        (DOLLARFR, (1.02, -16)),
    ],
)
def test_unprobed_regions_have_no_value(definition: Definition, args: Sequence[float]) -> None:
    with pytest.raises(NoValueError):
        definition.evaluate([mp(a) for a in args])


def test_dollar_fractions() -> None:
    # gen-m9 cells: D is the power of ten at or above the fraction
    assert value(DOLLARDE, 2.5, 13) == pytest.approx(5.846153846153847, rel=1e-15)
    assert value(DOLLARDE, 7, 10) == 7
    assert value(DOLLARFR, 0.2, 12) == pytest.approx(0.024000000000000004, rel=1e-15)
    assert value(DOLLARDE, 123456789.123, 2) == pytest.approx(123456789.61499998, rel=1e-15)
    assert value(DOLLARDE, -1.02, 16) == -1.125
    assert value(DOLLARFR, 3.7, 1) == pytest.approx(3.7, rel=1e-15)


@pytest.mark.parametrize("definition", ALL, ids=lambda d: d.function)
def test_draws_evaluate(definition: Definition) -> None:
    # every stratum's draws evaluate (a domain error is an evidenced `#NUM!` point), most of them to a value
    sampler = Sampler(definition, random.Random(4110))
    values = 0
    for stratum in STRATA:
        for index in range(40):
            args = sampler.draw(stratum, index)
            math_args = [math_arg(p, a) for p, a in zip(definition.params_for(len(args)), args, strict=True)]
            try:
                definition.evaluate(math_args)
            except (DomainError, NoValueError):
                continue
            values += 1
    assert values > 150


def test_the_aimed_draw_reaches_the_underflowed_short_ranges() -> None:
    # T-4059: `--stratum aimed` draws T-3141's region: at most 32 periods, 1 - v^nper is 1 in doubles, the leading
    # power v^(nper-end+1) normal, type 1 from period 2, and leading exponents on both sides of the engine's 0.3 cut
    for definition in (CUMIPMT, CUMPRINC):
        sampler = Sampler(definition, random.Random(4059))
        leads = []
        for i in range(200):
            r, nper, _pv, start, end, kind = (float(str(a)) for a in sampler.draw(AIMED, i))
            u = math.log1p(r)
            leads.append((nper - end + 1) * u)
            assert end - start < 32 and start >= 1 + kind and end <= nper <= 1e15
            assert math.exp(-nper * u) < 2.0**-53 and math.exp(-leads[-1]) > 2.2e-308
        assert min(leads) < 0.3 < max(leads)


def test_strata_without_an_aimed_draw_refuse_it() -> None:
    with pytest.raises(ValueError, match="no aimed draw"):
        Sampler(EFFECT, random.Random(1)).draw(AIMED, 0)
