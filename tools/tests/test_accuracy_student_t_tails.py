"""Gate A definitions of T.DIST.RT, T.DIST.2T, TDIST and CONFIDENCE.T (T-4102) against published values."""

from __future__ import annotations

from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import student_t
from rustsheet_tools.accuracy.model import REGISTRY, DomainError, MathArg, NoValueError

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(function: str, *args: float | str) -> mpmath.mpf:
    # each literal as its double, as a table cell reads it (the cut tests sit exactly on evidenced literals)
    math_args: Sequence[MathArg] = [mp(float(a)) for a in args]
    return REGISTRY[function].evaluate(math_args)


def close(got: mpmath.mpf, want: str, digits: int) -> bool:
    return bool(abs(got - mp(want)) <= abs(mp(want)) * mp(10) ** -digits)


def test_microsofts_examples() -> None:
    # Microsoft's T.DIST.RT, T.DIST.2T and TDIST pages: x 1.959999998 at df 60 is 0.027322 one-tailed and
    # 0.054645 two-tailed (to the digits shown); CONFIDENCE.T(0.05,1,50) = 0.284196855.
    assert close(value("T.DIST.RT", "1.959999998", 60), "0.027322", 4)
    assert close(value("T.DIST.2T", "1.959999998", 60), "0.054645", 4)
    assert close(value("TDIST", "1.959999998", 60, 1), "0.027322", 4)
    assert close(value("TDIST", "1.959999998", 60, 2), "0.054645", 4)
    assert close(value("CONFIDENCE.T", "0.05", 1, 50), "0.284196855", 8)


def test_the_semantics_entries_values() -> None:
    # statistics.md "Student t sweep (T-1043)": a negative x is valid for T.DIST.RT, and CONFIDENCE.T(0.05,1,10.9) is
    # size 10 (the table's size is already an integer: the eval reader truncates).
    assert close(value("T.DIST.RT", -2, 1), "0.8524163823495667", 15)
    assert close(value("CONFIDENCE.T", "0.05", 1, 10), "0.7153569059706648", 15)
    # "T.DIST.RT and T.DIST.2T are not 1 - T.DIST": Excel's far tail 1.230468749943611E-56 is 33 ulp off the exact
    # value, which the definition keeps to full relative precision (no complement).
    assert close(value("T.DIST.RT", 1e6, 10), "1.230468749943611E-56", 13)
    # "Student-t quantile at a tiny probability (T-1531)": CONFIDENCE.T(1E-300,1,2) = 4.50158158078553E299.
    assert close(value("CONFIDENCE.T", "1e-300", 1, 2), "4.50158158078553E299", 14)


def test_the_two_tailed_value_is_twice_the_upper_tail() -> None:
    for x, df in ((0.5, 3), (4.0, 1), (30.0, 25)):
        rt = value("T.DIST.RT", x, df)
        assert value("T.DIST.2T", x, df) == 2 * rt
        assert value("TDIST", x, df, 2) == 2 * rt
        assert value("TDIST", x, df, 1) == rt
    assert value("T.DIST.2T", 0, 7) == 1
    # df 1 is the Cauchy law: P(T > x) = 1/2 - atan(x)/pi
    assert close(value("T.DIST.RT", 3, 1), str(mp(1) / 2 - mpmath.atan(3) / mpmath.pi), 30)


def test_the_upper_tail_is_the_lower_tail_reflected() -> None:
    for x in (-5.0, -0.25, 0.25, 5.0):
        assert value("T.DIST.RT", x, 4) == student_t.cdf(mp(-x), mp(4))


def test_confidence_t_is_the_two_tailed_quantile_over_root_n() -> None:
    t = student_t.two_tailed_quantile(mp(0.1), mp(19))
    assert value("CONFIDENCE.T", "0.1", 3, 20) == 3 * t / mpmath.sqrt(20)


@pytest.mark.parametrize(
    ("function", "args"),
    [
        # statistics.md T-1043 and T-1048
        ("T.DIST.2T", (-1, 3)),
        ("TDIST", (-1, 3, 1)),
        ("TDIST", (-1, 3, 2)),
        ("TDIST", (1, 3, 0)),
        ("TDIST", (1, 3, 3)),
        ("T.DIST.RT", (2, 0)),
        ("T.DIST.RT", (2, 15e9)),
        ("T.DIST.2T", (2, 2e10)),
        ("CONFIDENCE.T", (0.05, 1, 1)),  # #DIV/0!
        ("CONFIDENCE.T", (0.05, 0, 10)),
        ("CONFIDENCE.T", (0, 1, 10)),
        ("CONFIDENCE.T", (1, 1, 10)),
        ("CONFIDENCE.T", (1.5, 1, 10)),
        ("CONFIDENCE.T", (0.05, 1, 10000000002)),  # df 1E10 + 1 (t1518-t-df-cut)
    ],
)
def test_the_evidenced_error_points_are_outside_the_domain(function: str, args: tuple[float, ...]) -> None:
    with pytest.raises(DomainError):
        value(function, *args)


def test_the_df_cut_points_inside_the_domain_have_values() -> None:
    assert value("CONFIDENCE.T", "0.05", 1, 10000000001) > 0  # df 1E10 is a number (t1518-t-df-cut)
    assert value("T.DIST.RT", 2, 1e10) > 0


def test_a_confidence_margin_past_the_double_range_has_no_value() -> None:
    with pytest.raises(NoValueError):
        value("CONFIDENCE.T", "1e-300", "1e100", 2)
    with pytest.raises(NoValueError):
        value("CONFIDENCE.T", "0.5", "1e-310", 4)


@pytest.mark.parametrize(
    ("function", "args"),
    [
        # the smallest evidenced #NUM! |x| of each bracket (statistics.md "Scalar distributions at extreme arguments,
        # part 2"), and gen-m9's x = 1E+300
        ("T.DIST.RT", ("2.22973889209998E+176", 2723)),
        ("T.DIST.RT", ("1E+300", 7)),
        ("T.DIST.RT", ("-7.96044688386386E+165", 13)),
        ("T.DIST.2T", ("4.22238650908343E+161", 15)),
        ("TDIST", ("2.08984434704072E+166", 2, 1)),
        ("TDIST", ("1E+300", 8, 1)),
    ],
)
def test_a_huge_x_at_or_past_the_evidenced_error_is_outside_the_domain(
    function: str, args: tuple[str | int, ...]
) -> None:
    with pytest.raises(DomainError):
        value(function, *args)


@pytest.mark.parametrize(
    ("function", "args"),
    [
        ("T.DIST.RT", ("1E+100", 5)),
        ("T.DIST.RT", ("-1E+150", 5)),
        ("T.DIST.2T", ("1E+150", 5)),
        ("TDIST", ("1E+160", 5, 2)),
    ],
)
def test_a_huge_x_inside_the_unprobed_bracket_has_no_value(
    function: str, args: tuple[str | int, ...]
) -> None:
    with pytest.raises(NoValueError):
        value(function, *args)


def test_the_largest_evidenced_numbers_keep_their_values() -> None:
    # probes/t4456-beta-t-zero-reach F2, accuracy/T.DIST.RT/s1 A116, T.DIST.2T/s1 A88 and TDIST/s1 A185
    assert value("T.DIST.RT", "1.34305338528263E+60", 5) > 0
    assert value("T.DIST.RT", "-1.35098712206146E+123", 445854763) == 1
    assert value("T.DIST.2T", "5.76102699358893E+146", 21) >= 0
    assert value("TDIST", "1.76387937163976E+150", 7, 2) >= 0


def test_a_negative_standard_deviation_has_no_value() -> None:
    # no cell shows CONFIDENCE.T at a negative sd; sd 0 is the evidenced #NUM! (statistics.md T-1043)
    with pytest.raises(NoValueError):
        value("CONFIDENCE.T", "0.05", -1, 10)
