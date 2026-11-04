# 0281 — T-3252: fitted lock 2 to 3 in incomplete_gamma/mod.rs for NEAR_MODE_BELOW

- **Status:** accepted (2026-11-04, T-3252). It applies decision 0188 R4 and adds no rule.
- **Context:** T-3252 makes the large-shape gamma prefactor (`gamma_prefactor` in
  crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs) take Excel's ratio form below the mode
  from (a - x) > 0.334a at shapes 30 to 500, instead of TOMS 654's 0.4a (docs/excel-semantics/statistics.md, the T-3252 paragraph of
  "Incomplete gamma: accurate kernel"). The bound `NEAR_MODE_BELOW` was placed between evidenced cells, so it is a fit
  (decision 0188 R4):
  - It carries `FITTED(T-3252, 266 cells, x/a 0.55..0.99, shape 30..500)` and applies only to the `Exponent::Excel`
    form (GAMMA.DIST, CHISQ.DIST and their legacy names) at shapes 30 to 500 (`NEAR_MODE_BELOW_SHAPES`).
  - Training was probes/t3252-gamma-prefactor-window. A first freeze at 0.3 failed its fresh hold-out,
    probes/t3252-gamma-prefactor-holdout, which was written after that freeze. At 0.68a the ratio form is 86 to 208
    ulp from Excel, against 43 for `rcomp`.
  - A second freeze at 0.33 fell outside probes/t3252-gamma-prefactor-holdout-2's bracket (0.332, 0.336), so both
    hold-outs became training data. 0.334, the bracket's middle, was frozen before the fresh sweep
    probes/t3252-gamma-prefactor-sweep (40 log-uniform points, shapes 30 to 500, x/a 0.55 to 0.75, density and cdf).
    On it the fit scores 22 / 61 / 69 of 80 (bit-exact / 4 ulp / 15 digits), against 3 / 15 / 35 for correctly
    rounded; T-3252's Outcome has the full table.
- **Decision:** quality/fitted.json rises by one for incomplete_gamma/mod.rs (2 to 3), for this one new fit, written
  by `scripts/check_fitted.py --update`.
- **Consequences:** one counted tag. The fitted ratchet stays one-way for new fits, each with its own record. No gated
  or ungated cell is lost (`rustsheet-eval check`: 0 regressions in every suite). In the window case, 74 of 188 cells
  now pass, against 69 before.
