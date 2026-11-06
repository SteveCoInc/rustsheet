"""MDETERM, MINVERSE and MMULT (Gate A, T-3686; decision 0228 named them for tables once the generator
writes 2-D array literals).

The math is exact: the determinant, the inverse or the product of the matrix literals' doubles in rational
arithmetic, then rounded to the working precision. MINVERSE and MMULT return arrays, so a point is one element
read through INDEX (`=INDEX(MINVERSE({...}),row,column)`, `Definition.indexed`). The Excel rules that depend only
on those doubles are part of the definition (arrays.md):
- "MDETERM: partial-pivot LU, no singularity cutoff": a pivot product past the double range is `#NUM!`
  (`MDETERM({1E+200,0;0,1E+200})`, probes/minverse-singular-near-singular H1). The definition reads that as a
  determinant past the double range; the edge draws overflow by orders of magnitude, where the two agree.
  An underflowing product is 0 (`{1E-200,0;0,1E-200}`), inside Gate A's 1E-300 absolute floor of the math 1E-400.
- "MINVERSE: singularity rule": `#NUM!` only for an exactly-zero pivot (`{1,2;2,4}`). The definition reads a
  zero determinant as `#NUM!`; its singular draws are 2x2 with one row twice the other, whose partial-pivot
  multiplier is exactly 1/2 and whose last pivot is exactly 0, so the two agree. A singular matrix whose
  elimination leaves a rounding residual (`{1,2,3;4,5,6;7,8,9}`) is a number in Excel and is never drawn.
  An inverse element past the double range makes the whole result `#NUM!`
  (probes/minverse-overflow-computed-pivot), so any exact element past the range is a domain error.
Not claimed, never drawn (NoValueError): MMULT of non-conformable matrices, an MMULT element or product past the
double range, and an INDEX row or column outside the result.
"""

from __future__ import annotations

import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    by_stratum,
    domain_of,
    fraction_to_mpf,
    one_of,
    signed_log_uniform,
)
from rustsheet_tools.accuracy.literal import Arg, Matrix, round15
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    Interval,
    MathArg,
    NoValueError,
    Param,
    register,
)

MDETERM_SOURCE = "arrays.md#mdeterm-partial-pivot-lu-no-singularity-cutoff"
MINVERSE_SOURCE = "arrays.md#minverse-singularity-rule"
SOURCES = (MDETERM_SOURCE, MINVERSE_SOURCE, "decision 0228")
DOUBLE_MAX = Fraction(1.7976931348623157e308)

Exact = list[list[Fraction]]


def exact(a: MathArg) -> Exact:
    """A matrix argument's doubles as exact rationals (the literal's double is exact at any working precision)."""
    if not isinstance(a, tuple) or not all(isinstance(row, tuple) for row in a):
        msg = "a matrix argument is not a matrix"
        raise TypeError(msg)
    rows = [[Fraction(float(x)) for x in row] for row in a if isinstance(row, tuple)]
    if not rows or len({len(r) for r in rows}) != 1:
        msg = "the matrix rows differ in length"
        raise NoValueError(msg)
    return rows


def index_of(args: Sequence[MathArg]) -> tuple[int, int]:
    """INDEX's row and column, the last two arguments of an indexed definition (shared by the array regression)."""
    row, column = args[-2], args[-1]
    assert not isinstance(row, (bool, tuple)) and not isinstance(column, (bool, tuple))
    return int(row), int(column)


def _element(m: Exact, args: Sequence[MathArg]) -> Fraction:
    """The INDEX row and column of `m` (1-based); NoValueError outside it (INDEX's #REF! is not claimed)."""
    r, c = index_of(args)
    if not (1 <= r <= len(m) and 1 <= c <= len(m[0])):
        msg = f"INDEX({r},{c}) is outside the {len(m)}x{len(m[0])} result"
        raise NoValueError(msg)
    return m[r - 1][c - 1]


def _square(m: Exact) -> int:
    n = len(m)
    if len(m[0]) != n:
        msg = "the matrix is not square"
        raise NoValueError(msg)
    return n


def determinant(m: Exact) -> Fraction:
    """The exact determinant (Gaussian elimination in rationals: any nonzero pivot is exact)."""
    n = _square(m)
    a = [row[:] for row in m]
    det = Fraction(1)
    for c in range(n):
        p = next((r for r in range(c, n) if a[r][c] != 0), None)
        if p is None:
            return Fraction(0)
        if p != c:
            a[c], a[p] = a[p], a[c]
            det = -det
        det *= a[c][c]
        for r in range(c + 1, n):
            f = a[r][c] / a[c][c]
            if f:
                a[r] = [x - f * y for x, y in zip(a[r], a[c], strict=True)]
    return det


def inverse(m: Exact) -> Exact:
    """The exact inverse (Gauss-Jordan in rationals); DomainError when the matrix is singular."""
    n = _square(m)
    a = [row[:] + [Fraction(int(i == j)) for j in range(n)] for i, row in enumerate(m)]
    for c in range(n):
        p = next((r for r in range(c, n) if a[r][c] != 0), None)
        if p is None:
            msg = "a singular matrix (an exactly-zero pivot)"
            raise DomainError(msg)
        a[c], a[p] = a[p], a[c]
        pivot = a[c][c]
        a[c] = [x / pivot for x in a[c]]
        for r in range(n):
            if r != c and a[r][c]:
                f = a[r][c]
                a[r] = [x - f * y for x, y in zip(a[r], a[c], strict=True)]
    return [row[n:] for row in a]


def product(a: Exact, b: Exact) -> Exact:
    """The exact matrix product; NoValueError when the columns of `a` are not the rows of `b`."""
    if len(a[0]) != len(b):
        msg = "MMULT of non-conformable matrices"
        raise NoValueError(msg)
    return [
        [
            sum((x * y for x, y in zip(row, column, strict=True)), Fraction(0))
            for column in zip(*b, strict=False)
        ]
        for row in a
    ]


def _mdeterm(args: Sequence[MathArg]) -> mpmath.mpf:
    det = determinant(exact(args[0]))
    if abs(det) > DOUBLE_MAX:
        msg = "the pivot product is beyond the double range"
        raise DomainError(msg)
    return fraction_to_mpf(det)


def _minverse(args: Sequence[MathArg]) -> mpmath.mpf:
    inv = inverse(exact(args[0]))
    if any(abs(x) > DOUBLE_MAX for row in inv for x in row):
        msg = "an inverse element is beyond the double range"
        raise DomainError(msg)
    return fraction_to_mpf(_element(inv, args))


def _mmult(args: Sequence[MathArg]) -> mpmath.mpf:
    prod = product(exact(args[0]), exact(args[1]))
    if any(abs(x) > DOUBLE_MAX for row in prod for x in row):
        msg = "a product element is beyond the double range (not claimed)"
        raise NoValueError(msg)
    return fraction_to_mpf(_element(prod, args))


# -- draws -------------------------------------------------------------------------------------


def _uniform(rng: random.Random, rows: int, columns: int, bound: float) -> Matrix:
    return tuple(tuple(round15(rng.uniform(-bound, bound)) for _ in range(columns)) for _ in range(rows))


def _log(rng: random.Random, rows: int, columns: int, lo: float, hi: float) -> Matrix:
    return tuple(
        tuple(round15(signed_log_uniform(rng, lo, hi, 0.5)) for _ in range(columns)) for _ in range(rows)
    )


def _integers(rng: random.Random, rows: int, columns: int) -> Matrix:
    return tuple(tuple(float(rng.randint(-9, 9)) for _ in range(columns)) for _ in range(rows))


def _near_singular(rng: random.Random, n: int) -> Matrix:
    """An n x n matrix whose last row is the first one perturbed by a relative 1E-10 to 1E-4: ill-conditioned, so
    Excel's elimination is off the exact math by much more than an ulp and the table records by how much."""
    m = [list(row) for row in _uniform(rng, n, n, 10.0)]
    delta = signed_log_uniform(rng, 1e-10, 1e-4, 0.5)
    m[-1] = [round15(x * (1 + delta * rng.uniform(0.5, 1.5))) for x in m[0]]
    return tuple(tuple(row) for row in m)


def _singular_2x2(rng: random.Random) -> Matrix:
    """`{a,b;2a,2b}`: the partial-pivot multiplier is exactly 1/2 and the last pivot exactly 0 (Excel's `#NUM!`
    for MINVERSE, 0 for MDETERM, as `{1,2;2,4}`)."""
    a, b = round15(rng.uniform(-100, 100)), round15(rng.uniform(-100, 100))
    return ((a, b), (2 * a, 2 * b))


def _diagonal(values: Sequence[float]) -> Matrix:
    n = len(values)
    return tuple(tuple(values[i] if i == j else 0.0 for j in range(n)) for i in range(n))


def _nonsingular_integers(rng: random.Random, n: int) -> Matrix:
    """An n x n matrix of integers from -9 to 9 with a nonzero determinant (a singular integer matrix can leave
    a rounding residual in Excel's elimination, a number where MINVERSE's math is a domain error)."""
    while True:
        m = _integers(rng, n, n)
        if determinant([[Fraction(x) for x in row] for row in m]) != 0:
            return m


def _square_edge(rng: random.Random) -> Matrix:
    """The edges MDETERM and MINVERSE share: a 1x1 matrix, integer and permutation matrices, a 10x10 matrix,
    an exactly singular 2x2 and a zero column (both exactly-zero pivots), and pivot products or inverse
    elements that overflow or underflow."""
    big = round15(rng.uniform(1e155, 1e200))
    tiny = round15(rng.uniform(1e-200, 1e-155))
    n = rng.randint(2, 5)
    shapes: list[Callable[[], Matrix]] = [
        lambda: (
            (round15(signed_log_uniform(rng, 1e-300, 1e300, 0.5)),),
        ),  # 1x1: the number, or its reciprocal
        lambda: _nonsingular_integers(rng, n),
        lambda: tuple(tuple(1.0 if j == p else 0.0 for j in range(n)) for p in rng.sample(range(n), n)),
        lambda: _uniform(rng, 10, 10, 10.0),
        lambda: _singular_2x2(rng),
        lambda: tuple(
            tuple(0.0 if j == 0 else x for j, x in enumerate(row)) for row in _uniform(rng, n, n, 10.0)
        ),
        lambda: _diagonal([big, big]),  # MDETERM #NUM! (the pivot product overflows); MINVERSE 1/big
        lambda: _diagonal([tiny, tiny]),  # MDETERM 0 (underflow); MINVERSE 1/tiny
        lambda: ((tiny, 1.0), (0.0, tiny)),  # MINVERSE: the corner overflows, #NUM! in every cell
    ]
    return rng.choice(shapes)()


def _sized(rng: random.Random, lo: int, hi: int, make: Callable[[int], Matrix]) -> Matrix:
    """`make(n)` for an n drawn from lo to hi."""
    return make(rng.randint(lo, hi))


def _square_draw(function: str, indexed: bool) -> Callable[[str, random.Random], list[Arg]]:
    def point(m: Matrix, rng: random.Random) -> list[Arg]:
        if not indexed:
            return [m]
        return [m, rng.randint(1, len(m)), rng.randint(1, len(m[0]))]

    def tail(rng: random.Random) -> Matrix:
        if rng.random() < 0.5:
            return _near_singular(rng, rng.randint(2, 5))
        return _sized(rng, 5, 8, lambda n: _uniform(rng, n, n, 10.0))

    return by_stratum(
        {
            "typical": lambda rng: point(_sized(rng, 2, 4, lambda n: _uniform(rng, n, n, 100.0)), rng),
            "tail": lambda rng: point(tail(rng), rng),
            "edge": lambda rng: point(_square_edge(rng), rng),
            "extreme": lambda rng: point(_sized(rng, 2, 3, lambda n: _log(rng, n, n, 1e-80, 1e80)), rng),
            "log-uniform": lambda rng: point(_sized(rng, 1, 6, lambda n: _log(rng, n, n, 1e-6, 1e6)), rng),
        },
        f"{function} draws: square matrices by stratum ({'one element of the result' if indexed else 'the value'}).",
    )


def _mmult_point(
    rng: random.Random, a: Matrix, columns: int, b_draw: Callable[[int, int], Matrix]
) -> list[Arg]:
    b = b_draw(len(a[0]), columns)
    return [a, b, rng.randint(1, len(a)), rng.randint(1, columns)]


def _mmult_shape(rng: random.Random, hi: int) -> tuple[int, int, int]:
    return rng.randint(1, hi), rng.randint(1, hi), rng.randint(1, hi)


def _mmult_draws() -> Callable[[str, random.Random], list[Arg]]:
    def uniform(bound: float, hi: int) -> Callable[[random.Random], list[Arg]]:
        def draw(rng: random.Random) -> list[Arg]:
            r, k, c = _mmult_shape(rng, hi)
            return _mmult_point(
                rng, _uniform(rng, r, k, bound), c, lambda k_, c_: _uniform(rng, k_, c_, bound)
            )

        return draw

    def logged(lo: float, hi_mag: float, hi: int) -> Callable[[random.Random], list[Arg]]:
        def draw(rng: random.Random) -> list[Arg]:
            r, k, c = _mmult_shape(rng, hi)
            return _mmult_point(
                rng, _log(rng, r, k, lo, hi_mag), c, lambda k_, c_: _log(rng, k_, c_, lo, hi_mag)
            )

        return draw

    def edge(rng: random.Random) -> list[Arg]:
        k = rng.randint(1, 6)
        shapes: list[Callable[[], list[Arg]]] = [
            lambda: [_uniform(rng, 1, k, 10.0), _uniform(rng, k, 1, 10.0), 1, 1],  # a row times a column
            lambda: [_uniform(rng, 1, 1, 1e6), _uniform(rng, 1, 1, 1e6), 1, 1],  # 1x1 times 1x1
            lambda: _mmult_point(
                rng, _uniform(rng, 3, 3, 10.0), 3, lambda k_, _c: _diagonal([1.0] * k_)
            ),  # identity
            lambda: _mmult_point(rng, _integers(rng, 4, k), 4, lambda k_, c_: _integers(rng, k_, c_)),
            lambda: _mmult_point(
                rng, _uniform(rng, 2, 30, 10.0), 2, lambda k_, c_: _uniform(rng, k_, c_, 10.0)
            ),
            lambda: _mmult_point(  # entries near 1E+100: products near 1E+200, still finite
                rng, _log(rng, 2, 3, 1e99, 1e100), 2, lambda k_, c_: _log(rng, k_, c_, 1e99, 1e100)
            ),
        ]
        return one_of(rng, shapes)

    return by_stratum(
        {
            "typical": uniform(100.0, 4),
            "tail": uniform(10.0, 8),
            "edge": edge,
            "extreme": logged(1e-80, 1e80, 3),
            "log-uniform": logged(1e-6, 1e6, 6),
        },
        "MMULT draws: conformable matrices by stratum, one element of the product.",
    )


def row_column(source: str) -> tuple[Param, Param]:
    """INDEX's row and column of an indexed definition (not the catalog's parameters: the function's own end
    before them)."""
    whole = Interval(1, float("inf"), lo_open=False)
    return Param("row_num", "int", whole, (1, 8), source=source), Param(
        "column_num", "int", whole, (1, 8), source=source
    )


MDETERM = register(
    Definition(
        function="MDETERM",
        params=(Param("array", "matrix", source=MDETERM_SOURCE),),
        value=_mdeterm,
        domain=domain_of(_mdeterm),
        draw=_square_draw("MDETERM", indexed=False),
        sources=SOURCES,
    )
)

MINVERSE = register(
    Definition(
        function="MINVERSE",
        params=(Param("array", "matrix", source=MINVERSE_SOURCE), *row_column(MINVERSE_SOURCE)),
        value=_minverse,
        domain=domain_of(_minverse),
        draw=_square_draw("MINVERSE", indexed=True),
        indexed=True,
        sources=SOURCES,
    )
)

MMULT = register(
    Definition(
        function="MMULT",
        params=(
            Param("array1", "matrix", source=MDETERM_SOURCE),
            Param("array2", "matrix", source=MDETERM_SOURCE),
            *row_column(MDETERM_SOURCE),
        ),
        value=_mmult,
        domain=domain_of(_mmult),
        draw=_mmult_draws(),
        indexed=True,
        sources=SOURCES,
    )
)
