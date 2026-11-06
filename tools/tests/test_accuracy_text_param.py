"""The Gate A `text` parameter kind (T-4084): complex-number and fixed-list samplers, the quoted
literal, parsing it back, and the math and eval-cell paths that read it."""

from __future__ import annotations

import io
import json
import math
import random
from collections.abc import Sequence
from pathlib import Path

import mpmath
import pytest

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.eval_math import case_math, fit_args
from rustsheet_tools.accuracy.model import REGISTRY, Definition, DomainError, Interval, MathArg, Param
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg
from rustsheet_tools.accuracy.tables import batch_files, compute, draw_points, formula, parse_formula
from rustsheet_tools.fitlab.sweep import parse_complex_text, read_complex_arg

PROBE = Path(__file__).resolve().parents[2] / "eval/suites/probes/cases/eng-inumber-grammar-digits"
UNITS = ("m", "km", "mi", "in", 'say "hi"')  # the last one exercises the quote escape


def _modulus(args: Sequence[MathArg]) -> mpmath.mpf:
    (z,) = args
    return abs(z)


def _scale(args: Sequence[MathArg]) -> mpmath.mpf:
    x, unit = args
    assert isinstance(unit, str) and not isinstance(x, (bool, str, tuple))
    return x * (UNITS.index(unit) + 1)


def _quotient_domain(args: Sequence[MathArg]) -> str | None:
    return "#NUM!" if args[1] == 0 else None


def _quotient(args: Sequence[MathArg]) -> mpmath.mpf:
    return abs(args[0] / args[1])


ZABS = Definition(
    "ZABS",
    (Param("inumber", "text", typical=(-10.0, 10.0), extreme=(1e100, 1e300)),),
    _modulus,
    ("test",),
)
ZSCALE = Definition(
    "ZSCALE",
    (Param("number", "real", typical=(-5.0, 5.0)), Param("unit", "text", choices=UNITS)),
    _scale,
    ("test",),
)
ZDIV = Definition(
    "ZDIV",
    (
        Param("inumber1", "text", domain=Interval(-100.0, 100.0), typical=(-1.0, 1.0)),
        Param("inumber2", "text", domain=Interval(-100.0, 100.0), typical=(-1.0, 1.0)),
    ),
    _quotient,
    ("test",),
    domain=_quotient_domain,
)


def test_a_text_is_written_quoted_with_quotes_doubled() -> None:
    assert literal.text("m") == '"m"'
    assert literal.text('say "hi"') == '"say ""hi"""'
    assert literal.text("") == '""'


def test_complex_text_writes_both_parts_as_15_digit_literals() -> None:
    assert literal.complex_text(3.0, -4.0) == "3-4i"
    assert literal.complex_text(1.5, 2e-5, "j") == "1.5+2E-05j"
    assert literal.complex_text(-1e300, 0.0) == "-1E+300+0i"
    assert literal.complex_text(0.1 + 0.2, 1 / 3) == "0.3+0.333333333333333i"
    with pytest.raises(ValueError, match="suffix"):
        literal.complex_text(1.0, 1.0, "k")


@pytest.mark.parametrize("stratum", STRATA)
def test_the_complex_sampler_draws_parsable_texts_in_the_ranges(stratum: str) -> None:
    sampler = Sampler(ZABS, random.Random(7))
    suffixes = set()
    for index in range(40):
        (text,) = sampler.draw(stratum, index)
        assert isinstance(text, str)
        suffixes.add(text[-1])
        re, im = parse_complex_text(text)
        assert literal.complex_text(re, im, text[-1]) == text  # the parts are 15-digit literals
        for part in (re, im):
            assert literal.round15(part) == part
            if stratum == "typical":
                assert -10 <= part <= 10
    assert suffixes == {"i", "j"}


def test_one_point_s_complex_texts_share_a_suffix_and_stay_in_the_domain() -> None:
    sampler = Sampler(ZDIV, random.Random(3))
    for stratum in STRATA:
        for index in range(30):
            a, b = sampler.draw(stratum, index)
            assert isinstance(a, str) and isinstance(b, str)
            assert a[-1] == b[-1]
            for part in (*parse_complex_text(a), *parse_complex_text(b)):
                assert -100 < part < 100


def test_the_fixed_list_sampler_draws_from_the_choices_only() -> None:
    sampler = Sampler(ZSCALE, random.Random(11))
    drawn = {sampler.draw(stratum, i)[1] for stratum in STRATA for i in range(60)}
    assert drawn == set(UNITS)
    assert Sampler(ZSCALE, random.Random(11)).draw("typical", 0) == Sampler(ZSCALE, random.Random(11)).draw(
        "typical", 0
    )


def test_choices_belong_to_a_text_parameter() -> None:
    with pytest.raises(ValueError, match="text parameter"):
        Param("x", "real", choices=("a",))
    assert Param("z", "text").is_complex and not Param("u", "text", choices=("m",)).is_complex


@pytest.mark.parametrize("choices", [(), ("m", "m"), ("m", "")])
def test_choices_are_distinct_non_empty_texts(choices: tuple[str, ...]) -> None:
    with pytest.raises(ValueError, match="choices"):
        Param("u", "text", choices=choices)


def test_text_formulas_round_trip_and_the_math_gets_the_parsed_values() -> None:
    assert formula(ZABS, ["3-4j"]) == '=ZABS("3-4j")'
    assert parse_formula(ZABS, '=ZABS("3-4j")') == ["3-4j"]
    assert formula(ZSCALE, [2.5, 'say "hi"']) == '=ZSCALE(2.5,"say ""hi""")'
    assert parse_formula(ZSCALE, '=ZSCALE(2.5,"say ""hi""")') == [2.5, 'say "hi"']
    with pytest.raises(ValueError, match="not a string literal"):
        parse_formula(ZABS, "=ZABS(5)")
    assert math_arg(ZABS.params[0], "3-4j") == mpmath.mpc(3, -4)
    assert math_arg(ZABS.params[0], "+i") == mpmath.mpc(0, 1)  # engineering.md "Leading plus in IM*..."
    assert math_arg(ZSCALE.params[1], "km") == "km"
    assert compute(ZABS, ["3-4j"]).value == "5.0"
    assert compute(ZSCALE, [2.0, "mi"]).value == "6.0"


def test_a_complex_part_outside_the_domain_is_a_domain_error() -> None:
    assert compute(ZDIV, ["1+i", "0+0i"]).why == "#NUM!"
    assert compute(ZDIV, ["1+200i", "1+i"]).why == "inumber1 outside (-100, 100)"
    with pytest.raises(DomainError):
        ZDIV.evaluate([mpmath.mpc(1, 1), mpmath.mpc(-100, 0)])


def test_a_text_batch_is_reproducible_and_reads_back() -> None:
    first = draw_points(ZDIV, "s1", 30, log=io.StringIO())
    again = draw_points(ZDIV, "s1", 30, log=io.StringIO())
    assert batch_files(ZDIV, "s1", first) == batch_files(ZDIV, "s1", again)
    case = json.loads(batch_files(ZDIV, "s1", first)[0])
    for point, (address, text) in zip(first, case["sheets"][0]["cells"].items(), strict=True):
        assert parse_formula(ZDIV, text) == list(point.args)
        assert case["hypotheses"][f"Calc!{address}"].endswith(",".join(literal.text(a) for a in point.args))


def test_fit_args_reads_text_parameters() -> None:
    assert fit_args(ZABS, ["3-4i"]) == ["3-4i"]
    assert fit_args(ZABS, ["+i"]) == ["+i"]
    assert fit_args(ZABS, [3.0]) == "non-text for a text parameter"
    assert fit_args(ZABS, ["++i"]) == "not a complex number text"
    assert fit_args(ZABS, ["1.23456789012345+2i"]) == ["1.23456789012345+2i"]
    assert fit_args(ZABS, ["1.234567890123456+2i"]) == ["1.234567890123456+2i"]  # cut by the math (T-4593)
    assert fit_args(ZABS, ["1.2345678901234567E-310+1i"]) == "subnormal complex number part"
    assert fit_args(ZABS, ["1.50000000000000000+2E-300i"]) == ["1.50000000000000000+2E-300i"]
    assert fit_args(ZABS, ["1E-320+1i"]) == "subnormal complex number part"
    assert fit_args(ZABS, ["1-2.2250738585072E-308i"]) == "subnormal complex number part"
    assert fit_args(ZABS, ["0+2.2250738585073E-308i"]) == ["0+2.2250738585073E-308i"]
    assert fit_args(ZDIV, ["1+i", "2-3i"]) == ["1+i", "2-3i"]
    assert fit_args(ZDIV, ["1+i", "2-3j"]) == "mixed complex suffixes"
    assert fit_args(ZSCALE, [1.0, "km"]) == [1.0, "km"]
    assert fit_args(ZSCALE, [1.0, "KM"]) == "text outside the parameter's list"
    assert fit_args(ZSCALE, ["1", "km"]) == "non-numeric argument"


def test_eval_cells_with_string_literals_get_math(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    for definition in (ZABS, ZSCALE, ZDIV):
        monkeypatch.setitem(REGISTRY, definition.function, definition)
    cells = {
        "A1": '=ZABS("3+4i")',
        "A2": "=ZABS(B1)",  # a text cell reads as its text
        "A3": '=ZSCALE(2,"in")',
        "A4": '=ZSCALE(2,"yd")',  # outside the list: skipped
        "A5": '=ZABS("3+4k")',  # no complex number: skipped
        "A6": '=ZABS("3+4i")*2^3',  # an exact outer scale
        "A7": '=ZSCALE(ZABS("3+4i"),"m")',  # a composite: the inner call's double
        "B1": "-6-8j",
    }
    oracle = {a: {"t": "n", "v": 1.0} for a in ("A1", "A2", "A3", "A4", "A5", "A6", "A7")}
    case = tmp_path / "c.case.json"
    case.write_text(
        json.dumps({"format": "rustsheet-case/v1", "sheets": [{"name": "S", "cells": cells}], "expected": {}})
    )
    sidecar = tmp_path / "c.oracle.json"
    sidecar.write_text(json.dumps({"format": "rustsheet-oracle/v1", "sheets": {"S": oracle}}))
    math = case_math("t/c.case.json", case, sidecar)
    got = {a: c["v"] for a, c in math.cells["S"].items()}
    assert got == {"A1": "5.0", "A2": "10.0", "A3": "8.0", "A6": "40.0", "A7": "5.0"}
    assert math.skipped == {"text outside the parameter's list": 1, "not a complex number text": 1}


def test_complex_parts_past_15_digits_are_cut_as_excel_reads_them() -> None:
    # probes/eng-inumber-grammar-digits (T-3644): column A holds the texts, B and C Excel's IMREAL and
    # IMAGINARY of them; each part is cut, not rounded, to 15 significant digits (T-4593).
    cells = json.loads(PROBE.with_suffix(".case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads(PROBE.with_suffix(".oracle.json").read_text())["sheets"]["S"]
    rows = [a1[1:] for a1 in cells if a1.startswith("A")]
    assert len(rows) == 6
    for row in rows:
        text = oracle[f"A{row}"]["v"]
        assert fit_args(ZABS, [text]) == [text]
        parts = (oracle[f"B{row}"]["v"], oracle[f"C{row}"]["v"])
        assert read_complex_arg(text) == parts
        assert math_arg(ZABS.params[0], text) == mpmath.mpc(*parts)
    assert read_complex_arg("1e999999999+1e-9999999999i") == (math.inf, 0.0)  # no Decimal overflow


def test_eval_cells_read_long_complex_parts_cut(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setitem(REGISTRY, ZABS.function, ZABS)
    cells = {"A1": '=ZABS("3.0000000000000009+4.0000000000000009i")'}
    case = tmp_path / "c.case.json"
    case.write_text(
        json.dumps({"format": "rustsheet-case/v1", "sheets": [{"name": "S", "cells": cells}], "expected": {}})
    )
    sidecar = tmp_path / "c.oracle.json"
    sidecar.write_text(
        json.dumps({"format": "rustsheet-oracle/v1", "sheets": {"S": {"A1": {"t": "n", "v": 5.0}}}})
    )
    math_values = case_math("t/c.case.json", case, sidecar)
    assert {a: c["v"] for a, c in math_values.cells["S"].items()} == {"A1": "5.0"}
