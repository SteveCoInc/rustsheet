"""T-3879 R4.2 fresh sweep (decision 0188 R4.2): t3879-bpser-split-fresh, drawn after T-3873 froze the split-z form.

Shapes log-uniform: the small shape m in [0.02, 1), the other shape M in [0.02, 8) (so min(a, b) < 1 and b0 < 8), each
orientation half the time (a = m or b = m); x log-uniform in [1e-4, 0.999]. A cell is kept only where r2-bratio's GL53
kernel calls bpser once on the cell's own (a, b, x) (no swap, so the cdf is that call's I) and the sub-branch is
b0 <= 1 or 1 < b0 < 8: the cells the split applies to. Seed 3879, fixed before any fill. Every number is written
exactly (w2-forensics' `num`).
Rows: 'beta' =BETA.DIST(x, a, b, TRUE); then 'gl' =GAMMALN(v) at every argument the GL53 model's L reads (collected
from the model, so the scoring uses Excel's GAMMALN, never the engine's; T-3042).
usage: gen_fresh.py [--write]   (run with bindings/python/.venv/bin/python from this directory)
"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "t2464" / "r7-bpser-nondouble-z"))
import c7  # noqa: E402

cm = c7.cm
lib = cm.lib
NAME = "t3879-bpser-split-fresh"
N_SHAPES, X_PER_SHAPE, TRIES = 300, 8, 200
SUBS = ("bpser:b0<=1", "bpser:1<b0<8")


def logu(rng: random.Random, lo: float, hi: float) -> float:
    return float(f"{math.exp(rng.uniform(math.log(lo), math.log(hi))):.17g}")


def build() -> list:
    rng = random.Random(3879)
    rows, i = [], 0
    for _ in range(N_SHAPES):
        m, big = logu(rng, 0.02, 0.999), logu(rng, 0.02, 7.99)
        a, b = (m, big) if rng.random() < 0.5 else (big, m)
        got = 0
        for _ in range(TRIES):
            if got == X_PER_SHAPE:
                break
            x = logu(rng, 1e-4, 0.999)
            _, _, br, trace = lib.kernel(a, b, x, 1.0 - x, cm.GL53)
            if trace != [("bpser", a, b, x)] or not br.startswith("w_bpser") or lib.sub_branch("bpser", a, b) not in SUBS:
                continue
            i += 1
            got += 1
            rows.append((i, "beta", x, a, b))
    return rows


def gl_args(beta: list) -> list[float]:
    """The GAMMALN arguments the GL53 model's L reads on these cells (r2-bratio's _GL_MISSING)."""
    bx = lib.bx
    saved = dict(bx._GL)
    bx._GL.clear()
    bx._GL_MISSING.clear()
    for _, _, x, a, b in beta:
        cm.MODEL.lnb_w3(a, b)
    need = sorted(bx._GL_MISSING)
    bx._GL.update(saved)
    bx._GL_MISSING.clear()
    return need


def rows() -> list:
    beta = build()
    n = len(beta)
    return beta + [(n + k, "gl", v, 0.0, 0.0) for k, v in enumerate(gl_args(beta), 1)]


FORMULA = {
    "beta": lambda x, a, b: f"=BETA.DIST({lib.w2p.num(x)},{lib.w2p.num(a)},{lib.w2p.num(b)},TRUE)",
    "gl": lambda x, a, b: f"=GAMMALN({lib.w2p.num(x)})",
}


def main() -> None:
    rs = rows()
    cells = {f"A{i}": FORMULA[k](x, a, b) for i, k, x, a, b in rs}
    print(NAME, len(cells), "cells:", sum(r[1] == "beta" for r in rs), "beta,", sum(r[1] == "gl" for r in rs), "gl")
    if "--write" in sys.argv:
        case = {
            "format": "rustsheet-case/v1",
            "description": "T-3879 R4.2 fresh sweep: bpser cells at min(a, b) < 1, b0 < 8 (BETA.DIST) and Excel's "
            "GAMMALN at the arguments of their L",
            "tags": ["M9", "probe", "statistics", "T-3879", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}],
            "expected": {},
        }
        (lib.CASES / f"{NAME}.case.json").write_text(json.dumps(case, indent=1) + "\n")
        json.dump(rs, open(HERE / f"{NAME}.meta.json", "w"))


if __name__ == "__main__":
    main()
