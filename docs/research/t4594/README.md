# T-4594: placeholders after a fraction with a `.` among them

- `../../../eval/suites/probes/cases/t4594-fraction-tail-point-explore.case.json` (320 cells, excel-graph
  2026-11-06) is the exploration grid the model was fitted to, with probes/t4515-fresh-sweep rows 1-8 and
  probes/t4515-fraction-tail-holdout row 6.
- The model then froze (commit of this file) and `holdout-predictions.json` holds the engine's output for
  every cell of `t4594-fraction-tail-point-holdout`, written before that probe was filled. The hold-out
  draws 30 of 280 (code, integer shape, decimal shape) triples with `random.seed(4594)`, over codes 32,
  50, 100 and 8 and seven values, none of them in the exploration grid.
- The first hold-out matched 192 of 210 cells. Its misses (three-wide decimals after an integer side `?0`)
  led to a second exploration grid, `t4594-fraction-tail-point-explore-wide` (648 cells: every three-wide
  decimal shape after `0`, `?0`, `?` and `??`), and one refit rule: an integer side ending in `0` or `#`
  after a `?` that printed a spilled digit or a space starts the decimals with two zeros, not one.
- The refit then froze, and `holdout-2-predictions.json` holds the engine's output for every cell of
  `t4594-fraction-tail-point-holdout-2`, written before that probe was filled: 36 (code, integer shape,
  decimal shape) triples drawn with `random.seed(45942)` over codes 16, 9, 250 and 3, twelve integer
  shapes (six of them new) and seven new values.
