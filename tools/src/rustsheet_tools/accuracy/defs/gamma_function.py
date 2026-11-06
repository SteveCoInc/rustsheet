"""The gamma function and its logarithm: GAMMA, GAMMALN and GAMMALN.PRECISE (T-4106).

The math is Γ(x) and ln Γ(x) (Abramowitz and Stegun 6.1.1, 6.1.40). Excel's GAMMA and GAMMALN are its own x87 routines,
a few ulps (GAMMA near 150 up to about 40 ulps, from x^(x/2) used twice) off the math; Gate A's envelope judges that.
Each rule below is VERIFIED on excel-graph cells:
- GAMMA (statistics.md "GAMMA and GAMMALN sweep (T-1042)", "GAMMA is Excel's own x87 routine (T-1941, T-2129)"):
  `#NUM!` at 0 and the negative integers (`GAMMA(0)`, `(-1)`, `(-2)`, `(-10)`), defined between them
  (`GAMMA(-0.5)`, `GAMMA(-2.25)`, `GAMMA(-9.9)`). The last finite argument is 171.6243769563027 and the next double up is
  `#NUM!` (probes/gamma-x87-edges); the math overflows a double at exactly that double too, so the overflow rule is the
  math's (`finite_double`). Below about -171 the reflection's GAMMA(-x) overflows and Excel gives 0 (`GAMMA(-171.55)`,
  `GAMMA(-200.3)`): the math there is below 1E-300, which the envelope's absolute floor accepts, so it is no domain rule.
- GAMMALN and GAMMALN.PRECISE (the same entries; statistics.md "GAMMALN is Excel's own x87 routine (T-1941, T-2128)": one
  routine for both names): `#NUM!` at any x <= 0 (0, -1E-300, negative non-integers), no upper limit
  (`GAMMALN(1E+300)` = 6.897755278982137E+302). GAMMALN.PRECISE reuses GAMMALN's definition (T-3798's alias default).
"""

from __future__ import annotations

import math
import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, finite_double, one_of, scalars, signed
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

SWEEP_SOURCE = "statistics.md#gamma-and-gammaln-sweep-t-1042"
GAMMA_SOURCE = "statistics.md#gamma-is-excels-own-x87-routine-t-1941-t-2129"
GAMMALN_SOURCE = "statistics.md#gammaln-is-excels-own-x87-routine-t-1941-t-2128"
INTEGER_SOURCE = "statistics.md#gamma-is-exact-at-small-positive-integers-but-not-always-correctly-rounded"
GAMMA_LAST = 171.6243769563027  # the last finite GAMMA (probes/gamma-x87-edges); the next double up is #NUM!
GAMMA_NEXT = math.nextafter(GAMMA_LAST, math.inf)
TINY = 1e-300  # the smallest |x| drawn: 1/x of a smaller one approaches the double range


def gamma_domain(args: Sequence[MathArg]) -> str | None:
    """GAMMA's poles: 0 and the negative integers are `#NUM!` (SWEEP_SOURCE)."""
    (x,) = scalars(args)
    return "a pole at 0 or a negative integer (#NUM!)" if x <= 0 and x == mpmath.floor(x) else None


def _gamma(args: Sequence[MathArg]) -> mpmath.mpf:
    (x,) = scalars(args)
    return finite_double(mpmath.gamma(x), "GAMMA")


def _gammaln(args: Sequence[MathArg]) -> mpmath.mpf:
    (x,) = scalars(args)
    return mpmath.loggamma(x)


def _near_pole(rng: random.Random) -> list[Arg]:
    """A negative x a relative 1E-13 to 1E-2 from a pole 0 to -20 (a small |x| near 0)."""
    pole = rng.randint(0, 20)
    if pole == 0:
        return [signed(rng, log_uniform(rng, TINY, 1e-2))]
    return [-pole + signed(rng, pole * log_uniform(rng, 1e-13, 1e-2))]


def _gamma_edge(rng: random.Random) -> list[Arg]:
    return one_of(
        rng,
        (
            lambda: [float(rng.randint(1, 171))],
            lambda: [signed(rng, rng.randint(0, 171) + 0.5)],
            lambda: [float(-rng.randint(0, 12))],
            lambda: [
                rng.choice(
                    (GAMMA_LAST, GAMMA_NEXT, math.nextafter(GAMMA_LAST, 0), 171.5, 171.6, 171.7, 172.0)
                )
            ],
            lambda: [
                rng.choice((-171.0000000001, -171.55, -172.0000000001, -170.5, -10.0000001, -9.9999999))
            ],
            lambda: [rng.choice((TINY, -TINY, 1e-10, -1e-10, 0.7, 1.0 - 2**-53, 2.0, 10.0, 88.0, 89.0))],
        ),
    )


_gamma_draw = by_stratum(
    {
        "typical": lambda rng: [
            rng.uniform(0.05, 10.0) if rng.random() < 0.75 else rng.uniform(-10.0, -0.05)
        ],
        "tail": lambda rng: [rng.uniform(150.0, GAMMA_LAST)] if rng.random() < 0.3 else _near_pole(rng),
        "edge": _gamma_edge,
        "extreme": lambda rng: [
            rng.uniform(10.0, GAMMA_LAST) if rng.random() < 0.5 else -log_uniform(rng, 10.0, 1e3)
        ],
        "log-uniform": lambda rng: [
            log_uniform(rng, TINY, GAMMA_LAST) if rng.random() < 0.7 else -log_uniform(rng, TINY, 1e3)
        ],
    },
    "[x]: typical (0.05, 10) and (-10, -0.05), tails toward the overflow at 171.62 and within 1E-13 to 1E-2 of a "
    "pole 0 to -20, edges (integers to 171, half-integers, the poles, the overflow doubles, the reflection's underflow "
    "below -171, +-1E-300, the kernel cuts 0.7, 2, 10, 89), extreme (10, 171.62) and (-1000, -10), and log-uniform "
    "1E-300 to 171.62 and -1000 to -1E-300",
)


GAMMA = register(
    Definition(
        function="GAMMA",
        params=(Param("x", "real", Interval(), source=SWEEP_SOURCE),),
        value=_gamma,
        domain=gamma_domain,
        draw=_gamma_draw,
        sources=(SWEEP_SOURCE, GAMMA_SOURCE, INTEGER_SOURCE, "Abramowitz and Stegun 6.1.1"),
    )
)


def _gammaln_definition(function: str) -> Definition:
    """GAMMALN's definition under `function`'s name: GAMMALN.PRECISE is the same routine (GAMMALN_SOURCE)."""
    return register(
        Definition(
            function=function,
            params=(
                Param(
                    "x",
                    "real",
                    Interval(0, math.inf),
                    typical=(0.05, 30.0),
                    extreme=(TINY, 1e300),
                    edges=(0.7, 1.0, 1.5, 2.0, 3.0, 8.0),
                    source=SWEEP_SOURCE,
                ),
            ),
            value=_gammaln,
            sources=(SWEEP_SOURCE, GAMMALN_SOURCE, "Abramowitz and Stegun 6.1.40"),
        )
    )


GAMMALN = _gammaln_definition("GAMMALN")
GAMMALN_PRECISE = _gammaln_definition("GAMMALN.PRECISE")
