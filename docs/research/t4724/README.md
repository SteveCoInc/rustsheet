# T-4724: VDB's unprobed straight-line-from-period-0 shapes

`gen.py` (seed 472420261107) builds probes/t4724-vdb-straight-from-zero and `predictions.tsv`. Both were committed
before the excel-graph fill. Each row gives H1, H2 and the engine's value before T-4724 (`engine_before`).

- **W** (30 rows): a whole start with a fractional end, in the regime where straight line wins at period 0. Rows are
  drawn where H1 and H2 differ.
  - H1: `share·sl + n·sl`.
  - H2: `(end - start)·sl`, with `sl = (c - s)/life`.
  - The engine matches H2 on 27 rows and neither on 3 (W A7, A15, A20).
- **F** (20 rows): factor 0 from a fractional start.
  - H1: T-4111's rule, `(end - start)·(c - s)/life`, which the engine follows.
  - H2: the fractional grid, `(end - start)·(c - s)/(life - f)`, where f is the start's fraction.

The fill decides each sheet. The engine follows whichever hypothesis Excel matches. If Excel matches neither, the
cells are recorded and nothing is tuned.

Run: `python3 -I docs/research/t4724/gen.py eval/suites/probes/cases/t4724-vdb-straight-from-zero.case.json docs/research/t4724/predictions.tsv`
(the `engine_before` column was added afterwards from `rustsheet-eval show`).
