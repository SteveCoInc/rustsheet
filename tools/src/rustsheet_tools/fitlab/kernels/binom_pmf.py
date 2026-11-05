"""BINOM.DIST and NEGBINOM.DIST pmfs: Loader's kernel against the exact value (T-2146, decision 0188 R4).

T-2146 moved both pmfs from T-0927's exact double-double walk (about the correctly rounded value)
onto Excel's Loader kernel (`loader.rs` `excel_dbinom_exponent`) everywhere off the closed forms.
The engine pseudo-candidate is that build; pass the walk as a second build
(`--build walk=<main's rustsheet-eval>`). The correctly rounded pmf (60-digit mpmath) is the
reference. The margin is the default 5 points, stated before scoring. Results: statistics.md
"BINOM.DIST and NEGBINOM.DIST pmfs take Loader's kernel everywhere".
"""

from __future__ import annotations

import mpmath

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.holdout import REFERENCE
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell, numeric_arg

_CASES = [
    "probes-numeric/t2513-r3-binom-pmf-a",
    "probes-numeric/t2513-r3-binom-pmf-n",
    "probes-numeric/t2513-r3-binom-pmf-hold",
    "probes-numeric/t2513-verify-r3-binom-pmf-h",
    "probes-numeric/t2145-binomial-sweep",
    "probes-numeric/t2145-negbinom-sweep",
]


def _is_pmf(cell: Cell) -> bool:
    return len(cell.args) == 4 and cell.args[3] in (False, 0, 0.0)


def binom_cr(*args: Arg) -> float:
    """C(n, k) p^k (1 - p)^(n - k) at 60 digits, rounded once to a double."""
    k, n, p = (int(numeric_arg(a)) if i < 2 else numeric_arg(a) for i, a in enumerate(args[:3]))
    with mpmath.workdps(60):
        mp = mpmath.mpf(p)
        return float(mpmath.binomial(n, k) * mp**k * (1 - mp) ** (n - k))


def negbinom_cr(*args: Arg) -> float:
    """C(f + r - 1, f) p^r (1 - p)^f at 60 digits, rounded once to a double."""
    f, r, p = (int(numeric_arg(a)) if i < 2 else numeric_arg(a) for i, a in enumerate(args[:3]))
    with mpmath.workdps(60):
        mp = mpmath.mpf(p)
        return float(mpmath.binomial(f + r - 1, f) * mp**r * (1 - mp) ** f)


def _binom_candidates() -> list[Candidate]:
    return [candidate(REFERENCE, binom_cr, "reference")]


def _negbinom_candidates() -> list[Candidate]:
    return [candidate(REFERENCE, negbinom_cr, "reference")]


def _binom_interior(cell: Cell) -> bool:
    if not _is_pmf(cell):
        return False
    k, n = numeric_arg(cell.args[0]), numeric_arg(cell.args[1])
    return 0 < k < n


FITS = [
    Fit(
        name="binom-pmf",
        description="BINOM.DIST interior pmf (T-2146): engine (Loader) vs walk build vs correctly rounded",
        cases=_CASES,
        pattern=r"^=BINOM\.DIST\((.*)\)$",
        candidates=_binom_candidates,
        where=_binom_interior,
    ),
    Fit(
        name="negbinom-pmf",
        description="NEGBINOM.DIST pmf (T-2146): engine (Loader) vs walk build vs correctly rounded",
        cases=_CASES,
        pattern=r"^=NEGBINOM\.DIST\((.*)\)$",
        candidates=_negbinom_candidates,
        where=_is_pmf,
    ),
]
