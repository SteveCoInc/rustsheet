"""FACT, FACTDOUBLE, COMBIN, COMBINA and MULTINOMIAL (Gate A slice 4h, T-3675, decision 0228).

The math is the exact combinatorial value of the integer arguments Excel reads. Excel's product
and log-sum orders make large results inexact (numbers.md "COMBIN is inexact", "MULTINOMIAL is
inexact", "FACT and FACTDOUBLE of a huge argument"); Gate A's envelope judges that against the
math. Each rule below is VERIFIED on excel-graph cells:
- FACT (numbers.md "FACT and FACTDOUBLE of a huge argument"): the argument is floored
  (`FACT(170.9)` is `FACT(170)`), a negative one is `#NUM!` (`FACT(-0.5)`, `FACT(-1)`), and so is
  one whose factorial overflows a double (`FACT(171)`).
- FACTDOUBLE (same entry): floored; `FACTDOUBLE(-1)` is 1, below -1 `#NUM!` (`(-1.5)`, `(-2)`);
  `FACTDOUBLE(300)` is finite and `(301)` overflows (`#NUM!`).
- COMBIN (numbers.md "COMBIN is inexact" and "COMBINA is COMBIN(n+k-1, k)", "Bound"): both
  arguments floored, `#NUM!` for a negative one, for k > n, for n at or above 2^31-1 whatever k,
  and where the coefficient overflows (`COMBIN(1030,514)`).
- COMBINA (same entries; statistics.md "Statistical argument-domain errors and extreme values
  (T-2621)" for the reading of fractions): COMBIN(n+k-1, k) with `number` truncated toward zero and
  `number_chosen` floored (truncated when `number` is 0, so `COMBINA(-0.5,-0.0001)` is 1);
  `#NUM!` for a negative one, for n = 0 with k > 0, and from n+k-1 = 2^31-1 on. `COMBINA(0,0)` is 1.
- MULTINOMIAL (numbers.md "MULTINOMIAL is inexact", "MULTINOMIAL of a huge argument"): each
  argument truncated (`MULTINOMIAL(2.5,3)` is `MULTINOMIAL(2,3)`), a negative one `#NUM!` before the
  truncation, and `#NUM!` once the truncated arguments add up to more than 170, even where the ratio
  is small (`MULTINOMIAL(100,100)`). The value is sum! / prod(a_i!).
"""

from __future__ import annotations

import math
import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy import model
from rustsheet_tools.accuracy.defs._common import by_stratum, one_of, scalars
from rustsheet_tools.accuracy.literal import Arg
from rustsheet_tools.accuracy.model import Definition, Draw, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

FACT_SOURCE = "numbers.md#fact-and-factdouble-of-a-huge-argument"
COMBIN_SOURCE = "numbers.md#combin-is-inexact"
COMBINA_SOURCE = "numbers.md#combina-is-combinnk-1-k"
COMBINA_FRACTION_SOURCE = "statistics.md#statistical-argument-domain-errors-and-extreme-values-t-2621"
MULTINOMIAL_SOURCE = "numbers.md#multinomial-is-inexact"
MULTINOMIAL_HUGE_SOURCE = "numbers.md#multinomial-of-a-huge-argument"
DECISION = "decision 0228"
COUNT_LIMIT = 2**31 - 1  # COMBIN's n and COMBINA's n+k-1 are #NUM! from here up
MULTINOMIAL_SUM_LIMIT = 170  # the largest sum whose factorial is finite
_DOUBLE_MAX_EXACT = mpmath.mpf(2) ** 1024 * (1 - mpmath.mpf(2) ** -54)  # the overflow rounding point


def _finite(value: int, what: str) -> mpmath.mpf:
    """`value` as an mpf; DomainError (`#NUM!`) where it rounds past the largest double."""
    result = mpmath.mpf(value)
    if result >= _DOUBLE_MAX_EXACT:
        msg = f"{what} overflows a double (#NUM!)"
        raise model.DomainError(msg)
    return result


def double_factorial(n: int) -> int:
    """n!! for n >= -1 ((-1)!! and 0!! are 1)."""
    out = 1
    for i in range(n, 1, -2):
        out *= i
    return out


def fact_domain(args: Sequence[MathArg]) -> str | None:
    (x,) = scalars(args)
    return "a negative number (#NUM!)" if mpmath.floor(x) < 0 else None


def factdouble_domain(args: Sequence[MathArg]) -> str | None:
    (x,) = scalars(args)
    return "a number below -1 (#NUM!)" if mpmath.floor(x) < -1 else None


def _fact(args: Sequence[MathArg]) -> mpmath.mpf:
    (x,) = scalars(args)
    n = int(mpmath.floor(x))
    if n > 170:  # n! is past the largest double from 171 on; no big-int product of a huge n
        return _finite(math.factorial(171), "n!")
    return _finite(math.factorial(n), "n!")


def _factdouble(args: Sequence[MathArg]) -> mpmath.mpf:
    (x,) = scalars(args)
    n = int(mpmath.floor(x))
    if n > 301:  # n!! is past the largest double from 301 on
        return _finite(double_factorial(301), "n!!")
    return _finite(double_factorial(n), "n!!")


def combin_value(n: int, k: int) -> mpmath.mpf:
    """C(n, k) of integers 0 <= k <= n < 2^31-1; DomainError where it overflows a double."""
    j = min(k, n - k)
    # C(n, j) only grows with j up to n/2; past 1030 choose 515 it overflows, so a huge n with a
    # large j is decided by a log-gamma bound before any big-int product.
    if mpmath.loggamma(n + 1) - mpmath.loggamma(j + 1) - mpmath.loggamma(n - j + 1) > 711:
        msg = "the coefficient overflows a double (#NUM!)"
        raise model.DomainError(msg)
    return _finite(math.comb(n, j), "the coefficient")


def combin_domain(args: Sequence[MathArg]) -> str | None:
    n, k = (int(mpmath.floor(a)) for a in scalars(args))
    if n < 0 or k < 0:
        return "a negative argument (#NUM!)"
    if k > n:
        return "number_chosen above number (#NUM!)"
    if n >= COUNT_LIMIT:
        return "number at or above 2^31-1 (#NUM!)"
    return None


def _combin(args: Sequence[MathArg]) -> mpmath.mpf:
    n, k = (int(mpmath.floor(a)) for a in scalars(args))
    return combin_value(n, k)


def _combina_args(args: Sequence[MathArg]) -> tuple[int, int]:
    """(n, k) as COMBINA reads them: `number` truncated, `number_chosen` floored, or truncated
    when `number` is 0."""
    x, y = scalars(args)
    n = int(mpmath.floor(x)) if x >= 0 else -int(mpmath.floor(-x))
    k = (int(mpmath.floor(y)) if y >= 0 else -int(mpmath.floor(-y))) if n == 0 else int(mpmath.floor(y))
    return n, k


def combina_domain(args: Sequence[MathArg]) -> str | None:
    n, k = _combina_args(args)
    if n < 0 or k < 0:
        return "a negative argument (#NUM!)"
    if n == 0 and k > 0:
        return "number 0 with number_chosen above 0 (#NUM!)"
    if n + k - 1 >= COUNT_LIMIT:
        return "number + number_chosen - 1 at or above 2^31-1 (#NUM!)"
    return None


def _combina(args: Sequence[MathArg]) -> mpmath.mpf:
    n, k = _combina_args(args)
    if n == 0 and k == 0:
        return mpmath.mpf(1)
    return combin_value(n + k - 1, k)


def multinomial_domain(args: Sequence[MathArg]) -> str | None:
    values = scalars(args)
    if any(v < 0 for v in values):
        return "a negative argument (#NUM!)"
    if sum(int(mpmath.floor(v)) for v in values) > MULTINOMIAL_SUM_LIMIT:
        return "the arguments add up to more than 170 (#NUM!)"
    return None


def _multinomial(args: Sequence[MathArg]) -> mpmath.mpf:
    parts = [int(mpmath.floor(v)) for v in scalars(args)]
    denominator = math.prod(math.factorial(p) for p in parts)
    return mpmath.mpf(math.factorial(sum(parts))) / denominator


def _fraction(rng: random.Random, whole: int) -> float:
    """`whole` plus a fraction one time in five (the arguments are floored or truncated)."""
    if rng.random() < 0.2:
        return float(whole) + rng.choice((0.25, 0.5, 0.9, 0.999))
    return float(whole)


def _fact_draw(limit: int) -> Draw:
    """[number] for FACT (limit 170) or FACTDOUBLE (limit 300)."""
    edges = (0.0, 0.5, 1.0, 1.5, 2.0, 3.0, -0.5, -1.0, -1.5, -2.0, float(limit), limit + 0.9, limit + 1.0)
    return by_stratum(
        {
            "typical": lambda rng: [_fraction(rng, rng.randint(0, 30))],
            "tail": lambda rng: [_fraction(rng, rng.randint(limit - 30, limit + 2))],
            "edge": lambda rng: [rng.choice(edges)],
            "extreme": lambda rng: [float(rng.randint(limit + 1, 10**6))],
            "log-uniform": lambda rng: [log_uniform(rng, 1.0, 1e300)],
        },
        "typical 0 to 30, tails up to and just past the overflow, edges about 0, -1 and the limit, "
        "extreme counts past the limit, and log-uniform counts 1 to 1E+300",
    )


def _combin_typical(rng: random.Random) -> list[Arg]:
    n = rng.randint(0, 60)
    return [_fraction(rng, n), _fraction(rng, rng.randint(0, n))]


def _combin_tail(rng: random.Random) -> list[Arg]:
    n = rng.randint(60, 1030)
    j = rng.choice((rng.randint(0, n), n // 2 + rng.randint(-20, 20), rng.randint(n // 3, n // 2)))
    return [float(n), float(max(0, min(n, j)))]


def _combin_edge(rng: random.Random) -> list[Arg]:
    n = rng.randint(1, 200)
    return one_of(
        rng,
        (
            lambda: [float(n), float(rng.choice((0, 1, 2, n - 1, n, n + 1)))],
            lambda: [
                float(rng.choice((COUNT_LIMIT - 1, COUNT_LIMIT, COUNT_LIMIT + 2))),
                float(rng.choice((0, 1, 2))),
            ],
            lambda: [rng.choice((-1.0, -0.5, 5.0)), rng.choice((-1.0, -0.5, 2.0))],
            lambda: [float(rng.choice((1029, 1030))), float(rng.choice((514, 515, 516)))],
            lambda: [rng.choice((10.5, 123456789.123, 40.9)), rng.choice((3.7, 10.0, 20.2))],
            lambda: [0.0, 0.0],
        ),
    )


def _combin_log_uniform(rng: random.Random) -> list[Arg]:
    n = round(log_uniform(rng, 1.0, 1e9))
    return [float(n), float(rng.randint(0, min(n, 60)))]


_combin_draw = by_stratum(
    {
        "typical": _combin_typical,
        "tail": _combin_tail,
        "edge": _combin_edge,
        "extreme": lambda rng: [float(rng.randint(10**6, COUNT_LIMIT - 1)), float(rng.randint(0, 40))],
        "log-uniform": _combin_log_uniform,
    },
    "[number, number_chosen]: typical n up to 60, tails n 60 to 1030 (overflow from about 1030 choose 515), edges "
    "(k 0, 1, 2, n-1, n, n+1, the 2^31-1 cut, negatives, fractions), extreme n up to 2^31-2 with k up to 40, and "
    "log-uniform n 1 to 1E+9 with k up to 60",
)


def _combina_tail(rng: random.Random) -> list[Arg]:
    k = rng.randint(30, 515)
    return [float(rng.randint(1, 1031 - k)), float(k)]


def _combina_edge(rng: random.Random) -> list[Arg]:
    return one_of(
        rng,
        (
            lambda: [rng.choice((0.0, 1.0, 2.0)), rng.choice((0.0, 1.0, 2.0))],
            lambda: [
                float(rng.choice((COUNT_LIMIT - 1, COUNT_LIMIT - 2, COUNT_LIMIT))),
                float(rng.choice((0, 1, 2))),
            ],
            lambda: [float(rng.choice((1, 2, 3))), float(COUNT_LIMIT - rng.choice((1, 2, 3, 4)))],
            lambda: [rng.choice((-1.0, -0.5, 7.0, 24.0)), rng.choice((-0.0001, -1.0, 2.0))],
            lambda: [rng.choice((10.5, 20.9, 25.0)), rng.choice((3.7, 19.0, 123456789.123))],
        ),
    )


_combina_draw = by_stratum(
    {
        "typical": lambda rng: [_fraction(rng, rng.randint(1, 30)), _fraction(rng, rng.randint(0, 30))],
        "tail": _combina_tail,
        "edge": _combina_edge,
        "extreme": lambda rng: [float(rng.randint(10**6, COUNT_LIMIT - 41)), float(rng.randint(0, 40))],
        "log-uniform": lambda rng: [float(round(log_uniform(rng, 1.0, 1e9))), float(rng.randint(0, 40))],
    },
    "[number, number_chosen]: typical up to 30 each, tails with n+k-1 60 to 1030, edges (0, 1, the 2^31-1 cut on "
    "n+k-1, negatives, fractions, a number in (-1, 0)), extreme n up to 2^31-41 with k up to 40, and log-uniform n "
    "1 to 1E+9 with k up to 40",
)


def _multinomial_tail(rng: random.Random) -> list[Arg]:
    count = rng.randint(1, 6)
    total = rng.randint(120, 172)
    cuts = sorted(rng.randint(0, total) for _ in range(count - 1))
    return [float(b - a) for a, b in zip([0, *cuts], [*cuts, total], strict=True)]


def _multinomial_edge(rng: random.Random) -> list[Arg]:
    return one_of(
        rng,
        (
            lambda: [float(rng.choice((0, 1, 4, 5, 6, 50, 170, 171)))],
            lambda: [0.0] * rng.randint(1, 6),
            lambda: [rng.choice((-1.0, -0.5)), float(rng.randint(0, 9))],
            lambda: [2.5, 3.0, rng.choice((0.9, 4.999))],
            lambda: [float(rng.choice((85, 86, 169, 170))), float(rng.choice((0, 1, 85)))],
        ),
    )


def _multinomial_typical(rng: random.Random) -> list[Arg]:
    count = rng.randint(1, 6)
    return [_fraction(rng, rng.randint(0, 40 // count)) for _ in range(count)]


_multinomial_draw = by_stratum(
    {
        "typical": _multinomial_typical,
        "tail": _multinomial_tail,
        "edge": _multinomial_edge,
        "extreme": lambda rng: [float(rng.choice((1e3, 2.0**31, 1e15, 1e300))), float(rng.randint(0, 3))],
        "log-uniform": lambda rng: [
            float(round(log_uniform(rng, 1.0, 100.0))) for _ in range(rng.randint(1, 6))
        ],
    },
    "1 to 6 arguments: typical sums up to 40, tails with sums 120 to 172, edges (a lone argument, zeros, a "
    "negative, fractions, the sum 170 against 171), extreme single huge arguments, and log-uniform arguments 1 to "
    "100 (the larger sums are #NUM!)",
)


_NONNEGATIVE = Interval()  # the domain functions decide the signs: fractions in (-1, 0) are read


def _number(name: str, source: str) -> Param:
    return Param(name, "real", _NONNEGATIVE, source=source)


FACT = register(
    Definition(
        function="FACT",
        params=(_number("number", FACT_SOURCE),),
        value=_fact,
        domain=fact_domain,
        draw=_fact_draw(170),
        sources=(DECISION, FACT_SOURCE),
    )
)
FACTDOUBLE = register(
    Definition(
        function="FACTDOUBLE",
        params=(_number("number", FACT_SOURCE),),
        value=_factdouble,
        domain=factdouble_domain,
        draw=_fact_draw(300),
        sources=(DECISION, FACT_SOURCE),
    )
)
COMBIN = register(
    Definition(
        function="COMBIN",
        params=(_number("number", COMBIN_SOURCE), _number("number_chosen", COMBIN_SOURCE)),
        value=_combin,
        domain=combin_domain,
        draw=_combin_draw,
        sources=(DECISION, COMBIN_SOURCE, COMBINA_SOURCE),
    )
)
COMBINA = register(
    Definition(
        function="COMBINA",
        params=(_number("number", COMBINA_SOURCE), _number("number_chosen", COMBINA_SOURCE)),
        value=_combina,
        domain=combina_domain,
        draw=_combina_draw,
        sources=(DECISION, COMBINA_SOURCE, COMBINA_FRACTION_SOURCE),
    )
)
MULTINOMIAL = register(
    Definition(
        function="MULTINOMIAL",
        params=(_number("number", MULTINOMIAL_SOURCE),),
        value=_multinomial,
        domain=multinomial_domain,
        draw=_multinomial_draw,
        variadic=True,
        sources=(DECISION, MULTINOMIAL_SOURCE, MULTINOMIAL_HUGE_SOURCE),
    )
)
