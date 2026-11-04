"""SUMPRODUCT, SUMX2MY2, SUMX2PY2 and SUMXMY2 (Gate A slice 4j, T-3677, decision 0228).

The math: the exact sum, over the array literals' doubles, of the products (SUMPRODUCT) or of
x^2-y^2, x^2+y^2 and (x-y)^2 (the SUMX family). The Excel rules that depend only on those doubles
are part of the definition (probes/t3677-sumx-running-overflow-snap, excel-graph, 2026-11-03, and
numbers.md):
- Overflow where it happens ("Overflow in `+ - * /` is #NUM! where it happens"): a running product
  or a square, a term (x^2+y^2, x-y, (x-y)^2) or a running sum beyond the double range is `#NUM!`,
  even when the exact total is finite. `SUMPRODUCT({9E+307;9E+307;-9E+307},{1;1;1})` is `#NUM!`
  while the order 9E+307, -9E+307, 9E+307 gives 9E+307 (A1, A2); SUMX2MY2 adds one x^2-y^2 per pair,
  so (1.2E+154, 0), (1.2E+154, 0), (0, 1.2E+154) is `#NUM!` but (1.2E+154, 0), (0, 1.2E+154),
  (1.2E+154, 0) is 1.44E+308 and two (1.2E+154, 1.2E+154) pairs are 0 (A4:A6); `SUMX2MY2({1.5E+154},
  {1.4E+154})` is `#NUM!` (a square, T-3808); `SUMX2PY2({1.2E+154},{1.2E+154})` (A20) and
  `SUMXMY2({1E+154,-1E+154},{-1E+154,1E+154})` (A19) are `#NUM!` (a term). The running values follow
  Excel's x87 model (each operation rounded to a 64-bit significand, then to a double;
  `fitlab.exact.double_round`), left to right, as SUM's do.
- No near-zero snap: SUMPRODUCT never snaps (numbers.md "Summing functions", T-3048) and neither does
  SUMX2MY2 (A8, A10, A11: 1 - (1+2^-52)^2 is -4.44E-16 at 2 ULP of 1). The other two add terms that
  are never negative, so they cannot cancel.
- Underflow ("Subnormal results become 0"): a subnormal literal reads as 0, SUMPRODUCT flushes every
  running product (T-1007) and the SUMX family flushes each square (A12:A15: `SUMX2PY2({1E-155,
  1E-150},{0,0})` is exactly 1E-300, as `SUMSQ` is). The s1 draws keep every value normal, so the
  tables score the sums; the probes math sidecars carry the flush cells.
- Arrays of different lengths are `#VALUE!` (SUMPRODUCT) or `#N/A` (the SUMX family; coercion.md
  "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing"): a domain error.
Inferred, unprobed: whether SUMX2MY2 adds x^2 and then subtracts y^2 from its running total (which
could overflow where the term does not) or adds the term; the draws avoid the points that differ.
"""

from __future__ import annotations

import math
import random
import sys
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, domain_of, one_of, signed_log_uniform
from rustsheet_tools.accuracy.defs.aggregate import SNAP_SOURCE, running_total
from rustsheet_tools.accuracy.defs.product import OVERFLOW_SOURCE, flush, running_product, squares
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, Param, register
from rustsheet_tools.fitlab.exact import double_round

EVIDENCE = "probes/t3677-sumx-running-overflow-snap"
FLUSH_SOURCE = "numbers.md#subnormal-results-become-0"
PAIRING_SOURCE = "coercion.md#sumx2my2--sumx2py2--sumxmy2-pairing"
MAX_PRODUCT_LITERALS = 300  # about 6,600 characters: inside Excel's 8192-character formula


def _arrays(args: Sequence[MathArg]) -> list[tuple[mpmath.mpf, ...]]:
    out: list[tuple[mpmath.mpf, ...]] = []
    for a in args:
        assert isinstance(a, tuple)
        out.append(a)
    if len({len(a) for a in out}) > 1:
        raise DomainError("the arrays differ in length")
    return out


def _total(terms: Sequence[float]) -> None:
    """DomainError when a term or the running sum of the terms leaves the double range."""
    if not all(math.isfinite(t) for t in terms) or not math.isfinite(running_total(terms)[1]):
        raise DomainError("a running value is beyond the double range")


def _sumproduct(args: Sequence[MathArg]) -> mpmath.mpf:
    """The exact sum of the products whose running products are not flushed."""
    columns = list(zip(*_arrays(args), strict=True))
    terms = [running_product([float(x) for x in column])[-1] for column in columns]
    _total(terms)
    return mpmath.fsum(mpmath.fprod(column) for column, t in zip(columns, terms, strict=True) if t != 0)


def _pairs(args: Sequence[MathArg]) -> list[tuple[mpmath.mpf, mpmath.mpf]]:
    xs, ys = _arrays(args)
    return list(zip(xs, ys, strict=True))


def _square(x: float) -> tuple[float, bool]:
    """(x^2 x87-rounded and flushed, whether it is kept); DomainError past the double range."""
    (square,) = squares([x])
    if not math.isfinite(square):
        raise DomainError("a square is beyond the double range")
    return square, square != 0


def _rounded(exact: Fraction) -> float:
    try:
        return double_round(exact)
    except OverflowError as e:
        raise DomainError("a term is beyond the double range") from e


def _x2my2_term(x: mpmath.mpf, y: mpmath.mpf) -> tuple[float, mpmath.mpf]:
    (x2, keep_x), (y2, keep_y) = _square(float(x)), _square(float(y))
    return _rounded(Fraction(x2) - Fraction(y2)), (x * x if keep_x else 0) - (y * y if keep_y else 0)


def _x2py2_term(x: mpmath.mpf, y: mpmath.mpf) -> tuple[float, mpmath.mpf]:
    (x2, keep_x), (y2, keep_y) = _square(float(x)), _square(float(y))
    return _rounded(Fraction(x2) + Fraction(y2)), (x * x if keep_x else 0) + (y * y if keep_y else 0)


def _xmy2_term(x: mpmath.mpf, y: mpmath.mpf) -> tuple[float, mpmath.mpf]:
    d = _rounded(Fraction(flush(float(x))) - Fraction(flush(float(y))))
    square, keep = _square(d)
    return square, ((x - y) ** 2 if keep else mpmath.mpf(0))


Term = Callable[[mpmath.mpf, mpmath.mpf], tuple[float, mpmath.mpf]]


def _pair_sum(term: Term) -> Callable[[Sequence[MathArg]], mpmath.mpf]:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        doubles, exact = zip(*(term(x, y) for x, y in _pairs(args)), strict=True)
        _total(doubles)
        return mpmath.fsum(exact)

    return value


# -- draws -------------------------------------------------------------------------------------


def _normal(value: Callable[[Sequence[MathArg]], mpmath.mpf], args: list[Arg]) -> bool:
    """Whether every running product, square and difference is zero or normal (underflow is not claimed),
    or the point is a domain error."""
    arrays = [tuple(mpmath.mpf(x) for x in a) for a in args if isinstance(a, tuple)]
    try:
        value(arrays)
    except DomainError:
        return True
    if value is _sumproduct:
        values = [
            p for column in zip(*arrays, strict=True) for p in running_product([float(x) for x in column])
        ]
    else:  # the squares before their flush (the point is finite, so none overflows)
        bases = [float(x) for a in arrays for x in a]
        bases += [double_round(Fraction(float(x)) - Fraction(float(y))) for x, y in _pairs(arrays)]
        values = [double_round(Fraction(b) ** 2) for b in bases]
    return all(v == 0 or abs(v) >= sys.float_info.min for v in values)


def _array(rng: random.Random, count: int, lo: float, hi: float, negative: float = 0.3) -> tuple[float, ...]:
    return tuple(round15(signed_log_uniform(rng, lo, hi, negative)) for _ in range(count))


def _redraw(value: Callable[[Sequence[MathArg]], mpmath.mpf], draw: Callable[[], list[Arg]]) -> list[Arg]:
    while True:
        args = draw()
        if _normal(value, args):
            return args


def _arrays_of(rng: random.Random, k: int, n: int, lo: float, hi: float) -> list[Arg]:
    return [_array(rng, n, lo, hi) for _ in range(k)]


def _overflow_order(rng: random.Random, cancel_last: bool) -> list[Arg]:
    """Three terms big, big, -big (a running sum past the double range, `#NUM!`; A1, A4) or big, -big,
    big (A2, A5), for SUMPRODUCT as (b, b, -b)·(1, 1, 1) and for the pairs as squares."""
    big = round15(rng.uniform(5e307, 9e307))
    order = (big, big, -big) if cancel_last else (big, -big, big)
    return [order, (1.0, 1.0, 1.0)]


def _sumproduct_edge(rng: random.Random) -> list[Arg]:
    big = round15(rng.uniform(1e155, 1e200))
    return one_of(
        rng,
        [
            lambda: _overflow_order(rng, cancel_last=True),
            lambda: _overflow_order(rng, cancel_last=False),
            lambda: [(big,), (big,)],  # a product past the double range
            lambda: [_array(rng, 3, 1e-3, 1e6), _array(rng, 2, 1e-3, 1e6)],  # #VALUE!: lengths differ
            lambda: [_array(rng, rng.randint(1, 8), 1e-3, 1e6)],  # one array: its sum
            lambda: _arrays_of(rng, 5, rng.randint(1, 6), 0.1, 10.0),  # five arrays
            lambda: [
                tuple(0.0 if rng.random() < 0.4 else x for x in _array(rng, 6, 1e-3, 1e3)) for _ in range(2)
            ],  # zeros
            lambda: _arrays_of(rng, 2, MAX_PRODUCT_LITERALS // 2 - 30, 1e-3, 1e6),  # long arrays
        ],
    )


def _pair_edge(function: str, rng: random.Random) -> list[Arg]:
    near = round15(rng.uniform(1.2e154, 1.3e154))  # a finite square whose double is past half the range
    huge = round15(rng.uniform(1.4e154, 1e200))  # a square past the double range
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [_array(rng, 3, 1e-3, 1e6), _array(rng, 2, 1e-3, 1e6)],  # #N/A: lengths differ
        lambda: [_array(rng, 1, 1e-3, 1e6), _array(rng, 1, 1e-3, 1e6)],  # one pair
        lambda: [(huge,), (round15(huge * 0.9),)],  # SUMX2MY2: #NUM! on the square; SUMXMY2 finite
        lambda: [
            tuple(0.0 if rng.random() < 0.4 else x for x in _array(rng, 6, 1e-3, 1e3)) for _ in range(2)
        ],  # zeros
        lambda: [_array(rng, 120, 1e-3, 1e6), _array(rng, 120, 1e-3, 1e6)],  # long arrays
    ]
    if function == "SUMX2MY2":
        shapes += [
            lambda: [(near, near, 0.0), (0.0, 0.0, near)],  # the running sum overflows (A4)
            lambda: [(near, 0.0, near), (0.0, near, 0.0)],  # the same terms in an order that does not (A5)
            lambda: [(near, near), (near, near)],  # finite squares that cancel (A6)
        ]
    elif function == "SUMX2PY2":
        shapes += [
            lambda: [(near,), (near,)],  # the term overflows (A20)
            lambda: [(near, 0.0), (0.0, near)],  # the running sum overflows
        ]
    else:
        shapes += [
            lambda: [(near,), (-near,)],  # the difference's square overflows (A19)
            lambda: [(near, 0.0), (0.0, near)],  # the running sum overflows
        ]
    return one_of(rng, shapes)


def _sumproduct_draw() -> Callable[[str, random.Random], list[Arg]]:
    v = _sumproduct
    return by_stratum(
        {
            "typical": lambda rng: _redraw(
                v, lambda: _arrays_of(rng, rng.randint(2, 3), rng.randint(1, 10), 1e-3, 1e6)
            ),
            "tail": lambda rng: _redraw(
                v, lambda: _arrays_of(rng, 2, rng.randint(10, 40), 0.5, 2.0)
            ),  # mixed-sign products near 1 that partly cancel
            "edge": _sumproduct_edge,
            "extreme": lambda rng: _redraw(
                v, lambda: _arrays_of(rng, rng.randint(2, 4), rng.randint(10, 50), 1e-20, 1e20)
            ),
            "log-uniform": lambda rng: _redraw(
                v, lambda: _arrays_of(rng, rng.randint(2, 3), rng.randint(1, 6), 1e-100, 1e100)
            ),
        },
        "SUMPRODUCT draws: 2 to 5 equal-length arrays by stratum; running products stay normal or overflow.",
    )


def _pair_draw(
    function: str, value: Callable[[Sequence[MathArg]], mpmath.mpf]
) -> Callable[[str, random.Random], list[Arg]]:
    return by_stratum(
        {
            "typical": lambda rng: _redraw(value, lambda: _arrays_of(rng, 2, rng.randint(1, 10), 1e-3, 1e6)),
            "tail": lambda rng: _redraw(value, lambda: _arrays_of(rng, 2, rng.randint(10, 40), 1e-6, 1e6)),
            "edge": lambda rng: _pair_edge(function, rng),
            "extreme": lambda rng: _redraw(
                value, lambda: _arrays_of(rng, 2, rng.randint(10, 60), 1e-20, 1e20)
            ),
            "log-uniform": lambda rng: _redraw(
                value, lambda: _arrays_of(rng, 2, rng.randint(1, 6), 1e-150, 1e150)
            ),
        },
        f"{function} draws: two equal-length arrays by stratum; squares and terms stay normal or overflow.",
    )


SOURCES = (OVERFLOW_SOURCE, FLUSH_SOURCE, EVIDENCE, "decision 0228")

SUMPRODUCT = register(
    Definition(
        function="SUMPRODUCT",
        params=(Param("array", "array", source=OVERFLOW_SOURCE),),
        value=_sumproduct,
        domain=domain_of(_sumproduct),
        draw=_sumproduct_draw(),
        variadic=True,
        sources=(*SOURCES, SNAP_SOURCE),
    )
)


def _pair_definition(function: str, term: Term) -> Definition:
    value = _pair_sum(term)
    return register(
        Definition(
            function=function,
            params=(
                Param("array_x", "array", source=PAIRING_SOURCE),
                Param("array_y", "array", source=PAIRING_SOURCE),
            ),
            value=value,
            domain=domain_of(value),
            draw=_pair_draw(function, value),
            sources=(*SOURCES, PAIRING_SOURCE),
        )
    )


SUMX2MY2 = _pair_definition("SUMX2MY2", _x2my2_term)
SUMX2PY2 = _pair_definition("SUMX2PY2", _x2py2_term)
SUMXMY2 = _pair_definition("SUMXMY2", _xmy2_term)
