# 0257 — T-3812: fitted lock 0 to 1 in bfrac.rs for LENTZ_EPS, and the ungated cells the Lentz model loses

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
  T-3812 also accepts the ungated cells the better model loses (decision 0082's precedent; directive 2 for the removed
  test pins):
  - probes-numeric t2464-w2f-xruns-1, (100, 120) run (`gate = false`): 11 cdf cells that TOMS's recurrence matched bit
    for bit are now 1 to 7 ulp off (865 region cells, bit-exact 43 -> 32, within 4 ulp 325 -> 351, within half a unit
    of the 15th digit 480 -> 496). They were cancellations against the engine's density (T-3244), not Excel's
    arithmetic: at A14913 and A15779 the density cells A14914 and A15780 are 5 ulp off Excel's, TOMS was exact, the
    Lentz loop is 7 and 5 ulp off, and the same model on Excel's density is 1 and 4 ulp off.
  - bratio/tests.rs `bfrac_matches_excel` no longer pins A14913 and A15779 (named in its comment with the cause);
    `bfrac_lentz_matches_excel` pins four Lentz-region cells where the engine's density equals Excel's, each off before.
  - On the t3571 probes the same change gains 19, 57 and 21 bit-exact cdf cells (tune, hold, fresh).
- **Consequences:** one counted tag. The fitted ratchet stays one-way for new fits, each with its own record. The PR
  loses no gated cell; the ungated losses above are accepted, and they should return once the engine's density is
  Excel's (T-3244).
