"""The discrete distributions: BINOM.DIST, NEGBINOM.DIST and POISSON.DIST.

Sources:
- statistics.md, "BINOM.DIST is #NUM! from 2^31 - 1 trials (T-2464, T-2645)": the accepted trial count is n < 2^31 - 1,
  whatever the other arguments; NEGBINOM.DIST has no such limit.
- statistics.md, "BINOM.DIST's pmf at k = 0 is #NUM! when p = 1 (T-2435)": `BINOM.DIST(0,n,1,FALSE)` is `#NUM!` from
  one trial on, while the cdf there is 0.
- statistics.md, "POISSON at x 0 with a negative mean": at x 0 a negative mean gives EXP(-mean), cumulative or not;
  at x 1 or more a negative mean is `#NUM!`.
- statistics.md, "Legacy distribution names sweep, second family (T-1048)": NEGBINOM's probability is `#NUM!` at
  p = 1 and at p = 0 (an open interval), where BINOM.DIST's is closed.
- Microsoft's documentation: number_s > trials, a BINOM.DIST probability outside [0, 1], number_f < 0, number_s < 1
  (NEGBINOM) and x < 0 are `#NUM!`.
- The math: the binomial pmf C(n, k) p^k (1-p)^(n-k) and cdf I_{1-p}(n-k, k+1); the negative binomial pmf
  C(f+s-1, f) p^s (1-p)^f and cdf I_p(s, f+1) (Abramowitz and Stegun 26.5.24, 26.5.26, the regularized incomplete
  beta); the Poisson pmf e^-m m^x / x! and cdf Q(x+1, m), the regularized upper incomplete gamma (A&S 26.4.21).
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, scalars, sum_away_from_mode
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import (
    Definition,
    Draw,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)
from rustsheet_tools.accuracy.sampler import log_uniform

TRIALS_SOURCE = "statistics.md#binomdist-is-num-from-231---1-trials-t-2464-t-2645"
P1_SOURCE = "statistics.md#binomdists-pmf-at-k--0-is-num-when-p--1-t-2435"
NEGATIVE_MEAN_SOURCE = "statistics.md#poisson-at-x-0-with-a-negative-mean"
TRIALS_LIMIT = 2**31 - 2  # the largest accepted (truncated) trial count
COUNT_DRAW_HI = 1e12  # the largest count drawn (a cap on the math, not an Excel limit)
_PROBABILITY = Param(
    "probability_s",
    "real",
    Interval(0, 1, lo_open=False, hi_open=False),
    typical=(0.01, 0.99),
    extreme=(1e-300, 1e-6),
    source="Microsoft documentation",
)
NEGBINOM_P_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"
# NEGBINOM.DIST's probability is open at both ends: `NEGBINOM.DIST(21,4,0,TRUE)` and `NEGBINOM.DIST(20,1,1,FALSE)`
# are #NUM! in Excel (accuracy/NEGBINOM.DIST/s1 A117 and A119; also A127, A137 at p = 0, A129, A139 at p = 1).
_NEGBINOM_PROBABILITY = Param(
    "probability_s",
    "real",
    Interval(0, 1),
    typical=(0.01, 0.99),
    extreme=(1e-300, 1e-6),
    source=NEGBINOM_P_SOURCE,
)
_CUMULATIVE = Param("cumulative", "bool", source="catalog")


def _reals(args: Sequence[MathArg]) -> list[mpmath.mpf]:
    """The numeric arguments: every argument but the cumulative flag, which the legacy pmf-only names (NEGBINOMDIST,
    HYPGEOMDIST) do not take."""
    return scalars([a for a in args if not isinstance(a, bool)])


def _cumulative(args: Sequence[MathArg]) -> bool:
    """The trailing cumulative flag, False for a legacy pmf-only call that has none."""
    return args[-1] is True


def binom_pmf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K = k) for K ~ Binomial(n, p), 0 ≤ k ≤ n."""
    if p in (0, 1):
        return mpmath.mpf(1 if k == (n if p == 1 else 0) else 0)
    return mpmath.binomial(n, k) * p**k * (1 - p) ** (n - k)


def binom_cdf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K ≤ k): I_{1-p}(n-k, k+1), and 1 at k = n."""
    if k >= n or p == 0:
        return mpmath.mpf(1)
    if p == 1:
        return mpmath.mpf(0)
    return mpmath.betainc(n - k, k + 1, 0, 1 - p, regularized=True)


def negbinom_pmf(f: mpmath.mpf, s: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(F = f) for the number of failures F before the s-th success, 0 < p < 1."""
    return mpmath.binomial(f + s - 1, f) * p**s * (1 - p) ** f


def negbinom_cdf(f: mpmath.mpf, s: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(F ≤ f): I_p(s, f+1)."""
    return mpmath.betainc(s, f + 1, 0, p, regularized=True)


def poisson_pmf(x: mpmath.mpf, m: mpmath.mpf) -> mpmath.mpf:
    """P(X = x) for X ~ Poisson(m), m ≥ 0."""
    if m == 0:
        return mpmath.mpf(1 if x == 0 else 0)
    return mpmath.exp(x * mpmath.log(m) - m - mpmath.loggamma(x + 1))


def poisson_cdf(x: mpmath.mpf, m: mpmath.mpf) -> mpmath.mpf:
    """P(X ≤ x): Q(x+1, m). NoValueError past `COUNT_DRAW_HI`: mpmath's upper gamma at an integer order sums x terms
    (3 s at x = 1E12, hours at the probes' `POISSON(1E16,1E16,TRUE)`), so such an eval cell gets no math value."""
    if m == 0:
        return mpmath.mpf(1)
    if x > COUNT_DRAW_HI:
        msg = "count past the math's range"
        raise NoValueError(msg)
    return mpmath.gammainc(x + 1, m, mpmath.inf, regularized=True)


def binom_sf(k: mpmath.mpf, n: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(K > k): I_p(k+1, n-k), taken directly so a tiny upper tail keeps its digits."""
    if k >= n or p == 0:
        return mpmath.mpf(0)
    if p == 1:
        return mpmath.mpf(1)
    return mpmath.betainc(k + 1, n - k, 0, p, regularized=True)


def negbinom_sf(f: mpmath.mpf, s: mpmath.mpf, p: mpmath.mpf) -> mpmath.mpf:
    """P(F > f): I_{1-p}(f+1, s)."""
    return mpmath.betainc(f + 1, s, 0, 1 - p, regularized=True)


def poisson_sf(x: mpmath.mpf, m: mpmath.mpf) -> mpmath.mpf:
    """P(X > x): P(x+1, m), the regularized lower incomplete gamma."""
    if m == 0:
        return mpmath.mpf(0)
    return mpmath.gammainc(x + 1, 0, m, regularized=True)


Tail = Callable[[mpmath.mpf], mpmath.mpf]


def integer_quantile(cdf: Tail, sf: Tail, q: mpmath.mpf, upper: bool, hi: float) -> mpmath.mpf:
    """The smallest integer k in [0, hi] with cdf(k) ≥ q (lower tail) or sf(k) ≤ q (upper tail): a doubling search
    from 1 brackets it, so no tail is evaluated far beyond the answer, then bisection over the integers. ValueError
    when no such k is at most `hi`."""

    def reached(k: mpmath.mpf) -> bool:
        return bool(sf(k) <= q) if upper else bool(cdf(k) >= q)

    lo, top = mpmath.mpf(0), mpmath.mpf(1)
    while not reached(top):
        lo, top = top + 1, top * 2
        if lo > hi:
            msg = "tail quantile beyond the count range"
            raise ValueError(msg)
    top = min(top, mpmath.mpf(int(hi)))
    if reached(lo):
        return lo
    while top - lo > 1:
        mid = mpmath.floor((lo + top) / 2)
        if reached(mid):
            top = mid
        else:
            lo = mid
    return top


def _binom_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    _, n, p = _reals(args)
    return integer_quantile(lambda k: binom_cdf(k, n, p), lambda k: binom_sf(k, n, p), q, upper, float(n))


def _negbinom_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    _, s, p = _reals(args)
    return integer_quantile(
        lambda f: negbinom_cdf(f, s, p), lambda f: negbinom_sf(f, s, p), q, upper, COUNT_DRAW_HI
    )


def _poisson_tail(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    _, m = _reals(args)
    if m < 0:
        msg = "no tail map at a negative mean"
        raise ValueError(msg)
    return integer_quantile(lambda x: poisson_cdf(x, m), lambda x: poisson_sf(x, m), q, upper, COUNT_DRAW_HI)


def _binom_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    k, n, p = _reals(args)
    return binom_cdf(k, n, p) if _cumulative(args) else binom_pmf(k, n, p)


def _binom_domain(args: Sequence[MathArg]) -> str | None:
    k, n, p = _reals(args)
    if k > n:
        return "number_s above trials"
    if p == 1 and k == 0 and n >= 1 and not _cumulative(args):
        return "pmf at k = 0 with p = 1 (Excel's LN(1 - p) closed form)"
    return None


def _negbinom_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    f, s, p = _reals(args)
    return negbinom_cdf(f, s, p) if _cumulative(args) else negbinom_pmf(f, s, p)


def _poisson_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, m = _reals(args)
    if m < 0:
        # Excel's quirk: x = 0 with a negative mean is EXP(-mean), whatever the cumulative flag.
        value = mpmath.exp(-m)
        if value > mpmath.mpf("1.7976931348623157e308"):
            msg = "EXP(-mean) overflows"
            raise NoValueError(msg)
        return value
    return poisson_cdf(x, m) if _cumulative(args) else poisson_pmf(x, m)


def _poisson_domain(args: Sequence[MathArg]) -> str | None:
    x, m = _reals(args)
    if m < 0 and x > 0:
        return "negative mean above x = 0"
    return None


def _count(
    name: str,
    lo: float,
    typical: tuple[float, float],
    extreme: tuple[float, float],
    source: str,
) -> Param:
    """A count with no Excel upper limit: Excel accepts huge counts (`NEGBINOM.DIST(2^31,1,1-1E-12,FALSE)` is 0,
    statistics.md "BINOM.DIST is #NUM! from 2^31 - 1 trials"), so the domain is unbounded above and only the
    draws are capped."""
    return Param(
        name,
        "int",
        Interval(lo, float("inf"), lo_open=False),
        typical=typical,
        extreme=extreme,
        source=source,
        draw_hi=COUNT_DRAW_HI,
    )


def _binom_definition(function: str) -> Definition:
    """BINOM.DIST, and BINOMDIST with the same arguments and rules: equal on every paired cell (statistics.md "Legacy
    distribution names sweep (T-1047)"), and the trial limit and the p = 1 pmf are probed on both names."""
    return Definition(
        function=function,
        params=(
            # number_s and trials are drawn apart, so their ranges keep number_s ≤ trials in the typical and extreme
            # strata (number_s > trials is `#NUM!`); the tail stratum sets number_s from trials' own quantile.
            Param(
                "number_s",
                "int",
                Interval(0, TRIALS_LIMIT, lo_open=False, hi_open=False),
                typical=(0, 20),
                extreme=(1e2, 1e6),
                source="Microsoft documentation",
            ),
            Param(
                "trials",
                "int",
                Interval(0, TRIALS_LIMIT, lo_open=False, hi_open=False),
                typical=(20, 80),
                extreme=(1e6, TRIALS_LIMIT),
                edges=(TRIALS_LIMIT,),
                source=TRIALS_SOURCE,
            ),
            _PROBABILITY,
            _CUMULATIVE,
        ),
        value=_binom_dist,
        domain=_binom_domain,
        tail=_binom_tail,
        tail_param="number_s",
        sources=(TRIALS_SOURCE, P1_SOURCE, "Abramowitz and Stegun 26.5.24"),
    )


def _negbinom_definition(function: str, *, cumulative: bool) -> Definition:
    """NEGBINOM.DIST, and NEGBINOMDIST without the cumulative flag: it is NEGBINOM.DIST's pmf on every paired cell
    (statistics.md "Legacy distribution names sweep, second family (T-1048)"), p = 1 included."""
    params = (
        _count("number_f", 0, (0, 30), (1e3, 1e12), "Microsoft documentation"),
        _count("number_s", 1, (1, 30), (1e3, 1e12), "Microsoft documentation"),
        _NEGBINOM_PROBABILITY,
    )
    return Definition(
        function=function,
        params=(*params, _CUMULATIVE) if cumulative else params,
        value=_negbinom_dist,
        tail=_negbinom_tail,
        tail_param="number_f",
        sources=(TRIALS_SOURCE, NEGBINOM_P_SOURCE, "Abramowitz and Stegun 26.5.24, 26.5.26"),
    )


def _poisson_definition(function: str) -> Definition:
    """POISSON.DIST, and POISSON with the same arguments and rules: equal on every paired cell (statistics.md "Legacy
    distribution names sweep, second family (T-1048)"), and the negative-mean quirk is probed on both names."""
    return Definition(
        function=function,
        params=(
            _count("x", 0, (0, 40), (1e3, 1e12), "Microsoft documentation"),
            Param(
                "mean",
                "real",
                Interval(),
                typical=(0.1, 40),
                extreme=(1e3, 1e12),
                edges=(0.0,),
                source=NEGATIVE_MEAN_SOURCE,
                # `POISSON.DIST(2,-2.09435621420062E+249,TRUE)` alone keeps the excel-graph oracle from opening its
                # workbook (createSession times out; -1.1E+32 is answered), so the mean's magnitude is capped.
                draw_hi=1e12,
            ),
            _CUMULATIVE,
        ),
        value=_poisson_dist,
        domain=_poisson_domain,
        tail=_poisson_tail,
        tail_param="x",
        sources=(NEGATIVE_MEAN_SOURCE, "Abramowitz and Stegun 26.4.21"),
    )


BINOM_DIST = register(_binom_definition("BINOM.DIST"))
BINOMDIST = register(_binom_definition("BINOMDIST"))
NEGBINOM_DIST = register(_negbinom_definition("NEGBINOM.DIST", cumulative=True))
NEGBINOMDIST = register(_negbinom_definition("NEGBINOMDIST", cumulative=False))
POISSON_DIST = register(_poisson_definition("POISSON.DIST"))
POISSON = register(_poisson_definition("POISSON"))


# -- HYPGEOM.DIST and HYPGEOMDIST (T-4105) ---------------------------------------------------------------------------


def hypgeom_support(n: mpmath.mpf, k: mpmath.mpf, pop: mpmath.mpf) -> tuple[mpmath.mpf, mpmath.mpf]:
    """The possible success counts of a sample of n from a population of `pop` holding k successes."""
    return max(mpmath.mpf(0), n + k - pop), min(n, k)


def hypgeom_pmf(x: mpmath.mpf, n: mpmath.mpf, k: mpmath.mpf, pop: mpmath.mpf) -> mpmath.mpf:
    """P(X = x) = C(k, x) C(pop - k, n - x) / C(pop, n), 0 outside the support."""
    lo, hi = hypgeom_support(n, k, pop)
    if x < lo or x > hi:
        return mpmath.mpf(0)
    return mpmath.binomial(k, x) * mpmath.binomial(pop - k, n - x) / mpmath.binomial(pop, n)


def hypgeom_cdf(x: mpmath.mpf, n: mpmath.mpf, k: mpmath.mpf, pop: mpmath.mpf) -> mpmath.mpf:
    """P(X ≤ x): below the mode the lower tail summed downward from x, else 1 - P(X ≥ x + 1) summed upward, each
    away from the mode (`sum_away_from_mode`), so a deep tail keeps its digits and the sum stays short."""
    lo, hi = hypgeom_support(n, k, pop)
    if x < lo:
        return mpmath.mpf(0)
    if x >= hi:
        return mpmath.mpf(1)
    rest = pop - k - n  # pmf(j + 1) / pmf(j) = (k - j)(n - j) / ((j + 1)(rest + j + 1))
    mode = mpmath.floor((n + 1) * (k + 1) / (pop + 2))
    if x < mode:
        return sum_away_from_mode(
            hypgeom_pmf(x, n, k, pop), x, lo, -1, lambda j: j * (rest + j) / ((k - j + 1) * (n - j + 1))
        )
    up = x + 1
    return 1 - sum_away_from_mode(
        hypgeom_pmf(up, n, k, pop), up, hi, 1, lambda j: (k - j) * (n - j) / ((j + 1) * (rest + j + 1))
    )


def _hypgeom_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, n, k, pop = _reals(args)
    return hypgeom_cdf(x, n, k, pop) if _cumulative(args) else hypgeom_pmf(x, n, k, pop)


def _hypgeom_domain(args: Sequence[MathArg]) -> str | None:
    _, n, k, pop = _reals(args)
    if k > pop:
        return "population_s above number_pop"
    if n > pop:
        return "number_sample above number_pop"
    return None


HYPGEOM_SD_CAP = 1000.0
"""The largest standard deviation a cdf point is drawn with: past about 3,400 the engine's cdf sum has more than 2^18
terms and is `#NUM!` (the per-call cap DEVIATION of statistics.md "HYPGEOM.DIST: per-call cap and lifting (T-1873)"),
where Excel's value is unprobed (T-1896). A wider point is drawn as a pmf instead."""
HYPGEOM_CDF_TERMS = 2**17
"""The most terms a cdf point is drawn with, counted from the lowest possible sample_s as Excel sums them (statistics.md
"HYPGEOM.DIST is not exact for small integer arguments": the cdf is the sum of Excel's own pmf terms from the lowest).
A cap on the draws, not an Excel limit: the excel-graph oracle's calculate request fails on every retry for a workbook
holding `HYPGEOM.DIST(153800,1626691102,202463,2147483646,TRUE)` alone, while
`HYPGEOM.DIST(108560,269566,29690062,73480449,TRUE)` (108,560 terms) is answered (T-4105); what Excel times out
on is unpinned.
statistics.md "HYPGEOM.DIST of an empty population is 1, and a deep-tail pmf can be 0 (T-4105)" records it. A longer
point is drawn as a pmf instead."""


def _hypgeom_sd(n: int, k: int, pop: int) -> float:
    if pop < 2:
        return 0.0
    share = k / pop
    return math.sqrt(max(0.0, n * share * (1 - share) * (pop - n) / (pop - 1)))  # 0 at an invalid order


def _hypgeom_at(n: int, k: int, pop: int, sds: float) -> int:
    """A success count `sds` standard deviations from the mean, inside the support."""
    lo, hi = max(0, n + k - pop), min(n, k)
    x = round(n * k / pop + sds * _hypgeom_sd(n, k, pop)) if pop else 0
    return min(max(x, lo), hi)


def _hypgeom_draw(function: str, *, legacy: bool) -> Draw:
    """The strata of HYPGEOM.DIST (and HYPGEOMDIST, its pmf: `legacy` leaves out the flag). The four counts are drawn
    together, so a point is mostly valid (number_sample and population_s at most number_pop, sample_s in the support)
    and the invalid orders are the edge stratum's. A cdf point wider than `HYPGEOM_SD_CAP` or longer than
    `HYPGEOM_CDF_TERMS` is drawn as a pmf."""

    def point(x: int, n: int, k: int, pop: int, rng: random.Random) -> list[Arg]:
        short = _hypgeom_sd(n, k, pop) <= HYPGEOM_SD_CAP and x - max(0, n + k - pop) < HYPGEOM_CDF_TERMS
        cumulative = rng.random() < 0.5 and short
        return [x, n, k, pop] if legacy else [x, n, k, pop, cumulative]

    def some_count(rng: random.Random, pop: int) -> int:
        """Uniform over [0, pop] or log-uniform, so small counts in a huge population are drawn too."""
        if rng.random() < 0.5:
            return rng.randint(0, pop)
        return min(pop, round(log_uniform(rng, 1, pop + 1)) - 1)

    def near_mean(rng: random.Random, pop: int, spread: float) -> list[Arg]:
        k, n = some_count(rng, pop), some_count(rng, pop)
        return point(_hypgeom_at(n, k, pop, rng.uniform(-spread, spread)), n, k, pop, rng)

    def typical(rng: random.Random) -> list[Arg]:
        pop = rng.randint(1, 100)
        k, n = rng.randint(0, pop), rng.randint(0, pop)
        return point(rng.randint(max(0, n + k - pop), min(n, k)), n, k, pop, rng)

    def tail(rng: random.Random) -> list[Arg]:
        """Deep in one tail: 4 to 40 standard deviations out, at populations up to a million."""
        pop = round(log_uniform(rng, 20, 1e6))
        k, n = max(1, some_count(rng, pop)), max(1, some_count(rng, pop))
        return point(_hypgeom_at(n, k, pop, rng.choice((-1.0, 1.0)) * rng.uniform(4, 40)), n, k, pop, rng)

    def extreme(rng: random.Random) -> list[Arg]:
        """A population from 1E6 to the count limit, within 8 standard deviations of the mean."""
        pop = round(log_uniform(rng, 1e6, TRIALS_LIMIT)) if rng.random() < 0.8 else TRIALS_LIMIT
        return near_mean(rng, pop, 8)

    def whole(rng: random.Random) -> list[Arg]:
        """A population log-uniform over [1, the count limit], within 12 standard deviations of the mean."""
        return near_mean(rng, round(log_uniform(rng, 1, TRIALS_LIMIT)), 12)

    def edge(rng: random.Random) -> list[Arg]:
        """The bounds: empty sample, population and successes, the support's ends and one past them, every count at
        the limit and one past it, negative counts and the documented #NUM! orders (statistics.md T-1048, T-2255).
        An empty population (0, 0, 0, 0) is unprobed: the math's pmf is 1 there, and the oracle settles it."""
        pop = rng.randint(2, 60)
        k, n = rng.randint(1, pop - 1), rng.randint(1, pop - 1)
        lo, hi = max(0, n + k - pop), min(n, k)
        big = TRIALS_LIMIT
        shapes: list[tuple[int, int, int, int]] = [
            (0, 0, 0, 0),
            (0, 0, k, pop),
            (0, n, 0, pop),
            (lo, n, k, pop),
            (hi, n, k, pop),
            (hi + 1, n, k, pop),
            (max(lo - 1, 0), n, k, pop),
            (n, n, pop, pop),
            (k, pop, k, pop),
            (_hypgeom_at(n, k, big, rng.uniform(-3, 3)), n, k, big),
            (1, n, k, big + 1),
            (big + 1, n, k, pop),
            (1, big + 1, k, pop),
            (1, n, big + 1, pop),
            (lo, n, pop + 1, pop),
            (lo, pop + 1, k, pop),
            (-1, n, k, pop),
            (lo, n, -1, pop),
        ]
        return point(*rng.choice(shapes), rng)

    return by_stratum(
        {"typical": typical, "tail": tail, "edge": edge, "extreme": extreme, "log-uniform": whole},
        f"{function} draws: small populations, deep tails, populations up to the count limit, and the bounds.",
    )


HYPGEOM_LIMIT_SOURCE = (
    "statistics.md#hypgeomdist-a-sample-below-the-possible-range-is-0-a-negative-or-huge-count-is-num-t-2255"
)
HYPGEOM_EXACT_SOURCE = "statistics.md#hypgeomdist-is-not-exact-for-small-integer-arguments"
HYPGEOM_CAP_SOURCE = "statistics.md#hypgeomdist-per-call-cap-and-lifting-t-1873"
_HYPGEOM_COUNT = Interval(0, TRIALS_LIMIT, lo_open=False, hi_open=False)


def _hypgeom_definition(function: str, *, cumulative: bool) -> Definition:
    """HYPGEOM.DIST, and HYPGEOMDIST without the flag: it is HYPGEOM.DIST's pmf on every paired cell (statistics.md
    "Legacy distribution names sweep, second family (T-1048)"). The count limit, 2^31 - 1 or more is `#NUM!` in any
    count, is BINOM.DIST's trial limit (statistics.md T-2255), so the two share TRIALS_LIMIT. Above the support the
    pmf is 0 and the cdf 1, below it both are 0 (T-1048, T-2255): the math's own values, so no domain rule."""
    params = (
        Param("sample_s", "int", _HYPGEOM_COUNT, typical=(0, 100), source=HYPGEOM_LIMIT_SOURCE),
        Param("number_sample", "int", _HYPGEOM_COUNT, typical=(0, 100), source=HYPGEOM_LIMIT_SOURCE),
        Param("population_s", "int", _HYPGEOM_COUNT, typical=(0, 100), source=HYPGEOM_LIMIT_SOURCE),
        Param("number_pop", "int", _HYPGEOM_COUNT, typical=(1, 100), source=HYPGEOM_LIMIT_SOURCE),
    )
    return Definition(
        function=function,
        params=(*params, _CUMULATIVE) if cumulative else params,
        value=_hypgeom_dist,
        domain=_hypgeom_domain,
        draw=_hypgeom_draw(function, legacy=not cumulative),
        sources=(
            HYPGEOM_LIMIT_SOURCE,
            NEGBINOM_P_SOURCE,
            HYPGEOM_EXACT_SOURCE,
            HYPGEOM_CAP_SOURCE,
            "Abramowitz and Stegun 26.1.21",
        ),
    )


HYPGEOM_DIST = register(_hypgeom_definition("HYPGEOM.DIST", cumulative=True))
HYPGEOMDIST = register(_hypgeom_definition("HYPGEOMDIST", cumulative=False))
