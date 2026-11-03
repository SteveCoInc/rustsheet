"""T-3769 probes t3769-r6-bpser-{beta,exp,aux}: bpser at FIXED shapes (a0 < 1 < b0 < 8, direct orientation) along x,
so L is one constant per shape and the remaining x-linked pieces (alx, z, E, s) can be separated.

beta: BETA.DIST(x, a0, b0, TRUE) at 9 shapes, a0 in {0.3, 0.5, 0.7} x b0 in {2.5, 6.25} (tune) and b0 4.75 (hold-out,
      fixed before scoring). Per shape 128 consecutive doubles up from x = 0.0371 and 384 short decimals
      x = 0.0100 + 0.0001 k (k < 384, up to 0.0483; (x b0)^a0 <= 0.7 at (0.3, 6.25) needs x <= 0.0488), every one a w_bpser cell (checked).
exp:  =EXP(z53 + k ulps), k in {-1, 0, 1}, at the GL53/z53 model's stored z of every beta cell (E measured, not modelled).
aux:  =LN(x) at every x, =GAMMALN at every a0, b0 and fl(a0 + b0), and =a0*LN(x) (the worksheet's own product).
Exact doubles are written with w2-forensics' `num`, so no literal is cut to 15 digits.
usage: make_probe.py [--write]
"""

from __future__ import annotations

import json
import math
import sys

import common as cm

lib = cm.lib
A0S = (0.3, 0.5, 0.7)
TUNE_B0S = (2.5, 6.25)
HOLD_B0S = (4.75,)
X_RUN0, N_RUN = 0.0371, 128
N_DEC = 384


def xs() -> list[float]:
    out, v = [], X_RUN0
    for _ in range(N_RUN):
        out.append(v)
        v = math.nextafter(v, 1.0)
    out += [float(f"{0.0100 + 0.0001 * k:.4f}") for k in range(N_DEC)]
    return out


def shapes() -> list[tuple[float, float, str]]:
    return [(a, b, "tune") for b in TUNE_B0S for a in A0S] + [(a, b, "hold") for b in HOLD_B0S for a in A0S]


def build() -> dict[str, list]:
    beta, i = [], 0
    for a0, b0, part in shapes():
        for x in xs():
            assert lib.kernel(a0, b0, x, 1.0 - x, cm.GL53)[2].split("/")[0] == "w_bpser", (a0, b0, x)
            i += 1
            beta.append((i, f"{part}:{b0}:{a0}", x, a0, b0))
    return {"beta": beta}


def aux_rows(beta) -> list:
    rows, k = [], 0
    for x in sorted({r[2] for r in beta}):
        k += 1
        rows.append((k, "ln", x, 0.0, 0.0))
    for v in sorted({v for _, _, _, a, b in beta for v in (a, b, a + b)}):
        k += 1
        rows.append((k, "gl", v, 0.0, 0.0))
    for a in A0S:
        for x in sorted({r[2] for r in beta}):
            k += 1
            rows.append((k, "alx", x, a, 0.0))
    return rows


def exp_rows(beta) -> list:
    """Needs measured GAMMALN loaded (aux) so z53 uses Excel's L; called after aux is filled."""
    zs = set()
    for _, _, x, a, b in beta:
        z = cm.pieces(a, b, x)["z53"]
        zs.update(cm.step(z, k) for k in (-1, 0, 1))
    return [(k, "exp", z, 0.0, 0.0) for k, z in enumerate(sorted(zs), 1)]


def snum(v: float) -> str:
    """w2-forensics' exact `num` for any sign: its encoder handles only v > 0 (a negative mantissa skips the
    two-part split, and a 16-digit literal is cut to 15 digits by Excel's parser)."""
    return f"-{lib.w2p.num(-v)}" if v < 0 else lib.w2p.num(v)


FORMULA = {
    "beta": lambda x, a, b: f"=BETA.DIST({lib.w2p.num(x)},{lib.w2p.num(a)},{lib.w2p.num(b)},TRUE)",
    "ln": lambda x, a, b: f"=LN({lib.w2p.num(x)})",
    "gl": lambda x, a, b: f"=GAMMALN({lib.w2p.num(x)})",
    "alx": lambda x, a, b: f"={lib.w2p.num(a)}*LN({lib.w2p.num(x)})",
    "exp": lambda x, a, b: f"=EXP({snum(x)})",
}


def write(name: str, rows: list, what: str) -> None:
    cells = {}
    for i, k, x, a, b in rows:
        kind = "beta" if k.startswith(("tune", "hold")) else k
        cells[f"A{i}"] = FORMULA[kind](x, a, b)
    print(name, len(cells), "cells")
    if "--write" in sys.argv:
        case = {
            "format": "rustsheet-case/v1",
            "description": f"T-3769 r6-bpser-xlinked: bpser at fixed shapes along x ({what})",
            "tags": ["M9", "probe", "statistics", "T-2464", "T-3769", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}],
            "expected": {},
        }
        (lib.CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
        json.dump(rows, open(cm.HERE / f"{name}.meta.json", "w"))


def main() -> None:
    beta = build()["beta"]
    if "exp" in sys.argv:
        cm.load_measured_gl()
        write("t3769-r6-bpser-exp", exp_rows(beta), "EXP at the model's stored z, +-1 ulp")
        return
    write("t3769-r6-bpser-beta", beta, "BETA.DIST")
    write("t3769-r6-bpser-aux", aux_rows(beta), "LN, GAMMALN and a0*LN(x)")


if __name__ == "__main__":
    main()
