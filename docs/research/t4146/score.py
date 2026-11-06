"""T-4146: score the plain and twice-rounded annuity forms (forms.py) on filled FV/PV probe cases.

The split and accept rule are the fit lab's (tools/src/rustsheet_tools/fitlab/holdout.py, decision 0188 R4.1):
train and test by the case/cell hash, margin `DEFAULT_MARGIN` (5 points, fixed before scoring), and a candidate is
accepted only when it beats the correctly rounded one on test. "correctly rounded" is the exact value of the
call (rational (1+r)^n, no intermediate rounding) rounded once to a double.
Prints per function and side: bit-exact / within 4 ulp of each candidate.

Run (from tools/): uv run python ../docs/research/t4146/score.py <case-id>...   (e.g. probes/t4146-annuity-twice-rounded)
"""

import json
import math
import re
import struct
import sys
from fractions import Fraction
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import forms  # noqa: E402

from rustsheet_tools.fitlab.holdout import DEFAULT_MARGIN, HOLDOUT_PERMILLE, HOLDOUT_SALT  # noqa: E402
import hashlib  # noqa: E402

ROOT = Path(__file__).resolve().parents[3]


def is_holdout(case: str, key: str) -> bool:
    # The fit lab's `is_holdout` on a bare (case, Sheet!A1) pair.
    d = hashlib.sha256(f"{HOLDOUT_SALT}\0{case}\0{key}".encode()).digest()
    return int.from_bytes(d[:8], "big") % 1000 < HOLDOUT_PERMILLE


def exact(func: str, r: float, n: float, pmt: float, x: float, t: float) -> float:
    fr, fn = Fraction(r), int(n)
    p = (1 + fr) ** fn
    k = (1 + fr) if t else Fraction(1)
    a = k * (p - 1) / fr
    if func == "FV":
        return float(-(Fraction(x) * p + Fraction(pmt) * a))
    return float(-(Fraction(x) + Fraction(pmt) * a) / p)


def ulps(a: float, b: float) -> int:
    def o(v: float) -> int:
        i = struct.unpack("<q", struct.pack("<d", v))[0]
        return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)
    return abs(o(a) - o(b))


def main() -> None:
    cands = {"plain": None, "twice": None, "correctly rounded": None}
    tally: dict[tuple[str, str, str], list[int]] = {}
    for case_id in sys.argv[1:]:
        suite, _, name = case_id.partition("/")
        base = ROOT / "eval/suites" / suite / "cases" / name
        case = json.loads(Path(f"{base}.case.json").read_text())
        oracle = json.loads(Path(f"{base}.oracle.json").read_text())["sheets"]
        for sh in case["sheets"]:
            for cell, f in sh["cells"].items():
                m = re.fullmatch(r"=(FV|PV)\((.*)\)", f)
                e = oracle.get(sh["name"], {}).get(cell)
                if not m or not e or e["t"] != "n":
                    continue
                func, args = m[1], [float(a) for a in m[2].split(",")]
                side = "test" if is_holdout(case_id, f"{sh['name']}!{cell}") else "train"
                for c in cands:
                    if c == "correctly rounded":
                        v = exact(func, *args)
                    else:
                        v = (forms.fv if func == "FV" else forms.pv)(c, *args)
                    t = tally.setdefault((func, side, c), [0, 0, 0])
                    t[0] += 1
                    t[1] += v == e["v"]
                    t[2] += math.isfinite(v) and ulps(v, e["v"]) <= 4
    print(f"margin {DEFAULT_MARGIN}")
    for (func, side, c), (n, ex, u4) in sorted(tally.items()):
        print(f"{func} {side:5} {c:18} bit-exact {ex}/{n}  4ulp {u4}/{n}")


main()
