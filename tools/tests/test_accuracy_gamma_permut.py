"""Gate A definitions of GAMMA, GAMMALN, GAMMALN.PRECISE, PERMUT and PERMUTATIONA (T-4106) against published values,
the semantics entries' Excel values and their domains."""

from __future__ import annotations

import contextlib
import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.combinatorics import PERMUT, PERMUTATIONA
from rustsheet_tools.accuracy.defs.gamma_function import (
    GAMMA,
    GAMMA_LAST,
    GAMMA_NEXT,
    GAMMALN,
    GAMMALN_PRECISE,
)
from rustsheet_tools.accuracy.model import Definition, DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


def value(definition: Definition, *args: float) -> mpmath.mpf:
    with mpmath.workdps(50):
        return definition.evaluate([mp(a) for a in args])


def test_published_values() -> None:
    # Abramowitz and Stegun 6.1.8 and 6.1.6: Γ(1/2) = √π and Γ(n+1) = n!
    with mpmath.workdps(40):
        assert mpmath.almosteq(value(GAMMA, 0.5), mpmath.sqrt(mpmath.pi), mp("1e-38"))
        assert mpmath.almosteq(value(GAMMALN, 0.5), mpmath.log(mpmath.sqrt(mpmath.pi)), mp("1e-38"))
    assert value(GAMMA, 5) == 24
    assert value(GAMMALN, 1) == 0
    assert value(GAMMALN, 2) == 0


@pytest.mark.parametrize(
    ("definition", "args", "excel", "rel"),
    [
        # statistics.md "GAMMA is exact at small positive integers ...", "GAMMA and GAMMALN sweep (T-1042)".
        (GAMMA, (6,), 120.0, 0),
        (GAMMA, (26,), 1.5511210043330984e25, 2e-16),
        (GAMMA, (-0.5,), -3.5449077018110318, 2e-16),
        (GAMMA, (-2.25,), -1.742814865728253, 1e-15),
        (GAMMA, (-9.9,), 3.542684553080843e-06, 1e-15),
        (GAMMA, (1e-10,), 9999999999.422787, 1e-15),
        (GAMMA, (-1e-10,), -10000000000.577215, 1e-15),
        (GAMMA, (1e-300,), 9.999999999999999e299, 1e-15),
        (GAMMA, (171.5,), 9.483367566824801e307, 1e-14),
        # statistics.md "GAMMA is Excel's own x87 routine": the last finite argument, about 17 ulps from the math.
        (GAMMA, (GAMMA_LAST,), 1.7976931348622233e308, 1e-14),
        (GAMMA, (-171.0000000001,), 8.058901809475927e-300, 1e-12),
        (GAMMALN, (3,), 0.6931471805599454, 2e-16),
        (GAMMALN, (200,), 857.9336698258575, 1e-15),
        (GAMMALN, (1e300,), 6.897755278982137e302, 1e-15),
        (GAMMALN, (1e-300,), 690.7755278982137, 1e-15),
        (GAMMALN_PRECISE, (1.5,), -0.12078223763524523, 1e-15),
        # statistics.md T-1290: PERMUT(170,170) = 170!.
        (PERMUT, (170, 170), 7.257415615307994e306, 1e-15),
        (PERMUT, (2147483646, 1), 2147483646.0, 0),
        (PERMUT, (2147483646.5, 1), 2147483646.0, 0),
        (PERMUT, (2147483646, 0), 1.0, 0),
        # statistics.md T-2256, T-2621: number_chosen truncates toward zero, 0^0 is 1.
        (PERMUTATIONA, (10, -0.75), 1.0, 0),
        (PERMUTATIONA, (7, -0.1), 1.0, 0),
        (PERMUTATIONA, (0, -0.0001), 1.0, 0),
        (PERMUTATIONA, (123456789.123, 1), 123456789.0, 0),
        (PERMUTATIONA, (3, 4), 81.0, 0),
    ],
)
def test_excel_values(definition: Definition, args: tuple[float, ...], excel: float, rel: float) -> None:
    got = value(definition, *args)
    assert abs(got - mp(excel)) <= rel * abs(mp(excel))


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        # statistics.md "GAMMA and GAMMALN sweep (T-1042)": the poles, the overflow, GAMMALN of x <= 0.
        (GAMMA, (0,)),
        (GAMMA, (-1,)),
        (GAMMA, (-10,)),
        (GAMMA, (GAMMA_NEXT,)),
        (GAMMA, (171.7,)),
        (GAMMA, (200,)),
        (GAMMALN, (0,)),
        (GAMMALN, (-1e-300,)),
        (GAMMALN_PRECISE, (-0.5,)),
        # statistics.md T-1290: PERMUT's limit and overflow.
        (PERMUT, (2147483647, 1)),
        (PERMUT, (1e10, 1)),
        (PERMUT, (1e30, 0)),
        (PERMUT, (171, 171)),
        (PERMUT, (200, 180)),
        (PERMUT, (5, 6)),
        (PERMUT, (-1, 0)),
        # statistics.md T-2256: PERMUTATIONA's huge number.
        (PERMUTATIONA, (1e15, 0)),
        (PERMUTATIONA, (6402373705728000, 2)),
        (PERMUTATIONA, (5, -1)),
        (PERMUTATIONA, (2, 1024)),
    ],
)
def test_domain_errors(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        value(definition, *args)


def test_gamma_reflection_underflow_is_a_value() -> None:
    # Excel gives 0 below about -171.6 (`GAMMA(-200.3)`); the math is far below the envelope's 1E-300 floor.
    assert 0 < abs(value(GAMMA, -200.3)) < 1e-300


def test_permutationa_at_a_huge_number_chosen() -> None:
    # statistics.md "PERMUTATIONA is binary powering in doubles, and #NUM! at a huge number_chosen (T-4106)":
    # PERMUTATIONA(1,2147483647) and (0,1E10) are #NUM! (accuracy/PERMUTATIONA/s1 A76, A71); (0,1000) is 0 (A72).
    for args in ((1, 2147483647), (1, 1e10), (0, 1e10)):
        with pytest.raises(DomainError):
            value(PERMUTATIONA, *args)
    assert value(PERMUTATIONA, 0, 1000) == 0
    assert value(PERMUTATIONA, 1, 2147483646) == 1
    assert value(PERMUTATIONA, 2, 1023) == mp(2) ** 1023


@pytest.mark.parametrize("definition", [GAMMA, PERMUT, PERMUTATIONA])
def test_draws_evaluate_or_raise_domain_errors(definition: Definition) -> None:
    assert definition.draw is not None
    rng = random.Random(4106)
    for stratum in STRATA:
        for _ in range(200):
            args = definition.draw(stratum, rng)
            with contextlib.suppress(DomainError):
                value(definition, *(float(a) for a in args if not isinstance(a, (bool, tuple))))
