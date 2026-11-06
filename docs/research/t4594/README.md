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
- The second hold-out matched 199 of 252 cells. Its misses (integer sides `0#`, `#0`, `??0`, `?0?`) led
  to the second refit, *pending zeros*: a `0` or `#` before the point leaves one plus one per placeholder
  before it, except a `?` that printed a lacking zero; a `?` that then prints a zero for nothing due takes
  one; whatever is left starts the decimals. That refit matches all 252.
- It then froze, and `holdout-3-predictions.json` holds the engine's output for every cell of
  `t4594-fraction-tail-point-holdout-3`, written before that probe was filled: 40 triples drawn with
  `random.seed(45943)` over codes 64, 5, 2000 and 12, seventeen integer shapes (twelve of them new, with
  literals among them) and seven new values.
- The third hold-out matched 271 of 280 cells. Its misses (`#?#` and `? ?0`) led to the third refit: a `?`
  that prints a zero for nothing due is not counted toward a later `0` or `#`. That refit matches all 280.
- It then froze, and `holdout-4-predictions.json` holds the engine's output for every cell of
  `t4594-fraction-tail-point-holdout-4`, written before that probe was filled: 48 triples drawn with
  `random.seed(45944)` over codes 128, 6, 4000 and 25, every 1-3 wide integer shape and 1-4 wide decimal
  shape (plus some with literals), and seven new values.
