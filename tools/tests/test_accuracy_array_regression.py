"""Gate A definitions of LINEST, LOGEST, TREND and GROWTH (T-4096) against Microsoft's published examples, the
semantics entries' Excel values, the paired regression definitions and Excel's evidenced errors."""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.array_regression import GROWTH, LINEST, LOGEST, TREND
from rustsheet_tools.accuracy.defs.paired_regression import INTERCEPT, RSQ, SLOPE, STEYX
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg

mp = mpmath.mpf
Rows = Sequence[Sequence[float]]


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def _row(values: Sequence[float]) -> tuple[mpmath.mpf, ...]:
    return tuple(mp(v) for v in values)


def _matrix(rows: Rows) -> tuple[tuple[mpmath.mpf, ...], ...]:
    return tuple(_row(r) for r in rows)


def estimate(
    definition: Definition,
    y: Sequence[float],
    x: Rows,
    cell: tuple[int, int],
    *,
    const: bool = True,
    stats: bool = True,
) -> mpmath.mpf:
    return definition.evaluate([_row(y), _matrix(x), const, stats, *cell])


def predict(
    definition: Definition,
    y: Sequence[float],
    x: Rows,
    new_x: Rows,
    cell: tuple[int, int],
    *,
    const: bool = True,
) -> mpmath.mpf:
    return definition.evaluate([_row(y), _matrix(x), _matrix(new_x), const, *cell])


UNITS = (
    33100,
    47300,
    69000,
    102000,
    150000,
    220000,
)  # Microsoft's LOGEST and GROWTH example, months 11 to 16
MONTHS = [tuple(range(11, 17))]
COSTS = (133890, 135000, 135790, 137300, 138130, 139100, 139900, 141120, 141890, 143230, 144000, 145290)


def test_microsofts_examples() -> None:
    # LINEST example 1: slope 2, intercept 1
    assert estimate(LINEST, (1, 9, 5, 7), [(0, 4, 2, 3)], (1, 1), stats=False) == 2
    assert estimate(LINEST, (1, 9, 5, 7), [(0, 4, 2, 3)], (1, 2), stats=False) == 1
    # LOGEST: m 1.463275628, b 495.3047702; GROWTH at months 17 and 18: 320196.7184 and 468536.0542
    assert float(estimate(LOGEST, UNITS, MONTHS, (1, 1))) == pytest.approx(1.463275628, abs=5e-10)
    assert float(estimate(LOGEST, UNITS, MONTHS, (1, 2))) == pytest.approx(495.3047702, abs=5e-8)
    assert float(predict(GROWTH, UNITS, MONTHS, [(17, 18)], (1, 1))) == pytest.approx(320196.7184, abs=5e-5)
    assert float(predict(GROWTH, UNITS, MONTHS, [(17, 18)], (1, 2))) == pytest.approx(468536.0542, abs=5e-5)
    # TREND: months 13 to 17 of the twelve monthly costs, 146172 to 150244 shown to the unit
    trend = [
        predict(TREND, COSTS, [tuple(range(1, 13))], [tuple(range(13, 18))], (1, j)) for j in range(1, 6)
    ]
    assert [round(float(v)) for v in trend] == [146172, 147190, 148208, 149226, 150244]


def test_excel_values_of_the_semantics_entries() -> None:
    # statistics.md "LINEST and LOGEST: shapes, ...": LINEST({2,4},{1,2}) is {2,0}
    assert estimate(LINEST, (2, 4), [(1, 2)], (1, 1), stats=False) == 2
    assert estimate(LINEST, (2, 4), [(1, 2)], (1, 2), stats=False) == 0
    # "TREND: defaults, ...": TREND({2,4,5},{1,2,3}) starts 2.1666666666666665 (13/6)
    assert predict(TREND, (2, 4, 5), [(1, 2, 3)], [(1, 2, 3)], (1, 1)) == mp(13) / 6
    # "LINEST, LOGEST, GROWTH: probe findings": Excel's 1.9999999999999998, 0.5000000000000001, 15.999999999999996
    # and 31.999999999999968 are exactly 2, 0.5, 16 and 32 (GROWTH's to the ln's guard digits)
    assert estimate(LOGEST, (1, 2, 4, 8), [(1, 2, 3, 4)], (1, 1), stats=False) == 2
    assert estimate(LOGEST, (1, 2, 4, 8), [(1, 2, 3, 4)], (1, 2), stats=False) == mp(1) / 2
    assert predict(GROWTH, (1, 2, 4, 8), [(1, 2, 3, 4)], [(5, 6)], (1, 1)) == pytest.approx(16, rel=1e-45)
    assert predict(GROWTH, (1, 2, 4, 8), [(1, 2, 3, 4)], [(5, 6)], (1, 2)) == pytest.approx(32, rel=1e-45)


def test_one_variable_stats_are_the_paired_functions() -> None:
    y, x = (2, 3, 9, 1, 8, 7, 5), (6, 5, 11, 7, 5, 4, 4)
    paired = [_row(y), _row(x)]
    assert estimate(LINEST, y, [x], (1, 1)) == pytest.approx(SLOPE.evaluate(paired), rel=1e-40)
    assert estimate(LINEST, y, [x], (1, 2)) == pytest.approx(INTERCEPT.evaluate(paired), rel=1e-40)
    assert estimate(LINEST, y, [x], (3, 1)) == pytest.approx(RSQ.evaluate(paired), rel=1e-40)
    assert estimate(LINEST, y, [x], (3, 2)) == pytest.approx(STEYX.evaluate(paired), rel=1e-40)
    assert estimate(LINEST, y, [x], (4, 2)) == 5  # df = n - 2
    r2 = RSQ.evaluate(paired)
    assert estimate(LINEST, y, [x], (4, 1)) == pytest.approx(
        r2 / (1 - r2) * 5, rel=1e-40
    )  # F = r^2 df / (1 - r^2)
    ss_reg, ss_resid = estimate(LINEST, y, [x], (5, 1)), estimate(LINEST, y, [x], (5, 2))
    mean = mp(sum(y)) / len(y)
    assert ss_reg + ss_resid == pytest.approx(sum((v - mean) ** 2 for v in y), rel=1e-40)
    # se of the slope: se_y / sqrt(Sxx)
    sxx = sum((v - mp(sum(x)) / len(x)) ** 2 for v in x)
    se_slope = estimate(LINEST, y, [x], (2, 1))
    assert se_slope == pytest.approx(STEYX.evaluate(paired) / mpmath.sqrt(sxx), rel=1e-40)


def test_two_variables_and_no_constant() -> None:
    # y = 1 + 2 x1 + 3 x2 exactly: row 1 is m2, m1, b
    x1, x2 = (1, 2, 3, 4, 5), (2, 1, 4, 3, 7)
    y = tuple(1 + 2 * a + 3 * b for a, b in zip(x1, x2, strict=True))
    assert [estimate(LINEST, y, [x1, x2], (1, c), stats=False) for c in (1, 2, 3)] == [3, 2, 1]
    assert predict(TREND, y, [x1, x2], [(10, 0), (0, 10)], (1, 2)) == 1 + 30
    # const FALSE: b is 0 (LOGEST's 1) and the fit goes through the origin, r^2 about 0
    assert estimate(LINEST, (2, 4, 7), [(1, 2, 3)], (1, 2), const=False) == 0
    assert estimate(LINEST, (2, 4, 7), [(1, 2, 3)], (1, 1), const=False) == mp(31) / 14
    assert estimate(LOGEST, (2, 4, 7), [(1, 2, 3)], (1, 2), const=False) == 1
    assert estimate(LINEST, (2, 4, 7), [(1, 2, 3)], (4, 2), const=False) == 2  # df = n - k
    ss_resid = mp(4) + 16 + 49 - mp(31) ** 2 / 14
    assert estimate(LINEST, (2, 4, 7), [(1, 2, 3)], (3, 1), const=False) == pytest.approx(1 - ss_resid / 69)
    assert predict(TREND, (5,), [(2,)], [(4,)], (1, 1), const=False) == 10  # one point through the origin


def test_domain_errors_follow_excel() -> None:
    # unequal shapes are #REF!
    for definition in (LINEST, LOGEST):
        with pytest.raises(DomainError):
            estimate(definition, (1, 2, 3), [(1, 2, 3, 4)], (1, 1))
    for definition in (TREND, GROWTH):
        with pytest.raises(DomainError):
            predict(definition, (1, 2, 3), [(1, 2)], [(1,)], (1, 1))
    # LOGEST and GROWTH of a y at or below 0 are #NUM!
    with pytest.raises(DomainError):
        estimate(LOGEST, (1, 0, 3), [(1, 2, 3)], (1, 1))
    with pytest.raises(DomainError):
        predict(GROWTH, (1, -2, 3), [(1, 2, 3)], [(4,)], (1, 1))
    # the stats block's #N/A cells: rows 3-5 right of the second column, b's se without const
    x1, x2 = (1, 2, 3, 4, 5), (2, 1, 4, 3, 7)
    for cell in ((3, 3), (4, 3), (5, 3)):
        with pytest.raises(DomainError):
            estimate(LINEST, (1, 3, 2, 5, 4), [x1, x2], cell)
    with pytest.raises(DomainError):
        estimate(LINEST, (1, 3, 2, 5, 4), [x1], (2, 2), const=False)


def test_unclaimed_points_have_no_value() -> None:
    # one point with a constant (linest-modes K141: Excel drops x), a constant x, collinear x (K106)
    with pytest.raises(NoValueError):
        estimate(LINEST, (9,), [(4,)], (1, 1))
    with pytest.raises(NoValueError):
        predict(GROWTH, (5,), [(5,)], [(5,)], (1, 1))  # Excel 4.999999999999999
    with pytest.raises(NoValueError):
        estimate(LINEST, (1, 2, 3), [(2, 2, 2)], (1, 1))
    with pytest.raises(NoValueError):
        estimate(LINEST, (1, 3, 2, 5), [(1, 2, 3, 4), (2, 4, 6, 8)], (1, 1))
    with pytest.raises(NoValueError):  # within 1E-7 of collinear
        estimate(LINEST, (1, 3, 2, 5), [(1, 2, 3, 4), (2, 4, 6, 8.0000000001)], (1, 1))
    # an exact fit's residual cells (Excel's F is #NUM! or 3.1E+34 there) and a constant y's r^2
    for cell in ((2, 1), (3, 2), (4, 1), (5, 2)):
        with pytest.raises(NoValueError):
            estimate(LINEST, (2, 4, 6), [(1, 2, 3)], cell)
    assert estimate(LINEST, (2, 4, 6), [(1, 2, 3)], (3, 1)) == 1
    with pytest.raises(NoValueError):
        estimate(LINEST, (5, 5, 5), [(1, 2, 3)], (3, 1))
    # INDEX outside the result
    with pytest.raises(NoValueError):
        estimate(LINEST, (1, 3, 2), [(1, 2, 3)], (2, 1), stats=False)
    with pytest.raises(NoValueError):
        predict(TREND, (1, 3, 2), [(1, 2, 3)], [(4, 5)], (1, 3))


@pytest.mark.parametrize("definition", [LINEST, LOGEST, TREND, GROWTH])
def test_draws_are_reproducible_and_evaluate(definition: Definition) -> None:
    for stratum in STRATA:
        points = []
        for seed in range(2):
            sampler = Sampler(definition, random.Random(seed))
            points.append([sampler.draw(stratum, i) for i in range(12)])
        assert points[0] != points[1]
        again = Sampler(definition, random.Random(0))
        assert [again.draw(stratum, i) for i in range(12)] == points[0]
        for args in points[0]:
            params = definition.params_for(len(args))
            math_args = [math_arg(p, a) for p, a in zip(params, args, strict=True)]
            try:
                definition.evaluate(math_args)
            except DomainError:
                assert stratum == "edge"  # only the edge stratum aims at Excel's errors
            except NoValueError:
                pass  # the generator redraws


def test_ln_below_one_keeps_its_sign() -> None:
    # y below 1 has a negative ln: y = 2^-x is LOGEST's m = 1/2, b = 1, and GROWTH extrapolates it
    halves = (0.5, 0.25, 0.125)
    assert estimate(LOGEST, halves, [(1, 2, 3)], (1, 1), stats=False) == pytest.approx(0.5, rel=1e-45)
    assert predict(GROWTH, halves, [(1, 2, 3)], [(4,)], (1, 1)) == pytest.approx(0.0625, rel=1e-45)
