"""Source blocks for the gen-pivot policy (T-4132): one seeded record table per pivot, from an edge pool.

Every block has the same nine columns, A to I: six label fields (`LABELS`: five texts and `When`, a date)
and three value fields (`VALUES`). A pool decides what goes into one label field (its `edge` field, which the
pivot then puts on an axis) and, for three pools, into the `Score` value field:

| pool | edge field | what it holds |
|---|---|---|
| plain | - | small item sets, numbers |
| types | Tier, Score | numbers, booleans, numeric text, text in one field (T-4134 I2, I5) |
| blanks | Channel, Score | blank cells: the `(blank)` item, blanks under aggregates |
| errors | Product, Score | error values: error items, errors under aggregates (pivots.md "Aggregates over...") |
| case | Region | case variants of one item (East, east, EAST): one item, first-seen casing (T-4134 I1) |
| dates | When | dates with times, several times a day and repeats (T-4164, T-4582 J4) |
| ties | - | repeated records and 0.1/0.2/0.3 sums (summation order, equal totals) |
| many | Rep | 25 to 35 items over 36 to 48 records |

Values come from the shared pools where those hold cell values: the error literals of `POOLS["errors"]`, the
plain date serials of `POOLS["date_serial"]` and the time fractions of `POOLS["time_serial"]` (gen/pools.py),
and the number pool of the GETPIVOTDATA source (gen/pivots.py). Text that reads as a number is a typed text
cell (`{"t": "s"}`), so spec2xlsx keeps it text. No cell is `""` (spec2xlsx writes `=""`, a formula the
builder caches as missing) and none starts with `=`.
"""

from __future__ import annotations

import random
from dataclasses import dataclass, field
from typing import Any

from xlsxwriter.utility import xl_rowcol_to_cell

from rustsheet_tools.gen.pivots import NUMBERS
from rustsheet_tools.gen.pools import POOLS

TEXT_ITEMS: dict[str, list[str]] = {
    "Region": ["East", "West", "North", "South"],
    "Product": ["A", "B", "C"],
    "Channel": ["Web", "Store"],
    "Rep": ["Ann", "Bo", "Cy", "Di"],
    "Tier": ["Gold", "Silver", "Bronze"],
}
DATE_FIELD = "When"
LABELS = [*TEXT_ITEMS, DATE_FIELD]
VALUES = ["Sales", "Units", "Score"]
HEADER = [*LABELS, *VALUES]
POOL_NAMES = ["plain", "types", "blanks", "errors", "case", "dates", "ties", "many"]
EDGE_FIELD = {
    "types": "Tier",
    "blanks": "Channel",
    "errors": "Product",
    "case": "Region",
    "dates": DATE_FIELD,
    "many": "Rep",
}
# The pools that also put their edge values into `Score`, the value field the pivot's first data field reads.
VALUE_EDGE = {"types", "blanks", "errors"}
EDGE_VALUE_FIELD = "Score"
DATE_FORMAT = "m/d/yyyy"
DATE_TIME_FORMAT = "m/d/yyyy h:mm"
# 1900-01-01 to 9999-12-31: the serials a date cell can hold (dates.md).
LAST_SERIAL = 2958465


def _numbers(pool: str) -> list[float]:
    """The entries of `POOLS[pool]` that are plain number literals (not `(-1)`, `1/86400` or `1E+15`)."""
    out = []
    for entry in POOLS[pool]:
        try:
            out.append(float(entry))
        except ValueError:
            continue
    return out


ERRORS = [e for e in POOLS["errors"] if e.startswith("#")]
DAYS = sorted({int(v) for v in _numbers("date_serial") if 1 <= v <= LAST_SERIAL and v == int(v)})
TIMES = sorted({v for v in _numbers("time_serial") if 0 <= v < 1} | {0.25, 0.75})
TYPE_MIX: list[Any] = [1, 2.5, True, False, {"t": "s", "v": "1"}, "Gold", "gold"]
SCORE_TYPE_MIX: list[Any] = [3, 0.5, "abc", True, {"t": "s", "v": "2"}]
TIE_NUMBERS: list[float] = [10, 20, 0.1, 0.2, 0.3]


@dataclass
class Source:
    """A record table: its header, its records (raw spec values, None for a blank) and the number formats of
    its date cells, plus the pool's edge field."""

    pool: str
    rows: list[list[Any]]
    edge: str | None
    formats: dict[str, str] = field(default_factory=dict)

    def column(self, name: str) -> list[Any]:
        i = HEADER.index(name)
        return [r[i] for r in self.rows]

    def items(self, name: str) -> list[Any]:
        """The field's distinct non-blank values in first-seen order (case-insensitively for texts, as the
        builder merges them; the first spelling)."""
        seen: set[Any] = set()
        out = []
        for v in self.column(name):
            if v is None:
                continue
            key = _identity(v)
            if key not in seen:
                seen.add(key)
                out.append(v)
        return out

    def cells(self) -> dict[str, Any]:
        out: dict[str, Any] = {}
        for c, name in enumerate(HEADER):
            out[xl_rowcol_to_cell(0, c)] = name
        for r, row in enumerate(self.rows, start=1):
            for c, v in enumerate(row):
                if v is not None:
                    out[xl_rowcol_to_cell(r, c)] = v
        return out

    @property
    def ref(self) -> str:
        return f"A1:{xl_rowcol_to_cell(len(self.rows), len(HEADER) - 1)}"


def _identity(v: Any) -> tuple[str, str]:
    if isinstance(v, bool):
        return ("b", str(v))
    if isinstance(v, int | float):
        return ("n", repr(float(v)))
    if isinstance(v, dict):
        return (str(v["t"]), str(v["v"]).lower())
    return ("s", str(v).lower())


def _variants(text: str) -> list[str]:
    return [text, text.lower(), text.upper()]


def build_source(rng: random.Random, pool: str) -> Source:
    """A seeded record table for `pool` (module docstring)."""
    if pool not in POOL_NAMES:
        raise ValueError(f"unknown pivot data pool {pool!r}")
    records = rng.randint(36, 48) if pool == "many" else rng.randint(6, 14)
    days = rng.sample(DAYS, 3)
    reps = [f"Rep{n:02d}" for n in range(1, rng.randint(25, 35) + 1)] if pool == "many" else TEXT_ITEMS["Rep"]
    rows: list[list[Any]] = []
    for _ in range(records):
        row: dict[str, Any] = {name: rng.choice(items) for name, items in TEXT_ITEMS.items()}
        row["Rep"] = rng.choice(reps)
        row[DATE_FIELD] = rng.choice(days) + (rng.choice(TIMES) if pool == "dates" else 0)
        row["Sales"] = rng.choice(TIE_NUMBERS if pool == "ties" else NUMBERS)
        row["Units"] = 5 if pool == "ties" else rng.randint(1, 9)
        row["Score"] = rng.choice([1.5, 2, 4.25, 8, -1, 0])
        _apply_edge(rng, pool, row)
        rows.append([row[name] for name in HEADER])
    if pool == "ties" and rows:
        rows.extend(rng.sample(rows, min(3, len(rows))))  # repeated records: equal items, equal totals
    if pool == "many":
        # Every Rep at least once, so the item count is the pool's, not the draw's.
        col = HEADER.index("Rep")
        for k, rep in enumerate(reps[: len(rows)]):
            rows[k][col] = rep
    when = HEADER.index(DATE_FIELD)
    fmt = DATE_TIME_FORMAT if pool == "dates" else DATE_FORMAT
    formats = {xl_rowcol_to_cell(r, when): fmt for r in range(1, len(rows) + 1)}
    return Source(pool, rows, EDGE_FIELD.get(pool), formats)


def _apply_edge(rng: random.Random, pool: str, row: dict[str, Any]) -> None:
    """Puts the pool's edge values into the row, about a third of the time per field."""
    if rng.random() > 0.4:
        return
    if pool == "types":
        row["Tier"] = rng.choice(TYPE_MIX)
        row["Score"] = rng.choice(SCORE_TYPE_MIX)
    elif pool == "blanks":
        row["Channel"] = None
        row["Score"] = None if rng.random() < 0.6 else row["Score"]
    elif pool == "errors":
        row["Product"] = {"t": "e", "v": rng.choice(ERRORS)}
        row["Score"] = {"t": "e", "v": rng.choice(ERRORS)} if rng.random() < 0.6 else row["Score"]
    elif pool == "case":
        row["Region"] = rng.choice(_variants(row["Region"]))


def ensure_error(rng: random.Random, source: Source, name: str) -> None:
    """Puts one error into value field `name` (for an `errorCaption` pivot over a pool without errors)."""
    col = HEADER.index(name)
    rows = source.rows
    if not any(isinstance(r[col], dict) and r[col].get("t") == "e" for r in rows):
        rows[rng.randrange(len(rows))][col] = {"t": "e", "v": rng.choice(ERRORS)}
