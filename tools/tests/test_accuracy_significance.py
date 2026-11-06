"""Gate A definitions of CHISQ.TEST, CHITEST, Z.TEST, ZTEST and PROB (T-4098) against Microsoft's published examples,
the semantics entries' Excel values and Excel's evidenced errors (probes-numeric/t1049-chisqtest,
probes/t4098-tests-prob-domains)."""

from __future__ import annotations

import contextlib
import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.significance import (
    CHISQ_TEST,
    CHITEST,
    MICROSOFT_ACTUAL,
    MICROSOFT_EXPECTED,
    MICROSOFT_PROB_P,
    MICROSOFT_PROB_X,
    MICROSOFT_Z_DATA,
    PROB,
    Z_TEST,
    ZTEST,
    degrees_of_freedom,
)
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, math_arg

mp = mpmath.mpf
Table = Sequence[Sequence[float]]


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def matrix(rows: Table) -> tuple[tuple[mpmath.mpf, ...], ...]:
    return tuple(tuple(mp(x) for x in row) for row in rows)


def array(values: Sequence[float]) -> tuple[mpmath.mpf, ...]:
    return tuple(mp(x) for x in values)


def chisq(actual: Table, expected: Table, definition: Definition = CHISQ_TEST) -> mpmath.mpf:
    return definition.evaluate([matrix(actual), matrix(expected)])


def z_test(data: Sequence[float], *scalars: float, definition: Definition = Z_TEST) -> mpmath.mpf:
    return definition.evaluate([array(data), *(mp(s) for s in scalars)])


def prob(xs: Sequence[float], ps: Sequence[float], *limits: float) -> mpmath.mpf:
    return PROB.evaluate([array(xs), array(ps), *(mp(v) for v in limits)])


def test_microsofts_examples() -> None:
    for definition in (CHISQ_TEST, CHITEST):
        value = chisq(MICROSOFT_ACTUAL, MICROSOFT_EXPECTED, definition)
        assert float(value) == pytest.approx(0.0003082, abs=5e-8)
    for definition in (Z_TEST, ZTEST):
        assert float(z_test(MICROSOFT_Z_DATA, 4, definition=definition)) == pytest.approx(0.090574, abs=5e-7)
        assert float(z_test(MICROSOFT_Z_DATA, 6, definition=definition)) == pytest.approx(0.863043, abs=5e-7)
    assert prob(MICROSOFT_PROB_X, MICROSOFT_PROB_P, 2) == pytest.approx(0.1, rel=1e-15)
    assert prob(MICROSOFT_PROB_X, MICROSOFT_PROB_P, 1, 3) == pytest.approx(0.8, rel=1e-15)


def test_chisq_excel_values() -> None:
    # probes-numeric/t1049-chisqtest A29:A31: one row, a column against a row (df 2) and a 2x2 table (df 1)
    assert chisq([[10, 20, 30]], [[15, 15, 30]]) == pytest.approx(0.18887560283756183, rel=1e-15)
    assert chisq([[10], [20], [30]], [[15, 15, 30]]) == pytest.approx(0.18887560283756183, rel=1e-15)
    assert chisq([[10, 20], [30, 40]], [[15, 15], [30, 40]]) == pytest.approx(0.06788915486182905, rel=1e-15)
    # chi2 = 18 at df 2 is EXP(-9), as Excel's CHITEST gives it (m9-legacy-chitest A23)
    assert chisq([[5, 0, 0]], [[1, 1, 1]], CHITEST) == pytest.approx(float(mpmath.exp(-9)), rel=1e-15)
    assert chisq([[3, 4]], [[3, 4]]) == 1
    # a negative actual is a number (c6cfda6bd10cae93 A375: -1 against 1 beside equal cells, exp(-2))
    assert chisq([[-1, 1, 1]], [[1, 1, 1]]) == pytest.approx(float(mpmath.exp(-2)), rel=1e-15)


def test_degrees_of_freedom() -> None:
    assert degrees_of_freedom(1, 5) == 4
    assert degrees_of_freedom(5, 1) == 4
    assert degrees_of_freedom(3, 2) == 2
    assert degrees_of_freedom(4, 4) == 9


@pytest.mark.parametrize(
    ("actual", "expected", "code"),
    [
        ([[1, 2, 3]], [[1, 2]], "#N/A"),  # t1049-chisqtest A16:A18
        ([[5]], [[4]], "#N/A"),  # t4098 A42, A46
        ([[10, 20, 30]], [[15, 0, 30]], "#DIV/0!"),  # t1049 A33
        ([[0, 1, 2]], [[0, 1, 3]], "#DIV/0!"),  # t4098 A43
        ([[10, 20, 30]], [[15, 15, -30]], "#NUM!"),  # t1049 A32
        ([[1e200, 1, 1]], [[1, 1, 1]], "#NUM!"),  # t4098 A44
        ([[1e160, 1, 1]], [[1, 1, 1]], "#NUM!"),  # t4098 A45
    ],
)
def test_chisq_domain(actual: Table, expected: Table, code: str) -> None:
    for definition in (CHISQ_TEST, CHITEST):
        with pytest.raises(DomainError, match=code):
            chisq(actual, expected, definition)


def test_chisq_unprobed() -> None:
    with pytest.raises(NoValueError):  # a table against a column of the same count (T-2650)
        chisq([[1, 2], [3, 4]], [[2], [2], [2], [2]])
    with pytest.raises(NoValueError):  # finite squares whose quotients overflow
        chisq([[1e154, 1]], [[1e-300, 1]])


def test_z_test_excel_values() -> None:
    # t4098 A55: constant data with sigma 1 is a number; A47: the squares overflow, the deviations do not
    assert z_test([3, 3, 3], 2, 1) == pytest.approx(0.041632258331775196, rel=1e-14)
    assert z_test([1e160, 1.00000000000001e160, 1.00000000000002e160], 1e160) == pytest.approx(
        0.041632258331775196, rel=1e-6
    )
    assert z_test([1e200, 1e200, -1e200], 0, 1) == 0  # A52: past the flush


@pytest.mark.parametrize(
    ("data", "scalars", "code"),
    [
        ([5], (4, 1), "#DIV/0!"),  # t4098 A53
        ([5], (4,), "#DIV/0!"),
        ([3, 3, 3], (2,), "#DIV/0!"),  # A54
        ([1, 2, 3], (2, -1), "#NUM!"),  # A56
        ([1, 2, 3, 4], (2, 0), "#NUM!"),
        ([1e200, 1e200, -1e200], (0,), "#NUM!"),  # A51
        ([1, 2, 3], (-1e300, 1e-300), "#NUM!"),  # A48
    ],
)
def test_z_test_domain(data: Sequence[float], scalars: Sequence[float], code: str) -> None:
    for definition in (Z_TEST, ZTEST):
        with pytest.raises(DomainError, match=code):
            z_test(data, *scalars, definition=definition)


def test_z_test_unprobed() -> None:
    with pytest.raises(NoValueError):  # Excel 0 (A50), the math #DIV/0!
        z_test([0.1, 0.1, 0.1], 0)
    with pytest.raises(NoValueError):  # a z past the range above 0 with sigma omitted
        z_test([1e-300, 2e-300, 3e-300 * (1 + 1e-15)], -1e300)


def test_z_test_overflow_below_zero_is_one() -> None:
    # t4098 A49: Excel 0.5, kept as the target (decision 0300); the math is Phi(1.7E+600) = 1
    assert z_test([1, 2, 3], 1e300, 1e-300) == 1


def test_prob_num_band_widens_with_more_probabilities() -> None:
    # two probabilities 1E-15 below 1 are #NUM! (t4098 A3); with more terms the band moves by 2^-53 an addition
    with pytest.raises(DomainError, match="#NUM!"):
        prob([1, 2, 3], [0.5, 0.25, 0.25 - 1e-14], 1)
    with pytest.raises(NoValueError):
        prob(range(1, 11), [*([0.0625] * 8), 0.25, 0.25 - 1.1e-15], 1)


def test_prob_excel_values() -> None:
    tenths = [0.1] * 10
    assert prob(range(1, 11), tenths, 1) == pytest.approx(0.1, rel=1e-15)  # t4098 A1
    assert prob([1, 2], [0.5, 0.500000000000001], 1) == mp(0.5)  # A2
    assert prob([1, 2], [1.5, -0.5], 1) == mp(1.5)  # A32
    assert prob([1, 2, 3], [1.2, -0.1, -0.1], 1) == mp(1.2)  # A33
    assert prob([1, 2], [1, 0], 2) == 0  # A35
    assert prob([1], [1], 1) == 1  # A36
    assert prob([1, 2], [0.5, 0.5], 2, 1) == 0  # A39: lower above upper
    assert prob([1, 2, 3], [0.25, 0.25, 0.5], -1e300, 1e300) == 1  # A41


@pytest.mark.parametrize(
    ("ps", "code"),
    [
        ([0.5, 0.499999999999999], "#NUM!"),  # t4098 A3
        ([0.5, 0.50000000000001], "#NUM!"),  # A4
        ([0.5, 0.4], "#NUM!"),  # A31
        ([0.5, 0.6], "#NUM!"),  # A30
    ],
)
def test_prob_domain(ps: Sequence[float], code: str) -> None:
    with pytest.raises(DomainError, match=code):
        prob([1, 2], ps, 1)
    with pytest.raises(DomainError, match="#N/A"):
        prob([1, 2, 3], [0.5, 0.5], 1)


def test_prob_unprobed_band() -> None:
    with pytest.raises(NoValueError):
        prob([1, 2], [0.5, 0.500000000000005], 1)
    with pytest.raises(NoValueError):
        prob([1, 2], [0.5, 0.4999999999999995], 1)


def _evaluate(definition: Definition, point: list[Arg]) -> None:
    args: list[MathArg] = [
        math_arg(p, a) for p, a in zip(definition.params_for(len(point)), point, strict=True)
    ]
    with contextlib.suppress(DomainError, NoValueError):
        definition.evaluate(args)


@pytest.mark.parametrize("definition", [CHISQ_TEST, CHITEST, Z_TEST, ZTEST, PROB], ids=lambda d: d.function)
def test_draws_evaluate(definition: Definition) -> None:
    assert definition.draw is not None
    rng = random.Random(4098)
    for stratum in STRATA:
        for _ in range(20):
            _evaluate(definition, definition.draw(stratum, rng))
