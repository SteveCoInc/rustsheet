"""T-4865: compare Excel's BETA.DIST at the replay's iterates (kernel_probe.oracle.json) with the
50-digit cdf the replay uses, and replay AS 109 on Excel's own values where the iterates agree.

Run: cd tools && uv run python ../docs/research/t4865/replay_excel_kernel.py
"""

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from gen_kernel_probe import CELLS, col, literals  # noqa: E402
from iteration_counts import as109  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
ORACLE = json.loads((HERE / "kernel_probe.oracle.json").read_text())["sheets"]["Calc"]


def main() -> None:
    for r, (fn, ref) in enumerate(CELLS, 1):
        p, a, b = (float(v) for v in literals(fn, ref))
        trace: list = []
        v, it, divs, _ = as109(p, a, b, True, trace=trace)
        excel_at = {}
        worst = 0.0
        errs = 0
        for k, (x, pp, qq, yy) in enumerate(trace):
            ex = ORACLE.get(f"{col(k)}{r}", {}).get("v")
            if not isinstance(ex, (int, float)):
                errs += 1
                continue
            excel_at[(x, pp, qq)] = float(ex)
            if yy:
                worst = max(worst, abs(ex - yy) / abs(yy))

        def excel_cdf(x, pp, qq, _memo=excel_at):
            if (x, pp, qq) in _memo:
                return _memo[(x, pp, qq)]
            raise KeyError("off-trace")

        try:
            v2, it2, divs2, _ = as109(p, a, b, True, cdf=excel_cdf)
            again = f"replay-on-excel={v2!r} iters={it2} divs={divs2}"
        except KeyError:
            again = "replay-on-excel leaves the probed iterates"
        print(f"{fn:8} {ref:5} replay={v!r} iters={it} divs={divs} excel-vs-mp max rel={worst:.1e} errors={errs} {again}")


if __name__ == "__main__":
    main()
