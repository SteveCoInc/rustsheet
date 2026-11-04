"""BESSELI, BESSELJ, BESSELK and BESSELY (Gate A, T-4083, decision 0270).

The math: mpmath `besseli`, `besselj`, `besselk` and `bessely` of the order truncated toward zero and of the
double of the written x, at the working precision. Excel's argument order is (x, n).

Sources (engineering.md):
- "BESSELJ / BESSELY (T-0900)": Excel is the Numerical Recipes rational approximations below x = 8 and the NR
  asymptotic branch from 8 (with three non-book coefficients), up to about 7E-5 relative off the math near x = 8
  to 10 for 1 <= n < x. That is inside decision 0188 R3's G = 1e-4, so Excel stays the target: a point is inside
  the envelope through Excel's 15 digits. Near a root of J or Y the relative error against the math is unbounded,
  so such points can pass G; they are listed as R3 candidates in the task's Outcome, never marked (decision 0188
  R3). A fractional order truncates; `BESSELY` of x <= 0 is `#NUM!`. Past the phase 134217727 the sine and cosine
  do not answer and BESSELJ and BESSELY are `#NUM!`; whether the cut itself answers is unprobed, so draws stay
  below `PHASE_DRAW` (well short of it) and the edge stratum draws the evidenced `#NUM!` side only.
- "BESSEL order cap": the engine runs no recurrence above order 100,000 (a directive 5 DEVIATION), where Excel's
  own cap is 10,000,000. Draws stay at or below `ORDER_CAP` so the table never scores the deviation.
  `BESSELJ(1E-300,100001)` is `#NUM!` from a Miller overflow at a tiny x, so tiny x is drawn with small orders only.
- "BESSELK underflow, BESSELI overflow and BIT shifts of 49 to 53 (T-2421)", "BESSELK near the bottom of the double
  range, BESSELI overflow point (T-2550)", "BESSELI overflow point (T-2613)": BESSELI is `#NUM!` for every order
  from |x| > 709.7 (finite at 709.7, `#NUM!` at every probed x above), so |x| > `I_CUT` is a domain error; BESSELK
  flushes to 0 below the smallest normal double, which Gate A's absolute floor of 1e-300 covers. BESSELK grows past
  the double range at a large order and a small x (`BESSELK(2.5,171)` overflows); such a point has no math value in
  the double range and is redrawn.
- BESSELK of x <= 0 is `#NUM!` (K is singular at 0 and undefined below); BESSELI and BESSELJ take any real x
  (I_n(-x) = (-1)^n I_n(x), J_n(-x) = (-1)^n J_n(x)). A negative order is `#NUM!` in all four.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, scalars
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

FIT_SOURCE = "engineering.md#besselj--bessely-t-0900"
CAP_SOURCE = "engineering.md#bessel-order-cap"
I_SOURCE = "engineering.md#besseli-overflow-point-t-2613"
K_SOURCE = "engineering.md#besselk-near-the-bottom-of-the-double-range-besseli-overflow-point-t-2550"
SPLIT = 8.0  # Excel's rational approximations below, the asymptotic branch from here
I_CUT = 709.7  # BESSELI is finite up to here and `#NUM!` above (T-2613)
K_FLUSH = 705.5  # BESSELK is 0 from about here (T-2421)
PHASE_DRAW = 1e8  # J and Y are drawn below this |x|; the phase cut is 134217727
PHASE_NUM = (
    134217730.0  # past this |x| both phases (x - 0.785398164, x - 2.356194491) are past the cut: `#NUM!`
)
ORDER_CAP = 100_000  # the engine's order cap (a DEVIATION above it)
TINY_ORDER = 4  # the largest order drawn at a tiny x (a Miller overflow at a large one)
TINY_X = 1e-6


def _order(args: Sequence[MathArg]) -> tuple[mpmath.mpf, int]:
    """(x, the order truncated toward zero); `#NUM!` for a negative order."""
    x, n = scalars(args)
    order = int(mpmath.floor(n)) if n >= 0 else int(mpmath.ceil(n))
    if order < 0:
        msg = "a negative order"
        raise DomainError(msg)
    return x, order


def _past_phase(x: mpmath.mpf) -> None:
    """J and Y are `#NUM!` once every phase their recurrence needs is past the cut (T-2547)."""
    if abs(x) > PHASE_NUM:
        msg = f"|x| past the phase cut ({PHASE_NUM:g})"
        raise DomainError(msg)


def _besselj(args: Sequence[MathArg]) -> mpmath.mpf:
    x, n = _order(args)
    _past_phase(x)
    return mpmath.besselj(n, x)


def _bessely(args: Sequence[MathArg]) -> mpmath.mpf:
    x, n = _order(args)
    if x <= 0:
        msg = "BESSELY of x <= 0"
        raise DomainError(msg)
    _past_phase(x)
    return mpmath.bessely(n, x)


def _besseli(args: Sequence[MathArg]) -> mpmath.mpf:
    x, n = _order(args)
    if abs(x) > I_CUT:
        msg = f"BESSELI past |x| = {I_CUT:g}"
        raise DomainError(msg)
    return mpmath.besseli(n, x)


def _besselk(args: Sequence[MathArg]) -> mpmath.mpf:
    x, n = _order(args)
    if x <= 0:
        msg = "BESSELK of x <= 0"
        raise DomainError(msg)
    return mpmath.besselk(n, x)


def _signed(rng: random.Random, x: float, signed: bool) -> float:
    return -x if signed and rng.random() < 0.5 else x


def _small_order(rng: random.Random) -> int:
    return rng.randrange(0, 11)


def _draws(
    x_hi: float, signed: bool, edges: Callable[[random.Random], list[Arg]]
) -> dict[str, Callable[..., list[Arg]]]:
    """The strata of one function: `x_hi` is the largest |x| drawn, `signed` whether a negative x is in the
    domain, `edges` the function's own evidenced breaks."""

    def typical(rng: random.Random) -> list[Arg]:
        return [_signed(rng, rng.uniform(0.05, 20.0), signed), _small_order(rng)]

    def tail(rng: random.Random) -> list[Arg]:  # past the asymptotic split, moderate orders
        return [_signed(rng, rng.uniform(SPLIT, min(x_hi, 1000.0)), signed), rng.randrange(0, 51)]

    def extreme(rng: random.Random) -> list[Arg]:  # large x or a large order, or a tiny x with a small order
        shape = rng.randrange(3)
        if shape == 0:
            return [_signed(rng, log_uniform(rng, 1000.0, x_hi), signed), _small_order(rng)]
        if shape == 1:
            n = int(log_uniform(rng, 100.0, ORDER_CAP))
            return [_signed(rng, log_uniform(rng, 1.0, min(x_hi, 2.0 * n)), signed), n]
        return [_signed(rng, log_uniform(rng, 1e-300, TINY_X), signed), rng.randrange(0, TINY_ORDER + 1)]

    def whole(rng: random.Random) -> list[Arg]:
        x = log_uniform(rng, TINY_X, x_hi)
        return [_signed(rng, x, signed), int(log_uniform(rng, 1.0, 1000.0)) - 1]

    return {"typical": typical, "tail": tail, "extreme": extreme, "log-uniform": whole, "edge": edges}


def _jy_edges(signed: bool) -> Callable[[random.Random], list[Arg]]:
    def edge(rng: random.Random) -> list[Arg]:
        shape = rng.randrange(5)
        if shape == 0:  # the split between the rational and the asymptotic branch
            x = rng.choice(
                (SPLIT, math.nextafter(SPLIT, 0.0), math.nextafter(SPLIT, 20.0), rng.uniform(7.5, 8.5))
            )
            return [_signed(rng, x, signed), rng.randrange(0, 4)]
        if shape == 1:  # the turning point n = x, where Miller's recurrence meets the upward one
            n = rng.choice((10, 50, 100, 1000, 10000))
            return [float(n + rng.choice((-1, 0, 1, 0.5))), n]
        if shape == 2:  # a fractional order truncates
            return [_signed(rng, rng.uniform(0.5, 30.0), signed), rng.uniform(0.0, 10.0)]
        if shape == 3:  # past the phase cut: `#NUM!` (T-2547)
            return [_signed(rng, rng.choice((1e10, 1e14, 2e14, 1e15)), signed), rng.randrange(0, 3)]
        return [
            _signed(rng, rng.choice((0.0, 1.0, 2.0, 5.0)) if signed else rng.choice((1.0, 2.0, 5.0)), signed),
            rng.randrange(0, 6),
        ]

    return edge


def _i_edge(rng: random.Random) -> list[Arg]:
    shape = rng.randrange(4)
    if shape == 0:  # just inside the overflow cut, finite (T-2613)
        return [_signed(rng, rng.choice((709.0, 709.5, I_CUT)), True), rng.randrange(0, 6)]
    if shape == 1:  # past it: `#NUM!`
        return [_signed(rng, rng.choice((709.78, 710.0, 713.0, 1e15)), True), rng.randrange(0, 6)]
    if shape == 2:
        return [rng.uniform(0.5, 30.0), rng.uniform(0.0, 10.0)]
    return [rng.choice((0.0, 1.0, 2.0, 5.0)), rng.randrange(0, 6)]


def _k_edge(rng: random.Random) -> list[Arg]:
    shape = rng.randrange(4)
    if shape == 0:  # around the underflow flush (T-2421, T-2550)
        return [rng.choice((700.0, 702.0, 705.0, K_FLUSH, 707.0, 709.0, 1000.0)), rng.randrange(0, 6)]
    if shape == 1:  # the large-order overflow at a small x
        return [2.5, rng.choice((150, 160, 170))]
    if shape == 2:
        return [rng.uniform(0.5, 30.0), rng.uniform(0.0, 10.0)]
    return [rng.choice((1.0, 2.0, 5.0)), rng.randrange(0, 6)]


def _definition(
    function: str,
    value: Callable[[Sequence[MathArg]], mpmath.mpf],
    *,
    x_hi: float,
    signed: bool,
    edge: Callable[[random.Random], list[Arg]],
    sources: tuple[str, ...],
) -> Definition:
    draw = by_stratum(_draws(x_hi, signed, edge), f"{function}: (x, n) by stratum.")
    return register(
        Definition(
            function=function,
            params=(
                Param("x", "real", source=sources[0]),
                Param("n", "real", source=CAP_SOURCE),
            ),
            value=value,
            draw=draw,
            sources=sources,
        )
    )


BESSELJ = _definition(
    "BESSELJ", _besselj, x_hi=PHASE_DRAW, signed=True, edge=_jy_edges(True), sources=(FIT_SOURCE, CAP_SOURCE)
)
BESSELY = _definition(
    "BESSELY",
    _bessely,
    x_hi=PHASE_DRAW,
    signed=False,
    edge=_jy_edges(False),
    sources=(FIT_SOURCE, CAP_SOURCE),
)
BESSELI = _definition(
    "BESSELI", _besseli, x_hi=I_CUT, signed=True, edge=_i_edge, sources=(I_SOURCE, CAP_SOURCE)
)
BESSELK = _definition(
    "BESSELK", _besselk, x_hi=1000.0, signed=False, edge=_k_edge, sources=(K_SOURCE, CAP_SOURCE)
)
