# 0277 — T-3409: fitted lock 10 to 8 in huge_shape.rs, one search model for three band cuts

- **Status:** accepted (2026-11-04, T-3409). It applies decisions 0010 (a ratchet moves down in the PR that pays it
  down) and 0188 R4 (the one new fit), and adds no rule.
- **Context:** T-3409 models Excel's gamma inverse search at a large shape as Newton's method on the odds P/Q from
  Excel's start, failing past a step limit (`search_converges` in crates/rustsheet-engine/src/functions/statistical/
  incomplete_gamma/huge_shape.rs; docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV at a huge shape";
  docs/research/t3409/README.md). That model removes three fitted constants, `BAND_UPPER_SLOPE`, `BAND_UPPER_EDGE` and
  `BAND_LOWER_EDGE`, and adds one, `SEARCH_STEP_LIMIT` = 207 (with its tolerance), tagged `FITTED(T-3409, 1087 cells,
  ...)`. T-3070's fitted 1E9 cut keeps its tag as `START_SHAPE_MIN` (the start rule only), and `HUGE_SHAPE_MIN` is
  now the Temme quantile shape 1E5, not a fit. Its R4 results (train, held-out, and the fresh
  sweep probes-numeric/t3409-r4-gamma-inv-search-sweep drawn after the freeze) are in T-3409's Outcome.
- **Decision:** quality/fitted.json falls for huge_shape.rs from 10 to 8 (the lock from 91 to 89), written by
  `scripts/check_fitted.py --update`.
- **Consequences:** none beyond the counted tags. The PR loses no gated cell (eval delta in T-3409's Outcome).
