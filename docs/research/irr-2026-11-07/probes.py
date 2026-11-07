"""T-4844's IRR probe cases: the guess ladder, the path-invariant transforms and the trailing-zero padding ladder.

All three take level annuities of probes/t3406-irr-num-boundary where Excel returns a root and the engine's secant from
0.1 fails (financial.md "IRR's second stage ignores the guess and reads only NPV's values").

- t4844-irr-guess-ladder: 12 annuities (21 to 61 flows). Per annuity: the default IRR, the fixed point IRR(flows,
  default) with the default passed by reference so its 17 digits survive, then guesses -0.9..0.9 by 0.05. If one
  guess reproduced the default on every annuity, Excel's recovery would be a secant run from that guess.
- t4844-irr-stage-invariance: 4 of them. Flows times 2^k (exact in binary: the secant's path in d is unchanged, so
  only an absolute tolerance can move the value), 1, 2, 5 and 20 trailing zeros (NPV unchanged, n larger), the flows
  negated, and negated with 5 trailing zeros.
- t4844-irr-pad-ladder: the same 4, padded with trailing zeros to N flows around the replica's underflow threshold.
  IRR's NPV multiplies the discount out to the last flow, so at a visited rate with x = 1 + r < 1 the discount x^(N-1)
  drops below the smallest normal double and the call is #DIV/0! ("IRR is #DIV/0! where its secant cannot evaluate
  NPV"). Excel's threshold therefore measures the smallest x any stage visits.

    python3 -I docs/research/irr-2026-11-07/probes.py   # writes each case when missing, else checks it
"""

from __future__ import annotations

import json
import sys
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CASES = ROOT / "eval/suites/probes/cases"
SOURCE = CASES / "t3406-irr-num-boundary.case.json"
LADDER_SETS = ["A6", "A8", "A10", "A11", "A28", "A30", "A32", "A51", "A53", "A55", "A75", "A77"]
GUESSES = [round(-0.9 + 0.05 * k, 2) for k in range(37)]
FOUR = ["A6", "A10", "A51", "A77"]
POWERS = [-30, -10, -3, -1, 1, 3, 10, 30]
ZEROS = [1, 2, 5, 20]
# The replica's threshold N per annuity: the smallest x its secant from 0.1 visits gives x^(N-1) < 2^-1022.
THRESHOLDS = [892, 293, 580, 183]
FACTORS = [0.5, 0.7, 0.8, 0.9, 0.95, 0.98, 1.0, 1.02, 1.05, 1.1, 1.3, 1.6]


def source_flows(name: str) -> list[str]:
    formula = json.loads(SOURCE.read_text())["sheets"][0]["cells"][name]
    return formula[formula.index("{") + 1 : formula.rindex("}")].split(",")


def literal(flows: list[str]) -> str:
    return "{" + ",".join(flows) + "}"


def guess_ladder() -> list[str]:
    formulas: list[str] = []
    for name in LADDER_SETS:
        flows = literal(source_flows(name))
        default = f"A{len(formulas) + 1}"
        formulas += [f"=IRR({flows})", f"=IRR({flows},{default})"]
        formulas += [f"=IRR({flows},{g!r})" for g in GUESSES]
    return formulas


def stage_invariance() -> list[str]:
    formulas: list[str] = []
    for name in FOUR:
        flows = source_flows(name)
        plain = literal(flows)
        padded = {z: literal(flows + ["0"] * z) for z in ZEROS}
        formulas += [f"=IRR({plain})", *(f"=IRR({plain}*2^{k})" for k in POWERS)]
        formulas += [*(f"=IRR({padded[z]})" for z in ZEROS), f"=IRR(-{plain})", f"=IRR(-{padded[5]})"]
    return formulas


def pad_ladder() -> list[str]:
    formulas: list[str] = []
    for name, threshold in zip(FOUR, THRESHOLDS, strict=True):
        flows = source_flows(name)
        for factor in FACTORS:
            total = max(len(flows) + 1, round(threshold * factor))
            formulas.append(f"=IRR({literal(flows + ['0'] * (total - len(flows)))})")
    return formulas


PROBES: dict[str, tuple[Callable[[], list[str]], str]] = {
    "t4844-irr-guess-ladder": (
        guess_ladder,
        f"T-4844: IRR guess ladder on 12 level annuities of t3406-irr-num-boundary where Excel returns and the "
        f"engine's secant fails ({', '.join(LADDER_SETS)}). Blocks of {2 + len(GUESSES)} rows: the default IRR, "
        "IRR(flows, default) by reference (fixed point), then guesses -0.9..0.9 by 0.05.",
    ),
    "t4844-irr-stage-invariance": (
        stage_invariance,
        f"T-4844: IRR's guess-independent stage under path-invariant transforms, on t3406-irr-num-boundary "
        f"{', '.join(FOUR)}. Blocks of {1 + len(POWERS) + len(ZEROS) + 2} rows: the flows, times 2^k for k in "
        f"{POWERS}, with {ZEROS} trailing zeros, negated, negated with 5 trailing zeros.",
    ),
    "t4844-irr-pad-ladder": (
        pad_ladder,
        f"T-4844: IRR on four t3406-irr-num-boundary annuities ({', '.join(FOUR)}) padded with trailing zeros to N "
        f"flows, N = the replica's underflow threshold ({', '.join(map(str, THRESHOLDS))}) times {FACTORS}, "
        f"{len(FACTORS)} rows per annuity.",
    ),
}


def main() -> int:
    status = 0
    for name, (build, description) in PROBES.items():
        path = CASES / f"{name}.case.json"
        sheets = [{"name": "P", "cells": {f"A{i}": f for i, f in enumerate(build(), start=1)}}]
        if not path.exists():
            case = {
                "format": "rustsheet-case/v1",
                "description": f"{description} docs/research/irr-2026-11-07/probes.py.",
                "tags": ["M9", "probe", "financial", "irr"],
                "sheets": sheets,
                "expected": {},
            }
            path.write_text(json.dumps(case, indent=2) + "\n")
            print(f"wrote {path.relative_to(ROOT)}")
        elif json.loads(path.read_text())["sheets"] != sheets:
            print(f"{path.relative_to(ROOT)} differs from the generator")
            status = 1
        else:
            print(f"{name}: up to date")
    return status


if __name__ == "__main__":
    sys.exit(main())
