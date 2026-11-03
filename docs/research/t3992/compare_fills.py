"""T-3992: compare independent excel-graph fills of the same case bytes, cell by cell.

Each refill is `rustsheet-oracle fill -b excel-graph --force` with the oracle cache
(eval/.cache/oracle/excel-graph) emptied first, so every fill is a fresh Excel for the web
recalculation; its sidecar is copied aside and the committed one restored (no sidecar under eval/
changes). This script reads the committed sidecar plus every `<fills>/<case>.<tag>.oracle.json`,
and writes, per case, the cells whose bits differ in any fill (all values, and the spread in ulps
between the extreme doubles) to docs/research/t3992/diffs.json, then prints the per-case summary.

Usage (repo root): tools/.venv/bin/python docs/research/t3992/compare_fills.py <fills-dir>
"""

import json
import struct
import sys
from pathlib import Path

# suite/case, then the kernel family it measures.
TABLE = """
probes-numeric/t3349-t-inv-2t-sweep Student t quantile
probes-numeric/t3478-irr-r4-wide IRR
probes/t3850-rate-same-state RATE
probes/fin-cumipmt-tiny-rate-structure CUMPRINC/CUMIPMT
probes-numeric/t2143-ln-near-ties-fill1 LN (control)
probes-numeric/t2143-ln-near-ties-fill2 LN (control, D1 label 2)
probes-numeric/t2143-ln-near-ties-fill3 LN (control, D1 label 3)
probes-numeric/t3311-gamma-inv-lower-tail-cdf gamma quantile
probes-numeric/t3570-r5-bpser-lg-aux lnGamma/GAMMA
probes-numeric/t3953-f-tail-bfrac-tiny incomplete beta (bfrac)
probes-numeric/t2464-r2-bpser-small-gl incomplete beta (bpser)
"""
CASES = [
    (kernel, *path.split("/")) for path, kernel in (row.split(" ", 1) for row in TABLE.strip().splitlines())
]


def ordered(x: float) -> int:
    """Doubles mapped to integers that are adjacent exactly when the doubles are adjacent."""
    (i,) = struct.unpack("<q", struct.pack("<d", x))
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def cells(path: Path) -> dict[str, object]:
    sidecar = json.loads(path.read_text())
    return {f"{s}!{a}": v for s, sheet in sidecar["sheets"].items() for a, v in sheet.items()}


def stamp(path: Path) -> str:
    """When the fill ran (the sidecar's `produced_at`)."""
    return str(json.loads(path.read_text())["source"]["produced_at"])


def main() -> None:
    fills = Path(sys.argv[1])
    out: dict[str, object] = {}
    for kernel, suite, case in CASES:
        committed = Path(f"eval/suites/{suite}/cases/{case}.oracle.json")
        runs = {"committed": cells(committed)}
        produced = {"committed": stamp(committed)}
        for p in sorted(fills.glob(f"{case}.*.oracle.json")):
            tag = p.name.removeprefix(case + ".").removesuffix(".oracle.json")
            runs[tag], produced[tag] = cells(p), stamp(p)
        base = runs["committed"]
        differ = {}
        max_ulp = 0
        # Every cell any fill reports, so a cell missing from one fill counts as a difference.
        for key in sorted(set().union(*runs.values())):
            v = base.get(key)
            seen = {tag: r.get(key) for tag, r in runs.items()}
            if all(x == v for x in seen.values()):
                continue
            nums = [x["v"] for x in seen.values() if isinstance(x, dict) and x.get("t") == "n"]
            ulp = None
            if len(nums) == len(seen):
                ulp = max(ordered(float(n)) for n in nums) - min(ordered(float(n)) for n in nums)
                max_ulp = max(max_ulp, ulp)
            differ[key] = {"ulp": ulp, **{t: (x or {}).get("v") for t, x in seen.items()}}
        numeric = sum(1 for v in base.values() if isinstance(v, dict) and v.get("t") == "n")
        out[case] = {
            "kernel": kernel,
            "suite": suite,
            "fills": produced,
            "cells": len(base),
            "differ": differ,
        }
        print(
            f"{kernel:26} {case:36} fills={len(runs)} cells={len(base)} numeric={numeric} "
            f"differ={len(differ)} max_ulp={max_ulp}"
        )
    Path("docs/research/t3992/diffs.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
