"""CONFIDENCE.NORM and CONFIDENCE (Gate A, T-4101).

The math: the normal margin z*sd/sqrt(n) with z = NORM.S.INV(1 - alpha/2), through normal.py's `standard_quantile`
at the working precision (1 - alpha/2 exact there), and n the size truncated toward zero.

Sources:
- statistics.md, "Legacy distribution names sweep (T-1047)": CONFIDENCE = CONFIDENCE.NORM on every paired cell, and
  the size truncates (`CONFIDENCE(0.05,1,100.9)` is size 100, `CONFIDENCE(0.05,1,1.5)` size 1, `CONFIDENCE(0.05,1,0.9)`
  `#NUM!`; probes-numeric/m9-legacy-confidence A19-A21). alpha 0 or 1 or outside, sd 0 or below, and a size below 1
  are `#NUM!` (A22-A28), as Microsoft's CONFIDENCE.NORM page states. The truncation was probed on CONFIDENCE only, so
  only CONFIDENCE's size `truncates`: a CONFIDENCE.NORM eval cell with a non-integer size gets no math value.
- statistics.md, "CONFIDENCE.NORM forms 1 - alpha/2 in doubles (T-4101)": Excel's value is NORM.S.INV of the double
  1 - alpha/2 times sd/sqrt(n), within an ulp or so on every literal cell. Where that double is 1 (alpha <= 2^-53)
  it is `#NUM!`, as `NORM.S.INV(1)` is: `CONFIDENCE(1E-16,..)` and `CONFIDENCE.NORM(1E-100,..)` are `#NUM!` while
  `CONFIDENCE(1E-15,0.01,9)` is a number. That is the definition's one domain rule beyond the intervals. The cut is
  pinned from below by `NORM.S.INV(1-1E-16)` = 8.209536151601386 (probes-numeric/t1543-inverse-cdf-tiny-p A11): 1-1E-16
  is the double 1-2^-53, the largest p below 1, so alpha > 2^-53 is a number; only (1E-16, 2^-53] has no direct cell.
- statistics.md, "NORM.S.INV is AS 241 PPND16 in the tails; the central region is unfitted (T-1746)": the quantile
  Excel uses is up to 31 ulp off in the centre, far under the gate's tolerance.

The cancellation region (decision 0188 R3; Excel kept as the target, decision 0300). Rounding 1 - alpha/2 to a
double moves p by up to 2^-54, which moves z by about 2^-54/phi(z) and the value by 2^-54/(phi(z)*|z|) relative:
1.6E-3 at `CONFIDENCE(1E-15,0.01,9)` and all of it at `CONFIDENCE(1-1E-16,0.1,6)` = 0. A point where that bound passes
`CONDITION_LIMIT` (alpha below about 4E-8 or above about 1 - 1.1E-6) is redrawn, as GAUSS's region is (normal.py),
so no table point depends on it; its eval cells keep their math values and stay in the eval suites.

Also left out, as unprobed: points whose sd, sd*z, sd/sqrt(n), z/sqrt(n) or value leaves the normal double range below
1E+300 (no cell shows where Excel's product overflows or underflows): `_common.confidence_margin`, shared with
CONFIDENCE.T (T-4581).
"""

from __future__ import annotations

import math
import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import CONDITION_LIMIT, confidence_margin, real_args
from rustsheet_tools.accuracy.defs.continuous import LEGACY_FIRST_SOURCE
from rustsheet_tools.accuracy.defs.normal import INV_SOURCE, density, standard_quantile
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

DOUBLE_P_SOURCE = "statistics.md#confidencenorm-forms-1---alpha2-in-doubles-t-4101"
MICROSOFT_SOURCE = "Microsoft CONFIDENCE.NORM / CONFIDENCE pages: argument limits"

P_ROUNDING = 2.0**-54  # the largest rounding of 1 - alpha/2 in [1/2, 1)
REDRAWS = 50


def p_rounds_to_one(alpha: float) -> bool:
    """Whether Excel's double 1 - alpha/2 is 1 (its `#NUM!`, statistics.md T-4101)."""
    return 1.0 - alpha / 2 == 1.0


def confidence_cancels(alpha: float) -> bool:
    """Whether rounding 1 - alpha/2 to a double can move the value by more than `CONDITION_LIMIT` (the excluded
    region). A point outside (0, 1), or whose 1 - alpha/2 rounds to 1, is an error, not in the region."""
    if not 0 < alpha < 1 or p_rounds_to_one(alpha):
        return False
    with mpmath.workdps(30):
        z = standard_quantile(1 - mpmath.mpf(alpha) / 2)
        return bool(CONDITION_LIMIT * density(z) * abs(z) < P_ROUNDING)


def _domain(args: Sequence[MathArg]) -> str | None:
    alpha = float(real_args(args[:1])[0])
    if 0 < alpha < 1 and p_rounds_to_one(alpha):
        return "1 - alpha/2 rounds to 1 in doubles (NORM.S.INV(1) is #NUM!)"
    return None


def _confidence(args: Sequence[MathArg]) -> mpmath.mpf:
    alpha, sd, size = real_args(args)
    return confidence_margin(standard_quantile(1 - alpha / 2), sd, size)


# Evidenced error points and their neighbours (probes-numeric/m9-legacy-confidence, gen-m9 literal cells).
_EDGES: tuple[tuple[float, float, int], ...] = (
    (0.0, 1.0, 100),  # #NUM!
    (1.0, 1.0, 100),  # #NUM!
    (-0.1, 1.0, 100),  # #NUM!
    (1.5, 0.5, 6),  # #NUM!
    (0.05, 0.0, 100),  # #NUM!
    (0.05, -1.0, 100),  # #NUM!
    (0.05, 1.0, 0),  # #NUM!
    (0.05, 1.0, -5),  # #NUM!
    (0.05, 1.0, 1),
    (1e-16, 0.5, 6),  # #NUM!: 1 - alpha/2 rounds to 1
    (1e-100, 2.0, 5),  # #NUM!
    (1e-300, 99999.0, 9),  # #NUM!
    (0.05, 1e-300, 2),
    (0.05, 1e299, 3),
    (0.5, 1.0, 2),
    (0.999, 3.0, 7),
)


def _alpha(stratum: str, rng: random.Random) -> float:
    if stratum == "tail":  # toward either end, just outside the cancellation region
        return log_uniform(rng, 4e-8, 1e-3) if rng.random() < 0.6 else 1 - log_uniform(rng, 1.2e-6, 1e-2)
    if stratum == "extreme":
        return log_uniform(rng, 4e-8, 0.999)
    if stratum == "log-uniform":  # mostly valid, sometimes in the `#NUM!` region below 2^-53
        return log_uniform(rng, 1e-300, 1.1e-16) if rng.random() < 0.1 else log_uniform(rng, 4e-8, 0.999999)
    return rng.uniform(0.001, 0.999)


def _sd_size(stratum: str, rng: random.Random) -> tuple[float, float]:
    if stratum == "extreme":
        return log_uniform(rng, 1e-250, 1e250), log_uniform(rng, 1, 1e12)
    if stratum == "log-uniform":
        return log_uniform(rng, 1e-300, 1e300), log_uniform(rng, 1, 1e15)
    return log_uniform(rng, 0.1, 100), rng.uniform(1, 100)


def _draw(stratum: str, rng: random.Random) -> list[Arg]:
    """A point of `stratum` outside the cancellation region (the edges are the evidenced error points)."""
    if stratum == "edge":
        return list(rng.choice(_EDGES))
    for _ in range(REDRAWS):
        alpha = float(f"{_alpha(stratum, rng):.15g}")
        if not confidence_cancels(alpha):
            sd, size = _sd_size(stratum, rng)
            return [alpha, sd, size]
    return [0.05, 1.0, 10]


def _definition(function: str, *, truncates: bool) -> Definition:
    """CONFIDENCE.NORM, and CONFIDENCE with the same rules (statistics.md T-1047: equal on every paired cell); only
    `truncates` differs, by the evidence each name has."""
    return Definition(
        function=function,
        params=(
            Param("alpha", "real", Interval(0, 1), typical=(0.001, 0.999), source=MICROSOFT_SOURCE),
            Param(
                "standard_dev",
                "real",
                Interval(0, math.inf),
                typical=(0.1, 100),
                extreme=(1e-250, 1e250),
                source=MICROSOFT_SOURCE,
            ),
            Param(
                "size",
                "int",
                Interval(1, math.inf, lo_open=False),
                typical=(1, 100),
                extreme=(1e3, 1e12),
                source=LEGACY_FIRST_SOURCE,
                truncates=LEGACY_FIRST_SOURCE if truncates else "",
            ),
        ),
        value=_confidence,
        domain=_domain,
        draw=_draw,
        sources=(LEGACY_FIRST_SOURCE, DOUBLE_P_SOURCE, INV_SOURCE, MICROSOFT_SOURCE),
    )


CONFIDENCE_NORM = register(_definition("CONFIDENCE.NORM", truncates=False))
CONFIDENCE = register(_definition("CONFIDENCE", truncates=True))
