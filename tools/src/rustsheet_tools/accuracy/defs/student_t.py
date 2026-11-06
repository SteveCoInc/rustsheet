"""Student t: T.DIST and T.INV (the definitions of SPEC-0029's first distribution and inverse).

Sources:
- statistics.md, "Student t sweep: df truncation, df limits, T.INV.2T above 1 (T-1043)": df < 1 is
  an error; the cumulative T.DIST and T.INV are `#NUM!` above df = 1E10 (the cut is on the
  truncated df), while the density stays finite above it; p outside (0, 1) is `#NUM!`.
- statistics.md, "Student-t quantile at a tiny probability (T-1531)": T.INV is finite for p down to
  1E-300 (the power-law tail), so p's extreme range reaches 1E-300.
- The math: the lower tail is I_y(df/2, 1/2)/2 with y = df/(df + x²) (Abramowitz and Stegun
  26.7.1, the regularized incomplete beta), the density Γ((df+1)/2)/(√(df·π) Γ(df/2)) ·
  (1 + x²/df)^(-(df+1)/2) (A&S 26.7.1); the quantile solves the lower tail.
"""

from __future__ import annotations

from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import DF_CUT
from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_log_bracket,
)
from rustsheet_tools.fitlab.kernels.beta import betacf

DF_SOURCE = "statistics.md#student-t-sweep-df-truncation-df-limits-tinv2t-above-1-t-1043"
TINY_P_SOURCE = "statistics.md#student-t-quantile-at-a-tiny-probability-t-1531"
# The continued fraction converges for y < (a + 1)/(a + b + 2), which holds for x² ≥ 3 at every df; from |x| = 2 it
# takes at most about 600 terms (df 1E10), fewer as |x| grows.
CF_MIN_X = 2
CF_MAX_TERMS = 100_000  # directive 5: a bound far above the ~600 terms the fraction needs
CF_GUARD_DPS = 20


def lower_tail(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """P(T ≤ x) for x ≤ 0: I_y(df/2, 1/2)/2, y = df/(df + x²) (T-3425 for both branches):
    - |x| < `CF_MIN_X`: 1/2 - I_z(1/2, df/2)/2 with z = 1 - y = x²/(df + x²), which keeps x² that y = df/(df + x²)
      rounds away (at df 1E10 and x = -e^-60, y is 1 at 60 digits); the series has about x²/2 terms and the
      subtraction from 1 loses at most two digits (the tail is at least 0.0227);
    - from |x| = `CF_MIN_X` on: the continued fraction for I_y itself, exact in the far tail since no complement is
      taken. mpmath's hypergeometric series stalls there for a df above about 5E6 (from |x| ≈ 90 up to about √df,
      tails below e^-4000 that T.INV's root search still evaluates)."""
    if x == 0:
        return mpmath.mpf(1) / 2
    a, b = df / 2, mpmath.mpf(1) / 2
    with mpmath.extradps(CF_GUARD_DPS):  # ln B(a, b) cancels about log10(df) digits of loggamma(a)
        x2 = x * x
        if -x < CF_MIN_X:
            tail = (1 - mpmath.betainc(b, a, 0, x2 / (df + x2), regularized=True)) / 2
        else:
            tail = _far_tail(a, b, x2, df)
    return +tail  # rounded to the caller's precision


def _far_tail(a: mpmath.mpf, b: mpmath.mpf, x2: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """I_y(a, b)/2 for y = df/(df + x²) by the continued fraction, at the current precision."""
    log_prefactor = (
        -a * mpmath.log1p(x2 / df)  # a·ln y
        + b * mpmath.log(x2 / (df + x2))  # b·ln(1 - y)
        - mpmath.log(a)
        - (mpmath.loggamma(a) + mpmath.loggamma(b) - mpmath.loggamma(a + b))
    )
    tol = mpmath.mpf(10) ** -(mpmath.mp.dps + 2)
    fraction, converged = betacf(a, b, df / (df + x2), tol, CF_MAX_TERMS)
    if not converged:
        msg = f"the t continued fraction did not converge in {CF_MAX_TERMS} terms"
        raise mpmath.libmp.NoConvergence(msg)
    return mpmath.exp(log_prefactor) * fraction / 2


def cdf(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """P(T ≤ x)."""
    return lower_tail(x, df) if x <= 0 else 1 - lower_tail(-x, df)


def density(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """The t density, through log-gamma so a huge df neither overflows nor cancels. loggamma((df+1)/2) and
    loggamma(df/2) are about df·ln(df)/2 and cancel to about ln(df)/2, so the guard digits grow with log10(df)
    (T-3425: at df 8.3E+129 the 60- and 80-digit values disagreed)."""
    with mpmath.extradps(int(mpmath.log10(df)) + CF_GUARD_DPS):
        log_value = (
            mpmath.loggamma((df + 1) / 2)
            - mpmath.loggamma(df / 2)
            - mpmath.log(df * mpmath.pi) / 2
            - (df + 1) / 2 * mpmath.log1p(x * x / df)
        )
        value = mpmath.exp(log_value)
    return +value


def lower_quantile(p: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """The x ≤ 0 with P(T ≤ x) = p, for 0 < p ≤ 1/2: solves ln P(T ≤ -e^s) = ln p in s (log
    scales keep a 1E-300 tail and a centre point equally well conditioned). NoValueError when the
    quantile is beyond the double range."""
    if p == mpmath.mpf(1) / 2:
        return mpmath.mpf(0)
    target = mpmath.log(p)

    def f(s: mpmath.mpf) -> mpmath.mpf:
        return target - mpmath.log(lower_tail(-mpmath.exp(s), df))

    # e^-60 ≈ 9E-27 is closer to 0 than any 15-digit p below 1/2 puts the quantile; e^710 is past
    # the double range, where the quantile would overflow.
    return -mpmath.exp(solve_log_bracket(f, -60, 710))


def quantile(p: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """The x with P(T ≤ x) = p (the t distribution is symmetric, so p > 1/2 is -x(1 - p), with
    1 - p exact at the working precision)."""
    return lower_quantile(p, df) if p <= mpmath.mpf(1) / 2 else -lower_quantile(1 - p, df)


def _tail_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    df = args[1]
    if isinstance(df, (bool, tuple)) or df < 1:
        msg = "no tail map below df 1"
        raise ValueError(msg)
    x = lower_quantile(q, mpmath.floor(df))
    return -x if upper else x


def _t_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df, cumulative = args
    assert not isinstance(x, bool) and not isinstance(df, bool)
    return cdf(x, df) if cumulative else density(x, df)


def _t_dist_domain(args: Sequence[MathArg]) -> str | None:
    df, cumulative = args[1], args[2]
    if cumulative and not isinstance(df, (bool, tuple)) and df > DF_CUT:
        return "cumulative df above 1E+10"
    return None


def _t_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, df = args
    assert not isinstance(p, bool) and not isinstance(df, bool)
    return quantile(p, df)


def bounded_df(
    name: str, typical: tuple[float, float], source: str, edges: tuple[float, ...] = (), truncates: str = ""
) -> Param:
    """A df parameter the function rejects above DF_CUT, so tables stay inside [1, DF_CUT]: the
    inverses here and every F function (statistics.md, "Student-t sweep" and "F-distribution df
    limit"). `truncates` cites the entry showing Excel truncates a non-integer df (`Param.truncates`)."""
    return Param(
        name,
        "int",
        Interval(1, DF_CUT, lo_open=False, hi_open=False),
        typical=typical,
        extreme=(1e3, DF_CUT),
        edges=edges,
        source=source,
        truncates=truncates,
    )


_DF = Param(
    "deg_freedom",
    "int",
    Interval(1, float("inf"), lo_open=False),
    typical=(1, 30),
    extreme=(1e3, DF_CUT),
    edges=(DF_CUT, DF_CUT + 1),
    source=DF_SOURCE,
    # `T.DIST(2,1.999,TRUE)` = `T.DIST(2,1,TRUE)`, and the pdf's df in (-1, 1) is #DIV/0! (truncated to 0)
    truncates=DF_SOURCE,
)

_X = Param(
    "x",
    "real",
    Interval(),
    typical=(-6, 6),
    extreme=(1e2, 1e300),
    edges=(0.0,),
    source="A&S 26.7",
)

T_DIST = register(
    Definition(
        function="T.DIST",
        params=(
            _X,
            _DF,
            Param("cumulative", "bool", source="catalog"),
        ),
        value=_t_dist,
        domain=_t_dist_domain,
        tail=_tail_x,
        tail_param="x",
        sources=(DF_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)

T_INV = register(
    Definition(
        function="T.INV",
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 1),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=TINY_P_SOURCE,
            ),
            bounded_df("deg_freedom", (1, 30), DF_SOURCE),
        ),
        value=_t_inv,
        sources=(DF_SOURCE, TINY_P_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)


def two_tailed_quantile(p: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """T.INV.2T(p, df) = T.INV(1 - p/2, df): the positive x with P(|T| > x) = p for p ≤ 1, and the
    negative quantile T.INV(1 - p/2) for p in (1, 2) (statistics.md, "T.INV.2T does not reject p in
    (1,2)"). p/2 and 1 - p/2 are exact at the working precision, so no tail is lost."""
    half = p / 2
    return -lower_quantile(half, df) if p <= 1 else lower_quantile(1 - half, df)


def _t_inv_2t(args: Sequence[MathArg]) -> mpmath.mpf:
    p, df = args
    assert not isinstance(p, bool) and not isinstance(df, bool)
    return two_tailed_quantile(p, df)


def _two_tailed(function: str) -> Definition:
    return Definition(
        function=function,
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 2),
                typical=(0.001, 1.999),
                extreme=(1e-300, 1e-15),
                edges=(1.0,),
                source=DF_SOURCE,
            ),
            bounded_df("deg_freedom", (1, 30), DF_SOURCE),
        ),
        value=_t_inv_2t,
        sources=(DF_SOURCE, TINY_P_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )


T_INV_2T = register(_two_tailed("T.INV.2T"))
# TINV is T.INV.2T's legacy name with the same rules (statistics.md, "T.INV.2T(p,1) and TINV(p,1)
# are t2(p) for p up to 1, and -t2(2-p) for p in (1, 2)").
TINV = register(_two_tailed("TINV"))


# -- The upper tails: T.DIST.RT, T.DIST.2T and TDIST (T-4102) -----------------------------------------------------
#
# Each is the tail `lower_tail` computes, so no complement 1 - cdf is taken (statistics.md, "T.DIST.RT and T.DIST.2T
# are not 1 - T.DIST": Excel computes the upper tail on its own and keeps full relative precision in the far tail).
# Like the cumulative T.DIST, they are #NUM! above df 1E10 (`bounded_df`; statistics.md T-1043 probes T.DIST.RT and
# T.DIST.2T at 1.5E10 and 2E10). Where x² overflows (|x| past about 1.34E+154) Excel is #NUM! while the math is a
# number ("Scalar distributions at extreme arguments, part 2"): an Excel error against a math number is held to
# Excel's code, so no domain rule claims that unprobed cut.
UPPER_SOURCE = "statistics.md#tdistrt-and-tdist2t-are-not-1---tdist"
LEGACY_T_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"
HUGE_X_SOURCE = "statistics.md#scalar-distributions-at-extreme-arguments-part-2-t-2256"


def upper_tail(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """P(T > x) = P(T ≤ -x) by symmetry: the far tail of either sign keeps its digits."""
    return cdf(-x, df)


def _upper_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    """The x ≥ 0 whose upper tail is q (a two-tailed function's x is never negative, so `upper` is ignored)."""
    del upper
    return _tail_x(q, True, args)


def _t_dist_rt(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df = args
    assert not isinstance(x, (bool, tuple)) and not isinstance(df, (bool, tuple))
    return upper_tail(x, df)


def _tails_upper(args: Sequence[MathArg]) -> mpmath.mpf:
    """T.DIST.2T(x, df) and TDIST(x, df, tails): `tails` (2 for T.DIST.2T) times the upper tail at x ≥ 0."""
    x, df = args[0], args[1]
    tails = args[2] if len(args) > 2 else mpmath.mpf(2)
    assert not isinstance(x, (bool, tuple)) and not isinstance(df, (bool, tuple))
    assert not isinstance(tails, (bool, tuple))
    return tails * lower_tail(-x, df)


def _tail_df(truncates: str) -> Param:
    return bounded_df("deg_freedom", (1, 30), DF_SOURCE, truncates=truncates)


T_DIST_RT = register(
    Definition(
        function="T.DIST.RT",
        # a negative x is valid: `T.DIST.RT(-2,1)` = 0.8524163823495667 (statistics.md T-1043)
        params=(_X, _tail_df(DF_SOURCE)),
        value=_t_dist_rt,
        tail=_tail_x,
        tail_param="x",
        sources=(UPPER_SOURCE, DF_SOURCE, HUGE_X_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)

# x ≥ 0: `T.DIST.2T(-1,3)`, `TDIST(-1,3,1)` and `TDIST(-1,3,2)` are #NUM! (statistics.md T-1043, T-1048).
_X_NONNEGATIVE = Param(
    "x",
    "real",
    Interval(0, float("inf"), lo_open=False),
    typical=(0, 6),
    extreme=(1e2, 1e300),
    source=LEGACY_T_SOURCE,
)

T_DIST_2T = register(
    Definition(
        function="T.DIST.2T",
        params=(_X_NONNEGATIVE, _tail_df(DF_SOURCE)),
        value=_tails_upper,
        tail=_upper_x,
        tail_param="x",
        sources=(UPPER_SOURCE, DF_SOURCE, LEGACY_T_SOURCE, HUGE_X_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)

# TDIST(x, df, 2) = T.DIST.2T(x, df), and TDIST(x, df, 1) = T.DIST.RT(x, df) at x ≥ 0, where TDIST alone rejects a
# negative x (statistics.md T-1048). df truncates (`TDIST(1,1.9,2)` = `T.DIST.2T(1,1.9)`), tails truncates
# (`TDIST(1,3,1.9)` is tails 1, `TDIST(1,3,2.1)` tails 2), and tails 0 or 3 is #NUM! (T-1048).
TDIST = register(
    Definition(
        function="TDIST",
        params=(
            _X_NONNEGATIVE,
            _tail_df(LEGACY_T_SOURCE),
            Param(
                "tails",
                "int",
                Interval(1, 2, lo_open=False, hi_open=False),
                typical=(1, 2),
                source=LEGACY_T_SOURCE,
                truncates=LEGACY_T_SOURCE,
            ),
        ),
        value=_tails_upper,
        tail=_upper_x,
        tail_param="x",
        sources=(UPPER_SOURCE, DF_SOURCE, LEGACY_T_SOURCE, HUGE_X_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)


# -- CONFIDENCE.T (T-4102) ----------------------------------------------------------------------------------------
#
# CONFIDENCE.T(alpha, sd, n) = T.INV.2T(alpha, n - 1)·sd/√n, with n the size truncated toward zero
# (`CONFIDENCE.T(0.05,1,10.9)` = 0.7153569059706648 is n = 10; statistics.md T-1043). n = 1 is #DIV/0!, sd 0 and alpha
# 0, 1, -0.1 or 1.5 are #NUM! (T-1043), so alpha is in (0, 1), unlike T.INV.2T's (0, 2). The df n - 1 is cut at 1E10:
# n = 10000000001 is a number and 10000000002 #NUM! (T-1043, `t1518-t-df-cut`). A tiny alpha has a finite margin
# (`CONFIDENCE.T(1E-300,1,2)` = 4.50158158078553E299; statistics.md T-1531).
CONFIDENCE_RANGE = (2.2250738585072014e-308, 1e300)  # a normal double, clear of the overflow


def confidence_margin(alpha: mpmath.mpf, sd: mpmath.mpf, n: mpmath.mpf) -> mpmath.mpf:
    """T.INV.2T(alpha, n - 1)·sd/√n. NoValueError where sd·t, sd/√n, t/√n or the margin leaves the normal double
    range: no cell shows where Excel's products overflow or underflow, so such a point is unprobed."""
    t = two_tailed_quantile(alpha, n - 1)
    root = mpmath.sqrt(n)
    for intermediate in (sd * t, sd / root, t / root, sd * t / root):
        if not CONFIDENCE_RANGE[0] <= abs(intermediate) <= CONFIDENCE_RANGE[1]:
            msg = "an intermediate leaves the normal double range (unprobed)"
            raise NoValueError(msg)
    return sd * t / root


def _confidence_t(args: Sequence[MathArg]) -> mpmath.mpf:
    alpha, sd, n = args
    assert not isinstance(alpha, (bool, tuple)) and not isinstance(sd, (bool, tuple))
    assert not isinstance(n, (bool, tuple))
    return confidence_margin(alpha, sd, n)


CONFIDENCE_T = register(
    Definition(
        function="CONFIDENCE.T",
        params=(
            Param(
                "alpha",
                "real",
                Interval(0, 1),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=DF_SOURCE,
            ),
            Param(
                "standard_dev",
                "real",
                Interval(0, float("inf")),
                typical=(0.1, 100),
                extreme=(1e-100, 1e100),
                source=DF_SOURCE,
            ),
            Param(
                "size",
                "int",
                Interval(2, DF_CUT + 1, lo_open=False, hi_open=False),
                typical=(2, 100),
                extreme=(1e3, DF_CUT + 1),
                source=DF_SOURCE,
                truncates=DF_SOURCE,
            ),
        ),
        value=_confidence_t,
        sources=(DF_SOURCE, TINY_P_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)
