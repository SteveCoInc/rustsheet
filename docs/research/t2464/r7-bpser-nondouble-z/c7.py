"""T-3873 r7-bpser-nondouble-z: shared helpers. Imports r6-bpser-xlinked's common.py (and through it r5/r2); nothing
is copied. Run with bindings/python/.venv/bin/python from this directory."""

from __future__ import annotations

import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r6-bpser-xlinked"))
import common as cm  # noqa: E402

sx, mpf, fl53, fl64 = cm.sx, cm.mpf, cm.fl53, cm.fl64


def cells(probe: str = "r6"):
    """Every beta cell of r6's probe (probe 'r6') or r5's t3570-r5-bpser-lngamma-beta ('r5'; part = 'r5') with GL53's pieces: dict(part, a, b, x, I, alx, L, z (exact), s, nd (z not a double),
    q (z's position above the double below it, in ulps of that double))."""
    rows = cm.beta_rows() if probe == "r6" else [("r5", b, a, x, e) for _, x, a, b, e in cm.r5_rows()]
    return cells_from(rows)


def cells_from(rows) -> list:
    """cells() for any (part, b0, a0, x, excel) rows at a0 < 1 < b0 < 8 (direct bpser)."""
    return [cell(part, a, b, x, e) for part, b, a, x, e in rows]


def cell(part: str, a: float, b: float, x: float, e: float, L: float | None = None) -> dict:
    """One cell's GL53 pieces (L given, or GL53's from the loaded GAMMALN values)."""
    alx = sx("*", a, cm.LN(x))
    L = float(cm.MODEL.lnb_w3(a, b)) if L is None else L
    z = mpf(alx) - mpf(L)
    n = float(z)
    lo = n if mpf(n) <= z else math.nextafter(n, -math.inf)
    q = float((z - mpf(lo)) / mpf(math.ulp(lo)))
    return dict(part=part, a=a, b=b, x=x, I=e, alx=alx, L=L, z=z, zlo=lo, q=q, nd=q != 0.0,
                s=cm.MODEL.series_w3(a, b, x))


def e_ok(c: dict) -> list[float]:
    """The doubles E (near EXP of z) with fl(E s) equal to Excel's I: Excel's E is one of them."""
    e0 = cm.EXP(c["zlo"])
    got = []
    for k in range(-8, 9):
        E = cm.step(e0, k)
        if sx("*", E, c["s"]) == c["I"]:
            got.append(E)
    return got


def shape(c: dict) -> str:
    return f"{c['part']} b0={c['b']} a0={c['a']}"
