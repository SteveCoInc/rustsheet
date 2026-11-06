"""T-4349: one row per CUMPRINC/CUMIPMT cell of the fit lab's cumulative fits, with both short-range forms' values.

`base` is today's engine (the cut `GEOMETRIC_MIN_LEAD` 0.3) and `geo` the same engine with the cut at 0 (the geometric
sum at every leading exponent). The two differ only on short ranges below a lead of 0.3, so a short-range selector
between the two forms is scored from this table without another build: a cell takes `geo` where the selector says
geometric and `base` where it does not. `holdout` is the fit lab's fixed split (holdout.is_holdout).

Run from tools/: uv run python ../docs/research/t4349/dump.py BASE GEO [FIT ...] > rows.tsv
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.holdout import is_holdout
from rustsheet_tools.fitlab.kernels import registry
from rustsheet_tools.fitlab.score import Candidate, ExcelError, ulp_distance, within_fifteen_digits
from rustsheet_tools.fitlab.sweep import Cell, load_sweep, numeric_arg


def value(candidate: Candidate, cell: Cell) -> float | None:
    try:
        v = candidate.evaluate(cell)
    except (ExcelError, KeyError, TypeError, ValueError):
        return None
    return v if isinstance(v, float) else None


def distance(expected: float, v: float | None) -> tuple[str, str]:
    if v is None:
        return "error", "1"
    return str(ulp_distance(expected, v)), "0" if within_fifteen_digits(expected, v) else "1"


def main() -> None:
    base, geo = Path(sys.argv[1]), Path(sys.argv[2])
    fits = sys.argv[3:] or ["cumulative", "cumulative-fresh", "cumulative-tiny"]
    head = "fit case key func rate nper pv start end type lead count holdout expected"
    print("\t".join([*head.split(), "base", "base_ulps", "base_vis", "geo", "geo_ulps", "geo_vis"]))
    for name in fits:
        spec = registry()[name]
        cells = [c for c in load_sweep(spec.cases, spec.pattern).cells if isinstance(c.expected, float)]
        cases = sorted({c.case for c in cells})
        cands = (fit.engine_candidate(cases, base, "base"), fit.engine_candidate(cases, geo, "geo"))
        for cell in cells:
            try:
                rate, nper, pv, start, end, kind = (numeric_arg(a) for a in cell.args)
            except (TypeError, ValueError):
                continue
            first, last = math.ceil(start), math.trunc(end)
            lead = (nper - last + 1) * math.log1p(rate) if rate > -1 else math.nan
            out = [name, cell.case, cell.key, cell.formula.split("(")[0][1:]]
            out += [repr(x) for x in (rate, nper, pv, start, end, kind)]
            out += [repr(lead), str(last - first + 1), "1" if is_holdout(cell) else "0", repr(cell.expected)]
            for cand in cands:
                v = value(cand, cell)
                out += ["error" if v is None else repr(v), *distance(cell.expected, v)]
            print("\t".join(out))


if __name__ == "__main__":
    main()
