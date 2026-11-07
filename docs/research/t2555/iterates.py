"""T-2555 attempt 2: replay AS 109's Newton steps on Excel's own BETA.DIST (excel_cdf_probe.*.json here).

Run with mpmath: cd tools && uv run python ../docs/research/t2555/iterates.py. For each cell and iterate k it prints
Excel's CDF at x_k against the mpmath CDF, and the step x_k - (F_excel(x_k) - p) / f(x_k) (f the exact density) next
to Excel's BETA.INV value.
"""

import importlib.util
import json
import math
import os

import mpmath

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "as109_replay", os.path.join(HERE, "as109_replay.py")
)
assert _spec and _spec.loader
replay = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(replay)
CASE = os.path.join(HERE, "excel_cdf_probe")


def main() -> None:
    with open(CASE + ".case.json") as fh:
        calc = json.load(fh)["sheets"][0]["cells"]
    with open(CASE + ".oracle.json") as fh:
        oracle = json.load(fh)["sheets"]["Calc"]
    for r, (name, p, a, b, excel) in enumerate(replay.cells, 1):
        lb = replay.lbeta_mp(a, b)
        print(name, "p", p, "Excel BETA.INV", excel)
        for col in "ABCD":
            cell = f"{col}{r}"
            if cell not in calc:
                continue
            x = float(calc[cell].split("(")[1].split(",")[0])
            fx = float(oracle[cell]["v"])
            exact = float(mpmath.betainc(a, b, 0, x, regularized=True))
            dens = math.exp(-lb + (a - 1) * math.log(x) + (b - 1) * math.log1p(-x))
            step = x - (fx - p) / dens
            print(
                f"  x{col}: F_excel-F_exact {(fx - exact) / exact:+.2e}  next {step!r} rel to Excel {(step - excel) / excel:+.2e}"
            )


if __name__ == "__main__":
    main()
