"""`rustsheet-gen --policy typed-input` (T-3462, T-3463): strata shapes, exclusions, readers and determinism."""

from __future__ import annotations

import random
import re
from collections import Counter
from collections.abc import Callable
from pathlib import Path

import pytest

from rustsheet_tools.gen.generator import write_case
from rustsheet_tools.gen.typed_dates import DATE_STRATA, yearless
from rustsheet_tools.gen.typed_input import (
    CASES,
    COUNTS,
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
    # A date has a digit and a separator or a word; a clock has a colon and only clock characters.
    "date": lambda t: (
        re.fullmatch(r"[A-Za-z0-9 ,./:-]+", t) is not None
        and any(c.isdigit() for c in t)
        and (bool(set(t) & set("/-.,")) or re.search(r"[A-Za-z]{3}", t) is not None)
    ),
    "clock": lambda t: ":" in t and re.fullmatch(r"[0-9 :.AaPpMmX]+", t) is not None,
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


@pytest.mark.parametrize("stratum", sorted(DATE_STRATA))
def test_no_date_or_clock_variant_draws_a_yearless_date(stratum: str) -> None:
    rng = random.Random(stratum)
    for variant, draw in STRATA[stratum].items():
        for _ in range(200):
            text = draw(rng)
            assert not yearless(text), (stratum, variant, text)


def test_yearless() -> None:
    for text in [
        "7/1",
        "7-1",
        "Jul-2",
        "1-Jul",
        "Feb 29",
        "July 1",
        "1 Jul",
        "12/31",
        "July",
        "7/1 10:00",
        "1Jul",
    ]:
        assert yearless(text), text
        assert excluded(text, "date"), text
    for text in ["1/2/30", "Jul 1, 29", "Jul-99", "July 2026", "1Jul2026", "10:00", "25:30", "1:2:3.5 PM",
                 "10:00 XM", "7/1/2026 10:00", "10:00 PM 1-Jan-2026", "1/2/3/4", "Jul 32, 2026"]:  # fmt: skip
        assert not yearless(text), text


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
            assert not excluded(text, stratum), (tag, text)
        drawn = Counter(case["hypotheses"][f"Sheet1!A{i}"].split("/")[1] for i in range(1, n + 1))
        assert drawn == COUNTS  # every stratum gets its full share; a failed redraw would shrink it
    assert len(entries) == len(set(entries))  # no entry repeats across the batch
    assert sum(COUNTS[s] for s in DATE_STRATA) * CASES >= 150
    assert not any(t.lstrip().startswith("=") or len(t) > MAX_ENTRY for t in entries)
    tags = {t.split("/")[1] for case in cases for t in case["hypotheses"].values()}
    assert tags == set(STRATA)


def test_generation_is_deterministic(tmp_path: Path) -> None:
    def files(out: Path) -> dict[str, str]:
        return {write_case(s, out).name: write_case(s, out).read_text() for s in generate_typed_input(7)}

    first, second = files(tmp_path / "a"), files(tmp_path / "b")
    assert first == second
    assert generate_typed_input(8) != generate_typed_input(7)
