"""T-4402: record the engine's values for probes/t4402-pivotby-parent-set-holdout before the fill.

Run from the repo root once `make build` and gen_probe.py have run: python3 -I docs/research/t4402/predict_engine.py
writes docs/research/t4402/predictions.tsv with docs/research/t4344/predict_engine.py's `main`. The engine is the
model under test: T-4402's parent sets with T-4388's subtotal rule (PR #3430) merged in.
"""

from importlib import util

SPEC = util.spec_from_file_location("predict", "docs/research/t4344/predict_engine.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("docs/research/t4344/predict_engine.py is missing")
predict = util.module_from_spec(SPEC)
SPEC.loader.exec_module(predict)
predict.CASE = "eval/suites/probes/cases/t4402-pivotby-parent-set-holdout"
predict.OUT = "docs/research/t4402/predictions.tsv"

if __name__ == "__main__":
    predict.main()
