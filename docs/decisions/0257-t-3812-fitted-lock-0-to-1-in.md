# 0257 — T-3812: fitted lock 0 to 1 in bfrac.rs for LENTZ_EPS

- **Status:** accepted (2026-11-03, T-3812). It applies decision 0188 R4.3 and adds no rule.
- **Context:** T-3812 ports T-3571's frozen model of Excel's bfrac (Numerical Recipes' modified Lentz loop on
  closed-form terms) into `lentz_fraction` in crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs
  (docs/excel-semantics/statistics.md, "Incomplete beta bfrac is TOMS's fraction in a Lentz loop"). Its stop
  `LENTZ_EPS` = 10·DBL_EPSILON was chosen by scoring stop thresholds on the tune probe, so it is a fit (decision 0188
  R4):
  - it carries `FITTED(T-3571, 1536 cells, a, b 40..5000, unswapped call (x below the mean), z -8..-0.2 sd)` and
    applies only in that range (`lentz_region`); TOMS's recurrence is kept everywhere else;
  - its hold-out plan (committed before scoring), the fit lab's hash split (test 75.2% against train 75.7%), the fresh
    sweep built after the freeze and the EPS bracket sensitivity are in docs/research/t2464/r5-bfrac/RESULTS.md; the
    Gate A envelope recheck with the engine's density (1,200 of 1,200) is in T-3812's Outcome.
  - TINY = 1e-30 is Numerical Recipes' FPMIN, copied, not a fit; the loop and term forms are structural choices.
- **Decision:** quality/fitted.json rises by one for bfrac.rs (0 to 1), for this one new fit, written by
  `scripts/check_fitted.py --update`. The same run records gamma.rs at 6, the count of tags that file already has on
  main (the lock said 7); the total stays at the lock.
- **Consequences:** none beyond the one counted tag. The fitted ratchet stays one-way for new fits, each with its own
  record. The PR loses no gated cell (eval delta in T-3812's Outcome).
