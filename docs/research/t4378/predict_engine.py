"""T-4378: record the engine's values for probes/t4378-pivotby-subtotal-holdout before the fill.

Run from the repo root after `make build` and gen_probe.py: python3 -I docs/research/t4378/predict_engine.py. It is
docs/research/t4344/predict_engine.py pointed at this case (predictions.tsv here).
"""

import importlib.util

_spec = importlib.util.spec_from_file_location(
    "t4344_predict_engine", "docs/research/t4344/predict_engine.py"
)
assert _spec and _spec.loader
engine = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(engine)

engine.CASE = "eval/suites/probes/cases/t4378-pivotby-subtotal-holdout"
engine.OUT = "docs/research/t4378/predictions.tsv"

if __name__ == "__main__":
    engine.main()
