# T-4465: rule frozen before the hold-out fill (decision 0188 R4)

Scored on the fixed cases only (`probes/t4465-extent-near-integer`, 760 cells, and
`-randarray`, 95 cells; excel-graph 2026-11-06). The hold-out cases
(`-holdout`, `-holdout-randarray`, offsets k*2^-30 from `random.Random(4465)`) were generated
with the fixed cases at 6555ba068 and are filled only after this note is committed.

Misses per candidate reading rule (`predict.py`; `predictions.tsv`):

| constructor | shared (`near_integer_whole`) | eps23 (`EXTENT_EPSILON`) | f32 | floor | trunc | engine before |
|---|---|---|---|---|---|---|
| SEQUENCE rows | 0 | 21 | 19 | 30 | 33 | 21 |
| SEQUENCE columns | 0 | 16 | 17 | 23 | 26 | 16 |
| RANDARRAY | 0 | 21 | 19 | 30 | 33 | 30 |
| MAKEARRAY | 70 | 65 | 63 | 63 | 63 | 0 |
| TAKE | 48 | 30 | 43 | 25 | 0 | 0 |
| DROP | 48 | 30 | 43 | 25 | 0 | 0 |
| EXPAND | 28 | 8 | 19 | 0 | 0 | 0 |
| WRAPROWS | 14 | 4 | 1 | 0 | 0 | 0 |
| WRAPCOLS | 14 | 4 | 1 | 0 | 0 | 0 |

Frozen rule:
- SEQUENCE's rows and columns and RANDARRAY's rows read `near_integer_whole` (the DATE/TIME/
  XMATCH rule: floor, up-snap within 2^-22 + 2^-33, half that below 0). `EXTENT_EPSILON` goes.
- MAKEARRAY refuses any fractional rows or columns with `#VALUE!` (the engine's rule, unchanged).
- TAKE, DROP, EXPAND, WRAPROWS and WRAPCOLS truncate toward zero with no snap (the engine's
  rules, unchanged).

Correction made before scoring: `predict.py` first read the base-0 cells `-2^-k` as -(2^-k);
Excel's unary minus binds before `^`, so they are (-2)^-k, positive for even k (the engine
parses them the same way). The candidate rules are unchanged; only the argument values of those
cells were re-derived.
