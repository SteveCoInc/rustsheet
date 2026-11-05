"""T-3524: what the geometric-principal cuts decide, cell by cell, and the Gate A envelope (decision 0188 R2).

For the old cells (fit `cumulative`) and the fresh sweep (`cumulative-fresh`) it compares today's engine with each
build given as NAME=PATH (docs/research/t3524/*.patch): the cells one gets bit-exact and the other does not, with the
other's ulp distance and whether it is visibly off (beyond half a unit of Excel's 15th digit), and the exact counts by
the leading exponent `lead = (nper-end+1)·ln(1+rate)` and the range kind (short: end - start < 32). It writes
docs/research/t3524/gate_a.tsv for the fresh sweep: per cell the 60-digit value (`kernels.cumulative.exact`), each
candidate's value and relative error, and its Gate A verdict (`math` within 1e-9 of the 60-digit value, `excel` within
15 significant digits of Excel, else OUTSIDE).

Run from tools/: uv run python ../docs/research/t3524/analyze.py NAME=PATH ...
"""

from __future__ import annotations

import itertools
import math
import re
import sys
from pathlib import Path

import mpmath

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import cumulative, registry
from rustsheet_tools.fitlab.score import Candidate, ExcelError, ulp_distance, within_fifteen_digits
from rustsheet_tools.fitlab.sweep import Cell, load_sweep, numeric_arg

OUT = Path(__file__).resolve().parent / "gate_a.tsv"
FUNCTION = re.compile(r"^=(CUMPRINC|CUMIPMT)\(")
BUCKETS = [0.0, 0.01, 0.095, 0.3, 0.398, 1.0, 2.9, 3.6, 4.3, 10.0, math.inf]


def value(candidate: Candidate, cell: Cell) -> float | None:
    try:
        v = candidate.evaluate(cell)
    except (ExcelError, KeyError, TypeError, ValueError):
        return None
    return v if isinstance(v, float) else None


def lead_of(cell: Cell) -> tuple[float, bool]:
    rate, nper, _, start, end, _ = (numeric_arg(a) for a in cell.args)
    return (nper - math.trunc(end) + 1) * math.log1p(rate), math.trunc(end) - math.ceil(start) < 32


def bucket(lead: float) -> str:
    for lo, hi in itertools.pairwise(BUCKETS):
        if lo <= lead < hi:
            return f"{lo:g}..{hi:g}"
    return "?"


def compare(fit_name: str, builds: dict[str, Path]) -> tuple[list[Cell], dict[str, Candidate]]:
    spec = registry()[fit_name]
    sweep = load_sweep(spec.cases, spec.pattern)
    cases = sorted({c.case for c in sweep.cells})
    candidates = {"engine": fit.engine_candidate(cases)}
    for name, path in builds.items():
        candidates[name] = fit.engine_candidate(cases, path, name)
    engine = candidates["engine"]
    print(f"== {fit_name}: {len(sweep.cells)} cells")
    table: dict[tuple[str, bool], dict[str, int]] = {}
    for cell in sweep.cells:
        key = (bucket(lead_of(cell)[0]), lead_of(cell)[1])
        row = table.setdefault(key, {"cells": 0})
        row["cells"] += 1
        for name, cand in candidates.items():
            if value(cand, cell) == cell.expected:
                row[name] = row.get(name, 0) + 1
    print("lead bucket\trange\tcells\t" + "\t".join(candidates))
    for (b, short), row in sorted(
        table.items(), key=lambda kv: (not kv[0][1], float(kv[0][0].split("..")[0]))
    ):
        counts = "\t".join(str(row.get(n, 0)) for n in candidates)
        print(f"{b}\t{'short' if short else 'long'}\t{row['cells']}\t{counts}")
    for name, cand in candidates.items():
        if name == "engine":
            continue
        for label, a, b in (("lost", engine, cand), ("won", cand, engine)):
            print(
                f"-- {name}: cells the {'engine' if label == 'lost' else 'build'} gets exact and the other does not"
            )
            for cell in sweep.cells:
                va, vb = value(a, cell), value(b, cell)
                if va == cell.expected and vb != cell.expected:
                    ulps = "error" if vb is None else str(ulp_distance(cell.expected, vb))
                    visible = vb is None or not within_fifteen_digits(cell.expected, vb)
                    lead, short = lead_of(cell)
                    case = cell.case.rsplit("/", 1)[-1].removesuffix(".case.json")
                    print(
                        f"  {case} {cell.key} {cell.formula} lead {lead:.3g} {'short' if short else 'long'}"
                        f" {ulps} ulp{' VISIBLE' if visible else ''}"
                    )
    return sweep.cells, candidates


def gate_a(cells: list[Cell], candidates: dict[str, Candidate]) -> None:
    head = ["case", "cell", "formula", "mpmath_60", "excel"]
    for name in candidates:
        head += [name, f"{name}_rel", f"{name}_gate_a"]
    rows = ["\t".join(head)]
    outside: dict[str, list[str]] = {n: [] for n in candidates}
    for cell in cells:
        m = FUNCTION.match(cell.formula)
        if m is None:
            continue
        with mpmath.workdps(60):
            exact = cumulative.exact_mp(m[1], [numeric_arg(a) for a in cell.args])
            out = [cell.case.rsplit("/", 1)[-1].removesuffix(".case.json"), cell.key, cell.formula]
            out += [mpmath.nstr(exact, 20), repr(cell.expected)]
            for name, cand in candidates.items():
                v = value(cand, cell)
                if v is None:
                    out += ["error", "-", "OUTSIDE"]
                    outside[name].append(cell.key)
                    continue
                rel = abs(mpmath.mpf(v) - exact) / abs(exact)
                verdict = (
                    "math"
                    if rel <= mpmath.mpf("1e-9")
                    else ("excel" if within_fifteen_digits(cell.expected, v) else "OUTSIDE")
                )
                if verdict == "OUTSIDE":
                    outside[name].append(cell.key)
                out += [repr(v), f"{float(rel):.3g}", verdict]
        rows.append("\t".join(out))
    OUT.write_text("\n".join(rows) + "\n")
    for name, keys in outside.items():
        print(f"Gate A {name}: {len(cells) - len(keys)} of {len(cells)} inside; outside: {keys}")


def main() -> None:
    builds = {s.split("=", 1)[0]: Path(s.split("=", 1)[1]) for s in sys.argv[1:]}
    compare("cumulative", builds)
    cells, candidates = compare("cumulative-fresh", builds)
    gate_a(cells, candidates)


if __name__ == "__main__":
    main()
