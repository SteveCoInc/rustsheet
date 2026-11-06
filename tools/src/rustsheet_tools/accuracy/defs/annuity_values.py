"""FV, PV, PMT, IPMT, PPMT, ISPMT and PDURATION (Gate A, T-4109, decision 0273).

The math, on the doubles of the written arguments, with `1 + rate` exact (u = ln(1 + rate) at the working precision,
P = (1 + rate)^nper = e^(nper·u), k = 1 + rate·type):
- FV = -(pv·P + pmt·k·(P - 1)/rate), and -(pv + pmt·nper) at rate 0;
- PV = -(fv + pmt·k·(P - 1)/rate)/P, and -(fv + pmt·nper) at rate 0;
- PMT = -(fv + pv·P)·rate/(k·(P - 1)), and -(pv + fv)/nper at rate 0;
- IPMT and PPMT split PMT: with A = (pv + fv)/(1 - P^-1), x = (per - 1 - nper)·u and d = rate (type 0) or
  rate/(1 + rate) (type 1), IPMT = -(A·(1 - e^x) - fv)·d and PPMT = -A·e^x·d; in period 1 of type 1 IPMT is 0 and
  PPMT is PMT; at rate 0 IPMT is 0 and PPMT -(pv + fv)/nper;
- ISPMT = (per - nper)·pv·rate/nper;
- PDURATION = (ln fv - ln pv)/ln(1 + rate).
FV, PV and PMT reuse `present_value` (defs/annuity.py, T-3535), the flows' value at u: PV is -present_value(u, nper,
pmt, 0, fv), FV is -P·present_value(u, nper, pmt, pv, 0) and PMT is -present_value(u, nper, 0, pv, fv) over
present_value(u, nper, 1, 0, 0).

Sources (financial.md):
- "Annuity evaluation order and `(1+r)^n` (PV/FV/PMT/NPER/IPMT/PPMT)" and "FV and PV round each operation twice": the
  PV and FV forms; Excel's own value is those forms in twice-rounded doubles, so a sum that cancels is off the math
  and passes through Excel's 15 digits.
- "PMT, IPMT and PPMT are the exact payment": PMT, IPMT and PPMT are the exact payment on the double arguments, with
  `1+rate` exact, for integer and fractional nper and period; the balance B there gives IPMT's type-1 form, which the
  tests check against the split form here.
- "PMT is a discounted payment", "PPMT discounts with an atanh-series log", "IPMT cancels on fv", "IPMT follows
  Excel's own discount where it moves the value" and "IPMT and PPMT split Excel's discounted payment": Excel's
  operation order (a few ulp, or more where IPMT cancels on fv or PPMT's discount leaves the normal range, where
  PPMT is 0 and the absolute floor of 1e-300 covers the math).
- "ISPMT's operation order": ISPMT has no domain on per (`ISPMT(1E-10,361,124,371658)` is a number, gen-m9
  d912671baa1364aa A215).
Domains (Excel's errors, `DomainError`), each evidenced:
- PV: rate -1 with nper 0 is `#NUM!` (0^0, `PV(-1,0,-713)`); a P below the normal range is `#DIV/0!` (rate -1 with
  nper > 0, `PV(-0.5,1023,-1,1)`; "Annuity and DB argument domains at their limits", "PV, PMT and IRR near a rate of
  -1"); a P past the double range is `#NUM!` (`PV(100,184,1)`, `PV(1E+300,2,-421)`; "PMT when (1+rate)^nper
  overflows or underflows").
- FV: a P past the double range is `#NUM!` (`FV(100,184,1)`, `FV(1E+300,2,-421)`); rate -1 is a value
  (`FV(-1,5,-421)` 421, t2657 P!A41).
- PMT: rate -1 and a P below the normal range are `#NUM!` (`PMT(-1,5,-421)`, `PMT(-0.5,1070,1)`, `PMT(1,-2000,1)`;
  t2657 P!I1:L29); a P past the double range is a value (the quotient divided through by P).
- IPMT and PPMT: a period below 1 or at or past nper + 1 is `#NUM!` (gen-m9 9a1fa5366bc50918 A272
  `IPMT(0.5,12,0,347880)`, 5c6a23c4f6597f1c A8 `IPMT(1E+300,0.2,161,210756,227)`, a91bad4e84f49739 A354
  `PPMT(1,0.5,323,401723)`, c0681a8b150ce301 A375 `PPMT(0.1,9,1.5,404118,62,1)`), while a period between nper and
  nper + 1 is the closed form at that period (financial.md "IPMT and PPMT take a period up to nper + 1": accuracy/PPMT/s1
  A112 and A118 are values, A128 and A139 at nper + 1 `#NUM!`; IPMT's band is unprobed but keeps the closed form,
  T-4516); a P whose reciprocal overflows is `#NUM!` but for IPMT in period 1 of type 1, which is 0
  (`IPMT(-0.5,1,2000,1)`, `PPMT(-0.5,1,2000,1)`; "PMT when (1+rate)^nper overflows or underflows").
- PDURATION: a rate, pv or fv of 0 is `#NUM!` (gen-m9 31474c20fc1dafd6 A194 `PDURATION(0,7968,32308)`,
  9a1fa5366bc50918 A196 `PDURATION(100,0,49370)`; fv: financial.md "M9 financial argument kinds", a blank fv), and a
  rate so small that 1 + rate is 1 is `#DIV/0!` (gen-m9 e168be7f6d3f84c2 A180 `PDURATION(1E-300,4938,46197)`). A
  negative rate, pv or fv is a domain error here as the logarithms' domain; unprobed before this table, whose edge
  stratum draws them (the Outcome of T-4109 and financial.md "PDURATION's domain" record what Excel gave).
Not drawn and given no math value (`NoValueError`), as no evidence names Excel's result: a rate below -1; nper 0 for
PMT, IPMT, PPMT and ISPMT; nper at most 0 at rate -1; PDURATION where 1 + rate is 1 and fv equals pv (0/0, T-4516);
IPMT and PPMT at rate -1; P within a hair of the normal range's bottom or the double range's top (Excel's double P and
the exact one may fall on either side); a subnormal P whose reciprocal is finite for IPMT and PPMT; an FV or PV term
past the double range under a finite P. IPMT is `#NUM!` on
some points at |rate| from 1E-300 to 1E-12 and near pv or fv of 1E+290 ("IPMT near the double range and at tiny rates",
T-4314); the draws stay at |rate| >= 1E-12 and amounts below 1E+15, and such eval cells keep their math value.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import Shape, by_stratum_with_edge, scalars, signed
from rustsheet_tools.accuracy.defs.annuity import draw_annuity, present_value
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)
from rustsheet_tools.accuracy.sampler import log_uniform

ORDER_SOURCE = "financial.md#annuity-evaluation-order-and-1rn-pvfvpmtnperipmtppmt"
TWICE_SOURCE = "financial.md#fv-and-pv-round-each-operation-twice"
EXACT_SOURCE = "financial.md#pmt-ipmt-and-ppmt-are-the-exact-payment"
DISCOUNT_SOURCE = "financial.md#pmt-is-a-discounted-payment"
SPLIT_SOURCE = "financial.md#ipmt-and-ppmt-split-excels-discounted-payment"
CANCEL_SOURCE = "financial.md#ipmt-cancels-on-fv"
RANGE_SOURCE = "financial.md#pmt-when-1ratenper-overflows-or-underflows"
NEAR_MINUS_ONE_SOURCE = "financial.md#pv-pmt-and-irr-near-a-rate-of--1"
LIMITS_SOURCE = "financial.md#annuity-and-db-argument-domains-at-their-limits"
ISPMT_SOURCE = "financial.md#ispmts-operation-order"
PDURATION_SOURCE = "financial.md#pdurations-domain"
KINDS_SOURCE = "financial.md#m9-financial-argument-kinds-t-3184"
# any non-zero type is the period's start
EDGE_POOL_SOURCE = "financial.md#financial-edge-pool-smoke-check-t-2248"
PERIOD_SOURCE = "financial.md#ipmt-and-ppmt-take-a-period-up-to-nper--1"

LOG2_MAX = 1024.0  # P at or past 2^1024 is past the double range
LOG2_NORMAL = -1022.0  # P below 2^-1022 is below the normal range
LOG2_RECIPROCAL = -1024.0  # below about 2^-1024 the reciprocal 1/P overflows
BAND = 1e-6  # log2 P this close to a cut: Excel's double P may be on either side (no value)
ON_CUT = 1e-30  # closer than this, log2 P is the cut itself (rounding of the working precision)
NORMAL_MAX = 1.7976931348623157e308


def _type(args: Sequence[mpmath.mpf], i: int) -> bool:
    """The type argument at `i` (0 when the call leaves it out): any non-zero is the start of the period."""
    return len(args) > i and args[i] != 0


def _amount(args: Sequence[mpmath.mpf], i: int) -> mpmath.mpf:
    """The optional amount at `i` (0 when the call leaves it out)."""
    return args[i] if len(args) > i else mpmath.mpf(0)


def _log2_growth(u: mpmath.mpf, n: mpmath.mpf, cuts: Sequence[float]) -> mpmath.mpf:
    """log2 of P = e^(n·u); NoValueError within `BAND` of one of the function's `cuts` but not on it (a P exactly
    on a cut, a power of two, is the same double in Excel and the math)."""
    log2 = n * u / mpmath.log(2)
    for cut in cuts:
        if abs(log2 - cut) <= ON_CUT:
            return mpmath.mpf(cut)
        if abs(log2 - cut) < BAND:
            msg = "(1 + rate)^nper at a cut of the double range"
            raise NoValueError(msg)
    return log2


def _below_minus_one(r: mpmath.mpf) -> None:
    if r < -1:
        msg = "a rate below -1 (unprobed)"
        raise NoValueError(msg)


def _finite(x: mpmath.mpf, what: str) -> None:
    """NoValueError when an intermediate is past the double range under a finite P (unprobed)."""
    if abs(x) > NORMAL_MAX:
        msg = f"{what} past the double range"
        raise NoValueError(msg)


def _fv(args: Sequence[MathArg]) -> mpmath.mpf:
    a = scalars(args)
    r, n, pmt = a[0], a[1], a[2]
    pv, due = _amount(a, 3), _type(a, 4)
    _below_minus_one(r)
    if r == -1:  # P = 0: FV is -pmt·k (FV(-1,5,-421) is 421)
        if n <= 0:
            msg = "rate -1 with nper at most 0 (unprobed)"
            raise NoValueError(msg)
        return -pmt * (0 if due else 1)
    if r == 0:
        return -(pv + pmt * n)
    u = mpmath.log1p(r)
    if _log2_growth(u, n, (LOG2_MAX,)) >= LOG2_MAX:
        msg = "(1 + rate)^nper past the double range: #NUM!"
        raise DomainError(msg)
    growth = mpmath.exp(n * u)
    _finite(pv * growth, "pv·P")
    _finite(pmt * (1 + r * (1 if due else 0)) * mpmath.expm1(n * u) / r, "the payments' term")
    return -growth * present_value(u, n, pmt, pv, mpmath.mpf(0), due=due)


def _pv(args: Sequence[MathArg]) -> mpmath.mpf:
    a = scalars(args)
    r, n, pmt = a[0], a[1], a[2]
    fv, due = _amount(a, 3), _type(a, 4)
    _below_minus_one(r)
    if r == -1:
        if n < 0:
            msg = "rate -1 with a negative nper (unprobed)"
            raise NoValueError(msg)
        msg = "0^0: #NUM!" if n == 0 else "P = 0: #DIV/0!"
        raise DomainError(msg)
    if r == 0:
        return -(fv + pmt * n)
    u = mpmath.log1p(r)
    log2 = _log2_growth(u, n, (LOG2_MAX, LOG2_NORMAL))
    if log2 >= LOG2_MAX:
        msg = "(1 + rate)^nper past the double range: #NUM!"
        raise DomainError(msg)
    if log2 < LOG2_NORMAL:
        msg = "(1 + rate)^nper below the normal range: #DIV/0!"
        raise DomainError(msg)
    _finite(pmt * (1 + r * (1 if due else 0)) * mpmath.expm1(n * u) / r, "the payments' term")
    return -present_value(u, n, pmt, mpmath.mpf(0), fv, due=due)


def _payment(r: mpmath.mpf, n: mpmath.mpf, pv: mpmath.mpf, fv: mpmath.mpf, due: bool) -> mpmath.mpf:
    """PMT at a rate above -1 other than 0, P inside the normal range or past the top.

    The payment is the quotient of two present values at u = ln(1 + rate): the flows' value with no payment over
    the value of a unit payment, -PV(pv, fv)/PV(1). That is PMT = -(fv + pv·P)·rate/(k·(P - 1)) divided through by P,
    which stays finite where P is past the double range (RANGE_SOURCE: PMT is a value there, the quotient divided
    through by P). The math is the exact payment on the double arguments with `1 + rate` exact (EXACT_SOURCE), not
    any one operation order: T-1528 found that Excel's PMT is not the accurate value (ORDER_SOURCE: the expm1 form
    `-(pv*P+fv)/((P-1)/r*k)` fixes 30 Excel-cached cells and regresses 22), and Excel's discounted form
    (DISCOUNT_SOURCE) is a few ulps from this value, which the envelope absorbs.
    """
    u = mpmath.log1p(r)
    return -present_value(u, n, mpmath.mpf(0), pv, fv, due=due) / present_value(
        u, n, mpmath.mpf(1), mpmath.mpf(0), mpmath.mpf(0), due=due
    )


def _pmt(args: Sequence[MathArg]) -> mpmath.mpf:
    a = scalars(args)
    r, n, pv = a[0], a[1], a[2]
    fv, due = _amount(a, 3), _type(a, 4)
    _below_minus_one(r)
    if n == 0 or (r == -1 and n < 0):
        msg = "nper 0, or rate -1 with a negative nper (unprobed)"
        raise NoValueError(msg)
    if r == -1:
        msg = "rate -1: #NUM!"
        raise DomainError(msg)
    if r == 0:
        return -(pv + fv) / n
    if _log2_growth(mpmath.log1p(r), n, (LOG2_NORMAL,)) < LOG2_NORMAL:
        msg = "(1 + rate)^nper below the normal range: #NUM!"
        raise DomainError(msg)
    return _payment(r, n, pv, fv, due)


def split(args: Sequence[MathArg]) -> tuple[mpmath.mpf, mpmath.mpf]:
    """(IPMT, PPMT) of IPMT's and PPMT's arguments (rate, per, nper, pv, [fv], [type])."""
    a = scalars(args)
    r, per, n, pv = a[0], a[1], a[2], a[3]
    fv, due = _amount(a, 4), _type(a, 5)
    if per < 1 or per >= n + 1:
        msg = "a period outside [1, nper + 1): #NUM!"
        raise DomainError(msg)
    _below_minus_one(r)
    if r == -1:
        msg = "rate -1 (unprobed)"
        raise NoValueError(msg)
    if r == 0:
        return mpmath.mpf(0), -(pv + fv) / n
    u = mpmath.log1p(r)
    log2 = _log2_growth(u, n, (LOG2_RECIPROCAL, LOG2_NORMAL))
    if log2 < LOG2_RECIPROCAL:
        if per == 1 and due:
            return mpmath.mpf(0), mpmath.mpf(0)  # IPMT is 0 here; PPMT is PMT's #NUM! (checked by the caller)
        msg = "1/(1 + rate)^nper overflows: #NUM!"
        raise DomainError(msg)
    if log2 < LOG2_NORMAL:
        msg = "a subnormal (1 + rate)^nper with a finite reciprocal (unprobed)"
        raise NoValueError(msg)
    if per == 1 and due:
        return mpmath.mpf(0), _payment(r, n, pv, fv, due)
    balance = (pv + fv) / -mpmath.expm1(-n * u)
    x = (per - 1 - n) * u
    d = r / (1 + r) if due else r
    return -(balance * -mpmath.expm1(x) - fv) * d, -balance * mpmath.exp(x) * d


def _ipmt(args: Sequence[MathArg]) -> mpmath.mpf:
    # IPMT at a period strictly between nper and nper + 1 is unprobed (PERIOD_SOURCE) but keeps the closed form, as
    # PPMT's evidenced band does: a NoValueError there would drop the math of probes/t4314-ipmt-balance-overflow
    # D1:D2 (per 1, nper 1E-10, Excel #NUM! for T-4314's reason). T-4516's Outcome records why.
    return split(args)[0]


def _ppmt(args: Sequence[MathArg]) -> mpmath.mpf:
    a = scalars(args)
    if _type(a, 5) and a[1] == 1 and a[0] > -1 and a[0] != 0:
        n = a[2]
        if n >= 1 and _log2_growth(mpmath.log1p(a[0]), n, (LOG2_RECIPROCAL,)) < LOG2_RECIPROCAL:
            msg = "PPMT in period 1 of type 1 is PMT, #NUM! here"
            raise DomainError(msg)
    return split(args)[1]


def _ispmt(args: Sequence[MathArg]) -> mpmath.mpf:
    r, per, n, pv = scalars(args)
    if n == 0:
        msg = "nper 0 (unprobed)"
        raise NoValueError(msg)
    return (per - n) * pv * r / n


def _pduration(args: Sequence[MathArg]) -> mpmath.mpf:
    r, pv, fv = scalars(args)
    if r <= 0 or pv <= 0 or fv <= 0:
        msg = "a rate, pv or fv at most 0: #NUM!"
        raise DomainError(msg)
    if 1.0 + float(r) == 1.0:
        if fv == pv:  # (LN fv - LN pv) / LN(1) is 0/0; only fv != pv is evidenced (PDURATION_SOURCE)
            msg = "1 + rate is 1 with fv equal to pv (unprobed)"
            raise NoValueError(msg)
        msg = "1 + rate is 1: #DIV/0!"
        raise DomainError(msg)
    return (mpmath.log(fv) - mpmath.log(pv)) / mpmath.log1p(r)


# -- draws -----------------------------------------------------------------------------------------


def _rate(stratum: str, rng: random.Random, r: float) -> float:
    """The tail stratum's rate: near 0 (either sign, |rate| from 1E-12) or large; otherwise `r`."""
    if stratum != "tail":
        return r
    return signed(rng, log_uniform(rng, 1e-12, 1e-4)) if rng.random() < 0.5 else rng.uniform(0.5, 5)


def _base(stratum: str, rng: random.Random) -> tuple[float, int, float, float, float, int]:
    """(rate, nper, pmt, pv, fv, type) of a stratum: RATE's and NPER's annuity (`draw_annuity`), with a payment drawn
    on its own (not the one that zeroes the flows, which would make FV and PV cancel)."""
    sign, n, r, pv, fv, due = draw_annuity(stratum, rng)
    pmt = -sign * abs(pv) * log_uniform(rng, 1e-4, 0.5)
    return _rate(stratum, rng, r), n, pmt, pv, fv, 1 if due else 0


def _fractional(rng: random.Random, n: int) -> float:
    return n + rng.choice((0.25, 0.5, 0.731))


def _strata(
    make: Callable[[str, random.Random], list[Arg]], edges: Sequence[Shape], doc: str
) -> Callable[[str, random.Random], list[Arg]]:
    """Every stratum but the edge from `make`; the edge one of `edges` or, one time in four, `make`'s typical."""

    def edge(rng: random.Random) -> list[Arg]:
        if rng.random() < 0.25:
            return make("typical", rng)
        return rng.choice(edges)(rng)

    return by_stratum_with_edge(make, edge, doc)


def _fv_like(stratum: str, rng: random.Random) -> list[Arg]:
    r, n, pmt, pv, fv, due = _base(stratum, rng)
    del fv
    nper: float = _fractional(rng, n) if stratum == "extreme" and rng.random() < 0.3 else n
    return [r, nper, pmt, pv, due]


def _pv_like(stratum: str, rng: random.Random) -> list[Arg]:
    r, n, pmt, pv, fv, due = _base(stratum, rng)
    nper: float = _fractional(rng, n) if stratum == "extreme" and rng.random() < 0.3 else n
    return [r, nper, pmt, fv if fv != 0 else 0.0 if rng.random() < 0.5 else -pv, due]


def _pmt_like(stratum: str, rng: random.Random) -> list[Arg]:
    r, n, _pmt, pv, fv, due = _base(stratum, rng)
    nper: float = _fractional(rng, n) if stratum == "extreme" and rng.random() < 0.3 else n
    return [r, nper, pv, fv, due]


def _per_like(stratum: str, rng: random.Random) -> list[Arg]:
    r, n, _pmt, pv, fv, due = _base(stratum, rng)
    per: float = rng.randint(1, n)
    if stratum in ("extreme", "log-uniform") and rng.random() < 0.3:
        per = min(per + rng.random(), n)
    return [r, per, n, pv, fv, due]


def _edge_shapes(tail: Callable[[random.Random, float, float], list[Arg]]) -> list[Shape]:
    return [
        lambda rng: tail(rng, 0.0, rng.randint(1, 400)),
        lambda rng: tail(rng, -1.0, rng.randint(1, 50)),
        lambda rng: tail(rng, rng.choice((100.0, 1e10, 1e300, 2.0)), rng.choice((184, 2000, 3000))),
        lambda rng: tail(rng, rng.choice((-0.5, -0.9, -0.9999999999)), rng.choice((1100, 2000, 400, 100))),
        lambda rng: tail(rng, rng.choice((-0.5, -0.9)), rng.choice((1000, 300, 30))),
        lambda rng: tail(rng, rng.choice((1.0, 0.5, 2.0)), rng.choice((60, 500, 1000))),
        lambda rng: tail(rng, rng.uniform(0.01, 0.2), _fractional(rng, rng.randint(1, 100))),
        lambda rng: tail(rng, rng.uniform(0.01, 0.2), 1),
    ]


def _amounts(rng: random.Random) -> tuple[float, float, float, int]:
    """(pmt, pv, fv, type) of an edge point: modest amounts of mixed signs."""
    pv = signed(rng, log_uniform(rng, 1, 1e6))
    pmt = -math.copysign(log_uniform(rng, 1, 1e4), pv)
    fv = 0.0 if rng.random() < 0.5 else signed(rng, log_uniform(rng, 1, 1e6))
    return pmt, pv, fv, rng.randrange(2)


def _fv_edge(rng: random.Random, r: float, n: float) -> list[Arg]:
    pmt, pv, _fv, due = _amounts(rng)
    return [r, n, pmt, pv, due]


def _pv_edge(rng: random.Random, r: float, n: float) -> list[Arg]:
    pmt, _pv, fv, due = _amounts(rng)
    return [r, n, pmt, fv, due]


def _pmt_edge(rng: random.Random, r: float, n: float) -> list[Arg]:
    _pmt, pv, fv, due = _amounts(rng)
    return [r, n, pv, fv, due]


def _per_edge(rng: random.Random, r: float, n: float) -> list[Arg]:
    _pmt, pv, fv, due = _amounts(rng)
    whole = max(1, math.floor(n))
    per = rng.choice((1, whole, rng.randint(1, whole), 1.5 if whole >= 2 else 1))
    return [r, per, n, pv, fv, due]


def _per_outside(rng: random.Random) -> list[Arg]:
    """A period below 1 or at nper + 1 (`#NUM!`), or between nper and nper + 1 (a value)."""
    _pmt, pv, fv, due = _amounts(rng)
    n = rng.randint(1, 360)
    per = rng.choice((0, 0.5, n + 1, n + 0.5, -1))
    return [log_uniform(rng, 1e-3, 0.2), per, n, pv, fv, due]


def _annuity_definition(  # noqa: PLR0917 (the definition's parts, each named at the call)
    function: str,
    names: Sequence[str],
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    make: Callable[[str, random.Random], list[Arg]],
    edges: Sequence[Shape],
    sources: tuple[str, ...],
) -> Definition:
    """A definition whose last two parameters are the optional amount and type."""
    params = [Param(name, "real", Interval(), source=sources[0]) for name in names[:-2]]
    params.append(Param(names[-2], "real", Interval(), source=sources[0], optional=True))
    params.append(Param(names[-1], "int", Interval(), source=EDGE_POOL_SOURCE, optional=True))
    return register(
        Definition(
            function=function,
            params=tuple(params),
            value=value,
            draw=_strata(make, edges, f"{function}: ({', '.join(names)}) by stratum."),
            sources=sources,
        )
    )


FV = _annuity_definition(
    "FV",
    ("rate", "nper", "pmt", "pv", "type"),
    _fv,
    _fv_like,
    _edge_shapes(_fv_edge),
    (ORDER_SOURCE, TWICE_SOURCE, RANGE_SOURCE, NEAR_MINUS_ONE_SOURCE),
)
PV = _annuity_definition(
    "PV",
    ("rate", "nper", "pmt", "fv", "type"),
    _pv,
    _pv_like,
    _edge_shapes(_pv_edge),
    (ORDER_SOURCE, TWICE_SOURCE, LIMITS_SOURCE, NEAR_MINUS_ONE_SOURCE, RANGE_SOURCE),
)
PMT = _annuity_definition(
    "PMT",
    ("rate", "nper", "pv", "fv", "type"),
    _pmt,
    _pmt_like,
    _edge_shapes(_pmt_edge),
    (EXACT_SOURCE, DISCOUNT_SOURCE, RANGE_SOURCE, NEAR_MINUS_ONE_SOURCE),
)
IPMT = _annuity_definition(
    "IPMT",
    ("rate", "per", "nper", "pv", "fv", "type"),
    _ipmt,
    _per_like,
    [*_edge_shapes(_per_edge), _per_outside, _per_outside],
    (EXACT_SOURCE, SPLIT_SOURCE, CANCEL_SOURCE, RANGE_SOURCE, PERIOD_SOURCE),
)
PPMT = _annuity_definition(
    "PPMT",
    ("rate", "per", "nper", "pv", "fv", "type"),
    _ppmt,
    _per_like,
    [*_edge_shapes(_per_edge), _per_outside, _per_outside],
    (EXACT_SOURCE, SPLIT_SOURCE, CANCEL_SOURCE, RANGE_SOURCE, PERIOD_SOURCE),
)


def _ispmt_draw(stratum: str, rng: random.Random) -> list[Arg]:
    r, n, _pmt, pv, _fv, _due = _base(stratum, rng)
    per: float = rng.randint(0, n)
    if stratum == "log-uniform":
        per = rng.uniform(-n, 2 * n)
    if stratum == "extreme" and rng.random() < 0.3:
        return [r, per, _fractional(rng, n), pv]
    return [r, per, n, pv]


def _ispmt_edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(1, 400)
    pv = signed(rng, log_uniform(rng, 1, 1e6))
    r = log_uniform(rng, 1e-3, 0.3)
    shapes: list[list[Arg]] = [
        [0.0, rng.randint(0, n), n, pv],  # rate 0
        [r, n, n, pv],  # per = nper: 0
        [r, 0, n, pv],
        [r, n + rng.randint(1, 100), n, pv],  # past nper: no domain
        [r, -rng.randint(1, 10), n, pv],
        [r, rng.randint(0, n), n, 0.0],
        [r, rng.randint(0, n), _fractional(rng, n), pv],
        [rng.choice((1e10, 1e100, 1e300)), rng.randint(0, n), n, signed(rng, log_uniform(rng, 1, 1e4))],
        [rng.choice((1e-10, 1e-100, 1e-300)), rng.randint(0, n), n, pv],
        [r, rng.randint(0, 9), rng.choice((1e-15, 0.5, 1e15)), pv],
    ]
    return rng.choice(shapes)


def _pduration_draw(stratum: str, rng: random.Random) -> list[Arg]:
    pv = log_uniform(rng, 1, 1e6)
    fv = log_uniform(rng, 1, 1e6)
    r = log_uniform(rng, 1e-3, 0.2)
    if stratum == "tail":  # tiny rates (1 + rate still not 1) or large ones
        r = log_uniform(rng, 2.3e-16, 1e-6) if rng.random() < 0.5 else log_uniform(rng, 1, 1e300)
    elif stratum == "extreme":
        pv, fv = log_uniform(rng, 1e-300, 1e300), log_uniform(rng, 1e-300, 1e300)
    elif stratum == "log-uniform":
        r = log_uniform(rng, 1e-12, 1e12)
        pv, fv = log_uniform(rng, 1e-6, 1e12), log_uniform(rng, 1e-6, 1e12)
    return [r, pv, fv]


def _pduration_edge(rng: random.Random) -> list[Arg]:
    pv = log_uniform(rng, 1, 1e6)
    fv = log_uniform(rng, 1, 1e6)
    r = log_uniform(rng, 1e-3, 0.2)
    shapes: list[list[Arg]] = [
        [0.0, pv, fv],  # #NUM!
        [rng.choice((1e-300, 1e-17, 1.1e-16)), pv, fv],  # 1 + rate is 1: #DIV/0!
        [rng.choice((2.3e-16, 4.5e-16, 1e-15)), pv, fv],  # just past it
        [r, pv, pv],  # 0
        [r, 0.0, fv],
        [r, pv, 0.0],
        [-r, pv, fv],  # negative: unprobed before this table
        [r, -pv, fv],
        [r, pv, -fv],
        [rng.uniform(0.01, 0.99) - 1, pv, fv],
    ]
    return rng.choice(shapes)


ISPMT = register(
    Definition(
        function="ISPMT",
        params=tuple(
            Param(name, "real", Interval(), source=ISPMT_SOURCE) for name in ("rate", "per", "nper", "pv")
        ),
        value=_ispmt,
        draw=_strata(_ispmt_draw, [_ispmt_edge], "ISPMT: (rate, per, nper, pv) by stratum."),
        sources=(ISPMT_SOURCE,),
    )
)
PDURATION = register(
    Definition(
        function="PDURATION",
        params=tuple(
            Param(name, "real", Interval(), source=PDURATION_SOURCE) for name in ("rate", "pv", "fv")
        ),
        value=_pduration,
        draw=_strata(_pduration_draw, [_pduration_edge], "PDURATION: (rate, pv, fv) by stratum."),
        sources=(PDURATION_SOURCE, KINDS_SOURCE),
    )
)
