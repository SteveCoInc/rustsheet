# T-0975: normal density kernel

- `gen_fresh.py OUT.case.json` writes the post-freeze fresh sweep, probes-numeric `t0975-normpdf-fresh`.
- `collect.py` gathers every NORM.S.DIST / PHI / NORM.DIST / NORMDIST / LOGNORM.DIST density cell with literal
  arguments and an oracle value from `eval/suites/*/cases` into `cells.json` (in the current directory).
- `guards.py` (with `lib.py`, an mpmath port of Excel's EXP model V6 from `docs/research/t1778/round8/exp_v6.py`)
  scores the chosen kernel `EXP(-d*d/2) / s * K` and the correctly rounded density on the train / hold-out split
  (sha256 of case and cell, 30% hold-out) and the fresh sweep, and lists fresh-sweep points outside the Gate A
  envelope.

Run from a scratch directory: `python3 collect.py && tools/.venv/bin/python guards.py` (mpmath from the tools venv).
Results are in docs/excel-semantics/statistics.md, "Normal density is Excel's EXP times 1/sqrt(2*pi), then / sigma".
