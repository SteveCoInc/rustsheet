"""Path-invariant transforms of an IRR flow set (docs/research/irr-2026-11-07/plan.md Phase 0 item 4, T-4834).

Each transform maps flows to flows with the same roots, so a probe of the original and the transformed set
isolates one mechanism:
- `scale_pow2`: x2^k is exact in binary, so a scale-invariant method (the secant's step is a ratio of NPVs) walks
  bit-identical iterates, while an absolute test (|NPV| < 1e-7, 1e-4) sees a 2^k-times larger residual. A type
  change along a 2^k ladder isolates an absolute threshold.
- `negate`: the same roots with NPV's sign flipped: is the path symmetric in that sign?
- `pad_leading`: leading zero flows, which IRR drops, so they must be invisible.
- `pad_trailing`: trailing zero flows change n (and the noise floor) without changing the root.
"""

from __future__ import annotations

import math


def scale_pow2(flows: list[float], k: int) -> list[float]:
    """Every flow times 2^k, exactly (`math.ldexp`); ValueError if a flow would overflow or lose bits."""
    try:
        out = [math.ldexp(c, k) for c in flows]
    except OverflowError:
        out = [math.inf]
    if len(out) != len(flows) or any(
        math.ldexp(o, -k) != c or not math.isfinite(o) for c, o in zip(flows, out, strict=True)
    ):
        msg = f"2^{k} scaling is not exact on these flows"
        raise ValueError(msg)
    return out


def negate(flows: list[float]) -> list[float]:
    """Every flow with its sign flipped."""
    return [-c for c in flows]


def pad_leading(flows: list[float], n: int) -> list[float]:
    """`n` zero flows before the first."""
    return [0.0] * n + flows


def pad_trailing(flows: list[float], n: int) -> list[float]:
    """`n` zero flows after the last."""
    return flows + [0.0] * n
