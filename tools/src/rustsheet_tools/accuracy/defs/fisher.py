"""The F distribution: F.DIST, F.INV, their right tails F.DIST.RT and F.INV.RT, and the legacy FDIST and FINV.

Sources:
- statistics.md, "F-distribution df limit (T-1530)": either df above 1E10 (after truncation) makes
  every F function `#NUM!`, the density too; df is truncated like Student-t's.
- statistics.md, "F.INV is a crossing of F.DIST's tails (T-0974)": F.INV solves the smaller tail;
  `F.INV(1E-300,5,5)` = 5.081599884913164E-121 shows p reaches 1E-300.
- statistics.md, "Legacy distribution names sweep (T-1047)": FDIST = F.DIST.RT and FINV = F.INV.RT on every paired
  cell, so each legacy name is its modern name's Definition under another function name; df truncates
  (`FDIST(1,2.9,5)` = `FDIST(1,2,5)`); `FINV(1,..)` is 0 and `FINV(0,..)` `#NUM!`.
- statistics.md, "Thin distribution functions: argument coercion, truncation and error edges (T-1049)":
  `F.DIST.RT(0,...)` is 1, the right-tail inverses take p = 1 (to 0) and reject p = 0; `F.INV.RT(0.5,3.9,7.9)` and
  `FINV(0.5,2.9,5.9)` equal Excel's integer-df cells bit for bit (t1049-finvrt A22 = A8, m9-legacy-finv A69 = A35).
- statistics.md, "F's incomplete-beta arguments keep r >= 1 (T-3567)": F.DIST.RT and FDIST truncate df.
- statistics.md, "The F cdf is #NUM! where df2/(df1 x) overflows (T-2435)" and "Overflowing arguments, PERMUT's
  limit and argument error order (T-1290)": the right tails are `#NUM!` where fl(df2/fl(df1·x)) overflows
  and `#DIV/0!` where fl(df1·x) does.
- The math: P(F ≤ x) = I_z(d1/2, d2/2) with z = d1·x/(d1·x + d2), and its complement
  I_{1-z}(d2/2, d1/2) taken directly (Abramowitz and Stegun 26.6.2); the density
  d1^(d1/2) d2^(d2/2) x^(d1/2-1) / (B(d1/2, d2/2) (d1·x + d2)^((d1+d2)/2)) (A&S 26.6.1).
"""

from __future__ import annotations

import math
import sys
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import DF_CUT, scalars
from rustsheet_tools.accuracy.defs.student_t import bounded_df
from rustsheet_tools.accuracy.model import (
    Definition,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
    solve_smaller_tail,
)

DF_SOURCE = "statistics.md#f-distribution-df-limit-t-1530"
INV_SOURCE = "statistics.md#finv-is-a-crossing-of-fdists-tails-t-0974"
# T-2586 round 5: "d1 and d2 truncate toward zero" (1,606 of 1,606 rule cells), and `F.DIST(2,150.7,3,FALSE)` is the
# df-150 density ("F density at large degrees of freedom (T-1864)").
TRUNC_SOURCE = "statistics.md#f-density-is-not-the-beta-density-rs-df-or-a-gammaln-formula-t-2513"


def lower_tail(x: mpmath.mpf, d1: mpmath.mpf, d2: mpmath.mpf) -> mpmath.mpf:
    """P(F ≤ x), x ≥ 0."""
    if x == 0:
        return mpmath.mpf(0)
    return mpmath.betainc(d1 / 2, d2 / 2, 0, d1 * x / (d1 * x + d2), regularized=True)


def upper_tail(x: mpmath.mpf, d1: mpmath.mpf, d2: mpmath.mpf) -> mpmath.mpf:
    """P(F > x), x ≥ 0, from the complement's own incomplete beta (no 1 - P cancellation)."""
    if x == 0:
        return mpmath.mpf(1)
    return mpmath.betainc(d2 / 2, d1 / 2, 0, d2 / (d1 * x + d2), regularized=True)


def density(x: mpmath.mpf, d1: mpmath.mpf, d2: mpmath.mpf) -> mpmath.mpf:
    """The F density in log space; at x = 0 it is infinite for d1 = 1 (no single value), 1 for
    d1 = 2 and 0 above."""
    if x == 0:
        if d1 == 1:
            msg = "the density is infinite at x = 0 for d1 = 1"
            raise NoValueError(msg)
        return mpmath.mpf(1) if d1 == 2 else mpmath.mpf(0)
    log_value = (
        d1 / 2 * mpmath.log(d1)
        + d2 / 2 * mpmath.log(d2)
        + (d1 / 2 - 1) * mpmath.log(x)
        - (d1 + d2) / 2 * mpmath.log(d1 * x + d2)
        - mpmath.log(mpmath.beta(d1 / 2, d2 / 2))
    )
    return mpmath.exp(log_value)


def quantile(p: mpmath.mpf, d1: mpmath.mpf, d2: mpmath.mpf, *, upper: bool = False) -> mpmath.mpf:
    """The x with P(F ≤ x) = p, or with `upper` P(F > x) = p, 0 < p < 1: the smaller tail is solved in log scales
    (`solve_smaller_tail`) without forming 1 - p, so `F.INV.RT(1E-16,9,3)` keeps the digits 1 - p would lose
    ("F.INV is a crossing of F.DIST's tails"). NoValueError when the root is beyond the double range."""
    s = solve_smaller_tail(
        p, lambda x: lower_tail(x, d1, d2), lambda x: upper_tail(x, d1, d2), -1500, 710, upper=upper
    )
    return mpmath.exp(s)


def _df(name: str, truncates: str = "") -> Param:
    return bounded_df(name, (1, 60), DF_SOURCE, edges=(DF_CUT,), truncates=truncates)


def _f_dist(args: Sequence[MathArg]) -> mpmath.mpf:
    x, d1, d2, cumulative = args
    assert not isinstance(x, bool) and not isinstance(d1, bool) and not isinstance(d2, bool)
    return lower_tail(x, d1, d2) if cumulative else density(x, d1, d2)


def _tail_x(q: mpmath.mpf, upper: bool, args: Sequence[MathArg]) -> mpmath.mpf:
    d1, d2 = args[1], args[2]
    if isinstance(d1, bool) or isinstance(d2, bool):
        msg = "no tail map for a logical df"
        raise ValueError(msg)
    return quantile(1 - q if upper else q, mpmath.floor(d1), mpmath.floor(d2))


def _f_inv(args: Sequence[MathArg]) -> mpmath.mpf:
    p, d1, d2 = args
    assert not isinstance(p, bool) and not isinstance(d1, bool) and not isinstance(d2, bool)
    # F.INV(0, d1, d2) is 0 in Excel (accuracy/F.INV/s1 A111 and A125, excel-graph):
    # the quantile of probability 0 is the support's lower end.
    return mpmath.mpf(0) if p == 0 else quantile(p, d1, d2)


F_DIST = register(
    Definition(
        function="F.DIST",
        params=(
            Param(
                "x",
                "real",
                Interval(0, float("inf"), lo_open=False),
                typical=(0.05, 20),
                extreme=(1e-300, 1e300),
                source="Abramowitz and Stegun 26.6",
            ),
            _df("deg_freedom1", TRUNC_SOURCE),
            _df("deg_freedom2", TRUNC_SOURCE),
            Param("cumulative", "bool", source="catalog"),
        ),
        value=_f_dist,
        tail=_tail_x,
        tail_param="x",
        sources=(DF_SOURCE, TRUNC_SOURCE, "Abramowitz and Stegun 26.6.1, 26.6.2"),
    )
)

F_INV = register(
    Definition(
        function="F.INV",
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 1, lo_open=False),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=INV_SOURCE,
            ),
            _df("deg_freedom1"),
            _df("deg_freedom2"),
        ),
        value=_f_inv,
        sources=(DF_SOURCE, INV_SOURCE, "Abramowitz and Stegun 26.6.2"),
    )
)


LEGACY_SOURCE = "statistics.md#legacy-distribution-names-sweep-t-1047"
THIN_SOURCE = "statistics.md#thin-distribution-functions-argument-coercion-truncation-and-error-edges-t-1049"
RT_ARGS_SOURCE = "statistics.md#fs-incomplete-beta-arguments-keep-r--1-t-3567"
CDF_OVERFLOW_SOURCE = "statistics.md#the-f-cdf-is-num-where-df2df1-x-overflows-t-2435"
X_OVERFLOW_SOURCE = "statistics.md#overflowing-arguments-permuts-limit-and-argument-error-order-t-1290"
SATURATION_SOURCE = "statistics.md#finvrt-saturates-at-3696066238512854e150df1-in-the-far-tail-t-2435"


def _f_dist_rt(args: Sequence[MathArg]) -> mpmath.mpf:
    x, d1, d2 = scalars(args)
    return upper_tail(x, d1, d2)


def _f_dist_rt_domain(args: Sequence[MathArg]) -> str | None:
    """The right tail's errors from Excel's own arguments (T-3567's r = fl(df2/fl(df1·x))): `#DIV/0!` where
    fl(df1·x) overflows (`F.DIST.RT(1E308,2,3)`, `FDIST(1E308,2,3)`, T-1290's A3 and A4) and `#NUM!` where the
    quotient does (`F.DIST.RT(1E-300,6,1E+10)`, `FDIST(1E-300,6,1E+10)`, T-2435), and the L140 bup band
    (`bup_ratio_overflows`); these are probed on both names. A positive subnormal x passes no rule: number text below
    the smallest normal double reads as 0 (numbers.md "Subnormal results become 0"), shown for F only by
    `FDIST(1E-308,6,1E+8)` = 1 (probes/t2435-dist-extremes A130); a negative subnormal x is unprobed (x < 0 is the
    interval's error, which the math keeps)."""
    x, d1, d2 = (float(a) for a in scalars(args))
    if x < sys.float_info.min:  # x <= 0 is the interval's; a subnormal x reads as 0
        return None
    mx = d1 * x
    if math.isinf(mx):
        return "df1·x overflows the double range (#DIV/0!)"
    if math.isinf(d2 / mx):
        return "df2/(df1·x) overflows the double range (#NUM!)"
    if bup_ratio_overflows(d1, d2, x):
        return "bratio's L140 bup sum overflows (#NUM!)"
    return None


LN_DBL_MAX = math.log(sys.float_info.max)
BUP_MAX_B0 = 40  # L140 bup takes b0 < 40 only
L140_MIN_B0X0 = 0.7  # and b0·x0 > 0.7; x0 <= 0.7 is bup + bpser
BGRAT_MIN_A0 = 15  # bup + bgrat needs a0 > 15 at x0 > 0.7 (bup + bup + bgrat below)


def bup_ratio_overflows(df1: float, df2: float, x: float) -> bool:
    """Whether Excel's right tail is `#NUM!` because bratio's L140 bup running product overflows (T-4044). Public so
    the unit tests can pin the band edges directly. F's incomplete beta takes shapes (a, b) = (df1/2, df2/2) on
    T-3567's r >= 1 arguments (u, v); bratio swaps to (a0, b0, x0) = (b, a, v) where its lambda is negative, and takes
    L140 where 1 < min(a, b), b0 < 40 and b0·x0 > 0.7, then bup + bpser for x0 <= 0.7 or bup + bgrat for a0 > 15
    (statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s (T-3553)", its T-3576 L140 rule; the engine's
    `bratio::l140_overflows`). With n = int(b0) (one less at an integer b0) and c = b0 - n, Excel is `#NUM!` where the
    product of (c + a0 + l)/(c + 1 + l) for l < n - 1 passes the largest double: 713.6 in ln at (2.25E9, b0 39.5),
    `#NUM!`, and 709.1 at (2E9, 39.5), a number (statistics.md "Incomplete beta at a huge first shape", its T-4080
    and T-4044 items). TOMS 708's early break is not modelled; the rule gives Excel's type on every positive-x
    F.DIST.RT and FDIST cell of probes-numeric t4017-f-huge-df2-*, t4030-f-dist-rt-df1-boundary and
    t4045-f-dist-rt-zero-cut."""
    mx = df1 * x
    r = df2 / mx if mx <= df2 else mx / df2
    u, v = (1 / (1 + r), r / (1 + r)) if mx <= df2 else (r / (1 + r), 1 / (1 + r))
    a, b = df1 / 2, df2 / 2
    if min(a, b) <= 1:
        return False
    lam = (a + b) * v - b if a > b else a - (a + b) * u  # TOMS 708's lambda, as the engine forms it
    a0, b0, x0 = (b, a, v) if lam < 0 else (a, b, u)
    if not (b0 < BUP_MAX_B0 and b0 * x0 > L140_MIN_B0X0 and (x0 <= L140_MIN_B0X0 or a0 > BGRAT_MIN_A0)):
        return False
    n = math.floor(b0) - (1 if b0 == math.floor(b0) else 0)
    c = b0 - n
    log_product = sum(math.log((c + a0 + i) / (c + 1 + i)) for i in range(n - 1))
    return log_product > LN_DBL_MAX


def _f_inv_rt(args: Sequence[MathArg]) -> mpmath.mpf:
    p, d1, d2 = args
    assert not isinstance(p, bool) and not isinstance(d1, bool) and not isinstance(d2, bool)
    # p = 1 is 0 on both names (`F.INV.RT(1,3,7)`, `FINV(1,2,5)`): the right tail's support end.
    return mpmath.mpf(0) if p == 1 else quantile(p, d1, d2, upper=True)


def _f_dist_rt_definition(function: str, truncates: str) -> Definition:
    """F.DIST.RT, and FDIST with the same arguments and rules: equal on every paired cell (statistics.md "Legacy
    distribution names sweep (T-1047)"); x = 0, x < 0, df truncation and both overflow rules are probed on both names.
    `truncates` cites each name's own truncation evidence."""
    return Definition(
        function=function,
        params=(
            Param(
                "x",
                "real",
                Interval(0, float("inf"), lo_open=False),
                typical=(0.05, 20),
                extreme=(1e-300, 1e300),
                source=THIN_SOURCE,
            ),
            _df("deg_freedom1", truncates),
            _df("deg_freedom2", truncates),
        ),
        value=_f_dist_rt,
        domain=_f_dist_rt_domain,
        tail=_tail_x,
        tail_param="x",
        sources=(
            DF_SOURCE,
            LEGACY_SOURCE,
            THIN_SOURCE,
            RT_ARGS_SOURCE,
            CDF_OVERFLOW_SOURCE,
            X_OVERFLOW_SOURCE,
            "Abramowitz and Stegun 26.6.2",
        ),
    )


def _f_inv_rt_definition(function: str, truncates: str) -> Definition:
    """F.INV.RT, and FINV with the same arguments and rules: equal on every paired cell (statistics.md "Legacy
    distribution names sweep (T-1047)"); p = 0 and p = 1 and df truncation are probed on both names. Excel's far-tail
    saturation at df2 below 2 (`SATURATION_SOURCE`) is not math: the table keeps the math there."""
    return Definition(
        function=function,
        params=(
            Param(
                "probability",
                "real",
                Interval(0, 1, lo_open=True, hi_open=False),
                typical=(0.001, 0.999),
                extreme=(1e-300, 1e-15),
                source=THIN_SOURCE,
            ),
            _df("deg_freedom1", truncates),
            _df("deg_freedom2", truncates),
        ),
        value=_f_inv_rt,
        sources=(
            DF_SOURCE,
            LEGACY_SOURCE,
            THIN_SOURCE,
            INV_SOURCE,
            SATURATION_SOURCE,
            "Abramowitz and Stegun 26.6.2",
        ),
    )


F_DIST_RT = register(_f_dist_rt_definition("F.DIST.RT", RT_ARGS_SOURCE))
FDIST = register(_f_dist_rt_definition("FDIST", LEGACY_SOURCE))
F_INV_RT = register(_f_inv_rt_definition("F.INV.RT", THIN_SOURCE))
FINV = register(_f_inv_rt_definition("FINV", LEGACY_SOURCE))
