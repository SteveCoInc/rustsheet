"""Typed date and clock entries for policy `typed-input` (T-3463): the `date` and `clock` strata.

Clock independence is the constraint: year-less date text (`7/1`, `Jul-2`, `1-Jul`, `Feb 29`) takes the current
year of the machine that typed it (docs/excel-semantics/dates.md "The en-US date-text grammar", its year-less
bullet), so the oracle's value is not reproducible against the engine's pinned clock. Every date drawn here writes
its year: a four-digit year, a two-digit or one-digit year in a three-part date (`1/2/30`, `Jul 1, 29`), or a
month-year whose number cannot be a day (`Jul-99`). A pure clock has no date part and is allowed. `yearless`
rejects anything else, whichever variant drew it.
"""

from __future__ import annotations

import random
import re
from collections.abc import Callable

Draw = Callable[[random.Random], str]

MONTHS = (
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
)  # fmt: skip
DAYS_IN = (31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31)  # a non-leap year; the leap day is its own variant


def _year(rng: random.Random) -> int:
    return rng.choice([rng.randint(1900, 2100), rng.randint(1900, 9999)])


def _mdy(rng: random.Random) -> tuple[int, int, int]:
    """A valid (month, day, four-digit year), never 29 February."""
    m = rng.randint(1, 12)
    return m, rng.randint(1, DAYS_IN[m - 1]), _year(rng)


def _cased(rng: random.Random, name: str) -> str:
    return rng.choice([name, name.lower(), name.upper()])


def _mon(rng: random.Random, m: int) -> str:
    """Month `m` as its full or three-letter name, in a drawn case."""
    full = MONTHS[m - 1]
    return _cased(rng, rng.choice([full, full[:3]]))


def _numeric(rng: random.Random, sep: str) -> str:
    m, d, y = _mdy(rng)
    return f"{m}{sep}{d}{sep}{y}"


def _iso(rng: random.Random, sep: str) -> str:
    m, d, y = _mdy(rng)
    return f"{y}{sep}{m:0{rng.choice([1, 2])}d}{sep}{d:0{rng.choice([1, 2])}d}"


def _pivot(rng: random.Random) -> str:
    """A three-part date whose two-digit year sits at the 29/30 pivot (below 30 is 20xx)."""
    m, d, _ = _mdy(rng)
    return f"{m}/{d}/{rng.choice(['28', '29', '30', '31', '00', '99', '09'])}"


def _clock(rng: random.Random) -> str:
    """A plain clock: h:mm or h:mm:ss, with an AM/PM suffix half the time."""
    h, mi = rng.randint(0, 23), rng.randint(0, 59)
    text = f"{h}:{mi:02d}" if rng.random() < 0.5 else f"{h}:{mi:02d}:{rng.randint(0, 59):02d}"
    if h <= 12 and rng.random() < 0.5:
        text += rng.choice([" AM", " PM", "AM", "pm", " a", " P"])
    return text


def _named(form: str) -> Draw:
    """A draw of a valid date written with `form`'s fields: `d`, `y` (four digits), `yy` (two digits, at the
    29/30 pivot or not), `mon` (the full or three-letter name) and `month` (the full name), names in a drawn case."""

    def draw(rng: random.Random) -> str:
        m, d, y = _mdy(rng)
        yy = rng.choice(["29", "30", "05", "75", f"{rng.randint(0, 99):02d}"])
        return form.format(d=d, y=y, yy=yy, mon=_mon(rng, m), month=_cased(rng, MONTHS[m - 1]))

    return draw


def _date(rng: random.Random) -> str:
    return rng.choice(
        [lambda r: _numeric(r, "/"), lambda r: _iso(r, "-"), DATE["mmm-d-yyyy"], DATE["d-mmm-yyyy"]]
    )(rng)


DATE: dict[str, Draw] = {
    "mdy-slash": lambda r: _numeric(r, "/"),
    "mdy-dash": lambda r: _numeric(r, "-"),
    "ymd-dash": lambda r: _iso(r, "-"),
    "ymd-slash": lambda r: _iso(r, "/"),
    "two-digit-year": _pivot,
    "one-digit-year": lambda r: f"{r.randint(1, 12)}/{r.randint(1, 28)}/{r.randint(0, 9)}",
    "invalid-day": lambda r: (
        r.choice(["2/30", "2/31", "4/31", "6/31", "9/31", "11/31", "1/32", "13/1"]) + f"/{_year(r)}"
    ),
    "leap-1900": lambda r: r.choice(["2/29/1900", "2/28/1900", "3/1/1900", "1900-02-29", "29-Feb-1900"]),
    "leap-day": lambda r: f"2/29/{r.choice([2000, 2023, 2024, 2100, 2400, 1904])}",
    "year-edges": lambda r: r.choice(
        ["12/31/9999", "1/1/10000", "12/31/1899", "1/1/1900", "1/0/1900", "9999-12-31", "Dec 31, 9999"]
    ),
    "d-mmm-yyyy": _named("{d}-{mon}-{y}"),
    "mmm-d-yyyy": _named("{mon} {d}, {y}"),
    "d-mmmm-yyyy": _named("{d} {month} {y}"),
    "mmm-d-yy": _named("{mon} {d}, {yy}"),
    "d-mmm-yy-slash": _named("{d}/{mon}/{yy}"),
    "month-year": lambda r: (
        f"{_mon(r, r.randint(1, 12))}{r.choice(['-', ' ', '/'])}"
        + r.choice([str(r.randint(32, 99)), str(_year(r))])
    ),
    "glued": lambda r: _named(r.choice(["{d}{mon}{y}", "{mon}{y}"]))(r),
    "date-clock": lambda r: _date(r) + r.choice([" ", "  "]) + _clock(r),
    "clock-date": lambda r: _clock(r) + r.choice([" ", "  "]) + _date(r),
    "near-miss": lambda r: r.choice(
        [
            f"1/2/3/{r.randint(4, 9)}",
            f"Jul 32, {_year(r)}",
            f"Ju 1, {_year(r)}",
            f"July 1 {_year(r)}",
            f"{_year(r)}.01.01",
            f"Jul-1-{_year(r)}",
            f"Wednesday, July 1, {_year(r)}",
            f"July 1st, {_year(r)}",
        ]
    ),
}

CLOCK: dict[str, Draw] = {
    "h-mm": lambda r: f"{r.randint(0, 23)}:{r.randint(0, 59):02d}",
    "h-mm-ss": lambda r: f"{r.randint(0, 23)}:{r.randint(0, 59):02d}:{r.randint(0, 59):02d}",
    "fraction-seconds": lambda r: (
        f"{r.randint(0, 23)}:{r.randint(0, 59):02d}:{r.randint(0, 59):02d}."
        + str(r.randint(0, 999)).zfill(r.choice([1, 2, 3]))
    ),
    "hours-past-24": lambda r: f"{r.randint(24, 9999)}:{r.randint(0, 59):02d}",
    "minutes-past-59": lambda r: f"{r.randint(0, 23)}:{r.randint(60, 99)}",
    "seconds-past-59": lambda r: f"{r.randint(0, 23)}:{r.randint(0, 59):02d}:{r.randint(60, 99)}",
    "am-pm": lambda r: (
        f"{r.randint(1, 12)}:{r.randint(0, 59):02d}"
        + r.choice([" AM", " PM", "AM", "PM", " am", " pm", " A", " P", "a", "p"])
    ),
    "am-pm-edges": lambda r: r.choice(["0:00 AM", "12:00 AM", "12:00 PM", "13:00 PM", "0:30 PM", "12:59 AM"]),
    "short-parts": lambda r: f"{r.randint(0, 9)}:{r.randint(0, 9)}" + r.choice(["", f":{r.randint(0, 9)}"]),
    "near-miss": lambda r: r.choice(
        [f"{r.randint(1, 12)}:00 XM", "1:2:3:4", "10::00", ":30", "10:", "1:30 AMX", "25:00 PM"]
    ),
}

DATE_STRATA = {"date": DATE, "clock": CLOCK}

# A clock token (h:mm[:ss[.f]] and an optional AM/PM or A/P suffix), removed before the date part is read.
CLOCK_TOKEN = re.compile(r"\d*:\d*(?::\d*(?:\.\d*)?)*(?:\s*[AaPp][Mm]?\b)?")
TOKEN = re.compile(r"\d+|[A-Za-z]+")


def _is_month(token: str) -> bool:
    """A month name as the date grammar reads it: a prefix of at least three letters of the full name."""
    word = token.lower()
    return len(word) >= 3 and any(m.lower().startswith(word) for m in MONTHS)


def yearless(text: str) -> bool:
    """True when `text` has a date part with no written year: one or two parts (a month name or numbers) where the
    number could be a day. Three parts, a four-digit number, or a month-year number above 31 write the year; a text
    with no date part (a pure clock, or no digits and no month) is not year-less."""
    tokens = TOKEN.findall(CLOCK_TOKEN.sub(" ", text))
    months = [t for t in tokens if _is_month(t)]
    numbers = [t for t in tokens if t.isdigit()]
    parts = len(months) + len(numbers)
    if parts == 0 or parts >= 3 or any(len(n) == 4 for n in numbers):
        return False
    if months and not numbers:
        return True  # a month name alone
    return all(int(n) <= 31 for n in numbers)
