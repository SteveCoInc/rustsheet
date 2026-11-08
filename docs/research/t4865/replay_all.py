"""T-4865: run the AS 109 replay of iteration_counts.py over every one-call BETA.INV/BETAINV cell of
the T-3351 probe, the T-3351 sweep and Gate A s1 whose Excel value is #NUM!, 0, 1 or within 2^-52
of an end, and print Excel's value next to the replay's value, iteration count and g-divisions.

Run: cd tools && uv run python ../docs/research/t4865/replay_all.py > ../docs/research/t4865/replay_all.txt
"""

import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from iteration_counts import ROOT, as109  # noqa: E402

CASES = [
    "eval/suites/probes/cases/t3351-beta-inv-p-near-1",
    "eval/suites/probes-numeric/cases/t3351-beta-inv-sweep",
    "eval/accuracy/tables/BETA.INV/s1",
    "eval/accuracy/tables/BETAINV/s1",
]
CALL = re.compile(r"^=(BETA\.INV|BETAINV)\(([-+0-9.E,]+)\)$")


def near_end(v):
    return v == "#NUM!" or (isinstance(v, float) and (v <= 2**-52 or v >= 1 - 2**-52))


def rows():
    for case in CASES:
        base = ROOT / case
        doc = json.loads(base.with_name(base.name + ".case.json").read_text())
        oracle = json.loads(base.with_name(base.name + ".oracle.json").read_text())["sheets"]["Calc"]
        for ref, formula in doc["sheets"][0]["cells"].items():
            m = CALL.match(formula)
            if not m:
                continue
            args = [eval(a) for a in m.group(2).split(",")]  # noqa: S307 - literal arithmetic only
            if len(args) != 3:
                continue
            p, a, b = (float(v) for v in args)
            if not (0 < p < 1 and 1e-300 < a < 1e5 and 1e-300 < b < 1e5 and 1e-150 < min(p, 1 - p)):
                continue
            ex = oracle.get(ref, {}).get("v")
            if isinstance(ex, int):
                ex = float(ex)
            if near_end(ex):
                yield case.rsplit("/", 1)[1], ref, formula, p, a, b, ex


if __name__ == "__main__":
    only = sys.argv[1:]
    for case, ref, formula, p, a, b, ex in rows():
        if only and case not in only:
            continue
        try:
            v, it, divs, kmax = as109(p, a, b, True)
        except (ValueError, OverflowError, ZeroDivisionError) as e:
            v, it, divs, kmax = f"err:{type(e).__name__}", 0, 0, 0
        print(f"{case}\t{ref}\t{formula}\texcel={ex!r}\treplay={v!r}\titers={it}\tdivs={divs}\tkmax={kmax}", flush=True)
