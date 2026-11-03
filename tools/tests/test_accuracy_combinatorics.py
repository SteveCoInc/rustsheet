"""Gate A definitions of FACT, FACTDOUBLE, COMBIN, COMBINA, MULTINOMIAL, QUOTIENT and MOD (T-3675,
decision 0228) against the values numbers.md cites."""

from __future__ import annotations

import math
import random

import mpmath
import pytest

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.defs.combinatorics import COMBIN, COMBINA, FACT, FACTDOUBLE, MULTINOMIAL
from rustsheet_tools.accuracy.defs.remainder import MOD, QUOTIENT
from rustsheet_tools.accuracy.model import Definition, DomainError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler

mp = mpmath.mpf


def value(definition: Definition, *args: float) -> mpmath.mpf:
    with mpmath.workdps(50):
        return definition.evaluate([mp(a) for a in args])


def near(got: mpmath.mpf, excel: float) -> bool:
    """Excel's inexact value is within 1E-12 relative of the exact math (the envelope judges the rest)."""
    return bool(abs(got - mp(excel)) <= 1e-12 * abs(mp(excel)))


@pytest.mark.parametrize(
    ("definition", "args", "excel"),
    [
        # numbers.md "FACT and FACTDOUBLE of a huge argument": floored, descending product.
        (FACT, (170,), 7.257415615308004e306),
        (FACT, (170.9,), 7.257415615308004e306),
        (FACT, (42,), 1.4050061177528801e51),
        (FACT, (0.5,), 1.0),
        (FACTDOUBLE, (297,), 1.2552756225993057e304),
        (FACTDOUBLE, (300,), 8.154414069380598e307),
        (FACTDOUBLE, (-1,), 1.0),
        # numbers.md "COMBIN is inexact", "COMBINA is COMBIN(n+k-1, k)".
        (COMBIN, (10, 4), 209.99999999999997),
        (COMBIN, (40, 20), 137846528819.99997),
        (COMBIN, (170, 85), 9.144841845131565e49),
        (COMBIN, (1029, 514), 1.4298206864989032e308),
        (COMBIN, (2**31 - 2, 2), 2.3058430038449848e18),
        (COMBINA, (20, 19), 35345263799.999985),
        (COMBINA, (1000, 200), 1.6676833067813678e233),
        (COMBINA, (0, 0), 1.0),
        (COMBINA, (-0.5, -0.0001), 1.0),
        (COMBINA, (2**31 - 2, 1), 2147483646.0),
        # numbers.md "MULTINOMIAL is inexact", "MULTINOMIAL of a huge argument".
        (MULTINOMIAL, (2, 3), 9.999999999999998),
        (MULTINOMIAL, (2.5, 3), 9.999999999999998),
        (MULTINOMIAL, (169, 1), 170.0000000000337),
        (MULTINOMIAL, (85, 85), 9.144841845133712e49),
        (MULTINOMIAL, (0, 0), 1.0),
        # numbers.md "MOD is an exact remainder", "... is #NUM! for a huge quotient", "... is 0".
        (MOD, (10.0000000001, 0.1), 9.999945316252479e-11),
        (MOD, (10.0000000001, -0.1), -0.09999999990000055),
        (MOD, (5, 1e-10), 9.999981783901343e-11),
        (MOD, (1125899999999, 1), 0.0),
        (MOD, (1, -(2.0**53)), -9007199254740991.0),
        (MOD, (1, -(2.0**54)), 0.0),
        (MOD, (1, 1e-12), 2.01e-17),
        (MOD, (2251799999999, 2), 1.0),
        # QUOTIENT on the double quotient: 1/0.1 is the double 10.
        (QUOTIENT, (7, 2), 3.0),
        (QUOTIENT, (-7, 2), -3.0),
        (QUOTIENT, (1, 0.1), 10.0),
    ],
)
def test_values(definition: Definition, args: tuple[float, ...], excel: float) -> None:
    got = value(definition, *args)
    if excel == 0:
        assert got == 0
    elif definition is MOD and args == (1, 1e-12):
        assert abs(got - mp(excel)) < 1e-18  # Excel's 15-digit display of the exact remainder
    else:
        assert near(got, excel)


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (FACT, (171,)),
        (FACT, (-0.5,)),
        (FACT, (1e300,)),
        (FACTDOUBLE, (301,)),
        (FACTDOUBLE, (-1.5,)),
        (FACTDOUBLE, (-2,)),
        (COMBIN, (1030, 514)),
        (COMBIN, (2**31 - 1, 0)),
        (COMBIN, (3, 4)),
        (COMBIN, (-0.5, 0)),
        (COMBINA, (2**31 - 2, 2)),
        (COMBINA, (2**31 - 1, 1)),
        (COMBINA, (0, 1)),
        (COMBINA, (7, 1 - 1.005)),
        (COMBINA, (2, 1e300)),
        (MULTINOMIAL, (171,)),
        (MULTINOMIAL, (100, 100)),
        (MULTINOMIAL, (-1, 2, 3)),
        (MULTINOMIAL, (-0.5, 3)),
        (MOD, (1125900000000, 1)),
        (MOD, (1.2, 1e-12)),
        (MOD, (562950000000.5, 0.5)),
        (MOD, (1e-300 * (1 + 2**-52), 1e-300)),
        (MOD, (1, 0)),
        (QUOTIENT, (1, 0)),
    ],
)
def test_errors(definition: Definition, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        value(definition, *args)


@pytest.mark.parametrize("definition", [FACT, FACTDOUBLE, COMBIN, COMBINA, MULTINOMIAL, QUOTIENT, MOD])
def test_draws_are_writable_literals(definition: Definition) -> None:
    sampler = Sampler(definition, random.Random(11))
    for stratum in STRATA:
        for i in range(60):
            for a in sampler.draw(stratum, i):
                f = float(a)
                assert math.isfinite(f)
                assert f == 0 or literal.MIN_LITERAL <= abs(f) <= literal.MAX_LITERAL
                assert float(literal.text(a)) == f
