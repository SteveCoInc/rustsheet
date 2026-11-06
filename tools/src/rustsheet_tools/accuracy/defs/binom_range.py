"""BINOM.DIST.RANGE, BINOM.INV and CRITBINOM at trial counts up to 2^31 - 2 (T-3741).

Sources:
- statistics.md, "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's own cdf (T-3444)": BINOM.INV (and
  CRITBINOM, equal on every probe row) is the smallest k with P(K <= k) >= alpha; BINOM.DIST.RANGE(n, p, s1, s2) is
  P(s1 <= K <= s2), and a single-point range (number_s2 left out) is the pmf.
- statistics.md, "BINOM.DIST is #NUM! from 2^31 - 1 trials (T-2464, T-2645)" and "BINOM.INV and CRITBINOM take
  BINOM.DIST's trial limit (T-2435)": the accepted trial count is n < 2^31 - 1.
- statistics.md, "BINOM.DIST's pmf at k = 0 is #NUM! when p = 1 (T-2435)": BINOM.DIST.RANGE is 0 there, not #NUM!
  (`BINOM.DIST.RANGE(2,1,0)`).
- statistics.md, "Legacy distribution names sweep" (T-1048): BINOM.INV's and CRITBINOM's probability and alpha are
  #NUM! at 0 and at 1 (open intervals).
- Microsoft's documentation: BINOM.DIST.RANGE is #NUM! when number_s > trials, number_s2 < number_s or
  number_s2 > trials.
- The math: the binomial pmf C(n, k) p^k (1-p)^(n-k) summed outward from k (Abramowitz and Stegun 26.1.20).

Why a sum and not `discrete.binom_cdf`: at n near 2^31 and k near the mean, mpmath's incomplete beta (a 2F1 series
with a = n - k) does not converge, so BINOM.DIST's own table has no near-mean point at a huge n. The pmf's term ratio
moves monotonically away from the mode, so summing from k away from the mode is a positive series whose tail is at
most term * r / (1 - r) once the ratio r is below 1; near the mean at n = 2^31 - 2 it takes about 20 standard
deviations, under 500,000 terms, each charged to the work budget.
"""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, real_args, sum_away_from_mode
from rustsheet_tools.accuracy.defs.discrete import TRIALS_LIMIT, TRIALS_SOURCE
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register, spend
from rustsheet_tools.accuracy.sampler import log_uniform

T3444_SOURCE = "statistics.md#binomdistrange-and-binominv-at-a-large-n-sit-on-binomdists-own-cdf-t-3444"
INV_TRIALS_SOURCE = "statistics.md#binominv-and-critbinom-take-binomdists-trial-limit-t-2435"
INV_OPEN_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"
P1_SOURCE = "statistics.md#binomdists-pmf-at-k--0-is-num-when-p--1-t-2435"
MS_DOCS = "Microsoft documentation"
MAX_WALK = 1_000_000  # quantile steps past the normal start: a deterministic drop, never reached in practice
REFRESH = (
    mpmath.mpf(10) ** -10
)  # a downward walk recomputes its cdf once it has fallen this far (lost digits)


def _log_pmf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """ln P(K = k) for 0 < p < 1, from log-gammas so a count of 2^31 does not overflow."""
    return (
        mpmath.loggamma(n + 1)
        - mpmath.loggamma(k + 1)
        - mpmath.loggamma(n - k + 1)
        + k * mpmath.log(p)
        + (n - k) * mpmath.log1p(-p)
    )


def _pmf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K = k), 0 <= k <= n, any p in [0, 1]."""
    if p in (0, 1):
        return mpmath.mpf(1 if k == (n if p == 1 else 0) else 0)
    return mpmath.exp(_log_pmf(k, n, p))


def _mode(n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    return mpmath.floor((n + 1) * p)


def _sum_from(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf, *, upward: bool) -> mpmath.mpf:
    """P(K >= k) (upward) or P(K <= k), summed term by term from k away from the mode, for 0 < p < 1 and
    0 <= k <= n (`sum_away_from_mode`)."""
    q = 1 - p
    if upward:
        return sum_away_from_mode(_pmf(k, n, p), k, n, 1, lambda j: (n - j) * p / ((j + 1) * q))
    return sum_away_from_mode(_pmf(k, n, p), k, mpmath.mpf(0), -1, lambda j: j * q / ((n - j + 1) * p))


def binom_cdf_summed(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K <= k), the smaller side summed: below the mode the lower tail directly, else 1 - P(K >= k + 1)."""
    if k < 0:
        return mpmath.mpf(0)
    if k >= n:
        return mpmath.mpf(1)
    if p in (0, 1):
        return mpmath.mpf(1 if p == 0 else 0)
    if k < _mode(n, p):
        return _sum_from(k, n, p, upward=False)
    return 1 - _sum_from(k + 1, n, p, upward=True)


def binom_sf_summed(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K > k), the smaller side summed (the upper tail directly above the mode)."""
    if k < 0:
        return mpmath.mpf(1)
    if k >= n:
        return mpmath.mpf(0)
    if p in (0, 1):
        return mpmath.mpf(0 if p == 0 else 1)
    if k + 1 > _mode(n, p):
        return _sum_from(k + 1, n, p, upward=True)
    return 1 - _sum_from(k, n, p, upward=False)


def binom_range(n: mpmath.mpf, p: mpmath.mpf, s1: mpmath.mpf, s2: mpmath.mpf) -> mpmath.mpf:
    """P(s1 <= K <= s2): the pmf for one point; else a difference of the two lower tails when s1 is at or below
    the mode, of the two upper tails above it, so the difference never cancels to fewer than the range's digits."""
    if s1 == s2:
        return _pmf(s1, n, p)
    if p in (0, 1):
        at = n if p == 1 else 0
        return mpmath.mpf(1 if s1 <= at <= s2 else 0)
    if s1 <= _mode(n, p):
        return binom_cdf_summed(s2, n, p) - binom_cdf_summed(s1 - 1, n, p)
    return binom_sf_summed(s1 - 1, n, p) - binom_sf_summed(s2, n, p)


def _normal_start(alpha: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """A start for the quantile walk: the normal approximation, clamped to [0, n]. Past erfinv's range (alpha within
    1E-300 of 0 or 1) the tail asymptote z = sqrt(2 ln(1 / q)) is close enough; the walk finishes the job."""
    centre = 2 * alpha - 1
    z = mpmath.erfinv(centre)
    if not mpmath.isfinite(z):
        tail = min(alpha, 1 - alpha)
        z = mpmath.sqrt(-2 * mpmath.log(tail)) * (1 if alpha > mpmath.mpf(0.5) else -1)
    x = mpmath.floor(n * p + z * mpmath.sqrt(n * p * (1 - p)))
    return min(max(x, mpmath.mpf(0)), n)


def binom_quantile(alpha: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """The smallest k in [0, n] with P(K <= k) >= alpha, for 0 < p < 1 and 0 < alpha < 1: from the normal start, a
    walk of one pmf step at a time (each step one ratio, so a start a few standard deviations off costs little
    where a bisection would sum hundreds of thousands of terms per probe near the mean). A downward walk subtracts,
    so once its cdf has fallen by REFRESH it is summed afresh; the answer is then checked with two fresh sums."""
    k = _normal_start(alpha, n, p)
    cdf = binom_cdf_summed(k, n, p)
    q = 1 - p
    if cdf < alpha:
        term = _pmf(k, n, p)
        while cdf < alpha:
            spend()
            term *= (n - k) * p / ((k + 1) * q)
            k += 1
            cdf += term
    else:
        term = _pmf(k, n, p)
        anchor = cdf
        for _ in range(MAX_WALK):
            if k == 0 or cdf - term < alpha:
                break
            spend()
            cdf -= term
            term *= k * q / ((n - k + 1) * p)
            k -= 1
            if cdf < anchor * REFRESH:
                cdf = anchor = binom_cdf_summed(k, n, p)
    if binom_cdf_summed(k, n, p) < alpha or (k > 0 and binom_cdf_summed(k - 1, n, p) >= alpha):
        msg = "quantile walk ended off the crossing"
        raise mpmath.libmp.NoConvergence(msg)
    return k


# -- BINOM.DIST.RANGE ---------------------------------------------------------------------------------------------


def _range_value(args: Sequence[MathArg]) -> mpmath.mpf:
    n, p, s1, *rest = real_args(args)
    return binom_range(n, p, s1, rest[0] if rest else s1)


def _range_domain(args: Sequence[MathArg]) -> str | None:
    n, _, s1, *rest = real_args(args)
    s2 = rest[0] if rest else s1
    if s1 > n:
        return "number_s above trials"
    if s2 < s1:
        return "number_s2 below number_s"
    if s2 > n:
        return "number_s2 above trials"
    return None


def _spread_count(n: int, p: float, sds: float) -> int:
    """A count `sds` standard deviations from the mean, inside [0, n]."""
    x = round(n * p + sds * float((n * p * (1 - p)) ** 0.5))
    return min(max(x, 0), n)


def _huge_trials(rng: random.Random) -> int:
    return round(log_uniform(rng, 1e6, TRIALS_LIMIT)) if rng.random() < 0.8 else TRIALS_LIMIT


def _probability(rng: random.Random) -> float:
    """A probability: mostly central, sometimes small, sometimes near 1."""
    shape = rng.random()
    if shape < 0.6:
        return round15(rng.uniform(0.01, 0.99))
    if shape < 0.85:
        return round15(log_uniform(rng, 1e-6, 1e-2))
    return round15(1 - log_uniform(rng, 1e-6, 1e-2))


def _range_args(n: int, p: float, s1: int, s2: int | None) -> list[Arg]:
    return [n, p, s1] if s2 is None else [n, p, s1, s2]


def _range_typical(rng: random.Random) -> list[Arg]:
    n = rng.randint(1, 80)
    p = round15(rng.uniform(0.01, 0.99))
    s1 = rng.randint(0, n)
    return _range_args(n, p, s1, None if rng.random() < 0.25 else rng.randint(s1, n))


def _range_tail(rng: random.Random) -> list[Arg]:
    """A range deep in one tail: both ends 4 to 40 standard deviations out, at up to a million trials."""
    n = round(log_uniform(rng, 50, 1e6))
    p = _probability(rng)
    side = rng.choice((-1.0, 1.0))
    a, b = sorted(side * rng.uniform(4, 40) for _ in range(2))
    s1, s2 = _spread_count(n, p, a), _spread_count(n, p, b)
    return _range_args(n, p, s1, None if s1 == s2 or rng.random() < 0.2 else s2)


def _range_extreme(rng: random.Random) -> list[Arg]:
    """A huge trial count, the range anywhere from 8 sd below the mean to 8 above (mean + 3 sd to mean + 5 sd is
    T-3740's upper-tail form), a tenth of a standard deviation to 10 wide, or one point."""
    n = _huge_trials(rng)
    p = _probability(rng)
    s1 = _spread_count(n, p, rng.uniform(-8, 8))
    if rng.random() < 0.2:
        return _range_args(n, p, s1, None)
    width = log_uniform(rng, 0.1, 10) * (n * p * (1 - p)) ** 0.5
    return _range_args(n, p, s1, min(n, s1 + max(1, round(width))))


def _range_log_uniform(rng: random.Random) -> list[Arg]:
    """Trials log-uniform over [1, 2^31 - 2], a probability log-uniform over [1E-300, 1], the range around the
    mean (within 12 sd)."""
    n = round(log_uniform(rng, 1, TRIALS_LIMIT))
    p = round15(log_uniform(rng, 1e-300, 1))
    a, b = sorted(rng.uniform(-12, 12) for _ in range(2))
    s1, s2 = _spread_count(n, p, a), _spread_count(n, p, b)
    return _range_args(n, p, s1, None if rng.random() < 0.2 else s2)


def _range_edge(rng: random.Random) -> list[Arg]:
    """The bounds: no trials, the whole range, the last count, p of 0 and 1, the trial limit and one past it, and
    the documented #NUM! orders."""
    n = rng.randint(1, 60)
    p = round15(rng.uniform(0.01, 0.99))
    s = rng.randint(0, n)
    shapes: list[list[Arg]] = [
        [0, p, 0],
        [0, p, 0, 0],
        [n, p, 0, n],
        [n, p, n],
        [n, p, 0],
        [n, 0, 0, s],
        [n, 0, s],
        [n, 1, s, n],
        [n, 1, 0],
        [n, 1, s],
        [TRIALS_LIMIT, p, _spread_count(TRIALS_LIMIT, p, rng.uniform(-3, 3))],
        [TRIALS_LIMIT, p, 0, _spread_count(TRIALS_LIMIT, p, rng.uniform(-3, 3))],
        [TRIALS_LIMIT + 1, p, 0],
        [n, p, n + 1],
        [n, p, s, n + 1],
        [n, p, s + 1, s] if s < n else [n, p, n, n - 1],
        [n, round15(log_uniform(rng, 1e-300, 1e-15)), 0, s],
        [n, round15(1 - log_uniform(rng, 1e-15, 1e-6)), s, n],
    ]
    return rng.choice(shapes)


_COUNT = Interval(0, TRIALS_LIMIT, lo_open=False, hi_open=False)

BINOM_DIST_RANGE = register(
    Definition(
        function="BINOM.DIST.RANGE",
        params=(
            Param(
                "trials", "int", _COUNT, typical=(1, 80), extreme=(1e6, TRIALS_LIMIT), source=TRIALS_SOURCE
            ),
            Param(
                "probability_s",
                "real",
                Interval(0, 1, lo_open=False, hi_open=False),
                typical=(0.01, 0.99),
                source=P1_SOURCE,
            ),
            Param("number_s", "int", _COUNT, typical=(0, 80), source=MS_DOCS),
            Param("number_s2", "int", _COUNT, typical=(0, 80), source=MS_DOCS, optional=True),
        ),
        value=_range_value,
        domain=_range_domain,
        draw=by_stratum(
            {
                "typical": _range_typical,
                "tail": _range_tail,
                "edge": _range_edge,
                "extreme": _range_extreme,
                "log-uniform": _range_log_uniform,
            },
            "BINOM.DIST.RANGE draws: small ranges, deep tails, huge trial counts near the mean, and the bounds.",
        ),
        sources=(T3444_SOURCE, TRIALS_SOURCE, P1_SOURCE, MS_DOCS, "Abramowitz and Stegun 26.1.20"),
    )
)


# -- BINOM.INV and CRITBINOM --------------------------------------------------------------------------------------


def _inv_value(args: Sequence[MathArg]) -> mpmath.mpf:
    n, p, alpha = real_args(args)
    if n == 0:
        return mpmath.mpf(0)
    return binom_quantile(alpha, n, p)


def _alpha(rng: random.Random) -> float:
    """A tail probability: alpha log-uniform over [1E-300, 0.1] in the lower tail, or 1 - q with q log-uniform over
    [1E-15, 0.1] in the upper (a 1 - q below that is 1 as a 15-digit literal: alpha 1 is BINOM.INV's #NUM!, which the
    edge stratum covers)."""
    if rng.random() < 0.5:
        return round15(log_uniform(rng, 1e-300, 0.1))
    return round15(1 - log_uniform(rng, 1e-15, 0.1))


def _inv_typical(rng: random.Random) -> list[Arg]:
    return [rng.randint(1, 80), round15(rng.uniform(0.01, 0.99)), round15(rng.uniform(0.001, 0.999))]


def _inv_tail(rng: random.Random) -> list[Arg]:
    return [round(log_uniform(rng, 1, 1e6)), _probability(rng), _alpha(rng)]


def _inv_extreme(rng: random.Random) -> list[Arg]:
    alpha = round15(rng.uniform(0.001, 0.999)) if rng.random() < 0.5 else _alpha(rng)
    return [_huge_trials(rng), _probability(rng), alpha]


def _inv_log_uniform(rng: random.Random) -> list[Arg]:
    alpha = round15(log_uniform(rng, 1e-300, 1))
    return [round(log_uniform(rng, 1, TRIALS_LIMIT)), round15(log_uniform(rng, 1e-300, 1)), alpha]


def _inv_edge(rng: random.Random) -> list[Arg]:
    """The bounds: no trials, one trial, the trial limit and one past it, p and alpha at 0, 1 and beside them."""
    n = rng.randint(1, 60)
    p = round15(rng.uniform(0.01, 0.99))
    alpha = round15(rng.uniform(0.01, 0.99))
    shapes: list[list[Arg]] = [
        [0, p, alpha],
        [1, p, alpha],
        [TRIALS_LIMIT, p, alpha],
        [TRIALS_LIMIT + 1, p, alpha],
        [n, 0, alpha],
        [n, 1, alpha],
        [n, p, 0],
        [n, p, 1],
        [n, round15(log_uniform(rng, 1e-300, 1e-15)), alpha],
        [n, round15(1 - log_uniform(rng, 1e-15, 1e-6)), alpha],
        [n, p, round15(log_uniform(rng, 1e-300, 1e-200))],
        [n, p, round15(1 - log_uniform(rng, 1e-15, 1e-12))],
    ]
    return rng.choice(shapes)


def _inv_definition(function: str) -> Definition:
    return Definition(
        function=function,
        params=(
            Param(
                "trials",
                "int",
                _COUNT,
                typical=(1, 80),
                extreme=(1e6, TRIALS_LIMIT),
                source=INV_TRIALS_SOURCE,
            ),
            Param("probability_s", "real", Interval(0, 1), typical=(0.01, 0.99), source=INV_OPEN_SOURCE),
            Param("alpha", "real", Interval(0, 1), typical=(0.001, 0.999), source=INV_OPEN_SOURCE),
        ),
        value=_inv_value,
        draw=by_stratum(
            {
                "typical": _inv_typical,
                "tail": _inv_tail,
                "edge": _inv_edge,
                "extreme": _inv_extreme,
                "log-uniform": _inv_log_uniform,
            },
            f"{function} draws: small counts, tail alphas, huge trial counts and the bounds.",
        ),
        sources=(T3444_SOURCE, INV_TRIALS_SOURCE, INV_OPEN_SOURCE, "Abramowitz and Stegun 26.1.20"),
    )


BINOM_INV = register(_inv_definition("BINOM.INV"))
CRITBINOM = register(_inv_definition("CRITBINOM"))
