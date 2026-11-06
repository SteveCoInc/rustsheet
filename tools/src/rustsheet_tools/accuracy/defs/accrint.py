"""ACCRINT (Gate A, T-4114, decision 0273).

ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method]) is par·rate/frequency times
a sum of period fractions over a coupon schedule anchored on first_interest and stepped by 12/frequency months (each
date from the one before it, its day clamped to the month, or the month's last day when first_interest is one):
- the settlement's period: with calc_method TRUE (the default) and the settlement after first_interest, the coupon
  date before the settlement on the schedule stepped forward from first_interest, else the regular period that ends on
  first_interest. It adds days(max(issue, its start), settlement) over the period length E (360/frequency on bases 0,
  2 and 4, 365/frequency on basis 3, and on basis 1 the serial days of the regular period that ends on first_interest);
- every earlier period down to the issue: one that starts on or after the issue adds 1 with calc_method TRUE and 0
  with FALSE; the issue's own period adds days(issue, its end) over its length (bases 0 and 2: US 30/360
  moving both dates, basis 3: 365/frequency, bases 1 and 4: as days).
The days are the basis day counts of defs/yearfrac.py on Excel's serials (US 30/360 for basis 0, European for basis 4,
serial days otherwise). The math is that sum, exact, times the exact doubles of rate and par. Sources: the engine's
`compute_accrint` (crates/rustsheet-engine/src/functions/financial/accrint.rs), the port this definition checks
against Excel, and the financial.md entries cited below for the schedule's ends. It departs from the engine in
basis 2's issue-period length, which Excel counts as basis 0 does (`AMOUNTS_SOURCE`; T-4566 owns the engine).

Domains (`DomainError`, Excel's `#NUM!`), each evidenced:
- a date that does not floor to 0..2958465 ("ACCRINT check order: calc_method first, dates as read"; a settlement past
  it: probes/date-system-1900-accrint-settlement-bound A1-A20), an issue on or after the settlement, a rate at most 0
  or a negative par (`AMOUNTS_SOURCE`), a negative basis ("Day-count basis: a negative basis is #NUM!");
- with calc_method FALSE or the settlement not after first_interest, a coupon date the accrual reads before
  1900-01-01 ("ACCRINT at the calendar's start"): a period start on the walk back to the issue, the settlement's period
  start, and on basis 1 the regular period's start. With calc_method TRUE and the settlement after first_interest such
  a point has no math value: Excel for the web gave 0 there on 2026-11-06 (probes/accrint-zero-results A1:A9, A12,
  A14, A16:A19) where earlier fills gave `#NUM!`;
- with calc_method TRUE and the settlement after first_interest, a next coupon date past 9999-12-31 on bases 0 and 1
  ("ACCRINT with calc_method TRUE: a next coupon past the calendar is #NUM!"; bases 2 to 4 are unprobed there).
No math value (`NoValueError`): a par of 0, a frequency other than 1, 2 or 4 and a basis from 5 (unprobed), a date or
coupon date before 1900-03-01 that is not before 1900-01-01 (Excel's serials 0 to 60 and the fictitious 1900-02-29,
which this definition does not model), and a sum that cancels (`ill_conditioned`).
"""

from __future__ import annotations

import datetime
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    by_stratum_with_edge,
    fraction_to_mpf,
    ill_conditioned,
    one_of,
    scalars,
)
from rustsheet_tools.accuracy.defs.clock import LAST_DAY
from rustsheet_tools.accuracy.defs.yearfrac import (
    displayed_date,
    eu_30_360,
    serial_of,
    us_30_360,
    us_30_360_both,
    whole_serial,
)
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.kernels.solvers import add_months, month_end
from rustsheet_tools.fitlab.sweep import EPOCH

ENGINE_SOURCE = "crates/rustsheet-engine/src/functions/financial/accrint.rs (compute_accrint)"
START_SOURCE = "financial.md#accrint-at-the-calendars-start"
END_SOURCE = "financial.md#accrint-with-calc_method-true-a-next-coupon-past-the-calendar-is-num"
ORDER_SOURCE = "financial.md#accrint-check-order-calc_method-first-dates-as-read-t-2676"
BASIS_SOURCE = "financial.md#day-count-basis-a-negative-basis-is-num"
TYPES_SOURCE = "financial.md#vdb-ddb-and-accrint-argument-types"
AMOUNTS_SOURCE = "financial.md#accrint-accrintm-pricemat-and-yieldmat-amount-domains-t-4114"

FIRST_DAY = datetime.date(1900, 1, 1)
FIRST_MODELLED = datetime.date(1900, 3, 1)  # serial 61: from here on a serial counts real days from EPOCH
FIRST_MODELLED_SERIAL = 61


def _num(why: str) -> DomainError:
    return DomainError(f"{why}: #NUM!")


def _serial(date: datetime.date, *, walked: bool) -> int:
    """The serial of a coupon date the accrual reads: `#NUM!` before 1900-01-01, no value before 1900-03-01. With
    `walked` (calc_method TRUE, settlement after first_interest) a date before 1900-01-01 has no value either: Excel
    for the web has given both 0 and `#NUM!` there (`START_SOURCE`)."""
    if date < FIRST_DAY:
        if walked:
            msg = f"a coupon date {date} before 1900-01-01 after a forward walk (Excel gives 0 or #NUM!)"
            raise NoValueError(msg)
        raise _num(f"a coupon date {date} before 1900-01-01")
    if date < FIRST_MODELLED:
        msg = f"a coupon date {date} in January or February 1900 (not modelled)"
        raise NoValueError(msg)
    return (date - EPOCH).days


def _days(start: int, end: int, basis: int) -> int:
    """The basis day count from serial `start` to serial `end` (both from 61)."""
    if basis == 0:
        return us_30_360(displayed_date(start), displayed_date(end))
    if basis == 4:
        return eu_30_360(displayed_date(start), displayed_date(end))
    return end - start


def _issue_period(start: int, end: int, basis: int, frequency: int) -> Fraction:
    """The length of the issue's period from `start` to `end`."""
    if basis in (
        0,
        2,
    ):  # basis 2 too, not plain US 30/360: probes/accrint-zero-results C3, C4, C9, C10, C15, C16
        return Fraction(us_30_360_both(displayed_date(start), displayed_date(end)))
    if basis == 3:
        return Fraction(365, frequency)
    return Fraction(_days(start, end, basis))


def _settlement_period(
    first: datetime.date, settlement: int, months: int, eom: bool, basis: int
) -> datetime.date:
    """The coupon date before the settlement on the schedule stepped forward from `first`, `#NUM!` (no value on
    bases 2 to 4) when the date after it is past 9999-12-31."""
    previous = nxt = first
    while (nxt - EPOCH).days < settlement:
        previous = nxt
        try:
            nxt = add_months(nxt, months, eom)
        except ValueError:
            if basis in (0, 1):
                raise _num("a next coupon date past 9999-12-31") from None
            msg = "a next coupon date past 9999-12-31 on bases 2 to 4 (unprobed)"
            raise NoValueError(msg) from None
    return previous


def accrued_periods(dates: tuple[int, int, int], frequency: int, basis: int, *, calc: bool) -> Fraction:
    """ACCRINT's sum of period fractions (module docstring) for serials (issue, first_interest, settlement) from 61,
    the issue before the settlement."""
    issue, first, settlement = dates
    months = 12 // frequency
    first_date = EPOCH + datetime.timedelta(days=first)
    eom = month_end(first_date)
    regular = add_months(first_date, -months, eom)
    walk_forward = settlement > first and calc
    if basis == 1:
        length = Fraction(first - _serial(regular, walked=walk_forward))
    else:
        length = Fraction(360 if basis != 3 else 365, frequency)
    start_date = _settlement_period(first_date, settlement, months, eom, basis) if walk_forward else regular
    start = _serial(start_date, walked=walk_forward)
    terms = [Fraction(_days(max(issue, start), settlement, basis)) / length]
    front, end = start_date, start
    while end > issue:
        front = add_months(front, -months, eom)
        begin = _serial(front, walked=walk_forward)
        if issue <= begin:
            terms.append(Fraction(int(calc)))
        else:
            terms.append(Fraction(_days(issue, end, basis)) / _issue_period(begin, end, basis, frequency))
        end = begin
    total = sum(terms, Fraction(0))
    if total != 0:
        ill_conditioned(float(sum(abs(t) for t in terms) / abs(total)))
    return total


def _accrint(args: Sequence[MathArg]) -> mpmath.mpf:
    calc = args[7] if len(args) > 7 else True
    assert isinstance(calc, bool)
    numbers = scalars(args[:7])
    issue, first = whole_serial(float(numbers[0])), whole_serial(float(numbers[1]))
    settlement = whole_serial(float(numbers[2]))
    rate, par, frequency = numbers[3], numbers[4], int(numbers[5])
    basis = int(numbers[6]) if len(numbers) > 6 else 0
    if basis < 0:
        raise _num("a negative basis")
    if issue >= settlement:
        raise _num("issue on or after settlement")
    if rate <= 0 or par < 0:
        raise _num("a rate at most 0 or a negative par")
    unprobed = [
        (par == 0, "a par of 0"),
        (frequency not in (1, 2, 4), f"frequency {frequency}"),
        (basis > 4, f"basis {basis}"),
        (min(issue, first, settlement) < FIRST_MODELLED_SERIAL, "a date before 1900-03-01, not modelled"),
    ]
    for hit, why in unprobed:
        if hit:
            msg = f"{why} (unprobed)"
            raise NoValueError(msg)
    total = accrued_periods((issue, first, settlement), frequency, basis, calc=calc)
    return par * rate / frequency * fraction_to_mpf(total)


# -- draws -------------------------------------------------------------------------------------------------------

MODERN = (32874, 58440)  # 1990-01-01 .. 2059-12-31
Shape = Callable[[random.Random], list[Arg]]


def _first_interest(rng: random.Random, issue: int, frequency: int, stratum: str) -> float:
    """A first coupon date: within the issue's first period or a few periods on, on a day from 1 to 28, a month end, or
    (in the tail) on the 29th to 31st or before the issue."""
    offset = rng.randint(1, 366 // frequency * rng.choice((1, 1, 1, 2, 4)))
    when = EPOCH + datetime.timedelta(days=min(LAST_DAY, issue + offset))
    if stratum == "tail":
        if rng.random() < 0.2:
            when -= datetime.timedelta(days=rng.randint(1, 400))
        last = (add_months(when.replace(day=1), 0, eom=True)).day
        return serial_of(when.year, when.month, min(rng.choice((29, 30, 31, last)), last))
    if rng.random() < 0.3:
        return float((add_months(when, 0, eom=True) - EPOCH).days)
    return serial_of(when.year, when.month, rng.randint(1, 28))


def _settlement(rng: random.Random, issue: int, first: float, stratum: str, last: int) -> float:
    """A settlement after the issue: up to 30 years on, near first_interest in the tail."""
    if stratum == "tail" and rng.random() < 0.5:
        return min(float(last), max(float(issue + 1), first + rng.choice((-1, 0, 1, 2, rng.randint(1, 400)))))
    return float(min(last, issue + int(log_uniform(rng, 1, 10958))))


def _optional(rng: random.Random, point: list[Arg]) -> list[Arg]:
    """`point` with basis and calc_method, basis only, or neither (bases 0..4, calc_method TRUE or FALSE)."""
    r = rng.random()
    if r < 1 / 6:
        return point
    if r < 1 / 2:
        return [*point, rng.randrange(5)]
    return [*point, rng.randrange(5), rng.random() < 0.5]


def _amounts(rng: random.Random, stratum: str) -> tuple[float, float]:
    """(rate, par): wide in the extreme and log-uniform strata."""
    if stratum in ("extreme", "log-uniform"):
        return round15(log_uniform(rng, 1e-12, 1e3)), round15(log_uniform(rng, 1e-6, 1e100))
    return round15(rng.uniform(0.001, 0.25)), round15(log_uniform(rng, 1, 1e6))


def _point(rng: random.Random, stratum: str) -> list[Arg]:
    frequency = rng.choice((1, 2, 4))
    if stratum == "extreme":
        issue = rng.randint(367, LAST_DAY - 1)  # 1901-01-01 on
        if rng.random() < 0.3:
            issue = LAST_DAY - rng.randint(1, 3000)  # the calendar's end
    else:
        issue = rng.randint(*MODERN)
    first = min(float(LAST_DAY), _first_interest(rng, issue, frequency, stratum))
    settlement = _settlement(rng, issue, first, stratum, LAST_DAY)
    return _optional(rng, [float(issue), first, settlement, *_amounts(rng, stratum), frequency])


def _edge(rng: random.Random) -> list[Arg]:
    """Errors and the schedule's ends: dates off the calendar, issue at or after settlement, a rate of 0, a negative
    basis, the calendar's start (probes/accrint-calendar-start-1900) and end (probes/date-system-1900-accrint-
    settlement-bound)."""
    base = _point(rng, "typical")
    issue = float(rng.randint(*MODERN))
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [rng.choice((-1.0, 2958466.0)), *base[1:]],
        lambda: [base[0], rng.choice((-1.0, 2958466.0)), *base[2:]],
        lambda: [issue, base[1], rng.choice((-1.0, issue, issue - rng.randint(1, 400))), *base[3:]],
        lambda: [*base[:3], 0.0, *base[4:]],
        lambda: [*base[:6], rng.choice((-1, -0.5)), *base[7:]],
        lambda: [
            61.0,
            rng.choice((182.0, 91.0, 183.0, 367.0)),
            float(rng.choice((150, 400, 18263, 45000))),
            0.05,
            1000.0,
            rng.choice((1, 2, 4)),
            rng.randrange(5),
            rng.random() < 0.5,
        ],
        lambda: [
            2956276.0,
            rng.choice((2956456.0, 2956454.0)),
            float(rng.choice((2958282, 2958283, 2958400, 2958463, 2958465))),
            0.05,
            1000.0,
            rng.choice((1, 2, 4)),
            rng.choice((0, 1)),
            True,
        ],
    ]
    return one_of(rng, shapes)


ACCRINT = register(
    Definition(
        function="ACCRINT",
        params=(
            Param("issue", "real", source=ORDER_SOURCE),
            Param("first_interest", "real", source=ORDER_SOURCE),
            Param("settlement", "real", source=ORDER_SOURCE),
            Param("rate", "real", source=AMOUNTS_SOURCE),
            Param("par", "real", source=AMOUNTS_SOURCE),
            Param("frequency", "int", source=ORDER_SOURCE),
            Param("basis", "int", source=BASIS_SOURCE, optional=True),
            Param("calc_method", "bool", source=TYPES_SOURCE, optional=True),
        ),
        value=_accrint,
        draw=by_stratum_with_edge(
            lambda stratum, rng: _point(rng, stratum),
            _edge,
            "ACCRINT: (issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method]) by stratum.",
        ),
        serials_1900=True,
        sources=(
            ENGINE_SOURCE,
            START_SOURCE,
            END_SOURCE,
            ORDER_SOURCE,
            BASIS_SOURCE,
            TYPES_SOURCE,
            AMOUNTS_SOURCE,
        ),
    )
)
