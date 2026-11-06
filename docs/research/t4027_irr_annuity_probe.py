"""T-4027: separate what decides IRR's type on level annuities where the secant fails.

Usage (from tools/, mpmath is not a tools dependency yet):

    uv run --with mpmath python ../docs/research/t4027_irr_annuity_probe.py

Writes eval/suites/probes/cases/t4027-irr-annuity-scale-guess.case.json (when missing; otherwise
checks it) and its `.roots.tsv` (Excel's value from the oracle sidecar, when filled, and the
50-digit root). t3406 put the type cut of `IRR({-P,1,...,1})` between roots -0.22 and -0.24 at
n = 20 and between -0.14 and -0.16 at n = 40. This probe straddles those cuts and varies one thing
at a time:
- scale: every flow times s (s = 2, 1000, 0.001). The root and the secant's iterates in the rate
  are unchanged, so a type change can only come from an absolute tolerance on NPV;
- guess: IRR's second argument (0, -0.1, -0.2, 0.3), which moves the secant's start and so its
  far point, but not the root.
Rows: block 1 (A1-A24) n = 20, roots -0.20 .. -0.25 by 0.01, scales 1, 2, 1000, 0.001; block 2
(A25-A48) n = 20, the same roots, guesses 0, -0.1, -0.2, 0.3; block 3 (A49-A68) n = 40, roots
-0.13 .. -0.17 by 0.01, scale 1, scale 1000, guess 0, guess -0.15.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from mpmath import findroot, mp, mpf, nstr

mp.dps = 60
ROOT = Path(__file__).resolve().parents[2]
CASE = ROOT / "eval/suites/probes/cases/t4027-irr-annuity-scale-guess.case.json"
ORACLE = CASE.with_name(CASE.name.replace(".case.json", ".oracle.json"))
ROOTS = CASE.with_name(CASE.name.replace(".case.json", ".roots.tsv"))
DESCRIPTION = (
    "T-4027: IRR's type on level annuities -P,1,...,1 where the secant fails, straddling t3406's cuts. "
    "A1-A24 n = 20, roots -0.20..-0.25, all flows scaled by 1, 2, 1000, 0.001; A25-A48 n = 20, same roots, "
    "guess 0, -0.1, -0.2, 0.3; A49-A68 n = 40, roots -0.13..-0.17, scale 1, scale 1000, guess 0, guess -0.15. "
    "50-digit roots in t4027-irr-annuity-scale-guess.roots.tsv (docs/research/t4027_irr_annuity_probe.py)."
)


def number(x: float) -> str:
    text = repr(x)
    return text[:-2] if text.endswith(".0") else text


def outlay(n: int, rate: float) -> float:
    return round(sum(1 / (1 + rate) ** (i + 1) for i in range(n)), 2)


def flows(n: int, rate: float, scale: float) -> list[float]:
    p = outlay(n, rate)
    if scale == 1:
        return [-p, *([1.0] * n)]
    return [-float(mpf(p) * mpf(scale)), *([float(scale)] * n)]


def points() -> list[tuple[str, list[float], float | None]]:
    out: list[tuple[str, list[float], float | None]] = []
    roots20 = [-0.20, -0.21, -0.22, -0.23, -0.24, -0.25]
    for s in (1.0, 2.0, 1000.0, 0.001):
        out.extend((f"n20-scale{number(s)}", flows(20, r, s), None) for r in roots20)
    for g in (0.0, -0.1, -0.2, 0.3):
        out.extend((f"n20-guess{number(g)}", flows(20, r, 1.0), g) for r in roots20)
    roots40 = [-0.13, -0.14, -0.15, -0.16, -0.17]
    for label, s, g in (
        ("scale1", 1.0, None),
        ("scale1000", 1000.0, None),
        ("guess0", 1.0, 0.0),
        ("guess-0.15", 1.0, -0.15),
    ):
        out.extend((f"n40-{label}", flows(40, r, s), g) for r in roots40)
    return out


def formula(values: list[float], guess: float | None) -> str:
    body = "=IRR({" + ",".join(map(number, values)) + "}"
    return body + (f",{number(guess)})" if guess is not None else ")")


def exact_root(values: list[float]) -> Any:
    npv = lambda r: sum(mpf(c) / (1 + r) ** i for i, c in enumerate(values))  # noqa: E731
    lo, hi = mpf("1e-6"), mpf(2)
    for _ in range(200):
        mid = (lo + hi) / 2
        if (npv(mid - 1) > 0) == (npv(hi - 1) > 0):
            hi = mid
        else:
            lo = mid
    return findroot(npv, (lo + hi) / 2 - 1)


def main() -> None:
    pts = points()
    cells = {f"A{i + 1}": formula(v, g) for i, (_, v, g) in enumerate(pts)}
    case = {
        "format": "rustsheet-case/v1",
        "description": DESCRIPTION,
        "tags": ["M9", "probe", "financial", "irr"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
        "check": [f"P!A{i + 1}" for i in range(len(pts))],
    }
    if CASE.exists():
        if json.loads(CASE.read_text()) != case:
            raise SystemExit(f"{CASE} differs from the generator")
    else:
        CASE.write_text(json.dumps(case, indent=2) + "\n")
    excel: dict[str, Any] = {}
    if ORACLE.exists():
        excel = json.loads(ORACLE.read_text())["sheets"]["P"]
    lines = ["cell\tblock\tformula\texcel\troot50"]
    for i, (block, v, g) in enumerate(pts):
        cell = f"A{i + 1}"
        e = excel.get(cell, {}).get("v", "")
        lines.append(f"{cell}\t{block}\t{formula(v, g)}\t{e}\t{nstr(exact_root(v), 50)}")
    ROOTS.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
