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

from collections.abc import Callable, Sequence
from dataclasses import dataclass

import mpmath

from rustsheet_tools.accuracy.defs._common import DF_CUT, is_real
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
HUGE_X_SOURCE = "statistics.md#scalar-distributions-at-extreme-arguments-part-2-t-2256"
DENSITY_HUGE_X_SOURCE = (
    "statistics.md#tdists-density-is-num-at-a-huge-x-and-tinv2t-and-tinv-at-p--22e-308-t-3358"
)
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
    if not is_real(df) or df < 1:
        msg = "no tail map below df 1"
        raise ValueError(msg)
    x = lower_quantile(q, mpmath.floor(df))
    return -x if upper else x


def _t_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df, cumulative = args
    assert is_real(x) and is_real(df)
    _t_dist_cut(x, cumulative=bool(cumulative)).check_probed(x)
    return cdf(x, df) if cumulative else density(x, df)


def _t_dist_domain(args: Sequence[MathArg]) -> str | None:
    df, cumulative = args[1], args[2]
    if cumulative and is_real(df) and df > DF_CUT:
        return "cumulative df above 1E+10"
    x = _x_of(args)
    return _t_dist_cut(x, cumulative=bool(cumulative)).domain(x)


def _t_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, df = args
    assert is_real(p) and is_real(df)
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
        sources=(DF_SOURCE, HUGE_X_SOURCE, DENSITY_HUGE_X_SOURCE, "Abramowitz and Stegun 26.7.1"),
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
    assert is_real(p) and is_real(df)
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
# T.DIST.2T at 1.5E10 and 2E10, the tables at 1E10 + 1). At a huge |x| Excel is #NUM! ("Scalar distributions at extreme
# arguments, part 2"): `HugeXCut` holds each function's evidenced bracket.
UPPER_SOURCE = "statistics.md#tdistrt-and-tdist2t-are-not-1---tdist"
LEGACY_T_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"


@dataclass(frozen=True)
class HugeXCut:
    """Excel's #NUM! at a huge |x|, as far as cells show it: the largest |x| with a number (`last_number`) and the
    smallest with #NUM! (`first_error`). From `first_error` on, x is outside the domain; strictly between the two, the
    point is unprobed and has no value (the cut, near where x² overflows a double, is not pinned)."""

    last_number: float
    first_error: float

    def domain(self, x: mpmath.mpf) -> str | None:
        """Why x is outside the domain, or None."""
        return f"|x| at or above {self.first_error:.15g} is #NUM!" if abs(x) >= self.first_error else None

    def check_probed(self, x: mpmath.mpf) -> None:
        """NoValueError when |x| lies strictly inside the unprobed bracket."""
        if self.last_number < abs(x) < self.first_error:
            msg = f"|x| between {self.last_number:.15g} and {self.first_error:.15g}: Excel's #NUM! cut is unprobed"
            raise NoValueError(msg)


# Each bracket from every Excel cell with valid other arguments and |x| above 1E10: the Gate A tables, probes and
# gen-m9 (T-4102). Per sign for T.DIST.RT, whose negative x is valid.
RT_POSITIVE_CUT = HugeXCut(  # probes/t4456-beta-t-zero-reach F2; accuracy/T.DIST.RT/s1 A101
    last_number=1.34305338528263e60, first_error=2.22973889209998e176
)
RT_NEGATIVE_CUT = HugeXCut(  # accuracy/T.DIST.RT/s1 A116 (x -1.35E+123, value 1) and A102 (x -7.96E+165)
    last_number=1.35098712206146e123, first_error=7.96044688386386e165
)
T_DIST_2T_CUT = HugeXCut(  # accuracy/T.DIST.2T/s1 A88 (value 0) and A99
    last_number=5.76102699358893e146, first_error=4.22238650908343e161
)
TDIST_CUT = HugeXCut(  # accuracy/TDIST/s1 A185 (value 0) and A176
    last_number=1.76387937163976e150, first_error=2.08984434704072e166
)
# T.DIST's brackets, keyed by (cumulative, x ≥ 0) (T-4580, decision 0321), from every Excel cell of one T.DIST call
# with |x| above 1E10: only its three Gate A tables have one (no probe or gen-m9 cell does outside a composite). A
# cumulative cell whose df is above 1E10 is #NUM! for its df whatever x is, so it brackets nothing.
T_DIST_CUTS = {
    (True, True): HugeXCut(  # accuracy/T.DIST/e-T-3425 A92 (value 1) and r4-T-2585 A217
        last_number=7.33561600112214e146, first_error=1.89648831921934e159
    ),
    (True, False): HugeXCut(  # accuracy/T.DIST/r4-T-2585 A223 (value 0) and e-T-3425 A88
        last_number=2.01264664929796e153, first_error=5.43165733249003e196
    ),
    (False, True): HugeXCut(  # accuracy/T.DIST/s1 A97 (value 0) and r4-T-2585 A267
        last_number=9.93424108915371e137, first_error=3.83716580917453e200
    ),
    (False, False): HugeXCut(  # accuracy/T.DIST/r4-T-2585 A236 (value 0) and A266
        last_number=1.12173766596864e153, first_error=9.72379926420781e176
    ),
}


def _t_dist_cut(x: mpmath.mpf, *, cumulative: bool) -> HugeXCut:
    """T.DIST's huge-x bracket for `cumulative` and the sign of x (x = 0 is far from every cut)."""
    return T_DIST_CUTS[(cumulative, x >= 0)]


def _rt_cut(x: mpmath.mpf) -> HugeXCut:
    return RT_POSITIVE_CUT if x > 0 else RT_NEGATIVE_CUT


def _x_of(args: Sequence[MathArg]) -> mpmath.mpf:
    x = args[0]
    assert is_real(x)
    return x


def upper_tail(x: mpmath.mpf, df: mpmath.mpf) -> mpmath.mpf:
    """P(T > x) = P(T ≤ -x) by symmetry: the far tail of either sign keeps its digits."""
    return cdf(-x, df)


def _upper_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    """The x ≥ 0 whose upper tail is q (a two-tailed function's x is never negative, so `upper` is ignored)."""
    del upper
    return _tail_x(q, True, args)


def _t_dist_rt(args: Sequence[MathArg]) -> mpmath.mpf:
    x, df = args
    assert is_real(x) and is_real(df)
    _rt_cut(x).check_probed(x)
    return upper_tail(x, df)


def _t_dist_rt_domain(args: Sequence[MathArg]) -> str | None:
    x = _x_of(args)
    return _rt_cut(x).domain(x)


def _tails_upper(
    cut: HugeXCut,
) -> tuple[Callable[[Sequence[MathArg]], mpmath.mpf], Callable[[Sequence[MathArg]], str | None]]:
    """The value and domain of T.DIST.2T(x, df) and TDIST(x, df, tails): `tails` (2 for T.DIST.2T) times the upper
    tail at x ≥ 0, within the function's huge-x `cut`."""

    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        x, df = args[0], args[1]
        tails = args[2] if len(args) > 2 else mpmath.mpf(2)
        assert is_real(x) and is_real(df)
        assert is_real(tails)
        cut.check_probed(x)
        return tails * upper_tail(x, df)

    def domain(args: Sequence[MathArg]) -> str | None:
        return cut.domain(_x_of(args))

    return value, domain


def _tail_df(truncates: str) -> Param:
    return bounded_df("deg_freedom", (1, 30), DF_SOURCE, truncates=truncates)


T_DIST_RT = register(
    Definition(
        function="T.DIST.RT",
        # a negative x is valid: `T.DIST.RT(-2,1)` = 0.8524163823495667 (statistics.md T-1043)
        params=(_X, _tail_df(DF_SOURCE)),
        value=_t_dist_rt,
        domain=_t_dist_rt_domain,
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

_T_DIST_2T_VALUE, _T_DIST_2T_DOMAIN = _tails_upper(T_DIST_2T_CUT)
T_DIST_2T = register(
    Definition(
        function="T.DIST.2T",
        params=(_X_NONNEGATIVE, _tail_df(DF_SOURCE)),
        value=_T_DIST_2T_VALUE,
        domain=_T_DIST_2T_DOMAIN,
        tail=_upper_x,
        tail_param="x",
        sources=(UPPER_SOURCE, DF_SOURCE, LEGACY_T_SOURCE, HUGE_X_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)

# TDIST(x, df, 2) = T.DIST.2T(x, df), and TDIST(x, df, 1) = T.DIST.RT(x, df) at x ≥ 0, where TDIST alone rejects a
# negative x (statistics.md T-1048). df truncates (`TDIST(1,1.9,2)` = `T.DIST.2T(1,1.9)`), tails truncates
# (`TDIST(1,3,1.9)` is tails 1, `TDIST(1,3,2.1)` tails 2), and tails 0 or 3 is #NUM! (T-1048).
_TDIST_VALUE, _TDIST_DOMAIN = _tails_upper(TDIST_CUT)
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
        value=_TDIST_VALUE,
        domain=_TDIST_DOMAIN,
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
# (`CONFIDENCE.T(1E-300,1,2)` = 4.50158158078553E299; statistics.md T-1531). sd 0 is #NUM! (T-1043), but no cell shows
# a negative sd, so a negative sd has no value (NoValueError, unprobed) rather than an assumed error.
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
    assert is_real(alpha) and is_real(sd)
    assert is_real(n)
    if sd < 0:
        msg = "a negative standard_dev is unprobed"
        raise NoValueError(msg)
    return confidence_margin(alpha, sd, n)


def _confidence_t_domain(args: Sequence[MathArg]) -> str | None:
    return "standard_dev 0 is #NUM!" if args[1] == 0 else None


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
                Interval(),  # sd 0 is a `domain` rule and a negative sd has no value: neither bound is evidenced
                typical=(0.1, 100),
                extreme=(1e-100, 1e100),
                edges=(0.0,),
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
        domain=_confidence_t_domain,
        sources=(DF_SOURCE, TINY_P_SOURCE, "Abramowitz and Stegun 26.7.1"),
    )
)
