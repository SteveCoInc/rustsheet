"""2-D array literals and the Gate A definitions of MDETERM, MINVERSE and MMULT (T-3686) against the values
arrays.md cites."""

from __future__ import annotations

import io
import json
import random
from collections.abc import Sequence
from fractions import Fraction
from pathlib import Path

import mpmath
import pytest

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.defs.cashflow import IRR
from rustsheet_tools.accuracy.defs.matrix import MDETERM, MINVERSE, MMULT, excel_zero_pivot, index_of
from rustsheet_tools.accuracy.eval_math import case_math, fit_args
from rustsheet_tools.accuracy.model import Definition, NoValueError, Param
from rustsheet_tools.accuracy.sampler import Sampler, math_arg
from rustsheet_tools.accuracy.tables import compute, draw_points, formula, parse_formula
from rustsheet_tools.fitlab.sweep import UnparsedError, Workbook, parse_arg, parse_matrix_arg, parse_rows


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


def _exact(m: literal.Matrix) -> list[list[Fraction]]:
    return [[Fraction(x) for x in row] for row in m]


def test_minverse_of_a_residual_pivot_has_no_value() -> None:
    # arrays.md "MINVERSE: singularity rule": Excel's elimination keeps a 1.11E-16 pivot and inverts it, a number
    # where the exact matrix is singular: no math value, not the #NUM! of an exactly-zero pivot
    residual = ((1.0, 2.0, 3.0), (4.0, 5.0, 6.0), (7.0, 8.0, 9.0))
    assert not excel_zero_pivot(_exact(residual))
    with pytest.raises(NoValueError, match="residual pivot"):
        compute(MINVERSE, [residual, 1, 1])
    # `{1,2;2,4}` and a zero column meet an exactly-zero pivot: #NUM! (probes/minverse-singular-near-singular B1:B4)
    for m in (((1.0, 2.0), (2.0, 4.0)), ((0.0, 1.0), (0.0, 2.0))):
        assert excel_zero_pivot(_exact(m))
        assert why(MINVERSE, [m, 1, 1]) == "a singular matrix (an exactly-zero pivot)"


def test_index_of_refuses_a_non_number() -> None:
    with pytest.raises(TypeError, match="row and column"):
        index_of([((1.0,),), True, 1])


def test_the_sweep_reader_keeps_a_matrix_shape() -> None:
    cells = {"A1": 1.0, "B1": 2.0, "A2": 3.0, "B2": 4.0, "C1": "=1+1"}
    book = Workbook({"S": cells}, {"S": {"C1": {"t": "n", "v": 2}}})
    assert parse_rows("{1,2;3,4}", "S", book) == [[1.0, 2.0], [3.0, 4.0]]
    assert parse_rows("{1,2,3}", "S", book) == [[1.0, 2.0, 3.0]]
    assert parse_rows('{"a;b",1;TRUE,2}', "S", book) == [["a;b", 1.0], [True, 2.0]]  # `;` in a text
    assert parse_rows("A1:B2", "S", book) == [[1.0, 2.0], [3.0, 4.0]]
    assert parse_rows("B2:A1", "S", book) == [[1.0, 2.0], [3.0, 4.0]]
    assert parse_rows("A1:A2", "S", book) == [[1.0], [3.0]]
    assert parse_rows("S!B1", "S", book) == [[2.0]]
    assert parse_rows("C1:C2", "S", book) == [[2.0], [None]]
    with pytest.raises(UnparsedError):
        parse_rows("A1*2", "S", book)
    # parse_arg still flattens both, row-major
    assert parse_arg("{1,2;3,4}", "S", book) == [1.0, 2.0, 3.0, 4.0]
    assert parse_arg("A1:B2", "S", book) == [1.0, 2.0, 3.0, 4.0]
    assert parse_matrix_arg("{1,2;3,4}", "S", book) == [[1.0, 2.0], [3.0, 4.0]]
    assert parse_matrix_arg("2*3", "S", book) == 6.0


def test_fit_args_reads_a_matrix_by_its_rows() -> None:
    assert fit_args(MDETERM, [[[1.0, 2.0], [3.0, 4.0]]]) == [((1.0, 2.0), (3.0, 4.0))]
    assert fit_args(MDETERM, [5.0]) == [((5.0,),)]  # `MDETERM(5)` is 5 (stats-evidence-floor-t1623 S!Q112)
    assert fit_args(MDETERM, [[1.0, 2.0, 3.0, 4.0]]) == "a matrix argument without its shape"
    assert fit_args(MDETERM, [[[1.0, 2.0], [3.0]]]) == "ragged matrix rows"
    assert fit_args(MDETERM, [[[True, 2.0], [3.0, 4.0]]]) == "non-numeric matrix element"
    assert fit_args(MDETERM, [[[None]]]) == "non-numeric matrix element"
    assert fit_args(MDETERM, [True]) == "non-numeric argument"
    assert fit_args(MDETERM, ["5"]) == "non-numeric argument"
    assert fit_args(MINVERSE, [[[2.0]], 1.0, 1.0]) == [((2.0,),), 1, 1]


def test_eval_cells_with_matrix_arguments_get_math(tmp_path: Path) -> None:
    cells: dict[str, object] = {
        "A1": "=MDETERM({1,2;3,4})",
        "A2": "=MDETERM(C1:D2)",
        "A3": "=INDEX(MINVERSE(C1:D2),2,1)",
        "A4": "=INDEX(MMULT({1,2;3,4},{5;6}),2,1)",
        "A5": "=INDEX(MINVERSE({1,2;2,4}),1,1)",  # Excel's #NUM!: a domain error
        "A6": "=INDEX(MINVERSE({1,2,3;4,5,6;7,8,9}),1,1)",  # Excel's residual inverse: no value, listed
        "A7": "=MINVERSE({1,2;3,4})",  # an array result outside INDEX: no value
        "A8": "=MDETERM(C1:C2)",  # not square: no value, listed
        "A9": "=MDETERM({TRUE,2;3,4})",  # Excel's #VALUE!: skipped
        "A10": "=INDEX(MINVERSE({4}),1,1)*2^3",  # an exact scale of an INDEX cell
        "A11": "=MDETERM(E1)",  # a blank cell: skipped
        "A12": "=SUM(MDETERM({1,2;3,4}),1)",  # a composite keeps the matrix's rows
        "C1": 1.0,
        "D1": 2.0,
        "C2": 3.0,
        "D2": 4.0,
    }
    oracle = {a: {"t": "n", "v": 0} for a in cells if a.startswith("A")}
    case = tmp_path / "m.case.json"
    case.write_text(json.dumps({"format": "rustsheet-case/v1", "sheets": [{"name": "S", "cells": cells}]}))
    sidecar = tmp_path / "m.oracle.json"
    sidecar.write_text(json.dumps({"format": "rustsheet-oracle/v1", "sheets": {"S": oracle}}))
    math = case_math("t/m.case.json", case, sidecar)
    got = math.cells["S"]
    want = {"A1": -2, "A2": -2, "A3": 1.5, "A4": 39, "A10": 2, "A12": -1}
    assert {a: mpmath.mpf(got[a]["v"]) for a in want} == want
    assert got["A5"]["t"] == "e"
    assert set(got) == {*want, "A5"}
    assert set(math.dropped["S"]) == {"A6", "A8"}
    assert math.skipped["an array result outside INDEX"] == 1
    assert math.skipped["non-numeric matrix element"] == 2
