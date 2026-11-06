"""ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD (Gate A, T-4115, decision 0273).

Excel's odd-period formulas are those of Microsoft's ODDFPRICE and ODDLPRICE pages, with the schedules and day counts
the engine's port of ExcelFinancialFunctions' oddbonds.fs walks (`OddFirstSchedule`, `odd_l_func`), which match Excel
bit-exact on the eval cells (financial.md "PRICE and ODDFPRICE discount with Excel's `^`"). The math is exact on the
double arguments, discounting by q^-t, q = 1 + yld/f, c = 100·rate/f. The day counts are bonds.py's (30/360 on basis 0
and 4, actual days on bases 1-3, each clamped at 0 as oddbonds.fs' `DaysBetween` is), E is PRICE's E of the
quasi-coupon period around settlement on the first coupon's schedule (360/f, 365/f on basis 3, its actual days on
basis 1; `bonds.coupons`).

ODDFPRICE (settlement, maturity, issue, first_coupon): DFC = days from issue to the first coupon.
- Short first period (DFC < E): N the coupons from settlement to maturity on the maturity's schedule, DSC the days
  from settlement to the first coupon, A from issue to settlement, s = DSC/E:
  red·q^-(N-1+s) + c·DFC/E·q^-s + Σ_{k=2..N} c·q^-(k-1+s) - c·A/E.
- Long first period (DFC > E): the NC quasi-coupon periods from issue to the first coupon are walked back from the
  first coupon a period at a time, each date clamped from the one after it (no month-end rule); period i has length
  NL_i (its actual days on basis 1, else E), DC_i its days (NL_i, the first one's from issue) and A_i its days before
  settlement; DCNL = Σ DC_i/NL_i, ANL = Σ A_i/NL_i. DSC is E less the days from the settlement's previous quasi-coupon
  date (bases 0, 1 and 4) or the days to its next one (bases 2 and 3); Nq is oddbonds.fs' `coupNumber`, which moves
  the settlement to its month's end when the first coupon is a month end and so can count one quasi period more
  (financial.md "ODDFYIELD stops short of ODDFPRICE's root", T-3988: ODDFPRICE's own exponents); n the coupons after
  the first; s = DSC/E: red·q^-(s+Nq+n) + c·DCNL·q^-(Nq+s) + Σ_{k=1..n} c·q^-(k+Nq+s) - c·ANL.
- ODDFYIELD is the root of ODDFPRICE(yld) = pr ("ODDFYIELD stops short of ODDFPRICE's root": Excel's Newton,
  its slope not ODDFPRICE's, lands 3e-12 to 3e-8 relative from that root). Every flow is positive and comes at
  t >= 0 after the dirty price pr + accrued at t = 0, so the root is unique (`annuity.unique_rate`); a point with a
  flow before t = 0 gets no value. Where Excel's Newton fails ("ODDFYIELD and ODDLYIELD at a price far from par",
  "ODDFYIELD's #NUM! at extreme prices") Excel's `#NUM!` or iterate is the target and the math the root.

ODDLPRICE and ODDLYIELD (settlement, maturity, last_interest): the NC quasi-coupon periods from last_interest to the
maturity (NC counted on the maturity's schedule) are walked forward a period at a time from last_interest, each date
clamped from the one before it ("Coupon dates keep the maturity's day": EXCEPTION, the ODDL* walk drifts); NL_i is
period i's length and DC_i its days (the last one's to the maturity), both on US 30/360 with both dates modified on
basis 0 (`yearfrac.us_30_360_both`); A_i its days before settlement and DSC_i its days from settlement to the maturity
on the basis. DCNL, ANL and DSCNL are the sums over NL_i. ODDLPRICE = (c·DCNL + red)/(1 + DSCNL·yld/f) - c·ANL, and
ODDLYIELD is its closed-form inverse ((c·DCNL + red) - (c·ANL + pr))/(c·ANL + pr) · f/DSCNL (Microsoft's ODDLYIELD
page; "ODDFYIELD and ODDLYIELD at a price far from par": ODDLYIELD is closed form).

Domains (Excel's `#NUM!`, `DomainError`), each evidenced: a date that does not floor to 0..2958465, a frequency other
than 1, 2 or 4, a basis outside 0..4, a negative rate, a negative yld, a pr of 0 or below, a redemption of 0 or below
("ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order"); for ODDF* anything but issue < settlement < first_coupon
< maturity ("ODDFPRICE and ODDFYIELD need issue < settlement < first_coupon"), a first coupon off the maturity's
schedule or DFC = E on bases 0, 1 and 4 or basis 2 semiannually ("ODDFPRICE and ODDFYIELD need the first coupon on
the maturity's schedule", and the table's annual cells); for ODDL* anything but
last_interest < settlement < maturity, and on bases 0 and 1 a quasi-coupon date past 9999-12-31 ("ODDLPRICE and
ODDLYIELD: a quasi-coupon date past the calendar is #NUM!"); ODDFPRICE's q^(redemption's exponent) past the double
range ("ODDFPRICE and ODDLPRICE at a huge yield").
Not drawn and given no math value (`NoValueError`), as no evidence names Excel's result there: a date before 1901-03-01
(the oddbonds.fs walks read real dates, Excel's 1900 calendar counts a fictitious 29 February; unprobed); a walk past
9999-12-31 for ODDF*, or on bases 2-4 for ODDL*; ODDFPRICE's power within 1E-9 (in its logarithm) of the double range's
top; 100·rate, a dirty sum or a price past the double range (unprobed for the odd periods); ODDLYIELD with DSCNL = 0;
DFC = E on basis 2 annually or quarterly or on basis 3 annually; an ODDL* quasi period of 0 days (unreachable).
"""

from __future__ import annotations

import calendar
import datetime
import math
import random
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import fraction_to_mpf, real_args
from rustsheet_tools.accuracy.defs.annuity import unique_rate
from rustsheet_tools.accuracy.defs.bonds import (
    CARET_SOURCE,
    DAY_SOURCE,
    FREQUENCIES,
    VALUE_EDGES,
    YEAR,
    bond_definition,
    coupons,
    date_serial,
    ln_or_minus_inf,
    past_double_top,
    price_or_par,
    value_edge,
)
from rustsheet_tools.accuracy.defs.clock import LAST_DAY
from rustsheet_tools.accuracy.defs.yearfrac import (
    basis_days,
    displayed_date,
    real_date,
    serial_of,
    us_30_360_both,
    whole_serial,
)
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import DomainError, MathArg, NoValueError
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.kernels.solvers import add_months, month_end, schedule

ORDER_SOURCE = "financial.md#oddfprice-oddfyield-oddlprice-and-oddlyield-check-order"
FIRST_ORDER_SOURCE = "financial.md#oddfprice-and-oddfyield-need-issue--settlement--first_coupon"
FIRST_SCHEDULE_SOURCE = "financial.md#oddfprice-and-oddfyield-need-the-first-coupon-on-the-maturitys-schedule"
LAST_CALENDAR_SOURCE = "financial.md#oddlprice-and-oddlyield-a-quasi-coupon-date-past-the-calendar-is-num"
HUGE_SOURCE = "financial.md#oddfprice-and-oddlprice-at-a-huge-yield"
FAR_SOURCE = "financial.md#oddfyield-and-oddlyield-at-a-price-far-from-par"
SHORT_SOURCE = "financial.md#oddfyield-stops-short-of-oddfprices-root"
EXTREME_SOURCE = "financial.md#oddfyields-num-at-extreme-prices"
WORK_SOURCE = "financial.md#yield-and-odd-coupon-work-cap"
ENGINE_SOURCE = (
    "crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (OddFirstSchedule, coup_number) and "
    "financial_bonds/odd_last.rs (odd_l_func): ports of ExcelFinancialFunctions oddbonds.fs"
)

EARLIEST = int(serial_of(1901, 3, 1))  # dates before it get no value (module docstring)


@dataclass(frozen=True)
class OddArgs:
    """A call's checked arguments: its date serials, rate, yld or pr, redemption, frequency and basis."""

    dates: tuple[int, ...]
    rate: mpmath.mpf
    fourth: mpmath.mpf
    redemption: mpmath.mpf
    frequency: int
    basis: int


def odd_args(args: Sequence[MathArg], n_dates: int, *, price_given: bool) -> OddArgs:
    """The arguments read and checked ("ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order": every check here
    is `#NUM!`); the date order is the caller's."""
    a = real_args(args)
    dates = tuple(whole_serial(float(x)) for x in a[:n_dates])
    rate, fourth, red, f = a[n_dates : n_dates + 4]
    rest = a[n_dates + 4 :]
    if int(f) not in FREQUENCIES:
        msg = f"frequency {f} is not 1, 2 or 4"
        raise DomainError(msg)
    basis = int(rest[0]) if rest else 0
    if not 0 <= basis <= 4:
        msg = f"basis {basis} is outside 0..4"
        raise DomainError(msg)
    if rate < 0 or red <= 0 or (fourth <= 0 if price_given else fourth < 0):
        msg = "a negative rate or yld, a pr or redemption of 0 or below"
        raise DomainError(msg)
    return OddArgs(dates, rate, fourth, red, int(f), basis)


def _early(dates: Sequence[int]) -> None:
    if min(dates) < EARLIEST:
        msg = "a date before 1901-03-01 (the 1900 calendar's reading is unprobed)"
        raise NoValueError(msg)


def _days(start: int, end: int, basis: int) -> Fraction:
    """oddbonds.fs' `DaysBetween`, clamped at 0."""
    return max(Fraction(0), basis_days(start, end, basis))


def _step(d: datetime.date, months: int) -> datetime.date:
    """`d` moved `months` months, its day clamped; NoValueError past 9999 (a walk off the calendar)."""
    try:
        return add_months(d, months, eom=False)
    except ValueError:
        msg = "a walk past 9999-12-31"
        raise NoValueError(msg) from None


@dataclass(frozen=True)
class Flows:
    """A price's flows: runs of `count` equal `amount`s at exponents t, t + 1, ... (in periods), less `accrued`;
    `front` is the redemption's exponent."""

    runs: tuple[tuple[mpmath.mpf, mpmath.mpf, int], ...]
    accrued: mpmath.mpf
    front: mpmath.mpf

    def dirty(self, u: mpmath.mpf) -> mpmath.mpf:
        """Σ amount·e^(-t·u) over every flow, u = ln q."""
        total = mpmath.mpf(0)
        for amount, t, count in self.runs:
            if count == 0 or amount == 0:
                continue
            g = mpmath.mpf(count) if u == 0 else mpmath.expm1(-count * u) / mpmath.expm1(-u)
            total += amount * mpmath.exp(-t * u) * g
        return total

    def amounts(self) -> list[mpmath.mpf]:
        """The flows in time order (every run's exponents are increasing in the order the runs are listed)."""
        return [
            amount for amount, _, count in sorted(self.runs, key=lambda r: r[1]) for _ in range(min(count, 1))
        ]


def coup_number(anchor: datetime.date, settlement: datetime.date, months: int) -> int:
    """oddbonds.fs' `coupNumber` (whole number): the quasi periods from settlement to the anchor, walking forward from
    the settlement moved to its month's end under the month-end rule (the anchor a month end, or a 29th or 30th that
    is not one outside February with the settlement a month end)."""
    last = calendar.monthrange(anchor.year, anchor.month)[1]
    eom = month_end(anchor)
    if not eom and anchor.month != 2 and 28 < anchor.day < last:
        eom = month_end(settlement)
    try:
        start = add_months(settlement, 0, eom)
        count = int(settlement < start)
        front = add_months(start, months, eom)
        while front < anchor:
            count += 1
            front = add_months(front, months, eom)
    except ValueError:
        msg = "a walk past 9999-12-31"
        raise NoValueError(msg) from None
    return count


def odd_first_flows(o: OddArgs) -> Flows:
    """ODDFPRICE's flows (module docstring), after the ODDF* date checks."""
    settlement, maturity, issue, first = o.dates
    if not issue < settlement < first < maturity:
        msg = "not issue < settlement < first_coupon < maturity: #NUM!"
        raise DomainError(msg)
    _early(o.dates)
    f, basis, months = o.frequency, o.basis, 12 // o.frequency
    if schedule(real_date(first - 1), real_date(maturity), f)[1] != real_date(first):
        msg = "the first coupon is off the maturity's schedule: #NUM!"
        raise DomainError(msg)
    around = coupons(settlement, first, f, basis)
    e = around.e
    dfc = _days(issue, first, basis)
    if dfc == e:
        # evidenced on bases 0, 1 and 4 at every frequency (the entry's semiannual and quarterly cells, the table's
        # annual ones: accuracy/ODDFPRICE/s1 A126, accuracy/ODDFYIELD/s1 A169, A168) and on basis 2 semiannually;
        # basis 2 annual or quarterly and basis 3 annual (E = 365, the only basis-3 period a whole DFC can equal) are
        # unprobed
        if basis in (0, 1, 4) or (basis == 2 and f == 2):
            msg = "a first period of exactly one period (DFC = E): #NUM!"
            raise DomainError(msg)
        msg = "a first period of exactly one period (DFC = E) on an unprobed basis and frequency"
        raise NoValueError(msg)
    c = 100 * o.rate / f
    if dfc < e:
        n = schedule(real_date(settlement), real_date(maturity), f)[2]
        s = fraction_to_mpf(_days(settlement, first, basis) / e)
        runs = ((o.redemption, n - 1 + s, 1), (c * fraction_to_mpf(dfc / e), s, 1), (c, 1 + s, n - 1))
        return Flows(runs, c * fraction_to_mpf(_days(issue, settlement, basis) / e), n - 1 + s)
    nc = schedule(real_date(issue), real_date(first), f)[2]
    late = real_date(first)
    dcnl, anl = Fraction(0), Fraction(0)
    for index in range(nc, 0, -1):
        early = _step(late, -months)
        early_s, late_s = date_serial(early), date_serial(late)
        nl = _days(early_s, late_s, basis) if basis == 1 else e
        dci = nl if index > 1 else _days(issue, late_s, basis)
        dcnl += dci / nl
        anl += _days(max(issue, early_s), min(settlement, late_s), basis) / nl
        late = early
    dsc = (
        _days(settlement, around.ncd, basis)
        if basis in (2, 3)
        else e - basis_days(around.pcd, settlement, basis)
    )
    nq = coup_number(real_date(first), real_date(settlement), months)
    n = schedule(real_date(first), real_date(maturity), f)[2]
    s = fraction_to_mpf(dsc / e)
    runs = ((o.redemption, s + nq + n, 1), (c * fraction_to_mpf(dcnl), nq + s, 1), (c, 1 + nq + s, n))
    return Flows(runs, c * fraction_to_mpf(anl), s + nq + n)


def _finite_flows(o: OddArgs, fl: Flows, u: mpmath.mpf) -> mpmath.mpf:
    """The dirty sum at u; NoValueError past the double range or for an infinite 100·rate (unprobed)."""
    if past_double_top(ln_or_minus_inf(100 * o.rate), "100·rate"):
        msg = "100·rate past the double range (unprobed)"
        raise NoValueError(msg)
    dirty = fl.dirty(u)
    if past_double_top(ln_or_minus_inf(dirty), "the dirty sum"):
        msg = "the odd-period flows' sum past the double range (unprobed)"
        raise NoValueError(msg)
    return dirty


def oddfprice(o: OddArgs) -> mpmath.mpf:
    """ODDFPRICE at yld = `o.fourth` (module docstring)."""
    fl = odd_first_flows(o)
    u = mpmath.log1p(o.fourth / o.frequency)
    if past_double_top(fl.front * u, "(1 + yld/f)^t"):
        msg = "(1 + yld/f)^t past the double range at the redemption's exponent: #NUM!"
        raise DomainError(msg)
    value = _finite_flows(o, fl, u) - fl.accrued
    if past_double_top(ln_or_minus_inf(abs(value)), "ODDFPRICE"):
        msg = "ODDFPRICE past the double range (unprobed)"
        raise NoValueError(msg)
    return value


def oddfyield(o: OddArgs) -> mpmath.mpf:
    """ODDFYIELD: ODDFPRICE's unique root at pr = `o.fourth` (module docstring)."""
    fl = odd_first_flows(o)
    if any(t < 0 for amount, t, count in fl.runs if amount != 0 and count > 0):
        msg = "a flow before the dirty price: no evidenced root choice"
        raise NoValueError(msg)
    _finite_flows(o, fl, mpmath.mpf(0))
    dirty_price = o.fourth + fl.accrued
    return o.frequency * unique_rate([-dirty_price, *fl.amounts()], lambda u: fl.dirty(u) - dirty_price)


@dataclass(frozen=True)
class OddLast:
    """ODDL*'s sums over the quasi periods: DCNL, ANL and DSCNL."""

    dcnl: Fraction
    anl: Fraction
    dscnl: Fraction


def _hack_days(start: int, end: int, basis: int) -> Fraction:
    """`_days`, but basis 0 counts US 30/360 with both dates modified (ODDL*'s period lengths)."""
    if basis == 0:
        return max(Fraction(0), Fraction(us_30_360_both(displayed_date(start), displayed_date(end))))
    return _days(start, end, basis)


def odd_last_sums(o: OddArgs) -> OddLast:
    """ODDL*'s walk (module docstring), after the ODDL* date checks."""
    settlement, maturity, last = o.dates
    if not last < settlement < maturity:
        msg = "not last_interest < settlement < maturity: #NUM!"
        raise DomainError(msg)
    _early(o.dates)
    basis, months = o.basis, 12 // o.frequency
    nc = schedule(real_date(last), real_date(maturity), o.frequency)[2]
    early = real_date(last)
    dcnl, anl, dscnl = Fraction(0), Fraction(0), Fraction(0)
    for index in range(1, nc + 1):
        try:
            late = add_months(early, months, eom=False)
        except ValueError:
            if basis in (0, 1):
                msg = "a quasi-coupon date past 9999-12-31: #NUM!"
                raise DomainError(msg) from None
            msg = "a quasi-coupon date past 9999-12-31 on bases 2-4 (unprobed)"
            raise NoValueError(msg) from None
        early_s, late_s = date_serial(early), date_serial(late)
        nl = _hack_days(early_s, late_s, basis)
        if nl == 0:  # a whole period of 0 days: unreachable on a real calendar, and unprobed
            msg = "a quasi-coupon period of 0 days"
            raise NoValueError(msg)
        dci = nl if index < nc else _hack_days(early_s, maturity, basis)
        if late_s < settlement:
            a = dci
        elif early_s < settlement:
            a = _days(early_s, settlement, basis)
        else:
            a = Fraction(0)
        dcnl += dci / nl
        anl += a / nl
        dscnl += _days(max(settlement, early_s), min(maturity, late_s), basis) / nl
        early = late
    return OddLast(dcnl, anl, dscnl)


def _odd_last_terms(o: OddArgs, sums: OddLast) -> tuple[mpmath.mpf, mpmath.mpf]:
    """(c·DCNL + red, c·ANL); NoValueError for an infinite 100·rate or a term past the double range (unprobed)."""
    c = 100 * o.rate / o.frequency
    term1 = c * fraction_to_mpf(sums.dcnl) + o.redemption
    for x, what in ((100 * o.rate, "100·rate"), (term1, "c·DCNL + red")):
        if past_double_top(ln_or_minus_inf(x), what):
            msg = f"{what} past the double range (unprobed)"
            raise NoValueError(msg)
    return term1, c * fraction_to_mpf(sums.anl)


def oddlprice(o: OddArgs) -> mpmath.mpf:
    """ODDLPRICE (module docstring)."""
    sums = odd_last_sums(o)
    term1, accrued = _odd_last_terms(o, sums)
    return term1 / (1 + fraction_to_mpf(sums.dscnl) * o.fourth / o.frequency) - accrued


def oddlyield(o: OddArgs) -> mpmath.mpf:
    """ODDLYIELD (module docstring)."""
    sums = odd_last_sums(o)
    if sums.dscnl == 0:
        msg = "DSCNL is 0 (unprobed)"
        raise NoValueError(msg)
    term1, accrued = _odd_last_terms(o, sums)
    dirty = accrued + o.fourth
    return (term1 - dirty) / dirty * o.frequency / fraction_to_mpf(sums.dscnl)


def _oddfprice(args: Sequence[MathArg]) -> mpmath.mpf:
    return oddfprice(odd_args(args, 4, price_given=False))


def _oddfyield(args: Sequence[MathArg]) -> mpmath.mpf:
    return oddfyield(odd_args(args, 4, price_given=True))


def _oddlprice(args: Sequence[MathArg]) -> mpmath.mpf:
    return oddlprice(odd_args(args, 3, price_given=False))


def _oddlyield(args: Sequence[MathArg]) -> mpmath.mpf:
    return oddlyield(odd_args(args, 3, price_given=True))


# -- draws -------------------------------------------------------------------------------------------------------------

MODERN = (32874, 51501)  # 1990-01-01 to 2040-12-31
WIDE = (int(serial_of(1902, 1, 1)), LAST_DAY - 800)
MAX_LN_GROWTH = 600.0


@dataclass
class Draft:
    """A bond being drawn: the dates (settlement, maturity, then issue and first coupon, or last interest), rate, yld
    or pr, redemption, frequency and basis (None: left out)."""

    dates: list[float]
    rate: float
    fourth: float
    redemption: float
    frequency: int
    basis: int | None

    def args(self) -> list[Arg]:
        out: list[Arg] = [*self.dates, self.rate, self.fourth, self.redemption, self.frequency]
        if self.basis is not None:
            out.append(self.basis)
        return out

    def checked(self, n_dates: int) -> OddArgs | None:
        """The draft as the math reads it, or None outside the domain."""
        try:
            return odd_args([mpmath.mpf(a) for a in self.args()], n_dates, price_given=False)
        except DomainError:
            return None


def _basis(rng: random.Random) -> int | None:
    """A basis 0..4, left out one time in six."""
    return None if rng.random() < 1 / 6 else rng.randrange(5)


def _values(stratum: str, rng: random.Random) -> tuple[float, float, float]:
    """(rate, yld, redemption) per stratum."""
    if stratum == "extreme":
        return log_uniform(rng, 1e-6, 5.0), log_uniform(rng, 1e-6, 5.0), log_uniform(rng, 1e-2, 1e6)
    if stratum == "log-uniform":
        return log_uniform(rng, 1e-8, 2.0), log_uniform(rng, 1e-8, 2.0), log_uniform(rng, 1, 1e4)
    rate = 0.0 if rng.random() < 0.05 else rng.uniform(0.0, 0.15)
    redemption = 100.0 if rng.random() < 0.6 else rng.uniform(90, 110)
    return rate, log_uniform(rng, 1e-3, 0.2), redemption


def _month_end_date(rng: random.Random, year: int) -> datetime.date:
    """A month end, a 29th or 30th, or the day before a month end."""
    month = rng.randint(1, 12)
    last = calendar.monthrange(year, month)[1]
    return datetime.date(year, month, rng.choice((last, min(last, 29), min(last, 30), last - 1)))


def _first_base(stratum: str, rng: random.Random) -> Draft:
    """An ODDF* bond: a maturity, a first coupon whole periods before it on its schedule, an issue a short (under a
    period) or long (over one) first period before the first coupon, a settlement between issue and first coupon."""
    rate, yld, redemption = _values(stratum, rng)
    f = rng.choice(FREQUENCIES)
    period = YEAR / f
    lo, hi = WIDE if stratum == "extreme" else MODERN
    start = rng.randint(lo, hi)
    years = log_uniform(rng, 0.1, 30) if stratum != "extreme" else log_uniform(rng, 0.1, 100)
    first_length = rng.uniform(0.02, 0.98) if rng.random() < 0.5 else rng.uniform(1.02, 4.0)
    shape = rng.randrange(6) if stratum == "tail" else -1
    if shape == 0:  # one coupon left after the first (a short bond)
        years = 1 / f
    elif shape == 1:  # long bonds
        years = rng.uniform(30, 100)
    elif shape == 2:  # yields near 0 or at 0
        yld = 0.0 if rng.random() < 0.2 else log_uniform(rng, 1e-10, 1e-4)
    elif shape == 3:  # high coupons and yields
        rate, yld = rng.uniform(0.15, 1.0), rng.uniform(0.15, 1.0)
    elif shape == 4:  # very short or very long first periods
        first_length = rng.choice((rng.uniform(0.005, 0.05), rng.uniform(4.0, 12.0)))
    maturity_serial = min(LAST_DAY - 1, start + max(1, round(years * YEAR)))
    maturity = real_date(maturity_serial)
    if shape == 5 or rng.random() < 0.15:  # month-end, 29th and 30th maturities (the month-end rule, Nq)
        maturity = _month_end_date(rng, maturity.year)
    periods_after = max(1, round(years * f))
    first = add_months(maturity, -(12 // f) * periods_after, month_end(maturity))
    first_s = date_serial(first)
    issue = first_s - max(2, round(first_length * period))
    settlement = rng.randint(issue + 1, first_s - 1)
    if (
        stratum == "tail" and rng.random() < 0.3
    ):  # settlement a day after issue or a day before the first coupon
        settlement = rng.choice((issue + 1, first_s - 1))
    # keep (1 + yld/f)^(f·years) inside the double range: past it ODDFPRICE is #NUM!, which the edge stratum covers
    terms = (maturity_serial - settlement) / period + 1
    if math.log1p(yld / f) * terms > MAX_LN_GROWTH:
        yld = f * math.expm1(MAX_LN_GROWTH / terms) * rng.uniform(0.1, 1.0)
    return Draft([settlement, date_serial(maturity), issue, first_s], rate, yld, redemption, f, _basis(rng))


def _first_edge(rng: random.Random, *, price_given: bool) -> Draft:
    """ODDF* domain edges (each `#NUM!`): the date order broken one way at a time, a first coupon a day off the
    maturity's schedule, a first period of exactly one period (DFC = E), a negative rate, a negative yld or a pr of
    0, a redemption of 0 or below, frequency 3, basis 5 or -1, a date past the calendar, and for ODDFPRICE yields
    whose power overflows (and below it, a number)."""
    d = _first_base("typical", rng)
    _, maturity, issue, first = (int(x) for x in d.dates)
    shape = rng.randrange(12)
    if shape == 0:  # settlement on or before the issue
        d.dates[0] = issue - rng.choice((0, 1, 400))
    elif shape == 1:  # settlement on the first coupon or after it
        d.dates[0] = rng.choice((first, first + 1, (first + maturity) // 2))
    elif shape == 2:  # first coupon on the maturity
        d.dates[3] = maturity
    elif shape == 3:  # a first coupon a day off the schedule
        d.dates[3] = first + rng.choice((-1, 1))
    elif shape == 4:  # issue exactly a period before the first coupon (DFC = E on bases 0, 1 and 4)
        months = 12 // d.frequency
        d.dates[2] = date_serial(add_months(real_date(first), -months, month_end(real_date(maturity))))
        d.dates[0] = rng.randint(int(d.dates[2]) + 1, first - 1)
        d.basis = rng.choice((0, 1, 4))
    elif shape < 5 + VALUE_EDGES:
        value_edge(d, shape - 5, rng, price_given=price_given)
    elif shape == 10:  # a date off the calendar
        d.dates[rng.randrange(4)] = rng.choice((-1.0, 2958466.0))
    elif not price_given:  # huge yields: the redemption's power overflows past about 1E+15 for 10-year bonds
        d.fourth = rng.choice((1e8, 1e15, 1e16, 1e17, 1e100, 1e300))
    return d


def _price_given(d: Draft, price: Callable[[OddArgs], mpmath.mpf], n_dates: int, rng: random.Random) -> None:
    """Replaces the drawn yld by the price at it (cut to 15 digits by the sampler): the math is then the yield at that
    price. A price outside 1E-3 to 1E4 of the redemption is replaced by one within half the redemption of it."""
    pr = math.nan
    o = d.checked(n_dates)
    if o is not None and d.fourth > -d.frequency:
        o = OddArgs(o.dates, o.rate, mpmath.mpf(d.fourth), o.redemption, o.frequency, o.basis)
        with mpmath.workdps(30):
            try:
                pr = float(price(o))
            except (DomainError, NoValueError):
                pr = math.nan
    d.fourth = price_or_par(pr, d.redemption, rng)


def _oddfprice_draw(stratum: str, rng: random.Random) -> list[Arg]:
    d = _first_edge(rng, price_given=False) if stratum == "edge" else _first_base(stratum, rng)
    return d.args()


def _oddfyield_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """A bond priced at a drawn yld, which keeps the price from 1E-3 to 1E4 of the redemption: Excel's Newton is
    `#NUM!` at a price of 1E5 and more ("ODDFYIELD and ODDLYIELD at a price far from par"); the tail stratum adds
    negative yields."""
    if stratum == "edge":
        d = _first_edge(rng, price_given=True)
        if rng.random() < 0.5:
            return d.args()
    else:
        d = _first_base(stratum, rng)
        if stratum == "tail" and rng.random() < 0.3:
            d.fourth = -rng.uniform(0.001, 0.2)
    _price_given(d, oddfprice, 4, rng)
    return d.args()


def _last_base(stratum: str, rng: random.Random) -> Draft:
    """An ODDL* bond: a last interest date, an odd last period of under one or a few periods to the maturity, a
    settlement inside it."""
    rate, yld, redemption = _values(stratum, rng)
    f = rng.choice(FREQUENCIES)
    period = YEAR / f
    lo, hi = WIDE if stratum == "extreme" else MODERN
    last = rng.randint(lo, hi)
    length = rng.uniform(0.02, 0.98) if rng.random() < 0.5 else rng.uniform(1.02, 4.0)
    if stratum == "tail":
        shape = rng.randrange(4)
        if shape == 0:  # last interest on a month end, a 29th or 30th (the drifting walk)
            last = date_serial(_month_end_date(rng, real_date(last).year))
        elif shape == 1:  # a maturity on a month end
            maturity = _month_end_date(rng, real_date(last + round(length * period)).year)
            length = max(2, date_serial(maturity) - last) / period
        elif shape == 2:  # high coupons and yields
            rate, yld = rng.uniform(0.15, 1.0), rng.uniform(0.15, 1.0)
        else:  # very short and long odd periods
            length = rng.choice((rng.uniform(0.005, 0.05), rng.uniform(4.0, 12.0)))
    elif (
        stratum == "extreme" and rng.random() < 0.2
    ):  # the calendar's last year (bases 2-4 past it: no value)
        last = rng.randint(LAST_DAY - 400, LAST_DAY - 10)
    maturity_s = min(LAST_DAY, last + max(2, round(length * period)))
    settlement = rng.randint(last + 1, maturity_s - 1)
    if stratum == "tail" and rng.random() < 0.3:
        settlement = rng.choice((last + 1, maturity_s - 1))
    return Draft([settlement, maturity_s, last], rate, yld, redemption, f, _basis(rng))


def _last_edge(rng: random.Random, *, price_given: bool) -> Draft:
    """ODDL* domain edges (each `#NUM!` but the huge yields): settlement on or before the last interest, on or after
    the maturity, a last interest on the maturity, a negative rate, a negative yld or a pr of 0, a redemption of 0 or
    below, frequency 3, basis 5 or -1, a date past the calendar, a quasi-coupon date past 9999-12-31 on bases 0 and 1,
    and for ODDLPRICE huge yields (a number: it is linear in the yield)."""
    d = _last_base("typical", rng)
    _, maturity, last = (int(x) for x in d.dates)
    shape = rng.randrange(11)
    if shape == 0:
        d.dates[0] = last - rng.choice((0, 1, 400))
    elif shape == 1:
        d.dates[0] = maturity + rng.choice((0, 1))
    elif shape == 2:
        d.dates[2] = maturity + rng.choice((0, 1))
    elif shape < 3 + VALUE_EDGES:
        value_edge(d, shape - 3, rng, price_given=price_given)
    elif shape == 8:
        d.dates[rng.randrange(3)] = rng.choice((-1.0, 2958466.0))
    elif shape == 9:  # a quasi-coupon date past 9999-12-31 (`ODDLPRICE(DATE(9999,8,1),DATE(9999,12,31),...)`)
        d.dates = [
            LAST_DAY - rng.randint(20, 140),
            LAST_DAY - rng.randint(0, 15),
            LAST_DAY - rng.randint(150, 230),
        ]
        d.frequency, d.basis = rng.choice((2, 4)), rng.choice((0, 1))
    elif not price_given:
        d.fourth = rng.choice((1e15, 1e100, 1e300, 9.99999999999999e307))
    return d


def _oddlprice_draw(stratum: str, rng: random.Random) -> list[Arg]:
    d = _last_edge(rng, price_given=False) if stratum == "edge" else _last_base(stratum, rng)
    return d.args()


def _oddlyield_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """A bond priced at a drawn yld (ODDLYIELD is closed form: every price is a value, "ODDFYIELD and ODDLYIELD at a
    price far from par"); the tail stratum adds negative yields, the extreme one prices from 1E-6 to 1E8."""
    if stratum == "edge":
        d = _last_edge(rng, price_given=True)
        if rng.random() < 0.5:
            return d.args()
    else:
        d = _last_base(stratum, rng)
        if stratum == "tail" and rng.random() < 0.3:
            d.fourth = -rng.uniform(0.001, 0.2)
    _price_given(d, oddlprice, 3, rng)
    if stratum == "extreme" and rng.random() < 0.3:
        d.fourth = log_uniform(rng, 1e-6, 1e8)
    return d.args()


_FIRST = ("settlement", "maturity", "issue", "first_coupon", "rate")
_LAST = ("settlement", "maturity", "last_interest", "rate")
_COMMON = (ORDER_SOURCE, DAY_SOURCE, WORK_SOURCE, ENGINE_SOURCE)  # ORDER_SOURCE first: the parameters cite it
_FIRST_SOURCES = (FIRST_ORDER_SOURCE, FIRST_SCHEDULE_SOURCE, CARET_SOURCE)

ODDFPRICE = bond_definition(
    "ODDFPRICE",
    (*_FIRST, "yld", "redemption", "frequency", "basis"),
    _oddfprice,
    _oddfprice_draw,
    (*_COMMON, *_FIRST_SOURCES, HUGE_SOURCE),
)
ODDFYIELD = bond_definition(
    "ODDFYIELD",
    (*_FIRST, "pr", "redemption", "frequency", "basis"),
    _oddfyield,
    _oddfyield_draw,
    (*_COMMON, *_FIRST_SOURCES, SHORT_SOURCE, FAR_SOURCE, EXTREME_SOURCE),
    solver=True,
)
ODDLPRICE = bond_definition(
    "ODDLPRICE",
    (*_LAST, "yld", "redemption", "frequency", "basis"),
    _oddlprice,
    _oddlprice_draw,
    (*_COMMON, LAST_CALENDAR_SOURCE, HUGE_SOURCE),
)
ODDLYIELD = bond_definition(
    "ODDLYIELD",
    (*_LAST, "pr", "redemption", "frequency", "basis"),
    _oddlyield,
    _oddlyield_draw,
    (*_COMMON, LAST_CALENDAR_SOURCE, FAR_SOURCE),
)
