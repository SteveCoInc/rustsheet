"""PERCENTOF (Gate A slice 4i, T-3676, decision 0228).

PERCENTOF(data_subset, data_all) is SUM(data_subset) / SUM(data_all), each sum read as SUM reads it
(numbers.md "PERCENTOF: an error inside a reference or array is #NUM! (T-3653)": "reads its two
arguments as SUM does"). The math reuses SUM's definition (`defs/aggregate.py`, decision 0237) on the
two array literals' doubles, with the evidence of probes/percentof-snap-overflow-union (T-0416):
- each sum snaps like SUM's: `PERCENTOF({0.1,0.2,-0.3},1)` is 0 (D4), and a total that snaps to 0 is
  `#DIV/0!` (`PERCENTOF(A1:A2,A1:A3)`, D2), like a zero total (D10): a domain error;
- an overflowing sum is `#NUM!` (`PERCENTOF(F1:F2,F1:F2)`, D6): a domain error, and so is a quotient
  beyond the double range;
- `PERCENTOF(1E-300,1E+300)` is 0 (D14), an underflowing quotient flushed: the draws keep the quotient
  normal or zero, so that rule is not claimed for other values.
"""

from __future__ import annotations

import random
import sys
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, one_of
from rustsheet_tools.accuracy.defs.aggregate import SNAP_SOURCE, running_total, sum_value
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

SOURCE = "numbers.md#percentof-an-error-inside-a-reference-or-array-is-num-t-3653"
EVIDENCE = "probes/percentof-snap-overflow-union"


def _arrays(args: Sequence[MathArg]) -> tuple[tuple[mpmath.mpf, ...], tuple[mpmath.mpf, ...]]:
    subset, total = args
    assert isinstance(subset, tuple) and isinstance(total, tuple)
    return subset, total


def _sums(args: Sequence[MathArg]) -> tuple[mpmath.mpf, mpmath.mpf]:
    subset, total = _arrays(args)
    for xs in (subset, total):
        if not mpmath.isfinite(mpmath.mpf(running_total([float(x) for x in xs])[1])):
            raise DomainError("a running total is beyond the double range")
    return sum_value(subset), sum_value(total)


def _quotient(args: Sequence[MathArg]) -> mpmath.mpf:
    part, whole = _sums(args)
    if whole == 0:
        raise DomainError("the total is 0 (a snapped total too)")
    q = part / whole
    if abs(q) > sys.float_info.max:
        raise DomainError("the quotient is beyond the double range")
    return q


def _domain(args: Sequence[MathArg]) -> str | None:
    try:
        _quotient(args)
    except DomainError as why:
        return str(why)
    return None


# -- draws -------------------------------------------------------------------------------------


def _array(
    rng: random.Random, count: int, lo: float = 1e-3, hi: float = 1e6, negative: float = 0.2
) -> tuple[float, ...]:
    return tuple(
        round15(log_uniform(rng, lo, hi) * (-1 if rng.random() < negative else 1)) for _ in range(count)
    )


def _normal(args: list[Arg]) -> bool:
    """Whether the quotient is zero, normal or a domain error (underflow is not claimed)."""
    try:
        q = _quotient([tuple(mpmath.mpf(x) for x in a) for a in args if isinstance(a, tuple)])
    except DomainError:
        return True
    return bool(q == 0 or abs(q) >= sys.float_info.min)


def _point(rng: random.Random, lo: float, hi: float, sizes: tuple[int, int]) -> list[Arg]:
    while True:
        subset = _array(rng, rng.randint(*sizes), lo, hi)
        args: list[Arg] = [subset, (*subset, *_array(rng, rng.randint(*sizes), lo, hi))]
        if _normal(args):
            return args


def _cancelling(rng: random.Random) -> list[Arg]:
    """A near-cancelling total around SUM's snap bound (`defs/aggregate.py` `_cancelling`)."""
    head = [round15(rng.randint(1, 9999) / 10.0 ** rng.randint(0, 5)) for _ in range(rng.randint(1, 4))]
    _, total = running_total(head)
    last = -total + rng.randint(-3, 3) * abs(total) * 1e-15
    return [tuple(head), (*head, round15(last), round15(rng.uniform(1, 10)))]


def _long(rng: random.Random) -> list[Arg]:
    """80 subset values and a 160-value total: about 5,300 characters, inside Excel's 8192-character formula."""
    subset = _array(rng, 80)
    return [subset, (*subset, *_array(rng, 80))]


def _edge(rng: random.Random) -> list[Arg]:
    big = round15(rng.uniform(5e307, 9e307))
    decimals = (0.1, 0.2, -0.3)
    return one_of(
        rng,
        [
            lambda: [decimals, (round15(rng.uniform(1, 10)),)],  # a snapped subset sum: 0 (D4)
            lambda: [(0.1, 0.2), decimals],  # a snapped total: #DIV/0! (D2)
            lambda: [(big, big), (big, big)],  # an overflowing sum: #NUM! (D6)
            lambda: [(big,), (big, -big, big)],  # the same addends in an order that never overflows
            lambda: [(round15(log_uniform(rng, 1, 1e3)),), (0.0,)],  # a zero total: #DIV/0! (D10)
            lambda: [(-1.0,), (2.0,)],  # a negative share (D15)
            lambda: _long(rng),  # long arrays, inside the 8192-character formula limit
        ],
    )


DRAW = by_stratum(
    {
        "typical": lambda rng: _point(rng, 1e-3, 1e6, (1, 10)),
        "tail": _cancelling,
        "edge": _edge,
        "extreme": lambda rng: _point(rng, 1e-20, 1e20, (10, 60)),
        "log-uniform": lambda rng: _point(rng, 1e-150, 1e150, (1, 6)),
    },
    "PERCENTOF draws: two arrays by stratum (the total holds the subset in most); quotients normal or 0.",
)

PERCENTOF = register(
    Definition(
        function="PERCENTOF",
        params=(Param("data_subset", "array", source=SOURCE), Param("data_all", "array", source=SOURCE)),
        value=_quotient,
        domain=_domain,
        draw=DRAW,
        sources=(SOURCE, SNAP_SOURCE, EVIDENCE, "decision 0228"),
    )
)
