# 0281 — T-3252: fitted lock 2 to 3 in incomplete_gamma/mod.rs for NEAR_MODE_BELOW

- **Status:** accepted (2026-11-04, T-3252). It applies decision 0188 R4 and adds no rule.
- **Context:** T-3252 makes the large-shape gamma prefactor (`gamma_prefactor` in
  crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs) take Excel's ratio form below the mode
  from (a - x) > 0.33a, instead of TOMS 654's 0.4a (docs/excel-semantics/statistics.md, the T-3252 paragraph of
  "Incomplete gamma: accurate kernel"). The bound `NEAR_MODE_BELOW` was placed between evidenced cells, so it is a fit
  (decision 0188 R4):
  - It carries `FITTED(T-3252, 102 cells, x/a 0.55..0.99, shape 30..500)` and applies only to the `Exponent::Excel`
    form (GAMMA.DIST, CHISQ.DIST and their legacy names).
  - Training was probes/t3252-gamma-prefactor-window. A first freeze at 0.3 failed its fresh hold-out,
    probes/t3252-gamma-prefactor-holdout, which was written after that freeze. At 0.68a the ratio form is 86 to 208
    ulp from Excel, against 43 for `rcomp`.
  - The bound was re-frozen at 0.33 before probes/t3252-gamma-prefactor-holdout-2 was written. On that sweep every
    cell the bound gives the ratio form is Excel's to 0 to 2 ulp (x <= 0.664a, shapes 40, 70 and 300) or a tie
    (0.668a: -55 against 56).
- **Decision:** quality/fitted.json rises by one for incomplete_gamma/mod.rs (2 to 3), for this one new fit, written
  by `scripts/check_fitted.py --update`.
- **Consequences:** one counted tag. The fitted ratchet stays one-way for new fits, each with its own record. No gated
  or ungated cell is lost (`rustsheet-eval check`: 0 regressions in every suite). In the window case, 74 of 188 cells
  now pass, against 69 before.
