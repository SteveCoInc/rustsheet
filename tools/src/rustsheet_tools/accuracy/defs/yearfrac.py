"""YEARFRAC (Gate A slice 4k, T-3678, decision 0228).

The math is the exact quotient of a basis day count by a basis year length, each read the way
dates.md states Excel reads it (excel-graph cells):
- Both dates are floored to whole serials; one below 0 or past 2958465 (9999-12-31) is `#NUM!`, and
  so is a basis outside [0, 5) once truncated ("YEARFRAC accepts serial 0": only a negative serial
  is `#NUM!`). A reversed pair is swapped first, on every basis ("YEARFRAC swaps reversed dates").
- Basis 0 (US 30/360) counts on Excel's displayed day, month and year: serial 0 is day 0 of January
  1900, serial 60 the fictitious 1900-02-29, which is not February's last day (1900-02-28 is). The
  end's day becomes 30 when both dates are February's last day, or when it is 31 and the start's day
  is 30 or 31; the start's day becomes 30 when it is 31 or February's last day ("YEARFRAC accepts
  serial 0": `YEARFRAC(60,59,0)` is -1/360; "Serials below 61 ...": `YEARFRAC(60,425,0)` is 359/360).
  Over 360.
- Bases 2 and 3 count serial days (the fictitious day included) over 360 and 365.
- Basis 4 (European 30/360) caps each displayed day at 30 (serial 60 is day 29, serial 0 day 0),
  over 360. The engine reads basis 4's dates rounded to the nearest second first, as DAYS360 does;
  that is unprobed for YEARFRAC, so the draws keep every fraction of a day below 0.99.
- Basis 1 (actual/actual) counts serial days over a year length read on the real calendar, where
  serial 0 stands in as 1900-01-01 and serial 60 as 1900-02-28 ("YEARFRAC accepts serial 0":
  `YEARFRAC(0,33752,1)` averages 1900-1992, `YEARFRAC(59,60,1)` is 1/365). Dates more than a year
  apart take the average length of the calendar years from the start's to the end's, both
  included; dates at most a year apart take 366 when both are in one leap year, when the end is a
  29 February, or when a 29 February lies between them, else 365 (dates.md "YEARFRAC basis 1's year
  length", probes/t3678-yearfrac-basis1-within-a-year, every branch evidenced).
"""

from __future__ import annotations

import calendar
import datetime
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, domain_of, fraction_to_mpf, one_of
from rustsheet_tools.accuracy.defs.clock import LAST_DAY
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.sweep import EPOCH

SWAP_SOURCE = "dates.md#yearfrac-swaps-reversed-dates"
ZERO_SOURCE = "dates.md#yearfrac-accepts-serial-0"
EARLY_SOURCE = (
    "dates.md#serials-below-61-in-datedif-edate-days360-weeknum-and-yearfrac-gen-m6-triage-t-2263-t-2265"
)
BASIS1_SOURCE = (
    "crates/rustsheet-engine/src/functions/date_and_time/yearfrac.rs (ExcelFinancialFunctions port)"
)
DECISION = "decision 0228"
FICTITIOUS_DAY = 60  # 1900-02-29

YMD = tuple[int, int, int]


def displayed_date(serial: int) -> YMD:
    """Excel's displayed (year, month, day) of a whole serial in 0..2958465: serial 0 is
    1900-01-00 and serial 60 the fictitious 1900-02-29."""
    if serial == 0:
        return (1900, 1, 0)
    if serial == FICTITIOUS_DAY:
        return (1900, 2, 29)
    # Below 60 the displayed calendar counts from 1900-01-01 = 1; from 61 on it is one day behind.
    when = datetime.date(1899, 12, 31) if serial < FICTITIOUS_DAY else EPOCH
    when += datetime.timedelta(days=serial)
    return (when.year, when.month, when.day)


def real_date(serial: int) -> datetime.date:
    """The real calendar date basis 1 reads: day 0 stands in as day 1, and 1900-02-29 as 1900-02-28."""
    year, month, day = displayed_date(serial)
    return datetime.date(year, month, min(max(day, 1), calendar.monthrange(year, month)[1]))


def _feb_last(year: int, month: int, day: int) -> bool:
    return month == 2 and day == (29 if calendar.isleap(year) else 28)


def us_30_360(start: YMD, end: YMD) -> int:
    """Basis 0's day count (module docstring).

    Not `fitlab.kernels.solvers.days360`: that one takes `datetime.date`, which cannot hold the
    displayed day 0 of January 1900 or the fictitious 1900-02-29 this count reads.
    """
    (y1, m1, d1), (y2, m2, d2) = start, end
    start_feb_last, end_feb_last = _feb_last(*start), _feb_last(*end)
    if start_feb_last and end_feb_last:
        d2 = 30
    if d2 == 31 and d1 >= 30:
        d2 = 30
    if d1 == 31 or start_feb_last:
        d1 = 30
    return 360 * (y2 - y1) + 30 * (m2 - m1) + (d2 - d1)


def eu_30_360(start: YMD, end: YMD) -> int:
    """Basis 4's day count: each day capped at 30."""
    (y1, m1, d1), (y2, m2, d2) = start, end
    return 360 * (y2 - y1) + 30 * (m2 - m1) + (min(d2, 30) - min(d1, 30))


def _feb29_between(start: datetime.date, end: datetime.date) -> bool:
    """Whether a 29 February lies between dates in one year or consecutive years."""
    if start.year == end.year:
        return calendar.isleap(start.year) and start.month <= 2 < end.month
    if end.year == start.year + 1:
        if calendar.isleap(start.year):
            return start.month <= 2
        return calendar.isleap(end.year) and end.month > 2
    return False


def actual_year(start: datetime.date, end: datetime.date) -> Fraction:
    """Basis 1's year length for an ordered pair (module docstring)."""
    within = start.year == end.year or (
        end.year == start.year + 1 and (start.month, start.day) >= (end.month, end.day)
    )
    if not within:
        years = range(start.year, end.year + 1)
        return Fraction(sum(366 if calendar.isleap(y) else 365 for y in years), len(years))
    leap = (
        (start.year == end.year and calendar.isleap(start.year))
        or (end.month == 2 and end.day == 29)
        or _feb29_between(start, end)
    )
    return Fraction(366 if leap else 365)


def whole_serial(x: float) -> int:
    """A date argument floored to a whole serial; DomainError (`#NUM!`) outside 0..2958465 (YEARFRAC, and the bond
    functions' settlement and maturity, T-4113)."""
    whole = int(x // 1)
    if not 0 <= whole <= LAST_DAY:
        msg = f"date {x} is outside 1900-01-00..9999-12-31"
        raise DomainError(msg)
    return whole


def yearfrac_value(start: float, end: float, basis: int = 0) -> Fraction:
    """YEARFRAC's math (module docstring). DomainError for a date outside the calendar or a basis
    outside 0..4."""
    if not 0 <= basis < 5:
        msg = f"basis {basis} is outside 0..4"
        raise DomainError(msg)
    s, e = sorted((whole_serial(start), whole_serial(end)))
    days = e - s
    if basis == 0:
        return Fraction(us_30_360(displayed_date(s), displayed_date(e)), 360)
    if basis == 1:
        return days / actual_year(real_date(s), real_date(e))
    if basis == 2:
        return Fraction(days, 360)
    if basis == 3:
        return Fraction(days, 365)
    return Fraction(eu_30_360(displayed_date(s), displayed_date(e)), 360)


def _value(args: Sequence[MathArg]) -> mpmath.mpf:
    start, end, *rest = args
    assert not isinstance(start, bool | tuple) and not isinstance(end, bool | tuple)
    basis = 0
    if rest:
        (b,) = rest
        assert not isinstance(b, bool | tuple)
        basis = int(b)
    return fraction_to_mpf(yearfrac_value(float(start), float(end), basis))


# -- draws -------------------------------------------------------------------------------------


def _with_basis(rng: random.Random, start: float, end: float) -> list[Arg]:
    """[start, end] with basis 0-4, or with the basis left out one time in six (basis 0)."""
    if rng.random() < 1 / 6:
        return [start, end]
    return [start, end, rng.randrange(5)]


def _date(rng: random.Random, lo: int, hi: int, fraction: float = 0.0) -> float:
    """A whole serial in [lo, hi], with a fraction of a day below 0.99 with probability `fraction`."""
    day = float(rng.randint(lo, hi))
    if rng.random() < fraction:
        return round15(day + rng.uniform(0.0, 0.99))
    return day


def _serial_of(year: int, month: int, day: int) -> float:
    return float((datetime.date(year, month, day) - EPOCH).days)


def _typical(rng: random.Random) -> list[Arg]:
    """Two dates from 1900 to 2100 (serials 1 to 73050), one in four with a time of day."""
    return _with_basis(rng, _date(rng, 1, 73050, 0.25), _date(rng, 1, 73050, 0.25))


def _month_end(rng: random.Random, year: int) -> float:
    month = rng.randint(1, 12)
    last = calendar.monthrange(year, month)[1]
    return _serial_of(year, month, rng.choice((last, last - 1, 30, 1) if last >= 30 else (last, last - 1, 1)))


def _tail(rng: random.Random) -> list[Arg]:
    """Pairs where the bases' rules turn: month ends (the 30/360 adjustments), February's last days,
    pairs about one year apart (basis 1's within-a-year cut) and short spans across 29 February."""
    year = rng.randint(1901, 2099)
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: _with_basis(rng, _month_end(rng, year), _month_end(rng, year + rng.randint(0, 3))),
        lambda: _with_basis(
            rng,
            _serial_of(year, 2, calendar.monthrange(year, 2)[1]),
            _serial_of(year + rng.randint(0, 4), 2, 28) + rng.choice((0.0, 1.0)),
        ),
        lambda: _year_apart(rng, year),
        lambda: _with_basis(
            rng,
            _serial_of(year, rng.randint(1, 3), rng.randint(1, 28)),
            _serial_of(year + rng.randint(0, 1), rng.randint(2, 4), rng.randint(1, 28)),
        ),
    ]
    return one_of(rng, shapes)


def _year_apart(rng: random.Random, year: int) -> list[Arg]:
    """A start and an end one year on, a day either side of it, with 29 February around."""
    start = datetime.date(year, rng.randint(1, 12), rng.randint(1, 28))
    end_year = start.year + 1
    try:
        end = start.replace(year=end_year)
    except ValueError:
        end = datetime.date(end_year, 3, 1)
    end += datetime.timedelta(days=rng.choice((-1, 0, 1)))
    return [float((start - EPOCH).days), float((end - EPOCH).days), 1]


def _edge(rng: random.Random) -> list[Arg]:
    """Serial 0 and the early 1900 days, the last date, dates outside the calendar (`#NUM!`), bases
    outside 0..4 (`#NUM!`) and equal dates."""
    early = rng.choice((0.0, 0.5, 0.9999, 1.0, 31.0, 59.0, 60.0, 60.5, 61.0))
    other = rng.choice((0.0, 59.0, 60.0, 61.0, 366.0, 425.0, 45000.0, float(LAST_DAY)))
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: _with_basis(rng, early, other),
        lambda: _with_basis(rng, other, early),
        lambda: _with_basis(rng, float(LAST_DAY), _date(rng, 1, LAST_DAY)),
        lambda: _with_basis(rng, rng.choice((-1.0, -1e-10, 2958466.0, 3e6)), _date(rng, 1, 73050)),
        lambda: _with_basis(rng, _date(rng, 1, 73050), rng.choice((-1.0, -0.5, 2958466.0, 1e10))),
        lambda: [_date(rng, 1, 73050), _date(rng, 1, 73050), rng.choice((-1, 5, 6, 100))],
        lambda: _with_basis(rng, early, early),
    ]
    return one_of(rng, shapes)


def _extreme(rng: random.Random) -> list[Arg]:
    """Dates over the whole calendar, 1900 to 9999, one in two with a time of day."""
    return _with_basis(rng, _date(rng, 0, LAST_DAY, 0.5), _date(rng, 0, LAST_DAY, 0.5))


def _log_uniform(rng: random.Random) -> list[Arg]:
    """Both dates log-uniform in [1E-3, 2958465], cut to 15 digits (fractions of a day below 0.99)."""

    def date() -> float:
        x = log_uniform(rng, 1e-3, float(LAST_DAY))
        return round15(x) if x - x // 1 < 0.99 else float(x // 1)

    return _with_basis(rng, date(), date())


YEARFRAC = register(
    Definition(
        function="YEARFRAC",
        params=(
            Param("start_date", "real", Interval(), source=ZERO_SOURCE),
            Param("end_date", "real", Interval(), source=ZERO_SOURCE),
            Param("basis", "int", Interval(), source=SWAP_SOURCE, optional=True),
        ),
        value=_value,
        domain=domain_of(_value),
        draw=by_stratum(
            {
                "typical": _typical,
                "tail": _tail,
                "edge": _edge,
                "extreme": _extreme,
                "log-uniform": _log_uniform,
            },
            "YEARFRAC draws: date pairs by stratum on every basis, month ends, year-apart pairs and serial 0.",
        ),
        serials_1900=True,
        sources=(DECISION, ZERO_SOURCE, SWAP_SOURCE, EARLY_SOURCE, BASIS1_SOURCE),
    )
)
