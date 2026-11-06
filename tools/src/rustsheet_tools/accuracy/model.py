"""What a Gate A definition is (SPEC-0029, "The generator and the sampler").

A `Definition` names a function's parameters in the catalog's order, each with its valid domain,
typical range and extreme range (each cited), and the mpmath function that computes its value at
the working precision. Definitions register themselves in `REGISTRY` from `defs/<family>.py`.
"""

from __future__ import annotations

import contextlib
import math
import random
from collections.abc import Callable, Iterator, Sequence
from dataclasses import dataclass, field
from typing import Literal, TypeAlias

import mpmath

from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg

Kind = Literal["real", "int", "bool", "array", "matrix", "text"]
MathArg: TypeAlias = (
    "mpmath.mpf | mpmath.mpc | bool | str | tuple[mpmath.mpf, ...] | tuple[tuple[mpmath.mpf, ...], ...]"
)
"""An argument as the math sees it: the exact double (or integer) of the literal, a logical, a complex
number text's parsed parts (an `mpc` of the two exact doubles), a fixed-list text itself, an array
literal's elements, or a matrix literal's rows."""


class Unfit(str):
    """Why an eval cell's argument does not fit its parameter (`eval_math.fit_arg`): a reason, told
    apart from a fitted text argument, which is a plain str (T-4084)."""

    __slots__ = ()


class DomainError(ValueError):
    """The point is outside the function's domain: Excel's error code is the target."""


class NoValueError(ValueError):
    """The point has no single math value to record (an overflow, or several roots with no
    evidenced root choice): the sampler redraws or drops it."""


class OverBudgetError(Exception):
    """A point's math spent more than its work budget (`work_budget`): the generator drops it. The
    budget counts series terms, not seconds, so the same point is dropped on every machine (T-3680)."""


_WORK: dict[str, int | None] = {"spent": 0, "limit": None}


@contextlib.contextmanager
def work_budget(limit: int | None) -> Iterator[None]:
    """Counts the `spend` calls of the block against `limit` (None: count only)."""
    saved = dict(_WORK)
    _WORK.update(spent=0, limit=limit)
    try:
        yield
    finally:
        _WORK.update(saved)


def spend(units: int = 1) -> None:
    """Spends `units` of the current work budget; OverBudgetError once it is exceeded. A summed
    expansion calls it once per term, so a slow point stops at the same term everywhere."""
    spent = (_WORK["spent"] or 0) + units
    _WORK["spent"] = spent
    limit = _WORK["limit"]
    if limit is not None and spent > limit:
        msg = f"over the work budget of {limit} terms"
        raise OverBudgetError(msg)


def work_spent() -> int:
    """The units spent so far in the current `work_budget` block."""
    return _WORK["spent"] or 0


@dataclass(frozen=True)
class Interval:
    """A real interval with open or closed ends; infinite ends are open."""

    lo: float = -math.inf
    hi: float = math.inf
    lo_open: bool = True
    hi_open: bool = True

    def contains(self, x: float) -> bool:
        """Whether `x` is inside."""
        above = x > self.lo or (x == self.lo and not self.lo_open)
        below = x < self.hi or (x == self.hi and not self.hi_open)
        return above and below

    def describe(self) -> str:
        """`[1, 1E+10]`, `(0, 1)`: the interval in the usual notation."""
        return f"{'(' if self.lo_open else '['}{self.lo:g}, {self.hi:g}{')' if self.hi_open else ']'}"


@dataclass(frozen=True)
class Param:
    """One parameter: its kind, valid domain, typical and extreme ranges and their source.

    `typical` and `extreme` are inclusive draw ranges inside the domain; a range spanning more
    than two decades is drawn log-uniform. `extreme` is a magnitude range: for a parameter whose
    domain has negative values the sign is drawn uniformly. `edges` are extra values worth a point
    beside the domain bounds (a documented limit such as df = 1E10 and its neighbour). `draw_hi` caps
    the magnitude of whole-domain draws where the math cannot evaluate the domain's far end (a
    50-digit incomplete beta at a count of 1E+300) or the oracle cannot hold a point (a huge
    negative POISSON.DIST mean); unlike a domain bound it claims nothing about Excel, so it gets no
    edge points. An `optional` parameter (trailing ones only) may be left out of a call, as ERF's
    `upper_limit`: a point then has fewer arguments, and only a definition's own `draw` omits one.
    `truncates` cites the semantics entry showing that Excel truncates a non-integer given to this
    `int` parameter toward zero (F.DIST's, T.DIST's and CHISQ.INV.RT's df); without one an eval cell that passes a
    non-integer gets no math value, since whether Excel truncates is per function.
    A `text` parameter (T-4084) is written as a string literal. With `choices` (distinct, non-empty
    texts; an empty tuple is refused) it is drawn from that
    fixed list (CONVERT's unit names, engineering.md "CONVERT unit names") and the math receives the
    text; without (None), it is a complex number's text `a+bi` / `a-bj` whose two parts are drawn as reals
    over this parameter's domain and ranges (engineering.md "Leading plus in IM*..."), and the math
    receives the parsed parts as an `mpc`.
    """

    name: str
    kind: Kind
    domain: Interval = field(default_factory=Interval)
    typical: tuple[float, float] = (0.0, 1.0)
    extreme: tuple[float, float] | None = None
    edges: tuple[float, ...] = ()
    source: str = ""
    draw_hi: float | None = None
    optional: bool = False
    truncates: str = ""
    choices: tuple[str, ...] | None = None

    def __post_init__(self) -> None:
        if self.choices is not None and self.kind != "text":
            msg = f"{self.name}: only a text parameter takes choices"
            raise ValueError(msg)
        if self.choices is not None and self.kind == "text":
            if not self.choices:
                msg = f"{self.name}: empty choices (leave them out for a complex number)"
                raise ValueError(msg)
            if "" in self.choices or len(set(self.choices)) != len(self.choices):
                msg = f"{self.name}: choices must be distinct, non-empty texts"
                raise ValueError(msg)

    @property
    def is_complex(self) -> bool:
        """Whether this is a complex number's text (a `text` parameter without `choices`)."""
        return self.kind == "text" and self.choices is None

    def finite_bounds(self) -> list[tuple[float, bool, bool]]:
        """The finite domain bounds as (value, open, is_lower)."""
        out: list[tuple[float, bool, bool]] = []
        if math.isfinite(self.domain.lo):
            out.append((self.domain.lo, self.domain.lo_open, True))
        if math.isfinite(self.domain.hi) and self.domain.hi < MAX_LITERAL:
            out.append((self.domain.hi, self.domain.hi_open, False))
        return out


TailMap = Callable[[mpmath.mpf, bool, Sequence[MathArg]], mpmath.mpf]
"""(tail probability q, upper tail?, the other arguments) → the x whose cdf is q (or 1 - q)."""
Draw = Callable[[str, random.Random], list[Arg]]
"""(stratum, rng) → one whole point, for a solver whose arguments are drawn together."""


@dataclass(frozen=True)
class Definition:
    """A function's Gate A definition. `value` raises `DomainError` outside the domain and
    `NoValueError` where no single value exists; `domain` adds rules beyond the parameters'
    intervals (returns why the point is invalid, or None). `tail` sets parameter `tail_param`
    from a tail probability (a distribution's x). `draw` replaces the per-parameter sampler for a
    solver (`solver`: the 300-point count). With `variadic` the last parameter repeats (SUM's
    number2, number3, ...) and `draw` sets the argument count. `sources` cite where the
    definition's math and ranges come from. `serials_1900` marks math that reads date serials in
    the 1900 date system (YEARFRAC's calendar, HOUR's last-date bound): an eval cell in a 1904
    workbook gets no math value (T-3678). With `indexed` the function returns an array and a point
    is one element of it: the last two parameters are INDEX's row and column, and the cell is
    `=INDEX(FUNC(other args),row,column)` (MINVERSE, MMULT; T-3686). With `literal_arrays` an eval cell
    gets a math value only where every array parameter is written as an array literal: the math does not
    model what a range adds (AGGREGATE's hidden rows and the options that skip them; T-3693).
    `call_params` are trailing optional parameters an eval cell may pass but the sampler never draws
    (BETA.DIST's and BETA.INV's bounds, RATE's guess; T-3923): `params_for` accepts them after
    `params`, and `value` receives them when the call gives them. `rescaled_param` names the parameter
    they rescale (BETA.DIST's x, which [A, B] moves off [0, 1]): when a call gives call parameters,
    that parameter's interval is not checked, and `domain` checks it instead."""

    function: str
    params: tuple[Param, ...]
    value: Callable[[Sequence[MathArg]], mpmath.mpf]
    sources: tuple[str, ...]
    domain: Callable[[Sequence[MathArg]], str | None] | None = None
    tail: TailMap | None = None
    tail_param: str | None = None
    draw: Draw | None = None
    solver: bool = False
    variadic: bool = False
    serials_1900: bool = False
    indexed: bool = False
    literal_arrays: bool = False
    call_params: tuple[Param, ...] = ()
    rescaled_param: str | None = None

    def __post_init__(self) -> None:
        if self.call_params and self.variadic:
            msg = f"{self.function}: a variadic definition takes no call parameters"
            raise ValueError(msg)
        flags = [p.optional for p in self.params]
        if flags != sorted(flags):
            msg = f"{self.function}: an optional parameter is followed by a required one"
            raise ValueError(msg)
        if self.indexed and (
            self.variadic
            or len(self.params) < 3
            or any(p.kind != "int" or p.optional for p in self.params[-2:])
        ):
            msg = f"{self.function}: an indexed definition ends in two required integer parameters"
            raise ValueError(msg)

    def params_for(self, count: int) -> tuple[Param, ...]:
        """The parameters a call with `count` arguments passes: every required one and the first
        optional ones (then the first `call_params`), with a variadic definition's last parameter
        repeated to fill `count`. ValueError when `count` is fewer than the required, or more than
        all for a definition that is not variadic."""
        every = self.params + self.call_params
        required = sum(1 for p in self.params if not p.optional)
        if count < required or (count > len(every) and not self.variadic):
            if self.variadic:
                takes = f"at least {required}"
            elif required == len(every):
                takes = f"{required}"
            else:
                takes = f"{required} to {len(every)}"
            msg = f"{self.function} takes {takes} arguments, not {count}"
            raise ValueError(msg)
        if count <= len(every):
            return every[:count]
        return self.params + (self.params[-1],) * (count - len(self.params))

    def numeric_arity(self) -> int:
        """The number of real or integer parameters."""
        return sum(1 for p in self.params if p.kind != "bool")

    def check_domain(self, args: Sequence[MathArg]) -> None:
        """Raises `DomainError` when `args` are outside the parameters' intervals or `domain`."""
        rescaled = self.rescaled_param if len(args) > len(self.params) else None
        for p, a in zip(self.params_for(len(args)), args, strict=True):
            if isinstance(a, mpmath.mpc):  # a complex number's parts each take the parameter's interval
                parts = [a.real, a.imag]
            elif (
                p.kind in ("bool", "array", "matrix", "text")
                or isinstance(a, (bool, str, tuple))
                or p.name == rescaled
            ):
                # an array's or a matrix's rules, a list text's and a rescaled parameter's are the definition's `domain`
                continue
            else:
                parts = [a]
            for part in parts:
                if not p.domain.contains(float(part)):
                    msg = f"{p.name} outside {p.domain.describe()}"
                    raise DomainError(msg)
        if self.domain is not None and (why := self.domain(args)) is not None:
            raise DomainError(why)

    def evaluate(self, args: Sequence[MathArg]) -> mpmath.mpf:
        """The math value at the current mpmath precision (domain checked first)."""
        self.check_domain(args)
        return self.value(args)


REGISTRY: dict[str, Definition] = {}


def register(definition: Definition) -> Definition:
    """Adds `definition` to `REGISTRY` (one per function name)."""
    if definition.function in REGISTRY:
        msg = f"duplicate Gate A definition for {definition.function}"
        raise ValueError(msg)
    REGISTRY[definition.function] = definition
    return definition


def solve_bracket(
    f: Callable[[mpmath.mpf], mpmath.mpf], a: mpmath.mpf, b: mpmath.mpf, max_steps: int = 6000
) -> mpmath.mpf:
    """The root of `f` in [a, b], where f(a) and f(b) have opposite signs (or one is 0), to the
    working precision: the Illinois variant of regula falsi, which keeps the bracket (so it
    cannot leave the domain) and converges superlinearly; a bisection step every fourth step
    bounds the worst case. NoValueError when the bracket has no sign change or does not shrink."""
    fa, fb = f(a), f(b)
    if fa == 0:
        return a
    if fb == 0:
        return b
    if (fa > 0) == (fb > 0):
        msg = "no sign change on the bracket"
        raise NoValueError(msg)
    eps = mpmath.mpf(2) ** (-mpmath.mp.prec + 2)
    tiny = mpmath.mpf(10) ** -320  # a root this close to 0 is 0 for Gate A's absolute floor
    side = 0
    for step in range(max_steps):
        if abs(b - a) <= eps * max(abs(a), abs(b), tiny):
            return a if abs(fa) < abs(fb) else b
        c = (a + b) / 2 if step % 4 == 3 else b - fb * (b - a) / (fb - fa)
        if not (min(a, b) < c < max(a, b)):
            c = (a + b) / 2
        fc = f(c)
        if fc == 0:
            return c
        if (fc > 0) == (fb > 0):
            b, fb = c, fc
            if side == -1:
                fa /= 2
            side = -1
        else:
            a, fa = c, fc
            if side == 1:
                fb /= 2
            side = 1
    msg = "root bracket did not converge"
    raise NoValueError(msg)


LOG_SMALLEST_DOUBLE = math.log(5e-324)
"""ln of the smallest positive double (2^-1074): the lower end of BETA.INV's log bracket, since a root below it has no
double and Excel is #NUM! there (statistics.md "BETAINV is #NUM! where its quantile underflows", T-3578)."""


def solve_log_bracket(f: Callable[[mpmath.mpf], mpmath.mpf], lo: float, hi: float) -> mpmath.mpf:
    """The s in [lo, hi] where `f` changes sign, for a quantile solved in log scale (x = ±e^s), so
    a 1E-300 tail is as well conditioned as the centre. NoValueError when the root is not in the
    bracket: the quantile is outside the double range (the sampler redraws)."""
    try:
        return solve_bracket(f, mpmath.mpf(lo), mpmath.mpf(hi))
    except NoValueError as err:
        msg = f"quantile outside the double range ({err})"
        raise NoValueError(msg) from err


def solve_smaller_tail(
    p: mpmath.mpf,
    lower_tail: Callable[[mpmath.mpf], mpmath.mpf],
    upper_tail: Callable[[mpmath.mpf], mpmath.mpf] | None,
    lo: float,
    hi: float,
    *,
    upper: bool = False,
) -> mpmath.mpf:
    """The s in [lo, hi] with tail(e^s) = min(p, 1 - p) for a quantile of probability `p` (0 < p < 1): the smaller
    tail is solved in log scales, ln tail(e^s) = ln q, so a 1E-300 tail is as well conditioned as the centre.
    `lower_tail` and `upper_tail` map x = e^s to P(X ≤ x) and P(X > x). With `upper`, `p` is itself an upper-tail
    probability, Q(x) = p. `upper_tail` may be None for a quantile that only ever solves the lower tail.
    NoValueError when the root is not in the bracket (see `solve_log_bracket`)."""
    small = p <= mpmath.mpf(1) / 2
    lower = small != upper
    target = mpmath.log(p if small else 1 - p)

    def f(s: mpmath.mpf) -> mpmath.mpf:
        x = mpmath.exp(s)
        if lower:
            return mpmath.log(lower_tail(x)) - target
        if upper_tail is None:
            msg = "the upper tail is needed here but no upper_tail function was given"
            raise ValueError(msg)
        # the lower tail grows with s and the upper falls: f has opposite signs on the bracket ends
        return target - mpmath.log(upper_tail(x))

    return solve_log_bracket(f, lo, hi)
