"""Collation hold-out sweep generator (T-3465).

Draws random text pairs, deterministically from a fixed seed, from every character category the
engine's collation models (`cmp_text_excel`, crates/rustsheet-engine/src/functions/util/collation.rs)
and writes the probe case `probes/t3465-collation-holdout`. Each pair gets three readers in Excel:
`=A<n><B<n>`, `=B<n><A<n>` and `=A<n>=B<n>`, so the sweep also shows a pair Excel does not order
antisymmetrically. The texts are built by formulas (`=UNICHAR(..)&..`) so that typed-input parsing
never turns one into a number. A third of the pairs are "near" (one edit apart).

Run: `cd tools && uv run python -m rustsheet_tools.gen.collation_holdout` (rewrites the case file).
"""

from __future__ import annotations

import json
import random
from pathlib import Path

SEED = 3465
PAIRS = 600
CASES = Path(__file__).resolve().parents[4] / "eval/suites/probes/cases"
CASE = CASES / "t3465-collation-holdout.case.json"
# Row, near or far (and the edit), and the categories each text was drawn from.
DRAWS = CASES / "t3465-collation-holdout.draws.tsv"


def _chars(spec: str) -> list[int]:
    return [ord(c) for c in spec]


def _span(lo: int, hi: int) -> list[int]:
    return list(range(lo, hi + 1))


# Category name -> code points. Control characters other than TAB/LF are drawn from U+0001..U+001F.
CATEGORIES: dict[str, list[int]] = {
    "ascii-lower": _span(0x61, 0x7A),
    "ascii-upper": _span(0x41, 0x5A),
    "ascii-digit": _span(0x30, 0x39),
    "space": [0x20, 0xA0, 0x09, 0x0A],
    "ascii-punct": _chars('!"#$%&()*+,./:;<=>?@[\\]^_`{|}~'),
    "skipped": [0x2D, 0x27, 0x2013, 0x2014, 0x01, 0x02, 0x1F, 0x7F, 0xAD],
    # Latin-1 symbols by code point (U+00A1..U+00BF, then x and the division sign).
    "latin1-symbol": [*_span(0xA1, 0xAC), *_span(0xAE, 0xBF), 0xD7, 0xF7],
    "latin-accented": _chars(
        "àáâãäåçèéêëìíîïñòóôõöøùúûüýÿÀÁÂÃÄÅÇÈÉÊËÌÍÎÏÑÒÓÔÕÖØÙÚÛÜÝāăąćčďēėęěğģīįķĺľńņňőŕřśşšţťūůűųźżž"
    ),
    "combining": _span(0x300, 0x362),
    "expansion": _chars("æÆßþÞœŒĳĲ"),
    "greek": _chars("αβγδεζηθικλμνξοπρσςτυφχψωάέήίόύώϊΐΑΒΓΔΣΩ"),
    "cyrillic": _chars("абвгдеёжзийклмнопрстуфхцчшщъыьэюяАБВЁЖЯ"),
    "kana": [*_span(0x3041, 0x3093), *_span(0x30A1, 0x30F3), 0x30FC, 0x309D, 0x309E, 0x30FD, 0x30FE, 0x3005],
    "fullwidth": [
        *_span(0xFF10, 0xFF19),
        *_span(0xFF21, 0xFF3A),
        *_span(0xFF41, 0xFF5A),
        *_span(0xFF66, 0xFF9D),
    ],
    "hebrew": [*_span(0x5D0, 0x5EA), 0x5B7, 0x5BC],
    "arabic": [*_span(0x627, 0x64A), 0x64E, 0x651, *_span(0x660, 0x669)],
    "devanagari": [*_span(0x915, 0x939), 0x93E, 0x93F, 0x94D, *_span(0x966, 0x96F)],
    "thai": [*_span(0xE01, 0xE2E), 0xE30, 0xE32, 0xE40, 0xE41, 0xE48, *_span(0xE50, 0xE59)],
    "lao": [*_span(0xE81, 0xE84), 0xEC0, 0xEC1, *_span(0xED0, 0xED9)],
    "hangul": [0xAC00, 0xAC01, 0xB098, 0xB2E4, 0xB77C, 0xD7A3],
    "cjk": _chars("一丁七万丈三上下中乙人入八十龍"),
    "astral": [0x1D400, 0x1D41A, 0x1F600, 0x1F44D, 0x20000],
}
NAMES = sorted(CATEGORIES)
# Categories whose characters are tiebreaks or skipped: never the first character of a text.
NOT_FIRST = {"combining", "skipped"}


def _draw_text(rng: random.Random) -> tuple[list[int], list[str]]:
    cats = rng.sample(NAMES, rng.randint(1, 3))
    length = rng.randint(1, 6)
    starters = [c for c in cats if c not in NOT_FIRST] or ["ascii-lower"]
    units = [rng.choice(CATEGORIES[rng.choice(starters)])]
    for _ in range(length - 1):
        units.append(rng.choice(CATEGORIES[rng.choice(cats)]))
    return units, sorted(set(cats) | ({"ascii-lower"} if starters == ["ascii-lower"] else set()))


def _category_of(cp: int) -> str:
    return next(n for n in NAMES if cp in CATEGORIES[n])


def _near(rng: random.Random, units: list[int]) -> tuple[list[int], str]:
    out = list(units)
    kind = rng.choice(["substitute", "insert-skipped", "insert-mark"])
    if kind == "substitute":
        i = rng.randrange(len(out))
        out[i] = rng.choice(CATEGORIES[_category_of(out[i])])
    else:
        pool = CATEGORIES["skipped" if kind == "insert-skipped" else "combining"]
        out.insert(rng.randint(1, len(out)), rng.choice(pool))
    return out, kind


def _formula(units: list[int]) -> str:
    return "=" + "&".join(f"UNICHAR({u})" for u in units)


def build() -> tuple[dict[str, object], list[str]]:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    check: list[str] = []
    notes: list[str] = []
    for n in range(1, PAIRS + 1):
        a, cats_a = _draw_text(rng)
        if n % 3 == 0:
            b, kind = _near(rng, a)
            cats_b, how = sorted({_category_of(u) for u in b}), f"near:{kind}"
        else:
            b, cats_b = _draw_text(rng)
            how = "far"
        cells[f"A{n}"], cells[f"B{n}"] = _formula(a), _formula(b)
        cells[f"C{n}"], cells[f"D{n}"], cells[f"E{n}"] = f"=A{n}<B{n}", f"=B{n}<A{n}", f"=A{n}=B{n}"
        check += [f"Sheet1!C{n}", f"Sheet1!D{n}", f"Sheet1!E{n}"]
        notes.append(f"{n}\t{how}\t{'+'.join(cats_a)}\t{'+'.join(cats_b)}")
    return {
        "format": "rustsheet-case/v1",
        "description": (
            f"Collation hold-out sweep (T-3465): {PAIRS} random text pairs, seed {SEED}, drawn by "
            "tools/src/rustsheet_tools/gen/collation_holdout.py from the categories it lists; texts of 1 to 6 "
            "characters from 1 to 3 categories, every third pair one edit apart (a substitution within a "
            "category, an inserted skipped character or combining mark). Row n: A, B = the texts (UNICHAR "
            "formulas), C = A<B, D = B<A, E = A=B. The categories of each row's texts are in "
            "t3465-collation-holdout.draws.tsv beside this case."
        ),
        "tags": ["M2", "probe", "collation", "T-3465"],
        "sheets": [{"name": "Sheet1", "cells": cells}],
        "expected": {},
        "check": check,
    }, notes


def main() -> None:
    case, notes = build()
    CASE.write_text(json.dumps(case, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    DRAWS.write_text("row\tpair\tcategories_a\tcategories_b\n" + "\n".join(notes) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
