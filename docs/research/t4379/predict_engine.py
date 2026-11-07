"""T-4379: record the engine's values for probes/t4379-pivotby-repeated-unreached-leaf before the fill.

Run from the repo root after `make build` and gen_probe.py, before the case has an oracle sidecar:
python3 -I docs/research/t4379/predict_engine.py. It is docs/research/t4344/predict_engine.py pointed at this case;
the values (the engine before T-4379) go to docs/research/t4379/predictions.tsv as JSON strings.
"""

import importlib.util

_spec = importlib.util.spec_from_file_location("t4344_predict_engine", "docs/research/t4344/predict_engine.py")
assert _spec and _spec.loader
engine = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(engine)

engine.CASE = "eval/suites/probes/cases/t4379-pivotby-repeated-unreached-leaf"
engine.OUT = "docs/research/t4379/predictions.tsv"

if __name__ == "__main__":
    engine.main()
