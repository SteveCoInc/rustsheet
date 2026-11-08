"""T-4865: write kernel_probe.case.json, Excel's BETA.DIST at every AS 109 iterate of the replay
(iteration_counts.as109, start clamp on) for the 30 class 1 and 2 cells of accuracy/BETA.INV/s1 and
BETAINV/s1, so the search can be replayed on Excel's own CDF (replay_excel_kernel.py).

Row r is the r-th cell of `CELLS`; column k (A, B, ...) is iterate k. Each x is written exactly as a
dyadic expression (Excel cuts a literal to 15 digits: excel-kernel-playbook.md "Traps"): near 1 as
1-(m1*2^26+m0)*2^e, elsewhere as (m1*2^26+m0)*2^e. The shapes are the s1 literals.
Kept in docs/research, like T-2555's excel_cdf_probe: its cells are evidence for the BETA.INV cells,
not cells to score. Filled with `rustsheet-oracle fill -b excel-graph` on a copy in eval/suites/probes.

Run: cd tools && uv run python ../docs/research/t4865/gen_kernel_probe.py
"""

import json
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from iteration_counts import ROOT, as109  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
# (function, cell): the class 1 and 2 cells (Excel 1-2^-53, 2^-53 or #NUM!) of the s1 batches.
CELLS = [
    ("BETA.INV", r)
    for r in "A83 A86 A88 A89 A92 A94 A95 A97 A98 A100 A103 A105 A106 A108 A109".split()
] + [
    ("BETAINV", r)
    for r in "A82 A83 A85 A88 A89 A91 A94 A97 A99 A100 A103 A105 A106 A108 A109".split()
]


def col(k: int) -> str:
    s = ""
    k += 1
    while k:
        k, rem = divmod(k - 1, 26)
        s = chr(65 + rem) + s
    return s


def dyadic(v: float) -> str:
    m, e = math.frexp(v)  # v = m 2^e, 0.5 <= m < 1
    n = int(m * 2**53)
    e -= 53
    while n and n % 2 == 0:
        n //= 2
        e += 1
    hi, lo = divmod(n, 2**26)
    body = f"({hi}*2^26+{lo})" if hi else f"{lo}"
    return f"{body}*2^{e}" if e else body


def x_expr(x: float) -> str:
    if x > 0.5:
        return f"1-{dyadic(1 - x)}"  # 1 - x is exact here
    return dyadic(x)


def literals(fn: str, ref: str) -> tuple[str, str, str]:
    case = json.loads((ROOT / f"eval/accuracy/tables/{fn}/s1.case.json").read_text())
    p, a, b = case["sheets"][0]["cells"][ref].split("(")[1].rstrip(")").split(",")
    return p, a, b


def main() -> None:
    cells: dict[str, str] = {}
    for r, (fn, ref) in enumerate(CELLS, 1):
        p, a, b = literals(fn, ref)
        trace: list = []
        as109(float(p), float(a), float(b), True, trace=trace)
        flip = float(p) > 0.5
        pp, qq = (b, a) if flip else (a, b)
        for k, (x, *_rest) in enumerate(trace):
            assert eval(x_expr(x).replace("^", "**")) == x  # noqa: S307 - own arithmetic
            cells[f"{col(k)}{r}"] = f"=BETA.DIST({x_expr(x)},{pp},{qq},TRUE)"
    doc = {
        "format": "rustsheet-case/v1",
        "description": "T-4865 research: Excel's BETA.DIST at the AS 109 replay's iterates for the class 1 and 2 "
        "cells of accuracy/BETA.INV/s1 and BETAINV/s1 (docs/research/t4865/gen_kernel_probe.py).",
        "tags": ["M9", "statistical", "beta-dist", "probe"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    (HERE / "kernel_probe.case.json").write_text(json.dumps(doc, indent=1) + "\n")
    print(len(cells), "cells")


if __name__ == "__main__":
    main()
