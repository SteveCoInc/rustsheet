"""T-3963: score RS_V variants of the temporary switch (trace.diff) on the fit lab's hash split.

Prints, per variant, train and hold-out counts of bit-exact, within 4 ulp and within 15 significant digits
(Excel's displayed precision), plus the wrong-type cells of an optional fresh sweep. Errors count as equal only
when their text is the same.

Run (from tools/, whose venv has the fit lab): uv run python ../docs/research/t3963/holdout.py <cells.tsv> <probe-binary> <variant>...
(cells.tsv: case, cell, formula, oracle value as `repr(float)` or `E:<error>`; extract.py writes it)
"""

import collections
import hashlib
import os
import struct
import subprocess
import sys

from rustsheet_tools.fitlab.holdout import HOLDOUT_PERMILLE, HOLDOUT_SALT


def is_holdout(case: str, cell: str) -> bool:
    # The fit lab's split (tools/src/rustsheet_tools/fitlab/holdout.py `is_holdout`) on a bare (case, cell)
    # pair, since this script reads a TSV rather than the fit lab's `Cell`.
    d = hashlib.sha256(f"{HOLDOUT_SALT}\0{case}\0{cell}".encode()).digest()
    return int.from_bytes(d[:8], "big") % 1000 < HOLDOUT_PERMILLE


def norm(x: str) -> str:
    if x.startswith("E:"):
        # The probe prints String("#NUM!"); the sidecar extract prints #NUM!.
        return "E:" + x[2:].replace('String("', "").replace('")', "")
    return x


def bits(x: float) -> int:
    return int(struct.unpack("<q", struct.pack("<d", x))[0])


def grade(expected: str, got: str) -> set[str]:
    e, g = norm(expected), norm(got)
    if e.startswith("E:") or g.startswith("E:"):
        return {"exact", "ulp4", "d15"} if e == g else {"type"}
    x, y = float(e), float(g)
    out = set()
    if x == y:
        out |= {"exact", "ulp4", "d15"}
    elif abs(bits(x) - bits(y)) <= 4:
        out |= {"ulp4"}
    if f"{x:.14e}" == f"{y:.14e}":
        out.add("d15")
    return out


def main() -> None:
    cells = [line.rstrip("\n").split("\t") for line in open(sys.argv[1])]
    for variant in sys.argv[3:]:
        p = subprocess.run(
            [sys.argv[2]], input="\n".join(c[2] for c in cells) + "\n", capture_output=True, text=True,
            env={**os.environ, "RS_V": variant}, check=True,
        )
        out = p.stdout.split("\n")
        n: collections.Counter[str] = collections.Counter()
        for c, g in zip(cells, out):
            side = "test" if is_holdout(c[0], c[1]) else "train"
            n[side] += 1
            for k in grade(c[3], g):
                n[f"{side}_{k}"] += 1
        row = []
        for side in ("train", "test"):
            row.append(
                f"{side} {n[side + '_exact']}/{n[side + '_ulp4']}/{n[side + '_d15']} of {n[side]}"
                f" (wrong type {n[side + '_type']})"
            )
        print(f"RS_V={variant}: " + "; ".join(row))


main()
