"""T-4464: record the engine's values for probes/t4464-groupby-spent-filter-shapes before the fill.

Run from the repo root after `make build` and gen_probe.py, before the case has an oracle sidecar:
python3 -I docs/research/t4464/predict_engine.py writes docs/research/t4464/predictions.tsv with
docs/research/t4344/predict_engine.py's `main`, on sheet G and over each call's whole 3x3 spill area (A<r>:C<r+2>),
since these calls spill; a cell the engine leaves empty is recorded as `<empty>`.
"""

from importlib import util

SPEC = util.spec_from_file_location("predict", "docs/research/t4344/predict_engine.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("docs/research/t4344/predict_engine.py is missing")
predict = util.module_from_spec(SPEC)
SPEC.loader.exec_module(predict)


def spill_areas(cells: dict[str, object]) -> list[str]:
    """Every cell of A<r>:C<r+2> for each formula at A<r>."""
    rows = [int(a[1:]) for a, f in cells.items() if a.startswith("A") and str(f).startswith("=")]
    return [f"{c}{r + d}" for r in rows for d in range(3) for c in "ABC"]


predict.CASE = "eval/suites/probes/cases/t4464-groupby-spent-filter-shapes"
predict.OUT = "docs/research/t4464/predictions.tsv"
predict.SHEET = "G"
predict.AREA = spill_areas

if __name__ == "__main__":
    predict.main()
