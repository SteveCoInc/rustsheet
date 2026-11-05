"""The gamma and chi-squared quantiles: GAMMA.INV, CHISQ.INV and CHISQ.INV.RT.

Sources:
- statistics.md, "GAMMA.INV and CHISQ.INV at a huge shape (T-2982, T-3070)": CHISQ.INV and the cdfs are `#NUM!` above
  1E10 degrees of freedom, while CHISQ.INV.RT takes any df (evidenced to about 1E16); the shape is df/2 and the scale 2.
- statistics.md, "Chi-squared quantile at df 2 and gamma quantile at alpha 1 are closed forms (T-2860)":
  `GAMMA.INV(1E-300,1,1)` = 1E-300 shows p reaches 1E-300.
- The math: P(X ≤ x) = P(a, x/θ), the regularized lower incomplete gamma, and its complement Q(a, x/θ) taken
  directly (Abramowitz and Stegun 6.5.1, 26.4.19 for the chi-squared law); the quantile solves the smaller tail in log
  scales, so a 1E-300 tail is as well conditioned as the centre.
"""

from __future__ import annotations

import functools
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import DF_CUT
from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_smaller_tail,
    spend,
)

HUGE_SOURCE = "statistics.md#gammainv-and-chisqinv-at-a-huge-shape-the-true-quantile-num-or-a-wilson-hilferty-closed-form-t-2982-t-3070"
CLOSED_SOURCE = (
    "statistics.md#chi-squared-quantile-at-df-2-and-gamma-quantile-at-alpha-1-are-closed-forms-t-2860"
)
CHI_SCALE = mpmath.mpf(2)
# The largest df drawn (CHISQ.INV.RT; CHISQ.INV stops at its 1E10): "any" df is evidenced to about 1E16 (HUGE_SOURCE).
# The s1 batches were drawn up to df 2E7 (shape 1E7), the series' reach before Temme's expansion (T-3863).
MAX_DRAWN_DF = 1e16
MAX_DRAWN_SHAPE = (
    1e300  # GAMMA.INV's alpha: Excel's closed form is evidenced to 1E300 (HUGE_SOURCE, t2982 A7-A23)
)


MAX_TERMS = 10**7  # far above the about 20 sqrt(a) terms a 1E7 shape needs at 50 digits
# The largest shape the series and continued fraction are summed at (about 20 sqrt(a) terms each, so a 1E9 shape would
# take minutes per tail evaluation); above it the tails are Temme's uniform asymptotic expansion (T-3863). It is the
# s1 batches' largest drawn shape, so their committed math keeps the expansions it was computed with.
TEMME_SHAPE_MIN = mpmath.mpf(10) ** 7
# Temme's coefficients c_k(eta) are power series in eta, summed to TEMME_TERMS terms: their radius is 2 sqrt(pi)
# (about 3.5), so at |eta| <= TEMME_ETA_MAX the terms fall by about 7 each. Past it there is no value: from a shape of
# 1E7 a tail there is below e^(-1E6), and the quantile's bracket stays inside it.
TEMME_ETA_MAX = mpmath.mpf("0.5")
TEMME_TERMS = 80
TEMME_ORDERS = 20  # c_0 .. c_19: a^-19 is below 1E-133 from a shape of 1E7
TEMME_GUARD_DPS = 30  # digits added while summing, besides those a huge shape needs (`_temme`)
# The quantile's bracket at a Temme shape, as x/a: |eta| is 0.47 at 0.6 and 0.44 at 1.5.
TEMME_BRACKET = (mpmath.mpf("0.6"), mpmath.mpf("1.5"))


def _log_prefix(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """ln(y^a e^-y / Γ(a)), the factor both expansions share."""
    return a * mpmath.log(y) - y - mpmath.loggamma(a)


def _eps() -> mpmath.mpf:
    """The convergence tolerance of the series and the continued fraction: a few ulps, since delta - 1 is rounding
    noise at the last bit."""
    return mpmath.mpf(2) ** (-mpmath.mp.prec + 4)


def _series(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """P(a, y) by its power series, Σ y^n / (a (a+1) ... (a+n)) (A&S 6.5.29), for y < a + 1."""
    eps = _eps()
    term = total = 1 / a
    n = a
    for _ in range(MAX_TERMS):
        spend()
        n += 1
        term *= y / n
        total += term
        if abs(term) < abs(total) * eps:
            return total * mpmath.exp(_log_prefix(y, a))
    msg = "the incomplete gamma series did not converge"
    raise NoValueError(msg)


def _continued_fraction(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """Q(a, y) by its continued fraction (A&S 6.5.31, modified Lentz), for y ≥ a + 1."""
    eps = _eps()
    tiny = mpmath.mpf(10) ** -(mpmath.mp.dps * 4)
    b = y + 1 - a
    c = 1 / tiny
    d = 1 / b
    h = d
    for i in range(1, MAX_TERMS):
        spend()
        an = -i * (i - a)
        b += 2
        d = an * d + b
        d = tiny if d == 0 else d
        c = b + an / c
        c = tiny if c == 0 else c
        d = 1 / d
        delta = d * c
        h *= delta
        if abs(delta - 1) < eps:
            return h * mpmath.exp(_log_prefix(y, a))
    msg = "the incomplete gamma continued fraction did not converge"
    raise NoValueError(msg)


def _series_mul(a: list[mpmath.mpf], b: list[mpmath.mpf], n: int) -> list[mpmath.mpf]:
    """The first `n` coefficients of the power series a times b."""
    out = [mpmath.mpf(0)] * n
    for i, ai in enumerate(a[:n]):
        for j in range(min(len(b), n - i)):
            out[i + j] += ai * b[j]
    return out


def _series_recip(a: list[mpmath.mpf], n: int) -> list[mpmath.mpf]:
    """The first `n` coefficients of 1/a, a[0] ≠ 0."""
    out = [mpmath.mpf(0)] * n
    out[0] = 1 / a[0]
    for k in range(1, n):
        out[k] = -mpmath.fsum(a[j] * out[k - j] for j in range(1, min(k, len(a) - 1) + 1)) / a[0]
    return out


def _series_sqrt(a: list[mpmath.mpf], n: int) -> list[mpmath.mpf]:
    """The first `n` coefficients of sqrt(a), a[0] > 0."""
    out = [mpmath.mpf(0)] * n
    out[0] = mpmath.sqrt(a[0])
    for k in range(1, n):
        rest = mpmath.fsum(out[j] * out[k - j] for j in range(1, k))
        out[k] = ((a[k] if k < len(a) else 0) - rest) / (2 * out[0])
    return out


@functools.lru_cache(maxsize=4)
def temme_coefficients(dps: int) -> tuple[tuple[mpmath.mpf, ...], ...]:
    """The power series in eta of Temme's c_0 .. c_(TEMME_ORDERS-1), TEMME_TERMS coefficients each, at `dps` digits.

    With mu = lambda - 1 and eta^2 / 2 = mu - ln(1 + mu) (eta of mu's sign), mu(eta) is the series reversion of
    eta = mu w(mu), w = sqrt(2 (mu - ln(1 + mu))) / mu, by Lagrange's formula [eta^q] mu = [mu^(q-1)] w^-q / q. Then
    c_0 = 1/mu - 1/eta and c_k = c_(k-1)' / eta + kappa_k / mu, Temme's recursion (Temme 1979; DLMF 8.12.10, whose
    constant is the Stirling coefficient that keeps c_k analytic at eta = 0): kappa_k is taken here as the value that
    cancels the 1/eta pole, so no table of constants is copied. c_0(0) = -1/3, c_1(0) = -1/540, c_2(0) = 25/6048 (DLMF 8.12.11)
    are checked by the tests."""
    orders, terms = TEMME_ORDERS, TEMME_TERMS
    with mpmath.workdps(dps):
        m = terms + 2 * orders + 2  # each order uses two more coefficients of the one before
        w = _series_sqrt([mpmath.mpf(2) * (-1) ** j / (j + 2) for j in range(m + 1)], m + 1)
        w_recip = _series_recip(w, m + 1)
        mu = [mpmath.mpf(0)] * (m + 1)
        power = [mpmath.mpf(1)] + [mpmath.mpf(0)] * m
        for q in range(1, m + 1):
            spend()
            power = _series_mul(power, w_recip, m + 1)
            mu[q] = power[q - 1] / q
        v = _series_recip(mu[1:], m)  # eta / mu, so 1/mu = v / eta
        c = [v[i + 1] for i in range(m - 1)]
        out = [tuple(c[:terms])]
        for _ in range(1, orders):
            d = [(i + 1) * c[i + 1] for i in range(len(c) - 1)]
            kappa = -d[0]  # the 1/eta terms of c' / eta and kappa / (mu eta) cancel
            c = [d[i + 1] + kappa * v[i + 1] for i in range(len(d) - 1)]
            out.append(tuple(c[:terms]))
        return tuple(out)


def _temme(y: mpmath.mpf, a: mpmath.mpf) -> tuple[mpmath.mpf, mpmath.mpf]:
    """(P(a, y), Q(a, y)) by Temme's uniform asymptotic expansion (DLMF 8.12.3 and 8.12.8): Q = erfc(eta sqrt(a/2)) / 2
    + R and P = erfc(-eta sqrt(a/2)) / 2 - R, R = e^(-a eta^2 / 2) / sqrt(2 pi a) sum c_k(eta) a^-k. NoValueError past
    TEMME_ETA_MAX or where the sum in a^-k does not settle."""
    dps = mpmath.mp.dps
    coefficients = temme_coefficients(dps + TEMME_GUARD_DPS + 10)
    # The exponent a eta^2 / 2 is needed to dps digits after the point, and eta from mu ~ 1/sqrt(a) cancels about
    # log10(a) / 2 digits: a shape of 1E300 is summed at about 380 digits.
    with mpmath.workdps(dps + TEMME_GUARD_DPS + int(mpmath.log10(a))):
        mu = y / a - 1
        eta = mpmath.sign(mu) * mpmath.sqrt(2 * (mu - mpmath.log1p(mu)))
        if abs(eta) > TEMME_ETA_MAX:
            msg = f"y/a = {mpmath.nstr(y / a, 3)} is past the expansion's eta of {TEMME_ETA_MAX} (T-3863)"
            raise NoValueError(msg)
        eps = mpmath.mpf(10) ** -(dps + 10)
        total = mpmath.mpf(0)
        a_power = mpmath.mpf(1)
        for c in coefficients:
            spend()
            term = mpmath.polyval(list(reversed(c)), eta) * a_power
            total += term
            if abs(term) <= eps * abs(total):
                break
            a_power /= a
        else:
            msg = f"Temme's expansion did not settle at shape {mpmath.nstr(a, 3)}"
            raise NoValueError(msg)
        r = mpmath.exp(-a * eta**2 / 2) / mpmath.sqrt(2 * mpmath.pi * a) * total
        half = eta * mpmath.sqrt(a / 2)
        lower = mpmath.erfc(-half) / 2 - r
        upper = mpmath.erfc(half) / 2 + r
    return +lower, +upper


def lower_tail(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """P(a, y), y ≥ 0 (unit scale). mpmath's `gammainc` does not converge from a shape of about 1E5, so the classic
    series and continued fraction are summed here at the working precision up to TEMME_SHAPE_MIN; each side's
    complement is taken only where the other tail is below about 1/2, so no digits cancel. Above it, Temme's expansion."""
    if y == 0:
        return mpmath.mpf(0)
    if a > TEMME_SHAPE_MIN:
        return _temme(y, a)[0]
    return _series(y, a) if y < a + 1 else 1 - _continued_fraction(y, a)


def upper_tail(y: mpmath.mpf, a: mpmath.mpf) -> mpmath.mpf:
    """Q(a, y), y ≥ 0, from the continued fraction past a + 1 (no 1 - P cancellation in the upper tail), or Temme's
    expansion above TEMME_SHAPE_MIN."""
    if y == 0:
        return mpmath.mpf(1)
    if a > TEMME_SHAPE_MIN:
        return _temme(y, a)[1]
    return 1 - _series(y, a) if y < a + 1 else _continued_fraction(y, a)


def quantile(p: mpmath.mpf, a: mpmath.mpf, upper: bool) -> mpmath.mpf:
    """The unit-scale y with P(a, y) = p (or Q(a, y) = p when `upper`), 0 < p < 1: solves the smaller tail
    ln tail(e^s) = ln q in s. NoValueError when the root is beyond the double range. Unlike BETA.INV, a root below
    the smallest double keeps its value: Excel gives 0 there (accuracy/GAMMA.INV/s1 A82, A85, A145, A155 and A172;
    CHISQ.INV(1E-300,1)), which the Gate A envelope's absolute floor of 1E-300 already accepts, and s1 is immutable
    (SPEC-0029), so the bracket stays at e^-1500 (T-3578). Above TEMME_SHAPE_MIN the bracket is TEMME_BRACKET times
    a, where every tail at least 1E-300 has its root (T-3863)."""
    lo, hi = -1500.0, 710.0
    if a > TEMME_SHAPE_MIN:
        lo, hi = (float(mpmath.log(a * bound)) for bound in TEMME_BRACKET)
    s = solve_smaller_tail(p, lambda y: lower_tail(y, a), lambda y: upper_tail(y, a), lo, hi, upper=upper)
    return mpmath.exp(s)


def _gamma_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, alpha, beta = args
    assert (
        not isinstance(p, (bool, tuple))
        and not isinstance(alpha, (bool, tuple))
        and not isinstance(beta, (bool, tuple))
    )
    # The quantile of probability 0 is the support's lower end.
    return mpmath.mpf(0) if p == 0 else beta * quantile(p, alpha, upper=False)


def _chisq(upper: bool) -> Callable[[Sequence[MathArg]], mpmath.mpf]:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        p, df = args
        assert not isinstance(p, (bool, tuple)) and not isinstance(df, (bool, tuple))
        if p == (1 if upper else 0):
            return mpmath.mpf(0)
        return CHI_SCALE * quantile(p, mpmath.floor(df) / 2, upper=upper)

    return value


def _probability(source: str, lo_open: bool, hi_open: bool) -> Param:
    return Param(
        "probability",
        "real",
        Interval(0, 1, lo_open=lo_open, hi_open=hi_open),
        typical=(0.001, 0.999),
        extreme=(1e-300, 1e-15),
        source=source,
    )


def _df(hi: float, truncates: str = "") -> Param:
    """df from 1 to `hi` (CHISQ.INV's 1E10, CHISQ.INV.RT's none); the extreme draws reach `hi` or MAX_DRAWN_DF.
    `truncates` cites the entry showing Excel truncates a non-integer df (`Param.truncates`)."""
    return Param(
        "deg_freedom",
        "int",
        Interval(1, hi, lo_open=False, hi_open=False),
        typical=(1, 60),
        extreme=(1e3, min(hi, MAX_DRAWN_DF)),
        edges=(2.0,),
        source=HUGE_SOURCE,
        truncates=truncates,
    )


GAMMA_INV = register(
    Definition(
        function="GAMMA.INV",
        params=(
            _probability(CLOSED_SOURCE, lo_open=False, hi_open=True),
            Param(
                "alpha",
                "real",
                Interval(0, float("inf")),
                typical=(0.1, 50),
                extreme=(1e-3, MAX_DRAWN_SHAPE),
                edges=(1.0,),
                source=HUGE_SOURCE,
            ),
            Param(
                "beta",
                "real",
                Interval(0, float("inf")),
                typical=(0.1, 10),
                extreme=(1e-100, 1e100),
                source="Abramowitz and Stegun 6.5.1",
            ),
        ),
        value=_gamma_inv,
        sources=(HUGE_SOURCE, CLOSED_SOURCE, "Abramowitz and Stegun 6.5.1"),
    )
)

CHISQ_INV = register(
    Definition(
        function="CHISQ.INV",
        params=(_probability(CLOSED_SOURCE, lo_open=False, hi_open=True), _df(DF_CUT)),
        value=_chisq(upper=False),
        sources=(HUGE_SOURCE, CLOSED_SOURCE, "Abramowitz and Stegun 26.4.19"),
    )
)

# `CHISQ.INV.RT(1E-9,1.5)` equals the df-1 value (df truncation; probes t3336-chisq-df1-right-tail).
RT_TRUNC_SOURCE = "statistics.md#inverse-cdf-functions-at-a-tiny-or-near-1-probability-t-1543"

# CHISQ.INV.RT takes any df (statistics.md, the huge-shape entry). The s1 batches of all three stop at a shape of
# 1E7 (df 2E7); e-T-3863 draws the huge shapes, with Temme's expansion.
CHISQ_INV_RT = register(
    Definition(
        function="CHISQ.INV.RT",
        params=(_probability(CLOSED_SOURCE, lo_open=True, hi_open=False), _df(float("inf"), RT_TRUNC_SOURCE)),
        value=_chisq(upper=True),
        sources=(HUGE_SOURCE, CLOSED_SOURCE, "Abramowitz and Stegun 26.4.19"),
    )
)
