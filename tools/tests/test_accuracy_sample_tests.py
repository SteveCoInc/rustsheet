"""Gate A definitions of T.TEST, TTEST, F.TEST and FTEST (T-4097) against Microsoft's published examples, the
semantics entries' Excel values and Excel's evidenced errors."""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.sample_tests import (
    F_TEST,
    FTEST,
    MICROSOFT_F,
    MICROSOFT_T,
    T_TEST,
    TTEST,
)
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg

mp = mpmath.mpf
T_TESTS = [T_TEST, TTEST]
F_TESTS = [F_TEST, FTEST]
# probes-numeric t1049-ttest B1:B6, C1:C6, D1:D4 (D5 is text)
B = (3, 5, 8, 9, 14, 6)
C = (2, 4, 4.5, 11, 20, 7)


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(50):
        yield


def array(values: Sequence[float]) -> tuple[mpmath.mpf, ...]:
    return tuple(mp(v) for v in values)


def t_test(
    definition: Definition, x: Sequence[float], y: Sequence[float], tails: int, kind: int
) -> mpmath.mpf:
    return definition.evaluate([array(x), array(y), mp(tails), mp(kind)])


def f_test(definition: Definition, x: Sequence[float], y: Sequence[float]) -> mpmath.mpf:
    return definition.evaluate([array(x), array(y)])


@pytest.mark.parametrize("definition", T_TESTS)
def test_microsofts_t_test_example(definition: Definition) -> None:
    assert float(t_test(definition, *MICROSOFT_T, 2, 1)) == pytest.approx(0.196016, abs=5e-7)


@pytest.mark.parametrize("definition", F_TESTS)
def test_microsofts_f_test_example(definition: Definition) -> None:
    assert float(f_test(definition, *MICROSOFT_F)) == pytest.approx(0.648318, abs=5e-7)


@pytest.mark.parametrize("definition", T_TESTS)
def test_excel_t_test_values(definition: Definition) -> None:
    # probes-numeric t1049-ttest (excel-graph): Excel is within a few tens of ulps of the exact tail (statistics.md
    # "T.TEST rounds y = df/(df+t^2) to a double before the tail")
    cells = [
        (B, C, 1, 1, 0.3398262656772548),  # A10
        (B, C, 2, 2, 0.8553470471640956),  # A13
        (B, C, 2, 3, 0.8562490487708091),  # A15, Welch's fractional df
        (B[:5], C, 2, 2, 0.9359998654897697),  # A16, unequal lengths
        (B[:5], C, 2, 3, 0.9333273460051335),  # A17
        ((1, 2, 3), (2, 4, 7), 2, 1, 0.11808289631180313),  # A44
        ((1, 1, 1), (2, 3, 4), 2, 3, 0.07417990022744847),  # A48: one constant sample, Welch df exactly 2
    ]
    for x, y, tails, kind, excel in cells:
        assert t_test(definition, x, y, tails, kind) == pytest.approx(excel, rel=1e-13)
    assert t_test(definition, B, B, 2, 2) == 1  # A33: t = 0
    assert t_test(definition, B, B, 1, 3) == mp(1) / 2


@pytest.mark.parametrize("definition", F_TESTS)
def test_excel_f_test_values(definition: Definition) -> None:
    # probes-numeric t1049-ftest (excel-graph); F.TEST is symmetric in its arguments (A10, A11)
    assert f_test(definition, B[:5], C[:5]) == pytest.approx(0.3039384077307328, rel=1e-13)
    assert f_test(definition, C[:5], B[:5]) == pytest.approx(f_test(definition, B[:5], C[:5]), rel=1e-45)
    assert f_test(definition, (1, 2, 3), (4, 5, 6, 7)) == pytest.approx(0.7926367789592619, rel=1e-13)  # A27
    assert f_test(definition, B, B) == 1  # A12


@pytest.mark.parametrize("definition", T_TESTS)
def test_t_test_domain_errors_follow_excel(definition: Definition) -> None:
    cases: list[tuple[Sequence[float], Sequence[float], int, int]] = [
        (B, C, 3, 2),  # tails 3: #NUM! (t1049-ttest A22)
        (B, C, 0, 2),  # A23
        (B, C, 2, 0),  # type 0 (A24)
        (B, C, 2, 4),  # A25
        ((1,), (2, 3), 1, 0),  # the type check before the paired size check
        (B[:5], C, 2, 1),  # paired, lengths differ: #N/A (A18)
        ((3,), (2,), 2, 1),  # one pair: #DIV/0! (A41)
        ((3,), (2,), 2, 2),  # A38
        (B[:5], (3,), 2, 2),  # A50
        (B, B, 2, 1),  # paired, zero variance of the differences (A32)
        ((1, 1, 1), (2, 2, 2), 2, 2),  # A45
        ((1, 1, 1), (2, 2, 2), 2, 3),  # A46
        ((3, 3, 3, 3), (5,), 2, 3),  # one number beside a constant sample (gen-m9 607e85cfee83a689 A132)
        (B, (1e200, 1e200, -1e200), 1, 2),  # an overflowing variance: #NUM! (T-2432)
    ]
    for x, y, tails, kind in cases:
        with pytest.raises(DomainError):
            t_test(definition, x, y, tails, kind)


@pytest.mark.parametrize("definition", F_TESTS)
def test_f_test_domain_errors_follow_excel(definition: Definition) -> None:
    cases: list[tuple[Sequence[float], Sequence[float]]] = [
        ((3,), C[:5]),  # one number: #DIV/0! (t1049-ftest A15, A16)
        (B[:5], (3,)),
        ((1, 1, 1), B[:3]),  # a constant sample: #DIV/0! (A17, A18)
        (B[:3], (1, 1, 1)),
        ((1, 1, 1), (4, 4, 4)),  # A19
        (
            (100000000.1, 100000000.2, 100000000.3),
            (1e200, 1e200, -1e200),
        ),  # #NUM! (gen-m9 3e98b26cf6713e77 A339)
    ]
    for x, y in cases:
        with pytest.raises(DomainError):
            f_test(definition, x, y)


def test_unprobed_points_have_no_value() -> None:
    with pytest.raises(NoValueError):  # Welch with one number beside a varying sample
        t_test(T_TEST, B, (5,), 2, 3)
    with pytest.raises(NoValueError):  # an overflow in a Welch test
        t_test(T_TEST, B, (1e200, 1e200, -1e200), 2, 3)
    with pytest.raises(NoValueError):  # an overflow in one form only (one pass; the deviations are 0)
        t_test(T_TEST, B, (1e200, 1e200, 1e200), 2, 2)
    with pytest.raises(NoValueError):  # a constant sample whose double mean is not its value
        f_test(F_TEST, (0.1, 0.1, 0.1), B)
    with pytest.raises(NoValueError):  # a subnormal literal
        f_test(F_TEST, (1e-310, 1), B)


@pytest.mark.parametrize("definition", [*T_TESTS, *F_TESTS])
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
