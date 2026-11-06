"""T-4402: record the engine's values for probes/t4402-pivotby-parent-sets-holdout before the fill.

Run from the repo root once `make build` and gen_probe.py have run: python3 -I docs/research/t4402/predict_engine.py
writes docs/research/t4402/predictions.tsv with docs/research/t4344/predict_engine.py's `run`.
"""

import importlib.util
import sys

if __name__ == "__main__":
    loader = importlib.util.spec_from_file_location("t4344_predict", "docs/research/t4344/predict_engine.py")
    if loader is None or loader.loader is None:
        sys.exit("docs/research/t4344/predict_engine.py is missing")
    t4344 = importlib.util.module_from_spec(loader)
    loader.loader.exec_module(t4344)
    t4344.run("eval/suites/probes/cases/t4402-pivotby-parent-sets-holdout", "docs/research/t4402/predictions.tsv")
