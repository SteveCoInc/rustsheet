"""T-3993: score the two candidate math values of a composite cell (decision 0297).

Run from the repository root after `make build`: tools/.venv/bin/python docs/research/t3993/score.py

Every composite cell of probes and probes-numeric that the composite reader reads
(`rustsheet_tools.accuracy.composite`) gets two math values:
- (a) the exact composition: no inner rounding;
- (b) the outer node's math at the doubles its inner nodes round to.
For each, the script counts the cells where Excel's value (the oracle sidecar) and the engine's
(`rustsheet-eval show --all`) are within 1e-9 relative of it, and both (R1(c)'s Phase 1b evidence),
over all numeric cells and over the cells task T-3993 names. No candidate is fitted: the counts
decide between two structural definitions, and both are written out in full.
"""

import json
import re
import subprocess
import sys
from collections import Counter
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import mpmath

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/src"))
from rustsheet_tools.accuracy import composite  # noqa: E402 (the tools package is on sys.path above)
from rustsheet_tools.accuracy.eval_math import candidates  # noqa: E402 (as above)
from rustsheet_tools.accuracy.model import NoValueError  # noqa: E402 (as above)
from rustsheet_tools.accuracy.tables import DroppedError  # noqa: E402 (as above)

EVAL = ROOT / "target/release/rustsheet-eval"
TOLERANCE = mpmath.mpf("1e-9")
# The cells T-3993's Context names (one example cell per case; the case's other composites count
# under "all").
NAMED = {
    "probes-numeric/t3443-basym-sweep.case.json",
    "probes-numeric/t3443-basym-hold.case.json",
    "probes-numeric/t3443-basym-a.case.json",
    "probes-numeric/t3566-beta-inv-tiny-shape-and-range.case.json",
    "probes/t3444-binom-range-inv-large-n.case.json",
    "probes-numeric/t1746-norm-s-inv-central.case.json",
    "probes-numeric/m9-norm-s-inv.case.json",
}
_BLOCK = re.compile(r"^(?P<cell>\S+!\S+)\s+\S+\n(?:    .*\n)*?    actual  : (?P<actual>.*)$", re.MULTILINE)


def cases() -> list[dict[str, str]]:
    out = []
    for suite in ("probes", "probes-numeric"):
        listing = json.loads(
            subprocess.run([EVAL, "cases", suite], capture_output=True, check=True, text=True).stdout
        )
        out += [e for e in listing if e["path"].endswith(".case.json")]
    return out


def engine_values(case_id: str) -> dict[str, str]:
    shown = subprocess.run(
        [EVAL, "show", "--all", case_id], capture_output=True, check=True, text=True
    ).stdout
    return {m["cell"]: m["actual"].strip() for m in _BLOCK.finditer(shown + "\n")}


def score_case(entry: dict[str, str]) -> list[tuple[str, str, str, str, str | None, str | None]]:
    """(case, cell, Excel, engine, math a, math b) per composite cell with Excel and engine
    numbers. Option (b) is the committed math sidecar's value (`rustsheet-accuracy math`); option
    (a) is computed here."""
    found = [c for c in candidates(Path(entry["path"]), Path(entry["sidecar"])) if c.tree is not None]
    if not (found := [c for c in found if not isinstance(c.args, str)]):
        return []
    sidecar = Path(entry["sidecar"])
    oracle = json.loads(sidecar.read_text()) if sidecar.exists() else {}
    math_file = sidecar.with_name(sidecar.name.removesuffix(".oracle.json") + ".math.json")
    sheets = (json.loads(math_file.read_text()) if math_file.exists() else {}).get("sheets") or {}
    engine = engine_values(entry["id"])
    rows = []
    for c in found:
        excel = (oracle.get("sheets") or {}).get(c.sheet, {}).get(c.a1)
        actual = engine.get(f"{c.sheet}!{c.a1}")
        if not (isinstance(excel, dict) and excel.get("t") == "n") or actual is None or c.tree is None:
            continue
        try:
            exact = composite.compute(c.tree, rounded=False).value
        except (NoValueError, DroppedError):
            exact = None
        rounded = sheets.get(c.sheet, {}).get(c.a1, {}).get("v")
        rows.append((entry["id"], f"{c.sheet}!{c.a1}", repr(excel["v"]), actual, exact, rounded))
    return rows


def within(x: str, ref: str | None) -> bool:
    if ref is None:
        return False
    try:
        xv = mpmath.mpf(x)
    except ValueError:
        return False
    r = mpmath.mpf(ref)
    return abs(xv - r) <= TOLERANCE * abs(r) if r else xv == 0


def main() -> None:
    mpmath.mp.dps = 60
    with ProcessPoolExecutor() as pool:
        rows = [r for found in pool.map(score_case, cases()) for r in found]
    rows.sort()
    with (Path(__file__).parent / "cells.tsv").open("w") as out:
        out.write("case\tcell\texcel\tengine\tmath_a\tmath_b\n")
        for row in rows:
            out.write("\t".join(str(v) for v in row) + "\n")
    for label, subset in (("all", rows), ("named cases", [r for r in rows if r[0] in NAMED])):
        counts: Counter[str] = Counter()
        for _case, _cell, excel, engine, a, b in subset:
            for name, ref in (("a", a), ("b", b)):
                counts[f"{name} has value"] += ref is not None
                ex, en = within(excel, ref), within(engine, ref)
                counts[f"{name} excel"] += ex
                counts[f"{name} engine"] += en
                counts[f"{name} both"] += ex and en
        print(f"{label}: {len(subset)} composite cells with Excel and engine numbers")
        for name in ("a", "b"):
            print(
                f"  ({name}) value {counts[f'{name} has value']}, Excel within 1e-9 {counts[f'{name} excel']},"
                f" engine {counts[f'{name} engine']}, both {counts[f'{name} both']}"
            )


if __name__ == "__main__":
    main()
