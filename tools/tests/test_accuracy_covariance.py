"""Gate A definitions of COVAR, COVARIANCE.P, COVARIANCE.S and STEYX (T-4095) against Microsoft's published examples,
the semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.paired_regression import COVAR, COVARIANCE_P, COVARIANCE_S, STEYX
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg

mp = mpmath.mpf
POPULATION = [COVAR, COVARIANCE_P]
EVERY = [*POPULATION, COVARIANCE_S, STEYX]
# probes/regression-pairs-a and -b (excel-graph): A1:A6, B1:B6 and the large offset I1:I6
A = (2, 4, 5, 4, 5, 7)
B = (1, 2, 3, 4, 5, 6)
OFFSET = (10000000001, 10000000002, 10000000004, 10000000003, 10000000005, 10000000007)


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def pair(definition: Definition, y: Sequence[float], x: Sequence[float]) -> mpmath.mpf:
    return definition.evaluate([tuple(mp(v) for v in y), tuple(mp(v) for v in x)])


def test_microsofts_examples() -> None:
    for definition in POPULATION:
        assert pair(definition, (3, 2, 4, 5, 6), (9, 7, 12, 15, 17)) == pytest.approx(5.2, rel=1e-15)
    assert float(pair(COVARIANCE_S, (2, 4, 8), (5, 11, 12))) == pytest.approx(9.666666667, abs=5e-10)
    y, x = (2, 3, 9, 1, 8, 7, 5), (6, 5, 11, 7, 5, 4, 4)
    assert float(pair(STEYX, y, x)) == pytest.approx(3.305719, abs=5e-7)


def test_excel_values_of_the_eval_cells() -> None:
    # probes/regression-pairs-a and -b: COVAR L1 2.25, L21 3.1666666666666665, L22 2.8333333333333335, L28 -1.25;
    # STEYX M1 0.8783100656536799, M5 0.4082482904638623, M21 0.822308017832176, M24 1
    assert pair(COVAR, A, B) == mp(9) / 4
    assert pair(COVAR, OFFSET, B) == mp(19) / 6
    assert pair(COVAR, A, OFFSET) == mp(17) / 6
    assert pair(COVAR, (1, 2, 3, 4), (4, 3, 2, 1)) == mp(-5) / 4
    assert pair(STEYX, A, B) == pytest.approx(0.8783100656536799, rel=1e-15)
    assert pair(STEYX, A[:3], B[:3]) == pytest.approx(0.4082482904638623, rel=1e-15)
    assert pair(STEYX, OFFSET, B) == pytest.approx(0.822308017832176, rel=1e-12)
    assert pair(STEYX, B, A) == 1
    # stat-argument-modes H188, H191: COVARIANCE.P 5.75, COVARIANCE.S 6.571428571428571
    y, x = (1, 2, 4, 3, 7, 5, 8, 6), (2, 3, 5, 4, 9, 6, 9, 8)
    assert pair(COVARIANCE_P, y, x) == mp(23) / 4
    assert pair(COVARIANCE_S, y, x) == mp(46) / 7


def test_one_pair_and_constants() -> None:
    # COVAR of one pair is 0 (pairs-b L3), COVARIANCE.P(4,5) 0 and COVARIANCE.S(4,5) #DIV/0! (t2436/computed-scalar
    # A29, A30); a constant series gives COVAR 0 (pairs-b L6, L7) and STEYX 0 for a constant y (M7)
    for definition in POPULATION:
        assert pair(definition, (4,), (5,)) == 0
        assert pair(definition, A, (3, 3, 3, 3, 3, 3)) == 0
    with pytest.raises(DomainError):
        pair(COVARIANCE_S, (4,), (5,))
    assert pair(STEYX, (3, 3, 3, 3, 3, 3), B) == 0


@pytest.mark.parametrize("definition", EVERY)
def test_unequal_lengths_are_na(definition: Definition) -> None:
    with pytest.raises(DomainError, match="#N/A"):  # COVAR({2,4,5},{1,2}), pairs-a L17
        pair(definition, (2, 4, 5), (1, 2))


def test_steyx_domain() -> None:
    cases: list[tuple[Sequence[float], Sequence[float]]] = [
        ((2,), (1,)),  # one pair: #DIV/0! (pairs-b M3)
        ((2, 4), (1, 2)),  # two pairs: #DIV/0! (pairs-b M4)
        (A, (4, 4, 4, 4, 4, 4)),  # x constant: #DIV/0! (pairs-b M6)
        ((1, 2), (1e200, 2e200)),  # x's sum about the mean overflows before the two-pair #DIV/0! (Z69)
        ((1e200, 2e200, 3e200, 5e200), (1, 2, 3, 4)),  # y's alone, four pairs: #NUM! (T-4047 H4)
        (
            (1.8, 1.9, 2.0, 2.1, 2.2, 2.3, 2.4),
            (1, 2, 3, 4, 5, 6, 7),
        ),  # a residual below zero: #NUM! (steyx-exact-fit A8)
        ((1.9, 2.1, 2.3, 2.5, 2.7), (1, 2, 3, 4, 5)),  # A13
    ]
    for y, x in cases:
        with pytest.raises(DomainError):
            pair(STEYX, y, x)
    # Sxy^2 overflows near 1E+160 while the sums about the means do not: #NUM! (T-4047 J4)
    near = (1.6e160, 1.61e160, 1.63e160, 1.62e160)
    with pytest.raises(DomainError, match="#NUM!"):
        pair(STEYX, near, (1.5e160, 1.52e160, 1.51e160, 1.55e160))
    # an exact fit whose residual cancels: 0 (steyx-exact-fit A6, its 0.6000000000000001 read at 15 digits as 0.6)
    assert pair(STEYX, (0.4, 0.5, 0.6), (1, 2, 3)) == 0
    assert pair(STEYX, (2, 4, 6, 8), (1, 2, 3, 4)) == 0


def test_unprobed_points_have_no_value() -> None:
    with pytest.raises(NoValueError):  # y's overflow alone with three pairs ("Unprobed" in T-4047)
        pair(STEYX, (1e200, 2e200, 5e200), (1, 2, 3))
    for definition in EVERY:
        with pytest.raises(
            NoValueError
        ):  # a constant whose double mean is not its value ("Inexact constants")
            pair(definition, (1, 2, 4), (0.1, 0.1, 0.1))
    for definition in [*POPULATION, COVARIANCE_S]:
        with pytest.raises(NoValueError):  # the sum of x beyond the double range: unprobed for COVARIANCE
            pair(definition, (1, 2, 3, 4), (1.5e308, 1.6e308, 1.4e308, 1.55e308))
    with pytest.raises(NoValueError):  # a deviation product below the normal range
        pair(COVAR, (1, 2), (1e-160, 2e-160))


def test_covariance_with_y_of_1e200_is_finite() -> None:
    # statistics.md "Paired regression functions" (Overflow): COVARIANCE.P is finite where only y's square sum overflows;
    # correl-inexact-constant-overflow Z67 is 6.666666666666667E+199
    assert pair(COVARIANCE_P, (1, 2, 3), (1e200, 2e200, 3e200)) == pytest.approx(
        6.666666666666667e199, rel=1e-15
    )


@pytest.mark.parametrize("definition", EVERY)
def test_draws_are_reproducible_and_evaluate(definition: Definition) -> None:
    for stratum in STRATA:
        points = []
        for seed in range(2):
            sampler = Sampler(definition, random.Random(seed))
            points.append([sampler.draw(stratum, i) for i in range(12)])
        assert points[0] != points[1] or stratum == "edge"
        again = Sampler(definition, random.Random(0))
        assert [again.draw(stratum, i) for i in range(12)] == points[0]
        for args in points[0]:
            params = definition.params_for(len(args))
            math_args = [math_arg(p, a) for p, a in zip(params, args, strict=True)]
            try:
                definition.evaluate(math_args)
            except DomainError as why:
                # only the edge stratum aims at Excel's errors; STEYX's residual can round below zero anywhere
                assert stratum == "edge" or "below zero" in str(why)
            except NoValueError:
                pass  # the generator redraws
