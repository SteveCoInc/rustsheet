"""Policy `typed-input`: typed number entries drawn from a grammar of Excel's input-parser shapes (T-3462).

Every input cell is a typed entry (`{"entered": "..."}`, text as a user types it) drawn from one of
eight strata of number shapes: plain numbers, sign forms, currency and thousands separators, percent,
E-notation, fractions and mixed fractions, spacing, and near misses that should stay text. Beside each
entry in `A<n>` are three readers: `=A<n>+0` (the number, or #VALUE! when it stayed text),
`=ISTEXT(A<n>)` and `=CELL("format",A<n>)`; the entry cell itself is checked too. Each of the four
cells is tagged in `hypotheses` `typed-input/<stratum>/<variant>`.

The policy draws text only: it never predicts Excel's value (the cases carry no expectations until
`rustsheet-oracle fill`). It never draws an entry that starts with `=`, is over 255 characters, or
has a date shape (digit groups joined by `/` or `-`) without a four-digit group: Excel reads a
year-less date such as `1/2` in the oracle's current year, which the pinned clock cannot reproduce
(docs/excel-semantics/dates.md "The en-US date-text grammar"). Such a draw is redrawn.
"""

from __future__ import annotations

import random
import re
from collections.abc import Callable
from typing import Any

from rustsheet_tools.gen.common import REDRAWS

CASES = 2
ENTRIES_PER_STRATUM = 20  # per case, so every stratum has an equal 1/8 share of each case's 160 entries
MAX_ENTRY = 255
NBSP = chr(0xA0)  # CHAR(160), the no-break space

Draw = Callable[[random.Random], str]


def _digits(rng: random.Random, lo: int, hi: int) -> str:
    """A run of lo..hi digits with no leading zero (a lone `0` when the run is one digit long)."""
    n = rng.randint(lo, hi)
    first = rng.choice("123456789") if n > 1 else rng.choice("0123456789")
    return first + "".join(rng.choice("0123456789") for _ in range(n - 1))


def _int(rng: random.Random) -> str:
    return _digits(rng, 1, 6)


def _dec(rng: random.Random) -> str:
    return f"{_digits(rng, 1, 4)}.{_digits(rng, 1, 4)}"


def _num(rng: random.Random) -> str:
    """A plain integer or decimal, the body the other strata decorate."""
    return _int(rng) if rng.random() < 0.5 else _dec(rng)


def _grouped(rng: random.Random) -> str:
    """A validly grouped integer of 4 to 9 digits (`1,234,567`)."""
    s = _digits(rng, 4, 9)
    head = len(s) % 3 or 3
    return ",".join([s[:head]] + [s[i : i + 3] for i in range(head, len(s), 3)])


def _frac(rng: random.Random) -> str:
    """`n/d` with a one- or two-digit numerator and a denominator below 100."""
    return f"{rng.randint(1, 15)}/{rng.randint(2, 99)}"


def _mixed(rng: random.Random) -> str:
    return f"{rng.randint(1, 99)} {_frac(rng)}"


def _long_decimal(rng: random.Random) -> str:
    """15 to 20 significant digits with the decimal point 1 to 10 digits from the end."""
    s = _digits(rng, 15, 20)
    cut = len(s) - rng.randint(1, 10)
    return f"{s[:cut]}.{s[cut:]}"


PLAIN: dict[str, Draw] = {
    "integer": _int,
    "decimal": _dec,
    "leading-zeros": lambda r: "0" * r.randint(1, 3) + _num(r),
    "digits-15": lambda r: _digits(r, 15, 15),
    "digits-16-20": lambda r: _digits(r, 16, 20),
    "decimal-15-20": _long_decimal,
    "leading-dot": lambda r: "." + _digits(r, 1, 6),
    "trailing-dot": lambda r: _int(r) + ".",
    "zero": lambda r: r.choice(["0", "0.0", "00", "-0", ".0", "0."]),
}
SIGN: dict[str, Draw] = {
    "plus": lambda r: "+" + _num(r),
    "minus": lambda r: "-" + _num(r),
    "parens": lambda r: f"({_num(r)})",
    "parens-minus": lambda r: f"(-{_num(r)})",
    "trailing-minus": lambda r: _num(r) + "-",
    "minus-dollar": lambda r: "-$" + _num(r),
    "dollar-minus": lambda r: "$-" + _num(r),
    "dollar-parens": lambda r: f"(${_num(r)})",
    "minus-space": lambda r: "-" + " " * r.randint(1, 2) + _num(r),
    "plus-space": lambda r: "+ " + _num(r),
    "parens-space": lambda r: f"( {_num(r)} )",
    "double-sign": lambda r: r.choice(["--", "+-", "-+", "++"]) + _num(r),
}
CURRENCY: dict[str, Draw] = {
    "dollar": lambda r: "$" + _num(r),
    "dollar-grouped": lambda r: "$" + _grouped(r),
    "dollar-grouped-decimal": lambda r: f"${_grouped(r)}.{_digits(r, 1, 2)}",
    "grouped": _grouped,
    "grouped-decimal": lambda r: f"{_grouped(r)}.{_digits(r, 1, 3)}",
    "bad-group-short": lambda r: f"{_digits(r, 1, 3)},{_digits(r, 1, 2)}",
    "bad-group-decimal": lambda r: f"{_digits(r, 1, 2)},{_digits(r, 2, 2)}.{_digits(r, 1, 2)}",
    "bad-group-long": lambda r: f"{_digits(r, 1, 3)},{_digits(r, 4, 5)}",
    "leading-comma": lambda r: "," + _digits(r, 1, 3),
    "trailing-comma": lambda r: _int(r) + ",",
    "dollar-space": lambda r: "$ " + _num(r),
    "trailing-dollar": lambda r: _num(r) + "$",
    "euro-suffix": lambda r: _num(r) + "€",
}
PERCENT: dict[str, Draw] = {
    "trailing": lambda r: _num(r) + "%",
    "space-before": lambda r: _num(r) + " %",
    "space-after": lambda r: _num(r) + "% ",
    "leading": lambda r: "%" + _num(r),
    "double": lambda r: _num(r) + "%%",
    "dollar": lambda r: f"${_num(r)}%",
    "grouped": lambda r: _grouped(r) + "%",
    "signed": lambda r: r.choice(["-", "+"]) + _num(r) + "%",
    "parens": lambda r: f"({_num(r)}%)",
    "e-notation": lambda r: f"{_digits(r, 1, 2)}E{r.randint(-5, 5)}%",
    "mixed-fraction": lambda r: _mixed(r) + "%",
}
ENOTATION: dict[str, Draw] = {
    "upper": lambda r: f"{_num(r)}E{r.randint(0, 20)}",
    "lower-plus": lambda r: f"{_num(r)}e+{r.randint(0, 20)}",
    "negative-exp": lambda r: f"{_dec(r)}E-{r.randint(1, 20)}",
    "no-mantissa": lambda r: f"E{r.randint(0, 9)}",
    "no-exponent": lambda r: _num(r) + r.choice(["E", "e", "E+", "E-"]),
    "star-star": lambda r: f"{_digits(r, 1, 3)}**{r.randint(0, 9)}",
    "near-307": lambda r: f"{_digits(r, 1, 1)}.{_digits(r, 1, 3)}E{r.randint(305, 308)}",
    "past-308": lambda r: f"{_digits(r, 1, 2)}E{r.randint(309, 330)}",
    "tiny": lambda r: f"{_digits(r, 1, 2)}E-{r.randint(300, 330)}",
    "signed": lambda r: f"-{_num(r)}E{r.randint(-9, 9)}",
    "dot-mantissa": lambda r: f".{_digits(r, 1, 3)}E{r.randint(0, 9)}",
    "space-inside": lambda r: f"{_num(r)} E{r.randint(0, 9)}",
}
FRACTION: dict[str, Draw] = {
    "mixed": _mixed,
    "zero-whole": lambda r: "0 " + _frac(r),
    "improper": lambda r: f"{r.randint(1, 9)} {r.randint(3, 15)}/{r.randint(2, 3)}",
    "big-denominator": lambda r: f"{r.randint(1, 9)} {r.randint(1, 9)}/{r.randint(100, 9999)}",
    "leading-zero-denominator": lambda r: f"{r.randint(1, 9)} {r.randint(1, 9)}/0{r.randint(1, 99)}",
    "negative": lambda r: "-" + _mixed(r),
    "dollar": lambda r: "$" + _mixed(r),
    "four-digit-denominator": lambda r: f"{r.randint(1, 12)}/{r.randint(1000, 9999)}",
    "zero-denominator": lambda r: f"{r.randint(1, 9)} {r.randint(1, 9)}/0",
    "decimal-numerator": lambda r: f"{r.randint(1, 9)} 1.5/{r.randint(2, 9)}",
}
NEAR_MISS: dict[str, Draw] = {
    "two-dots": lambda r: f"{_int(r)}.{_int(r)}.{_int(r)}",
    "double-comma": lambda r: f"{_int(r)},,{_digits(r, 3, 3)}",
    "lone-symbol": lambda r: r.choice(["$", "-", "+", "()", "%", "."]),
    "e-alone": lambda r: _int(r) + "e",
    "slash-slash": lambda r: f"{_int(r)}//{_int(r)}",
    "letter-after": lambda r: _num(r) + r.choice("kqxz"),
    "letter-before": lambda r: r.choice("kqxz") + _num(r),
    "letter-inside": lambda r: f"{_int(r)}{r.choice('kqxz')}{_int(r)}",
    "decimal-comma": lambda r: f"{_int(r)},{_digits(r, 1, 2)}.{_digits(r, 1, 2)}.",
    "hex": lambda r: "0x" + _digits(r, 1, 4),
}


def _spaced(rng: random.Random) -> str:
    """Leading or trailing spaces, CHAR(160), a tab or a doubled inner space around a body from another stratum."""
    body = rng.choice([_num, _grouped, SIGN["minus"], CURRENCY["dollar"], PERCENT["trailing"], _mixed])(rng)
    form = rng.choice(["lead", "trail", "both", "nbsp-lead", "nbsp-trail", "tab-lead", "tab-trail", "inner"])
    pads = {"lead": (" ", ""), "trail": ("", " "), "both": (" ", " "), "nbsp-lead": (NBSP, ""),
            "nbsp-trail": ("", NBSP), "tab-lead": ("\t", ""), "tab-trail": ("", "\t")}  # fmt: skip
    if form == "inner":
        return body.replace(" ", "  ") if " " in body else body[:1] + "  " + body[1:]
    lead, trail = pads[form]
    return lead * rng.randint(1, 2) + body + trail * rng.randint(1, 2)


SPACING: dict[str, Draw] = {"spaced": _spaced}

STRATA: dict[str, dict[str, Draw]] = {
    "plain": PLAIN,
    "sign": SIGN,
    "currency": CURRENCY,
    "percent": PERCENT,
    "e-notation": ENOTATION,
    "fraction": FRACTION,
    "spacing": SPACING,
    "near-miss": NEAR_MISS,
}

# Digit groups joined by `/` or `-` (spaces around the separator are ignored, dates.md), optionally
# after a sign or currency and before a percent sign: a shape Excel may read as a date.
DATE_SHAPE = re.compile(r"^\s*[-+($]*\s*(\d+)\s*[/-]\s*(\d+)(?:\s*[/-]\s*(\d+))?\s*[)%]*\s*$")


def excluded(text: str) -> bool:
    """True for an entry the policy never draws: a formula, over 255 characters, or a date shape
    without a four-digit group (a year-less date reads in the oracle's current year)."""
    if text.lstrip().startswith("=") or len(text) > MAX_ENTRY:
        return True
    m = DATE_SHAPE.match(text)
    if m is None:
        return False
    return not any(g is not None and len(g) == 4 for g in m.groups())


def draw_entry(rng: random.Random, stratum: str, seen: set[str]) -> tuple[str, str] | None:
    """(variant, text) for one new entry of `stratum`, or None when every redraw repeats or is excluded."""
    variants = STRATA[stratum]
    names = sorted(variants)
    for _ in range(REDRAWS):
        variant = rng.choice(names)
        text = variants[variant](rng)
        if text not in seen and not excluded(text):
            seen.add(text)
            return variant, text
    return None


def _case(seed: int, index: int, entries: list[tuple[str, str, str]]) -> dict[str, Any]:
    """One `rustsheet-case/v1` spec: entry n in A<n>, its readers in B:D, all four cells checked."""
    cells: dict[str, Any] = {}
    hypotheses: dict[str, str] = {}
    for n, (stratum, variant, text) in enumerate(entries, 1):
        cells |= {
            f"A{n}": {"entered": text},
            f"B{n}": f"=A{n}+0",
            f"C{n}": f"=ISTEXT(A{n})",
            f"D{n}": f'=CELL("format",A{n})',
        }
        for col in "ABCD":
            hypotheses[f"Sheet1!{col}{n}"] = f"typed-input/{stratum}/{variant}"
    return {
        "format": "rustsheet-case/v1",
        "description": (
            f"typed-input entries seed={seed} case={index}: typed number shapes (T-3462); "
            'each entry\'s value, A+0, ISTEXT and CELL("format")'
        ),
        "tags": ["generated", "M2", "typed-input", "coercion", "CELL", "ISTEXT"],
        "sheets": [{"name": "Sheet1", "cells": cells}],
        "expected": {},
        "check": [f"Sheet1!{c}" for c in cells],
        "hypotheses": hypotheses,
    }


def generate_typed_input(seed: int) -> list[dict[str, Any]]:
    """Return `CASES` specs of `ENTRIES_PER_STRATUM` entries per stratum each, in a seeded order."""
    rng = random.Random(seed)
    seen: set[str] = set()
    cases: list[dict[str, Any]] = []
    for index in range(CASES):
        entries: list[tuple[str, str, str]] = []
        for stratum in STRATA:
            for _ in range(ENTRIES_PER_STRATUM):
                drawn = draw_entry(rng, stratum, seen)
                if drawn is not None:
                    entries.append((stratum, *drawn))
        rng.shuffle(entries)
        cases.append(_case(seed, index, entries))
    return cases
