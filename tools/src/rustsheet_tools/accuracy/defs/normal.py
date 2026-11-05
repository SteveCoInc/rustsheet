"""NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI and GAUSS (Gate A slice 4m, T-3783).

The math: Phi(z) = erfc(-z/sqrt 2)/2 and phi(z) = exp(-z^2/2)/sqrt(2 pi) at the working precision (mpmath `ncdf`
and `npdf` are the same functions; erfc keeps the lower tail's digits down to the double range), with
z = (x - mean)/sd for NORM.DIST; the inverses solve the smaller tail in log scales. GAUSS(z) is Phi(z) - 1/2.

Sources:
- statistics.md, "The normal CDF is not always correctly rounded" and "Normal CDF and ERFC sweep (T-1040)": Excel's
  lower tail is up to about 1.15E-13 relative from Phi and 0 from z = -37.625 down, far under R3's 1e-4; the
  absolute floor 1e-300 (decision 0188 R2) covers the flush, so the tail stratum draws it.
- statistics.md, "Normal density oracle sweep (T-1041)": PHI is NORM.S.DIST(z, FALSE) bit for bit.
- statistics.md, "Normal densities are #NUM! once the square overflows (T-2435)": NORM.DIST, NORM.S.DIST and PHI
  are `#NUM!` for a density whose standardized value z = (x - mean)/sd squares past the double range (from |z| =
  2^512); the cdf stays 1 or 0 at any magnitude. That is the error domain of the densities.
- statistics.md, "NORM.S.INV is AS 241 PPND16 in the tails; the central region is unfitted (T-1746)" and numbers.md
  "NORM.S.INV / NORMSINV / NORM.INV(p,0,1): oracle sweep (T-1006)": p reaches about 5E-309 and the central
  region is up to 31 ulp from the correctly rounded inverse; p outside (0, 1) is `#NUM!`.
- statistics.md, "GAUSS of a tiny argument is x times phi(0) (T-2255)": GAUSS keeps its leading term up to
  |z| = 1E-15, and from about 4E-15 it is the cancelled Phi(z) - 0.5 in doubles.
- A standard deviation of 0 or below is `#NUM!` (Microsoft's NORM.DIST and NORM.INV pages; the engine's
  `special` checks).

The cancellation regions (decision 0188 R3). Two places where Excel subtracts rounded doubles are left out of the
tables, as ERF's two-argument region is (erf.py), so the gate does not depend on an owner ruling (T-3794's kind):
- GAUSS for 1E-15 < |z| < `GAUSS_CANCEL_HI`: Phi(z) - 0.5 in doubles carries an absolute error near 2^-54, about
  1.4E-16/|z| relative, up to 2.6% at `GAUSS(4E-15)` (gen-m9/146b46e74f2dac3b A46: Excel 1.5543122344752192E-15
  against 1.5957691216057308E-15). A point there is redrawn; at the cut the error is about 1.4E-8.
- NORM.INV where mean + sd*z nearly cancels: a point is redrawn while |mean + sd*z| < `INV_CANCEL` *
  max(|mean|, |sd*z|), which bounds the sum's rounding near 2.2E-7 relative.
Their Excel cells stay in the eval suites; they are just not Gate A reference points.
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import scalars, signed, square_overflows
from rustsheet_tools.accuracy.defs.erf import erfc
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_smaller_tail,
)
from rustsheet_tools.accuracy.sampler import log_uniform

CDF_SOURCE = "statistics.md#the-normal-cdf-is-not-always-correctly-rounded"
SWEEP_SOURCE = "statistics.md#normal-cdf-and-erfc-sweep-t-1040"
DENSITY_SOURCE = "statistics.md#normal-density-oracle-sweep-t-1041"
OVERFLOW_SOURCE = "statistics.md#normal-densities-are-num-once-the-square-overflows-t-2435"
INV_SOURCE = "statistics.md#norms-inv-is-as-241-ppnd16-in-the-tails-the-central-region-is-unfitted-t-1746"
GAUSS_SOURCE = "statistics.md#gauss-of-a-tiny-argument-is-x-times-phi0-t-2255"
SD_SOURCE = "Microsoft NORM.DIST / NORM.INV: standard_dev <= 0 is #NUM!"
FLUSH = 37.625  # the lower tail and the density are 0 from z = -37.625 down (T-1040, T-1041)
GAUSS_LEADING = 1e-15  # GAUSS keeps x*phi(0) up to here (T-2255)
GAUSS_CANCEL_HI = (
    1e-8  # GAUSS points in (GAUSS_LEADING, GAUSS_CANCEL_HI) are redrawn (the cancellation region)
)
INV_CANCEL = 1e-9
REDRAWS = 50
OVERFLOW = 2.0**512  # |z| from which z*z overflows (T-2435)
# e^-60 is closer to 0 than any 15-digit p below 1/2 puts the standard quantile; e^4 > 38.5 is past the
# quantile of the smallest double probability.
QUANTILE_LO, QUANTILE_HI = -60.0, 4.0


def cdf(z: mpmath.mpf) -> mpmath.mpf:
    """Phi(z), as erfc(-z/sqrt 2)/2 so the lower tail keeps its digits (erf.py's `erfc`, which takes the
    asymptotic series where mpmath's own overflows)."""
    return erfc(-z / mpmath.sqrt(2)) / 2


def density(z: mpmath.mpf) -> mpmath.mpf:
    """phi(z)."""
    return mpmath.exp(-z * z / 2) / mpmath.sqrt(2 * mpmath.pi)


def standard_quantile(p: mpmath.mpf) -> mpmath.mpf:
    """The z with Phi(z) = p, 0 < p < 1, by the smaller tail: z = -e^s solves Phi(-e^s) = q for q = min(p, 1-p),
    with 1 - p exact at the working precision. p = 1/2 is 0."""
    half = mpmath.mpf(1) / 2
    if p == half:
        return mpmath.mpf(0)
    q = p if p < half else 1 - p
    s = solve_smaller_tail(q, lambda x: cdf(-x), None, QUANTILE_LO, QUANTILE_HI)
    z = mpmath.exp(s)
    return -z if p < half else z


def _density_domain(z: float, cumulative: MathArg) -> str | None:
    """A density whose standardized z squares past the double range is `#NUM!` (statistics.md T-2435); the cdf is
    a number at any z."""
    return "the density's z*z overflows" if not cumulative and square_overflows(z) else None


def _standardized(args: Sequence[MathArg]) -> float:
    """NORM.DIST's z = (x - mean)/sd as Excel forms it in doubles (only its overflow is read; an inf or nan stays one)."""
    x, mean, sd = (float(a) for a in scalars(args[:3]))
    return (x - mean) / sd


def _norm_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, mean, sd, cumulative = args
    assert not isinstance(x, (bool, tuple)) and not isinstance(mean, (bool, tuple))
    assert not isinstance(sd, (bool, tuple))
    if cumulative and math.isinf(_standardized(args)):
        # (x - mean)/sd past the double range: no Excel cell says what the cdf is there (T-2435 probed the
        # density only), so such a point is redrawn.
        msg = "z overflows a double"
        raise NoValueError(msg)
    z = (x - mean) / sd
    return cdf(z) if cumulative else density(z) / sd


def _norm_dist_domain(args: Sequence[MathArg]) -> str | None:
    return _density_domain(_standardized(args), args[3])


def _norm_s_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    z, cumulative = args
    assert not isinstance(z, (bool, tuple))
    return cdf(z) if cumulative else density(z)


def _norm_s_dist_domain(args: Sequence[MathArg]) -> str | None:
    return _density_domain(float(scalars(args[:1])[0]), args[1])


def _phi(args: Sequence[MathArg]) -> mpmath.mpf:
    (z,) = scalars(args)
    return density(z)


def _phi_domain(args: Sequence[MathArg]) -> str | None:
    return _density_domain(float(scalars(args)[0]), False)


def _gauss(args: Sequence[MathArg]) -> mpmath.mpf:
    (z,) = scalars(args)
    if abs(z) < 1:
        # Phi(z) - 1/2 cancels every digit at a tiny z (GAUSS(1E-300) is 3.99E-301); erf keeps them.
        return mpmath.erf(z / mpmath.sqrt(2)) / 2
    return cdf(z) - mpmath.mpf(1) / 2


def _norm_s_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    (p,) = scalars(args)
    return standard_quantile(p)


def _norm_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, mean, sd = scalars(args)
    return mean + sd * standard_quantile(p)


def _standard_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    """The standard z whose lower (or upper) tail is q."""
    del args
    z = standard_quantile(q)
    return -z if upper else z


def _general_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    """NORM.DIST's x whose lower (or upper) tail is q: mean + sd*z."""
    mean, sd = args[1], args[2]
    if isinstance(mean, (bool, tuple)) or isinstance(sd, (bool, tuple)) or sd <= 0:
        msg = "no tail map without a positive standard deviation"
        raise ValueError(msg)
    return mean + sd * _standard_tail(q, upper, args)


def _gauss_edge(rng: random.Random) -> float:
    """A GAUSS point at an evidenced break: 0, the leading term, just past the cancellation region, or the flush."""
    shape = rng.randrange(5)
    if shape == 0:
        return 0.0
    if shape == 1:  # the leading term up to its evidenced end
        return signed(rng, rng.choice((1e-300, GAUSS_LEADING, log_uniform(rng, 1e-300, GAUSS_LEADING))))
    if shape == 2:  # just past the cancellation region
        return signed(rng, log_uniform(rng, GAUSS_CANCEL_HI, 1e-4))
    if shape == 3:  # the lower tail's flush
        return signed(rng, rng.choice((FLUSH, 37.5, rng.uniform(37.0, 39.0))))
    return signed(rng, rng.choice((1.0, 2.0, 8.0)))


def _gauss_z(stratum: str, rng: random.Random) -> float:
    """One GAUSS argument of `stratum` (the caller redraws the cancellation region)."""
    if stratum == "tail":
        return signed(rng, rng.uniform(3.0, 40.0))
    if stratum == "edge":
        return _gauss_edge(rng)
    if stratum == "extreme":
        return signed(rng, log_uniform(rng, 40.0, 1e300))
    if stratum == "log-uniform":
        return signed(rng, log_uniform(rng, 1e-300, 1e300))
    return rng.uniform(-4.0, 4.0)


def gauss_cancels(z: float) -> bool:
    """Whether GAUSS(z) is in the excluded cancellation region."""
    return GAUSS_LEADING < abs(z) < GAUSS_CANCEL_HI


def _draw_gauss(stratum: str, rng: random.Random) -> list[Arg]:
    for _ in range(REDRAWS):
        z = float(f"{_gauss_z(stratum, rng):.15g}")
        if not gauss_cancels(z):
            return [z]
    return [1.0]


def inv_cancels(p: float, mean: float, sd: float) -> bool:
    """Whether mean + sd*NORM.S.INV(p) is below `INV_CANCEL` of its larger term (the excluded region)."""
    with mpmath.workdps(30):
        term = mpmath.mpf(sd) * standard_quantile(mpmath.mpf(p))
        total = mpmath.mpf(mean) + term
        return bool(abs(total) < INV_CANCEL * max(abs(mpmath.mpf(mean)), abs(term)))


def _inv_p(stratum: str, rng: random.Random) -> float:
    if stratum in ("tail", "extreme"):
        q = log_uniform(rng, 1e-300, 1e-15) if stratum == "extreme" else log_uniform(rng, 1e-15, 1e-3)
        return 1 - q if rng.random() < 0.3 and q > 1e-15 else q
    if stratum == "edge":
        return rng.choice(
            (0.5, 0.075, 0.925, 0.074999999999999, 0.925000000000001, 5e-309, 1e-300, 0.999999999999999)
        )
    if stratum == "log-uniform":
        q = log_uniform(rng, 1e-300, 0.5)
        return 1 - q if rng.random() < 0.5 and q > 1e-15 else q
    return rng.uniform(0.001, 0.999)


def _inv_mean_sd(stratum: str, rng: random.Random) -> tuple[float, float]:
    if stratum == "extreme":
        return signed(rng, log_uniform(rng, 1e-100, 1e100)), log_uniform(rng, 1e-100, 1e100)
    if stratum == "log-uniform":
        return signed(rng, log_uniform(rng, 1e-300, 1e300)), log_uniform(rng, 1e-300, 1e300)
    return rng.uniform(-100.0, 100.0), log_uniform(rng, 0.01, 100.0)


def _draw_norm_inv(stratum: str, rng: random.Random) -> list[Arg]:
    """NORM.INV's point: p from the stratum's probability range, mean and sd beside it, outside the cancellation
    region; a mean + sd*z beyond the double range is redrawn."""
    for _ in range(REDRAWS):
        p = float(f"{_inv_p(stratum, rng):.15g}")
        mean, sd = _inv_mean_sd(stratum, rng)
        mean, sd = float(f"{mean:.15g}"), float(f"{sd:.15g}")
        if 0 < p < 1 and sd > 0 and not inv_cancels(p, mean, sd):
            return [p, mean, sd]
    return [0.3, 1.0, 2.0]


def _draw_norm_s_inv(stratum: str, rng: random.Random) -> list[Arg]:
    p = float(f"{_inv_p(stratum, rng):.15g}")
    return [p if 0 < p < 1 else 0.5]


def _no_overflow(value: mpmath.mpf) -> mpmath.mpf:
    """`value`, or NoValueError past the largest double (the sampler redraws)."""
    if abs(value) > sys.float_info.max:
        msg = "beyond the double range"
        raise NoValueError(msg)
    return value


_PROBABILITY = Param(
    "probability",
    "real",
    Interval(0, 1),
    typical=(0.001, 0.999),
    extreme=(1e-300, 1e-15),
    edges=(0.5, 0.075, 0.925),
    source=INV_SOURCE,
)
_SD = Param(
    "standard_dev",
    "real",
    Interval(0, float("inf")),
    typical=(0.1, 10),
    extreme=(1e-100, 1e100),
    source=SD_SOURCE,
)

NORM_DIST = register(
    Definition(
        function="NORM.DIST",
        params=(
            Param("x", "real", typical=(-10, 10), extreme=(1e2, 1e100), edges=(0.0,), source=SWEEP_SOURCE),
            Param("mean", "real", typical=(-10, 10), extreme=(1e2, 1e100), source=SWEEP_SOURCE),
            _SD,
            Param("cumulative", "bool", source="catalog"),
        ),
        value=_norm_dist,
        domain=_norm_dist_domain,
        tail=_general_tail,
        tail_param="x",
        sources=(SWEEP_SOURCE, CDF_SOURCE, DENSITY_SOURCE, OVERFLOW_SOURCE, SD_SOURCE),
    )
)

NORM_S_DIST = register(
    Definition(
        function="NORM.S.DIST",
        params=(
            Param(
                "z",
                "real",
                typical=(-6, 6),
                extreme=(40, 1e300),
                edges=(0.0, -FLUSH, OVERFLOW),
                source=SWEEP_SOURCE,
            ),
            Param("cumulative", "bool", source="catalog"),
        ),
        value=_norm_s_dist,
        domain=_norm_s_dist_domain,
        tail=_standard_tail,
        tail_param="z",
        sources=(SWEEP_SOURCE, CDF_SOURCE, DENSITY_SOURCE, OVERFLOW_SOURCE),
    )
)

PHI = register(
    Definition(
        function="PHI",
        params=(
            Param(
                "x",
                "real",
                typical=(-6, 6),
                extreme=(40, 1e300),
                edges=(0.0, -FLUSH, OVERFLOW),
                source=DENSITY_SOURCE,
            ),
        ),
        value=_phi,
        domain=_phi_domain,
        sources=(DENSITY_SOURCE, OVERFLOW_SOURCE),
    )
)

GAUSS = register(
    Definition(
        function="GAUSS",
        params=(Param("z", "real", typical=(-4, 4), extreme=(40, 1e300), source=GAUSS_SOURCE),),
        value=_gauss,
        draw=_draw_gauss,
        sources=(GAUSS_SOURCE, SWEEP_SOURCE),
    )
)

NORM_S_INV = register(
    Definition(
        function="NORM.S.INV",
        params=(_PROBABILITY,),
        value=_norm_s_inv,
        draw=_draw_norm_s_inv,
        sources=(INV_SOURCE, SWEEP_SOURCE),
    )
)

NORM_INV = register(
    Definition(
        function="NORM.INV",
        params=(
            _PROBABILITY,
            Param("mean", "real", typical=(-100, 100), extreme=(1e-100, 1e100), source=INV_SOURCE),
            _SD,
        ),
        value=lambda args: _no_overflow(_norm_inv(args)),
        draw=_draw_norm_inv,
        sources=(INV_SOURCE, SD_SOURCE),
    )
)
