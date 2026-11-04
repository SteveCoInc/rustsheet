# 0259 — T-3930: fitted lock 3 to 4 in complex/trig.rs for LIBM_COSH_BELOW

- **Status:** accepted (2026-11-03, T-3930). It applies decision 0188 R4.3 and adds no rule.
- **Context:** T-3930 makes the IM hyperbolic kernels (`inverse_exp_sinh_cosh` in
  crates/rustsheet-engine/src/functions/engineering/complex/trig.rs) take libm's cosh below |x| = 1E-4 instead of
  the `1/u` form's (docs/excel-semantics/engineering.md, "IMSINH, IMCOSH, IMSECH and IMCSCH take libm's cosh below
  |x| 1E-4"). The cut `LIBM_COSH_BELOW` was placed between the evidenced cells, so it is a fit (decision 0188 R4):
  - It carries `FITTED(T-3930, 39 cells, |x| 3.6E-06..0.01; cut bracketed 8.8E-05..1.09E-04)` and applies only
    inside `INVERSE_EXP_LOW`.
  - Its hold-out on the fit lab's split is in T-3930's Outcome. On im-small-x-complex the cut scores train 1131,
    test 485 against correctly rounded 1052, 448, and it is accepted.
  - The fresh sweep probes-numeric/t3930-r4-im-small-x-cosh was drawn after the freeze at edb75c46a. It scores
    cut 480 of 480, form only 475, libm only 476.
- **Decision:** quality/fitted.json rises by one for complex/trig.rs (3 to 4), for this one new fit, written by
  `scripts/check_fitted.py --update`.
- **Consequences:** one counted tag. The fitted ratchet stays one-way for new fits, each with its own record. No
  gated or ungated cell is lost: the cut fixes 6 texts and breaks none across the 130 cases with IMSINH, IMCOSH,
  IMSECH or IMCSCH.
