"""AMORLINC and AMORDEGRC (Gate A, T-4111, decision 0273).

The math is the documented algorithm (the one `ExcelFinancialFunctions` ports and the engine's amortization.rs keeps) at
the working precision, on the doubles of the written arguments, with Excel's rounding steps applied where Excel
applies them:
- Dates floor to whole serials; serial s is the day `EPOCH + s` (serial 0 is the day before serial 1, financial.md
  "AMORLINC and AMORDEGRC check order": `AMORDEGRC(2400,0,366,300,1,0.15,1)` is 562).
- The first period's length counts US 30/360 days on basis 0, European 30/360 on basis 4 (defs/yearfrac.py's counts) and
  actual days on bases 1 and 3, a late February day of a leap year read as 28 February there; the year has 360 days
  on bases 0 and 4, 365 on basis 3 and the purchase year's length on basis 1. The first charge is
  days/year·rate·cost, the whole cost·rate when that is 0, and at most cost - salvage.
- The asset life is `ceil(1/rate)` on Excel's double quotient (a step Excel applies); past it a period charges 0.
- AMORLINC: period 0 is the first charge; whole period n charges cost·rate while n·cost·rate fits in what is left
  after the first charge, then the remainder once, then 0 ("AMORLINC and AMORDEGRC over many periods": a period
  truncates, and one below 1 charges a full period). Excel's walk subtracts in doubles, so the remainder period carries
  n roundings: it is redrawn when n·ulp of the available amount is past `CONDITION_LIMIT` of the remainder.
- AMORDEGRC: the rate is scaled by the coefficient of the whole life, 1.5 up to 4, 2 up to 6, 2.5 above ("AMORDEGRC
  life bands"); the first charge (period 0) is rounded like ROUND(x, 0) ("AMORDEGRC rounds its charge like ROUND": the
  15-digit cut of a charge of 1E15 or more, otherwise half away from zero), and from what it leaves each period charges
  rate·remain, the period asset_life - 3 half the remain and the next one the rest (asset_life one more when the first
  charge was prorated), nothing once the remain is below the salvage; the charge returned is rounded the same way.
  Excel's twice-rounded walk is its rounding, not a step: the math walks exactly. A charge within
  `(n + 2)·1e-14` relative of a rounding half, or a remain that close to the salvage, has no value (Excel's walk may
  fall on the other side).
Domains (Excel's errors, all `#NUM!`, "AMORLINC and AMORDEGRC check order"): a date that does not floor to
0..2958465, a negative cost or salvage, a salvage above the cost, a negative period, a rate of 0 or less, a purchase
after the first period, a basis but 0, 1, 3 or 4 after truncation (a negative one too, "Day-count basis: a negative
basis is #NUM!"); for AMORDEGRC an asset life 1/rate of 2 or less ("AMORDEGRC life bands").
A salvage equal to the cost is not `#NUM!` but 0 from period 1 on (this table's cells, recorded in that entry); below
period 1 it is unprobed.
No math value (`NoValueError`): AMORDEGRC at a period strictly between 0 and 1, which returns the value the last
AMORDEGRC or AMORLINC Excel ran left behind (workbook state, "AMORDEGRC below period 1"); a 30/360 basis with a date
below serial 61 (the stand-in calendar before 1900-03-01 is probed on basis 1 only); AMORLINC at a period below 1 whose
full charge does not fit in what is left (unprobed: the engine returns the full charge); periods past 1E6 (Excel's walk
time; "AMORLINC and AMORDEGRC over many periods" times out at 1E15).
"""

from __future__ import annotations

import calendar
import datetime
import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import ill_conditioned, one_of, scalars, trunc
from rustsheet_tools.accuracy.defs.depreciation import stratified
from rustsheet_tools.accuracy.defs.yearfrac import displayed_date, eu_30_360, us_30_360, whole_serial
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    MathArg,
    NoValueError,
    Param,
    register,
)
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.sweep import EPOCH

ORDER_SOURCE = "financial.md#amorlinc-and-amordegrc-check-order"
MANY_SOURCE = "financial.md#amorlinc-and-amordegrc-over-many-periods"
BELOW_ONE_SOURCE = "financial.md#amordegrc-below-period-1"
PAST_SOURCE = "financial.md#amordegrc-past-222-periods"
ROUND_SOURCE = "financial.md#amordegrc-rounds-its-charge-like-round"
BANDS_SOURCE = "financial.md#amordegrc-life-bands"
T1625_SOURCE = "financial.md#t-1625-probes-vdb-amordegrc-fvschedule-mduration-rri"
BASIS_SOURCE = "financial.md#day-count-basis-a-negative-basis-is-num"
DECISION = "decision 0273"

FIRST_REAL_DAY = 61  # 1900-03-01: from here a serial's displayed date is its real date
MAX_PERIOD = 1e6
HALF = mpmath.mpf(1) / 2


def _date(serial: int) -> datetime.date:
    return EPOCH + datetime.timedelta(days=serial)


def _feb_fix(day: datetime.date, basis: int) -> datetime.date:
    """`fix29February`: on bases 1 and 3 a 28 or 29 February of a leap year reads as 28 February."""
    if basis in (1, 3) and calendar.isleap(day.year) and day.month == 2 and day.day >= 28:
        return day.replace(day=28)
    return day


def first_days(purchased: int, first: int, basis: int) -> tuple[int, int]:
    """(the first period's days, the year's days) of whole serials on basis 0, 1, 3 or 4."""
    if basis in (0, 4):
        if min(purchased, first) < FIRST_REAL_DAY:
            msg = "a 30/360 date below serial 61 (unprobed)"
            raise NoValueError(msg)
        count = us_30_360 if basis == 0 else eu_30_360
        return count(displayed_date(purchased), displayed_date(first)), 360
    start, end = _feb_fix(_date(purchased), basis), _feb_fix(_date(first), basis)
    year = (366 if calendar.isleap(_date(purchased).year) else 365) if basis == 1 else 365
    return (end - start).days, year


def amor_round(x: mpmath.mpf, periods: int) -> mpmath.mpf:
    """ROUND(x, 0) of a charge: the 15-digit cut from 1E15, else half away from zero; NoValueError within
    (periods + 2)·1e-14 relative of a half, where Excel's walk may round the other way."""
    if x == 0:
        return x
    if abs(x) >= mpmath.mpf(10) ** 15:
        unit = mpmath.mpf(10) ** (int(mpmath.floor(mpmath.log10(abs(x)))) - 14)
        return mpmath.nint(x / unit) * unit
    whole = mpmath.floor(abs(x))
    if abs(abs(x) - whole - HALF) <= abs(x) * (periods + 2) * mpmath.mpf("1e-14"):
        msg = "a charge at a rounding half"
        raise NoValueError(msg)
    return mpmath.sign(x) * (whole + (1 if abs(x) - whole > HALF else 0))


class _Asset:
    """Checked arguments (module docstring)."""

    def __init__(self, args: Sequence[MathArg], *, degressive: bool) -> None:
        a = scalars(args)
        self.cost, self.salvage, self.period, self.rate = a[0], a[3], a[4], a[5]
        self.fully_salvaged = False
        purchased, first = whole_serial(float(a[1])), whole_serial(float(a[2]))
        if not (self.cost >= 0 and self.salvage >= 0 and self.salvage <= self.cost):
            msg = "a negative cost or salvage, or a salvage above the cost: #NUM!"
            raise DomainError(msg)
        if self.period < 0 or self.rate <= 0 or purchased > first:
            msg = "a negative period, a rate at most 0 or a purchase after the first period: #NUM!"
            raise DomainError(msg)
        basis_arg = a[6] if len(a) > 6 else mpmath.mpf(0)
        if not 0 <= basis_arg < 5 or trunc(basis_arg) == 2:
            msg = "a basis other than 0, 1, 3 or 4: #NUM!"
            raise DomainError(msg)
        self.basis = trunc(basis_arg)
        quotient = 1.0 / float(self.rate)  # Excel's double 1/rate
        if degressive and 0 <= quotient <= 2:
            msg = "an asset life of 2 or less: #NUM!"
            raise DomainError(msg)
        if self.period > MAX_PERIOD:
            msg = "a period past 1E6 (Excel's walk time)"
            raise NoValueError(msg)
        if self.salvage == self.cost:
            # 0 in Excel at the periods probed, 1 to 5 (accuracy/AMORDEGRC/s1 A114, A118, A128, A130, A131,
            # accuracy/AMORLINC/s1 A124, A125; financial.md "AMORLINC and AMORDEGRC check order")
            if self.period < 1:
                msg = "a salvage equal to the cost below period 1 (unprobed)"
                raise NoValueError(msg)
            self.fully_salvaged = True
        self.life = math.ceil(quotient)
        self.purchased, self.first = purchased, first

    def first_charge(self, rate: mpmath.mpf) -> tuple[mpmath.mpf, bool]:
        """(the first period's charge, whether it was prorated) at `rate`."""
        days, year = first_days(self.purchased, self.first, self.basis)
        prorated = mpmath.mpf(days) / year * rate * self.cost
        charge = prorated if prorated != 0 else self.cost * rate
        return min(charge, self.cost - self.salvage), prorated != 0


def _amorlinc(args: Sequence[MathArg]) -> mpmath.mpf:
    asset = _Asset(args, degressive=False)
    if asset.period > asset.life or asset.fully_salvaged:
        return mpmath.mpf(0)
    first, _ = asset.first_charge(asset.rate)
    if asset.period == 0:
        return first
    charge = asset.rate * asset.cost
    available = asset.cost - asset.salvage - first
    n = trunc(asset.period)
    if n == 0:  # a period below 1 charges a full period
        if charge > available:
            msg = "a period below 1 whose full charge does not fit (unprobed)"
            raise NoValueError(msg)
        return charge
    ratio = available / charge
    for k in (n, n - 1):
        if abs(ratio - k) <= (k + 1) * (ratio + 1) * mpmath.mpf(
            "1e-15"
        ):  # k roundings of the available amount
            msg = "the available amount at a whole number of charges"
            raise NoValueError(msg)
    if n * charge <= available:
        return charge
    rest = available - (n - 1) * charge
    if rest <= 0:
        return mpmath.mpf(0)
    ill_conditioned(float(available / rest) * n)
    return rest


def _coefficient(life: int) -> mpmath.mpf:
    if life <= 4:
        return mpmath.mpf("1.5")
    if life <= 6:
        return mpmath.mpf(2)
    return mpmath.mpf("2.5")


def _near(x: mpmath.mpf, y: mpmath.mpf, periods: int) -> bool:
    return bool(abs(x - y) <= (abs(x) + abs(y)) * (periods + 2) * mpmath.mpf("1e-14"))


def _amordegrc(args: Sequence[MathArg]) -> mpmath.mpf:  # noqa: PLR0911 (one return per stage of the walk)
    asset = _Asset(args, degressive=True)
    if asset.period > asset.life or asset.fully_salvaged:
        return mpmath.mpf(0)
    rate = asset.rate * _coefficient(asset.life)
    first_exact, prorated = asset.first_charge(rate)
    first = amor_round(first_exact, 0)
    if asset.period == 0:
        return first
    if asset.period < 1:
        msg = "a period between 0 and 1 returns workbook state (AMORDEGRC below period 1)"
        raise NoValueError(msg)
    n = trunc(asset.period)
    life = asset.life + (1 if prorated else 0)
    half = life - 3 if life - 3 >= 1 else None
    remain = asset.cost - first
    salvage = asset.salvage

    def below_salvage(value: mpmath.mpf, steps: int) -> bool:
        if _near(value, salvage, steps):
            msg = "the remain at the salvage"
            raise NoValueError(msg)
        return bool(value < salvage)

    if half is None or n < half:
        before = remain * (1 - rate) ** (n - 1)  # the remain before period n
        if below_salvage(before, n):
            return mpmath.mpf(0)
        return amor_round(rate * before, n)
    before_half = remain * (1 - rate) ** (half - 1)
    if below_salvage(before_half, half):
        return mpmath.mpf(0)
    if n == half:
        return amor_round(before_half / 2, n)
    rest = before_half / 2
    if n == half + 1 and not below_salvage(rest, half + 1):
        return amor_round(rest, n)
    return mpmath.mpf(0)


# -- draws -------------------------------------------------------------------------------------------

TYPICAL_FIRST = 36526  # 2000-01-01
TYPICAL_LAST = 54789  # 2049-12-31


def _with_basis(rng: random.Random, args: list[Arg]) -> list[Arg]:
    if rng.random() < 1 / 6:
        return args
    return [*args, rng.choice((0, 1, 3, 4))]


def _dates(rng: random.Random, lo: int = TYPICAL_FIRST, hi: int = TYPICAL_LAST) -> tuple[float, float]:
    """A purchase and a first period's end up to two years later (on it one time in ten)."""
    purchased = rng.randint(lo, hi)
    first = purchased if rng.random() < 0.1 else purchased + rng.randint(1, 730)
    return float(purchased), float(first)


def _month_end_dates(rng: random.Random) -> tuple[float, float]:
    """Dates where the day counts turn: month ends and February's last days, in leap years too."""
    year = rng.randint(2000, 2040)

    def serial(y: int, m: int, d: int) -> float:
        return float((datetime.date(y, m, d) - EPOCH).days)

    def month_end(y: int) -> float:
        m = rng.randint(1, 12)
        return serial(y, m, rng.choice((calendar.monthrange(y, m)[1], 30 if m != 2 else 28, 1)))

    purchased = month_end(year)
    first = max(purchased, month_end(year + rng.randint(0, 1)))
    return purchased, first


def _period(rng: random.Random, life: int, *, fraction: bool = True) -> float:
    n = rng.randint(0, life + 1)
    if fraction and n >= 1 and rng.random() < 0.2:
        return n + rng.choice((0.25, 0.5, 0.9))
    return float(n)


def _amor_draw(degressive: bool) -> Callable[[str, random.Random], list[Arg]]:
    top_rate = 0.49 if degressive else 0.9

    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        cost = log_uniform(rng, 100, 1e7)
        salvage = 0.0 if rng.random() < 0.15 else cost * log_uniform(rng, 1e-4, 0.5)
        rate = log_uniform(rng, 0.02, top_rate)
        purchased, first = _dates(rng)
        if stratum == "tail":
            if rng.random() < 0.5:
                purchased, first = _month_end_dates(rng)
            else:  # rates where the coefficient and the life turn
                rate = rng.choice((0.25, 0.2, 1 / 3, 1 / 4.5, 0.15, 0.1, 1 / 6, 1 / 2.01, 0.3, 0.4, 0.45))
        elif stratum == "extreme":
            cost = log_uniform(rng, 1e3, 1e300)
            salvage = 0.0 if rng.random() < 0.3 else cost * log_uniform(rng, 1e-12, 0.9)
            rate = log_uniform(rng, 1e-5, 0.02)
            purchased, first = _dates(rng, FIRST_REAL_DAY, 2958465 - 800)
        elif stratum == "log-uniform":
            cost = log_uniform(rng, 1, 1e12)
            salvage = cost * log_uniform(rng, 1e-6, 0.99)
            rate = log_uniform(rng, 1e-3, top_rate)
        life = math.ceil(1 / rate)
        if life > 2000:
            period: float = rng.choice(
                (float(rng.randint(0, 50)), float(rng.randint(0, 2000)), log_uniform(rng, 1, 1e5))
            )
            period = float(math.floor(period))
        else:
            period = _period(rng, life)
        return _with_basis(rng, [cost, purchased, first, salvage, period, rate])

    return draw


def _amor_edge(degressive: bool) -> Callable[[random.Random], list[Arg]]:
    def edge(rng: random.Random) -> list[Arg]:
        cost = log_uniform(rng, 100, 1e6)
        salvage = cost * log_uniform(rng, 1e-3, 0.3)
        rate = log_uniform(rng, 0.05, 0.45)
        purchased, first = _dates(rng)
        period = float(rng.randint(1, 5))
        shapes: list[Callable[[], list[Arg]]] = [
            lambda: [-cost, purchased, first, salvage, period, rate],  # negative cost: #NUM!
            lambda: [cost, purchased, first, -1.0, period, rate],  # negative salvage
            lambda: [cost, purchased, first, cost, period, rate],  # salvage equal to the cost: 0
            lambda: [cost, purchased, first, salvage, -1.0, rate],  # negative period
            lambda: [cost, purchased, first, salvage, period, rng.choice((0.0, -0.1))],  # rate at most 0
            lambda: [cost, first + 1, first, salvage, period, rate],  # purchase after the first period
            lambda: [cost, purchased, first, salvage, period, rate, rng.choice((2, 5, -1))],  # basis
            lambda: [
                cost,
                purchased,
                rng.choice((2958466.0, 3e6)),
                salvage,
                period,
                rate,
            ],  # past the calendar
            lambda: [cost, purchased, first, salvage, 0.0, rate, rng.choice((0, 1, 3, 4))],  # period 0
            lambda: [
                cost,
                first,
                first,
                salvage,
                period,
                rate,
                rng.choice((0, 1, 3, 4)),
            ],  # bought on its end
            lambda: [
                cost,
                purchased,
                first,
                salvage,
                float(math.ceil(1 / rate) + 1),
                rate,
            ],  # past the life: 0
            lambda: [cost, purchased, first, 0.0, period, rate],
            lambda: [cost, 0.0, rng.choice((366.0, 400.0)), salvage, period, rate, 1],  # serial 0
            lambda: [cost, purchased, first, salvage, period + 0.5, rate],  # a period truncates
        ]
        if degressive:
            shapes.append(
                lambda: [cost, purchased, first, salvage, period, rng.choice((0.5, 0.6, 1.0))]
            )  # life <= 2
        else:
            shapes.append(
                lambda: [cost, purchased, first, salvage, rng.choice((0.5, 0.25)), rate]
            )  # a full charge
        return one_of(rng, shapes)

    return edge


def _amor_definition(
    function: str,
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    sources: tuple[str, ...],
    *,
    degressive: bool,
) -> Definition:
    params = (
        Param("cost", "real", source=ORDER_SOURCE),
        Param("date_purchased", "real", source=ORDER_SOURCE),
        Param("first_period", "real", source=ORDER_SOURCE),
        Param("salvage", "real", source=ORDER_SOURCE),
        Param("period", "real", source=MANY_SOURCE),
        Param("rate", "real", source=ORDER_SOURCE),
        Param("basis", "int", source=BASIS_SOURCE, optional=True),
    )
    return register(
        Definition(
            function=function,
            params=params,
            value=value,
            draw=stratified(
                _amor_draw(degressive),
                _amor_edge(degressive),
                f"{function}: an asset, its dates and a period by stratum; the evidenced #NUM! edges.",
            ),
            serials_1900=True,
            sources=(DECISION, *sources),
        )
    )


AMORLINC = _amor_definition(
    "AMORLINC", _amorlinc, (ORDER_SOURCE, MANY_SOURCE, BASIS_SOURCE), degressive=False
)
AMORDEGRC = _amor_definition(
    "AMORDEGRC",
    _amordegrc,
    (
        ORDER_SOURCE,
        MANY_SOURCE,
        BELOW_ONE_SOURCE,
        PAST_SOURCE,
        ROUND_SOURCE,
        BANDS_SOURCE,
        T1625_SOURCE,
        BASIS_SOURCE,
    ),
    degressive=True,
)
