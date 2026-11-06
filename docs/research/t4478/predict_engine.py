"""T-4478: record the engine's values for probes/t4478-reduce-scan-reference-accumulator before the fill.

Run from the repo root once `make build` and gen_probe.py have run: python3 -I docs/research/t4478/predict_engine.py
writes docs/research/t4478/predictions.tsv with docs/research/t4344/predict_engine.py's `run`. The area is every
formula cell plus the three rows below each bare formula in J, L, N and P, so their spill extent is predicted too.
"""

import importlib.util
import sys

BARE_AREA = [f"{column}{row}" for column in "JLNP" for row in range(1, 5)]

if __name__ == "__main__":
    loader = importlib.util.spec_from_file_location("t4344_predict", "docs/research/t4344/predict_engine.py")
    if loader is None or loader.loader is None:
        sys.exit("docs/research/t4344/predict_engine.py is missing")
    t4344 = importlib.util.module_from_spec(loader)
    loader.loader.exec_module(t4344)
    t4344.AREA = lambda calc: sorted({c for c, f in calc.items() if str(f).startswith("=")} | set(BARE_AREA))
    t4344.run("eval/suites/probes/cases/t4478-reduce-scan-reference-accumulator", "docs/research/t4478/predictions.tsv")
