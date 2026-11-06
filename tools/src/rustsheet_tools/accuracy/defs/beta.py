"""The beta distribution: BETA.DIST and BETA.INV on the standard interval [0, 1].

Sources:
- statistics.md, "BETA.DIST's density is #NUM! at an x of 0 or 1 (T-2435)": the density is `#NUM!` at x = 0 when
  alpha ≤ 1 and at x = 1 when beta ≤ 1; the cdf is unaffected.
- statistics.md, "BETA.INV solves Excel's own CDF to the crossing (T-1745)" and "BETAINV is #NUM! where its quantile
  underflows (T-2435)": BETA.INV takes p in (0, 1) and is `#NUM!` where the quantile underflows (the sampler redraws).
- The math: P(X ≤ x) = I_x(a, b) and its complement I_{1-x}(b, a) taken directly (Abramowitz and Stegun 26.5.1,
  26.5.2); the density x^(a-1) (1-x)^(b-1) / B(a, b) in log space. The quantile solves the smaller tail in log
  scale, of x for the lower tail and of 1 - x for the upper, so a 1E-300 tail is as well conditioned as the centre.

The optional [A, B] bounds are a linear map of x (`A + (B - A) x`); the s1 batches stay on [0, 1] (T-3529), so
they are call parameters (`Definition.call_params`, T-3923) that only eval cells pass:
- BETA.DIST(x, alpha, beta, cumulative, A, B) is the standard beta at t = (x - A)/(B - A), taken exact (no
  evidence says how Excel rounds t, and rounding it here would be a fit), with the density divided by B - A
  (Microsoft's BETA.DIST page). x outside [A, B] and A = B are Excel's `#NUM!` (statistics.md, "BETADIST with
  bounds", the legacy function's evidence, which BETA.DIST relies on). The density keeps its
  check of the given x against 0 and 1 (DENSITY_SOURCE: `BETA.DIST(0,0.5,2,FALSE,-1,1)` is `#NUM!`); where t is
  an end of [0, 1] at a shape of 1 or less Excel gives 0 (the same entry), which is no math value.
- BETA.INV(p, alpha, beta, A, B) is A + (B - A) q for the standard quantile q, which Excel's own scaled calls
  equal bit for bit at 40 of 40 points (statistics.md, "BETA.INV solves Excel's own CDF to the crossing
  (T-1745)", the T-3566 paragraph). B < A is `#NUM!`; A = B is unprobed, so it has no math value.
"""

from __future__ import annotations

from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.model import (
    LOG_SMALLEST_DOUBLE,
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_smaller_tail,
    spend,
)

DENSITY_SOURCE = "statistics.md#betadists-density-is-num-at-an-x-of-0-or-1-t-2435"
INV_SOURCE = "statistics.md#betainv-solves-excels-own-cdf-to-the-crossing-t-1745"
BOUNDS_SOURCE = "Microsoft's BETA.DIST and BETA.INV pages (A + (B - A) x); " + INV_SOURCE + " (T-3566)"
UNDERFLOW_SOURCE = (
    "statistics.md#betainv-is-num-where-its-quantile-underflows-but-05-at-a-tiny-alpha-and-a-high-p-t-2435"
)


# Above this a + b, I_x(a, b) is `_tail_series` alone. mpmath's `betainc` sums the alternating
# 2F1(a, 1 - b; a + 1; x), which cancels so badly at large shapes that one call takes seconds to
# minutes before it gives up (BETA.INV(1E-100,1E5,1E5), probes-numeric/t1543-inverse-cdf-tiny-p A99:
# 34 calls, 100 s at dps 60, then hypsum fails). The old 10 s wall-clock drop hid that; T-3680
# replaced it with these deterministic budgets.
MPMATH_SHAPE_CAP = 20_000
# The term budget of `_tail_series`. Its terms decay like exp(-n^2 (1 - x) / 2a) from the side's
# switch point, so dps 80 needs about sqrt(370 a / (1 - x)) terms: 1E5 reaches a shape of about 1E7.
SERIES_MAXTERMS = 10**5
# Past this shape the series is not tried at all and mpmath alone answers, as before T-3680: it
# handles the astronomic shapes of the Gate A tables (BETA.DIST/s1 A189, a shape of 7.5E15 at a tiny
# x) at once, while spending the term budget in every step of a root solve took 75 s for
# BETA.INV(0.5,2.5,5E9).
SERIES_SHAPE_CAP = 1e7
# `_series` takes 1 - I_(1-x)(b, a) only while that tail is at most 1 - this (at most 6 digits lost).
COMPLEMENT_MARGIN = mpmath.mpf("1e-6")


def _tail_series(a: mpmath.mpf, b: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """I_x(a, b) for x ≤ (a + 1) / (a + b + 2) as x^a (1-x)^b / (a B(a, b)) 2F1(a + b, 1; a + 1; x)
    (DLMF 8.17.8 with 15.8.1): every term is positive and they fall from the first, so nothing cancels."""
    log_prefix = a * mpmath.log(x) + b * mpmath.log1p(-x) - mpmath.log(a) - mpmath.log(mpmath.beta(a, b))
    # x near 1 at a huge a: the sum would need about 210 / (1 - x) terms, where mpmath's
    # transformation answers at once (Gate A BETA.INV/s1 A176,
    # BETA.INV(2.42438848002334E-218,907658.84479535,6.34809182974387)).
    if _terms(a, b, x) > SERIES_MAXTERMS:
        series = mpmath.hyp2f1(a + b, 1, a + 1, x, maxterms=SERIES_MAXTERMS)
    else:
        try:
            series = _positive_2f1(a, b, x)
        except mpmath.libmp.NoConvergence:
            series = mpmath.hyp2f1(a + b, 1, a + 1, x, maxterms=SERIES_MAXTERMS)
    return mpmath.exp(log_prefix) * series


def _positive_2f1(a: mpmath.mpf, b: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """2F1(a + b, 1; a + 1; x) summed term by term. Not `mpmath.hyp2f1`: past z of about 0.75 it
    leaves the series for a transformation whose hypercomb cancels and failed after 329 s at
    I_0.832(62750.652, 383), a bracket step of BETA.INV(0.665410001077291,383,62750.652)
    (probes-numeric/t3443-basym-sweep A150). The term ratio x (a + b + n) / (a + 1 + n) moves
    monotonically to x, so no later ratio exceeds r = max(ratio, x) and the tail is at most
    term * r / (1 - r); NoConvergence past SERIES_MAXTERMS terms, a deterministic budget (T-3680)."""
    eps = mpmath.eps * 2**-10
    total = term = mpmath.mpf(1)
    for n in range(SERIES_MAXTERMS):
        spend()
        ratio = x * (a + b + n) / (a + 1 + n)
        term *= ratio
        total += term
        r = max(ratio, x)
        if r < 1 and term * r / (1 - r) <= eps * total:
            return total
    msg = f"2F1 series: no convergence in {SERIES_MAXTERMS} terms"
    raise mpmath.libmp.NoConvergence(msg)


def _terms(a: mpmath.mpf, b: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """About how many terms `_positive_2f1(a, b, x)` sums: the rise while its ratio
    x (a + b + n) / (a + 1 + n) is above 1, then the fall at its limit x to 2^-300. Infinite where
    x rounds to 1 (the complement of an x below the working epsilon)."""
    if x >= 1:
        return mpmath.inf
    rise = max(0, (x * (a + b) - (a + 1)) / (1 - x))
    return rise + 210 / -mpmath.log(x)


def _series(a: mpmath.mpf, b: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """I_x(a, b) from `_tail_series`, directly or as the complement 1 - I_(1-x)(b, a), whichever
    sums fewer terms. Near the switch point (a + 1) / (a + b + 2) at a skewed shape the classic
    choice sums about 6E4 terms per call where the other side sums a few hundred, and a BETA.INV
    root solve then spends its whole work budget (probes-numeric/t3443-basym-sweep A150,
    BETA.INV(0.665410001077291,383,62750.652)). The direct sum has positive terms and never
    cancels; the complement is kept only while I_(1-x)(b, a) <= 1 - COMPLEMENT_MARGIN, so 1 - it
    loses at most 6 digits, well inside the 1e-45 agreement of dps 60 and 80.
    Where the chosen side runs out of terms the other is tried."""
    if x <= 0 or x >= 1:
        return mpmath.mpf(0) if x <= 0 else mpmath.mpf(1)
    if _terms(a, b, x) <= _terms(b, a, 1 - x):
        try:
            return _tail_series(a, b, x)
        except mpmath.libmp.NoConvergence:
            return 1 - _tail_series(b, a, 1 - x)
    try:
        other = _tail_series(b, a, 1 - x)
    except mpmath.libmp.NoConvergence:
        return _tail_series(a, b, x)
    return 1 - other if other <= 1 - COMPLEMENT_MARGIN else _tail_series(a, b, x)


def _betainc(a: mpmath.mpf, b: mpmath.mpf, x: mpmath.mpf) -> mpmath.mpf:
    """I_x(a, b): mpmath's `betainc` up to a + b = MPMATH_SHAPE_CAP, with `_series` where it gives up
    (its hypsum loses the cancelling series at shapes of a few thousand,
    probes-numeric/t3571-r5-bfrac-fresh A239); `_series` alone above that cap; mpmath alone again
    above a shape of SERIES_SHAPE_CAP. Both budgets are
    deterministic, so a cell gets the same value or the same skip on any machine (T-3680).
    NoValueError (the sampler redraws) where no series converges within its budget; a ValueError
    from mpmath's hypsum (the point is dropped) where it gave up and the fallback
    did too."""
    if a + b > MPMATH_SHAPE_CAP and max(a, b) <= SERIES_SHAPE_CAP:
        try:
            return _series(a, b, x)
        except mpmath.libmp.NoConvergence as err:
            msg = f"the incomplete beta did not converge ({err})"
            raise NoValueError(msg) from err
    try:
        return mpmath.betainc(a, b, 0, x, regularized=True)
    except (mpmath.libmp.NoConvergence, ValueError) as err:
        first = err
    try:
        if max(a, b) > SERIES_SHAPE_CAP:
            raise first
        return _series(a, b, x)
    except (mpmath.libmp.NoConvergence, ValueError):
        if isinstance(first, mpmath.libmp.NoConvergence):
            msg = f"the incomplete beta did not converge ({first})"
            raise NoValueError(msg) from first
        raise first from None


def lower_tail(x: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """I_x(a, b)."""
    return _betainc(a, b, x)


def upper_tail(x: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """1 - I_x(a, b) as I_{1-x}(b, a), exact at the working precision."""
    return _betainc(b, a, 1 - x)


def density(x: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """The beta density; at an end it is 0 where the exponent is positive (the ≤ 1 cases are domain errors)."""
    if x in (0, 1):
        return mpmath.mpf(0)
    log_value = (a - 1) * mpmath.log(x) + (b - 1) * mpmath.log1p(-x) - mpmath.log(mpmath.beta(a, b))
    return mpmath.exp(log_value)


def quantile(p: mpmath.mpf, a: mpmath.mpf, b: mpmath.mpf) -> mpmath.mpf:
    """The x with I_x(a, b) = p, 0 < p < 1. NoValueError when no double strictly inside (0, 1) can be recorded, and
    each bound applies on one side only. For p <= 1/2 it is x underflowing; a quantile that rounds to 1 is recorded
    as 1 there (accuracy/BETA.INV/s1 A83, A86, A98, A100, A103, A106 and the huge-alpha points). For p > 1/2 it is
    x rounding to 1; an underflowing quantile is recorded as 0 (e.g. A88, A97, A108, A115, A125), where Excel is
    2^-53 or 0.5, not `#NUM!`."""
    if p <= mpmath.mpf(1) / 2:
        return mpmath.exp(solve_smaller_tail(p, lambda x: lower_tail(x, a, b), None, LOG_SMALLEST_DOUBLE, 0))
    # 1 - x = e^s: the upper tail I_{e^s}(b, a) grows with s, so it is the lower tail of 1 - x
    s = solve_smaller_tail(1 - p, lambda y: _betainc(b, a, y), None, -1500, 0)
    x = 1 - mpmath.exp(s)
    if x >= 1 - mpmath.mpf(2) ** -54:
        msg = "the quantile rounds to 1"
        raise NoValueError(msg)
    return x


def _bounds(given: Sequence[MathArg]) -> tuple[mpmath.mpf, mpmath.mpf]:
    """The [A, B] a call gives (A and B default to 0 and 1, as in Excel)."""
    lo, hi = (list(given) + [mpmath.mpf(0), mpmath.mpf(1)][len(given) :])[:2]
    assert not isinstance(lo, (bool, str, tuple)) and not isinstance(hi, (bool, str, tuple))
    return mpmath.mpf(lo), mpmath.mpf(hi)


def _beta_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, a, b, cumulative = args[:4]
    assert not isinstance(x, (bool, str, tuple))
    assert not isinstance(a, (bool, str, tuple))
    assert not isinstance(b, (bool, str, tuple))
    lo, hi = _bounds(args[4:])
    t = (x - lo) / (hi - lo)
    if cumulative:
        return lower_tail(t, a, b)
    if len(args) > 4 and ((t == 0 and a <= 1) or (t == 1 and b <= 1)):
        msg = "Excel's 0 at an end of [A, B] with a shape of 1 or less"
        raise NoValueError(msg)
    return density(t, a, b) / (hi - lo)


def _beta_dist_domain(args: Sequence[MathArg]) -> str | None:
    x, a, b, cumulative = args[:4]
    if len(args) > 4 and not isinstance(x, (bool, str, tuple)):
        lo, hi = _bounds(args[4:])
        if lo == hi or not lo <= x <= hi:
            return "x outside [A, B], or A = B"
        # A given x of 0 or 1 that is also an end of [A, B]: whether the given-x #NUM! or the end's 0 wins is
        # unprobed, so `_beta_dist` gives no math value there.
        if not cumulative and x in (0, 1) and x in (lo, hi):
            return None
    if (
        cumulative
        or isinstance(x, (bool, str, tuple))
        or isinstance(a, (bool, str, tuple))
        or isinstance(b, (bool, str, tuple))
    ):
        return None
    if x == 0 and a <= 1:
        return "density at x = 0 with alpha <= 1"
    if x == 1 and b <= 1:
        return "density at x = 1 with beta <= 1"
    return None


def _tail_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    a, b = args[1], args[2]
    if isinstance(a, (bool, str, tuple)) or isinstance(b, (bool, str, tuple)):
        msg = "no tail map for a logical shape"
        raise ValueError(msg)
    return quantile(1 - q if upper else q, a, b)


def _beta_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, a, b = args[:3]
    assert (
        not isinstance(p, (bool, str, tuple))
        and not isinstance(a, (bool, str, tuple))
        and not isinstance(b, (bool, str, tuple))
    )
    lo, hi = _bounds(args[3:])
    if lo == hi:
        msg = "A = B: no evidence of Excel's answer"
        raise NoValueError(msg)
    return lo + (hi - lo) * quantile(p, a, b)


def _beta_inv_domain(args: Sequence[MathArg]) -> str | None:
    lo, hi = _bounds(args[3:])
    return "B < A" if hi < lo else None


def _bound(name: str) -> Param:
    return Param(name, "real", Interval(), source=BOUNDS_SOURCE, optional=True)


_BOUNDS = (_bound("A"), _bound("B"))


def _shape(name: str) -> Param:
    return Param(
        name,
        "real",
        Interval(0, float("inf")),
        typical=(0.1, 20),
        extreme=(1e-3, 1e6),
        edges=(1.0,),
        source="Abramowitz and Stegun 26.5",
    )


BETA_DIST = register(
    Definition(
        function="BETA.DIST",
        params=(
            Param(
                "x",
                "real",
                Interval(0, 1, lo_open=False, hi_open=False),
                typical=(0.01, 0.99),
                extreme=(1e-300, 1e-15),
                source=DENSITY_SOURCE,
            ),
            _shape("alpha"),
            _shape("beta"),
            Param("cumulative", "bool", source="catalog"),
        ),
        value=_beta_dist,
        domain=_beta_dist_domain,
        tail=_tail_x,
        tail_param="x",
        sources=(DENSITY_SOURCE, BOUNDS_SOURCE, "Abramowitz and Stegun 26.5.1, 26.5.2"),
        call_params=_BOUNDS,
        rescaled_param="x",
    )
)

BETA_INV = register(
    Definition(
        function="BETA.INV",
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 1),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=INV_SOURCE,
            ),
            _shape("alpha"),
            _shape("beta"),
        ),
        value=_beta_inv,
        domain=_beta_inv_domain,
        sources=(INV_SOURCE, UNDERFLOW_SOURCE, BOUNDS_SOURCE, "Abramowitz and Stegun 26.5.1"),
        call_params=_BOUNDS,
    )
)
