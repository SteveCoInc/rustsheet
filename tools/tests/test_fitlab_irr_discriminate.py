"""The IRR discriminating-probe designer (T-4837): literals, the disagreement test, the search and the files."""

from __future__ import annotations

import json
import random
from pathlib import Path

from rustsheet_tools.fitlab.irr_discriminate import (
    Probe,
    disagree,
    distance,
    draw,
    literal,
    literal_text,
    search,
    write,
)
from rustsheet_tools.fitlab.kernels.irr_driver import Outcome, Run

BASE = "base, cap 200"
RATE = "rate: hold on flat + run on past d = 1, cap 200"


def test_literals_keep_15_digits_and_read_back_as_the_same_double() -> None:
    rng = random.Random(7)
    for _ in range(2000):
        x = literal(rng.uniform(-1, 1) * 10.0 ** rng.uniform(-300, 300))
        text = literal_text(x)
        assert float(text) == x
        mantissa = text.split("E")[0].lstrip("-").replace(".", "").strip("0")
        assert len(mantissa) <= 15, text
    # A large number is not printed as its binary expansion, which Excel would cut to 15 digits;
    # trailing zeros of an integer (`5949622754697460`) are not digits Excel cuts.
    assert literal_text(literal(1.23456789012345e20)) == "1.23456789012345E+20"
    assert literal_text(-100.0) == "-100"


def test_disagree_compares_types_and_15_digits() -> None:
    def out(value: float | str) -> Outcome:
        return Outcome(value, [Run()])

    assert disagree(out("#NUM!"), out(0.1))
    assert disagree(out("#NUM!"), out("#DIV/0!"))
    assert not disagree(out("#NUM!"), out("#NUM!"))
    assert not disagree(out(0.1), out(0.1 + 2**-56))  # equal at 15 digits
    assert disagree(out(0.1), out(0.1000000000001))
    assert distance(out("#NUM!"), out(0.1)) == float("inf")
    assert distance(out(2.0), out(1.0)) == 0.5


def test_draws_have_a_sign_change_axis_and_15_digit_inputs() -> None:
    rng = random.Random(3)
    probes = [draw(rng) for _ in range(300)]
    assert {len(p.flows) for p in probes} >= {2, 21, 120}
    for p in probes:
        assert all(literal(c) == c for c in p.flows)
        assert literal(p.guess) == p.guess
        assert p.formula().startswith("=IRR({")


def test_base_against_rate_finds_at_least_20_disagreeing_inputs(tmp_path: Path) -> None:
    found = search(BASE, RATE, count=20, seed=1)
    assert len(found) >= 20
    assert len({f.probe for f in found}) == len(found)
    assert all(disagree(f.a, f.b) for f in found)
    stem = tmp_path / "designed"
    write(stem, BASE, RATE, found, seed=1)
    case = json.loads(Path(f"{stem}.case.json").read_text(encoding="utf-8"))
    predictions = json.loads(Path(f"{stem}.predictions.json").read_text(encoding="utf-8"))
    cells = case["sheets"][0]["cells"]
    assert len(cells) == len(found) and case["expected"] == {}
    assert set(predictions["cells"]) == {f"P!{a1}" for a1 in cells}
    first = predictions["cells"]["P!A1"]
    assert first[BASE]["value"] == found[0].a.value and first[RATE]["event"] == found[0].b.event
    assert cells["A1"] == Probe(found[0].probe.flows, found[0].probe.guess).formula()
