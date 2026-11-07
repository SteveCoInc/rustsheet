"""Equivalence classes of the F family's arguments (T-4857, docs/research/t4857/ANALYSIS.md).

One definition of every class, shared by the coverage audit (`audit.py`), the probe generator
(`gen.py`) and the scorer (`score.py`). Each class is grounded in the engine's code or in
docs/excel-semantics/statistics.md; ANALYSIS.md cites the source of each edge.

The kernel classes mirror the engine's dispatch to the bit where they can:
- `f_beta_arguments` is `fisher.rs` `f_beta_arguments` (Excel's r >= 1 arguments, T-3567);
- `dispatch` is `bratio/mod.rs` `dispatch` and `small_shape_branch` (TOMS 708 bratio);
- `closed_class` is `incomplete_beta.rs` `closed_form_tails` / `shape_one_tails`;
- `l140_status` is `bratio/bup.rs` `bup_split`, `MAX_BUP_TERMS` and `MAX_BUP_A0`;
- `fallback_kernel` is `incomplete_beta.rs` `beta_tails` (half-integer series, large-shape
  expansion, Cephes);
- `density_class` is `f_density.rs` `excel_f_density` and `constant_offset`;
- `quantile_path` is `fisher.rs` `f_quantile`.
Python's float is the same IEEE double, so every comparison lands on the same side as the engine's
(the two `powf` calls of `small_shape_branch` may differ by an ulp at their cuts, as the engine's
comment says of bratio's own pow).
"""

from __future__ import annotations

import math
from dataclasses import dataclass

DBL_MIN = 2.2250738585072014e-308
DBL_MAX = 1.7976931348623157e308
EPS = 1e-15  # bratio's eps (bratio/mod.rs EPS)
W1_BPSER_MIN_X = 0.3  # bratio/mod.rs
A1_CLOSED_FORM_MAX_Y = 0.9  # incomplete_beta.rs
MAX_BUP_TERMS = 9.0  # bratio/bup.rs
MAX_BUP_A0 = 1e7  # bratio/bup.rs
MAXGAM = 171.6243769563027  # incomplete_beta.rs
SERIES_MIN_SHAPE_SUM = 10.0  # incomplete_beta.rs
LARGE_SHAPE_MIN = 1e6  # large_shape_beta.rs MIN_SHAPE
LARGE_SHAPE_MAX_B = 40.0  # large_shape_beta.rs MAX_SECOND_SHAPE
F_DF_MAX = 1e10  # fisher.rs
STATRS_DF_MAX = 1e5  # fisher.rs F_QUANTILE_STATRS_DF_MAX
STATRS_P_MIN = 1e-20  # fisher.rs F_QUANTILE_STATRS_P_MIN
REFINE_P_MIN = 1e-15  # fisher.rs F_QUANTILE_REFINE_P_MIN
LOG1P_SERIES_LIMIT = 0.375  # f_density.rs
LARGE_DF_MIN = 16.0  # f_density.rs
EXPONENT_FLOOR = -800.0  # f_density.rs


# --------------------------------------------------------------------------- argument classes


def df_class(raw: float) -> str:
    """A degrees-of-freedom argument's class, on the raw value (truncation is its own class).

    The edges are the engine's shape thresholds in df units (shape = df/2): a0 = 8 (df 16, the
    density's LARGE_DF_MIN and Stirling's STIRLING_MIN), a0 = 15 (df 30, L140's bup+bgrat split),
    b0 = 40 (df 80, bratio's b0 < 40 and large_shape_beta's MAX_SECOND_SHAPE), a0 = 100 (df 200,
    bfrac against basym), df 1E5 (the quantile's statrs/BETA.INV switch), shape 1E6 (df 2E6,
    large_shape_beta's MIN_SHAPE), shape 1E7 (df 2E7, MAX_BUP_A0), and the 1E10 cut.
    """
    if math.isnan(raw):
        return "nan"
    if raw < 1:
        return "lt1"
    d = math.trunc(raw)
    if d > F_DF_MAX:
        return "gt1e10"
    if raw != d and d == 1:
        return "frac1-2"
    for edge in (1, 2, 16, 30, 31, 79, 80, 81, 200, 201):
        if d == edge:
            return str(edge)
    if d < 16:
        return "3-15"
    if d < 30:
        return "17-29"
    if d < 79:
        return "32-78"
    if d < 200:
        return "82-199"
    if d < STATRS_DF_MAX:
        return "202-1e5"
    if d < 2 * LARGE_SHAPE_MIN:
        return "1e5-2e6"
    if d <= 2 * MAX_BUP_A0:
        return "2e6-2e7"
    if d < 1e9:
        return "2e7-1e9"
    if d < F_DF_MAX:
        return "1e9-1e10"
    return "1e10"


DF_CLASSES = (
    "lt1",
    "1",
    "frac1-2",
    "2",
    "3-15",
    "16",
    "17-29",
    "30",
    "31",
    "32-78",
    "79",
    "80",
    "81",
    "82-199",
    "200",
    "201",
    "202-1e5",
    "1e5-2e6",
    "2e6-2e7",
    "2e7-1e9",
    "1e9-1e10",
    "1e10",
    "gt1e10",
)
# Classes whose cells are errors before any kernel runs: they pair with nothing numeric.
DF_ERROR_CLASSES = ("lt1", "gt1e10")


def x_magnitude(x: float, negzero: bool = False) -> str:
    """x's magnitude class: the sign and the range edges of the double format."""
    if math.isnan(x):
        return "nan"
    if x == 0:
        return "-0" if negzero or math.copysign(1.0, x) < 0 else "0"
    if x < 0:
        return "neg"
    if x < DBL_MIN:
        return "subnormal"
    if x < 1e-307:
        return "dblmin"
    for upper, label in (
        (1e-100, "1e-307..1e-100"),
        (1e-16, "1e-100..1e-16"),
        (1e-3, "1e-16..1e-3"),
        (0.1, "1e-3..0.1"),
        (10, "0.1..10"),
        (1e3, "10..1e3"),
        (1e16, "1e3..1e16"),
        (1e100, "1e16..1e100"),
        (1e300, "1e100..1e300"),
    ):
        if x < upper:
            return label
    if x < DBL_MAX:
        return "1e300..max"
    return "dblmax"


X_MAG_CLASSES = (
    "neg",
    "-0",
    "0",
    "subnormal",
    "dblmin",
    "1e-307..1e-100",
    "1e-100..1e-16",
    "1e-16..1e-3",
    "1e-3..0.1",
    "0.1..10",
    "10..1e3",
    "1e3..1e16",
    "1e16..1e100",
    "1e100..1e300",
    "1e300..max",
    "dblmax",
)


def x_position(cdf: float | None, sf: float | None) -> str:
    """Where x sits in its distribution, from the math lower tail `cdf` and upper tail `sf`
    (either may be None when unknown): eight tail-probability bands, split at the underflow
    (1E-300), the double's 1E-16 resolution of 1 - p, 1E-3 and the median."""
    if cdf is None and sf is None:
        return "unknown"
    if cdf is not None and (sf is None or cdf <= sf):
        p, side = cdf, "lo"
    else:
        assert sf is not None
        p, side = sf, "hi"
    if p < 1e-300:
        band = "deep"
    elif p < 1e-16:
        band = "far"
    elif p < 1e-3:
        band = "tail"
    else:
        band = "body"
    return f"{band}-{side}"


X_POS_CLASSES = ("deep-lo", "far-lo", "tail-lo", "body-lo", "body-hi", "tail-hi", "far-hi", "deep-hi")


def p_class(p: float) -> str:
    """A probability argument's class (F.INV, F.INV.RT, FINV): the range edges of the double format
    near 0 and 1 and the quantile's own switches (statrs' window 1E-20, the refinement cut 1E-15)."""
    if math.isnan(p):
        return "nan"
    if p < 0:
        return "neg"
    if p == 0:
        return "0"
    if p < DBL_MIN:
        return "subnormal"
    if p < 1e-307:
        return "dblmin"
    for upper, label in (
        (1e-200, "1e-307..1e-200"),
        (STATRS_P_MIN, "1e-200..1e-20"),
        (REFINE_P_MIN, "1e-20..1e-15"),
        (1e-3, "1e-15..1e-3"),
        (0.5, "1e-3..0.5"),
    ):
        if p < upper:
            return label
    if p == 0.5:
        return "0.5"
    if p <= 1 - 1e-3:
        return "0.5..1-1e-3"
    if p < 1 - 1e-15:
        return "1-1e-3..1-1e-15"
    if p < 1:
        return "1-1e-15..1"
    if p == 1:
        return "1"
    return "gt1"


P_CLASSES = (
    "neg",
    "0",
    "subnormal",
    "dblmin",
    "1e-307..1e-200",
    "1e-200..1e-20",
    "1e-20..1e-15",
    "1e-15..1e-3",
    "1e-3..0.5",
    "0.5",
    "0.5..1-1e-3",
    "1-1e-3..1-1e-15",
    "1-1e-15..1",
    "1",
    "gt1",
)


# --------------------------------------------------------------------------- kernel classes


def f_beta_arguments(d1: float, d2: float, x: float) -> tuple[float, float]:
    """`fisher.rs` `f_beta_arguments`: Excel's (u, v) with r = fl(d2/fl(d1 x)) or its inverse, r >= 1."""
    scaled = d1 * x
    if scaled <= d2:
        r = d2 / scaled if scaled != 0 else math.inf
        if math.isinf(r):
            return (0.0, 1.0)
        return (1.0 / (1.0 + r), r / (1.0 + r))
    r = scaled / d2
    return (r / (1.0 + r), 1.0 / (1.0 + r))


@dataclass(frozen=True)
class Dispatch:
    """bratio's branch for one point after its swap (`bratio/mod.rs` `Dispatch`)."""

    branch: str
    swapped: bool
    a0: float
    b0: float
    x0: float
    y0: float
    lam: float


def _small_shape_branch(a0: float, b0: float, x0: float) -> str:
    if b0 < min(EPS, EPS * a0):
        return "Fpser"
    if a0 < min(EPS, EPS * b0) and b0 * x0 <= 1.0:
        return "Apser"
    if max(a0, b0) > 1.0:
        if b0 <= 1.0:
            return "WBpser"
        if x0 >= W1_BPSER_MIN_X:
            return "W1Bpser"
        if x0 < 0.1 and (x0 * b0) ** a0 <= 0.7:
            return "WBpser"
        if b0 > 15.0:
            return "Bgrat"
    else:
        if a0 >= min(0.2, b0) or x0**a0 <= 0.9:
            return "WBpser"
        if x0 >= W1_BPSER_MIN_X:
            return "W1Bpser"
    return "BupBgrat"


def dispatch(a: float, b: float, x: float, y: float) -> Dispatch | None:
    """`bratio/mod.rs` `dispatch`."""
    if not (0.0 <= x <= 1.0 and 0.0 <= y <= 1.0) or a < 0 or b < 0 or a + b <= 0:
        return None
    if x == 0.0:
        return Dispatch("Zero", False, a, b, x, y, 0.0)
    if y == 0.0:
        return Dispatch("One", False, a, b, x, y, 0.0)
    if max(a, b) < EPS * 0.001:
        return Dispatch("Tiny", False, a, b, x, y, 0.0)
    if min(a, b) <= 1.0:
        swapped = x > 0.5
        a0, b0, x0, y0 = (b, a, y, x) if swapped else (a, b, x, y)
        return Dispatch(_small_shape_branch(a0, b0, x0), swapped, a0, b0, x0, y0, 0.0)
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    swapped = lam < 0.0
    a0, b0, x0, y0 = (b, a, y, x) if swapped else (a, b, x, y)
    lam = abs(lam)
    if b0 < 40.0:
        if b0 * x0 <= 0.7:
            branch = "BpserLarge"
        elif x0 <= 0.7:
            branch = "L140BupBpser"
        elif a0 <= 15.0:
            branch = "L140BupBupBgrat"
        else:
            branch = "L140BupBgrat"
    elif (a0 > b0 and (b0 <= 100.0 or lam > b0 * 0.03)) or (a0 <= b0 and (a0 <= 100.0 or lam > a0 * 0.03)):
        branch = "Bfrac"
    else:
        branch = "Basym"
    return Dispatch(branch, swapped, a0, b0, x0, y0, lam)


def closed_class(a: float, b: float, x: float, y: float) -> str | None:
    """`closed_form_tails` / `shape_one_tails`: a df of 2 (shape 1) on the oriented arguments."""
    if not (a == 1.0 or b == 1.0) or not (x > 0.0 and y > 0.0):
        return None
    swap = x > 0.5
    a0, b0, _x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    if b0 == 1.0:
        return "closed-b0=1"
    if a0 == 1.0 and y0 <= A1_CLOSED_FORM_MAX_Y:
        return "closed-a0=1"
    return "shape1-open"


def _bup_split(b0: float) -> tuple[float, float]:
    n = math.trunc(b0)
    c = b0 - n
    return (n - 1.0, 1.0) if c == 0.0 else (float(n), c)


def l140_status(d: Dispatch) -> str:
    """Whether the engine's L140 bup+bgrat port takes the point (`bup.rs` `l140_bup_bgrat`)."""
    n, c = _bup_split(d.b0)
    takes_terms = n > 1.0 or c == 1.0
    if takes_terms and not (n <= MAX_BUP_TERMS and d.a0 <= MAX_BUP_A0):
        return "unported"
    return "ported"


def fallback_kernel(a: float, b: float, x: float, y: float) -> str:
    """`incomplete_beta.rs` `beta_tails`'s kernel for an F point the port does not take: F's shapes
    are always half integers, so the series where a + b is in [10, MAXGAM), else the large-shape
    expansion at a first shape (after the flip at the mean) of 1E6 or more with the second below 40,
    else Cephes' fraction."""
    flip = x > a / (a + b)
    a1, b1 = (b, a) if flip else (a, b)
    if SERIES_MIN_SHAPE_SUM <= a + b < MAXGAM:
        return "series"
    if a1 >= LARGE_SHAPE_MIN and b1 < LARGE_SHAPE_MAX_B:
        return "large-shape"
    return "cephes"


def kernel_class(d1: float, d2: float, x: float) -> str:
    """The incomplete-beta branch an F tail (F.DIST cdf, F.DIST.RT, FDIST) takes at truncated df
    (d1, d2) and x > 0, as bratio dispatches it on Excel's arguments, with the engine's own status
    where the port does not cover the branch."""
    if x <= 0:
        return "x<=0"
    if not math.isfinite(d1 * x):
        return "d1x-overflow"
    if x > 0 and not math.isfinite(d2 / (d1 * x)):
        return "ratio-overflow"
    u, v = f_beta_arguments(d1, d2, x)
    a, b = d1 / 2.0, d2 / 2.0
    closed = closed_class(a, b, u, v)
    if closed is not None:
        return closed
    d = dispatch(a, b, u, v)
    if d is None:
        return "none"
    if d.branch == "L140BupBgrat":
        if l140_status(d) == "ported":
            return "L140BupBgrat"
        return f"L140BupBgrat-fb:{fallback_kernel(a, b, u, v)}"
    if d.branch == "L140BupBupBgrat":
        return f"L140BupBupBgrat-fb:{fallback_kernel(a, b, u, v)}"
    if d.branch == "Bgrat" and d.a0 != 0.5:
        return f"Bgrat-fb:{fallback_kernel(a, b, u, v)}"
    return d.branch


def kernel_detail(d1: float, d2: float, x: float) -> Dispatch | None:
    """bratio's dispatch on Excel's arguments, for the edge finders."""
    if x <= 0 or not math.isfinite(d1 * x) or not math.isfinite(d2 / (d1 * x)):
        return None
    u, v = f_beta_arguments(d1, d2, x)
    return dispatch(d1 / 2.0, d2 / 2.0, u, v)


def density_class(d1: float, d2: float, x: float) -> str:
    """`f_density.rs` `excel_f_density`'s regime at x > 0: side 1 (fl(m x) <= n) or 2, r up to
    0.375 (Excel's log1p series) or above (LN(1 + r)), the small- or large-df constant
    (min(d) < 16), and |P| = |c ln x| below 4, 4 to 32, or above (T-2586's parity bands)."""
    if x <= 0:
        return "x<=0"
    if x < DBL_MIN:
        return "subnormal-x"
    mx = d1 * x
    if not math.isfinite(mx):
        return "d1x-overflow"
    side_one = mx <= d2
    r = mx / d2 if side_one else d2 / mx
    c = d1 / 2.0 if side_one else -d2 / 2.0
    p = abs(c * math.log(x))
    pband = "P<4" if p < 4 else ("P4-32" if p < 32 else "P>32")
    const = "big" if min(d1, d2) >= LARGE_DF_MIN else "small"
    series = "ser" if r <= LOG1P_SERIES_LIMIT else "ln"
    return f"s{1 if side_one else 2}-{series}-{const}-{pband}"


def quantile_path(p: float, d1: float, d2: float, right_tail: bool) -> str:
    """`fisher.rs` `f_quantile`'s path for p in [0, 1] at truncated df."""
    lower = 1.0 - p if right_tail else p
    if lower <= 0.0:
        return "zero"
    if lower >= 1.0 and not right_tail:
        return "inf"
    if d1 >= STATRS_DF_MAX or d2 >= STATRS_DF_MAX:
        return "beta-inv" + ("-flip" if p > 0.5 else "")
    statrs = STATRS_P_MIN <= lower <= 1.0 - STATRS_P_MIN
    if statrs and lower < REFINE_P_MIN:
        return "statrs-unrefined"
    side = "upper" if lower > 0.5 else "lower"
    return f"refine-{side}" + ("" if statrs else "-nostart")


# --------------------------------------------------------------------------- tail probabilities


def tails(x: float, d1: float, d2: float, dps: int = 20) -> tuple[float, float, str]:
    """(cdf, sf, how) of the F distribution at x > 0, for classing (a few digits suffice): mpmath's
    betainc on the exact ratio z = d1 x/(d1 x + d2), the smaller tail computed directly; where
    mpmath's hypergeometric series does not converge (huge shapes), Paulson's normal approximation
    of the cube-root transform (Abramowitz and Stegun 26.6.15), which is good to a few percent of
    the log tail, enough for the decade-wide position classes. A tail below the double range is
    returned as 0.0."""
    import mpmath  # noqa: PLC0415 - only the classing needs it

    with mpmath.workdps(dps):
        xm, a, b = mpmath.mpf(x), mpmath.mpf(d1) / 2, mpmath.mpf(d2) / 2
        z = d1 * xm / (d1 * xm + d2)
        try:
            if max(d1, d2) >= 1e6:  # mpmath's series stalls for minutes at huge shapes
                raise ValueError
            if z <= a / (a + b):
                cdf = mpmath.betainc(a, b, 0, z, regularized=True)
                sf = 1 - cdf
            else:
                sf = mpmath.betainc(b, a, 0, d2 / (d1 * xm + d2), regularized=True)
                cdf = 1 - sf
            return float(cdf), float(sf), "mpmath"
        except (ValueError, ZeroDivisionError, mpmath.libmp.libhyper.NoConvergence):
            pass
        c1, c2 = mpmath.mpf(2) / (9 * d1), mpmath.mpf(2) / (9 * d2)
        cube = mpmath.cbrt(xm)
        zz = ((1 - c2) * cube - (1 - c1)) / mpmath.sqrt(c1 + cube**2 * c2)
        lo = mpmath.ncdf(zz)
        hi = mpmath.ncdf(-zz)
        return float(lo), float(hi), "paulson"


def f_quantile(p: float, d1: float, d2: float, upper: bool, dps: int = 25) -> float | None:
    """The x with P(F <= x) = p (or P(F > x) = p when `upper`), by bisection in ln x on `tails`'
    mpmath side: for placing probe points, not for scoring. None where mpmath cannot evaluate."""
    lo, hi = -700.0, 700.0
    for _ in range(80):
        mid = (lo + hi) / 2
        x = math.exp(mid)
        cdf, sf, how = tails(x, d1, d2, dps)
        if how != "mpmath":
            return None
        below = (sf > p) if upper else (cdf < p)
        if below:
            lo = mid
        else:
            hi = mid
        if hi - lo < 1e-13:
            break
    return math.exp((lo + hi) / 2)


# --------------------------------------------------------------------------- function shape


def mean(d2: float) -> float | None:
    """E[F] = d2/(d2 - 2) for d2 > 2."""
    return d2 / (d2 - 2.0) if d2 > 2 else None


def mode(d1: float, d2: float) -> float | None:
    """The density's mode (d1 - 2)/d1 · d2/(d2 + 2) for d1 > 2; at d1 <= 2 the mode is 0."""
    return (d1 - 2.0) / d1 * d2 / (d2 + 2.0) if d1 > 2 else None


def centre_class(x: float, d1: float, d2: float) -> str:
    """Whether x is within 1% of the mode or the mean (the density's turning point and the
    incomplete beta's flip x = a/(a + b) sit near them)."""
    for name, value in (("mode", mode(d1, d2)), ("mean", mean(d2))):
        if value and value > 0 and abs(x / value - 1.0) < 0.01:
            return name
    return "other"
