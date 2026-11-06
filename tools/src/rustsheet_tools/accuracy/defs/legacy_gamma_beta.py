"""The legacy chi-squared, gamma and beta names: CHIDIST, CHIINV, GAMMADIST, GAMMAINV, BETADIST and BETAINV.

Each returns exactly the same value or error as its modern equivalent on every paired cell, so each reuses that
function's Definition under its own name (one definition per concept, T-4104):

- statistics.md, "Legacy distribution names sweep (T-1047)": BETADIST = BETA.DIST cumulative, BETAINV = BETA.INV,
  CHIDIST = CHISQ.DIST.RT, CHIINV = CHISQ.INV.RT; df truncates; `CHIINV(1,..)` is 0 and p = 0 is `#NUM!`;
  `BETAINV(0,..)` and `BETAINV(1,..)` are `#NUM!`; BETADIST's x outside [A, B] is `#NUM!`.
- statistics.md, "Legacy distribution names sweep, second family (T-1048)": GAMMADIST = GAMMA.DIST and
  GAMMAINV = GAMMA.INV; `GAMMAINV(0,2,1)` is 0 and `GAMMAINV(1,2,1)` is `#NUM!`.

BETADIST has no cumulative flag: it is BETA.DIST's cdf, with the same optional bounds [A, B].
"""

from __future__ import annotations

import dataclasses
from typing import TYPE_CHECKING

from rustsheet_tools.accuracy.defs.beta import BETA_DIST, BETA_INV
from rustsheet_tools.accuracy.defs.gamma import CHISQ_INV_RT, GAMMA_INV
from rustsheet_tools.accuracy.defs.gamma_dist import CHISQ_DIST_RT, GAMMA_DIST
from rustsheet_tools.accuracy.model import Definition, register

if TYPE_CHECKING:
    from collections.abc import Sequence

    import mpmath

    from rustsheet_tools.accuracy.model import MathArg

FIRST_SOURCE = "statistics.md#legacy-distribution-names-sweep-t-1047"
SECOND_SOURCE = "statistics.md#legacy-distribution-names-sweep-second-family-t-1048"


def _alias(modern: Definition, function: str, source: str) -> Definition:
    """`modern` under the legacy name `function`, citing the sweep that pairs the two."""
    return dataclasses.replace(modern, function=function, sources=(source, *modern.sources))


def _with_cumulative(args: Sequence[MathArg]) -> list[MathArg]:
    """BETADIST's arguments as BETA.DIST's: TRUE inserted after the shapes, the bounds after it."""
    return [*args[:3], True, *args[3:]]


def _betadist(args: Sequence[MathArg]) -> mpmath.mpf:
    return BETA_DIST.value(_with_cumulative(args))


def _betadist_domain(args: Sequence[MathArg]) -> str | None:
    assert BETA_DIST.domain is not None
    return BETA_DIST.domain(_with_cumulative(args))


CHIDIST = register(_alias(CHISQ_DIST_RT, "CHIDIST", FIRST_SOURCE))
CHIINV = register(_alias(CHISQ_INV_RT, "CHIINV", FIRST_SOURCE))
GAMMADIST = register(_alias(GAMMA_DIST, "GAMMADIST", SECOND_SOURCE))
GAMMAINV = register(_alias(GAMMA_INV, "GAMMAINV", SECOND_SOURCE))
BETAINV = register(_alias(BETA_INV, "BETAINV", FIRST_SOURCE))
BETADIST = register(
    dataclasses.replace(
        BETA_DIST,
        function="BETADIST",
        params=tuple(p for p in BETA_DIST.params if p.name != "cumulative"),
        value=_betadist,
        domain=_betadist_domain,
        sources=(FIRST_SOURCE, *BETA_DIST.sources),
    )
)
