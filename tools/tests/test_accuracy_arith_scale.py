"""Math sidecar forms of T-3924: an argument that is arithmetic over literals and exact-input cells,
and a call wrapped in an exact scale (`*2^k`, `/2^k`, `-0`)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import mpmath
import pytest

from rustsheet_tools.accuracy.eval_math import (
    INEXACT_SCALE,
    case_math,
    check_case,
    math_path,
    scale_math,
    scaled_call,
    sidecar_bytes,
)
from rustsheet_tools.accuracy.tables import MathValue
from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.sweep import UnparsedError, Workbook, arithmetic, parse_arg


def _case(tmp_path: Path, cells: dict[str, Any]) -> tuple[Path, Path]:
    case = tmp_path / "c.case.json"
    spec = {"format": "rustsheet-case/v1", "sheets": [{"name": "S", "cells": cells}], "expected": {}}
    case.write_text(json.dumps(spec))
    sidecar = tmp_path / "c.oracle.json"
    oracle = {a: {"t": "n", "v": 0.5} for a, v in cells.items() if isinstance(v, str) and a.startswith("C")}
    sidecar.write_text(json.dumps({"format": "rustsheet-oracle/v1", "sheets": {"S": oracle}}))
    return case, sidecar


def _close(text: str, want: mpmath.mpf) -> bool:
    """`text` is `want` to the 50 significant digits a sidecar holds."""
    return bool(mpmath.almosteq(mpmath.mpf(text), want, rel_eps=mpmath.mpf("1e-49")))


def test_arithmetic_reads_cells_only_through_a_resolver() -> None:
    cells = {"A1": 3.0, "A2": 0.1}

    def resolve(ref: str) -> float:
        return cells[ref.replace("$", "")]

    assert arithmetic("A1*A1", resolve) == 9.0
    assert arithmetic("1/(1+20/(4*A2))", resolve) == X87.div(
        1.0, X87.add(1.0, X87.div(20.0, X87.mul(4.0, 0.1)))
    )
    assert arithmetic("-$A$1^2", resolve) == 9.0  # a unary sign binds tighter than ^
    with pytest.raises(UnparsedError):
        arithmetic("A1*A1")  # no resolver: a reference is unparsed, as before
    with pytest.raises(UnparsedError):
        arithmetic("LOG10(A1)", resolve)  # a function name is never a reference
    with pytest.raises(UnparsedError):
        arithmetic("A1:A2*2", resolve)  # nor is a range


def test_parse_arg_reads_exact_input_and_constant_cells_with_cell_refs() -> None:
    book = Workbook(
        sheets={"S": {"A1": "=0.1", "B1": 2.0, "C1": "x", "D1": 2.0**-1070}, "T": {"A1": 5.0}},
        oracle={"S": {"A1": {"t": "n", "v": 0.25}}},
    )
    # The fit lab's own Workbook reads Excel's value of a formula cell; eval_math's LiteralBook
    # reads its arithmetic instead (test_case_math_*).
    assert parse_arg("B1*B1", "S", book, cell_refs=True) == 4.0
    assert parse_arg("A1*B1+T!A1", "S", book, cell_refs=True) == 5.5
    assert str(parse_arg("-(B1-B1)", "S", book, cell_refs=True)) == "0.0"  # Excel has no negative zero
    with pytest.raises(UnparsedError):
        parse_arg("B1*B1", "S", book)  # off by default: the fit lab's sweeps are unchanged
    for text in ("C1*2", "D1*2", "E1*2"):  # text, a subnormal, an empty cell
        with pytest.raises(UnparsedError):
            parse_arg(text, "S", book, cell_refs=True)


def test_scaled_call_takes_an_exact_power_of_two_or_minus_zero() -> None:
    assert scaled_call("=F.DIST(A1,1,10,FALSE)") == ("F.DIST", "A1,1,10,FALSE", None)
    assert scaled_call("=F.DIST(A1,1,10,FALSE)*2^600") == ("F.DIST", "A1,1,10,FALSE", 600)
    assert scaled_call("=F.DIST(A1,1,10,FALSE) / 2^-3") == ("F.DIST", "A1,1,10,FALSE", 3)
    assert scaled_call("=BINOM.DIST(3000,10000,0.3,TRUE)-0") == ("BINOM.DIST", "3000,10000,0.3,TRUE", 0)
    for text in (
        "=F.DIST(A1,1,10,FALSE)*3",
        "=F.DIST(A1,1,10,FALSE)*2^0.5",
        "=F.DIST(A1,1,10,FALSE)*2^1024",  # 2^1024 overflows
        "=F.DIST(A1,1,10,FALSE)*2^-1023",  # 2^-1023 is subnormal: Excel's ^ gives 0
        "=F.DIST(A1,1,10,FALSE)+1",
        "=F.DIST(A1,1,10,FALSE)*F.DIST(A1,1,10,FALSE)*2^3",
    ):
        assert scaled_call(text) is None, text


def test_scale_math_is_exact_only_in_the_normal_range() -> None:
    with mpmath.workdps(80):
        got = scale_math(MathValue(value="3.25e-300"), 600)
        assert got is not None and got.value is not None
        # the scaled math, written to 50 significant digits again
        assert _close(got.value, mpmath.mpf("3.25e-300") * mpmath.mpf(2) ** 600)
    assert scale_math(MathValue(value="1e-310"), 600) is None  # subnormal in Excel: 0 times 2^600
    assert scale_math(MathValue(value="1e-300"), -100) is None  # the scaled value is subnormal
    assert scale_math(MathValue(value="1e300"), 100) is None  # beyond the double range
    assert scale_math(MathValue(value="0"), 600) == MathValue(value="0")
    assert scale_math(MathValue(value="0.25"), 0) == MathValue(value="0.25")  # `-0`
    error = MathValue(why="x outside its domain")
    assert scale_math(error, 600) == error


def test_case_math_reads_arithmetic_arguments_and_scaled_calls(tmp_path: Path) -> None:
    cells = {
        "A1": "=(3*2^-1)",  # an exact-input cell
        "B1": 0.25,
        "C1": "=T.DIST(A1*A1,3,TRUE)",
        "C2": "=T.DIST(2.25,3,TRUE)",
        "C3": "=T.DIST(1/(1+20/(4*B1)),3,TRUE)",
        "C4": "=T.DIST(1/21,3,TRUE)",
        "C5": "=T.DIST(1.5,3,FALSE)*2^600",
        "C6": "=T.DIST(1.5,3,FALSE)",
        "C7": "=T.DIST(-1E+154,3,FALSE)*2^600",  # the density is subnormal: skipped
        "C8": "=T.DIST(1.5,3,TRUE)-0",
        "C9": "=T.DIST(C2*2,3,TRUE)",  # a formula cell that is a call: never read
    }
    case, sidecar = _case(tmp_path, cells)
    math = case_math("t/c.case.json", case, sidecar)
    got = math.cells["S"]
    assert got["C1"]["v"] == got["C2"]["v"]
    assert got["C3"]["v"] == got["C4"]["v"]
    assert got["C1"]["f"] == "=T.DIST(A1*A1,3,TRUE)"
    with mpmath.workdps(80):
        assert _close(got["C5"]["v"], mpmath.ldexp(mpmath.mpf(got["C6"]["v"]), 600))
    assert got["C5"]["f"] == "=T.DIST(1.5,3,FALSE)*2^600"
    assert "C7" not in got
    assert math.dropped["S"]["C7"] == {"f": cells["C7"], "why": INEXACT_SCALE}
    assert "C8" in got
    assert math.skipped == {INEXACT_SCALE: 1, "unparsed (UnparsedError)": 1}
    # the check holds the scaled cells and the dropped one as current
    assert check_case("t/c.case.json", case, sidecar)
    math_path(sidecar).write_bytes(sidecar_bytes(case, math))
    assert check_case("t/c.case.json", case, sidecar) == []
