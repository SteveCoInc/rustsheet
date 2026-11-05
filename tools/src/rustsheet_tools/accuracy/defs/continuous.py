"""LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER and FISHERINV (Gate A slice 4o, T-3785).

The math, at the double of each written literal and the working precision:
- LOGNORM.DIST is the normal of ln x: Phi(z) and phi(z)/(x*sd) with z = (ln x - mean)/sd, through normal.py's `cdf`
  and `density`; LOGNORM.INV is exp(mean + sd*z) with normal.py's `standard_quantile`.
- EXPON.DIST: the cdf -expm1(-lambda*x) and the density lambda*exp(-lambda*x).
- WEIBULL.DIST: with t = (x/beta)^alpha, the cdf -expm1(-t) and the density alpha/beta*(x/beta)^(alpha-1)*exp(-t).
- FISHER is atanh(x), FISHERINV tanh(y).

Sources:
- statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)":
  `LOGNORM.DIST(x,m,s,TRUE)` = `NORM.DIST(LN(x),m,s,TRUE)` on 60 of 60 cells; "Normal CDF and ERFC sweep (T-1040)":
  its deep lower tail is up to about 2E-13 relative off, far under decision 0188 R3's 1e-4.
- statistics.md, "LOGNORM.INV is EXP of NORM.INV's quantile (T-0925)": p in (0, 1), and Microsoft's pages for both:
  x <= 0 and standard_dev <= 0 are `#NUM!`.
- statistics.md, "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP (T-2437)" and
  "FISHERINV is (EXP(2y)-1)/(EXP(2y)+1), with its cancellation near 0": EXPON.DIST's cdf is Kahan's expm1 below
  t = 1 and 1 - EXP(-t) above, both within an ulp or so of the math; x < 0 and lambda <= 0 are `#NUM!` (Microsoft);
  FISHER is `#NUM!` outside (-1, 1) (`FISHER(1)`, Microsoft); FISHERINV(y) is 1 once EXP(2y) overflows.
- statistics.md, "WEIBULL.DIST sweep (T-1046)", "WEIBULL.DIST's cdf switches from expm1 to 1-EXP(-t) near t = 1
  (T-2239)", "WEIBULL's density divides by l^k: #DIV/0! when it underflows, #NUM! when it overflows (T-2239)" and
  "WEIBULL.DIST's density at x = 0 still computes beta^alpha (T-3312)": the error domain. Both forms are `#NUM!` once
  t = EXP(alpha*LN(x/beta)) overflows; the density is `#DIV/0!` where beta^alpha = EXP(alpha*LN(beta)) underflows to 0
  and `#NUM!` where it overflows, x = 0 included, though its limit is 0.
- At x = 0 the math is the math, not Excel's 0: the cdf is 0, and the density is 0 for alpha > 1, 1/beta for alpha = 1
  and unbounded for alpha < 1 (no value; NoValueError). Excel's density there is 0 for every shape (statistics.md
  "Density at 0", probes-numeric/t1048-weibull A16:B17), so the alpha = 1 cells are 100% off the math: an R3 candidate
  listed for the owner (T-4318), not a rule of the definition.

What is left out of the tables (the sampler redraws; their Excel cells stay in the eval suites):
- FISHER and FISHERINV on 0 < |x| < `FISHER_CANCEL_HI`. Excel computes 0.5*LN((1+x)/(1-x)) and
  (EXP(2y)-1)/(EXP(2y)+1) in doubles, which cancels near 0: an absolute error near 1E-16, so about 1E-16/|x|
  relative. `FISHER(1.49172367472348E-11)` = 1.4917178603246007E-11 is 3.9E-6 off atanh, and
  `FISHER(1.02759470998055E-16)` is 0 (probes/t2437-fit-separators A1, A4): past R3's 1e-4 below about
  |x| = 1E-12. The cut at 1E-6 keeps Excel's error near 1E-10, inside the gate's 1e-9, so the gate needs no R3 ruling
  (as GAUSS's region in normal.py); 0 itself stays in. At 1E-8 the doubles' error is still 5E-9 (`FISHER(1E-08)` in
  doubles is 9.999999950247593E-9).
- Ill-conditioned points, where one rounding of an input-sized intermediate that Excel (and any double evaluation)
  forms moves the value by more than `CONDITION_LIMIT` relative: LOGNORM.DIST's ln x - mean, LOGNORM.INV's
  mean + sd*z and WEIBULL.DIST's alpha*LN(x/beta) and its density's powers. The bound is kappa * 2^-52, kappa the
  relative condition number of the value in that intermediate.
- Overflow and underflow cuts no Excel cell pins: points within `EXP_BAND` of an EXP overflow; LOGNORM.DIST densities
  whose x*sd leaves the normal range or whose exp(-z^2/2) underflows before the division by x*sd lifts it, and cdfs
  whose z overflows; EXPON.DIST with lambda*x past 1E+300; WEIBULL.DIST
  with x/beta below the smallest normal double, a beta^alpha between EXP's underflow and 1E-300, and a density
  intermediate (alpha/beta^alpha, x^(alpha-1)) outside the normal range.
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import scalars, signed
from rustsheet_tools.accuracy.defs.exp_log import EXP_BAND
from rustsheet_tools.accuracy.defs.normal import (
    OVERFLOW_SOURCE,
    SWEEP_SOURCE,
    cdf,
    density,
    inverse_probability,
)
from rustsheet_tools.accuracy.defs.normal import standard_quantile as normal_quantile
from rustsheet_tools.accuracy.literal import MAX_LITERAL, MIN_LITERAL, Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)
from rustsheet_tools.accuracy.sampler import log_uniform

KERNEL_SOURCE = (
    "statistics.md#one-incomplete-gamma-kernel-under-erf-erfc-norm-chisq-gammadist-and-poisson-t-1778"
)
LOGNORM_INV_SOURCE = "statistics.md#lognorminv-is-exp-of-norminvs-quantile-t-0925"
OPERATORS_SOURCE = (
    "statistics.md#fisher-fisherinv-expondist-and-harmean-are-excel-operators-around-ln-and-exp-t-2437"
)
FISHERINV_SOURCE = "statistics.md#fisherinv-is-exp2y-1exp2y1-with-its-cancellation-near-0"
WEIBULL_SWEEP_SOURCE = "statistics.md#weibulldist-sweep-t-1046"
WEIBULL_SWITCH_SOURCE = "statistics.md#weibulldists-cdf-switches-from-expm1-to-1-exp-t-near-t--1-t-2239"
WEIBULL_DIVISOR_SOURCE = (
    "statistics.md#weibulls-density-divides-by-lk-div0-when-it-underflows-num-when-it-overflows-t-2239"
)
WEIBULL_ZERO_SOURCE = "statistics.md#weibulldists-density-at-x--0-still-computes-betaalpha-t-3312"
MICROSOFT_SOURCE = (
    "Microsoft LOGNORM.DIST / LOGNORM.INV / EXPON.DIST / WEIBULL.DIST / FISHER pages: argument limits"
)

FISHER_CANCEL_HI = 1e-6  # FISHER and FISHERINV points with 0 < |x| below this are redrawn (the cancellation)
CONDITION_LIMIT = 1e-10  # an ill-conditioned point (kappa * 2^-52 above this) is redrawn
EPS = 2.0**-52
EXP_MAX = EXP_BAND[0]  # an EXP argument from here up overflows or is in the unpinned band
EXP_ZERO = -745.2  # EXP of an argument below this is 0 (gradual underflow ends near -745.13)
NORMAL_MIN = sys.float_info.min
NORMAL_RANGE = (1e-300, 1e300)  # an intermediate inside this is a normal double with room to spare
EXPON_PRODUCT_MAX = 1e300  # lambda*x past this is unprobed
REDRAWS = 50


def _ill_conditioned(kappa: float) -> None:
    """NoValueError (redraw) when a relative condition number `kappa` puts one rounding past `CONDITION_LIMIT`."""
    if kappa * EPS > CONDITION_LIMIT:
        msg = f"ill-conditioned (kappa {kappa:.3g})"
        raise NoValueError(msg)


def _exp_or_inf(y: float) -> float:
    """exp(y) as a double, inf past the overflow, NoValueError inside the unpinned overflow band."""
    if EXP_BAND[0] <= y <= EXP_BAND[1]:
        msg = "an EXP argument in the unpinned overflow band"
        raise NoValueError(msg)
    return math.inf if y > EXP_BAND[1] else math.exp(y)


# -- LOGNORM.DIST and LOGNORM.INV ---------------------------------------------------------------------------------


def _lognorm_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, mean, sd, cumulative = args
    assert not isinstance(x, (bool, tuple)) and not isinstance(mean, (bool, tuple))
    assert not isinstance(sd, (bool, tuple))
    fx, fmean, fsd = float(x), float(mean), float(sd)
    ln_x = math.log(fx)
    z = (ln_x - fmean) / fsd
    if math.isinf(z) or (not cumulative and math.isinf(z * z)):
        # T-2435 pins the overflow of NORM.DIST's square, not LOGNORM.DIST's
        msg = "z or z*z overflows a double"
        raise NoValueError(msg)
    # ln x and mean each carry a rounding of their size; the lower tail and the density move by |z| per unit of z
    _ill_conditioned((abs(ln_x) + abs(fmean)) / fsd * max(1.0, abs(z)))
    w = mpmath.log(x)
    zz = (w - mean) / sd
    if cumulative:
        return cdf(zz)
    if not NORMAL_RANGE[0] <= fx * fsd <= NORMAL_RANGE[1]:
        msg = "the density's x*sd leaves the normal range (unprobed)"
        raise NoValueError(msg)
    if z * z / 2 > -math.log(NORMAL_RANGE[0]) and fx * fsd < 1:
        # exp(-z^2/2) underflows before the division by x*sd lifts it back (no cell pins what Excel does)
        msg = "the density's exp(-z^2/2) underflows before its division (unprobed)"
        raise NoValueError(msg)
    return density(zz) / (x * sd)


def _lognorm_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, mean, sd = scalars(args)
    z = normal_quantile(p)
    exponent = mean + sd * z
    if exponent >= EXP_MAX:
        msg = "LOGNORM.INV overflows or is next to the overflow (unprobed)"
        raise NoValueError(msg)
    # the sum's rounding, and NORM.S.INV's central 31 ulp (T-1746) scaled by sd*z
    _ill_conditioned(abs(float(mean)) + 32 * abs(float(sd * z)))
    return mpmath.exp(exponent)


def _lognorm_mean_sd(stratum: str, rng: random.Random) -> tuple[float, float]:
    if stratum == "extreme":
        return signed(rng, rng.uniform(10, 300)), log_uniform(rng, 1e-3, 15)
    if stratum == "log-uniform":
        return signed(rng, log_uniform(rng, 1e-300, 300)), log_uniform(rng, 1e-6, 1e3)
    return rng.uniform(-3, 3), log_uniform(rng, 0.1, 3)


def _quantile_x(q: float, upper: bool, mean: float, sd: float) -> float:
    """The x whose lower (or upper) lognormal tail is q, or 0 when it is outside the literal range."""
    with mpmath.workdps(30):
        z = normal_quantile(mpmath.mpf(q))
        exponent = mean + sd * (-z if upper else z)
        if not math.log(MIN_LITERAL) < exponent < math.log(MAX_LITERAL):
            return 0.0
        return float(mpmath.exp(exponent))


def _lognorm_edge_x(rng: random.Random, mean: float, sd: float) -> float:
    """LOGNORM.DIST's x at an edge: the domain bounds, the lower tail's flush (T-1040), the median, or near 1."""
    shape = rng.randrange(4)
    if shape == 0:
        return rng.choice((0.0, -1.0, 1.0, MIN_LITERAL, MAX_LITERAL))
    if shape == 1:
        return max(math.exp(mean - sd * rng.choice((37.5, 37.625, 38.0))), MIN_LITERAL)
    if shape == 2:
        return math.exp(mean)
    return log_uniform(rng, 0.5, 2)


def _lognorm_x(stratum: str, rng: random.Random, mean: float, sd: float) -> float:
    if stratum == "tail":
        x = _quantile_x(log_uniform(rng, 1e-300, 0.1), rng.random() < 0.5, mean, sd)
        return x if x > 0 else math.exp(mean)
    if stratum == "edge":
        return _lognorm_edge_x(rng, mean, sd)
    if stratum == "extreme":
        return log_uniform(rng, 1e-300, 1e300)
    if stratum == "log-uniform":
        return log_uniform(rng, MIN_LITERAL, MAX_LITERAL)
    return log_uniform(rng, 0.01, 100)


def _draw_lognorm_dist(stratum: str, rng: random.Random) -> list[Arg]:
    mean, sd = _lognorm_mean_sd(stratum, rng)
    x = _lognorm_x(stratum, rng, mean, sd)
    if stratum == "edge" and rng.random() < 0.2:
        sd = rng.choice((0.0, -1.0))  # #NUM!
    return [x, mean, sd, rng.random() < 0.5]


def _draw_lognorm_inv(stratum: str, rng: random.Random) -> list[Arg]:
    mean, sd = _lognorm_mean_sd(stratum, rng)
    p = float(f"{inverse_probability(stratum, rng):.15g}")
    if stratum == "edge" and rng.random() < 0.3:
        return list(rng.choice(((0.0, mean, sd), (1.0, mean, sd), (0.5, mean, 0.0), (0.3, mean, -1.0))))
    return [p if 0 < p < 1 else 0.5, mean, sd]


# -- EXPON.DIST ---------------------------------------------------------------------------------------------------


def _expon_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, lam, cumulative = args
    assert not isinstance(x, (bool, tuple)) and not isinstance(lam, (bool, tuple))
    if float(x) * float(lam) > EXPON_PRODUCT_MAX:
        msg = "lambda*x past 1E+300 (unprobed)"
        raise NoValueError(msg)
    t = lam * x
    return -mpmath.expm1(-t) if cumulative else lam * mpmath.exp(-t)


def _draw_expon(stratum: str, rng: random.Random) -> list[Arg]:
    cumulative = rng.random() < 0.5
    if stratum == "edge":
        lam = log_uniform(rng, 0.1, 10)
        shapes: tuple[tuple[float, float], ...] = (
            (0.0, lam),
            (-1e-300, lam),  # #NUM!
            (-1.0, lam),
            (1.0, 0.0),  # #NUM!
            (1.0, -1.0),
            (1.0, 1.0),  # t = 1, the expm1 / 1 - EXP cut
            (0.999999999999999, 1.0),
            (1.35079141386477, 1.0),
            (745.0, 1.0),
            (1e-300, 1.0),
        )
        x, lam = rng.choice(shapes)
        return [x, lam, cumulative]
    lam = log_uniform(rng, 0.1, 10)
    if stratum == "tail":
        kind = rng.randrange(3)
        if kind == 0:  # the cdf's small t, where Kahan's expm1 keeps the digits
            t = log_uniform(rng, 1e-300, 1e-3)
        elif kind == 1:  # around the expm1 / 1 - EXP switch (T-2437)
            t = rng.uniform(0.99, 1.36)
        else:  # the far tail
            t = rng.uniform(20, 740)
        return [t / lam, lam, cumulative]
    if stratum == "extreme":
        lam = log_uniform(rng, 1e-100, 1e100)
        return [log_uniform(rng, 1e-30, 700) / lam, lam, cumulative]
    if stratum == "log-uniform":
        return [log_uniform(rng, 1e-300, 1e300), log_uniform(rng, 1e-300, 1e300), cumulative]
    return [rng.uniform(0, 5), lam, cumulative]


# -- WEIBULL.DIST -------------------------------------------------------------------------------------------------


def _weibull_domain(args: Sequence[MathArg]) -> str | None:
    """The evidenced errors (T-2239, T-3312): t = EXP(alpha*LN(x/beta)) overflows, or the density's
    beta^alpha = EXP(alpha*LN(beta)) underflows to 0 or overflows. Next to a cut no cell pins, NoValueError."""
    x, alpha, beta = (float(a) for a in scalars(args[:3]))
    cumulative = args[3]
    if x < 0 or alpha <= 0 or beta <= 0:
        return None  # the parameters' intervals say it
    if x > 0:
        ratio = x / beta
        if ratio < NORMAL_MIN:
            msg = "x/beta below the smallest normal double (unprobed)"
            raise NoValueError(msg)
        if math.isinf(_exp_or_inf(alpha * math.log(ratio))):
            return "t = (x/beta)^alpha overflows"
    if cumulative:
        return None
    y = alpha * math.log(beta)
    if y < EXP_ZERO:
        return "beta^alpha underflows to 0"
    divisor = _exp_or_inf(y)
    if math.isinf(divisor):
        return "beta^alpha overflows"
    if divisor < NORMAL_RANGE[0]:
        msg = "beta^alpha between EXP's underflow and 1E-300 (unprobed)"
        raise NoValueError(msg)
    return None


def _weibull_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, alpha, beta, cumulative = args
    assert not isinstance(x, (bool, tuple)) and not isinstance(alpha, (bool, tuple))
    assert not isinstance(beta, (bool, tuple))
    if x == 0:
        if cumulative or alpha > 1:
            return mpmath.mpf(0)
        if alpha == 1:
            return 1 / beta  # Excel's 0 is not the math (R3 candidate, T-4318)
        msg = "the density at x = 0 is unbounded for alpha < 1"
        raise NoValueError(msg)
    fx, falpha, fbeta = float(x), float(alpha), float(beta)
    log_ratio = math.log(fx / fbeta)
    t_float = math.exp(falpha * log_ratio)
    # t's relative error is alpha times LN's absolute one; exp(-t) multiplies it by t
    kappa = falpha * (1 + abs(log_ratio)) * (1 + (0 if cumulative else t_float))
    if not cumulative:
        power_log = (falpha - 1) * math.log(fx)
        scale = falpha / math.exp(falpha * math.log(fbeta))
        if not (NORMAL_RANGE[0] <= scale <= NORMAL_RANGE[1] and abs(power_log) < math.log(NORMAL_RANGE[1])):
            msg = "a density intermediate leaves the normal range (unprobed)"
            raise NoValueError(msg)
        kappa += falpha * abs(math.log(fbeta)) + abs(power_log)
    _ill_conditioned(kappa)
    t = (x / beta) ** alpha
    if cumulative:
        return -mpmath.expm1(-t)
    return alpha / beta * (x / beta) ** (alpha - 1) * mpmath.exp(-t)


# Evidenced error points and their neighbours (statistics.md T-2239, T-3312).
_WEIBULL_EDGES: tuple[tuple[float, float, float, bool], ...] = (
    (0.1, 10000000000.5, 0.2, False),  # #DIV/0!
    (0.3, 99999.0, 99999.0, False),  # #NUM!
    (100.0, 1e10, 0.01, True),  # #NUM! (t overflows)
    (1e15, 42.0, 10000000001.0, True),  # a number
    (0.0, 309.0, 10.0, False),  # #NUM!
    (0.0, 400.0, 0.1, False),  # #DIV/0!
    (0.0, 308.0, 10.0, False),  # 0
    (0.0, 0.5, 1.0, False),  # Excel 0; the density is unbounded, so no math value (redrawn)
    (0.0, 1.0, 2.0, True),
    (1e-10, 0.5, 1.0, False),  # 49999.5000025 (T-1046)
    (1e-10, 1.0, 1.0, True),
    (1000.0, 0.5, 1.0, True),  # 0.9999999999999816
    (-1e-300, 1.0, 1.0, True),  # #NUM!
    (1.0, 0.0, 1.0, True),
    (1.0, 1.0, 0.0, False),
    (1.0, 1.0, 1.0, True),  # t = 1, the expm1 / 1 - EXP cut
    (0.9917, 1.0, 1.0, True),
    (1.3506, 1.0, 1.0, True),
)


def _weibull_shape_scale(stratum: str, rng: random.Random) -> tuple[float, float]:
    if stratum == "extreme":
        return log_uniform(rng, 1e-3, 1e3), log_uniform(rng, 1e-100, 1e100)
    if stratum == "log-uniform":
        return log_uniform(rng, 1e-6, 1e6), log_uniform(rng, 1e-300, 1e300)
    return log_uniform(rng, 0.5, 5), log_uniform(rng, 0.5, 10)


def _weibull_x(t: float, alpha: float, beta: float) -> float:
    """The x with (x/beta)^alpha = t, in logs and capped to the literal range (a capped x is still a valid point)."""
    log_x = math.log(beta) + math.log(t) / alpha
    return math.exp(min(max(log_x, math.log(MIN_LITERAL)), math.log(MAX_LITERAL)))


def _draw_weibull(stratum: str, rng: random.Random) -> list[Arg]:
    cumulative = rng.random() < 0.5
    if stratum == "edge":
        return list(rng.choice(_WEIBULL_EDGES))
    alpha, beta = _weibull_shape_scale(stratum, rng)
    if stratum == "tail":
        kind = rng.randrange(3)
        if kind == 0:  # the lower tail: small t
            t = log_uniform(rng, 1e-300, 1e-3)
        elif kind == 1:  # around the expm1 / 1 - EXP switch (T-2239)
            t = rng.uniform(0.99, 1.36)
        else:  # the upper tail
            t = rng.uniform(20, 700)
        return [_weibull_x(t, alpha, beta), alpha, beta, cumulative]
    if stratum == "extreme":
        return [_weibull_x(log_uniform(rng, 1e-30, 700), alpha, beta), alpha, beta, cumulative]
    if stratum == "log-uniform":
        return [log_uniform(rng, 1e-300, 1e300), alpha, beta, cumulative]
    return [rng.uniform(0, 3) * beta, alpha, beta, cumulative]


# -- FISHER and FISHERINV -----------------------------------------------------------------------------------------


def fisher_cancels(x: float) -> bool:
    """Whether x is in FISHER's and FISHERINV's excluded cancellation region."""
    return 0 < abs(x) < FISHER_CANCEL_HI


def _fisher_x(stratum: str, rng: random.Random) -> float:
    if stratum == "tail":  # near ±1, and just past the cancellation region
        return signed(
            rng,
            1 - 10 ** -rng.uniform(1, 14) if rng.random() < 0.6 else log_uniform(rng, FISHER_CANCEL_HI, 1e-3),
        )
    if stratum == "edge":
        return rng.choice(
            (0.0, 1.0, -1.0, 0.999999999999999, -0.999999999999999, 1.00000000000001, 2.0, FISHER_CANCEL_HI)
        )
    if stratum in ("extreme", "log-uniform"):
        return signed(rng, log_uniform(rng, FISHER_CANCEL_HI, 0.999999999999999))
    return rng.uniform(-0.99, 0.99)


def _fisherinv_y(stratum: str, rng: random.Random) -> float:
    if stratum == "tail":  # toward the saturation at ±1, and just past the cancellation region
        return signed(
            rng, rng.uniform(10, 400) if rng.random() < 0.6 else log_uniform(rng, FISHER_CANCEL_HI, 1e-3)
        )
    if stratum == "edge":
        return rng.choice(
            (0.0, FISHER_CANCEL_HI, -FISHER_CANCEL_HI, 18.7, 19.1, 354.9, 355.0, 1000.0, -1000.0, MAX_LITERAL)
        )
    if stratum == "extreme":
        return signed(rng, log_uniform(rng, 10, MAX_LITERAL))
    if stratum == "log-uniform":
        return signed(rng, log_uniform(rng, FISHER_CANCEL_HI, MAX_LITERAL))
    return rng.uniform(-4, 4)


def _cancel_free(
    draw_one: Callable[[str, random.Random], float],
) -> Callable[[str, random.Random], list[Arg]]:
    """A one-argument draw that redraws FISHER's and FISHERINV's cancellation region."""

    def draw(stratum: str, rng: random.Random) -> list[Arg]:
        for _ in range(REDRAWS):
            x = float(f"{draw_one(stratum, rng):.15g}")
            if not fisher_cancels(x):
                return [x]
        return [0.5]

    return draw


def _fisher(args: Sequence[MathArg]) -> mpmath.mpf:
    (x,) = scalars(args)
    return mpmath.atanh(x)


def _fisherinv(args: Sequence[MathArg]) -> mpmath.mpf:
    (y,) = scalars(args)
    return mpmath.tanh(y)


_SD = Param("standard_dev", "real", Interval(0, math.inf), typical=(0.1, 3), source=MICROSOFT_SOURCE)
_CUMULATIVE = Param("cumulative", "bool", source="catalog")

LOGNORM_DIST = register(
    Definition(
        function="LOGNORM.DIST",
        params=(
            Param("x", "real", Interval(0, math.inf), typical=(0.01, 100), source=MICROSOFT_SOURCE),
            Param("mean", "real", typical=(-3, 3), source=KERNEL_SOURCE),
            _SD,
            _CUMULATIVE,
        ),
        value=_lognorm_dist,
        draw=_draw_lognorm_dist,
        sources=(KERNEL_SOURCE, SWEEP_SOURCE, OVERFLOW_SOURCE, MICROSOFT_SOURCE),
    )
)

LOGNORM_INV = register(
    Definition(
        function="LOGNORM.INV",
        params=(
            Param("probability", "real", Interval(0, 1), typical=(0.001, 0.999), source=LOGNORM_INV_SOURCE),
            Param("mean", "real", typical=(-3, 3), source=LOGNORM_INV_SOURCE),
            _SD,
        ),
        value=_lognorm_inv,
        draw=_draw_lognorm_inv,
        sources=(LOGNORM_INV_SOURCE, MICROSOFT_SOURCE),
    )
)

EXPON_DIST = register(
    Definition(
        function="EXPON.DIST",
        params=(
            Param("x", "real", Interval(0, math.inf, lo_open=False), typical=(0, 5), source=MICROSOFT_SOURCE),
            Param("lambda", "real", Interval(0, math.inf), typical=(0.1, 10), source=MICROSOFT_SOURCE),
            _CUMULATIVE,
        ),
        value=_expon_dist,
        draw=_draw_expon,
        sources=(OPERATORS_SOURCE, MICROSOFT_SOURCE),
    )
)

WEIBULL_DIST = register(
    Definition(
        function="WEIBULL.DIST",
        params=(
            Param(
                "x",
                "real",
                Interval(0, math.inf, lo_open=False),
                typical=(0, 30),
                source=WEIBULL_SWEEP_SOURCE,
            ),
            Param("alpha", "real", Interval(0, math.inf), typical=(0.5, 5), source=WEIBULL_SWEEP_SOURCE),
            Param("beta", "real", Interval(0, math.inf), typical=(0.5, 10), source=WEIBULL_SWEEP_SOURCE),
            _CUMULATIVE,
        ),
        value=_weibull_dist,
        domain=_weibull_domain,
        draw=_draw_weibull,
        sources=(WEIBULL_SWEEP_SOURCE, WEIBULL_SWITCH_SOURCE, WEIBULL_DIVISOR_SOURCE, WEIBULL_ZERO_SOURCE),
    )
)

FISHER = register(
    Definition(
        function="FISHER",
        params=(Param("x", "real", Interval(-1, 1), typical=(-0.99, 0.99), source=OPERATORS_SOURCE),),
        value=_fisher,
        draw=_cancel_free(_fisher_x),
        sources=(OPERATORS_SOURCE, MICROSOFT_SOURCE),
    )
)

FISHERINV = register(
    Definition(
        function="FISHERINV",
        params=(Param("y", "real", typical=(-4, 4), source=FISHERINV_SOURCE),),
        value=_fisherinv,
        draw=_cancel_free(_fisherinv_y),
        sources=(FISHERINV_SOURCE, OPERATORS_SOURCE),
    )
)
