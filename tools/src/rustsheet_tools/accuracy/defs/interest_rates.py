"""EFFECT, NOMINAL and RRI (Gate A, T-4110, decision 0273): an interest rate converted between nominal and effective,
and the equivalent growth rate of an investment.

The math, on the doubles of the written arguments:
- EFFECT(nominal_rate, npery) = (1 + r/n)^n - 1 and NOMINAL(effect_rate, npery) = n·((1 + e)^(1/n) - 1), n = npery
  floored;
- RRI(nper, pv, fv) = (fv/pv)^(1/nper) - 1.

Sources (financial.md):
- "EFFECT and NOMINAL raise to their power with `^`": the forms, n floored (ironcalc-calc NOMINAL_EFFECT.xlsx C8
  `NOMINAL(0.5,4.5)`), Excel's `^` (a few ulps from the math; at a tiny rate `(1 + r/n)^n - 1` cancels, so Excel is off
  the math there and the point passes through its 15 digits, decision 0300), and the overflow: a power past the double
  range is `#NUM!` (`EFFECT(1E300,2)`, `EFFECT(1E13,365)`, probes/effect-overflow-power-of-ten A1-A14).
- "DURATION and RRI operation order": Excel's RRI is `EXP(LN(fv/pv)*(1/nper))-1`, and `fv/pv-1` at nper 1, a few ulps
  from the math but where the result is near 0 (EXP(x)-1 cancels); its edges (probes-numeric/m9-fin-rri-sweep rows
  131-139, probes/fin-rri-vdb-edges A1-A16): fv 0 is -1, pv and fv 0 is 0, pv 0 with fv not 0 is `#NUM!`, nper at most 0
  is `#NUM!`, a negative fv/pv is `#NUM!` (`RRI(1,-100,200)`), a negative pv and fv are a value (`RRI(10,-100,-200)`),
  and a result past the double range is `#NUM!` (`RRI(1E-5,1,2)`).
- "T-1625 probes: VDB, AMORDEGRC, FVSCHEDULE, MDURATION, RRI": RRI is ulps from the worksheet form.
- "EFFECT, NOMINAL and RRI near a rate of 0 are far from the math (T-4110)": the table's own evidence for the domains
  below and for Excel's cancellation near a rate of 0 (0 where `1 + r/n` rounds to 1; Excel kept, decision 0300).
Domains (Excel's `#NUM!`, `DomainError`), each evidenced:
- EFFECT and NOMINAL: a rate of 0 (gen-m9 5422bdbf173fc233 A70 `EFFECT(0,4)`, b73c6cc96e093f03 A248 `NOMINAL(0,3)`),
  an npery that floors to 0 (gen-m9 c0681a8b150ce301 A291 `EFFECT(0.05,0.5)`, 79d87fc527927626 A331
  `NOMINAL(1E-9,0.5)`), and EFFECT's power past the double range (above);
- RRI: as listed above.
Not drawn and given no math value (`NoValueError`), as no evidence names Excel's result: a negative rate and a negative
npery for EFFECT and NOMINAL; RRI with pv and fv both 0 (Excel's 0, which no math names); a power within a hair of the
double range's top (Excel's double and the exact one may fall on either side).
"""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    Shape,
    any_edge,
    by_stratum_with_edge,
    scalars,
    signed,
    signed_log_uniform,
)
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

EFFECT_SOURCE = "financial.md#effect-and-nominal-raise-to-their-power-with-"
RRI_SOURCE = "financial.md#duration-and-rri-operation-order"
RRI_PROBE_SOURCE = "financial.md#t-1625-probes-vdb-amordegrc-fvschedule-mduration-rri"
NEAR_ZERO_SOURCE = "financial.md#effect-nominal-and-rri-near-a-rate-of-0-are-far-from-the-math-t-4110"
LN_DOUBLE_TOP = 1024 * mpmath.log(2)  # ln 2^1024: a power at or past it is past the double range
BAND = 1e-9  # a power's logarithm this close to the top: Excel's double may be on either side (no value)


def _past_double_range(log_power: mpmath.mpf) -> bool:
    """Whether a power with natural logarithm `log_power` is past the double range; NoValueError within `BAND` of the
    top."""
    if abs(log_power - LN_DOUBLE_TOP) < BAND:
        msg = "a power at the top of the double range"
        raise NoValueError(msg)
    return bool(log_power > LN_DOUBLE_TOP)


def _rate_and_periods(args: Sequence[MathArg]) -> tuple[mpmath.mpf, mpmath.mpf]:
    """EFFECT's and NOMINAL's (rate, n): DomainError for a rate of 0 or an npery that floors to 0, NoValueError for a
    negative one (unprobed). An eval cell's npery reaches here truncated (`Param.truncates`)."""
    r, n = scalars(args)
    if r == 0 or n == 0:
        msg = "a rate of 0 or an npery below 1: #NUM!"
        raise DomainError(msg)
    if r < 0 or n < 0:
        msg = "a negative rate or npery (unprobed)"
        raise NoValueError(msg)
    return r, n


def _effect(args: Sequence[MathArg]) -> mpmath.mpf:
    r, n = _rate_and_periods(args)
    log_power = n * mpmath.log1p(r / n)
    if _past_double_range(log_power):
        msg = "(1 + rate/n)^n past the double range: #NUM!"
        raise DomainError(msg)
    return mpmath.expm1(log_power)


def _nominal(args: Sequence[MathArg]) -> mpmath.mpf:
    e, n = _rate_and_periods(args)
    return n * mpmath.expm1(mpmath.log1p(e) / n)


def _rri(args: Sequence[MathArg]) -> mpmath.mpf:
    nper, pv, fv = scalars(args)
    if nper <= 0:
        msg = "nper at most 0: #NUM!"
        raise DomainError(msg)
    if pv == 0:
        if fv == 0:
            msg = "pv and fv 0 (Excel's 0, no math value)"
            raise NoValueError(msg)
        msg = "pv 0: #NUM!"
        raise DomainError(msg)
    ratio = fv / pv
    if ratio < 0:
        msg = "a negative fv/pv: #NUM!"
        raise DomainError(msg)
    if ratio == 0:
        return mpmath.mpf(-1)
    log_growth = mpmath.log(ratio) / nper
    if _past_double_range(log_growth):
        msg = "(fv/pv)^(1/nper) past the double range: #NUM!"
        raise DomainError(msg)
    return mpmath.expm1(log_growth)


# -- draws -----------------------------------------------------------------------------------------


def _periods_per_year(stratum: str, rng: random.Random) -> int:
    if stratum == "extreme":
        return round(log_uniform(rng, 365, 1e12))
    if stratum == "log-uniform":
        return round(log_uniform(rng, 1, 1e9))
    return rng.choice((1, 2, 4, 12, 52, 365, rng.randint(1, 400)))


def _conversion_rate(stratum: str, rng: random.Random) -> float:
    if stratum == "tail":
        return log_uniform(rng, 1e-12, 1e-4) if rng.random() < 0.5 else log_uniform(rng, 1, 100)
    if stratum == "extreme":
        return log_uniform(rng, 1, 1e4) if rng.random() < 0.5 else log_uniform(rng, 1e-15, 1e-6)
    if stratum == "log-uniform":
        return log_uniform(rng, 1e-15, 1e3)
    return log_uniform(rng, 1e-3, 0.5)


def _conversion(stratum: str, rng: random.Random) -> list[Arg]:
    """[rate, npery]: the rate on its own, as EFFECT's and NOMINAL's argument order has it."""
    return [_conversion_rate(stratum, rng), _periods_per_year(stratum, rng)]


def _effect_overflow(rng: random.Random) -> list[Arg]:
    """A power past the double range (`#NUM!`, `EFFECT(1E13,365)`) or just inside it."""
    n = rng.choice((1, 2, 3, 11, 100, 365))
    return [n * (10 ** (rng.uniform(250, 340) / n) - 1) if n > 1 else rng.choice((1e300, 1e307)), n]


CONVERSION_EDGES: tuple[Shape, ...] = (
    lambda rng: [0.0, rng.randint(1, 400)],  # a rate of 0: #NUM!
    lambda rng: [log_uniform(rng, 1e-3, 0.5), 0],  # npery 0: #NUM!
    lambda rng: [log_uniform(rng, 1e-3, 0.5), 1],  # npery 1: the rate itself
    lambda rng: [rng.choice((9.0, 18.0, 270.0, 990.0)), rng.choice((1, 2, 30, 99))],  # 1 + r/n a power of ten
)


def _rri_draw(stratum: str, rng: random.Random) -> list[Arg]:
    """[nper, pv, fv]: a growth rate g over nper periods from pv (either sign) to fv = pv·(1 + g)^nper."""
    nper: float = rng.randint(1, 100) if rng.random() < 0.7 else log_uniform(rng, 0.1, 100)
    pv = signed(rng, log_uniform(rng, 1, 1e6))
    g = signed_log_uniform(rng, 1e-4, 0.5)
    if stratum == "tail":
        g = signed(rng, log_uniform(rng, 1e-12, 1e-5))
    elif stratum == "extreme":
        nper = log_uniform(rng, 1e3, 1e12) if rng.random() < 0.5 else log_uniform(rng, 1e-3, 1e-1)
        pv = signed(rng, log_uniform(rng, 1e-100, 1e100))
        return [nper, pv, pv * 10 ** signed(rng, log_uniform(rng, 1e-3, 200))]
    elif stratum == "log-uniform":
        nper = log_uniform(rng, 1e-3, 1e9)
        pv = signed(rng, log_uniform(rng, 1e-300, 1e300))
        return [
            nper,
            pv,
            pv * 10 ** rng.uniform(-300, 300) if abs(pv) < 1 else pv * 10 ** rng.uniform(-300, 0),
        ]
    return [nper, pv, pv * (1 + g) ** nper]


RRI_EDGES: tuple[Shape, ...] = (
    lambda rng: [rng.randint(1, 50), signed(rng, log_uniform(rng, 1, 1e6)), 0.0],  # fv 0: -1
    lambda rng: [
        rng.choice((0, -1, -0.5)),
        log_uniform(rng, 1, 1e4),
        log_uniform(rng, 1, 1e4),
    ],  # nper <= 0: #NUM!
    lambda rng: [rng.randint(1, 50), 0.0, log_uniform(rng, 1, 1e4)],  # pv 0: #NUM!
    lambda rng: [rng.randint(1, 50), log_uniform(rng, 1, 1e4), -log_uniform(rng, 1, 1e4)],  # fv/pv < 0: #NUM!
    lambda rng: [1, signed(rng, log_uniform(rng, 1, 1e6)), log_uniform(rng, 1, 1e6)],  # nper 1: fv/pv - 1
    lambda rng: [log_uniform(rng, 1e-6, 1e-3), 1.0, rng.uniform(1.5, 3)],  # past the double range: #NUM!
    lambda rng: [rng.choice((0.5, 0.25, 1 / 3)), rng.randint(1, 9), rng.randint(1, 99)],  # a fractional nper
)


EFFECT = register(
    Definition(
        function="EFFECT",
        params=(
            Param("nominal_rate", "real", source=EFFECT_SOURCE),
            Param("npery", "int", source=EFFECT_SOURCE, truncates=EFFECT_SOURCE),
        ),
        value=_effect,
        draw=by_stratum_with_edge(
            _conversion,
            any_edge((*CONVERSION_EDGES, _effect_overflow)),
            "EFFECT: [nominal_rate, npery] by stratum.",
        ),
        sources=(EFFECT_SOURCE, NEAR_ZERO_SOURCE),
    )
)

NOMINAL = register(
    Definition(
        function="NOMINAL",
        params=(
            Param("effect_rate", "real", source=EFFECT_SOURCE),
            Param("npery", "int", source=EFFECT_SOURCE, truncates=EFFECT_SOURCE),
        ),
        value=_nominal,
        draw=by_stratum_with_edge(
            _conversion, any_edge(CONVERSION_EDGES), "NOMINAL: [effect_rate, npery] by stratum."
        ),
        sources=(EFFECT_SOURCE, NEAR_ZERO_SOURCE),
    )
)

RRI = register(
    Definition(
        function="RRI",
        params=(
            Param("nper", "real", source=RRI_SOURCE),
            Param("pv", "real", source=RRI_SOURCE),
            Param("fv", "real", source=RRI_SOURCE),
        ),
        value=_rri,
        draw=by_stratum_with_edge(_rri_draw, any_edge(RRI_EDGES), "RRI: [nper, pv, fv] by stratum."),
        sources=(RRI_SOURCE, RRI_PROBE_SOURCE, NEAR_ZERO_SOURCE),
    )
)
