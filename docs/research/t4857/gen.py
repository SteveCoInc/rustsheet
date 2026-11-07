"""T-4857: the F family's equivalence-class probes (owner request, 2026-11-07; ANALYSIS.md).

Writes two cases, both seeded and reproducible:
- eval/suites/probes-numeric/cases/t4857-f-classes.case.json: numeric ranges. Sections (the `hypotheses` map
  tags every cell with its section and classes):
  - `pair-tail`, `pair-pdf`, `pair-inv`: a greedy covering array over the numeric classes of `fclasses.py`. Every
    (df1, df2) class pair gets one row per group, and each row's x (or p) is the candidate that covers the most
    not-yet-covered (x class, df class) and (kernel or density or path, position) pairs;
  - `domain`: every error or degenerate class of each argument (df < 1, a fractional df below 2, df above 1E10,
    x < 0, -0, 0, a subnormal x three ways, DBL_MIN, DBL_MAX, p outside [0, 1], p = 0 and 1, 1 - 2^-53);
  - `edge-x`: the exact double where the kernel's branch (or the density's regime) changes along x at a fixed df
    pair, found by bisection on the doubles, with points at it, one ulp either side, and 1E-9, 1E-5 and 1E-2
    relative either side (constant cells, so every double is exact);
  - `edge-df`: each df threshold just below, at and above (79/80/81, 30/31, 15/16/17, 199/200/201, 99999/1E5/100001,
    1999999/2E6/2000001, 2E7 +- 1, 1E10 and 1E10+1, 1.9999999 against 2) at fixed x, and `edge-p`: p across the
    quantile's switches (1E-20, 1E-15, 0.5) and the double range;
  - `sweep-*`: dense 1-D sweeps across the edges most likely to hide a jump or a kink (ANALYSIS.md section 4);
  - `sym`: F.DIST(x,d1,d2,TRUE) beside F.DIST.RT(1/x,d2,d1), and F.INV(p,d1,d2) beside F.INV.RT(p,d2,d1);
  - `zero`: `=(F.DIST(..))=0` and `=ISNUMBER(..)` companions where a result may be subnormal.
- eval/suites/probes/cases/t4857-f-kinds.case.json: argument kinds for every argument of every function (number,
  numeric text, text, empty text, TRUE/FALSE typed and through a reference, an empty cell, errors typed and through
  a reference, a one-cell and a multi-cell range, single- and multi-element arrays, the dynamic-array lift, an
  omitted argument), the cumulative flag's kinds, and F.TEST/FTEST argument kinds and shapes.

Literals carry at most 15 significant digits (Excel reads formula literals at 15, numbers.md "Formula number
literals are read at 15 significant digits"); a value that needs more is a typed constant cell in column B, which
xlsx keeps to the bit.

    cd tools && uv run python ../docs/research/t4857/gen.py [--write]
"""

from __future__ import annotations

import argparse
import json
import math
import random
import struct
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import fclasses as F

SEED = 4857
ROOT = Path(__file__).resolve().parents[3]
NUMERIC = ROOT / "eval/suites/probes-numeric/cases/t4857-f-classes.case.json"
KINDS = ROOT / "eval/suites/probes/cases/t4857-f-kinds.case.json"

# Each df class's draw range (truncated values); singletons are the edge values themselves.
DF_RANGE: dict[str, tuple[float, float]] = {
    "1": (1, 1),
    "frac1-2": (1.01, 1.99),
    "2": (2, 2),
    "3-15": (3, 15),
    "16": (16, 16),
    "17-29": (17, 29),
    "30": (30, 30),
    "31": (31, 31),
    "32-78": (32, 78),
    "79": (79, 79),
    "80": (80, 80),
    "81": (81, 81),
    "82-199": (82, 199),
    "200": (200, 200),
    "201": (201, 201),
    "202-1e5": (202, 99999),
    "1e5-2e6": (1e5, 1999999),
    "2e6-2e7": (2e6, 2e7),
    "2e7-1e9": (20000001, 999999999),
    "1e9-1e10": (1e9, 9999999999),
    "1e10": (1e10, 1e10),
}
NUMERIC_DF = tuple(DF_RANGE)
# Representative tail probabilities (log10 range) of each position band.
BAND_LOG10P = {"deep": (-320.0, -301.0), "far": (-290.0, -17.0), "tail": (-15.5, -3.2), "body": (-2.6, -0.31)}
XMAG_RANGE = {
    "dblmin": None,
    "1e-307..1e-100": (-306.5, -100.5),
    "1e-100..1e-16": (-99.5, -16.5),
    "1e-16..1e-3": (-15.5, -3.5),
    "1e-3..0.1": (-2.9, -1.1),
    "0.1..10": (-0.9, 0.9),
    "10..1e3": (1.1, 2.9),
    "1e3..1e16": (3.5, 15.5),
    "1e16..1e100": (16.5, 99.5),
    "1e100..1e300": (100.5, 299.5),
    "1e300..max": (300.2, 307.9),
}
P_DRAW = {
    "0": None,
    "dblmin": None,
    "1e-307..1e-200": (-306.5, -200.5),
    "1e-200..1e-20": (-199.5, -20.5),
    "1e-20..1e-15": (-19.8, -15.2),
    "1e-15..1e-3": (-14.8, -3.2),
    "1e-3..0.5": (-2.9, -0.32),
    "0.5": None,
    "0.5..1-1e-3": (-2.9, -0.32),  # log10 of 1 - p
    "1-1e-3..1-1e-15": (-14.8, -3.2),
    "1-1e-15..1": (-15.9, -15.05),
    "1": None,
}


# --------------------------------------------------------------------------- literals


def lit(v: float) -> str:
    """A formula literal of v, which must already be a 15-significant-digit value."""
    if v == 0:
        return "-0" if math.copysign(1, v) < 0 else "0"
    text = repr(v).upper().replace("E+", "E+")
    if "E" in text:
        mant, exp = text.split("E")
        mant = mant.removesuffix(".0")
        text = f"{mant}E{exp}"
    else:
        text = text.removesuffix(".0")
    assert float(text) == v, (v, text)
    return text


def r15(v: float) -> float:
    """v rounded to 15 significant digits: the double Excel reads from its literal."""
    return float(f"{v:.15g}")


def next_up(x: float) -> float:
    return math.nextafter(x, math.inf)


def next_down(x: float) -> float:
    return math.nextafter(x, -math.inf)


def _ordered(x: float) -> int:
    (bits,) = struct.unpack("<q", struct.pack("<d", x))
    return bits


def _from_ordered(n: int) -> float:
    (v,) = struct.unpack("<d", struct.pack("<q", n))
    return float(v)


# --------------------------------------------------------------------------- draws


def draw_df(rng: random.Random, cls: str, frac: bool = False) -> float:
    lo, hi = DF_RANGE[cls]
    if cls == "frac1-2":
        return round(rng.uniform(lo, hi), 4)
    v = lo if lo == hi else float(round(math.exp(rng.uniform(math.log(lo), math.log(hi)))))
    v = min(max(v, lo), hi)
    if frac and v < 1e9 and cls not in ("1", "2"):
        v += round(rng.uniform(0.05, 0.95), 2)
    return v


def x_for_band(job: tuple[str, str, float, float, float]) -> float | None:
    """An x whose smaller tail is in `band`'s range on `side`, or None where no double x reaches it."""
    band, side, d1, d2, u = job
    lo, hi = BAND_LOG10P[band]
    target = 10 ** (lo + (hi - lo) * u)
    if target < 1e-323:
        target = 1e-323
    lx_lo, lx_hi = math.log(F.DBL_MIN), math.log(F.DBL_MAX / max(d1, 1.0)) - 1e-9
    upper = side == "hi"

    def smaller(lx: float) -> float:
        cdf, sf, _ = F.tails(math.exp(lx), d1, d2, 15)
        return sf if upper else cdf

    # The tail is monotone in ln x: the lower tail rises, the upper falls.
    a, b = lx_lo, lx_hi
    end = smaller(b if upper else a)
    if end > target and target > 0:
        # The tail at the extreme double still exceeds the target: band unreachable at the far end.
        if F.x_position(*_both(math.exp(b if upper else a), d1, d2)) != f"{band}-{side}":
            return None
        return r15(math.exp(b if upper else a))
    for _ in range(34):
        m = (a + b) / 2
        v = smaller(m)
        if (v > target) == upper:
            a = m
        else:
            b = m
    return r15(math.exp((a + b) / 2))


def _both(x: float, d1: float, d2: float) -> tuple[float, float]:
    cdf, sf, _ = F.tails(x, d1, d2, 15)
    return cdf, sf


def x_for_mag(rng: random.Random, cls: str, d1: float) -> float:
    if cls == "dblmin":
        return F.DBL_MIN
    lo, hi = XMAG_RANGE[cls]  # type: ignore[misc]
    if cls == "1e300..max":
        hi = min(hi, math.log10(F.DBL_MAX / d1) - 0.05)
        lo = min(lo, hi - 0.1)
    return r15(10 ** rng.uniform(lo, hi))


def p_for_class(rng: random.Random, cls: str) -> float:
    if cls == "0":
        return 0.0
    if cls == "dblmin":
        return F.DBL_MIN
    if cls == "0.5":
        return 0.5
    if cls == "1":
        return 1.0
    lo, hi = P_DRAW[cls]  # type: ignore[misc]
    v = 10 ** rng.uniform(lo, hi)
    if cls in ("0.5..1-1e-3", "1-1e-3..1-1e-15", "1-1e-15..1"):
        return 1.0 - v  # an exact double, written as a constant cell when 15 digits cannot hold it
    return r15(v)


# --------------------------------------------------------------------------- the case builder


class Case:
    """Cells in column A (formula), B (an exact constant the formula reads), C (a companion)."""

    def __init__(self) -> None:
        self.cells: dict[str, object] = {}
        self.hyp: dict[str, str] = {}
        self.row = 0

    def add(self, formula: str, tag: str, const: float | None = None, companion: str | None = None) -> int:
        self.row += 1
        r = self.row
        if const is not None:
            self.cells[f"B{r}"] = const
            formula = formula.replace("{B}", f"B{r}")
        self.cells[f"A{r}"] = formula
        self.hyp[f"Calc!A{r}"] = tag
        if companion is not None:
            self.cells[f"C{r}"] = companion.replace("{A}", f"A{r}")
            self.hyp[f"Calc!C{r}"] = tag + "/companion"
        return r


def arg(v: float) -> tuple[str, float | None]:
    """The formula text of v, and the constant cell it needs (when 15 digits cannot hold it)."""
    if v == r15(v) and not (0 < abs(v) < F.DBL_MIN):
        return lit(v), None
    return "{B}", v


def call(
    case: Case, function: str, first: float, d1: float, d2: float, tag: str, companion: bool = False
) -> None:
    text, const = arg(first)
    fn = function
    if function == "F.DIST/pdf":
        formula = f"=F.DIST({text},{lit(d1)},{lit(d2)},FALSE)"
        fn = "F.DIST"
    elif function == "F.DIST":
        formula = f"=F.DIST({text},{lit(d1)},{lit(d2)},TRUE)"
    else:
        formula = f"={fn}({text},{lit(d1)},{lit(d2)})"
    comp = None
    if companion:
        inner = formula[1:]
        comp = f"=({inner})=0"
    case.add(formula, tag, const, comp)


# --------------------------------------------------------------------------- sections


TAIL_FUNCS = ("F.DIST", "F.DIST.RT", "FDIST")
INV_FUNCS = ("F.INV", "F.INV.RT", "FINV")


def pairwise_x(case: Case, rng: random.Random, group: str, pool: ProcessPoolExecutor) -> None:
    """One row per (df1, df2) class pair; x chosen greedily over band and magnitude candidates."""
    rows = []
    for c1 in NUMERIC_DF:
        for c2 in NUMERIC_DF:
            frac = rng.random() < 0.15
            rows.append((c1, c2, draw_df(rng, c1, frac), draw_df(rng, c2, frac and rng.random() < 0.5)))
    jobs = []
    for _c1, _c2, r1, r2 in rows:
        d1, d2 = math.trunc(r1), math.trunc(r2)
        for band in BAND_LOG10P:
            for side in ("lo", "hi"):
                jobs.append((band, side, float(d1), float(d2), rng.random()))
    xs = _cached_map(pool, jobs)
    covered: set[tuple[str, str, str]] = set()
    per = 2 * len(BAND_LOG10P)
    for i, (c1, c2, r1, r2) in enumerate(rows):
        d1, d2 = float(math.trunc(r1)), float(math.trunc(r2))
        cands = [x for x in xs[i * per : (i + 1) * per] if x is not None]
        cands += [x_for_mag(rng, m, d1) for m in XMAG_RANGE]
        best, best_score, best_keys = None, -1, []
        for x in cands:
            if not math.isfinite(d1 * x):
                continue
            cdf, sf, _ = F.tails(x, d1, d2, 15)
            pos = F.x_position(cdf, sf)
            mag = F.x_magnitude(x)
            other = F.density_class(d1, d2, x) if group == "pdf" else F.kernel_class(d1, d2, x)
            keys = [
                ("xmag-df1", mag, c1),
                ("xmag-df2", mag, c2),
                ("xpos-df1", pos, c1),
                ("xpos-df2", pos, c2),
                ("k-xpos", other, pos),
            ]
            score = sum(k not in covered for k in keys) + rng.random() * 0.1
            if score > best_score:
                best, best_score, best_keys = x, score, keys
        assert best is not None
        covered.update(best_keys)
        if group == "pdf":
            fn = "F.DIST/pdf"
        else:
            cdf, sf, _ = F.tails(best, d1, d2, 15)
            # Rotate the three tail names; the smaller tail on two rows in three keeps the deep bands visible.
            fn = TAIL_FUNCS[i % 3] if i % 3 == 0 else ("F.DIST" if cdf <= sf else TAIL_FUNCS[i % 3])
        call(case, fn, best, r1, r2, f"pair-{group}/{c1}/{c2}")


CACHE: dict[str, float | None] = {}


def _cached_map(pool: ProcessPoolExecutor, jobs: list) -> list[float | None]:
    """`x_for_band` over `jobs`, reusing a `--cache` file's answers (a rerun takes seconds, not minutes)."""
    todo = [j for j in jobs if repr(j) not in CACHE]
    for j, x in zip(todo, pool.map(x_for_band, todo, chunksize=8), strict=True):
        CACHE[repr(j)] = x
    return [CACHE[repr(j)] for j in jobs]


def pairwise_p(case: Case, rng: random.Random) -> None:
    covered: set[tuple[str, str, str]] = set()
    i = 0
    for c1 in NUMERIC_DF:
        for c2 in NUMERIC_DF:
            r1, r2 = draw_df(rng, c1), draw_df(rng, c2)
            d1, d2 = math.trunc(r1), math.trunc(r2)
            fn = INV_FUNCS[i % 3]
            i += 1
            best, best_score, best_keys = 0.0, -1.0, []
            for pc in P_DRAW:
                p = p_for_class(rng, pc)
                path = F.quantile_path(p, d1, d2, fn != "F.INV")
                keys = [("p-df1", pc, c1), ("p-df2", pc, c2), ("path-p", path, pc)]
                score = sum(k not in covered for k in keys) + rng.random() * 0.1
                if score > best_score:
                    best, best_score, best_keys = p, score, keys
            covered.update(best_keys)
            call(case, fn, best, r1, r2, f"pair-inv/{c1}/{c2}")


def domain(case: Case) -> None:
    """Each error or degenerate class, on every function it applies to (ANALYSIS.md section 2)."""
    for fn in ("F.DIST", "F.DIST/pdf", "F.DIST.RT", "FDIST"):
        for d1, d2 in (
            (0.999, 5),
            (5, 0.5),
            (0, 3),
            (-1, 3),
            (1.5, 7),
            (7, 1.999),
            (10000000000.5, 3),
            (3, 10000000001),
        ):
            call(case, fn, 1.5, d1, d2, "domain/df")
        for x in (-1.0, -1e-300, -0.0, 0.0, F.DBL_MIN, 1.7976931348623157e308, 1e308):
            call(case, fn, x, 3, 7, "domain/x")
        # A subnormal x three ways: a literal (number text below DBL_MIN reads as 0), 2^-1074, EXP(-720).
        for text in ("1E-310", "2^-1074", "EXP(-720)", "2^-1022/2"):
            case.add(_formula(fn, 3, 7, text), "domain/x-subnormal", companion="=ISNUMBER({A})")
    for fn in INV_FUNCS:
        for p in (-1e-300, -0.0, 0.0, F.DBL_MIN, 0.5, 1.0 - 2**-53, 1.0, 1.0 + 2**-52, 2.0):
            call(case, fn, p, 5, 7, "domain/p")
        for d1, d2 in ((0.999, 5), (5, 0.5), (1.5, 7), (7, 1.999), (10000000000.5, 3), (3, 10000000001)):
            call(case, fn, 0.3, d1, d2, "domain/df")
        for text in ("1E-310", "2^-1074", "EXP(-720)"):
            case.add(f"={fn}({text},5,7)", "domain/p-subnormal", companion="=ISNUMBER({A})")


def find_edges(d1: float, d2: float, label, lo: float = -705.0, hi: float = 709.0, n: int = 1500):
    """(x_before, x_after, label_before, label_after) for every change of `label(x)` along x, the
    two adjacent doubles found by bisection on the bit patterns of positive doubles."""
    out = []
    xs = [math.exp(lo + (hi - lo) * i / n) for i in range(n + 1)]
    xs = [x for x in xs if math.isfinite(d1 * x) and x >= F.DBL_MIN]
    labels = [label(x) for x in xs]
    for i in range(len(xs) - 1):
        if labels[i] == labels[i + 1]:
            continue
        a, b = _ordered(xs[i]), _ordered(xs[i + 1])
        la = labels[i]
        while b - a > 1:
            m = (a + b) // 2
            if label(_from_ordered(m)) == la:
                a = m
            else:
                b = m
        out.append((_from_ordered(a), _from_ordered(b), la, label(_from_ordered(b))))
    return out


def tail_label(d1: float, d2: float):
    def label(x: float) -> str:
        d = F.kernel_detail(d1, d2, x)
        k = F.kernel_class(d1, d2, x)
        return f"{k}{'/swap' if d is not None and d.swapped else ''}"

    return label


def pdf_label(d1: float, d2: float):
    def label(x: float) -> str:
        c = F.density_class(d1, d2, x)
        return "-".join(c.split("-")[:3])  # side, log1p form, constant: the engine's branches, not |P|

    return label


EDGE_DF = (
    (1, 1),
    (1, 7),
    (1, 400),
    (3, 5),
    (3, 2000),
    (5, 1),
    (6, 31),
    (9, 60),
    (12, 100),
    (20, 9),
    (24, 79),
    (31, 79),
    (40, 12),
    (60, 30),
    (79, 79),
    (80, 80),
    (100, 300),
    (150, 31),
    (201, 201),
    (300, 1000),
    (1000, 3),
    (2000, 50),
)
STEPS = (1e-9, 1e-3)


def edges_x(case: Case, rng: random.Random) -> None:
    """Boundary points around each branch change along x (tail kernel and density regime)."""
    seen: dict[tuple[str, str], int] = {}
    for group in ("tail", "pdf"):
        for d1, d2 in EDGE_DF:
            maker = tail_label if group == "tail" else pdf_label
            for xa, xb, la, lb in find_edges(float(d1), float(d2), maker(float(d1), float(d2))):
                kind = (group, f"{la}>{lb}")
                if seen.get(kind, 0) >= 2:
                    continue
                seen[kind] = seen.get(kind, 0) + 1
                points = [next_down(xa), xa, xb, next_up(xb)]
                for s in STEPS:
                    points += [xa * (1 - s), xb * (1 + s)]
                for x in sorted(set(points)):
                    if group == "pdf":
                        fn = "F.DIST/pdf"
                    else:
                        cdf, sf, _ = F.tails(x, d1, d2, 15)
                        fn = "F.DIST" if cdf <= sf else ("F.DIST.RT" if rng.random() < 0.7 else "FDIST")
                    tag = f"edge-x/{group}/{la}>{lb}/{d1},{d2}"
                    case.add(_formula(fn, d1, d2), tag, x)


def _formula(fn: str, d1: float, d2: float, first: str = "{B}") -> str:
    if fn == "F.DIST/pdf":
        return f"=F.DIST({first},{lit(d1)},{lit(d2)},FALSE)"
    if fn == "F.DIST":
        return f"=F.DIST({first},{lit(d1)},{lit(d2)},TRUE)"
    return f"={fn}({first},{lit(d1)},{lit(d2)})"


DF_EDGES = (
    (14, 15, 16, 17),
    (29, 30, 31, 32),
    (78, 79, 80, 81),
    (199, 200, 201),
    (99999, 1e5, 100001),
    (1999999, 2e6, 2000001),
    (19999999, 2e7, 20000001),
    (9999999999, 1e10, 10000000000.9, 10000000001),
)


def edges_df(case: Case) -> None:
    """Each df threshold below, at and above, in either position, at an x near the mean and one in the tail."""
    for values in DF_EDGES:
        quantile_edge = values[1] in (1e5, 1e10)
        for other in (5, 1000):
            for pos in (1, 2):
                below = values[1] - 0.5 if values[1] < 1e9 else values[1] + 0.5
                for v in (*values, below):
                    d1, d2 = (v, other) if pos == 1 else (other, v)
                    x = r15(1.0 + 1.0 / max(math.trunc(d2), 3)) if pos == 1 else 1.05
                    for fn in ("F.DIST.RT", "F.DIST/pdf") if other == 5 else ("F.DIST.RT",):
                        case.add(_formula(fn, d1, d2, lit(x)), f"edge-df/{fn}/{values[1]:g}/pos{pos}/{other}")
                    if quantile_edge:
                        for fn in ("F.INV", "F.INV.RT"):
                            case.add(f"={fn}(0.05,{lit(d1)},{lit(d2)})", f"edge-df/{fn}/{values[1]:g}/pos{pos}/{other}")
    # 1.9999999 truncates to 1 (a shape of 1/2), 2 is the closed form.
    for d in (1.9999999, 2, 2.0000001, 0.9999999, 1):
        for fn in ("F.DIST", "F.DIST.RT", "F.DIST/pdf"):
            case.add(_formula(fn, d, 7, "2.5"), f"edge-df/{fn}/2-trunc")
            case.add(_formula(fn, 7, d, "2.5"), f"edge-df/{fn}/2-trunc")


def edges_p(case: Case) -> None:
    """p across the quantile's switches and the ends of the double range."""
    centres = (1e-20, 1e-15, 0.5, F.DBL_MIN, 1e-300)
    for p0 in centres:
        pts = [next_down(p0), p0, next_up(p0)] + [p0 * (1 - s) for s in STEPS] + [p0 * (1 + s) for s in STEPS]
        for p in sorted(set(pts)):
            for fn, d1, d2 in (("F.INV", 5, 7), ("F.INV.RT", 5, 7), ("FINV", 30, 3)):
                text, const = arg(p)
                case.add(f"={fn}({text},{d1},{d2})", f"edge-p/{p0:g}/{fn}", const)
    for q in (2**-53, 2**-52, 1e-15, 1e-16, 3e-16):
        for fn in INV_FUNCS:
            case.add(f"={fn}({{B}},5,7)", f"edge-p/1-{q:g}/{fn}", 1.0 - q)


def sweep_x(
    case: Case, name: str, fn: str, d1: float, d2: float, centre: float, half_width: float, n: int
) -> None:
    for i in range(n):
        x = centre * math.exp(-half_width + 2 * half_width * i / (n - 1))
        case.add(_formula(fn, d1, d2), f"sweep-{name}/{d1},{d2}", x)


def sweeps(case: Case) -> None:
    """Dense 1-D sweeps across the edges ANALYSIS.md section 4 ranks highest."""
    for d1, d2, fn in ((10, 30, "F.DIST"), (50, 20, "F.DIST.RT"), (6, 60, "F.DIST")):
        edges = [e for e in find_edges(d1, d2, tail_label(d1, d2)) if "BpserLarge" in e[2] + e[3]]
        for xa, _xb, _la, _lb in edges[:1]:
            sweep_x(case, "b0x0-0.7", fn, d1, d2, xa, 0.25, 40)
    for d1, d2, fn in ((24, 50, "F.DIST.RT"), (60, 12, "F.DIST")):
        edges = [
            e for e in find_edges(d1, d2, tail_label(d1, d2)) if "L140BupBpser" in e[2] and "Bgrat" in e[3]
        ]
        for xa, _xb, _la, _lb in edges[:1]:
            sweep_x(case, "x0-0.7", fn, d1, d2, xa, 0.25, 50)
    # The bratio swap (lambda's sign) and the Cephes flip at the mean, on bfrac.
    for d1, d2 in ((100, 300), (300, 100)):
        sweep_x(case, "mean", "F.DIST.RT", d1, d2, d2 / (d2 - 2), 0.06, 40)
    # df1 across 80 (b0 = 40) and 30/31 (a0 = 15) at a fixed x, both tails.
    for x, d2, fn in ((1.05, 20, "F.DIST.RT"), (1.02, 1000, "F.DIST.RT"), (0.6, 20, "F.DIST")):
        for d1 in range(70, 91):
            case.add(
                f"={'F.DIST' if fn == 'F.DIST' else fn}({lit(x)},{d1},{d2}{',TRUE' if fn == 'F.DIST' else ''})",
                f"sweep-df1-80/{x},{d2}",
            )
    for d1 in range(22, 40):
        case.add(f"=F.DIST.RT(3,{d1},10)", "sweep-df1-30/3,10")
    # df2 across 2E6 (large_shape_beta's MIN_SHAPE) with df1 outside T-4434's 3..400.
    for d1 in (1, 2, 1000):
        for i in range(30):
            d2 = round(10 ** (6.0 + 0.6 * i / 29))
            case.add(f"=F.DIST.RT(1.3,{d1},{d2})", f"sweep-df2-2e6/{d1}")
    # Tiny p, both tails, through the 1E-20 and 1E-15 switches and to the bottom of the double range.
    for fn, d1, d2 in (("F.INV", 5, 7), ("F.INV.RT", 5, 7), ("F.INV", 30, 3), ("F.INV.RT", 3, 30)):
        for k in range(1, 308, 9):
            case.add(f"={fn}(1E-{k},{d1},{d2})", f"sweep-p/{fn}/{d1},{d2}")
    # The density across fl(m x) = n (side switch) and r = 0.375 (log1p form).
    for d1, d2 in ((5, 9), (40, 8)):
        sweep_x(case, "pdf-side", "F.DIST/pdf", d1, d2, d2 / d1, 0.6, 40)
    # df = 2 closed form's y0 = 0.9 cut along x.
    for d1, d2 in ((2, 9), (9, 2)):
        edges = [e for e in find_edges(d1, d2, tail_label(d1, d2)) if "closed-a0=1" in e[2] + e[3]]
        for xa, _xb, _la, _lb in edges[:1]:
            sweep_x(case, "closed-y0", "F.DIST.RT", d1, d2, xa, 0.02, 30)


def symmetry(case: Case, rng: random.Random) -> None:
    for i in range(70):
        d1 = draw_df(rng, rng.choice(NUMERIC_DF[:16]))
        d2 = draw_df(rng, rng.choice(NUMERIC_DF[:16]))
        x = r15(10 ** rng.uniform(-6, 6))
        case.add(f"=F.DIST({lit(x)},{lit(d1)},{lit(d2)},TRUE)", f"sym/{i}/cdf")
        case.add(f"=F.DIST.RT(1/{lit(x)},{lit(d2)},{lit(d1)})", f"sym/{i}/rt")
    for i in range(30):
        d1 = draw_df(rng, rng.choice(NUMERIC_DF[:16]))
        d2 = draw_df(rng, rng.choice(NUMERIC_DF[:16]))
        p = r15(10 ** rng.uniform(-12, -0.31))
        case.add(f"=F.INV({lit(p)},{lit(d1)},{lit(d2)})", f"sym-inv/{i}/inv")
        case.add(f"=F.INV.RT({lit(p)},{lit(d2)},{lit(d1)})", f"sym-inv/{i}/invrt")


def zeros(case: Case) -> None:
    """Results that may be subnormal, each with an `=(..)=0` companion (decision 0042)."""
    for d1, d2 in ((40, 1), (3, 5), (100, 100), (1, 1000)):
        for k in range(300, 312, 2):
            x = F.f_quantile(10.0**-k, float(d1), float(d2), upper=False, dps=30)
            if x is None:
                continue
            call(case, "F.DIST", r15(x), d1, d2, f"zero/cdf/{d1},{d2}/1e-{k}", companion=True)
        for k in range(300, 312, 3):
            x = F.f_quantile(10.0**-k, float(d1), float(d2), upper=True, dps=30)
            if x is None or not math.isfinite(d1 * x):
                continue
            call(case, "F.DIST.RT", r15(x), d1, d2, f"zero/rt/{d1},{d2}/1e-{k}", companion=True)


def numeric_case(pool: ProcessPoolExecutor) -> dict:
    rng = random.Random(SEED)
    case = Case()
    pairwise_x(case, rng, "tail", pool)
    pairwise_x(case, rng, "pdf", pool)
    pairwise_p(case, rng)
    domain(case)
    edges_x(case, rng)
    edges_df(case)
    edges_p(case)
    sweeps(case)
    symmetry(case, rng)
    zeros(case)
    n = sum(1 for k in case.cells if k[0] in "AC")
    return {
        "format": "rustsheet-case/v1",
        "description": (
            f"T-4857 (owner request 2026-11-07): F family equivalence-class probe, {n} cells: pairwise coverage of the "
            "numeric classes of x, df1, df2 and p, error and degenerate classes, exact-double boundary points at every "
            "kernel branch change along x, df thresholds, p switches, dense sweeps, symmetry pairs and =0 companions. "
            f"Generator docs/research/t4857/gen.py, seed {SEED}; classes docs/research/t4857/fclasses.py; "
            "ANALYSIS.md. Decision 0267 item 5 exception: see T-4857's decision record."
        ),
        "tags": ["M9", "probe", "statistics", "T-4857", "sweep", "incomplete-beta"],
        "hypotheses": case.hyp,
        "sheets": [{"name": "Calc", "cells": case.cells}],
        "expected": {},
    }


# --------------------------------------------------------------------------- kinds


DATA = {
    "A1": 2.5,  # number
    "A2": "2.5",  # numeric text
    "A3": "abc",  # text
    "A4": "",  # empty text (written as ="")
    "A5": True,
    "A6": False,
    # A7 is left empty
    "A8": "=NA()",
    "A9": "=1/0",
    "A10": 1.5,
    "A11": 2.5,
    "A12": 3.5,
    "A13": "5",
    "A14": 7,
    "A15": "x",
    "A16": True,
    "B1": 0.3,
    "B2": "0.3",
    "B10": 0.1,
    "B11": 0.3,
    "B12": 0.7,
}
KIND_ARGS = (
    ("number", None),
    ("numeric-text", '"{v}"'),
    ("text", '"abc"'),
    ("empty-text", '""'),
    ("true", "TRUE"),
    ("false", "FALSE"),
    ("ref-number", "Data!{num}"),
    ("ref-numeric-text", "Data!{numtext}"),
    ("ref-text", "Data!A3"),
    ("ref-empty-text", "Data!A4"),
    ("ref-true", "Data!A5"),
    ("ref-false", "Data!A6"),
    ("ref-empty", "Data!A7"),
    ("error-literal", "#N/A"),
    ("error-ref-na", "Data!A8"),
    ("error-ref-div0", "Data!A9"),
    ("range-1", "Data!{num}:{num}"),
    ("array-1", "{{{v}}}"),
    ("omitted", ""),
    ("text-padded", '" {v} "'),
    ("computed-text", 'TEXT({v},"0.0")'),
    ("computed-bool", "(1=1)"),
)
# Per function: its arguments' baseline values and the Data cells holding the same number as a number and as text.
BASE = {
    "F.DIST": (2.5, 5, 10, "TRUE"),
    "F.DIST.RT": (2.5, 5, 10),
    "FDIST": (2.5, 5, 10),
    "F.INV": (0.3, 5, 10),
    "F.INV.RT": (0.3, 5, 10),
    "FINV": (0.3, 5, 10),
}
REF_OF = {2.5: ("A1", "A2"), 5: ("A14", "A13"), 10: ("A14", "A13"), 0.3: ("B1", "B2")}


def kinds_case() -> dict:
    case = Case()
    cells = case.cells
    for fn, base in BASE.items():
        for pos, value in enumerate(base):
            for kind, template in KIND_ARGS:
                if fn == "F.DIST" and pos == 3:
                    continue
                args = [str(v) if not isinstance(v, str) else v for v in base]
                if template is not None:
                    num, numtext = REF_OF.get(value, ("A14", "A13"))  # type: ignore[arg-type]
                    args[pos] = template.format(v=value, num=num, numtext=numtext)
                case.add(f"={fn}({','.join(args)})", f"kind/{fn}/arg{pos + 1}/{kind}")
            # The dynamic-array lift: a multi-cell range, an array and a computed array, read element by element.
            lifts = {
                "range-multi": "Data!A10:A12"
                if pos == 0 and fn.startswith("F.D")
                else "Data!B10:B12"
                if pos == 0
                else "Data!A10:A12",
                "array-multi": "{1.5;2.5;3.5}"
                if pos == 0 and fn.startswith("F.D")
                else "{0.1;0.3;0.7}"
                if pos == 0
                else "{1.5;2.5;3.5}",
                "sequence": "SEQUENCE(3)/2" if pos != 0 or fn.startswith("F.D") else "SEQUENCE(3)/4",
                "mixed-array": '{1.5;"2.5";TRUE}'
                if pos == 0 and fn.startswith("F.D")
                else '{0.3;"0.3";TRUE}'
                if pos == 0
                else '{1.5;"2.5";TRUE}',
            }
            for kind, text in lifts.items():
                for k in (1, 2, 3) if kind == "mixed-array" else (2,):
                    args = [str(v) for v in base]
                    args[pos] = text
                    case.add(f"=INDEX({fn}({','.join(args)}),{k})", f"kind/{fn}/arg{pos + 1}/lift-{kind}/{k}")
            args = [str(v) for v in base]
            args[pos] = (
                "Data!A10:A11" if pos else ("Data!A10:A11" if fn.startswith("F.D") else "Data!B10:B11")
            )
            case.add(f"=ROWS({fn}({','.join(args)}))", f"kind/{fn}/arg{pos + 1}/lift-rows")
    # F.DIST's cumulative flag.
    for text, kind in (
        ("TRUE", "true"),
        ("FALSE", "false"),
        ("1", "1"),
        ("0", "0"),
        ("2", "2"),
        ("-1", "-1"),
        ("0.5", "0.5"),
        ("1E-300", "tiny"),
        ('"TRUE"', "text-TRUE"),
        ('"FALSE"', "text-FALSE"),
        ('"1"', "text-1"),
        ('"0"', "text-0"),
        ('"abc"', "text"),
        ('""', "empty-text"),
        ("Data!A5", "ref-true"),
        ("Data!A6", "ref-false"),
        ("Data!A1", "ref-number"),
        ("Data!A2", "ref-numeric-text"),
        ("Data!A3", "ref-text"),
        ("Data!A7", "ref-empty"),
        ("Data!A8", "ref-error"),
        ("#N/A", "error-literal"),
        ("", "omitted"),
        ("{TRUE}", "array-1"),
        ("Data!A5:A5", "range-1"),
    ):
        case.add(f"=F.DIST(2.5,5,10,{text})", f"kind/F.DIST/arg4/{kind}")
    for k in (1, 2):
        case.add(f"=INDEX(F.DIST(2.5,5,10,{{TRUE;FALSE}}),{k})", f"kind/F.DIST/arg4/lift-array/{k}")
        case.add(f"=INDEX(F.DIST(2.5,5,10,Data!A5:A6),{k})", f"kind/F.DIST/arg4/lift-range/{k}")
    # Too few and too many arguments.
    for f in (
        "=F.DIST(2.5,5,10)",
        "=F.DIST(2.5,5,10,TRUE,1)",
        "=F.DIST.RT(2.5,5)",
        "=FDIST(2.5,5,10,1)",
        "=F.INV(0.3,5)",
        "=FINV(0.3,5,10,1)",
    ):
        case.add(f, "kind/arity")
    # F.TEST and FTEST: argument kinds and shapes.
    cols = {
        "C1:C5": [1, 2, 4, 7, 11],
        "D1:D4": [2, 3, 3, 9],
        "E1:E5": [1, "x", 4, True, 11],
        "F1:F5": [1, None, 4, None, 11],
    }
    for col, values in cols.items():
        c = col[0]
        for i, v in enumerate(values, 1):
            if v is not None:
                DATA[f"{c}{i}"] = v
    DATA["G1"], DATA["H1"], DATA["G2"], DATA["H2"] = 1, 3, 4, 8
    DATA["C7"], DATA["C8"] = 5, 5  # zero variance
    other = "{2;3;3;9}"
    shapes = (
        ("range", "Data!C1:C5"),
        ("range-text-bool", "Data!E1:E5"),
        ("range-blanks", "Data!F1:F5"),
        ("range-2d", "Data!G1:H2"),
        ("row-array", "{1,2,4,7,11}"),
        ("col-array", "{1;2;4;7;11}"),
        ("2d-array", "{1,2;4,7}"),
        ("array-text-bool", '{1;"2";TRUE;4;7}'),
        ("two-points", "{1;3}"),
        ("one-point", "{1}"),
        ("zero-variance", "Data!C7:C8"),
        ("number", "5"),
        ("numeric-text", '"5"'),
        ("text", '"abc"'),
        ("empty-text", '""'),
        ("true", "TRUE"),
        ("ref-true", "Data!A5"),
        ("ref-empty", "Data!A7"),
        ("range-1-empty", "Data!A7:A7"),
        ("range-1-number", "Data!A1:A1"),
        ("error-literal", "#N/A"),
        ("error-ref", "Data!A8"),
        ("error-element", "{1;#N/A;3}"),
        ("omitted", ""),
        ("sequence", "SEQUENCE(5)^2"),
        ("huge", "{1E+200;2E+200;-1E+200}"),
        ("tiny", "{1E-200;2E-200;4E-200}"),
    )
    for fn in ("F.TEST", "FTEST"):
        for kind, text in shapes:
            case.add(f"={fn}({text},{other})", f"kind/{fn}/arg1/{kind}")
            case.add(f"={fn}({other},{text})", f"kind/{fn}/arg2/{kind}")
    data_cells: dict[str, object] = {}
    for k, v in DATA.items():
        data_cells[k] = '=""' if v == "" else v
    return {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4857 (owner request 2026-11-07): F family argument kinds - every argument of F.DIST, F.DIST.RT, FDIST, "
            "F.INV, F.INV.RT and FINV given a number, numeric text, text, empty text, TRUE/FALSE typed and through a "
            "reference, an empty cell, an error typed and through a reference, one-cell and multi-cell ranges, "
            "arrays and the dynamic-array lift; F.DIST's cumulative flag; F.TEST/FTEST argument kinds and shapes. "
            f"Generator docs/research/t4857/gen.py (seed {SEED}); ANALYSIS.md section 1."
        ),
        "tags": ["M9", "probe", "statistics", "T-4857"],
        "hypotheses": case.hyp,
        "sheets": [{"name": "Calc", "cells": cells}, {"name": "Data", "cells": data_cells}],
        "expected": {},
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--cache", help="a JSON file of x_for_band answers to reuse and update")
    args = parser.parse_args()
    if args.cache and Path(args.cache).exists():
        CACHE.update(json.loads(Path(args.cache).read_text(encoding="utf-8")))
    with ProcessPoolExecutor() as pool:
        numeric = numeric_case(pool)
    if args.cache:
        Path(args.cache).write_text(json.dumps(CACHE), encoding="utf-8")
    kinds = kinds_case()
    for path, doc in ((NUMERIC, numeric), (KINDS, kinds)):
        n = sum(1 for k in doc["sheets"][0]["cells"] if k[0] in "AC")
        print(f"{path.relative_to(ROOT)}: {n} cells")
        sections: dict[str, int] = {}
        for tag in doc["hypotheses"].values():
            head = tag.split("/")[0] if not tag.startswith("kind/") else "/".join(tag.split("/")[:2])
            sections[head] = sections.get(head, 0) + 1
        print("  " + ", ".join(f"{k} {v}" for k, v in sorted(sections.items())))
        if args.write:
            path.write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
