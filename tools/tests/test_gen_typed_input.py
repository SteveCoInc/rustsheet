"""`rustsheet-gen --policy typed-input` (T-3462): strata shapes, exclusions, readers and determinism."""

from __future__ import annotations

import random
import re
from collections.abc import Callable
from pathlib import Path

import pytest

from rustsheet_tools.gen.generator import write_case
from rustsheet_tools.gen.typed_input import (
    CASES,
    MAX_ENTRY,
    NBSP,
    STRATA,
    excluded,
    generate_typed_input,
)

SPACES = " \t" + NBSP


def _is_float(text: str) -> bool:
    try:
        float(text)
    except ValueError:
        return False
    return True


# What every draw of a stratum must look like (the stratum's shape, not Excel's reading of it).
SHAPES: dict[str, Callable[[str], bool]] = {
    "plain": lambda t: re.fullmatch(r"-?[0-9]*\.?[0-9]*", t) is not None and any(c.isdigit() for c in t),
    "sign": lambda t: re.fullmatch(r"[-+($ ]*[0-9.]+[ )\-]*", t) is not None and bool(set(t) & set("+-()")),
    "currency": lambda t: (
        re.fullmatch(r"[$ ]*[0-9.,]*[0-9][0-9.,]*[$€]?|[0-9.,]*,[0-9.,]*", t) is not None
        and bool(set(t) & set("$€,"))
    ),
    "percent": lambda t: "%" in t and re.fullmatch(r"[-+($%]*[0-9., /E-]+[ %)]*", t) is not None,
    "e-notation": lambda t: (
        ("E" in t or "e" in t or "**" in t) and re.fullmatch(r"[-.0-9 Ee+*]+", t) is not None
    ),
    "fraction": lambda t: "/" in t and re.fullmatch(r"-?\$?[0-9]+( [0-9.]+)?/[0-9]+", t) is not None,
    "spacing": lambda t: t[0] in SPACES or t[-1] in SPACES or "  " in t,
    "near-miss": lambda t: not _is_float(t.replace(",", "")) or t.count(".") > 1 or ",," in t,
}


def test_every_stratum_has_a_shape_check() -> None:
    assert set(SHAPES) == set(STRATA)


@pytest.mark.parametrize("stratum", sorted(STRATA))
def test_every_variant_draws_its_stratum_shape(stratum: str) -> None:
    rng = random.Random(stratum)
    for variant, draw in STRATA[stratum].items():
        for _ in range(100):
            text = draw(rng)
            assert SHAPES[stratum](text), (stratum, variant, text)


def test_exclusions() -> None:
    for text in [
        "=1",
        " =1+2",
        "1" * (MAX_ENTRY + 1),
        "1/2",
        "1 / 2",
        "10 / 20",
        "1-2",
        "1-2-3",
        "$1/2",
        "1/0",
    ]:
        assert excluded(text), text
    for text in ["1", "3 1/2", "0 1/2", "1/2026", "1-2-2026", "2026-1-2", "1E-5", "5-", "1" * MAX_ENTRY]:
        assert not excluded(text), text


def test_cases_carry_entries_readers_and_stratum_tags() -> None:
    cases = generate_typed_input(0)
    assert len(cases) == CASES >= 2
    entries: list[str] = []
    for case in cases:
        cells = case["sheets"][0]["cells"]
        n = sum(1 for c in cells if c.startswith("A"))
        assert n >= 150
        assert case["expected"] == {}
        assert len(case["check"]) == 4 * n
        for i in range(1, n + 1):
            text = cells[f"A{i}"]["entered"]
            entries.append(text)
            assert cells[f"B{i}"] == f"=A{i}+0"
            assert cells[f"C{i}"] == f"=ISTEXT(A{i})"
            assert cells[f"D{i}"] == f'=CELL("format",A{i})'
            tag = case["hypotheses"][f"Sheet1!A{i}"]
            stratum = tag.split("/")[1]
            assert tag.startswith("typed-input/") and stratum in STRATA
            assert SHAPES[stratum](text), (tag, text)
            assert all(case["hypotheses"][f"Sheet1!{c}{i}"] == tag for c in "BCD")
    assert len(entries) == len(set(entries))  # no entry repeats across the batch
    assert not any(excluded(t) for t in entries)
    assert not any(t.lstrip().startswith("=") or len(t) > MAX_ENTRY for t in entries)
    tags = {t.split("/")[1] for case in cases for t in case["hypotheses"].values()}
    assert tags == set(STRATA)


def test_generation_is_deterministic(tmp_path: Path) -> None:
    def files(out: Path) -> dict[str, str]:
        return {write_case(s, out).name: write_case(s, out).read_text() for s in generate_typed_input(7)}

    first, second = files(tmp_path / "a"), files(tmp_path / "b")
    assert first == second
    assert generate_typed_input(8) != generate_typed_input(7)
