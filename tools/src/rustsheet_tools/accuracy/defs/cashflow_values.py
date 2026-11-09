"""NPV, XNPV, MIRR and FVSCHEDULE: the values of a series of flows (Gate A, T-4116, decision 0273).

The math, on the doubles of the written literals (the flows and the schedule are one-row array literals, T-3586; XNPV's
dates are a second one, as XIRR's are in defs/cashflow.py), with `1 + rate` exact:
- NPV(rate, values) = sum_i v_i / (1 + rate)^i, i = 1..n;
- XNPV(rate, values, dates) = sum_i v_i / (1 + rate)^((d_i - d_0)/365), each date truncated to a whole day;
- MIRR(values, finance_rate, reinvest_rate) = (P / -N)^(1/(n - 1)) - 1, with P = sum of the positive flows
  v_i·(1 + reinvest_rate)^(n - 1 - i) and N = sum of the negative flows v_i/(1 + finance_rate)^i (i = 0..n-1);
- FVSCHEDULE(principal, schedule) = principal·prod_i (1 + s_i).
Excel's own value is these forms in doubles, in its own operation order (NPV's running power and deferred division,
MIRR's running powers and `/ (1+rr)` step, XNPV's `^`), a few ulp off the math or more where the sum cancels.

Sources (financial.md):
- "NPV's residue at IRR's root is not the double sum" and "NPV's discount past the double range": NPV divides v_i by
  the running power (1 + rate)^(i-1) and the sum once by 1 + rate; `#DIV/0!` when 1 + rate is 0 (`NPV(-1,{1,2})`) and
  when the power the last period builds on, (1 + rate)^(n-1), leaves the double range (`NPV(1E+200,1,1,1)`,
  `NPV(1E+300,0,0,0)`, while `NPV(1E+300,1,1E+300)` is 2E-300 and `NPV(1E+154,1,1,1)` 1E-154); a rate below -1 is a
  value (`NPV(-2,1,1)` 0, `NPV(-3,1,1,1)` -0.375). "NPV with a rate of -1 or 1E+300 is #DIV/0!" agrees. A power below
  the normal range is `#DIV/0!` only at 1074 zeros (2^-1073) and a value at 1023 (2^-1022); between them it is unprobed,
  so such a point gets no value. "NPV and FVSCHEDULE arguments": an array literal is read whole. "NPV, IRR and MIRR
  error order" and "NPV's error order" concern error arguments, which a numeric point never has.
- "XNPV discounts with Excel's `^`" (the form; date 0 is `#NUM!`), "XNPV reads its values and dates element by element
  (T-3815)" (negative dates, date 0, dates past 2958465 and a later date before the first are `#NUM!`; a later date is
  bounded before it is truncated, so 2958465.5 is `#NUM!`; other fractional dates are truncated, and later dates need
  not be sorted) and "XNPV's rate, first date and later dates (T-3312)" (a rate at or below 0 is `#NUM!`). "XIRR and
  XNPV under the gen-m9 statistical cells" (values and dates of different counts are `#NUM!`), "XNPV, XIRR and
  FVSCHEDULE beside their whole data" (the lift, not a point's concern), "XNPV reads one empty values cell as a single
  0 cash flow" and "XNPV is #VALUE! when a values range's first cell is no number" (cells, which a literal point never
  has). A discount past the double range is unprobed (the first entry's "Unprobed"): no value.
- "MIRR's evaluation order" (the form and the power overflows) and "MIRR error and edge cases": one flow and no
  negative flow are `#DIV/0!`; all-negative flows are -1; a reinvest rate of -1 is `#DIV/0!`; a finance rate of -1 is
  `#DIV/0!` when a negative flow follows the first and a value otherwise; rates below -1 are values and `#NUM!` where
  the ratio goes negative (`MIRR({50,-100,80},-2,..)`, three flows: the square root); either power past the double
  range is an error (`#DIV/0!` or `#NUM!` by the flows' signs; probes/fin-mirr-edges, fin-mirr-overflow-*; "MIRR at
  finance 1E+200 over five flows with a negative flow at index 2 is #DIV/0! (T-4116)", found by this table).
- "NPV and FVSCHEDULE arguments", "FVSCHEDULE with one empty schedule cell is the principal" and "XNPV, XIRR and
  FVSCHEDULE beside their whole data": FVSCHEDULE reads a numeric schedule whole and has no domain on it.
  "FVSCHEDULE whose running product overflows or underflows" is OPEN, so a product past the double range
  gets no value.
Not drawn and given no math value (`NoValueError`), as no evidence names Excel's result: a power within a hair of a cut
of the double range (Excel's double power may be on either side); NPV's power in the subnormal range; an XNPV discount
past the double range, a first date past 2958465 that is not a whole day, or a date below 1 that truncates to 0; MIRR
with a power past the range on one of n - 1 and n only (which power Excel forms last for the finance rate is
unprobed), a power below the normal range, a zero flow after the first at a finance rate of -1, or a negative ratio
over other than three flows (Excel's `^` takes odd roots of negatives); a FVSCHEDULE product or partial product past
the double range or below the normal range.
"""

from __future__ import annotations

import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import NORMAL_MIN, Shape, is_real, signed
from rustsheet_tools.accuracy.defs.annuity_values import (
    LOG2_MAX,
    LOG2_NORMAL,
    NORMAL_MAX,
    _log2_growth,
    _strata,
)
from rustsheet_tools.accuracy.defs.cashflow import DAYS_PER_YEAR, FIRST_DATE, _array, _irr_draw, _xirr_draw
from rustsheet_tools.accuracy.defs.clock import LAST_DAY
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

NPV_ORDER_SOURCE = "financial.md#npvs-residue-at-irrs-root-is-not-the-double-sum"
NPV_RANGE_SOURCE = "financial.md#npvs-discount-past-the-double-range"
NPV_DIV0_SOURCE = "financial.md#npv-with-a-rate-of--1-or-1e300-is-div0"
NPV_ARGS_SOURCE = "financial.md#npv-and-fvschedule-arguments"
NPV_ERROR_ORDER_SOURCE = "financial.md#npv-irr-and-mirr-error-order"
NPV_ERRORS_SOURCE = "financial.md#npvs-error-order"
XNPV_POWER_SOURCE = "financial.md#xnpv-discounts-with-excels-"
XNPV_ELEMENT_SOURCE = "financial.md#xnpv-reads-its-values-and-dates-element-by-element-t-3815"
XNPV_RATE_SOURCE = "financial.md#xnpvs-rate-first-date-and-later-dates-t-3312"
XNPV_SIZE_SOURCE = "financial.md#xirr-and-xnpv-under-the-gen-m9-statistical-cells-size-and-type-checks-t-2255"
XNPV_EMPTY_SOURCE = "financial.md#xnpv-reads-one-empty-values-cell-as-a-single-0-cash-flow"
XNPV_FIRST_SOURCE = "financial.md#xnpv-is-value-when-a-values-ranges-first-cell-is-no-number"
WHOLE_DATA_SOURCE = "financial.md#xnpv-xirr-and-fvschedule-beside-their-whole-data"
MIRR_ORDER_SOURCE = "financial.md#mirrs-evaluation-order"
MIRR_EDGES_SOURCE = "financial.md#mirr-error-and-edge-cases"
MIRR_OVERFLOW_SOURCE = (
    "financial.md#mirr-at-finance-1e200-over-five-flows-with-a-negative-flow-at-index-2-is-div0-t-4116"
)
FVSCHEDULE_EMPTY_SOURCE = "financial.md#fvschedule-with-one-empty-schedule-cell-is-the-principal"
FVSCHEDULE_OVERFLOW_SOURCE = (
    "financial.md#fvschedule-whose-running-product-overflows-excel-graph-cannot-open-it"
)

MIRR_SQRT_FLOWS = 3  # MIRR's root is a square root here: a negative ratio is #NUM! (fin-mirr-edges A23)


def _rate_power(r: mpmath.mpf, k: int) -> mpmath.mpf:
    """log2 |1 + r|^k (NoValueError within a hair of a cut of the double range, `_log2_growth`)."""
    return _log2_growth(mpmath.log(abs(1 + r)), mpmath.mpf(k), (LOG2_MAX, LOG2_NORMAL))


def _npv(args: Sequence[MathArg]) -> mpmath.mpf:
    r, flows = args[0], _array(args[1])
    assert is_real(r)
    if r == -1:
        msg = "1 + rate is 0: #DIV/0!"
        raise DomainError(msg)
    if len(flows) > 1:
        log2 = _rate_power(r, len(flows) - 1)
        if log2 >= LOG2_MAX:
            msg = "(1 + rate)^(n - 1) past the double range: #DIV/0!"
            raise DomainError(msg)
        if log2 < LOG2_NORMAL:
            msg = "(1 + rate)^(n - 1) below the normal range (unprobed between 2^-1073 and 2^-1022)"
            raise NoValueError(msg)
    x = 1 + r
    return mpmath.fsum(v / x ** (i + 1) for i, v in enumerate(flows))


def _days(dates: Sequence[mpmath.mpf]) -> list[int]:
    """XNPV's dates as whole days, DomainError (`#NUM!`) for a date Excel refuses."""
    days: list[int] = []
    for i, d in enumerate(dates):
        if d < 0 or d == 0:
            msg = "a negative date or date 0: #NUM!"
            raise DomainError(msg)
        if d < 1:
            msg = "a date in (0, 1), which truncates to 0 (unprobed)"
            raise NoValueError(msg)
        if d > LAST_DAY:
            if i == 0 and d < LAST_DAY + 1:
                msg = "a first date past 2958465 that truncates to it (unprobed)"
                raise NoValueError(msg)
            msg = "a date past 9999-12-31: #NUM!"
            raise DomainError(msg)
        day = int(mpmath.floor(d))
        if days and day < days[0]:
            msg = "a date before the first: #NUM!"
            raise DomainError(msg)
        days.append(day)
    return days


def _xnpv(args: Sequence[MathArg]) -> mpmath.mpf:
    r, flows, dates = args[0], _array(args[1]), _array(args[2])
    assert is_real(r)
    if len(flows) != len(dates):
        msg = "values and dates differ in size: #NUM!"
        raise DomainError(msg)
    if r <= 0:
        msg = "a rate at or below 0: #NUM!"
        raise DomainError(msg)
    days = _days(dates)
    u = mpmath.log1p(r)
    times = [mpmath.mpf(d - days[0]) / DAYS_PER_YEAR for d in days]
    if _log2_growth(u, max(times), (LOG2_MAX,)) >= LOG2_MAX:
        msg = "a discount past the double range (unprobed)"
        raise NoValueError(msg)
    return mpmath.fsum(v * mpmath.exp(-t * u) for v, t in zip(flows, times, strict=True))


def _mirr_powers(r: mpmath.mpf, n: int) -> None:
    """DomainError when |1 + r|^(n - 1) is past the double range (so is ^n); NoValueError when only ^n is, or either
    is below the normal range."""
    below, at = _rate_power(r, n - 1), _rate_power(r, n)
    if below >= LOG2_MAX:
        msg = "a power past the double range: #DIV/0! or #NUM!"
        raise DomainError(msg)
    if at >= LOG2_MAX or min(below, at) < LOG2_NORMAL:
        msg = "a power at the double range's edge or below the normal range (unprobed)"
        raise NoValueError(msg)


def _mirr(args: Sequence[MathArg]) -> mpmath.mpf:
    flows, fr, rr = _array(args[0]), args[1], args[2]
    assert is_real(fr)
    assert is_real(rr)
    n = len(flows)
    if n < 2 or not any(v < 0 for v in flows):
        msg = "one flow, or no negative flow: #DIV/0!"
        raise DomainError(msg)
    if rr == -1:
        msg = "a reinvest rate of -1: #DIV/0!"
        raise DomainError(msg)
    if fr == -1:
        if any(v < 0 for v in flows[1:]):
            msg = "a finance rate of -1 with a negative flow after the first: #DIV/0!"
            raise DomainError(msg)
        if any(v == 0 for v in flows[1:]):
            msg = "a finance rate of -1 with a zero flow after the first (unprobed)"
            raise NoValueError(msg)
    else:
        _mirr_powers(fr, n)
    _mirr_powers(rr, n)
    grow, discount = 1 + rr, 1 + fr
    positive = mpmath.fsum(v * grow ** (n - 1 - i) for i, v in enumerate(flows) if v > 0)
    negative = mpmath.fsum(v / discount**i for i, v in enumerate(flows) if v < 0)
    ratio = positive / -negative
    if ratio < 0:
        if n == MIRR_SQRT_FLOWS:
            msg = "a negative ratio under a square root: #NUM!"
            raise DomainError(msg)
        msg = "a negative ratio over other than three flows (unprobed)"
        raise NoValueError(msg)
    if ratio == 0:
        return mpmath.mpf(-1)
    return mpmath.root(ratio, n - 1) - 1


def _fvschedule(args: Sequence[MathArg]) -> mpmath.mpf:
    principal, schedule = args[0], _array(args[1])
    assert is_real(principal)
    factors = mpmath.mpf(1)
    value = principal
    for s in schedule:
        factors *= 1 + s
        value *= 1 + s
        for x in (factors, value):
            if abs(x) > NORMAL_MAX or (x != 0 and abs(x) < NORMAL_MIN):
                msg = "a running product past the double range or below the normal range (unprobed)"
                raise NoValueError(msg)
    return value


# -- draws -----------------------------------------------------------------------------------------


def _max_rate(periods: float) -> float:
    """The largest rate whose (1 + rate)^periods stays well inside the double range (2^1000)."""
    return float(2.0 ** (1000 / max(periods, 1.0))) - 1


def _row(a: Arg) -> tuple[float, ...]:
    """A drawn one-row array's numbers."""
    assert isinstance(a, tuple)
    return tuple(float(x) for x in a if not isinstance(x, tuple))


def _npv_rate(stratum: str, rng: random.Random, periods: float, positive: bool) -> float:
    """A discount rate for flows over `periods` (at or below 0 only when not `positive`)."""
    if stratum == "tail":
        shape = rng.randrange(3)
        if shape == 0:
            tiny = log_uniform(rng, 1e-12, 1e-4)
            return tiny if positive else signed(rng, tiny)
        if shape == 1 and not positive:
            return -1 + log_uniform(rng, 1e-3, 0.5)  # near -1
        return rng.uniform(1, min(5.0, _max_rate(periods)))
    if stratum == "extreme":
        top = min(1e6, _max_rate(periods))
        return log_uniform(rng, min(1.0, top / 2), top)
    if stratum == "log-uniform":
        r = log_uniform(rng, 1e-10, min(10.0, _max_rate(periods)))
        return -min(r, 0.9) if not positive and rng.random() < 0.3 else r
    return log_uniform(rng, 1e-3, 0.3)


def _npv_like(stratum: str, rng: random.Random) -> list[Arg]:
    (flows,) = _irr_draw(stratum, rng)
    assert isinstance(flows, tuple)
    return [_npv_rate(stratum, rng, len(flows), positive=False), flows]


def _small_flows(rng: random.Random, n: int) -> tuple[float, ...]:
    return tuple(signed(rng, log_uniform(rng, 1, 1e4)) for _ in range(n))


NPV_EDGES: list[Shape] = [
    lambda rng: [-1.0, _small_flows(rng, rng.randint(1, 5))],  # #DIV/0!
    lambda rng: [rng.choice((-2.0, -3.0, -1.5)), _small_flows(rng, rng.randint(1, 6))],  # below -1: a value
    lambda rng: [rng.choice((1e200, 1e300, 9e307)), _small_flows(rng, rng.randint(3, 4))],  # past the range
    lambda rng: [
        rng.choice((1e154, 1e200, 1e300)),
        (1.0, rng.choice((1.0, 1e300))),
    ],  # the last power is never formed
    lambda rng: [0.0, _small_flows(rng, rng.randint(1, 10))],  # rate 0: the sum
    lambda rng: [log_uniform(rng, 1e-3, 0.3), _small_flows(rng, 1)],
    lambda rng: [log_uniform(rng, 1e-3, 0.3), (0.0, *_small_flows(rng, rng.randint(1, 5)), 0.0)],
    lambda rng: [rng.choice((1.0, -0.5)), tuple(float(i) for i in range(1, rng.choice((50, 200, 254))))],
]


def _xnpv_like(stratum: str, rng: random.Random) -> list[Arg]:
    flows, dates = _xirr_draw(stratum, rng)
    days = _row(dates)
    span = (days[-1] - days[0]) / DAYS_PER_YEAR
    return [_npv_rate(stratum, rng, span, positive=True), flows, dates]


def _dated(rng: random.Random, n: int, gap: int = 400) -> tuple[int, ...]:
    """`n` increasing integer dates from a drawn first date."""
    dates = [rng.randint(*FIRST_DATE)]
    for _ in range(n - 1):
        dates.append(dates[-1] + rng.randint(1, gap))
    return tuple(dates)


def _xnpv_edge_dates(rng: random.Random) -> list[Arg]:
    n = rng.randint(2, 6)
    flows = _small_flows(rng, n)
    dates: list[float] = list(_dated(rng, n))
    shape = rng.randrange(6)
    if shape == 0:
        dates[rng.randrange(1, n)] = dates[0]  # a date equal to the first: a value
    elif shape == 1:
        dates[rng.randrange(1, n)] = dates[0] - rng.randint(1, 100)  # before the first: #NUM!
    elif shape == 2:
        dates[rng.randrange(n)] = rng.choice((0, -1, -45000))  # #NUM!
    elif shape == 3:
        dates = [d + rng.choice((0.25, 0.5, 0.9)) for d in dates]  # truncated
    elif shape == 4:
        later = dates[1:]
        rng.shuffle(later)  # later dates in any order
        dates = [dates[0], *later]
    else:
        dates[-1] = rng.choice((float(LAST_DAY), LAST_DAY + 0.5, float(LAST_DAY + 1)))
    return [log_uniform(rng, 1e-3, 0.3), flows, tuple(dates)]


XNPV_EDGES: list[Shape] = [
    lambda rng: [rng.choice((0.0, -0.5, -1.0, -2.0)), _small_flows(rng, 3), _dated(rng, 3)],  # #NUM!
    lambda rng: [log_uniform(rng, 1e-3, 0.3), _small_flows(rng, 3), _dated(rng, 4)],  # sizes differ: #NUM!
    lambda rng: [log_uniform(rng, 1e-3, 0.3), _small_flows(rng, 1), _dated(rng, 1)],
    _xnpv_edge_dates,
    _xnpv_edge_dates,
    lambda rng: [rng.choice((1e-15, 1e-300, 1e10, 1e100)), _small_flows(rng, 3), _dated(rng, 3)],
]


def _mirr_rates(stratum: str, rng: random.Random, n: int) -> tuple[float, float]:
    periods = float(n)
    if stratum == "tail":
        pick = [
            lambda: signed(rng, log_uniform(rng, 1e-12, 1e-4)),
            lambda: -1 + log_uniform(rng, 1e-2, 0.5),
            lambda: rng.uniform(1, min(5.0, _max_rate(periods))),
        ]
        return rng.choice(pick)(), rng.choice(pick)()
    if stratum == "extreme":
        top = min(1e6, _max_rate(periods))
        return log_uniform(rng, min(1.0, top / 2), top), log_uniform(rng, min(1.0, top / 2), top)
    if stratum == "log-uniform":
        top = min(10.0, _max_rate(periods))
        return tuple(  # type: ignore[return-value]
            -min(log_uniform(rng, 1e-10, 0.9), 0.9) if rng.random() < 0.3 else log_uniform(rng, 1e-10, top)
            for _ in range(2)
        )
    return log_uniform(rng, 1e-3, 0.3), log_uniform(rng, 1e-3, 0.3)


def _mirr_like(stratum: str, rng: random.Random) -> list[Arg]:
    flows = _row(_irr_draw(stratum, rng)[0])
    if not any(v < 0 for v in flows):  # the edge's one-sign series: the negative one (all-negative is -1)
        flows = tuple(-v for v in flows)
    fr, rr = _mirr_rates(stratum, rng, len(flows))
    return [flows, fr, rr]


def _mirr_flows(rng: random.Random) -> tuple[float, ...]:
    """An outlay, then 1 to 5 flows of which one may turn negative (a later negative flow) when another stays
    positive."""
    n = rng.randint(2, 6)
    flows = [-log_uniform(rng, 1, 1e4)] + [log_uniform(rng, 1, 1e4) for _ in range(n - 1)]
    if n > 2 and rng.random() < 0.5:  # a positive flow stays
        flows[rng.randrange(1, n)] *= -1
    return tuple(flows)


def _rate(rng: random.Random) -> float:
    return log_uniform(rng, 1e-3, 0.3)


MIRR_EDGES: list[Shape] = [
    lambda rng: [_small_flows(rng, 1), _rate(rng), _rate(rng)],  # one flow: #DIV/0!
    lambda rng: [
        tuple(log_uniform(rng, 1, 1e4) for _ in range(3)),
        _rate(rng),
        _rate(rng),
    ],  # no negative: #DIV/0!
    lambda rng: [
        tuple(-log_uniform(rng, 1, 1e4) for _ in range(3)),
        _rate(rng),
        _rate(rng),
    ],  # all negative: -1
    lambda rng: [_mirr_flows(rng), _rate(rng), -1.0],  # #DIV/0!
    lambda rng: [_mirr_flows(rng), -1.0, _rate(rng)],  # #DIV/0! with a later negative flow, else a value
    lambda rng: [
        rng.choice(((-100.0, 50.0, 80.0), (-50.0, 100.0), (-100.0, 50.0, -80.0, 120.0))),
        -1.0,
        _rate(rng),
    ],
    lambda rng: [_mirr_flows(rng), rng.choice((-2.0, -1.5, -3.0)), rng.choice((-2.0, -1.5, _rate(rng)))],
    lambda rng: [(50.0, -100.0, 80.0), -2.0, _rate(rng)],  # a negative ratio: #NUM!
    lambda rng: [_mirr_flows(rng), rng.choice((1e200, _rate(rng))), rng.choice((1e200, _rate(rng)))],
    lambda rng: [(-log_uniform(rng, 1, 1e4), log_uniform(rng, 1, 1e4)), _rate(rng), _rate(rng)],  # exponent 1
    lambda rng: [(-100.0, 120.0), _rate(rng), 0.03],  # "MIRR's evaluation order"
]


def _schedule(stratum: str, rng: random.Random) -> tuple[float, ...]:
    if stratum == "tail":
        n = rng.randint(1, 20)
        return tuple(
            signed(rng, log_uniform(rng, 1e-12, 1e-4))
            if rng.random() < 0.7
            else -1 + log_uniform(rng, 1e-6, 0.1)
            for _ in range(n)
        )
    if stratum == "extreme":
        n = rng.randint(50, 200)
        top = _max_rate(n) / 2
        return tuple(rng.uniform(-0.5, top) for _ in range(n))
    if stratum == "log-uniform":
        n = round(log_uniform(rng, 1, 100))
        return tuple(
            -log_uniform(rng, 1e-10, 0.9)
            if rng.random() < 0.3
            else log_uniform(rng, 1e-10, min(10.0, _max_rate(n)))
            for _ in range(n)
        )
    return tuple(rng.uniform(-0.05, 0.2) for _ in range(rng.randint(1, 20)))


def _principal(stratum: str, rng: random.Random) -> float:
    if stratum == "extreme":
        return signed(rng, log_uniform(rng, 1e-200, 1e200))
    if stratum == "log-uniform":
        return signed(rng, log_uniform(rng, 1e-10, 1e12))
    return signed(rng, log_uniform(rng, 1, 1e6))


def _fvschedule_like(stratum: str, rng: random.Random) -> list[Arg]:
    return [_principal(stratum, rng), _schedule(stratum, rng)]


FVSCHEDULE_EDGES: list[Shape] = [
    lambda rng: [_principal("typical", rng), (rng.uniform(0, 0.2), -1.0, rng.uniform(0, 0.2))],  # 0
    lambda rng: [0.0, _schedule("typical", rng)],
    lambda rng: [_principal("typical", rng), (0.0,) * rng.randint(1, 5)],  # the principal
    lambda rng: [
        _principal("typical", rng),
        tuple(rng.choice((-2.0, -1.5, -3.0)) for _ in range(rng.randint(1, 4))),
    ],
    lambda rng: [_principal("typical", rng), (rng.uniform(-0.05, 0.2),)],
    lambda rng: [1e300, (rng.uniform(-0.9, 0.5),)],
    lambda rng: [_principal("typical", rng), (rng.choice((1e-15, 1e-300, 1e100)),)],
]


def _definition(
    function: str,
    params: tuple[Param, ...],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    draws: tuple[Callable[[str, random.Random], list[Arg]], Sequence[Shape]],
    sources: tuple[str, ...],
) -> Definition:
    make, edges = draws
    names = ", ".join(p.name for p in params)
    return register(
        Definition(
            function=function,
            params=params,
            value=value,
            draw=_strata(make, edges, f"{function}: ({names}) by stratum."),
            sources=sources,
        )
    )


NPV = _definition(
    "NPV",
    (
        Param("rate", "real", source=NPV_RANGE_SOURCE),
        Param("value", "array", source=NPV_ARGS_SOURCE),
    ),
    _npv,
    (_npv_like, NPV_EDGES),
    (
        NPV_ORDER_SOURCE,
        NPV_RANGE_SOURCE,
        NPV_DIV0_SOURCE,
        NPV_ARGS_SOURCE,
        NPV_ERROR_ORDER_SOURCE,
        NPV_ERRORS_SOURCE,
    ),
)
XNPV = _definition(
    "XNPV",
    (
        Param("rate", "real", source=XNPV_RATE_SOURCE),
        Param("values", "array", source=XNPV_ELEMENT_SOURCE),
        Param("dates", "array", source=XNPV_ELEMENT_SOURCE),
    ),
    _xnpv,
    (_xnpv_like, XNPV_EDGES),
    (
        XNPV_POWER_SOURCE,
        XNPV_ELEMENT_SOURCE,
        XNPV_RATE_SOURCE,
        XNPV_SIZE_SOURCE,
        XNPV_EMPTY_SOURCE,
        XNPV_FIRST_SOURCE,
        WHOLE_DATA_SOURCE,
    ),
)
MIRR = _definition(
    "MIRR",
    (
        Param("values", "array", source=MIRR_ORDER_SOURCE),
        Param("finance_rate", "real", source=MIRR_EDGES_SOURCE),
        Param("reinvest_rate", "real", source=MIRR_EDGES_SOURCE),
    ),
    _mirr,
    (_mirr_like, MIRR_EDGES),
    (MIRR_ORDER_SOURCE, MIRR_EDGES_SOURCE, MIRR_OVERFLOW_SOURCE, NPV_ERROR_ORDER_SOURCE),
)
FVSCHEDULE = _definition(
    "FVSCHEDULE",
    (
        Param("principal", "real", source=NPV_ARGS_SOURCE),
        Param("schedule", "array", source=NPV_ARGS_SOURCE),
    ),
    _fvschedule,
    (_fvschedule_like, FVSCHEDULE_EDGES),
    (NPV_ARGS_SOURCE, FVSCHEDULE_EMPTY_SOURCE, FVSCHEDULE_OVERFLOW_SOURCE, WHOLE_DATA_SOURCE),
)
