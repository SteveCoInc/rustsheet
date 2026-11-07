"""Score IRR's hypothesis bank (T-4836, plan.md Phase 2) cell by cell, on the `irr-type` fit's cells.

For every candidate of `irr_driver.CANDIDATES` and the engine, each cell's outcome type and bits against Excel, on
the train and test sides of the fit lab's split (holdout.py), and the cells gained and lost against the engine.
Writes bank.tsv (one row per candidate) and predictions.tsv (each shortlisted model's value on every cell) next to
this script:

    cd tools && RUSTSHEET_EVAL_BIN=../target/release/rustsheet-eval \\
        uv run python -I ../docs/research/irr-2026-11-07/bank.py

The shortlist is fixed below, after reading bank.tsv; predictions.tsv is rewritten from it on every run.
"""

from __future__ import annotations

import sys
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

from rustsheet_tools.fitlab.fit import engine_candidate
from rustsheet_tools.fitlab.holdout import is_holdout
from rustsheet_tools.fitlab.kernels.irr_driver import CANDIDATES, FITS, irr_candidate
from rustsheet_tools.fitlab.score import _measure, _typed
from rustsheet_tools.fitlab.sweep import Cell, load_sweep

HERE = Path(__file__).resolve().parent
ENGINE = "engine"
SHORTLIST = [
    "rate: hold on flat + run on past d = 1, cap 200",
    "bank: RATE+H12 Newton fallback",
    "bank: RATE, H5 r-half + H12",
    "bank: RATE, H5 r-half for run on",
]


def _cells() -> list[Cell]:
    fit = FITS[0]
    sweep = load_sweep(
        fit.cases, fit.pattern, fit.where, fit.errors, fit.calls, complex_text=fit.complex_text
    )
    return sweep.cells


def _outcomes(name: str) -> tuple[str, list[tuple[bool, bool, str]]]:
    """Per cell: (typed, bit-exact, the value as text)."""
    cells = _cells()
    if name == ENGINE:
        cand = engine_candidate(sorted({c.case for c in cells}))
    else:
        opts, family = CANDIDATES[name]
        cand = irr_candidate(name, opts, family)
    out = []
    for cell in cells:
        r = _measure(cell, cand)
        typed = _typed(cell, r)
        exact = typed and (cell.error is not None or r.ulps == 0)
        value = r.error if r.error is not None else repr(r.actual)
        out.append((typed, exact, value))
    return name, out


def main() -> int:
    cells = _cells()
    names = [ENGINE, *CANDIDATES]
    with ProcessPoolExecutor(max_workers=4) as pool:
        results = dict(pool.map(_outcomes, names))
    test = [is_holdout(c) for c in cells]
    eng = results[ENGINE]
    rows = [
        "candidate\ttyped\ttyped_train\ttyped_test\texact\texact_train\texact_test\ttype_gained\ttype_lost"
    ]
    for name in names:
        res = results[name]
        typed = [t for t, _, _ in res]
        exact = [e for _, e, _ in res]
        gained = sum(t and not et for t, (et, _, _) in zip(typed, eng, strict=True))
        lost = sum(et and not t for t, (et, _, _) in zip(typed, eng, strict=True))
        rows.append(
            "\t".join(
                str(x)
                for x in (
                    name,
                    sum(typed),
                    sum(t for t, h in zip(typed, test, strict=True) if not h),
                    sum(t for t, h in zip(typed, test, strict=True) if h),
                    sum(exact),
                    sum(e for e, h in zip(exact, test, strict=True) if not h),
                    sum(e for e, h in zip(exact, test, strict=True) if h),
                    gained,
                    lost,
                )
            )
        )
    (HERE / "bank.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
    head = ["case", "cell", "side", "excel", ENGINE, *SHORTLIST]
    lines = ["\t".join(head)]
    for i, cell in enumerate(cells):
        excel = cell.error if cell.error is not None else repr(cell.expected)
        values = [results[n][i][2] for n in [ENGINE, *SHORTLIST]]
        side = "test" if test[i] else "train"
        lines.append("\t".join([cell.case, cell.key, side, excel, *values]))
    (HERE / "predictions.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(rows))
    print(f"{len(cells)} cells", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
