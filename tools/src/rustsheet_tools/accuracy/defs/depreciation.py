"""SLN, SYD, DB, DDB and VDB (Gate A, T-4111, decision 0273).

These follow Excel's own documented algorithms, so the math is that algorithm at the working precision, on the doubles
of the written arguments, with Excel's rounding steps applied where Excel applies them (the task's context):
- SLN = (cost - salvage)/life.
- SYD = (cost - salvage)·(life - per + 1)·2/(life·(life + 1)).
- DB (financial.md "DB and DDB operation order"): rate = ROUND(1 - (salvage/cost)^(1/life), 3), half away from zero, the
  one rounding step Excel applies; the first period charges cost·rate·month/12 (a period below 2 by its whole part), then
  the balance cost - first falls by balance·rate a period, a middle period charges balance·rate and period
  TRUNC(life) + 1 balance·rate·(12 - month)/12. `month` is truncated.
- DDB (same entry): rate = factor/life (1 once it reaches 1), old = cost·(1 - rate)^(period - 1), the charge
  max(0, min(old·rate, old - salvage)); a period below 1 is period 1; at rate 1, a period other than 1 has old = 0.
- VDB (financial.md "VDB charge order"): with `no_switch` TRUE the window's sum of each period's
  max(0, min(b·rate, b - salvage)), b = cost·(1 - rate)^(i - 1), rate = min(factor/life, 1), a partial period by its
  share; switching (the default), the schedule laid on the start's fraction: the first fraction f charges
  f·max(0, min(cost·rate, cost - salvage)), each later period min(b·rate, b - salvage) until the straight-line charge
  (b - salvage)/(life - t) is larger than the unclipped b·rate, and from there that straight-line charge; salvage above
  cost never switches. Where straight line wins at period 0 ((cost - salvage)/life above cost·rate, a factor below
  1 - salvage/cost), every period of the window charges (cost - salvage)/life, from a fractional start too (T-4111's
  table, recorded in that entry). The sums' order ("summed in doubles", the `n·sl` product) is Excel's rounding,
  not a step of the algorithm: the exact sum is the math.

Domains (Excel's errors, `DomainError`), each evidenced by a semantics entry; "SLN, SYD, DB, DDB and VDB argument
domains" (`DOMAINS_SOURCE`) cites the eval and table cells of the ones marked (eval cells):
- SLN: life 0 is `#DIV/0!`.
- SYD ("DB and SYD argument domains"): a negative salvage is `#NUM!`, then life·(life + 1) past the double range
  `#DIV/0!`, then a numerator past it `#NUM!`; a per above life or at most 0, and life 0, are `#NUM!` (eval cells).
- DB ("DB and SYD argument domains"): a negative salvage, a life above 2147483647 (but the wrapped last year from a
  life of 4294967295 with a month other than 12 and a period below 1, and any life when the period is above 2147483647,
  which is period 1), a period's whole part above life + 1, and with month 12 a period above the life are `#NUM!`; cost
  0 is 0. A negative cost, a month outside 1..12 and a period at most 0 are `#NUM!` (eval cells).
- DDB: a period above the life, a negative cost or salvage, a period at most 0 and a factor at most 0 are `#NUM!` (eval
  cells; "VDB, DDB and ACCRINT argument types" for the factor of 0).
- VDB: a negative cost, salvage or factor, a life at most 0, a start below 0, a start or an end above the life, and a
  start above the end are `#NUM!` (eval cells). Switching with life = start = end, the engine's `#NUM!`, is unprobed:
  no value.
Not drawn and given no math value (`NoValueError`): a DB period from 100,001 to 2147483647 and a VDB end past 100,000
(the engine's period cap, a DEVIATION: "DB and VDB period cap"), a DB rate within 1e-9 of a rounding half (Excel's
double rate may round either way), a value or an intermediate past the double range, a point whose value cancels
(SLN's and SYD's cost - salvage, DDB's and VDB's book value less the salvage) past `CONDITION_LIMIT`, and the arguments
no evidence covers (listed where each definition raises it).
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    EPS,
    StratumDraws,
    by_stratum,
    ill_conditioned,
    one_of,
    scalars,
    trunc,
)
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    Draw,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)
from rustsheet_tools.accuracy.sampler import log_uniform

DB_SYD_SOURCE = "financial.md#db-and-syd-argument-domains"
ORDER_SOURCE = "financial.md#db-and-ddb-operation-order"
CAP_SOURCE = "financial.md#db-and-vdb-period-cap"
VDB_SOURCE = "financial.md#vdb-charge-order"
T1625_SOURCE = "financial.md#t-1625-probes-vdb-amordegrc-fvschedule-mduration-rri"
TYPES_SOURCE = "financial.md#vdb-ddb-and-accrint-argument-types"
LIMITS_SOURCE = "financial.md#annuity-and-db-argument-domains-at-their-limits"
DOMAINS_SOURCE = "financial.md#sln-syd-db-ddb-and-vdb-argument-domains"
DECISION = "decision 0273"

DOUBLE_MAX = 1.7976931348623157e308
PERIOD_CAP = 100_000  # the engine's DB and VDB period cap (DEVIATION, "DB and VDB period cap")
DB_INTEGER_MAX = 2_147_483_647.0  # DB reads life and period as 32-bit integers
DB_WRAPPED_LIFE = 4_294_967_295.0  # from here the life saturates and life + 1 wraps to period 0
RATE_TIE_BAND = mpmath.mpf("1e-9")  # DB's rate this close to a rounding half (in units of 0.001): no value


def _finite(x: mpmath.mpf, what: str) -> mpmath.mpf:
    if abs(x) > DOUBLE_MAX:
        msg = f"{what} past the double range"
        raise NoValueError(msg)
    return x


def _cancels(kept: mpmath.mpf, difference: mpmath.mpf, steps: int = 1) -> None:
    """Redraw when `difference` (of terms of size `kept`) cancels past `CONDITION_LIMIT` over `steps` roundings."""
    if difference == 0:
        if kept != 0:
            msg = "the difference cancels to 0"
            raise NoValueError(msg)
        return
    ill_conditioned(float(abs(kept) / abs(difference)) * max(steps, 1))


def _num_if(condition: bool, why: str) -> None:
    """`#NUM!` (DomainError) when `condition` holds."""
    if condition:
        msg = f"{why}: #NUM!"
        raise DomainError(msg)


# -- SLN and SYD -------------------------------------------------------------------------------------


def _sln(args: Sequence[MathArg]) -> mpmath.mpf:
    cost, salvage, life = scalars(args)
    if life == 0:
        msg = "life 0: #DIV/0!"
        raise DomainError(msg)
    depreciable = _finite(cost - salvage, "cost - salvage")
    _cancels(abs(cost) + abs(salvage), depreciable)
    return _finite(depreciable / life, "SLN")


def _syd(args: Sequence[MathArg]) -> mpmath.mpf:
    cost, salvage, life, per = scalars(args)
    if salvage < 0:
        msg = "a negative salvage: #NUM!"
        raise DomainError(msg)
    _num_if(life == 0, "life 0")
    _num_if(per > life, "a per above the life")
    _num_if(per <= 0, "a per at most 0")
    if abs(float(life) * (float(life) + 1.0)) > DOUBLE_MAX:
        msg = "life·(life + 1) past the double range: #DIV/0!"
        raise DomainError(msg)
    numerator = (cost - salvage) * (life - per + 1) * 2
    if abs(numerator) > DOUBLE_MAX:
        msg = "the numerator past the double range: #NUM!"
        raise DomainError(msg)
    _cancels(abs(cost) + abs(salvage), cost - salvage)
    _cancels(abs(life) + 1, life - per + 1)
    return _finite(numerator / (life * (life + 1)), "SYD")


# -- DB ----------------------------------------------------------------------------------------------


def db_rate(cost: mpmath.mpf, salvage: mpmath.mpf, life: mpmath.mpf) -> mpmath.mpf:
    """DB's rate: 1 - (salvage/cost)^(1/life) rounded to 3 decimals, half away from zero (cost > 0, salvage >= 0).
    NoValueError within `RATE_TIE_BAND` of a half, where Excel's double may round the other way."""
    exact = 1 - (salvage / cost) ** (1 / life)
    thousandths = exact * 1000
    below = mpmath.floor(abs(thousandths))
    if abs(abs(thousandths) - below - mpmath.mpf(1) / 2) < RATE_TIE_BAND:
        msg = "DB's rate at a rounding half"
        raise NoValueError(msg)
    rounded = below + (1 if abs(thousandths) - below > mpmath.mpf(1) / 2 else 0)
    return mpmath.sign(thousandths) * rounded / 1000


def _db(args: Sequence[MathArg]) -> mpmath.mpf:
    a = scalars(args)
    cost, salvage, life, period = a[0], a[1], a[2], a[3]
    month = mpmath.mpf(trunc(a[4])) if len(a) > 4 else mpmath.mpf(12)
    huge_period = period > DB_INTEGER_MAX
    if huge_period:
        period = mpmath.mpf(1)
    wrapped = life >= DB_WRAPPED_LIFE and month != 12 and period < 1
    if month == 12 and period > life:
        msg = "month 12 and a period above the life: #NUM!"
        raise DomainError(msg)
    if trunc(period) > life + 1:
        msg = "a period's whole part above life + 1: #NUM!"
        raise DomainError(msg)
    _num_if(month <= 0, "a month at most 0")
    _num_if(month > 12, "a month above 12")
    _num_if(period <= 0, "a period at most 0")
    _num_if(cost < 0, "a negative cost")
    _num_if(salvage < 0, "a negative salvage")
    if life > DB_INTEGER_MAX and not huge_period and not wrapped:
        msg = "a life above 2147483647: #NUM!"
        raise DomainError(msg)
    if cost == 0:
        return mpmath.mpf(0)
    if life <= 0:
        msg = "a life at most 0 (unprobed)"
        raise NoValueError(msg)
    rate = db_rate(cost, salvage, life)
    if wrapped:
        return (cost - cost * rate * month / 12) * (rate * (12 - month) / 12)
    whole = trunc(period)
    if PERIOD_CAP < whole <= DB_INTEGER_MAX:
        msg = "a period past the engine's cap (DEVIATION)"
        raise NoValueError(msg)
    first = cost * rate * month / 12
    if whole <= 1:
        return first
    balance = _finite((cost - first) * (1 - rate) ** (whole - 2), "the balance")
    if whole - 1 == trunc(life):
        return balance * rate * (12 - month) / 12
    return balance * rate


# -- DDB ---------------------------------------------------------------------------------------------


def _ddb(args: Sequence[MathArg]) -> mpmath.mpf:
    a = scalars(args)
    cost, salvage, life, period = a[0], a[1], a[2], a[3]
    factor = a[4] if len(a) > 4 else mpmath.mpf(2)
    _num_if(period > life, "a period above the life")
    _num_if(cost < 0, "a negative cost")
    _num_if(salvage < 0, "a negative salvage")
    _num_if(period <= 0, "a period at most 0")
    _num_if(factor <= 0, "a factor at most 0")
    period = max(period, mpmath.mpf(1))
    rate = factor / life
    if rate >= 1:
        old = cost if period == 1 else mpmath.mpf(0)
        rate = mpmath.mpf(1)
    else:
        old = cost * (1 - rate) ** (period - 1)
    if old * rate <= old - salvage:
        return old * rate
    _cancels(old, old - salvage)
    return max(mpmath.mpf(0), old - salvage)


# -- VDB ---------------------------------------------------------------------------------------------


class _Charges:
    """A window's sum of charges, with the worst cancellation of a book value less the salvage met on the way."""

    def __init__(self, salvage: mpmath.mpf) -> None:
        self.salvage = salvage
        self.total = mpmath.mpf(0)
        self.kappa = 0.0
        self.steps = 0

    def less_salvage(self, book: mpmath.mpf) -> mpmath.mpf:
        """book - salvage, recording its cancellation."""
        diff = book - self.salvage
        self.steps += 1
        scale = abs(book) + abs(self.salvage)
        if scale != 0:
            self.kappa = max(self.kappa, math.inf if diff == 0 else float(scale / abs(diff)))
        return diff


def _vdb_no_switch(
    cost: mpmath.mpf,
    salvage: mpmath.mpf,
    life: mpmath.mpf,
    start: mpmath.mpf,
    end: mpmath.mpf,
    *,
    factor: mpmath.mpf,
) -> tuple[mpmath.mpf, _Charges]:
    sums = _Charges(salvage)
    rate = min(factor / life, mpmath.mpf(1))
    for period in range(int(mpmath.floor(start)) + 1, int(mpmath.ceil(end)) + 1):
        book = (cost if period == 1 else mpmath.mpf(0)) if rate == 1 else cost * (1 - rate) ** (period - 1)
        clipped = book * rate
        if clipped > book - salvage:
            clipped = sums.less_salvage(book)
        charge = max(mpmath.mpf(0), clipped)
        share = min(end, mpmath.mpf(period)) - max(start, mpmath.mpf(period - 1))
        sums.total += charge * share
    return sums.total, sums


def _vdb_switching(
    cost: mpmath.mpf,
    salvage: mpmath.mpf,
    life: mpmath.mpf,
    start: mpmath.mpf,
    end: mpmath.mpf,
    *,
    factor: mpmath.mpf,
) -> tuple[mpmath.mpf, _Charges]:
    sums = _Charges(salvage)
    switches = salvage <= cost
    rate = factor / life
    if switches and (cost - salvage) / life > cost * rate:
        # Straight line wins at period 0: every period of the window charges (cost - salvage)/life, a fractional start
        # too, with no declining-balance fraction first (financial.md "VDB charge order", accuracy/VDB/s1 A158, A183,
        # A187, A188, A189, A200, T-4111)
        return (end - start) * (cost - salvage) / life, sums
    whole = mpmath.floor(start)
    fraction = start - whole
    book = cost
    if fraction > 0:
        book -= fraction * max(mpmath.mpf(0), min(cost * rate, cost - salvage))
    window = end - start
    straight: mpmath.mpf | None = None
    for index in range(int(mpmath.ceil(end)) + 1):
        t = fraction + index
        if t >= end:
            break
        if straight is not None:
            charge = straight
        else:
            unclipped = book * rate
            left = sums.less_salvage(book)
            sln = left / (life - t)
            if switches and sln > unclipped:
                straight = charge = sln
            else:
                charge = min(unclipped, left)
        offset = index - whole
        if offset >= 0:
            sums.total += charge * (min(window, offset + 1) - offset)
        book -= charge
    return sums.total, sums


def _vdb(args: Sequence[MathArg]) -> mpmath.mpf:
    numbers = [a for a in args if not isinstance(a, bool)]
    a = scalars(numbers)
    cost, salvage, life, start, end = a[0], a[1], a[2], a[3], a[4]
    factor = a[5] if len(a) > 5 else mpmath.mpf(2)
    no_switch = len(args) > 6 and bool(args[6])
    _num_if(cost < 0, "a negative cost")
    _num_if(salvage < 0, "a negative salvage")
    _num_if(life <= 0, "a life at most 0")
    _num_if(factor < 0, "a negative factor")
    _num_if(start < 0, "a negative start")
    _num_if(start > life, "a start above the life")
    _num_if(end > life, "an end above the life")
    _num_if(start > end, "a start above the end")
    if not no_switch and life == start == end:  # the engine's #NUM!, from the library it ports; no eval cell
        msg = "switching, with life, start and end equal (unprobed)"
        raise NoValueError(msg)
    if mpmath.floor(end) > PERIOD_CAP:
        msg = "an end period past the engine's cap (DEVIATION)"
        raise NoValueError(msg)
    run = _vdb_no_switch if no_switch else _vdb_switching
    total, sums = run(cost, salvage, life, start, end, factor=factor)
    if sums.kappa * sums.steps * EPS > 1e-10 and total != 0:
        msg = "the book value less the salvage cancels"
        raise NoValueError(msg)
    return total


# -- draws -------------------------------------------------------------------------------------------

Shape = Callable[[], list[Arg]]


def _amounts(rng: random.Random, lo: float = 1.0, hi: float = 1e7) -> tuple[float, float]:
    """(cost, salvage): a cost log-uniform in [lo, hi] and a salvage below it (0 one time in eight)."""
    cost = log_uniform(rng, lo, hi)
    salvage = 0.0 if rng.random() < 0.125 else cost * log_uniform(rng, 1e-4, 0.9)
    return cost, salvage


def _life(rng: random.Random, hi: int = 40) -> int:
    return rng.randint(1, hi)


def _fractional(rng: random.Random, x: int) -> float:
    return x + rng.choice((0.25, 0.5, 0.75, 0.3, 0.9))


def stratified(
    make: Callable[[str, random.Random], list[Arg]], edge: Callable[[random.Random], list[Arg]], doc: str
) -> Draw:
    """A `draw` taking every stratum but the edge from `make` and the edge from `edge`."""

    def of(stratum: str) -> Callable[[random.Random], list[Arg]]:
        return lambda rng: make(stratum, rng)

    draws: StratumDraws = {s: of(s) for s in ("typical", "tail", "extreme", "log-uniform")}
    return by_stratum({**draws, "edge": edge}, doc)


def _sln_draw(stratum: str, rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life: float = _life(rng)
    if stratum == "tail":  # salvage above cost, a fractional or a negative life
        return one_of(
            rng,
            [
                lambda: [cost, cost * rng.uniform(1.1, 3), life],
                lambda: [cost, salvage, _fractional(rng, int(life))],
                lambda: [cost, salvage, -life],
                lambda: [-cost, salvage, life],
            ],
        )
    if stratum == "extreme":
        cost = log_uniform(rng, 1e-300, 1e300)
        return [cost, cost * rng.uniform(0, 0.9), log_uniform(rng, 1e-6, 1e12)]
    if stratum == "log-uniform":
        return [log_uniform(rng, 1e-6, 1e12), log_uniform(rng, 1e-6, 1e12), log_uniform(rng, 1e-6, 1e12)]
    return [cost, salvage, life]


def _sln_edge(rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    return one_of(
        rng,
        [
            lambda: [cost, salvage, 0.0],  # #DIV/0!
            lambda: [cost, cost, _life(rng)],  # 0
            lambda: [0.0, 0.0, _life(rng)],
            lambda: [cost, 0.0, 1.0],
            lambda: [cost, salvage, rng.choice((1e-300, 1e300, 0.5))],
        ],
    )


def _syd_draw(stratum: str, rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life = _life(rng)
    per: float = rng.randint(1, life)
    if stratum == "tail":
        return one_of(
            rng,
            [
                lambda: [cost, cost * rng.uniform(1.1, 3), life, per],  # salvage above cost: negative
                lambda: [cost, salvage, _fractional(rng, life), per],
                lambda: [cost, salvage, life, max(per - rng.uniform(0, 0.99), 1e-3)],
                lambda: [-cost, salvage, life, per],
            ],
        )
    if stratum == "extreme":
        big = rng.choice((1e6, 1e9, 1e12, 1e100))
        life_x = log_uniform(rng, 10, big)
        return [log_uniform(rng, 1e-300, 1e300), 0.0, life_x, life_x * rng.random()]
    if stratum == "log-uniform":
        life_l = log_uniform(rng, 1e-3, 1e6)
        return [log_uniform(rng, 1e-3, 1e12), log_uniform(rng, 1e-3, 1e12), life_l, life_l * rng.random()]
    return [cost, salvage, life, per]


def _syd_edge(rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life = _life(rng)
    return one_of(
        rng,
        [
            lambda: [cost, -salvage - 1.0, life, 1],  # negative salvage: #NUM!
            lambda: [cost, salvage, life, life],  # the last period
            lambda: [cost, salvage, life, life + rng.choice((0.5, 1.0))],  # past the life: #NUM!
            lambda: [cost, salvage, life, rng.choice((0.0, -1.0))],  # #NUM!
            lambda: [cost, 0.0, life, 1],
            lambda: [
                100.0,
                10.0,
                rng.choice((1.3e154, 1.4e154, 1e200)),
                1,
            ],  # life·(life+1) overflows: #DIV/0!
            lambda: [rng.choice((9e307, 5e307)), 0.0, rng.choice((3, 10)), 1],  # numerator overflows: #NUM!
        ],
    )


def _db_draw(stratum: str, rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    if salvage == 0 and rng.random() < 0.5:
        salvage = cost * log_uniform(rng, 1e-4, 0.9)
    life = _life(rng, 30)
    month = rng.randint(1, 12)
    per: float = rng.randint(1, life + (1 if month != 12 else 0))
    if stratum == "tail":
        return one_of(
            rng,
            [
                lambda: [cost, cost * rng.uniform(1.1, 50), life, per, month],  # a negative rate
                lambda: [cost, salvage, _fractional(rng, life), per, month],
                lambda: [
                    cost,
                    salvage,
                    life,
                    per + rng.choice((0.25, 0.5, 0.99)) if per <= life else per,
                    month,
                ],
                lambda: [cost, 0.0, life, per, month],  # rate 1
                lambda: [cost, salvage, life, life + 1, rng.randint(1, 11)],  # the last year
            ],
        )
    if stratum == "extreme":
        # long lives at salvage 0 (rate 1 - 0^(1/life) = 1: the first period only) or a small salvage; costs
        # over the whole double range
        life_x = rng.choice((rng.randint(100, 2000), rng.randint(2000, 50000)))
        c = log_uniform(rng, 1e-300, 1e300) if rng.random() < 0.5 else cost
        s = 0.0 if rng.random() < 0.3 else c * log_uniform(rng, 1e-300, 1e-3)
        return [c, s, life_x, rng.randint(1, min(life_x + 1, PERIOD_CAP)), month]
    if stratum == "log-uniform":
        c = log_uniform(rng, 1e-3, 1e12)
        return [c, c * log_uniform(rng, 1e-6, 0.99), life, per, month]
    if rng.random() < 0.2:
        return [cost, salvage, life, per]
    return [cost, salvage, life, per, month]


def _db_edge(rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life = _life(rng, 20)
    return one_of(
        rng,
        [
            lambda: [cost, -1.0, life, 1],  # negative salvage: #NUM!
            lambda: [0.0, salvage, life, 1],  # cost 0: 0
            lambda: [cost, salvage, life, life + 2, rng.randint(1, 11)],  # past life + 1: #NUM!
            lambda: [cost, salvage, life, life + 1, 12],  # month 12 past the life: #NUM!
            lambda: [cost, salvage, 2147483648.0, 1],  # a life past 32 bits: #NUM!
            lambda: [cost, salvage, 2147483647.0, 1],
            lambda: [
                cost,
                0.0,
                rng.choice((4294967296.0, 1e10)),
                0.5,
                rng.randint(1, 11),
            ],  # the wrapped last year
            lambda: [
                cost,
                salvage,
                life,
                rng.choice((2147483648.0, 1e15, 1e200)),
                rng.randint(1, 12),
            ],  # period 1
            lambda: [cost, salvage, life + 0.5, life + 1 + rng.choice((0.0, 0.5)), rng.randint(1, 11)],
            lambda: [cost, salvage, life, 1, 12],
        ],
    )


def _ddb_draw(stratum: str, rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life = _life(rng)
    per: float = rng.randint(1, life)
    factor = rng.choice((2.0, 2.0, 1.5, 1.0, 3.0, round(rng.uniform(0.5, 4), 3)))
    if stratum == "tail":
        return one_of(
            rng,
            [
                lambda: [cost, salvage, _fractional(rng, life), per, factor],
                lambda: [cost, salvage, life, max(per - rng.choice((0.25, 0.5, 0.75)), 0.5), factor],
                lambda: [cost, salvage, life, per, life * rng.uniform(1, 2)],  # rate 1
                lambda: [cost, cost * rng.uniform(1.1, 3), life, per, factor],  # salvage above cost: 0
            ],
        )
    if stratum == "extreme":
        life_x = log_uniform(rng, 50, 1e5)
        return [log_uniform(rng, 1e-300, 1e300), 0.0, life_x, rng.uniform(1, min(life_x, 300)), factor]
    if stratum == "log-uniform":
        c = log_uniform(rng, 1e-3, 1e12)
        life_l = log_uniform(rng, 1, 1e4)
        return [
            c,
            c * log_uniform(rng, 1e-6, 0.99),
            life_l,
            rng.uniform(1, min(life_l, 500)),
            log_uniform(rng, 0.1, 10),
        ]
    if rng.random() < 0.25:
        return [cost, salvage, life, per]
    return [cost, salvage, life, per, factor]


def _ddb_edge(rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life = _life(rng)
    return one_of(
        rng,
        [
            lambda: [cost, salvage, life, life + 1],  # #NUM!
            lambda: [cost, salvage, life, rng.choice((0.0, -1.0))],  # #NUM!
            lambda: [cost, -1.0, life, 1],
            lambda: [-cost, salvage, life, 1],
            lambda: [cost, salvage, life, 1, rng.choice((0.0, -1.0))],  # factor: #NUM!
            lambda: [cost, salvage, life, rng.choice((0.25, 0.5, 0.99))],  # below 1: period 1
            lambda: [cost, salvage, life, life],
            lambda: [0.0, 0.0, life, 1],
        ],
    )


def _vdb_window(rng: random.Random, life: float, fraction: bool) -> tuple[float, float]:
    top = math.floor(life)
    start: float = rng.randint(0, top)
    end: float = rng.randint(int(start), top)
    if fraction:
        start = min(start + rng.choice((0.25, 0.5, 0.3, 0.75)), life)
        end = min(max(end + rng.choice((0.0, 0.5, 0.2, 0.75)), start), life)
    return start, end


def _vdb_draw(stratum: str, rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life: float = _life(rng, 30)
    factor = rng.choice((2.0, 2.0, 1.5, 1.0, 3.0, round(rng.uniform(0.5, 4), 3)))
    if stratum == "tail":
        if rng.random() < 0.3:
            life = _fractional(rng, int(life))
        if rng.random() < 0.2:
            salvage = cost * rng.uniform(1.0, 2.0)
    if stratum == "extreme":
        life = rng.randint(50, 2000)
        cost = log_uniform(rng, 1e-300, 1e300)
        salvage = 0.0 if rng.random() < 0.5 else cost * log_uniform(rng, 1e-6, 0.5)
    if stratum == "log-uniform":
        cost = log_uniform(rng, 1e-3, 1e12)
        salvage = cost * log_uniform(rng, 1e-6, 0.99)
        factor = log_uniform(rng, 0.1, 10)
    start, end = _vdb_window(rng, life, stratum in ("tail", "log-uniform") or rng.random() < 0.3)
    shape = rng.randrange(4)
    if shape == 0:
        return [cost, salvage, life, start, end]
    if shape == 1:
        return [cost, salvage, life, start, end, factor]
    return [cost, salvage, life, start, end, factor, shape == 3]


def _vdb_edge(rng: random.Random) -> list[Arg]:
    cost, salvage = _amounts(rng)
    life = _life(rng, 20)
    flag = rng.random() < 0.5
    return one_of(
        rng,
        [
            lambda: [cost, salvage, life, 0.0, 0.0],
            lambda: [cost, salvage, life, life, life, 2.0, True],  # an empty window at the life
            lambda: [cost, salvage, life, 0.0, life, 2.0, flag],  # the whole life
            lambda: [cost, salvage, life, life + 1.0, life + 1.0],  # #NUM!
            lambda: [cost, salvage, life, 0.0, life + 0.5],  # #NUM!
            lambda: [cost, salvage, life, 2.0, 1.0],  # #NUM!
            lambda: [cost, salvage, life, -1.0, 1.0],
            lambda: [-cost, salvage, life, 0.0, 1.0],
            lambda: [cost, -1.0, life, 0.0, 1.0],
            lambda: [cost, salvage, life, 0.0, 1.0, -1.0],
            lambda: [cost, salvage, life, 0.0, 1.0, 0.0, flag],  # factor 0: straight line, or 0
            lambda: [cost, cost * rng.uniform(1.0, 2.0), life, 0.0, rng.choice((0.5, 1.0, 1.5)), 2.0, flag],
            lambda: [cost, salvage, life, life - 0.75, life, rng.choice((2.0, 4.0)), flag],
            lambda: [cost, salvage, 2.0, 0.0, 1.0, 3.0, True],  # factor above the life: rate 1
        ],
    )


def _definition(
    function: str,
    params: tuple[Param, ...],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    make: Callable[[str, random.Random], list[Arg]],
    edge: Callable[[random.Random], list[Arg]],
    *,
    sources: tuple[str, ...],
) -> Definition:
    names = ", ".join(p.name for p in params)
    return register(
        Definition(
            function=function,
            params=params,
            value=value,
            draw=stratified(make, edge, f"{function}: ({names}) by stratum."),
            sources=(DECISION, *sources),
        )
    )


def _real(name: str, source: str, *, optional: bool = False) -> Param:
    return Param(name, "real", Interval(), source=source, optional=optional)


SLN = _definition(
    "SLN",
    (_real("cost", ORDER_SOURCE), _real("salvage", ORDER_SOURCE), _real("life", ORDER_SOURCE)),
    _sln,
    _sln_draw,
    _sln_edge,
    sources=(ORDER_SOURCE, DOMAINS_SOURCE),
)
SYD = _definition(
    "SYD",
    tuple(_real(n, DB_SYD_SOURCE) for n in ("cost", "salvage", "life", "per")),
    _syd,
    _syd_draw,
    _syd_edge,
    sources=(DB_SYD_SOURCE, DOMAINS_SOURCE),
)
DB = _definition(
    "DB",
    (
        *(_real(n, DB_SYD_SOURCE) for n in ("cost", "salvage", "life", "period")),
        Param("month", "int", Interval(), source=ORDER_SOURCE, optional=True, truncates=ORDER_SOURCE),
    ),
    _db,
    _db_draw,
    _db_edge,
    sources=(ORDER_SOURCE, DB_SYD_SOURCE, CAP_SOURCE, LIMITS_SOURCE, DOMAINS_SOURCE),
)
DDB = _definition(
    "DDB",
    (
        *(_real(n, ORDER_SOURCE) for n in ("cost", "salvage", "life", "period")),
        _real("factor", TYPES_SOURCE, optional=True),
    ),
    _ddb,
    _ddb_draw,
    _ddb_edge,
    sources=(ORDER_SOURCE, TYPES_SOURCE, DOMAINS_SOURCE),
)
VDB = _definition(
    "VDB",
    (
        *(_real(n, VDB_SOURCE) for n in ("cost", "salvage", "life", "start_period", "end_period")),
        _real("factor", TYPES_SOURCE, optional=True),
        Param("no_switch", "bool", source=TYPES_SOURCE, optional=True),
    ),
    _vdb,
    _vdb_draw,
    _vdb_edge,
    sources=(VDB_SOURCE, T1625_SOURCE, TYPES_SOURCE, CAP_SOURCE, DOMAINS_SOURCE),
)
