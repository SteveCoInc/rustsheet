"""2-D array literals and the Gate A definitions of MDETERM, MINVERSE and MMULT (T-3686) against the values
arrays.md cites."""

from __future__ import annotations

import io
import random
from collections.abc import Sequence

import mpmath
import pytest

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.defs.cashflow import IRR
from rustsheet_tools.accuracy.defs.matrix import MDETERM, MINVERSE, MMULT
from rustsheet_tools.accuracy.model import Definition, NoValueError, Param
from rustsheet_tools.accuracy.sampler import Sampler, math_arg
from rustsheet_tools.accuracy.tables import compute, draw_points, formula, parse_formula


def value(definition: Definition, args: Sequence[literal.Arg]) -> mpmath.mpf:
    result = compute(definition, args)
    assert result.value is not None, result
    with mpmath.workdps(80):
        return mpmath.mpf(result.value)


def why(definition: Definition, args: Sequence[literal.Arg]) -> str | None:
    return compute(definition, args).why


def test_matrix_literals_are_written_row_by_row() -> None:
    assert literal.text(((1.0, 2.5), (3.0, -4e-7))) == "{1,2.5;3,-4E-07}"
    assert literal.text(((1.0, 2.0, 3.0),)) == "{1,2,3}"  # a one-row matrix
    assert literal.text(((1.0,), (2.0,))) == "{1;2}"  # a column
    assert literal.text((-1000.5, 300.0)) == "{-1000.5,300}"  # a one-row array is unchanged


def test_matrix_formulas_round_trip() -> None:
    m = ((1.0, 2.0), (3.0, 4.0))
    assert formula(MDETERM, [m]) == "=MDETERM({1,2;3,4})"
    assert parse_formula(MDETERM, "=MDETERM({1,2;3,4})") == [m]
    text = formula(MINVERSE, [m, 2, 1])
    assert text == "=INDEX(MINVERSE({1,2;3,4}),2,1)"
    assert parse_formula(MINVERSE, text) == [m, 2, 1]
    column = ((5.0,), (6.0,))
    text = formula(MMULT, [m, column, 2, 1])
    assert text == "=INDEX(MMULT({1,2;3,4},{5;6}),2,1)"
    assert parse_formula(MMULT, text) == [m, column, 2, 1]
    assert math_arg(MDETERM.params[0], m) == ((1, 2), (3, 4))


@pytest.mark.parametrize(
    "text",
    [
        "=MDETERM({1,2;3})",  # ragged rows
        "=MDETERM(5)",  # not an array literal
        "=MINVERSE({1,2;3,4})",  # an indexed definition's cell reads one element through INDEX
        "=INDEX(MDETERM({1,2;3,4}),1,1)",  # INDEX of another function
        "=INDEX(MINVERSE({1,2;3,4}),1)",  # INDEX without a column
    ],
)
def test_parse_formula_refuses_other_shapes(text: str) -> None:
    with pytest.raises(ValueError, match="not"):
        parse_formula(MINVERSE if "MINVERSE" in text else MDETERM, text)


def test_one_row_array_parameters_refuse_a_matrix() -> None:
    with pytest.raises(ValueError, match="one-row array"):
        parse_formula(IRR, "=IRR({-100,50;60,70})")


def test_an_indexed_definition_ends_in_row_and_column() -> None:
    with pytest.raises(ValueError, match="indexed"):
        Definition(
            function="X",
            params=(Param("array", "matrix"), Param("row_num", "int")),
            value=lambda _: mpmath.mpf(0),
            sources=(),
            indexed=True,
        )


def test_mdeterm_is_the_exact_determinant() -> None:
    assert value(MDETERM, [((1.0, 2.0), (3.0, 4.0))]) == -2
    # arrays.md: Excel's elimination gives -2.9999999999999982 and 6.66E-16; the math is exact
    assert value(MDETERM, [((1.0, 2.0, 3.0), (4.0, 5.0, 6.0), (7.0, 8.0, 10.0))]) == -3
    assert value(MDETERM, [((1.0, 2.0, 3.0), (4.0, 5.0, 6.0), (7.0, 8.0, 9.0))]) == 0
    assert value(MDETERM, [((1.0, 2.0), (2.0, 4.0))]) == 0
    assert float(value(MDETERM, [((1e-20,),)])) == 1e-20


def test_mdeterm_overflow_is_num_and_underflow_is_tiny() -> None:
    # probes/minverse-singular-near-singular H1: MDETERM({1E+200,0;0,1E+200}) is #NUM!
    assert why(MDETERM, [((1e200, 0.0), (0.0, 1e200))]) is not None
    # {1E-200,0;0,1E-200} is 0 in Excel: the math 1E-400 is inside Gate A's 1E-300 floor of it
    tiny = value(MDETERM, [((1e-200, 0.0), (0.0, 1e-200))])
    assert 0 < tiny < mpmath.mpf("1e-399")


def test_minverse_elements_and_singularity() -> None:
    m = ((1.0, 2.0), (3.0, 4.0))  # inverse {-2,1;1.5,-0.5}
    assert [value(MINVERSE, [m, r, c]) for r in (1, 2) for c in (1, 2)] == [-2, 1, 1.5, -0.5]
    assert why(MINVERSE, [((1.0, 2.0), (2.0, 4.0)), 1, 1]) is not None  # arrays.md: #NUM!
    # probes/minverse-overflow-computed-pivot: the overflowing corner makes every cell #NUM!
    assert why(MINVERSE, [((1e-200, 1.0), (0.0, 1e-200)), 1, 1]) is not None
    assert float(value(MINVERSE, [((1e-200, 0.0), (0.0, 1e-200)), 2, 2])) == 1 / 1e-200


def test_mmult_elements() -> None:
    a, b = ((1.0, 2.0), (3.0, 4.0)), ((5.0, 6.0), (7.0, 8.0))  # product {19,22;43,50}
    assert [value(MMULT, [a, b, r, c]) for r in (1, 2) for c in (1, 2)] == [19, 22, 43, 50]
    assert value(MMULT, [((1.0, 2.0, 3.0),), ((4.0,), (5.0,), (6.0,)), 1, 1]) == 32


@pytest.mark.parametrize(
    ("definition", "args"),
    [
        (MMULT, [((1.0, 2.0),), ((1.0, 2.0),), 1, 1]),  # not conformable
        (MMULT, [((1.0,),), ((1.0,),), 2, 1]),  # INDEX outside the product
        (MINVERSE, [((1.0, 2.0), (3.0, 4.0)), 1, 3]),  # INDEX outside the inverse
        (MDETERM, [((1.0, 2.0),)]),  # not square
        (MMULT, [((1e200,),), ((1e200,),), 1, 1]),  # past the double range: not claimed
    ],
)
def test_unclaimed_points_have_no_value(definition: Definition, args: list[literal.Arg]) -> None:
    with pytest.raises(NoValueError):
        compute(definition, args)


@pytest.mark.parametrize("definition", [MDETERM, MINVERSE, MMULT])
def test_draws_are_writable_matrices_inside_the_result(definition: Definition) -> None:
    log = io.StringIO()
    points = draw_points(definition, "t", 60, log=log)
    assert len(points) == 60, log.getvalue()
    for point in points:
        text = formula(definition, point.args)
        assert len(text) < 8192  # Excel's formula length
        assert parse_formula(definition, text) == list(point.args)
        matrices = [a for a in point.args if isinstance(a, tuple)]
        for m in matrices:
            widths = {len(row) if isinstance(row, tuple) else -1 for row in m}
            assert len(widths) == 1 and -1 not in widths


def test_domain_errors_are_only_the_evidenced_ones() -> None:
    errors: dict[str, set[str]] = {}
    for definition in (MDETERM, MINVERSE, MMULT):
        for point in draw_points(definition, "s1", log=io.StringIO()):
            if point.math.why is not None:
                errors.setdefault(definition.function, set()).add(point.math.why)
    assert set(errors) <= {"MDETERM", "MINVERSE"}
    assert errors["MDETERM"] == {"the pivot product is beyond the double range"}
    assert errors["MINVERSE"] <= {
        "a singular matrix (an exactly-zero pivot)",
        "an inverse element is beyond the double range",
    }


def test_sampler_finishes_matrix_elements_as_literals() -> None:
    sampler = Sampler(MDETERM, random.Random(1))
    finished = sampler.finish(MDETERM.params[0], ((0.1 + 0.2, 3), (1 / 3, 2.0)))
    assert finished == ((0.3, 3), (0.333333333333333, 2.0))
