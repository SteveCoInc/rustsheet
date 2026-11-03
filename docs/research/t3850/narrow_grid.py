"""T-3850: can T-3698's flat-secant restart be narrowed so that no point of probes/t3906-rate-restart-oos leaves the
Gate A envelope, with no other RATE cell regressing?

The candidate engine is a release `rustsheet-eval` built with a temporary, uncommitted switch in `rate_secant`
(crates/rustsheet-engine/src/functions/financial/financial_util.rs) that reads, in place of the constants:
  RS_OFF       1: never restart (the hold)
  RS_K         last step that may restart (RATE_RESTART_STEPS, 16)
  RS_FLO       |f1| floor (RATE_RESTART_F_MIN, 1.3e-7)
  RS_FHI       |f1| cut (RATE_RESTART_F_MAX, 1e-4)
  RS_DEPTH     restarts allowed in one call (1 today; the restarted run gets RS_DEPTH - 1)
  RS_FALLBACK  1: when the restarted run is an error, continue with the hold instead

It scores every `=RATE(` cell of the probes and probes-numeric cases where the rule and the hold differ (from the
`rule.json` and `hold.json` dumps this script writes with `dump`). Envelope: within half a unit of Excel's 15th
significant digit, or within 1e-9 relative of the case's math.tsv root; an error against a number is outside.

Run from a checkout that has the t3906 case:
  python3 docs/research/t3850/narrow_grid.py dump <eval-binary> <out-dir> rule        # current constants
  RS_OFF=1 python3 docs/research/t3850/narrow_grid.py dump <eval-binary> <out-dir> hold
  python3 docs/research/t3850/narrow_grid.py grid <eval-binary> <out-dir> '<json list of env dicts>'
"""

import csv
import json
import math
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/src"))
from rustsheet_tools.snapshot import parse_show  # noqa: E402

CASE = "probes/t3906-rate-restart-oos"


def rate_cases() -> list[str]:
    cases = []
    for suite in ("probes", "probes-numeric"):
        base = ROOT / "eval/suites" / suite / "cases"
        for p in sorted(base.rglob("*.case.json")):
            if "RATE(" in p.read_text() and Path(str(p).replace(".case.json", ".oracle.json")).exists():
                cases.append(f"{suite}/{p.relative_to(base).as_posix()[:-10]}")
    return cases


def show(binary: str, cases: list[str], env: dict[str, str]) -> dict[str, dict[str, str]]:
    def one(cid: str) -> tuple[str, dict[str, dict[str, str]]]:
        out = subprocess.run(
            [binary, "--root", str(ROOT), "show", cid + ".case.json", "--all"],
            capture_output=True,
            text=True,
            check=True,
            env={**os.environ, **env},
        ).stdout
        return cid, parse_show(out)

    res = {}
    with ThreadPoolExecutor(8) as ex:
        for cid, cells in ex.map(one, cases):
            for key, c in cells.items():
                if "RATE(" in c.get("formula", "").upper():
                    res[f"{cid} {key}"] = c
    return res


def num(s: str) -> float | None:
    try:
        return float(s)
    except ValueError:
        return None


def math_roots() -> dict[str, float]:
    roots = {}
    with (ROOT / f"eval/suites/{CASE.replace('/', '/cases/')}.math.tsv").open() as fh:
        for r in csv.DictReader(fh, delimiter="\t"):
            v = num(r["math"])
            if v is not None:
                roots[f"{CASE} {r['cell']}"] = v
    return roots


def inside(key: str, expected: str, actual: str, roots: dict[str, float]) -> bool:
    e, a = num(expected), num(actual)
    if e is None or a is None:
        return e is None and a is None and expected == actual
    if e == 0:
        return a == 0
    if abs(a - e) <= 0.5 * 10 ** (math.floor(math.log10(abs(e))) - 14):
        return True
    m = roots.get(key)
    return m is not None and abs(a - m) <= 1e-9 * abs(m)


def main() -> None:
    mode, binary, out = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    if mode == "dump":
        cells = show(binary, rate_cases(), {})
        dump = {k: [c["expected"], c["actual"]] for k, c in cells.items()}
        (out / f"{sys.argv[4]}.json").write_text(json.dumps(dump))
        return
    rule = json.loads((out / "rule.json").read_text())
    hold = json.loads((out / "hold.json").read_text())
    cases = sorted({k.split()[0] for k in rule if rule[k][1] != hold[k][1]})
    roots = math_roots()
    for env in json.loads(sys.argv[4]):
        got = {k: c["actual"] for k, c in show(binary, cases, env).items() if k in rule}
        # Regressions: cells outside the t3906 case that the merged rule gets bit-exact and the candidate does not.
        reg = [k for k in got if CASE not in k and rule[k][1] == rule[k][0] and got[k] != rule[k][0]]
        probe = [k for k in got if CASE in k]
        left = [
            k.split("!")[1]
            for k in probe
            if inside(k, rule[k][0], hold[k][1], roots) and not inside(k, rule[k][0], got[k], roots)
        ]
        print(
            json.dumps(
                {
                    "env": env,
                    "regressions": len(reg),
                    "inside": sum(inside(k, rule[k][0], got[k], roots) for k in probe),
                    "bit_exact": sum(got[k] == rule[k][0] for k in probe),
                    "left_vs_hold": left,
                }
            ),
            flush=True,
        )


if __name__ == "__main__":
    main()
