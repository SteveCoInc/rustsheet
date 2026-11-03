"""T-3769 r6-bpser-xlinked: shared loader. Imports r5-bpser-lngamma's make_probe/score_forms (and through them
r2-bpser-small's lib and r2-bratio's bratio_excel); nothing is copied. Run with bindings/python/.venv/bin/python.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r5-bpser-lngamma"))
import make_probe as r5  # noqa: E402
import score_forms as r5score  # noqa: E402

lib = r5.lib
sx, EXP, EXPX, LN, GAMMALN = lib.sx, lib.EXP, lib.EXPX, lib.LN, lib.GAMMALN
fl53, fl64, mpf = lib.fl53, lib.fl64, lib.mpf
GL53 = lib.V(lg="GL53")
MODEL = lib.Model(lib.bx.BEST, GL53)


def step(u: float, k: int) -> float:
    for _ in range(abs(k)):
        u = math.nextafter(u, math.inf if k > 0 else -math.inf)
    return u


def pieces(a: float, b: float, x: float) -> dict:
    """GL53 bpser's stored pieces at (a, b, x), a = a0 < 1 < b = b0 < 8 (direct orientation)."""
    lnx = LN(x)
    alx = sx("*", a, lnx)
    L = MODEL.lnb_w3(a, b)
    z = mpf(alx) - L
    e = EXPX(fl64(z))
    s = MODEL.series_w3(a, b, x)
    return dict(lnx=lnx, alx=alx, L=L, z=z, z53=fl53(z), e=e, s=s, i=sx("*", e, s))


def r5_rows() -> list:
    """(blk, x, a, b, excel) of r5's beta probe, with r5's measured GAMMALN loaded into the model."""
    lib.bx._GL.update(r5score.measured_gl())
    out = r5score.sidecar("t3570-r5-bpser-lngamma-beta")
    rows = []
    for i, blk, x, a, b in json.load(open(r5.HERE / "t3570-r5-bpser-lngamma-beta.meta.json")):
        c = out.get(f"A{i}")
        if c and c.get("t") == "n":
            rows.append((blk, x, a, b, float(c["v"])))
    return rows


def sidecar(name: str) -> dict:
    f = lib.CASES / f"{name}.oracle.json"
    return json.load(open(f))["sheets"]["Calc"] if f.exists() else {}


def measured(kind: str) -> dict:
    """Excel's values from t3769-r6-bpser-aux of one kind: {x: v} for ln/gl, {(a, x): v} for alx."""
    out = sidecar("t3769-r6-bpser-aux")
    got = {}
    for i, k, x, a, _ in json.load(open(HERE / "t3769-r6-bpser-aux.meta.json")):
        c = out.get(f"A{i}")
        if k == kind and c and c.get("t") == "n":
            got[(a, x) if kind == "alx" else x] = float(c["v"])
    return got


def load_measured_gl() -> None:
    lib.bx._GL.update(r5score.measured_gl())
    lib.bx._GL.update(measured("gl"))


def beta_rows() -> list:
    """(part, b0, a0, x, excel) of t3769-r6-bpser-beta, with measured GAMMALN loaded into the model."""
    load_measured_gl()
    out = sidecar("t3769-r6-bpser-beta")
    rows = []
    for i, blk, x, a, b in json.load(open(HERE / "t3769-r6-bpser-beta.meta.json")):
        c = out.get(f"A{i}")
        if c and c.get("t") == "n":
            rows.append((blk.split(":")[0], b, a, x, float(c["v"])))
    return rows
