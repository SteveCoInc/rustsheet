"""DISC, INTRATE, RECEIVED, PRICEDISC, YIELDDISC, TBILLEQ and TBILLPRICE (Gate A, T-4112, decision 0273).

The math is exact on the written arguments. Both dates are floored to whole serials (`whole_serial`, defs/yearfrac.py:
a date that does not floor to 0..2958465 is `#NUM!`). The five discount securities read YEARFRAC's two parts apart
(`day_count_parts`, defs/yearfrac.py): the basis day count D from settlement to maturity (30/360 for bases 0 and 4,
serial days otherwise) and the year length B (360, 365, or basis 1's YEARFRAC year). With pr the price, red the
redemption, inv the investment and r the discount rate:
- DISC = (1 - pr/red)·B/D;  INTRATE = (red - inv)/inv·B/D;  YIELDDISC = (red - pr)/pr·B/D;
- PRICEDISC = red - r·red·D/B;  RECEIVED = inv/(1 - r·D/B).
The T-bills count serial days d = maturity - settlement over a 360-day year:
- TBILLPRICE = 100·(1 - r·d/360);
- TBILLEQ = 365·r/(360 - r·d) up to 182 days, and from 183 days the root of the semi-annual equivalence:
  with p = 100 - r·d·100/360, t = d/Y (Y = 366 when d is 366, else 365),
  (-t + sqrt(t² - (2t - 1)(1 - 100/p)))/(t - 0.5).
Excel's own operation orders (financial.md "Discount securities: day count and year length", "TBILLEQ and
TBILLYIELD") put it a few ulps off these values; the math is the exact quotient, root or difference.

Domains (Excel's errors, `DomainError`), each evidenced:
- every function: a date off the calendar, settlement on or after maturity, and a basis outside 0..4 are `#NUM!`
  (financial.md "Discount-security functions: settlement 0, basis and argument modes", "Discount securities check
  order", "Day-count basis: a negative basis is #NUM!");
- a zero or negative price, investment or redemption is `#NUM!`, as is a zero or negative discount of PRICEDISC and
  RECEIVED; RECEIVED is `#NUM!` from 1 - r·D/B = 0 down ("Discount-security functions: settlement 0 ..."; for DISC's
  price and negative redemption, RECEIVED's discount and 1 - r·D/B below 0, `AMOUNTS_SOURCE`, which lists the cells);
- settlement 0 computes, but RECEIVED is `#NUM!` there ("Discount-security functions: settlement 0 ...";
  probes/discount-serial-zero for DISC);
- the T-bills: maturity more than a year after settlement (the dates Excel shows; a 29 February reaches its
  anniversary on 1 March) and a settlement in 9999 are `#NUM!` ("T-bill settlement below serial 1", "T-bills settled
  in 9999"); a negative discount is `#NUM!` (`AMOUNTS_SOURCE`); a negative price is `#NUM!` in both, and a zero price
  `#NUM!` in TBILLEQ from 183 days, while TBILLPRICE gives 0 there and TBILLEQ 0 up to 182 days ("TBILLEQ and
  TBILLYIELD").
Not given a math value (`NoValueError`), as no evidence names Excel's result: DISC at redemption 0, a T-bill discount
of 0, and a day count D of 0 (30/360 from a 30th to the 31st of the same month), for which the engine's `#DIV/0!` is
unprobed. A point whose value cancels (the relative condition number past 2^-52·1E-10, `ill_conditioned`) gets none
either: there Excel's doubles, not the math, decide the digits, and a price within a hair of 0 may be either side of it
in Excel (`TBILLEQ(1000,1100,3.6)` is 0: 3.6·100 is 360 in doubles, while the exact product is above it).
"""

from __future__ import annotations

import calendar
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    by_stratum,
    fraction_to_mpf,
    ill_conditioned,
    one_of,
    scalars,
)
from rustsheet_tools.accuracy.defs.clock import LAST_DAY
from rustsheet_tools.accuracy.defs.yearfrac import (
    check_basis,
    day_count_parts,
    displayed_date,
    serial_of,
    whole_serial,
)
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

MODES_SOURCE = "financial.md#discount-security-functions-settlement-0-basis-and-argument-modes"
PARTS_SOURCE = "financial.md#discount-securities-day-count-and-year-length"
ORDER_SOURCE = "financial.md#discount-securities-check-order"
BASIS_SOURCE = "financial.md#day-count-basis-a-negative-basis-is-num"
TBILL_SOURCE = "financial.md#tbilleq-and-tbillyield"
BELOW_ONE_SOURCE = "financial.md#t-bill-settlement-below-serial-1"
YEAR_9999_SOURCE = "financial.md#t-bills-settled-in-9999"
AMOUNTS_SOURCE = "financial.md#discount-securities-and-t-bills-amount-domains-t-4112"
SECURITY_SOURCES = (PARTS_SOURCE, MODES_SOURCE, ORDER_SOURCE, BASIS_SOURCE, AMOUNTS_SOURCE)
TBILL_SOURCES = (TBILL_SOURCE, BELOW_ONE_SOURCE, YEAR_9999_SOURCE, ORDER_SOURCE, AMOUNTS_SOURCE)

SETTLEMENT_0 = "settlement 0"


def _num(why: str) -> DomainError:
    return DomainError(f"{why}: #NUM!")


def _dates(settlement: mpmath.mpf, maturity: mpmath.mpf) -> tuple[int, int]:
    s, m = whole_serial(float(settlement)), whole_serial(float(maturity))
    if s >= m:
        raise _num("settlement on or after maturity")
    return s, m


def _positive(*amounts: mpmath.mpf) -> None:
    if any(a <= 0 for a in amounts):
        raise _num("an amount at most 0")


def _security(
    args: Sequence[MathArg], *, zero_second_unprobed: bool = False
) -> tuple[int, int, mpmath.mpf, mpmath.mpf, int, Fraction]:
    """(settlement, maturity, first amount, second amount, D, B) of a discount security's arguments, both amounts
    positive (the basis is read first: "Discount securities check order"). With `zero_second_unprobed`, a second
    amount of exactly 0 beside a positive first one has no math value (DISC's redemption 0), once the basis and the
    dates are valid."""
    a = scalars(args)
    basis = int(a[4]) if len(a) > 4 else 0
    check_basis(basis)
    s, m = _dates(a[0], a[1])
    if zero_second_unprobed and a[3] == 0 and a[2] > 0:
        msg = "a redemption of 0 (unprobed)"
        raise NoValueError(msg)
    _positive(a[2], a[3])
    days, year = day_count_parts(s, m, basis)
    return s, m, a[2], a[3], days, year


def _per_day_count(days: int) -> None:
    if days == 0:
        msg = "a day count of 0 (unprobed)"
        raise NoValueError(msg)


def _cancels(value: mpmath.mpf, scale: mpmath.mpf) -> None:
    """NoValueError where `value` is a difference of terms of size `scale` that nearly cancel; an exact 0 (equal
    amounts) is Excel's 0 too."""
    if value != 0:
        ill_conditioned(float(abs(scale) / abs(value)))


def _disc(args: Sequence[MathArg]) -> mpmath.mpf:
    _s, _m, pr, red, days, year = _security(args, zero_second_unprobed=True)
    _per_day_count(days)
    gap = 1 - pr / red
    _cancels(gap, mpmath.mpf(1))
    return gap * fraction_to_mpf(year / days)


def _intrate(args: Sequence[MathArg]) -> mpmath.mpf:
    _s, _m, inv, red, days, year = _security(args)
    _per_day_count(days)
    _cancels(red - inv, red)
    return (red - inv) / inv * fraction_to_mpf(year / days)


def _yielddisc(args: Sequence[MathArg]) -> mpmath.mpf:
    _s, _m, pr, red, days, year = _security(args)
    _per_day_count(days)
    _cancels(red - pr, red)
    return (red - pr) / pr * fraction_to_mpf(year / days)


def _pricedisc(args: Sequence[MathArg]) -> mpmath.mpf:
    _s, _m, rate, red, days, year = _security(args)
    share = rate * fraction_to_mpf(Fraction(days) / year)
    _price_share(share)
    return red * (1 - share)


def _price_share(share: mpmath.mpf) -> None:
    """NoValueError where 1 - `share` (a price over its redemption) nearly cancels but is not exactly 0: Excel's
    double may land on either side of 0 (`TBILLEQ(1000,1100,3.6)`: 3.6·100 is 360 in doubles, above it exactly)."""
    _cancels(1 - share, share)


def _received(args: Sequence[MathArg]) -> mpmath.mpf:
    s, _m, inv, rate, days, year = _security(args)
    if s == 0:
        raise _num("RECEIVED at settlement 0")
    share = rate * fraction_to_mpf(Fraction(days) / year)
    _price_share(share)
    if share >= 1:
        raise _num("1 - discount·D/B at most 0")
    return inv / (1 - share)


# -- T-bills -----------------------------------------------------------------------------------------------------


def _anniversary_ok(settlement: int, maturity: int) -> bool:
    """Whether `maturity` is on or before `settlement`'s first anniversary, on the dates Excel shows (1900-01-00's is
    1900-12-31, a 29 February's the next 1 March)."""
    y1, m1, d1 = displayed_date(settlement)
    y2, m2, d2 = displayed_date(maturity)
    anniversary = (3, 1) if (m1, d1) == (2, 29) else (m1, d1)
    return y2 == y1 or (y2 == y1 + 1 and (m2, d2) <= anniversary)


def _tbill(args: Sequence[MathArg]) -> tuple[int, mpmath.mpf]:
    """(days, discount) of a T-bill's arguments."""
    settlement, maturity, rate = scalars(args)
    s, m = _dates(settlement, maturity)
    if displayed_date(s)[0] >= 9999:
        raise _num("a settlement in 9999")
    if not _anniversary_ok(s, m):
        raise _num("maturity more than a year after settlement")
    if rate == 0:
        msg = "a discount of 0 (unprobed)"
        raise NoValueError(msg)
    if rate < 0:
        raise _num("a negative discount")
    return m - s, rate


def _tbillprice(args: Sequence[MathArg]) -> mpmath.mpf:
    days, rate = _tbill(args)
    share = rate * days / 360
    _price_share(share)
    if share > 1:
        raise _num("a negative price")
    return 100 * (1 - share)  # 0 at a price of 0: TBILLPRICE(1000,1100,3.6)


def _tbilleq(args: Sequence[MathArg]) -> mpmath.mpf:
    days, rate = _tbill(args)
    share = rate * days / 360
    _price_share(share)
    if share > 1:
        raise _num("a negative price")
    denominator = 360 - rate * days
    if denominator == 0:
        if days < 183:
            return mpmath.mpf(0)  # TBILLEQ(1000,1100,3.6) is 0 (a zero denominator)
        raise _num("a price of 0 from 183 days")
    if days < 183:
        return 365 * rate / denominator
    price = 100 * denominator / 360
    t = mpmath.mpf(days) / (366 if days == 366 else 365)
    return (-t + mpmath.sqrt(t * t - (2 * t - 1) * (1 - 100 / price))) / (t - mpmath.mpf(0.5))


# -- draws -------------------------------------------------------------------------------------------------------

Shape = Callable[[random.Random], list[Arg]]
MODERN = (32874, 58440)  # 1990-01-01 .. 2059-12-31


def _with_basis(rng: random.Random, point: list[Arg]) -> list[Arg]:
    """`point` with basis 0-4, or with the basis left out one time in six (basis 0)."""
    if rng.random() < 1 / 6:
        return point
    return [*point, rng.randrange(5)]


def _span(rng: random.Random, stratum: str, most: int) -> int:
    """Days from settlement to maturity: up to `most` by stratum (short spans in the tail)."""
    if stratum == "tail":
        return rng.choice((1, 2, 3, rng.randint(1, 31), rng.randint(355, 370)))
    return int(log_uniform(rng, 1, most))


def _month_end_pair(rng: random.Random, most: int) -> tuple[float, float]:
    """A settlement on a month's 28th to 31st and a maturity on another month's end (the 30/360 adjustments)."""
    year = rng.randint(1990, 2060)
    month = rng.randint(1, 12)
    last = calendar.monthrange(year, month)[1]
    start = int(serial_of(year, month, rng.randint(28, last)))
    end = start + rng.randint(1, most)
    return float(start), float(end)


def _dates_for(rng: random.Random, stratum: str, most: int) -> tuple[float, float]:
    if stratum == "tail" and rng.random() < 0.4:
        return _month_end_pair(rng, most)
    lo, hi = (1, LAST_DAY - most - 1) if stratum == "extreme" else MODERN
    start = rng.randint(lo, hi)
    return float(start), float(start + _span(rng, stratum, most))


def _amount(rng: random.Random, stratum: str) -> float:
    if stratum == "extreme":
        return round15(log_uniform(rng, 1e-100, 1e100))
    if stratum == "log-uniform":
        return round15(log_uniform(rng, 1e-6, 1e12))
    return round15(log_uniform(rng, 1, 1e6))


def _ratio(rng: random.Random, stratum: str) -> float:
    """A price over the redemption: near 1 in the tail, wide in the extreme and log-uniform strata."""
    if stratum == "tail":
        return 1 + rng.choice((-1, 1)) * log_uniform(rng, 1e-6, 1e-2)
    if stratum in ("extreme", "log-uniform"):
        return log_uniform(rng, 1e-3, 1e3)
    return rng.uniform(0.5, 1.2)


def _rate(rng: random.Random, stratum: str) -> float:
    if stratum == "tail":
        return round15(log_uniform(rng, 1e-10, 1e-3))
    if stratum in ("extreme", "log-uniform"):
        return round15(log_uniform(rng, 1e-12, 2))
    return round15(rng.uniform(0.001, 0.25))


def _priced(stratum: str, rng: random.Random) -> list[Arg]:
    """(settlement, maturity, price, redemption) of DISC, INTRATE and YIELDDISC, spans up to 30 years."""
    s, m = _dates_for(rng, stratum, 10958)
    red = _amount(rng, stratum)
    return _with_basis(rng, [s, m, round15(red * _ratio(rng, stratum)), red])


def _discounted(stratum: str, rng: random.Random) -> list[Arg]:
    """(settlement, maturity, discount, redemption) of PRICEDISC, spans up to 30 years."""
    s, m = _dates_for(rng, stratum, 10958)
    return _with_basis(rng, [s, m, _rate(rng, stratum), _amount(rng, stratum)])


def _received_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """(settlement, maturity, investment, discount) of RECEIVED, spans up to 30 years."""
    s, m = _dates_for(rng, stratum, 10958)
    return _with_basis(rng, [s, m, _amount(rng, stratum), _rate(rng, stratum)])


def _tbill_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """(settlement, maturity, discount): spans of 1 to 365 days, both TBILLEQ branches."""
    s, m = _dates_for(rng, stratum, 365)
    if stratum == "tail" and rng.random() < 0.5:
        m = s + rng.choice((182, 183, 184, 364, 365))
    return [s, m, _rate(rng, stratum)]


def _edge_dates(rng: random.Random) -> tuple[float, float]:
    """Dates at the calendar's ends and around 1900-02-29, or reversed, equal or off the calendar (`#NUM!`)."""
    shapes: list[Callable[[], tuple[float, float]]] = [
        lambda: (
            rng.choice((0.0, 1.0, 59.0, 60.0, 61.0)),
            rng.choice((59.0, 60.0, 61.0, 366.0, 425.0, 45000.0)),
        ),
        lambda: (float(LAST_DAY - rng.randint(1, 400)), float(LAST_DAY)),
        lambda: (45000.0, 45000.0),
        lambda: (45100.0, 45000.0 - rng.randint(0, 50)),
        lambda: (rng.choice((-1.0, -0.5)), 45000.0),
        lambda: (45000.0, rng.choice((2958466.0, 3e6))),
    ]
    return rng.choice(shapes)()


def _security_edge(amounts: Callable[[random.Random], tuple[float, float]]) -> Shape:
    def edge(rng: random.Random) -> list[Arg]:
        shapes: list[Shape] = [
            lambda rng: _with_basis(rng, [*_edge_dates(rng), *amounts(rng)]),
            lambda rng: [*_dates_for(rng, "typical", 3650), *amounts(rng), rng.choice((-1, 5, 6))],
            lambda rng: _with_basis(
                rng, [*_dates_for(rng, "typical", 3650), *(rng.choice((0.0, -1.0, a)) for a in amounts(rng))]
            ),
        ]
        return rng.choice(shapes)(rng)

    return edge


def _priced_amounts(rng: random.Random) -> tuple[float, float]:
    red = round15(log_uniform(rng, 1, 1e4))
    return round15(red * rng.uniform(0.5, 1.5)), red


def _discounted_amounts(rng: random.Random) -> tuple[float, float]:
    return round15(rng.uniform(0.001, 0.5)), round15(log_uniform(rng, 1, 1e4))


def _received_amounts(rng: random.Random) -> tuple[float, float]:
    return round15(log_uniform(rng, 1, 1e4)), round15(rng.uniform(0.001, 0.5))


def _received_edge(rng: random.Random) -> list[Arg]:
    """RECEIVED's edges, and discounts with 1 - r·D/B near or past 0 (`#NUM!` from 0 down)."""
    if rng.random() < 0.3:
        s, m = _dates_for(rng, "typical", 3650)
        return [s, m, round15(log_uniform(rng, 1, 1e4)), rng.choice((1.0, 2.0, 0.99, 1.5)), 2]
    return _security_edge(_received_amounts)(rng)


def _tbill_edge(rng: random.Random) -> list[Arg]:
    """T-bill edges: serial 0, the fictitious day, anniversaries, 9999, zero and negative prices and discounts."""
    rate = _rate(rng, "typical")
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [rng.choice((0.0, 0.5, 1.0)), rng.choice((1.0, 2.0, 59.0, 60.0, 61.0, 366.0, 367.0)), rate],
        lambda: [60.0, rng.choice((425.0, 426.0, 427.0)), rate],
        lambda: [61.0, rng.choice((426.0, 427.0)), rate],
        lambda: [45351.0, rng.choice((45716.0, 45717.0, 45718.0)), rate],  # 2024-02-29 to 2025-02-28 .. 03-02
        lambda: [45352.0, rng.choice((45716.0, 45717.0, 45718.0)), rate],  # 2024-03-01
        lambda: [rng.choice((2958100.0, 2958099.0, 2958101.0)), rng.choice((2958200.0, 2958465.0)), rate],
        lambda: [45000.0, 45000.0 + rng.randint(1, 365), rng.choice((0.0, -0.05))],
        lambda: [
            1000.0,
            rng.choice((1100.0, 1182.0, 1183.0, 1200.0, 1360.0)),
            rng.choice((3.6, 2.0, 1.8, 1.0, 4.0)),
        ],
        lambda: [45000.0, rng.choice((45000.0, 44999.0, 45366.0, 45400.0)), rate],
        lambda: [rng.choice((-1.0, -0.5)), 100.0, rate],
    ]
    return one_of(rng, shapes)


def _strata(
    make: Callable[[str, random.Random], list[Arg]], edge: Shape, doc: str
) -> Callable[..., list[Arg]]:
    """Every stratum but the edge from `make`; the edge stratum `edge`, or `make`'s typical one time in four."""

    def edges(rng: random.Random) -> list[Arg]:
        return make("typical", rng) if rng.random() < 0.25 else edge(rng)

    draws = {s: (lambda rng, s=s: make(s, rng)) for s in ("typical", "tail", "extreme", "log-uniform")}
    return by_stratum({**draws, "edge": edges}, doc)


def _security_definition(
    function: str,
    names: tuple[str, str],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    draw: Callable[..., list[Arg]],
) -> Definition:
    dates = (Param(n, "real", source=MODES_SOURCE) for n in ("settlement", "maturity"))
    amounts = (Param(n, "real", source=PARTS_SOURCE) for n in names)
    basis = Param("basis", "int", source=BASIS_SOURCE, optional=True)
    return register(
        Definition(
            function=function,
            params=(*dates, *amounts, basis),
            value=value,
            draw=draw,
            serials_1900=True,
            sources=SECURITY_SOURCES,
        )
    )


def _tbill_definition(function: str, value: Callable[[Sequence[MathArg]], mpmath.mpf]) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("settlement", "real", source=BELOW_ONE_SOURCE),
                Param("maturity", "real", source=BELOW_ONE_SOURCE),
                Param("discount", "real", source=TBILL_SOURCE),
            ),
            value=value,
            draw=_strata(
                _tbill_draw, _tbill_edge, f"{function}: (settlement, maturity, discount) by stratum."
            ),
            serials_1900=True,
            sources=TBILL_SOURCES,
        )
    )


DISC = _security_definition(
    "DISC",
    ("pr", "redemption"),
    _disc,
    _strata(
        _priced, _security_edge(_priced_amounts), "DISC: (settlement, maturity, pr, redemption, [basis])."
    ),
)
INTRATE = _security_definition(
    "INTRATE",
    ("investment", "redemption"),
    _intrate,
    _strata(
        _priced,
        _security_edge(_priced_amounts),
        "INTRATE: (settlement, maturity, investment, redemption, [basis]).",
    ),
)
YIELDDISC = _security_definition(
    "YIELDDISC",
    ("pr", "redemption"),
    _yielddisc,
    _strata(
        _priced,
        _security_edge(_priced_amounts),
        "YIELDDISC: (settlement, maturity, pr, redemption, [basis]).",
    ),
)
PRICEDISC = _security_definition(
    "PRICEDISC",
    ("discount", "redemption"),
    _pricedisc,
    _strata(
        _discounted,
        _security_edge(_discounted_amounts),
        "PRICEDISC: (settlement, maturity, discount, redemption, [basis]).",
    ),
)
RECEIVED = _security_definition(
    "RECEIVED",
    ("investment", "discount"),
    _received,
    _strata(
        _received_draw, _received_edge, "RECEIVED: (settlement, maturity, investment, discount, [basis])."
    ),
)
TBILLPRICE = _tbill_definition("TBILLPRICE", _tbillprice)
TBILLEQ = _tbill_definition("TBILLEQ", _tbilleq)
