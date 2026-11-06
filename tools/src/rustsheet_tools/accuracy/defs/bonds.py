"""PRICE, YIELD, DURATION and MDURATION (Gate A, T-4113, decision 0273).

The coupon schedule (financial.md "Coupon dates keep the maturity's day", "Coupon schedules at the calendar's edges"):
both dates are floored to whole serials; each coupon date is the maturity moved back a whole number of periods
(12/frequency months), its day clamped to the month, or the month's last day when the maturity is a month end
(`fitlab.kernels.solvers.schedule`, read on the real calendar: serial 60, the fictitious 1900-02-29, as 1900-02-28).
PCD is the last coupon date on or before settlement, NCD the next one, N the coupons left. The schedule's dates read
as 1900 serials (1900-01-01 is 1; from 1900-03-01 on the fictitious day counts). The day counts are PRICE's
(financial.md "PRICE and ODDFPRICE discount with Excel's `^`", the engine's `CouponSchedule`): A is PCD to settlement
on the basis (US 30/360 on basis 0, European 30/360 on basis 4, serial days on bases 1-3, on Excel's displayed days:
`yearfrac.us_30_360`, `yearfrac.eu_30_360`), E is 360/f on bases 0, 2 and 4, 365/f on basis 3 and NCD - PCD on basis 1
("COUPDAYS basis 1 counts from the chained next coupon": PRICE and DURATION use COUPNCD - COUPPCD), and DSC = E - A on
every basis, so s = DSC/E.

The math, exact on the double arguments, with c = 100·rate/f and q = 1 + yld/f:
- PRICE (financial.md "PRICE, DURATION and MDURATION sum order at a huge coupon rate and yield", Microsoft's PRICE
  page): with N > 1, red·q^-(s+N-1) + Σ_{k=0..N-1} c·q^-(k+s) - c·A/E; with N = 1 the linear discount
  (red + c)/(1 + s·yld/f) - c·A/E. Excel's own operation order (`^`, the sum's order) is a few ulp off this, up to
  32 ulp at a huge coupon rate and yield.
- DURATION: the Macaulay duration of the same flows (coupon 100·rate/f at k + s, redemption 100 at s + N - 1), discounted
  by q^t on every N (the engine's `duration_of`), over f. With one coupon left it is s/f
  (`DURATION(47300,47484,0.05,1E+300,2,0)` is 0.5). MDURATION is DURATION/q.
- YIELD with one coupon period left (N = 1) is the documented closed form, not PRICE's root ("YIELD with one coupon
  period left is closed form"): ((red/100 + rate/f) - (pr/100 + A/E'·rate/f)) / (pr/100 + A/E'·rate/f) · (f·E'/DSR),
  E' = 360/f on bases 0 and 4 and NCD - PCD on bases 1-3, DSR the basis' days from settlement to maturity; DSR 0
  (settlement the 30th, maturity the 31st on bases 0 and 4) is 0.
- YIELD with N > 1 is the root of PRICE(yld) = pr. PRICE falls strictly with yld when s >= 0 (every flow is at t >= 0
  after the dirty price at t = 0: one sign change, so one root, Descartes' rule, `annuity.unique_rate`), from +∞ at
  yld = -f to -c·A/E + c·[s = 0] <= 0, so a positive price has exactly one root. Excel's Newton from its fitted start
  converges to it and stops short of it ("YIELD stops short of PRICE's root"): the stop is an absolute step test, so the
  relative gap is about 1e-9 at roots of a few percent and larger at small roots (5.4E-6 at accuracy/YIELD/s1 A252, a
  root of 5.7E-6; 6.8E-8 at A230, 4.2E-8 at A261). Where that Newton leaves PRICE's domain or runs 100 steps Excel's
  value is `#NUM!` or the 100th iterate ("YIELD's #NUM! cells and its 100-step cap"; "YIELD is #NUM! at a price and
  redemption far from par"): Excel's error is the target there, and the math the root.

Domains (Excel's `#NUM!`, `DomainError`), each evidenced:
- a date that does not floor to 0..2958465, a settlement below 1 or not before the maturity, a frequency other than
  1, 2 or 4, a basis outside 0..4, a negative rate, a negative yld (PRICE, DURATION, MDURATION), a price of 0 or below
  (YIELD), a redemption of 0 or below ("PRICE, YIELD, DURATION and MDURATION check order", "Coupon settlement before
  the calendar");
- a PCD before 1900-01-01 ("Coupon schedules at the calendar's edges": COUPNUM, PRICE, YIELD, DURATION and MDURATION
  are `#NUM!` there);
- PRICE, DURATION and MDURATION with N > 1: q^(s+N-1) past the double range, and a dirty sum (the discounted flows)
  past it ("PRICE, DURATION and MDURATION at a huge yield or coupon"; a coupon rate of 1E+306 at a 5% yield, 10-year
  bonds).
Not drawn and given no math value (`NoValueError`), as no evidence names Excel's result: a power or a dirty sum within
1E-9 (relative, in its logarithm) of the double range's top; with one coupon left, a dirty sum or a PRICE past the
double range (the evidence covers N > 1 only); 100·rate past the double range, or a DURATION weighted sum
past it, while the dirty sum is finite (Excel's intermediate is infinite); PRICE's linear discount at 0 (s < 0 on bases
2 and 3, where A can exceed E); YIELD with N > 1 and s < 0 (a flow before the dirty price: no evidenced root choice);
YIELD with one period left and a settlement before 1900-03-01 (whether DSR counts the fictitious day is unprobed).
"""

from __future__ import annotations

import calendar
import contextlib
import datetime
import math
import random
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from fractions import Fraction
from typing import Protocol

import mpmath

from rustsheet_tools.accuracy.defs._common import OVERFLOW_POINT, fraction_to_mpf, scalars
from rustsheet_tools.accuracy.defs.annuity import unique_rate
from rustsheet_tools.accuracy.defs.clock import LAST_DAY
from rustsheet_tools.accuracy.defs.yearfrac import (
    FICTITIOUS_DAY,
    basis_days,
    real_date,
    serial_of,
    whole_serial,
)
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.fitlab.kernels.solvers import schedule

ORDER_SOURCE = "financial.md#price-yield-duration-and-mduration-check-order"
CALENDAR_SOURCE = "financial.md#coupon-settlement-before-the-calendar"
EDGES_SOURCE = "financial.md#coupon-schedules-at-the-calendars-edges"
DAY_SOURCE = "financial.md#coupon-dates-keep-the-maturitys-day"
COUPDAYS_SOURCE = "financial.md#coupdays-basis-1-counts-from-the-chained-next-coupon"
CARET_SOURCE = "financial.md#price-and-oddfprice-discount-with-excels-"
SUM_SOURCE = "financial.md#price-duration-and-mduration-sum-order-at-a-huge-coupon-rate-and-yield"
HUGE_SOURCE = "financial.md#price-duration-and-mduration-at-a-huge-yield-or-coupon"
ONE_PERIOD_SOURCE = "financial.md#yield-with-one-coupon-period-left-is-closed-form"
SHORT_SOURCE = "financial.md#yield-stops-short-of-prices-root"
NEWTON_SOURCE = "financial.md#yields-num-cells-and-its-100-step-cap"
WORK_SOURCE = "financial.md#yield-and-odd-coupon-work-cap"
ENGINE_SOURCE = (
    "crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (CouponSchedule, duration_of)"
)

FREQUENCIES = (1, 2, 4)
CALENDAR_START = datetime.date(1900, 1, 1)
MARCH_1900 = datetime.date(1900, 3, 1)
LN_OVERFLOW = mpmath.log(OVERFLOW_POINT)
BAND = (
    1e-9  # a logarithm this close (relative) to the double range's top: Excel's double may be on either side
)


@dataclass(frozen=True)
class Coupons:
    """The schedule around settlement: coupons left `n`, PRICE's `a` and `e`, PCD and NCD as serials."""

    n: int
    a: Fraction
    e: Fraction
    pcd: int
    ncd: int

    @property
    def s(self) -> Fraction:
        """DSC/E = (E - A)/E."""
        return (self.e - self.a) / self.e


def date_serial(d: datetime.date) -> int:
    """A schedule date from 1900-01-01 on as Excel's 1900 serial (the fictitious 1900-02-29 counts from March)."""
    if d < MARCH_1900:
        return (d - datetime.date(1899, 12, 31)).days
    return int(serial_of(d.year, d.month, d.day))


def coupons(settlement: int, maturity: int, frequency: int, basis: int) -> Coupons:
    """The schedule around settlement (module docstring); DomainError for a PCD before 1900-01-01."""
    pcd_date, ncd_date, n = schedule(real_date(settlement), real_date(maturity), frequency)
    if pcd_date < CALENDAR_START:
        msg = "the previous coupon date is before 1900-01-01: #NUM!"
        raise DomainError(msg)
    pcd, ncd = date_serial(pcd_date), date_serial(ncd_date)
    a = basis_days(pcd, settlement, basis)
    e = {0: Fraction(360, frequency), 1: Fraction(ncd - pcd), 3: Fraction(365, frequency)}.get(
        basis, Fraction(360, frequency)
    )
    return Coupons(n, a, e, pcd, ncd)


@dataclass(frozen=True)
class Bond:
    """A call's checked arguments: settlement and maturity serials, rate, the fourth argument (yld or pr),
    redemption (100 for DURATION), frequency and basis."""

    settlement: int
    maturity: int
    rate: mpmath.mpf
    fourth: mpmath.mpf
    redemption: mpmath.mpf
    frequency: int
    basis: int


def bond(args: Sequence[MathArg], *, redemption: bool, price_given: bool) -> Bond:
    """The arguments read and checked in the domain order of "PRICE, YIELD, DURATION and MDURATION check order"
    (the error code is `#NUM!` for every one of these checks)."""
    a = scalars(args)
    if redemption:
        s, m, rate, fourth, red, f = a[:6]
        rest = a[6:]
    else:
        s, m, rate, fourth, f = a[:5]
        red, rest = mpmath.mpf(100), a[5:]
    basis = int(rest[0]) if rest else 0
    if not 0 <= basis <= 4:
        msg = f"basis {basis} is outside 0..4"
        raise DomainError(msg)
    settlement, maturity = whole_serial(float(s)), whole_serial(float(m))
    if int(f) not in FREQUENCIES:
        msg = f"frequency {f} is not 1, 2 or 4"
        raise DomainError(msg)
    if settlement < 1 or settlement >= maturity:
        msg = "settlement below 1 or not before the maturity"
        raise DomainError(msg)
    if rate < 0 or red <= 0 or (fourth <= 0 if price_given else fourth < 0):
        msg = "a negative rate or yld, a price or redemption of 0 or below"
        raise DomainError(msg)
    return Bond(settlement, maturity, rate, fourth, red, int(f), basis)


def past_double_top(ln_x: mpmath.mpf, what: str) -> bool:
    """Whether ln x is past ln of the double range's top; NoValueError within `BAND` of it."""
    if abs(ln_x - LN_OVERFLOW) <= BAND * LN_OVERFLOW:
        msg = f"{what} at the double range's top"
        raise NoValueError(msg)
    return bool(ln_x > LN_OVERFLOW)


def ln_or_minus_inf(x: mpmath.mpf) -> mpmath.mpf:
    """ln x, or -inf for x <= 0 (no value to overflow)."""
    return mpmath.log(x) if x > 0 else mpmath.mpf("-inf")


def _coupon(b: Bond) -> mpmath.mpf:
    """c = 100·rate/f."""
    return 100 * b.rate / b.frequency


def _sums(b: Bond, c: Coupons, cf: mpmath.mpf, weighted: bool) -> tuple[mpmath.mpf, mpmath.mpf, mpmath.mpf]:
    """(coupon sum, redemption term, weighted sum) of the compound discount at u = ln q: Σ_{k<N} cf·e^(-(k+s)u),
    red·e^(-(s+N-1)u) and, with `weighted`, Σ t·flow·e^(-t·u) over every flow."""
    s, n = fraction_to_mpf(c.s), c.n
    u = mpmath.log1p(b.fourth / b.frequency)
    big_t = s + n - 1
    if n > 1 and past_double_top(big_t * u, "(1 + yld/f)^(s+N-1)"):
        msg = "(1 + yld/f)^(s+N-1) past the double range: #NUM!"
        raise DomainError(msg)
    extra = 10 + (2 * int(-mpmath.log10(abs(u))) if u != 0 else 0) + 2 * len(str(n))
    with mpmath.extradps(max(extra, 0)):
        if u == 0:
            g, h = mpmath.mpf(n), mpmath.mpf(n) * (n - 1) / 2
        else:
            x = mpmath.exp(-u)
            g = -mpmath.expm1(-n * u) / -mpmath.expm1(-u)  # Σ x^k, k < N
            h = x * (1 - n * x ** (n - 1) + (n - 1) * x**n) / (1 - x) ** 2  # Σ k·x^k, k < N
        lead = mpmath.exp(-s * u)
        coupon_sum = cf * lead * g
        red_term = b.redemption * mpmath.exp(-big_t * u)
        weighted_sum = cf * lead * (s * g + h) + big_t * red_term if weighted else mpmath.mpf(0)
    return +coupon_sum, +red_term, +weighted_sum


def _dirty(coupon_sum: mpmath.mpf, red_term: mpmath.mpf, rate: mpmath.mpf, n: int) -> mpmath.mpf:
    """The dirty sum; past the double range DomainError with more than one coupon left (the evidence's 10-year bonds),
    NoValueError with one (unprobed: PRICE's one-period form is linear, DURATION's s/f); NoValueError for an infinite
    100·rate below it."""
    dirty = coupon_sum + red_term
    if past_double_top(ln_or_minus_inf(dirty), "the dirty sum"):
        if n == 1:
            msg = "the one-period flows' sum past the double range (unprobed)"
            raise NoValueError(msg)
        msg = "the discounted flows' sum past the double range: #NUM!"
        raise DomainError(msg)
    if past_double_top(ln_or_minus_inf(100 * rate), "100·rate"):
        msg = "100·rate past the double range (unprobed)"
        raise NoValueError(msg)
    return dirty


def price_of(b: Bond, c: Coupons) -> mpmath.mpf:
    """PRICE at yld = `b.fourth` (module docstring)."""
    cf = _coupon(b)
    accrued = cf * fraction_to_mpf(c.a / c.e)
    if c.n == 1:
        _dirty(cf, b.redemption, b.rate, 1)
        den = 1 + fraction_to_mpf(c.s) * b.fourth / b.frequency
        if den == 0:
            msg = "the linear discount is 0 (unprobed)"
            raise NoValueError(msg)
        value = (b.redemption + cf) / den - accrued
    else:
        coupon_sum, red_term, _ = _sums(b, c, cf, weighted=False)
        value = _dirty(coupon_sum, red_term, b.rate, c.n) - accrued
    if past_double_top(ln_or_minus_inf(abs(value)), "PRICE"):
        if c.n == 1:  # a near-zero linear discount: unprobed
            msg = "the one-period PRICE past the double range (unprobed)"
            raise NoValueError(msg)
        msg = "PRICE past the double range: #NUM!"
        raise DomainError(msg)
    return value


def duration_of(b: Bond, c: Coupons) -> mpmath.mpf:
    """DURATION (module docstring)."""
    coupon_sum, red_term, weighted = _sums(b, c, _coupon(b), weighted=True)
    dirty = _dirty(coupon_sum, red_term, b.rate, c.n)
    if past_double_top(ln_or_minus_inf(abs(weighted)), "the weighted sum"):
        msg = "the weighted sum past the double range while the dirty sum is not (unprobed)"
        raise NoValueError(msg)
    return weighted / dirty / b.frequency


def _price(args: Sequence[MathArg]) -> mpmath.mpf:
    b = bond(args, redemption=True, price_given=False)
    return price_of(b, coupons(b.settlement, b.maturity, b.frequency, b.basis))


def _duration(args: Sequence[MathArg]) -> mpmath.mpf:
    b = bond(args, redemption=False, price_given=False)
    return duration_of(b, coupons(b.settlement, b.maturity, b.frequency, b.basis))


def _mduration(args: Sequence[MathArg]) -> mpmath.mpf:
    b = bond(args, redemption=False, price_given=False)
    return duration_of(b, coupons(b.settlement, b.maturity, b.frequency, b.basis)) / (
        1 + b.fourth / b.frequency
    )


def yield_one_period(b: Bond, c: Coupons) -> mpmath.mpf:
    """YIELD's closed form with one coupon period left (module docstring)."""
    if b.settlement < FICTITIOUS_DAY + 1:
        msg = "one period left from a settlement before 1900-03-01 (unprobed)"
        raise NoValueError(msg)
    e = Fraction(c.ncd - c.pcd) if b.basis in (1, 2, 3) else c.e
    dsr = basis_days(b.settlement, b.maturity, b.basis)
    if dsr == 0:
        return mpmath.mpf(0)
    f = b.frequency
    accrued = fraction_to_mpf(c.a / e) * b.rate / f
    return (
        ((b.redemption / 100 + b.rate / f) - (b.fourth / 100 + accrued))
        / (b.fourth / 100 + accrued)
        * fraction_to_mpf(f * e / dsr)
    )


def yield_of(b: Bond, c: Coupons) -> mpmath.mpf:
    """YIELD (module docstring): the closed form with one period left, else PRICE's unique root."""
    if c.n == 1:
        return yield_one_period(b, c)
    if c.s < 0:
        msg = "a coupon before the dirty price (s < 0): no evidenced root choice"
        raise NoValueError(msg)
    cf = _coupon(b)
    dirty_price = b.fourth + cf * fraction_to_mpf(c.a / c.e)
    s, n, big_t = fraction_to_mpf(c.s), c.n, fraction_to_mpf(c.s) + c.n - 1

    def f(u: mpmath.mpf) -> mpmath.mpf:
        g = mpmath.mpf(n) if u == 0 else -mpmath.expm1(-n * u) / -mpmath.expm1(-u)
        return cf * mpmath.exp(-s * u) * g + b.redemption * mpmath.exp(-big_t * u) - dirty_price

    # the flows in time order: the dirty price at t = 0, the coupons, the redemption (one sign change)
    return b.frequency * unique_rate([-dirty_price, cf, b.redemption], f)


def _yield(args: Sequence[MathArg]) -> mpmath.mpf:
    b = bond(args, redemption=True, price_given=True)
    return yield_of(b, coupons(b.settlement, b.maturity, b.frequency, b.basis))


# -- draws -------------------------------------------------------------------------------------------------------------

YEAR = 365.2425
MODERN = (32874, 51501)  # 1990-01-01 to 2040-12-31
LONG_BOND_YEARS = 100
MAX_LN_GROWTH = 600.0


@dataclass
class Draft:
    """A bond being drawn: [settlement, maturity, rate, fourth, redemption, frequency, basis or None]."""

    settlement: float
    maturity: int
    rate: float
    fourth: float
    redemption: float
    frequency: int
    basis: int | None

    def args(self, *, with_redemption: bool) -> list[Arg]:
        out: list[Arg] = [self.settlement, self.maturity, self.rate, self.fourth]
        if with_redemption:
            out.append(self.redemption)
        out.append(self.frequency)
        if self.basis is not None:
            out.append(self.basis)
        return out


def _basis(rng: random.Random) -> int | None:
    """A basis 0..4, left out one time in six."""
    return None if rng.random() < 1 / 6 else rng.randrange(5)


def _term(rng: random.Random, settlement: int, years: float) -> int:
    return min(LAST_DAY, settlement + max(1, round(years * YEAR) + rng.randint(-20, 20)))


def _month_end_maturity(rng: random.Random, settlement: int, years: int) -> int:
    """A maturity on a month end, a 29th or 30th, or a 28/29 February, 1 to `years` years after settlement."""
    year = real_date(settlement).year + rng.randint(1, years)
    month = rng.randint(1, 12)
    last = calendar.monthrange(year, month)[1]
    day = rng.choice((last, min(last, 29), min(last, 30), last - 1))
    return date_serial(datetime.date(year, month, day))


def _base(stratum: str, rng: random.Random) -> Draft:
    """A bond per stratum; the yld is drawn here and turned into a price by YIELD's draw."""
    settlement = rng.randint(*MODERN)
    years = log_uniform(rng, 0.1, 30)
    rate = 0.0 if rng.random() < 0.05 else rng.uniform(0.0, 0.15)
    yld = log_uniform(rng, 1e-3, 0.2)
    redemption = 100.0 if rng.random() < 0.6 else rng.uniform(90, 110)
    frequency = rng.choice(FREQUENCIES)
    if stratum == "tail":
        shape = rng.randrange(5)
        if shape == 0:  # one coupon period left (YIELD's closed form, PRICE's linear discount)
            years = rng.uniform(0.003, 1 / frequency)
        elif shape == 1:  # month-end, 29th, 30th and February maturities
            maturity = _month_end_maturity(rng, settlement, 30)
            return Draft(settlement, maturity, rate, yld, redemption, frequency, _basis(rng))
        elif shape == 2:  # yields near 0 or at 0
            yld = 0.0 if rng.random() < 0.2 else log_uniform(rng, 1e-10, 1e-4)
        elif shape == 3:  # long bonds
            years = rng.uniform(30, LONG_BOND_YEARS)
        else:  # high coupons and yields
            rate, yld = rng.uniform(0.15, 1.0), rng.uniform(0.15, 1.0)
    elif stratum == "extreme":
        settlement = rng.randint(400, LAST_DAY - 400)
        years = log_uniform(rng, 0.01, (LAST_DAY - settlement) / YEAR)
        rate = log_uniform(rng, 1e-6, 5.0)
        yld = log_uniform(rng, 1e-6, 5.0)
        redemption = log_uniform(rng, 1e-2, 1e6)
    elif stratum == "log-uniform":
        rate = log_uniform(rng, 1e-8, 2.0)
        yld = log_uniform(rng, 1e-8, 2.0)
        redemption = log_uniform(rng, 1, 1e4)
        years = log_uniform(rng, 0.01, 100)
    # keep (1 + yld/f)^(f·years) inside the double range: past it PRICE is #NUM!, which the edge stratum covers
    periods = years * frequency
    if math.log1p(yld / frequency) * periods > MAX_LN_GROWTH:
        yld = frequency * math.expm1(MAX_LN_GROWTH / periods) * rng.uniform(0.1, 1.0)
    maturity = _term(rng, settlement, years)
    return Draft(settlement, maturity, rate, yld, redemption, frequency, _basis(rng))


class BondDraft(Protocol):
    """A drawn bond's value arguments, which `value_edge` breaks."""

    rate: float
    fourth: float
    redemption: float
    frequency: int
    basis: int | None


VALUE_EDGES = 5  # the shapes of `value_edge`


def value_edge(d: BondDraft, shape: int, rng: random.Random, *, price_given: bool) -> None:
    """Breaks one value argument of a bond function, each `#NUM!` (`shape` 0..4): a negative rate, a negative yld or a
    pr of 0, a redemption of 0 or below, a frequency other than 1, 2 or 4, a basis outside 0..4 ("PRICE, YIELD,
    DURATION and MDURATION check order", "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order")."""
    if shape == 0:
        d.rate = -rng.choice((1e-9, 0.01, 0.5))
    elif shape == 1:
        d.fourth = 0.0 if price_given else -rng.choice((1e-9, 0.01))
    elif shape == 2:
        d.redemption = rng.choice((0.0, -100.0))
    elif shape == 3:
        d.frequency = rng.choice((0, 3, 6, 12))
    else:
        d.basis = rng.choice((-1, 5))


def price_or_par(pr: float, redemption: float, rng: random.Random) -> float:
    """A yield function's drawn price: `pr` (the price at a drawn yld) when it is from 1E-3 to 1E4 of the redemption,
    else one within half the redemption of it (the sampler cuts it to 15 digits)."""
    if 1e-3 * redemption <= pr <= 1e4 * redemption:
        return pr
    return redemption * rng.uniform(0.5, 1.5)


def bond_definition(
    function: str,
    names: Sequence[str],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    draw: Callable[[str, random.Random], list[Arg]],
    sources: Sequence[str],
    *,
    solver: bool = False,
) -> Definition:
    """A bond function's registered definition: real arguments, then the frequency and an optional basis (`int`);
    every domain rule is the value's (Excel reads the arguments in its own order), so no interval checks."""
    params = [Param(name, "real", source=sources[0]) for name in names[:-2]]
    params.append(Param(names[-2], "int", source=sources[0]))
    params.append(Param(names[-1], "int", source=sources[0], optional=True))
    return register(
        Definition(
            function=function,
            params=tuple(params),
            value=value,
            draw=draw,
            solver=solver,
            serials_1900=True,
            sources=tuple(sources),
        )
    )


def _edge(rng: random.Random, *, price_given: bool) -> Draft:
    """Domain edges: reversed or equal dates, a settlement below 1, a negative rate or yld, a price or redemption of
    0, frequency 3, basis 5 or -1, a PCD before 1900, the yld at which q^(s+N-1) overflows and coupons whose sum
    overflows (each `#NUM!`), and settlement on a coupon date (A = 0)."""
    d = _base("typical", rng)
    shape = rng.randrange(10)
    if shape == 0:
        d.maturity = int(d.settlement) - rng.choice((0, 1, 400))
    elif shape == 1:
        d.settlement = rng.choice((0.0, 0.5))
    elif shape < 2 + VALUE_EDGES:
        value_edge(d, shape - 2, rng, price_given=price_given)
    elif shape == 7:  # a PCD before 1900-01-01 (#NUM!) or on 1900-01-01 (serial 1)
        d.settlement = rng.randint(1, 120)
        d.maturity = rng.choice((1000, 43831, 43830, 54788, 47484))
    elif shape == 8 and not price_given:  # huge yields and coupons (#NUM! past the double range)
        d.maturity = int(d.settlement) + round(rng.choice((10, 20)) * YEAR)
        if rng.random() < 0.5:
            d.fourth = rng.choice((1e8, 1e10, 1e16, 1e17, 1e100, 1e300))
        else:
            d.rate = rng.choice((1e10, 1e100, 1e300, 1e306, 9.99999999999999e307))
    else:  # settlement on a coupon date
        d.settlement = d.maturity - round(rng.randint(1, 40) * YEAR / d.frequency)
        pcd, _, _ = schedule(real_date(d.settlement), real_date(d.maturity), d.frequency)
        d.settlement = max(1, date_serial(pcd))
    return d


def _price_draw(stratum: str, rng: random.Random) -> list[Arg]:
    d = _edge(rng, price_given=False) if stratum == "edge" else _base(stratum, rng)
    return d.args(with_redemption=True)


def _duration_draw(stratum: str, rng: random.Random) -> list[Arg]:
    d = _edge(rng, price_given=False) if stratum == "edge" else _base(stratum, rng)
    return d.args(with_redemption=False)


def _yield_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """A bond priced at a drawn yld, the price cut to 15 digits: the math is then the root at that price. The draws
    keep the price from 1E-3 to 1E4 of the redemption and the coupon rate below 5, which still includes regions where
    Excel's Newton is `#NUM!` (a price of 1E5 and more at a redemption near 100, a price and redemption far from par:
    "YIELD's #NUM! cells and its 100-step cap", "YIELD is #NUM! at a price and redemption far from par"); the tail
    stratum adds negative yields (a price above the undiscounted flows)."""
    if stratum == "edge":
        d = _edge(rng, price_given=True)
        if rng.random() < 0.5:
            return d.args(with_redemption=True)
    else:
        d = _base(stratum, rng)
        if stratum == "tail" and rng.random() < 0.3:
            d.fourth = -rng.uniform(0.001, 0.2)
    pr = math.nan
    if (
        1 <= d.settlement < d.maturity
        and d.frequency in FREQUENCIES
        and d.rate >= 0
        and d.fourth > -d.frequency
    ):
        b = Bond(
            int(d.settlement),
            d.maturity,
            mpmath.mpf(d.rate),
            mpmath.mpf(d.fourth),
            mpmath.mpf(d.redemption),
            d.frequency,
            d.basis or 0,
        )
        with mpmath.workdps(30), contextlib.suppress(DomainError, NoValueError):
            pr = float(price_of(b, coupons(b.settlement, b.maturity, b.frequency, b.basis)))
    d.fourth = price_or_par(pr, d.redemption, rng)
    return d.args(with_redemption=True)


def _bond_definition(
    function: str,
    names: Sequence[str],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    *,
    solver: bool = False,
) -> Definition:
    draw = _yield_draw if function == "YIELD" else _price_draw if len(names) == 7 else _duration_draw
    sources = (
        ORDER_SOURCE,
        CALENDAR_SOURCE,
        EDGES_SOURCE,
        DAY_SOURCE,
        COUPDAYS_SOURCE,
        CARET_SOURCE,
        SUM_SOURCE,
        HUGE_SOURCE,
        ENGINE_SOURCE,
        *((ONE_PERIOD_SOURCE, SHORT_SOURCE, NEWTON_SOURCE, WORK_SOURCE) if solver else ()),
    )
    return bond_definition(function, names, value, draw, sources, solver=solver)


_PRICE_NAMES = ("settlement", "maturity", "rate", "yld", "redemption", "frequency", "basis")
_DURATION_NAMES = ("settlement", "maturity", "coupon", "yld", "frequency", "basis")

PRICE = _bond_definition("PRICE", _PRICE_NAMES, _price)
YIELD = _bond_definition(
    "YIELD", ("settlement", "maturity", "rate", "pr", "redemption", "frequency", "basis"), _yield, solver=True
)
DURATION = _bond_definition("DURATION", _DURATION_NAMES, _duration)
MDURATION = _bond_definition("MDURATION", _DURATION_NAMES, _mduration)
