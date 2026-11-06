"""CUMIPMT and CUMPRINC (Gate A, T-4110, decision 0273): the interest and the principal paid from period `start_period`
to `end_period` of a loan of `pv` over `nper` periods at `rate`.

The math, on the doubles of the written arguments, with `1 + rate` exact (L = ln(1 + rate), v^j = e^(-j·L)):
- the payment is p = pv/A, A = (1 + rate·type)·(1 - v^nper)/rate, the annuity factor `present_value` gives at pmt 1
  (defs/annuity.py, T-3535): pv·rate/(1 - v^nper) for type 0 and that over 1 + rate for type 1;
- period k's principal is -p·v^(nper-k+1) and its interest -p·(1 - v^(nper-k+1)), except period 1 of type 1, whose
  principal is the whole payment -p and whose interest is 0;
- CUMPRINC and CUMIPMT sum those over the whole periods from ceil(start_period) to floor(end_period); the geometric sum
  is closed, Σ v^(nper-k+1) = v^(nper-b+1)·(1 - v^m)/(1 - v) over k = a..b, m = b - a + 1, so the period count costs
  nothing. The interest m - Σ v^j cancels where nper·L is small; the sum runs with enough extra digits to cover that.

Sources (financial.md):
- "CUMIPMT and CUMPRINC over many periods": Excel computes the sums in constant time and within a few ulps of the exact
  sums (a rate of 1E-300 is not lost), also where (1+rate)^nper overflows; the start rounds up and the end down
  (probes/fin-cumipmt-period-bound A23 `CUMIPMT(0.001,2000,1000,1.5,1999.7,0)` sums periods 2 to 1999); type 1's period
  1 adds the whole payment. Excel's CUMIPMT is `-pmt·m - CUMPRINC` in doubles (T-2192, T-2363), so where the interest is
  small against the payments it carries that cancellation: the tail stratum's tiny rates are off the math there and
  pass through Excel's 15 digits (decision 0300).
- "CUMIPMT of one period is not that period's IPMT (T-3179)": Excel's own form is open (T-4197); its values are ulps
  from the exact sums.
- "CUMIPMT and CUMPRINC work cap": a lifted-call resource limit (DEVIATION); the table's scalar calls are not capped.
Domains (Excel's `#NUM!`, `DomainError`), each evidenced (financial.md "CUMIPMT and CUMPRINC argument domains
(T-4110)" adds the table's own `#NUM!` points):
- a rate of 0: gen-m9 9d9ceba257f8001b A382 `CUMIPMT(0,310,54445,6,22,0)` and b73c6cc96e093f03 A188
  `CUMIPMT(0,1E+9,496779,8,24,1)`;
- a start period below 1, before it rounds up: gen-m9 0a957119475760c5 A40 `CUMPRINC(0.05,249,329832,0.3,19,0)`
  ("Financial functions in the gen-m9 financial sweep (T-2257)" in financial.md, and "CUMIPMT and CUMPRINC
  over many periods");
- an end period past nper, also below nper + 1: probes/fin-cumipmt-period-bound A20-A22
  (`CUMIPMT(0.01,100.5,1000,1,101,0)`), gen-m9 a91bad4e84f49739 A95 `CUMPRINC(1.005,1,18739,11,17,0)`;
- a start period after the end period: gen-m9 67a4c03afd63f988 A271 `CUMIPMT(0.05,272,417600,7,2,0)`, 5422bdbf173fc233 A233
  `CUMPRINC(100,224,131835,361,14,0)`.
Not drawn and given no math value (`NoValueError`), as no evidence names Excel's result: a negative rate (only -2,
gen-m9 d0854366830dcdee A232, and the overflow case -0.5 over 2000 periods, "PMT when (1+rate)^nper overflows or underflows", are
probed); a pv of 0 or below (no eval cell has one); a type other than 0 or 1 (only 0.5 is probed, `#NUM!`; it is not an
integer, so such an eval cell gets no math value anyway); a start and end in order whose rounding leaves no whole period
(ceil(start) > floor(end) with start <= end).
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs import annuity
from rustsheet_tools.accuracy.defs._common import Shape, any_edge, by_stratum_with_edge, real_args
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

MANY_SOURCE = "financial.md#cumipmt-and-cumprinc-over-many-periods"
ONE_PERIOD_SOURCE = "financial.md#cumipmt-of-one-period-is-not-that-periods-ipmt-t-3179"
CAP_SOURCE = "financial.md#cumipmt-and-cumprinc-work-cap"
SWEEP_SOURCE = "financial.md#financial-functions-in-the-gen-m9-financial-sweep-t-2257"
DOMAIN_SOURCE = "financial.md#cumipmt-and-cumprinc-argument-domains-t-4110"
EXTRA_DIGITS = 15  # beyond the digits nper·L cancels


def cumulative(args: Sequence[MathArg]) -> tuple[mpmath.mpf, mpmath.mpf]:
    """(interest, principal) of the periods ceil(start)..floor(end): CUMIPMT's and CUMPRINC's values."""
    r, nper, pv, start, end, kind = real_args(args)
    if r == 0:
        msg = "a rate of 0: #NUM!"
        raise DomainError(msg)
    if start < 1 or end > nper or start > end:
        msg = "a start below 1, an end past nper or a start after the end: #NUM!"
        raise DomainError(msg)
    if r < 0 or pv <= 0 or kind not in (0, 1):
        msg = "a negative rate, a pv of 0 or below or a type other than 0 and 1 (unprobed)"
        raise NoValueError(msg)
    a, b = int(mpmath.ceil(start)), int(mpmath.floor(end))
    if a > b:
        msg = "no whole period between start and end (unprobed)"
        raise NoValueError(msg)
    due = kind == 1
    lost = -mpmath.log10(mpmath.log1p(r) * nper)  # the digits the interest's m - Σ v^j cancels
    extra = EXTRA_DIGITS + max(0, int(mpmath.ceil(lost)))
    with mpmath.workdps(mpmath.mp.dps + extra):
        u = mpmath.log1p(r)
        p = pv / annuity.present_value(u, nper, mpmath.mpf(1), mpmath.mpf(0), mpmath.mpf(0), due=due)
        principal = mpmath.mpf(0)
        if due and a == 1:  # period 1 of type 1: the whole payment is principal
            principal, a = -p, 2
        interest = mpmath.mpf(0)
        if a <= b:
            m = b - a + 1
            ratio = mpmath.expm1(-m * u) / mpmath.expm1(-u)  # (1 - v^m)/(1 - v)
            geometric = mpmath.exp(-(nper - b + 1) * u) * ratio
            principal -= p * geometric
            interest = -p * (m - geometric)
        return +interest, +principal


def _cumipmt(args: Sequence[MathArg]) -> mpmath.mpf:
    return cumulative(args)[0]


def _cumprinc(args: Sequence[MathArg]) -> mpmath.mpf:
    return cumulative(args)[1]


# -- draws -----------------------------------------------------------------------------------------


def _periods(rng: random.Random, nper: int) -> tuple[int, int]:
    """A start and end period, 1 <= start <= end <= nper: a short range one time in two (Excel's short and long forms,
    "CUMIPMT and CUMPRINC over many periods"), anywhere in the term."""
    start = rng.randint(1, nper)
    if rng.random() < 0.5:
        return start, min(nper, start + rng.randint(0, 31))
    return start, rng.randint(start, nper)


def _make(stratum: str, rng: random.Random) -> list[Arg]:
    """[rate, nper, pv, start_period, end_period, type] from RATE's annuity (`draw_annuity`, defs/annuity.py) with a
    positive pv and rate; the tail stratum's rate is tiny or large, the extreme stratum adds long terms."""
    _sign, nper, r, pv, _fv, due = annuity.draw_annuity(stratum, rng)
    r, pv = abs(r), abs(pv)
    if stratum == "tail":
        r = log_uniform(rng, 1e-12, 1e-4) if rng.random() < 0.5 else rng.uniform(0.5, 5)
    elif stratum == "extreme" and rng.random() < 0.3:
        nper = round(log_uniform(rng, 1e4, 1e9))
    start, end = _periods(rng, nper)
    return [r, nper, pv, start, end, 1 if due else 0]


def _amount(rng: random.Random) -> float:
    return log_uniform(rng, 1e2, 1e6)


def _rate(rng: random.Random) -> float:
    return log_uniform(rng, 1e-3, 0.2)


def _type(rng: random.Random) -> int:
    return rng.randrange(2)


def _single(rng: random.Random) -> list[Arg]:
    """One period: the first, the last or any (T-3179's single-period cells)."""
    n = rng.randint(1, 360)
    k = rng.choice((1, n, rng.randint(1, n)))
    return [_rate(rng), n, _amount(rng), k, k, _type(rng)]


def _whole_term(rng: random.Random) -> list[Arg]:
    """Every period, where the principal is pv."""
    n = rng.randint(1, 400)
    return [_rate(rng), n, _amount(rng), 1, n, _type(rng)]


def _fractional_periods(rng: random.Random) -> list[Arg]:
    """A fractional start (rounds up) and end (rounds down), as probes/fin-cumipmt-period-bound A23."""
    n = rng.randint(4, 400)
    start = rng.randint(1, n - 2) + rng.choice((0.5, 0.25, 0.999))
    end = rng.randint(math.ceil(start), n - 1) + rng.choice((0.7, 0.5, 0.001))
    return [_rate(rng), n, _amount(rng), start, end, _type(rng)]


def _fractional_nper(rng: random.Random) -> list[Arg]:
    """A fractional nper with the end at or below it (probes/fin-cumipmt-overflow-and-forms A17,
    `CUMPRINC(1E10,31.5,1,1,20,0)`; t3179 `CUMIPMT(0.1,29.9,8000,3,10,1)`)."""
    whole = rng.randint(1, 300)
    n = whole + rng.choice((0.5, 0.9, 0.25))
    start = rng.randint(1, whole)
    return [_rate(rng), n, _amount(rng), start, rng.randint(start, whole), _type(rng)]


def _late_in_long_term(rng: random.Random) -> list[Arg]:
    """A short range late in a long term (`CUMIPMT(0.01,1E6,100000,1,10,0)`-like cells, A13)."""
    n = round(log_uniform(rng, 1e3, 1e12))
    start = rng.choice((1, n - rng.randint(0, 20), rng.randint(1, n)))
    return [
        log_uniform(rng, 1e-6, 0.05),
        n,
        _amount(rng),
        start,
        min(n, start + rng.randint(0, 10)),
        _type(rng),
    ]


def _rate_zero(rng: random.Random) -> list[Arg]:
    """A rate of 0 (`#NUM!`)."""
    n = rng.randint(1, 360)
    start = rng.randint(1, n)
    return [0.0, n, _amount(rng), start, rng.randint(start, n), _type(rng)]


def _bad_periods(rng: random.Random) -> list[Arg]:
    """A start below 1, an end past nper (also below nper + 1, over a fractional nper) or a start after the end
    (`#NUM!`)."""
    n = rng.randint(2, 360)
    nper: float = n
    shape = rng.randrange(4)
    start: float
    end: float
    if shape == 0:
        start, end = rng.choice((0, 0.3, 0.5)), rng.randint(1, n)
    elif shape == 1:
        start, end = rng.randint(1, n), n + rng.choice((1, 10))
    elif shape == 2:
        nper, start, end = n + 0.5, rng.randint(1, n), n + 1
    else:
        end = rng.randint(1, n - 1)
        start = rng.randint(end + 1, n)
    return [_rate(rng), nper, _amount(rng), start, end, _type(rng)]


def _large_rate(rng: random.Random) -> list[Arg]:
    """A large rate over a short term (the principal a deep discount, "T-2514"), kept within the double range."""
    r = rng.choice((1.0, 2.0, 10.0, 100.0, rng.uniform(1, 50)))
    n = rng.randint(1, max(1, min(200, int(600 / math.log1p(r)))))
    start = rng.randint(1, n)
    return [r, n, _amount(rng), start, rng.randint(start, n), _type(rng)]


# The aimed draw (`generate --stratum aimed`, batch r4-T-3141; T-4059): T-3141's core, a short range where
# `1 - v^nper` is 1 in doubles. s1 does not draw it, so its points stay as committed.
UNDERFLOWED_NPER_LOG = 40.0  # nper·L from which v^nper is below 2^-53 (from about 36.7), so 1 - v^nper is 1
SHORT_PERIODS = 32  # the engine's CUMULATIVE_LOOP_PERIODS: a range of at most 32 periods is a short one
GEOMETRIC_MIN_LEAD = 0.3  # the engine's cut between the core and the geometric sum (T-3067, T-3141)
DEEPEST_LEAD = 700.0  # below 708, where v^(nper-end+1) leaves the normal doubles
LARGEST_NPER = 1e15  # below 2^53, so every period is an exact integer


def _underflowed(rng: random.Random) -> list[Arg]:
    """A short range (at most 32 periods) where v^nper underflows: T-3141's core. The rate is fractional (1E-4 to 3);
    the leading exponent (nper-end+1)·L is below the engine's 0.3 cut one time in three, else log-uniform from 0.3 to
    700, so ranges far from nper whose leading power is still normal are drawn; m·L is log-uniform from 0.01 to 5,
    across PMT's expm1 switch at 1; type 1 starts from period 2 (its period 1 is the whole payment); nper is from the
    least that holds the range and underflows to 1E15, near that least one time in two."""
    r = log_uniform(rng, 1e-4, 0.05) if rng.random() < 0.5 else log_uniform(rng, 0.05, 3)
    u = math.log1p(r)
    lead = (
        rng.uniform(0, GEOMETRIC_MIN_LEAD)
        if rng.random() < 1 / 3
        else log_uniform(rng, GEOMETRIC_MIN_LEAD, DEEPEST_LEAD)
    )
    after = max(1, round(lead / u))  # nper - end + 1
    count = min(SHORT_PERIODS, max(1, round(log_uniform(rng, 0.01, 5) / u)))
    kind = rng.randrange(2)
    least = max(after + count - 1 + kind, math.ceil(UNDERFLOWED_NPER_LOG / u))
    top = min(LARGEST_NPER, least * 1e3) if rng.random() < 0.5 else LARGEST_NPER
    nper = round(log_uniform(rng, least, max(least, top)))
    end = nper - after + 1
    return [r, nper, _amount(rng), end - count + 1, end, kind]


EDGES: tuple[Shape, ...] = (
    _single,
    _whole_term,
    _fractional_periods,
    _fractional_nper,
    _late_in_long_term,
    _rate_zero,
    _bad_periods,
    _large_rate,
)


def _definition(function: str, value: Callable[[Sequence[MathArg]], mpmath.mpf]) -> Definition:
    return register(
        Definition(
            function=function,
            params=(
                Param("rate", "real", source=MANY_SOURCE),
                Param("nper", "real", source=MANY_SOURCE),
                Param("pv", "real", source=MANY_SOURCE),
                Param("start_period", "real", source=MANY_SOURCE),
                Param("end_period", "real", source=MANY_SOURCE),
                Param("type", "int", source=MANY_SOURCE),
            ),
            value=value,
            draw=by_stratum_with_edge(
                _make,
                any_edge(EDGES),
                f"{function}: [rate, nper, pv, start_period, end_period, type] by stratum.",
            ),
            aimed=_underflowed,
            sources=(MANY_SOURCE, DOMAIN_SOURCE, ONE_PERIOD_SOURCE, CAP_SOURCE, SWEEP_SOURCE),
        )
    )


CUMIPMT = _definition("CUMIPMT", _cumipmt)
CUMPRINC = _definition("CUMPRINC", _cumprinc)
