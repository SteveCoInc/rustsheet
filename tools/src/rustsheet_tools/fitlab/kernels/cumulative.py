"""CUMPRINC and CUMIPMT: the geometric-principal selectors of financial/cumulative.rs (T-3524).

The engine takes a principal as one geometric sum when the leading exponent `(nper-end+1)·ln(1+rate)` is at least
`GEOMETRIC_MIN_LEAD` over a short range (at most 32 periods), and the per-period sum or the closed form below it;
before T-3524 a longer range had its own cut, `LONG_GEOMETRIC_MIN_LEAD` (3.6), which this fit removed. CUMIPMT is PMT
times the period count less that principal, so the cuts reach it too. Decision 0188 R4 asks whether such tuned
cut-points generalize: the candidates are engine builds (`rustsheet-fitlab cumulative --build NAME=PATH`, made with
docs/research/t3524/*.patch against the engine before T-3524) and the correctly rounded reference.

- `cumulative`: the CUMPRINC and CUMIPMT cells that existed when the cuts were fitted (T-3067's probes and every
  other probes and gen-m9 case with a literal call).
- `cumulative-fresh`: the fresh sweep drawn after (probes-numeric/t3524-cumprinc-lead-fresh-sweep,
  docs/research/t3524/gen_sweep.py), rates 1E-4..2.
- `cumulative-tiny`: the second fresh sweep, rates 1E-12..1E-4 (probes-numeric/t3524-cumprinc-tiny-rate-sweep,
  `gen_sweep.py`).
"""

from __future__ import annotations

import re
from collections.abc import Sequence

import mpmath

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.holdout import REFERENCE
from rustsheet_tools.fitlab.score import Candidate
from rustsheet_tools.fitlab.sweep import Cell, numeric_arg

PATTERN = r"^=CUM(?:PRINC|IPMT)\((.*)\)$"
_FUNCTION = re.compile(r"^=(CUMPRINC|CUMIPMT)\(")


def exact_mp(function: str, args: Sequence[float]) -> mpmath.mpf:
    """The exact sum for valid arguments (the engine's own checks decide errors; this is only the number), at the
    caller's precision: the payment `pmt = pv·r/(1 - v^nper)`, over `1+r` for type 1, period k's principal
    `pmt·v^(nper-k+1)` (type 1's period 1 is the whole payment), the principal from `a` to `b` a geometric sum, and
    the interest the payments less it. Both are negative for a positive pv, as Excel's are. `args` are the call's
    six: rate, nper, pv, start_period, end_period and type."""
    rate, nper, pv, start, end, period_type = args
    period_start = period_type == 1.0
    r, n, p = mpmath.mpf(rate), mpmath.mpf(nper), mpmath.mpf(pv)
    a, b = mpmath.ceil(mpmath.mpf(start)), mpmath.floor(mpmath.mpf(end))
    log_growth = mpmath.log1p(r)
    payment = p * r / -mpmath.expm1(-n * log_growth)
    if period_start:
        payment /= 1 + r
    first = max(a, mpmath.mpf(2)) if period_start else a
    principal = mpmath.mpf(0)
    if b >= first:
        count = b - first + 1
        # the sum over k = first..b of v^(n-k+1) is v^(n-b+1)·(1 - v^count)/(1 - v)
        principal = (
            payment
            * mpmath.exp(-(n - b + 1) * log_growth)
            * -mpmath.expm1(-count * log_growth)
            / -mpmath.expm1(-log_growth)
        )
    if period_start and a == 1:
        principal += payment
    if function == "CUMPRINC":
        return -principal
    return -(payment * (b - a + 1) - principal)


def correctly_rounded(cell: Cell) -> float:
    """The exact value of the cell's call, rounded once to a double."""
    m = _FUNCTION.match(cell.formula)
    if m is None or len(cell.args) != 6:
        msg = f"not a six-argument CUMPRINC or CUMIPMT call: {cell.formula}"
        raise TypeError(msg)
    with mpmath.workdps(60):
        return float(exact_mp(m[1], [numeric_arg(a) for a in cell.args]))


def candidates() -> list[Candidate]:
    return [Candidate(REFERENCE, correctly_rounded)]


_NOTES = [
    "Engine builds are the selector's candidates: --build NAME=PATH; docs/research/t3524/*.patch make them.",
]

_OLD_CASES = [
    "probes/fin-argument-modes",
    "probes/fin-cumipmt-gate-edges",
    "probes/fin-cumipmt-overflow-and-forms",
    "probes/fin-cumipmt-period-bound",
    "probes/fin-cumipmt-tiny-rate-structure",
    "probes/fin-cumprinc-dyadic-rates",
    "probes/fin-cumprinc-form-boundaries",
    "probes/fin-cumprinc-near-1e15",
    "probes/fin-cumprinc-single-period-rate-scan",
    "probes/fin-cumprinc-underflow-ends",
    "probes/fin-remaining-later-argument-ranges",
    "probes/t3179-cumipmt-ipmt-sum",
    "gen-m9/0a957119475760c5",
    "gen-m9/1b60bf99d71424e7",
    "gen-m9/1cd3096e3e87c965",
    "gen-m9/1dce4774f5c0e736",
    "gen-m9/30e20ba3302fea8b",
    "gen-m9/31474c20fc1dafd6",
    "gen-m9/3aa168259335be54",
    "gen-m9/3c06f9298f65a3ba",
    "gen-m9/5422bdbf173fc233",
    "gen-m9/5c6a23c4f6597f1c",
    "gen-m9/5ded7594e597e31e",
    "gen-m9/6376e27be7a05871",
    "gen-m9/67a4c03afd63f988",
    "gen-m9/762cd0e4ec9a7aa9",
    "gen-m9/79d87fc527927626",
    "gen-m9/7d2979fba2c95d64",
    "gen-m9/81bcdc0bec0c8e6b",
    "gen-m9/89f0aee953a55a45",
    "gen-m9/9a1fa5366bc50918",
    "gen-m9/9d9ceba257f8001b",
    "gen-m9/a2e51f48717af2a1",
    "gen-m9/a91bad4e84f49739",
    "gen-m9/abb7d565af89b849",
    "gen-m9/b6a02893a8d6b151",
    "gen-m9/b72d952c5f52cb4d",
    "gen-m9/b73c6cc96e093f03",
    "gen-m9/ba4ddb817a8e700e",
    "gen-m9/bc04c1f478c9896c",
    "gen-m9/c0681a8b150ce301",
    "gen-m9/d0854366830dcdee",
    "gen-m9/d912671baa1364aa",
    "gen-m9/e168be7f6d3f84c2",
]


def _fit(name: str, description: str, cases: list[str]) -> Fit:
    """One CUMPRINC/CUMIPMT fit: the shared pattern, reference and notes."""
    return Fit(
        name=name, description=description, cases=cases, pattern=PATTERN, candidates=candidates, notes=_NOTES
    )


FITS = [
    _fit(
        "cumulative",
        "CUMPRINC/CUMIPMT on the cells that existed when T-3067 fitted the geometric-principal cuts",
        _OLD_CASES,
    ),
    _fit(
        "cumulative-fresh",
        "CUMPRINC/CUMIPMT on T-3524's fresh sweep, the leading exponent log-uniform (decision 0188 R4.2)",
        ["probes-numeric/t3524-cumprinc-lead-fresh-sweep"],
    ),
    _fit(
        "cumulative-tiny",
        "CUMPRINC/CUMIPMT on T-3524's second fresh sweep, rates 1E-12..1E-4 (decision 0188 R4.2)",
        ["probes-numeric/t3524-cumprinc-tiny-rate-sweep"],
    ),
]
