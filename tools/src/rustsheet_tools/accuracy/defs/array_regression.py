"""LINEST, LOGEST, TREND and GROWTH (Gate A array regression, T-4096, decision 0271).

All four fit known_y by least squares on the x variables (LOGEST and GROWTH fit ln y) and return arrays, so a point is
one element read through INDEX (`=INDEX(LINEST({..},{..},TRUE,TRUE),3,1)`, `Definition.indexed`). known_y is a
one-row array literal (T-3586), known_x and TREND's and GROWTH's new_x 2-D literals (T-3686). The shapes are Excel's
(statistics.md "LINEST and LOGEST: shapes, ..." and "TREND: defaults, shapes, ..."): an x of y's shape is one variable;
otherwise a row y takes one variable per row of x, which must have y's column count; anything else is `#REF!`.

The math is exact: with the design X (a column of ones first when const is TRUE, then the k variables), the normal
equations G b = X'y are solved in rational arithmetic on the literals' doubles, so the coefficients, the fitted
values and the sums of squares are the exact least-squares ones. For LOGEST and GROWTH, ln y is taken at 60 digits
above the working precision and solved exactly from there (the conditioning the draws allow, 1E+7 per column, keeps
that error far below the 50 digits written). Excel's solve is not this one (its last digits are T-2006's), but the
exact fit is the target of the math either way.

The LINEST and LOGEST result (statistics.md "LINEST, LOGEST, GROWTH: probe findings", Microsoft's LINEST page): row 1
holds m_k, ..., m_1 and b (b = 0 for LINEST and 1 for LOGEST when const is FALSE; LOGEST's are e^coefficient); with
stats, row 2 the standard errors of the same coefficients (of the ln fit for LOGEST), `#N/A` for b when const is
FALSE; row 3 r^2 and se_y; row 4 F and df; row 5 ss_reg and ss_resid; rows 3-5 are `#N/A` right of the second column.
With df = n - k - (1 if const), ss_resid the exact residual sum of squares and ss_tot the sum of squares of y about its
mean (about 0 without const): ss_reg = ss_tot - ss_resid, r^2 = ss_reg / ss_tot, se_y = sqrt(ss_resid / df),
F = (ss_reg / k) / (ss_resid / df) and se_j = se_y * sqrt((G^-1)_jj). TREND is b + sum(m_j * new_x_j) at each new
point, GROWTH e^ of that in the ln fit; new_x takes any shape for a one-variable x (the result has its shape), else
needs k rows (the result is one row).

Domain (an Excel error, never a math value): known_y and known_x shapes that do not line up (`#REF!`, statistics.md
"LINEST, LOGEST, GROWTH: probe findings": unequal lengths); a known_y at or below 0 for LOGEST and GROWTH (`#NUM!`,
same entry and "LINEST, LOGEST and GROWTH in the gen-m9 sweep"); the `#N/A` cells of the stats block.
Not claimed, never drawn (NoValueError):
- a rank-deficient design or one within a relative 1E-7 of it (a column whose distance from the span of the others,
  intercept included, is below 1E-7 of its norm): Excel drops such a column with coefficient 0, and which one and at
  what cut-off is unprobed (T-2569; "LINEST and LOGEST: shapes, ..."). One point with const TRUE is such a design;
- the residual cells (the standard errors, se_y, F and ss_resid) of an exact fit or of one with no degree of freedom
  left: Excel's residual there is its rounding (`#NUM!` F for `LINEST({2,4,6},{1,2,3},TRUE,TRUE)`, F 3.1E+34 for
  linest-modes K106; "Near-exact fits are not snapped"), not the math's 0; and r^2 of a constant y (0/0);
- a TREND or GROWTH new_x of the wrong row count for several variables (unprobed for a row y), and an INDEX row or
  column outside the result.
The draws keep the data at or below 1E+50 in magnitude, so no sum of squares nears the double range.

Draws: typical, 2 to 12 points of one to three variables, half of them a noisy plane; tail, large offsets with a small
relative spread (1E-6 to 1E-1, the paired draws' `offset_series`) and near-exact planes (noise from 1E-12 for the
coefficients, 1E-6 for the residual cells); edge, Microsoft's examples, exact small-integer lines, a fit with no
degree of freedom left, unequal shapes, the stats block's `#N/A` cells, LOGEST's and GROWTH's non-positive y and
fits through the origin; extreme, 20 to 40 points of up to four variables from 1E-20 to 1E+20; log-uniform, up to 8
points from 1E-50 to 1E+50. The arrays reuse AGGREGATE's `array_of` (T-3693) and TREND's and GROWTH's new x the
paired draws' `forecast_x`.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import (
    StratumDraws,
    by_stratum,
    domain_of,
    fraction_to_mpf,
    signed_log_uniform,
)
from rustsheet_tools.accuracy.defs.aggregate_array import array_of
from rustsheet_tools.accuracy.defs.matrix import index_of, inverse, row_column
from rustsheet_tools.accuracy.defs.paired_regression import forecast_x, offset_series
from rustsheet_tools.accuracy.literal import Arg, Matrix, Row, round15
from rustsheet_tools.accuracy.model import (
    Definition,
    DomainError,
    Draw,
    MathArg,
    NoValueError,
    Param,
    register,
)
from rustsheet_tools.accuracy.sampler import STRATA, log_uniform

PROBE_SOURCE = "statistics.md#linest-logest-growth-probe-findings"
SHAPES_SOURCE = (
    "statistics.md#linest-and-logest-shapes-one-point-collinear-columns-exact-fits-and-check-order-t-1741"
)
SWEEP_SOURCE = "statistics.md#linest-logest-and-growth-in-the-gen-m9-sweep-t-2255"
LEFT_SOURCE = "statistics.md#linest-logest-and-growth-cells-left-after-t-1741-t-1742"
TREND_SOURCE = "statistics.md#trend-defaults-shapes-single-points-and-error-inputs"
ERRORS_SOURCE = "statistics.md#linest-and-logest-report-error-values-first-t-2455"
SOURCES = (
    PROBE_SOURCE,
    SHAPES_SOURCE,
    SWEEP_SOURCE,
    LEFT_SOURCE,
    TREND_SOURCE,
    ERRORS_SOURCE,
    "decision 0271",
)
COLLINEAR = Fraction(1, 10**14)  # (1E-7)^2: a column's squared relative distance from the others' span
LOG_GUARD_DPS = 60  # extra digits of ln y before the exact solve
MAX_CELLS = 120  # known_x literals per point, as the paired draws' pairs

Exact = list[list[Fraction]]


def to_fraction(v: mpmath.mpf) -> Fraction:
    """An mpf's exact binary value as a rational."""
    sign, man, exp, _ = v._mpf_  # `man_exp` drops the sign
    return (-1 if sign else 1) * Fraction(int(man)) * Fraction(2) ** int(exp)


def _row(a: MathArg) -> list[Fraction]:
    if not isinstance(a, tuple) or not all(isinstance(v, mpmath.mpf) for v in a):
        msg = "known_y is not a one-row array"
        raise TypeError(msg)
    return [Fraction(float(v)) for v in a if isinstance(v, mpmath.mpf)]


def _matrix(a: MathArg) -> Exact:
    if not isinstance(a, tuple) or not all(isinstance(row, tuple) for row in a):
        msg = "a matrix argument is not a matrix"
        raise TypeError(msg)
    rows = [[Fraction(float(v)) for v in row] for row in a if isinstance(row, tuple)]
    if not rows or len({len(r) for r in rows}) != 1:
        msg = "the matrix rows differ in length"
        raise NoValueError(msg)
    return rows


def variables(n: int, x: Exact) -> Exact:
    """The x variables, one list of n values each, for a one-row y of n values: an x of y's shape is one variable,
    else each row of an x with n columns is one; DomainError (`#REF!`) otherwise."""
    if len(x[0]) != n:
        msg = "#REF!: known_x does not line up with known_y"
        raise DomainError(msg)
    return [row[:] for row in x]


@dataclass(frozen=True)
class Fit:
    """The exact least-squares fit: `coefficients` are the intercept (when `const`) then m_1..m_k."""

    k: int
    n: int
    const: bool
    coefficients: tuple[Fraction, ...]
    inverse_diagonal: tuple[Fraction, ...]
    ss_resid: Fraction
    ss_tot: Fraction

    @property
    def df(self) -> int:
        """The residual degrees of freedom."""
        return self.n - self.k - int(self.const)

    def slope(self, j: int) -> Fraction:
        """m_j (1-based)."""
        return self.coefficients[j - 1 + int(self.const)]

    @property
    def intercept(self) -> Fraction:
        """b (0 without const)."""
        return self.coefficients[0] if self.const else Fraction(0)

    def predict(self, point: Sequence[Fraction]) -> Fraction:
        """b + sum(m_j x_j) at one point of the k variables."""
        return self.intercept + sum((self.slope(j) * v for j, v in enumerate(point, start=1)), Fraction(0))


def fit(ys: Sequence[Fraction], columns: Exact, const: bool) -> Fit:
    """The exact fit of `ys` on the variable `columns`; NoValueError for a design within 1E-7 of rank deficiency."""
    n = len(ys)
    design = ([[Fraction(1)] * n] if const else []) + columns
    gram = [[sum((a * b for a, b in zip(u, v, strict=True)), Fraction(0)) for v in design] for u in design]
    try:
        g_inverse = inverse(gram)
    except DomainError as err:
        msg = "a rank-deficient design: Excel drops a column (unprobed which, T-2569)"
        raise NoValueError(msg) from err
    for j, row in enumerate(gram):
        # the squared sine of column j's angle to the others' span is 1 / ((G^-1)_jj G_jj)
        if 1 / (g_inverse[j][j] * row[j]) < COLLINEAR:
            msg = "a column within 1E-7 of the others' span (Excel's drop cut-off is unprobed, T-2569)"
            raise NoValueError(msg)
    rhs = [sum((a * y for a, y in zip(u, ys, strict=True)), Fraction(0)) for u in design]
    coefficients = tuple(
        sum((g * r for g, r in zip(row, rhs, strict=True)), Fraction(0)) for row in g_inverse
    )
    fitted = [
        sum((c * u[i] for c, u in zip(coefficients, design, strict=True)), Fraction(0)) for i in range(n)
    ]
    mean = sum(ys, Fraction(0)) / n if const else Fraction(0)
    return Fit(
        k=len(columns),
        n=n,
        const=const,
        coefficients=coefficients,
        inverse_diagonal=tuple(g_inverse[j][j] for j in range(len(design))),
        ss_resid=sum(((y - f) ** 2 for y, f in zip(ys, fitted, strict=True)), Fraction(0)),
        ss_tot=sum(((y - mean) ** 2 for y in ys), Fraction(0)),
    )


def _flag(a: MathArg) -> bool:
    assert isinstance(a, bool)
    return a


def _log_ys(ys: Sequence[Fraction]) -> list[Fraction]:
    """ln y, each to `LOG_GUARD_DPS` digits above the working precision; DomainError (`#NUM!`) for y <= 0."""
    if any(y <= 0 for y in ys):
        msg = "#NUM!: a known_y at or below 0"
        raise DomainError(msg)
    with mpmath.workdps(mpmath.mp.dps + LOG_GUARD_DPS):
        return [to_fraction(mpmath.log(fraction_to_mpf(y))) for y in ys]


def _data(args: Sequence[MathArg], log: bool) -> tuple[list[Fraction], Exact]:
    """(y, the x variables), shapes checked, y in ln for LOGEST and GROWTH."""
    ys = _row(args[0])
    columns = variables(len(ys), _matrix(args[1]))
    return (_log_ys(ys) if log else ys), columns


def _residual(value: Fraction, f: Fit) -> Fraction:
    """A residual cell's value; NoValueError for an exact fit or no degree of freedom (Excel's rounding decides)."""
    if f.ss_resid == 0 or f.df <= 0:
        msg = "a residual cell of an exact fit (Excel's residual is its rounding)"
        raise NoValueError(msg)
    return value


def _stats_layout(row: int, column: int, columns: int, const: bool, stats: bool) -> None:
    """NoValueError outside the result, DomainError (`#N/A`) on the stats block's error cells."""
    if not (1 <= row <= (5 if stats else 1) and 1 <= column <= columns):
        msg = f"INDEX({row},{column}) is outside the result"
        raise NoValueError(msg)
    if (row >= 3 and column >= 3) or (row == 2 and column == columns and not const):
        msg = "#N/A: an empty cell of the stats block"
        raise DomainError(msg)


def _estimate(args: Sequence[MathArg], log: bool) -> mpmath.mpf:
    """LINEST (`log` False) or LOGEST: the INDEX element of the result."""
    const, stats = _flag(args[2]), _flag(args[3])
    row, column = index_of(args)
    ys, columns = _data(args, log)
    k = len(columns)
    _stats_layout(row, column, k + 1, const, stats)
    f = fit(ys, columns, const)
    coefficient = f.intercept if column == k + 1 else f.slope(k + 1 - column)
    if row == 1:
        return mpmath.exp(fraction_to_mpf(coefficient)) if log else fraction_to_mpf(coefficient)
    if row == 2:
        diagonal = f.inverse_diagonal[0 if column == k + 1 else k - column + int(const)]
        return mpmath.sqrt(fraction_to_mpf(_residual(f.ss_resid, f) / f.df * diagonal))
    return _summary(f, row, column)


def _summary(f: Fit, row: int, column: int) -> mpmath.mpf:
    """Rows 3-5 of the stats block, columns 1 and 2."""
    ss_reg = f.ss_tot - f.ss_resid

    def variance() -> Fraction:
        return _residual(f.ss_resid, f) / f.df

    def r_squared() -> mpmath.mpf:
        if f.ss_tot == 0:
            msg = "r^2 of a constant y (0/0)"
            raise NoValueError(msg)
        return fraction_to_mpf(ss_reg / f.ss_tot)

    cells: dict[tuple[int, int], Callable[[], mpmath.mpf]] = {
        (3, 1): r_squared,
        (3, 2): lambda: mpmath.sqrt(fraction_to_mpf(variance())),
        (4, 1): lambda: fraction_to_mpf(ss_reg / f.k / variance()),
        (4, 2): lambda: mpmath.mpf(f.df),
        (5, 1): lambda: fraction_to_mpf(ss_reg),
        (5, 2): lambda: fraction_to_mpf(_residual(f.ss_resid, f)),
    }
    return cells[row, column]()


def _new_points(new_x: Exact, k: int, row: int, column: int) -> list[Fraction]:
    """The k-variable point of the result's INDEX element (see the module doc for the shapes)."""
    if k == 1:
        cells = (len(new_x), len(new_x[0]))
        if not (1 <= row <= cells[0] and 1 <= column <= cells[1]):
            msg = f"INDEX({row},{column}) is outside the {cells[0]}x{cells[1]} result"
            raise NoValueError(msg)
        return [new_x[row - 1][column - 1]]
    if len(new_x) != k:
        msg = "new_x's rows are not the variables (unprobed for a row known_y)"
        raise NoValueError(msg)
    if not (row == 1 and 1 <= column <= len(new_x[0])):
        msg = f"INDEX({row},{column}) is outside the 1x{len(new_x[0])} result"
        raise NoValueError(msg)
    return [r[column - 1] for r in new_x]


def _predict(args: Sequence[MathArg], log: bool) -> mpmath.mpf:
    """TREND (`log` False) or GROWTH: the INDEX element of the result."""
    const = _flag(args[3])
    row, column = index_of(args)
    ys, columns = _data(args, log)
    point = _new_points(_matrix(args[2]), len(columns), row, column)
    value = fraction_to_mpf(fit(ys, columns, const).predict(point))
    return mpmath.exp(value) if log else value


def _linest(args: Sequence[MathArg]) -> mpmath.mpf:
    return _estimate(args, log=False)


def _logest(args: Sequence[MathArg]) -> mpmath.mpf:
    return _estimate(args, log=True)


def _trend(args: Sequence[MathArg]) -> mpmath.mpf:
    return _predict(args, log=False)


def _growth(args: Sequence[MathArg]) -> mpmath.mpf:
    return _predict(args, log=True)


# -- draws -------------------------------------------------------------------------------------

Data = tuple[Row, Matrix]
"""(known_y, known_x): a one-row y and a k-row x."""


def _plane(rng: random.Random, x: Matrix, noise: float, log: bool) -> Row:
    """y on a random plane over the variables `x` plus Gaussian noise `noise` times the plane's spread; for `log`,
    ln y on such a plane (kept within e^+-200)."""
    n, k = len(x[0]), len(x)
    if log:
        scale = [max(abs(v) for v in row) or 1.0 for row in x]
        slopes = [signed_log_uniform(rng, 1e-3, 1.0, 0.5) * 100 / (k * s) for s in scale]
        a = rng.uniform(-5, 5)
        exponents = [a + sum(m * row[i] for m, row in zip(slopes, x, strict=True)) for i in range(n)]
        spread = (max(exponents) - min(exponents)) or 1.0
        return tuple(round15(math.exp(e + noise * spread * rng.gauss(0, 1))) for e in exponents)
    a = signed_log_uniform(rng, 1e-3, 1e4, 0.5)
    slopes = [signed_log_uniform(rng, 1e-4, 1e4, 0.5) for _ in range(k)]
    clean = [a + sum(m * row[i] for m, row in zip(slopes, x, strict=True)) for i in range(n)]
    spread = (max(clean) - min(clean)) or abs(a) or 1.0
    return tuple(round15(c + noise * spread * rng.gauss(0, 1)) for c in clean)


def _values(rng: random.Random, n: int, lo: float, hi: float, log: bool) -> Row:
    """n data values: signed for a linear y or an x, positive for LOGEST's and GROWTH's y."""
    return array_of(rng, n, lo, hi, 0.0 if log else 0.3)


def _x(rng: random.Random, k: int, n: int, lo: float = 1e-3, hi: float = 1e6) -> Matrix:
    """k variables of n values each; a one-variable x is 1, 2, ..., n a quarter of the time (a sheet's periods)."""
    if k == 1 and rng.random() < 0.25:
        return (tuple(float(i) for i in range(1, n + 1)),)
    return tuple(array_of(rng, n, lo, hi) for _ in range(k))


def _k(rng: random.Random, most: int = 3) -> int:
    return min(most, rng.choices((1, 2, 3, 4), (60, 25, 10, 5))[0])


def _typical(rng: random.Random, log: bool, residual: bool) -> Data:
    del residual
    k = _k(rng)
    n = rng.randint(k + 2, 12)
    x = _x(rng, k, n)
    if rng.random() < 0.5:
        return _values(rng, n, 1e-3, 1e6, log), x
    return _plane(rng, x, log_uniform(rng, 1e-3, 1.0), log), x


def _tail(rng: random.Random, log: bool, residual: bool) -> Data:
    k = _k(rng, 2)
    n = rng.randint(k + 3, 12)
    if rng.random() < 0.5:  # large offsets, a relative spread of 1E-6 to 1E-1
        x = tuple(offset_series(rng, n, (1.0, 1e10), (1e-6, 1e-1)) for _ in range(k))
        return _plane(rng, x, log_uniform(rng, 1e-6, 1e-2), log), x
    x = _x(rng, k, n)
    return _plane(rng, x, log_uniform(rng, 1e-6 if residual else 1e-12, 1e-4), log), x


def _extreme(rng: random.Random, log: bool, residual: bool) -> Data:
    del residual
    k = _k(rng, 4)
    n = rng.randint(max(20, k + 2), min(40, MAX_CELLS // k))
    return _values(rng, n, 1e-20, 1e20, log), _x(rng, k, n, 1e-20, 1e20)


def _log_uniform(rng: random.Random, log: bool, residual: bool) -> Data:
    del residual
    k = _k(rng, 2)
    n = rng.randint(k + 2, 8)
    return _values(rng, n, 1e-50, 1e50, log), _x(rng, k, n, 1e-50, 1e50)


def _exact_line(rng: random.Random, log: bool) -> Data:
    """Small integers on an exact line (LOGEST and GROWTH: y = 2^(a + b x), exact doubles and literals)."""
    n = rng.randint(2, 10)
    xs = tuple(float(i) for i in rng.sample(range(-5, 6), n))  # 2^-20 to 2^20: exact 15-digit literals
    a, b = rng.randint(-5, 5), rng.choice([i for i in range(-3, 4) if i != 0])
    if log:
        return tuple(2.0 ** (a + b * x) for x in xs), (xs,)
    return tuple(float(a + b * x) for x in xs), (xs,)


StratumData = Callable[[random.Random, bool, bool], Data]
_STRATA: dict[str, StratumData] = {
    "typical": _typical,
    "tail": _tail,
    "extreme": _extreme,
    "log-uniform": _log_uniform,
}
# Microsoft's examples (support.microsoft.com, the functions' pages): LINEST's slope and intercept, LOGEST's and
# GROWTH's units sold over months 11 to 16, TREND's twelve monthly costs
MICROSOFT_LINEST: Data = ((1.0, 9.0, 5.0, 7.0), ((0.0, 4.0, 2.0, 3.0),))
MICROSOFT_GROWTH: Data = (
    (33100.0, 47300.0, 69000.0, 102000.0, 150000.0, 220000.0),
    (tuple(float(m) for m in range(11, 17)),),
)
TREND_COSTS = (133890, 135000, 135790, 137300, 138130, 139100, 139900, 141120, 141890, 143230, 144000, 145290)
MICROSOFT_TREND: Data = (tuple(float(c) for c in TREND_COSTS), (tuple(float(m) for m in range(1, 13)),))


def _edge_data(rng: random.Random, log: bool) -> Data:
    """The data shapes the edge stratum shares: Microsoft's example, an exact line, no degree of freedom left
    (k + 1 points), and sheet-like integers."""
    shape = rng.randrange(4)
    if shape == 0:
        return MICROSOFT_GROWTH if log else (MICROSOFT_LINEST if rng.random() < 0.5 else MICROSOFT_TREND)
    if shape == 1:
        return _exact_line(rng, log)
    if shape == 2:
        k = rng.randint(1, 3)
        return _values(rng, k + 1, 1e-3, 1e6, log), _x(rng, k, k + 1)
    n = rng.randint(3, 10)
    return tuple(float(rng.randint(1, 50)) for _ in range(n)), (tuple(float(i) for i in range(1, n + 1)),)


def _nonpositive(rng: random.Random, y: Row) -> Row:
    """`y` with one value replaced by 0 or a negative number (LOGEST's and GROWTH's `#NUM!`)."""
    out = list(y)
    out[rng.randrange(len(out))] = rng.choice((0.0, -1.0, round15(-rng.uniform(1e-3, 1e3))))
    return tuple(out)


def _unequal(rng: random.Random, log: bool) -> Data:
    """An x one value longer than y (`#REF!`)."""
    n = rng.randint(2, 8)
    return _values(rng, n, 1e-3, 1e6, log), (array_of(rng, n + 1, 1e-3, 1e6),)


def _each_stratum(drawn: Draw) -> StratumDraws:
    """A `by_stratum` table calling `drawn(stratum, rng)` for every stratum."""

    def one(stratum: str) -> Callable[[random.Random], list[Arg]]:
        return lambda rng: drawn(stratum, rng)

    return {stratum: one(stratum) for stratum in STRATA}


def _estimate_draw(log: bool) -> Draw:
    """LINEST's or LOGEST's draw: (known_y, known_x, const, stats, row, column)."""

    def cell(rng: random.Random, k: int, const: bool, stats: bool) -> tuple[int, int]:
        if not stats or rng.random() < 0.3:
            return 1, rng.randint(1, k + 1)
        row = rng.randint(2, 5)
        if row == 2:
            return row, rng.randint(1, k + 1 if const else k)
        return row, rng.randint(1, min(2, k + 1))

    def drawn(stratum: str, rng: random.Random) -> list[Arg]:
        const, stats = rng.random() < 0.75, rng.random() < 0.5
        if stratum == "edge":
            return edge(rng, const, stats)
        residual = stats and rng.random() < 0.7
        y, x = _STRATA.get(stratum, _log_uniform)(rng, log, residual)
        row, column = cell(rng, len(x), const, stats)
        return [y, x, const, stats, row, column]

    def edge(rng: random.Random, const: bool, stats: bool) -> list[Arg]:
        shape = rng.randrange(8)  # 0-2 aim at Excel's errors, 3-7 at the edge data
        if shape == 0:  # the stats block's #N/A cells
            y, x = _typical(rng, log, False)
            k = len(x)
            spots = [(r, c) for r in (3, 4, 5) for c in range(3, k + 2)] + ([] if const else [(2, k + 1)])
            if not spots:
                spots = [(2, k + 1)]
                const = False
            row, column = rng.choice(spots)
            return [y, x, const, True, row, column]
        if shape == 1:
            y, x = _unequal(rng, log)
            return [y, x, const, stats, 1, 1]
        if shape == 2 and log:
            y, x = _edge_data(rng, log)
            return [_nonpositive(rng, y), x, const, stats, 1, 1]
        y, x = _edge_data(rng, log)
        if shape == 3:  # through the origin
            const = False
        row, column = cell(rng, len(x), const, stats)
        return [y, x, const, stats, row, column]

    function = "LOGEST" if log else "LINEST"
    return by_stratum(
        _each_stratum(drawn),
        f"{function} draws: (known_y, known_x, const, stats) by stratum, then one element of the result.",
    )


def _new_x(rng: random.Random, x: Matrix) -> Matrix:
    """New points near the known x's (`forecast_x`): any shape for one variable, else k rows."""
    if len(x) == 1:
        rows, columns = (
            (1, rng.randint(1, 4)) if rng.random() < 0.75 else (rng.randint(2, 3), rng.randint(1, 3))
        )
        return tuple(tuple(forecast_x(rng, x[0]) for _ in range(columns)) for _ in range(rows))
    columns = rng.randint(1, 4)
    return tuple(tuple(forecast_x(rng, row) for _ in range(columns)) for row in x)


def _predict_draw(log: bool) -> Draw:
    """TREND's or GROWTH's draw: (known_y, known_x, new_x, const, row, column)."""

    def point(rng: random.Random, y: Row, x: Matrix, const: bool) -> list[Arg]:
        new_x = _new_x(rng, x)
        rows = len(new_x) if len(x) == 1 else 1  # several variables: one row of results
        return [y, x, new_x, const, rng.randint(1, rows), rng.randint(1, len(new_x[0]))]

    def drawn(stratum: str, rng: random.Random) -> list[Arg]:
        const = rng.random() < 0.75
        if stratum == "edge":
            return edge(rng, const)
        y, x = _STRATA.get(stratum, _log_uniform)(rng, log, False)
        return point(rng, y, x, const)

    def edge(rng: random.Random, const: bool) -> list[Arg]:
        shape = rng.randrange(6)  # 0 and 1 aim at Excel's errors, 2-5 at the edge data
        if shape == 0:
            y, x = _unequal(rng, log)
            return [y, x, ((1.0,),), const, 1, 1]
        if shape == 1 and log:
            y, x = _edge_data(rng, log)
            return point(rng, _nonpositive(rng, y), x, const)
        if shape == 2:  # one point through the origin (TREND(5,2,4,FALSE) is unprobed: the math is 10)
            y, x = _values(rng, 1, 1e-3, 1e6, log), (array_of(rng, 1, 1e-3, 1e6),)
            return point(rng, y, x, False)
        y, x = _edge_data(rng, log)
        if shape == 3:
            const = False
        return point(rng, y, x, const)

    function = "GROWTH" if log else "TREND"
    return by_stratum(
        _each_stratum(drawn),
        f"{function} draws: (known_y, known_x, new_x, const) by stratum, then one element of the result.",
    )


def _definition(
    function: str, value: Callable[[Sequence[MathArg]], mpmath.mpf], *, predict: bool, log: bool
) -> Definition:
    data = (
        Param("known_y's", "array", source=PROBE_SOURCE),
        Param("known_x's", "matrix", source=SHAPES_SOURCE),
    )
    flags = (
        (Param("new_x's", "matrix", source=TREND_SOURCE), Param("const", "bool", source=PROBE_SOURCE))
        if predict
        else (Param("const", "bool", source=PROBE_SOURCE), Param("stats", "bool", source=PROBE_SOURCE))
    )
    return register(
        Definition(
            function=function,
            params=(*data, *flags, *row_column(PROBE_SOURCE)),
            value=value,
            domain=domain_of(value),
            draw=_predict_draw(log) if predict else _estimate_draw(log),
            indexed=True,
            literal_arrays=True,
            sources=SOURCES,
        )
    )


LINEST = _definition("LINEST", _linest, predict=False, log=False)
LOGEST = _definition("LOGEST", _logest, predict=False, log=True)
TREND = _definition("TREND", _trend, predict=True, log=False)
GROWTH = _definition("GROWTH", _growth, predict=True, log=True)
