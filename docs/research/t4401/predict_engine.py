"""T-4401: record the engine's values for probes/t4401-pivotby-unreached-subtotal-holdout before the fill.

Run from the repo root once `make build` and gen_probe.py have run: python3 -I docs/research/t4401/predict_engine.py
writes docs/research/t4401/predictions.tsv with docs/research/t4344/predict_engine.py's `main`. The engine is the
model under test: T-4388's rule (PR #3430), a column subtotal of a class no record reached holds every record of
its row line.
"""

from importlib import util

SPEC = util.spec_from_file_location("predict", "docs/research/t4344/predict_engine.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("docs/research/t4344/predict_engine.py is missing")
predict = util.module_from_spec(SPEC)
SPEC.loader.exec_module(predict)
predict.CASE = "eval/suites/probes/cases/t4401-pivotby-unreached-subtotal-holdout"
predict.OUT = "docs/research/t4401/predictions.tsv"

if __name__ == "__main__":
    predict.main()
