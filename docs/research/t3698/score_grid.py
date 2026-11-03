"""T-3698 train/test scorer: every =RATE( cell of probes and probes-numeric, split by the fit lab's hash
(`holdout.py`), scored bit-exact / within 4 ulp / within 15 digits on each side.

The candidate is the engine the release harness was built from. The grid was run with a temporary, uncommitted
switch in `rate_secant` reading RS_OFF (hold), RS_K, RS_FHI, RS_SMIN and RS_FLO in place of the constants.

Run: cd tools && uv run python -I ../docs/research/t3698/score_grid.py <name> <out-dir> [excluded case ids, comma-separated]
(the grid excluded probes-numeric/t3478-rate-r4-wide and probes-numeric/t3698-rate-r4-fresh)"""

import json
import math
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from rustsheet_tools.fitlab.holdout import is_holdout
from rustsheet_tools.fitlab.score import ulp_distance, within_fifteen_digits
from rustsheet_tools.fitlab.sweep import Cell
from rustsheet_tools.snapshot import parse_show

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(sys.argv[2])
EXCLUDE = set(sys.argv[3].split(",")) if len(sys.argv) > 3 and sys.argv[3] else set()
cases = []
for suite in ("probes", "probes-numeric"):
    for p in sorted((ROOT / "eval/suites" / suite / "cases").rglob("*.case.json")):
        if "RATE(" in p.read_text() and (ROOT / str(p).replace(".case.json", ".oracle.json")).exists():
            cid = f"{suite}/{p.relative_to(ROOT / 'eval/suites' / suite / 'cases').as_posix()[:-10]}"
            if cid not in EXCLUDE:
                cases.append(cid)


def run(cid):
    out = subprocess.run(
        [
            str(ROOT / "target/release/rustsheet-eval"),
            "--root",
            str(ROOT),
            "show",
            cid + ".case.json",
            "--all",
        ],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return cid, parse_show(out)


def num(s):
    try:
        return float(s)
    except ValueError:
        return None


side = {"train": [0, 0, 0, 0], "test": [0, 0, 0, 0]}
per = {}
with ThreadPoolExecutor(8) as ex:
    for cid, cells in ex.map(run, cases):
        for key, c in cells.items():
            if not c.get("formula", "").upper().startswith("=RATE("):
                continue
            sheet, a1 = key.split("!")
            cell = Cell(cid, sheet, a1, c["formula"], (), math.nan)
            s = side["test" if is_holdout(cell) else "train"]
            e, a = num(c["expected"]), num(c["actual"])
            if e is None or a is None:
                ok = e is None and a is None and c["expected"] == c["actual"]
                ex4 = d15 = ok
            else:
                ok = a == e
                ex4 = ulp_distance(a, e) <= 4
                d15 = within_fifteen_digits(e, a)
            s[0] += 1
            s[1] += ok
            s[2] += ex4
            s[3] += d15
            per[f"{cid} {key}"] = [int(ok), int(ex4), int(d15)]
name = sys.argv[1]
json.dump(per, open(OUT / f"per-{name}.json", "w"))
print(json.dumps({"cand": name, **side}))
