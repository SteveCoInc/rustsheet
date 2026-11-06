"""The stratified sampler of Gate A tables (SPEC-0029, "Strata"; decision 0188 R2).

A batch's points are split over five strata: typical 40%, tail 15% and domain edge 15% (together
the 30% of tails and edges), extreme-but-valid 20%, log-uniform 10%. Everything is drawn from one
`random.Random` seeded from `sha256(f"{FUNCTION}/{batch}")`: no clock, no environment, so the same
function, batch and tool version give the same points.
"""

from __future__ import annotations

import dataclasses
import hashlib
import math
import random
from collections.abc import Callable
from decimal import Decimal

import mpmath

from rustsheet_tools.accuracy.literal import (
    COMPLEX_SUFFIXES,
    MAX_LITERAL,
    MIN_LITERAL,
    Arg,
    complex_text,
    neighbour,
    round15,
)
from rustsheet_tools.accuracy.model import Definition, MathArg, Param
from rustsheet_tools.fitlab.sweep import read_complex_arg

STRATA = ("typical", "tail", "edge", "extreme", "log-uniform")
AIMED = "aimed"  # a definition's own region draw (`Definition.aimed`), never in the default split (T-4059)
SHARES = (0.40, 0.15, 0.15, 0.20, 0.10)
TAIL_DISTANCE = (1e-15, 1e-1)  # relative distance of a tail point from a finite domain bound
TAIL_PROBABILITY = (1e-300, 1e-1)  # a distribution's tail probability, drawn log-uniform
TAIL_MAP_DPS = 40  # enough for a 15-digit x


def seed(function: str, batch: str) -> int:
    """The batch's seed: `int(sha256(f"{FUNCTION}/{batch}"), 16) % 2**63`."""
    return int(hashlib.sha256(f"{function}/{batch}".encode()).hexdigest(), 16) % 2**63


def point_count(definition: Definition) -> int:
    """A first batch's size: 120 points up to two numeric parameters, 200 for three or more,
    300 for a solver."""
    if definition.solver:
        return 300
    return 120 if definition.numeric_arity() <= 2 else 200


def stratum_counts(count: int) -> dict[str, int]:
    """`count` split over the strata by `SHARES`, largest remainder first (ties in stratum order),
    so the counts sum to `count`: 120 → 48/18/18/24/12."""
    raw = [count * share for share in SHARES]
    counts = [math.floor(r) for r in raw]
    order = sorted(range(len(STRATA)), key=lambda i: (-(raw[i] - counts[i]), i))
    for i in order[: count - sum(counts)]:
        counts[i] += 1
    return dict(zip(STRATA, counts, strict=True))


def math_arg(param: Param, a: Arg) -> MathArg:
    """The math's view of a drawn argument: the exact double (or integer), the logical, a complex
    number text's parts as Excel reads them (an `mpc` of `read_complex_arg`'s parts, each cut to 15
    significant digits; ValueError for text that is no complex number) or a
    fixed-list text itself."""
    if param.kind == "bool" or isinstance(a, bool):
        return bool(a)
    if isinstance(a, str):
        if not param.is_complex:
            return a
        re, im = read_complex_arg(a)
        return mpmath.mpc(re, im)
    if param.kind == "matrix" and isinstance(a, tuple):
        return tuple(tuple(mpmath.mpf(x) for x in row) for row in a if isinstance(row, tuple))
    if isinstance(a, tuple):
        return tuple(mpmath.mpf(x) for x in a if not isinstance(x, tuple))
    return mpmath.mpf(a)


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    """A draw log-uniform over [lo, hi], 0 < lo ≤ hi."""
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def decimal_literal(rng: random.Random, digits: int, exponent: int, sign: float) -> float:
    """A number with `digits` significant digits (the last one nonzero) and decimal exponent
    `exponent`: a literal whose written decimal a decimal-rounding definition reads."""
    mantissa = rng.randrange(10 ** (digits - 1), 10**digits)
    if mantissa % 10 == 0:
        mantissa += rng.randrange(1, 10)
    return sign * float(Decimal(mantissa).scaleb(exponent - digits + 1))


def random_sign(rng: random.Random) -> float:
    """+1.0 or -1.0 with equal odds."""
    return 1.0 if rng.random() < 0.5 else -1.0


def _spread(rng: random.Random, lo: float, hi: float) -> float:
    """Uniform over [lo, hi], or log-uniform when the range lies on one side of 0 and spans more
    than two decades."""
    if lo > 0 and hi / lo > 100:
        return log_uniform(rng, lo, hi)
    if hi < 0 and lo / hi > 100:
        return -log_uniform(rng, -hi, -lo)
    return rng.uniform(lo, hi)


class Sampler:
    """Draws the points of one batch for `definition` from `rng`."""

    def __init__(self, definition: Definition, rng: random.Random) -> None:
        self.definition = definition
        self.rng = rng
        self.edges = self._edge_candidates()
        # One suffix per point for every complex text of the point (`literal.COMPLEX_SUFFIXES`); drawn
        # only for a definition with one, so no other definition's draws move. `suffix` is meaningful
        # only inside `draw()`, which sets it before the point's parameters are drawn.
        self.has_complex = any(p.is_complex for p in definition.params)
        self.suffix = COMPLEX_SUFFIXES[0]

    # -- per-parameter draws --------------------------------------------------------------

    def finish(self, param: Param, x: Arg) -> Arg:
        """A drawn value as a writable literal of the parameter's kind (an array element by
        element, a matrix row by row: integers stay integers, reals take 15 digits)."""
        if param.kind == "matrix" and isinstance(x, tuple):
            return tuple(self._elements(row) for row in x if isinstance(row, tuple))
        if param.kind == "text":
            if not isinstance(x, str):
                msg = f"{param.name}: {x!r} is not a text"
                raise TypeError(msg)
            return x
        if isinstance(x, tuple):
            return self._elements(x)
        if param.kind == "bool":
            return bool(x)
        if param.kind == "int":
            return int(round15(float(round(float(x)))))
        return round15(float(x))

    @staticmethod
    def _elements(x: tuple[object, ...]) -> tuple[float | int, ...]:
        out: list[float | int] = []
        for e in x:
            if isinstance(e, bool) or not isinstance(e, (int, float)):
                msg = f"array element {e!r} is not a number"
                raise TypeError(msg)
            out.append(e if isinstance(e, int) else round15(e))
        return tuple(out)

    def inside(self, param: Param, a: Arg) -> Arg:
        """`a`, or the nearest literal inside the domain when rounding left it."""
        if param.kind in ("bool", "array", "matrix", "text") or isinstance(a, (bool, str, tuple)):
            return a
        if param.domain.contains(float(a)):
            return a
        d = param.domain
        lower = float(a) < d.lo or (float(a) == d.lo and d.lo_open)
        bound = d.lo if lower else d.hi
        if param.kind == "int":
            step = 1 if lower else -1
            start = math.floor(bound) if lower else math.ceil(bound)
            value = start + step if (bound == start and (d.lo_open if lower else d.hi_open)) else start
            if not d.contains(float(value)):
                value += step
            return int(value)
        return neighbour(bound, up=lower)

    def text(self, param: Param, draw: Callable[[Param], Arg]) -> str:
        """A text parameter's draw: one of its `choices`, or a complex number's text whose real and
        imaginary parts are two draws of `draw` (`typical`, `extreme`, `whole_domain`) over the
        parameter as a real, written with the point's suffix."""
        if param.choices:
            return self.rng.choice(param.choices)
        part = dataclasses.replace(param, kind="real")
        re, im = draw(part), draw(part)
        if isinstance(re, (bool, str, tuple)) or isinstance(im, (bool, str, tuple)):
            msg = f"{param.name}: a complex part is not a real"
            raise TypeError(msg)
        return complex_text(float(re), float(im), self.suffix)

    def typical(self, param: Param) -> Arg:
        """A draw over the parameter's typical range."""
        if param.kind == "bool":
            return self.rng.random() < 0.5
        if param.kind == "text":
            return self.text(param, self.typical)
        lo, hi = param.typical
        return self.inside(param, self.finish(param, _spread(self.rng, lo, hi)))

    def extreme(self, param: Param) -> Arg:
        """A draw over the parameter's extreme magnitude range, signed when the domain allows."""
        if param.kind == "text":
            return self.text(param, self.extreme)
        if param.extreme is None:
            return self.typical(param)
        x = _spread(self.rng, *param.extreme)
        if param.domain.lo < 0 and self.rng.random() < 0.5:
            x = -x
        return self.inside(param, self.finish(param, x))

    def whole_domain(self, param: Param) -> Arg:
        """A draw log-uniform over the parameter's whole valid domain (sign uniform if signed)."""
        if param.kind == "bool":
            return self.rng.random() < 0.5
        if param.kind == "text":
            return self.text(param, self.whole_domain)
        floor = 1.0 if param.kind == "int" else MIN_LITERAL
        cap = MAX_LITERAL if param.draw_hi is None else param.draw_hi
        lo = max(param.domain.lo, -cap)
        hi = min(param.domain.hi, cap)
        negative = hi <= 0 or (lo < 0 and self.rng.random() < 0.5)
        top = -lo if negative else hi
        bottom = max(floor, -hi if negative else lo, floor)
        x = top if top <= bottom else log_uniform(self.rng, bottom, top)
        return self.inside(param, self.finish(param, -x if negative else x))

    # -- strata --------------------------------------------------------------------------

    def draw(self, stratum: str, index: int) -> list[Arg]:
        """Point number `index` of `stratum`."""
        d = self.definition
        if self.has_complex:
            self.suffix = self.rng.choice(COMPLEX_SUFFIXES)
        if stratum == AIMED or d.draw is not None:
            drawn = self._whole(stratum)
            return [self.finish(p, a) for p, a in zip(d.params_for(len(drawn)), drawn, strict=True)]
        if stratum == "tail":
            return self._tail(index)
        if stratum == "edge":
            return self._edge(index)
        if stratum == "extreme":
            return self._extreme()
        if stratum == "log-uniform":
            return [self.whole_domain(p) for p in d.params]
        return [self.typical(p) for p in d.params]

    def _whole(self, stratum: str) -> list[Arg]:
        """A whole point from the definition's own draw: `aimed`, or `draw` for any other stratum."""
        d = self.definition
        if stratum == AIMED:
            if d.aimed is None:
                msg = f"{d.function} has no aimed draw"
                raise ValueError(msg)
            return d.aimed(self.rng)
        if d.draw is None:
            msg = f"{d.function} has no draw"
            raise ValueError(msg)
        return d.draw(stratum, self.rng)

    def _extreme(self) -> list[Arg]:
        params = self.definition.params
        candidates = [i for i, p in enumerate(params) if p.extreme is not None]
        chosen = {i for i in candidates if self.rng.random() < 0.5}
        if candidates and not chosen:
            chosen = {self.rng.choice(candidates)}
        return [self.extreme(p) if i in chosen else self.typical(p) for i, p in enumerate(params)]

    def _tail_targets(self) -> list[int | None]:
        """Round-robin targets of tail points: None for the tail map, else a parameter index."""
        d = self.definition
        targets: list[int | None] = [None] if d.tail is not None else []
        targets += [i for i, p in enumerate(d.params) if p.kind == "real" and p.finite_bounds()]
        return targets

    def _tail(self, index: int) -> list[Arg]:
        params = self.definition.params
        args = [self.typical(p) for p in params]
        targets = self._tail_targets()
        if not targets:
            return args
        target = targets[index % len(targets)]
        if target is None:
            return self._tail_mapped(args)
        param = params[target]
        bounds = param.finite_bounds()
        bound, _, lower = bounds[(index // len(targets)) % len(bounds)]
        distance = log_uniform(self.rng, *TAIL_DISTANCE) * max(abs(bound), 1.0)
        args[target] = self.inside(param, self.finish(param, bound + distance if lower else bound - distance))
        return args

    def _tail_mapped(self, args: list[Arg]) -> list[Arg]:
        """Sets the tail parameter (a distribution's x) to the quantile of a log-uniform tail
        probability, in the lower or upper tail, so a cdf point is really in the tail."""
        d = self.definition
        assert d.tail is not None and d.tail_param is not None
        target = next(i for i, p in enumerate(d.params) if p.name == d.tail_param)
        q = mpmath.mpf(log_uniform(self.rng, *TAIL_PROBABILITY))
        upper = self.rng.random() < 0.5
        others = [math_arg(p, a) for p, a in zip(d.params, args, strict=True)]
        try:
            with mpmath.workdps(TAIL_MAP_DPS):
                x = d.tail(q, upper, others)
        except (ValueError, ZeroDivisionError, OverflowError):
            return args  # the typical draw stands (e.g. the other arguments are invalid)
        param = d.params[target]
        if not mpmath.isfinite(x) or abs(x) > MAX_LITERAL:
            return args
        args[target] = self.inside(param, self.finish(param, float(x)))
        return args

    def _edge_candidates(self) -> list[tuple[int, Arg]]:
        """Every (parameter index, value) edge: each finite bound itself, its nearest literal
        inside, the nearest invalid literal outside, and the parameter's extra `edges`."""
        out: list[tuple[int, Arg]] = []
        for i, p in enumerate(self.definition.params):
            if p.kind in ("bool", "text"):  # a text's draws are its choices or its parts' ranges
                continue
            values: list[float] = []
            for bound, is_open, lower in p.finite_bounds():
                if p.kind == "int":
                    step = 1.0 if lower else -1.0
                    values += [bound, bound + step, bound - step]
                else:
                    values += [bound, neighbour(bound, up=lower)]
                    if not is_open:
                        values.append(neighbour(bound, up=not lower))
            values += list(p.edges)
            for v in values:
                candidate = self.finish(p, v)
                if (i, candidate) not in out:
                    out.append((i, candidate))
        return out

    def _edge(self, index: int) -> list[Arg]:
        args = [self.typical(p) for p in self.definition.params]
        if self.edges:
            target, value = self.edges[index % len(self.edges)]
            args[target] = value
        return args
