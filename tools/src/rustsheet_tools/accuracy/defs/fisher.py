"""The F distribution: F.DIST and F.INV.

Sources:
- statistics.md, "F-distribution df limit (T-1530)": either df above 1E10 (after truncation) makes
  every F function `#NUM!`, the density too; df is truncated like Student-t's.
- statistics.md, "F.INV is a crossing of F.DIST's tails (T-0974)": F.INV solves the smaller tail;
  `F.INV(1E-300,5,5)` = 5.081599884913164E-121 shows p reaches 1E-300.
- The math: P(F ≤ x) = I_z(d1/2, d2/2) with z = d1·x/(d1·x + d2), and its complement
  I_{1-z}(d2/2, d1/2) taken directly (Abramowitz and Stegun 26.6.2); the density
  d1^(d1/2) d2^(d2/2) x^(d1/2-1) / (B(d1/2, d2/2) (d1·x + d2)^((d1+d2)/2)) (A&S 26.6.1).
"""

from __future__ import annotations

from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import DF_CUT
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


def quantile(p: mpmath.mpf, d1: mpmath.mpf, d2: mpmath.mpf) -> mpmath.mpf:
    """The x with P(F ≤ x) = p, 0 < p < 1: the smaller tail is solved in log scales (`solve_smaller_tail`).
    NoValueError when the root is beyond the double range."""
    s = solve_smaller_tail(p, lambda x: lower_tail(x, d1, d2), lambda x: upper_tail(x, d1, d2), -1500, 710)
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
