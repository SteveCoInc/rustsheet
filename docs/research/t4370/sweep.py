"""T-4370's fresh sweep, generated after model.py was frozen: python3 sweep.py SEED COUNT SEPARATING CASE_PATH META_PATH.

Rounding ties of the shown fraction of a second (sec = (n + 0.5)/s, log-uniform from 1E-3 s), 3 of 4 at a serial below
1 and the rest past whole days, as T-4332's sweep. The first SEPARATING cells are ones where model.py and T-4332's chain
print different text. The fresh sweep is seed 43700, 120 cells, 60 separating.
"""
import importlib.util
import os
import sys

from model import chain_fields, fmt, model_fields

# T-4332's sweep generator (same formats, days and draw); loaded by path, since this file is also sweep.py.
_spec = importlib.util.spec_from_file_location(
    "t4332_sweep", os.path.join(os.path.dirname(os.path.abspath(__file__)), "../t4332/sweep.py"))
t4332_sweep = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4332_sweep)

t4332_sweep.generate(
    *(int(a) for a in sys.argv[1:4]),
    lambda f, s, x: [fmt(f, s, m(x, s)) for m in (chain_fields, model_fields)],
    "T-4370 fresh sweep (generated after the 15-digit fraction model of docs/research/t4370/model.py was "
    "frozen): rounding ties of the shown fraction of a second, mostly at serials below 1, under elapsed "
    "and clock formats with .0, .00 and .000; the first 60 cells separate that model from T-4332's chain.",
    *sys.argv[4:6])
