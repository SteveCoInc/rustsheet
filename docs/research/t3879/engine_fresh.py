"""T-3879: the engine (the rustsheet bindings built from this branch) on t3879-bpser-split-fresh: bit-exact counts
of its BETA.DIST cells and of its GAMMALN at their L's arguments, against Excel's fill. The engine's misses that remain
are cells whose L takes an engine GAMMALN that is not Excel's (T-3042).
usage: engine_fresh.py   (bindings/python/.venv/bin/python, from this directory, after `make py-bindings`)"""

from __future__ import annotations

import json
from collections import Counter

import gen_fresh as gf
import rustsheet


def main() -> None:
    out = json.load(open(gf.lib.CASES / f"{gf.NAME}.oracle.json"))["sheets"]["Calc"]
    case = json.load(open(gf.lib.CASES / f"{gf.NAME}.case.json"))["sheets"][0]["cells"]
    meta = json.load(open(gf.HERE / f"{gf.NAME}.meta.json"))
    m = rustsheet.create("fresh")
    n: Counter = Counter()
    for i, k, *_ in meta:
        m.set_user_input(0, 1, 1, case[f"A{i}"])
        m.evaluate()
        got = float(m.get_cell_value_by_ref("Sheet1!A1"))
        n[(k, "cells")] += 1
        n[(k, "exact")] += got == float(out[f"A{i}"]["v"])
    for k in ("beta", "gl"):
        print(f"{k}: engine bit-exact {n[(k, 'exact')]:,} of {n[(k, 'cells')]:,}")


if __name__ == "__main__":
    main()
