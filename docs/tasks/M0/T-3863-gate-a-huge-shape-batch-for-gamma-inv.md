---
id: T-3863
title: "Gate A huge-shape batch for GAMMA.INV, CHISQ.INV and CHISQ.INV.RT with the R3 points (uniform asymptotic incomplete gamma)"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A filled Gate A batch for GAMMA.INV, CHISQ.INV and CHISQ.INV.RT at huge shapes (1E7 to the parameters' limits), with the
decision 0188 R3 points of "GAMMA.INV at a huge shape, closed-form branch" marked and cited.

## Context
- Split from T-3578 (its second acceptance item), which landed BETA.INV s1 and the beta quantile's underflow bound.
- The s1 batches stop at a shape of 1E7 (df 2E7): the generator's series and continued fraction for the incomplete
  gamma need about 20·sqrt(a) terms, `gamma.py` gives no value above `SHAPE_CAP` (1E8), and mpmath's `gammainc` gives
  NoConvergence from a shape of about 1E5. The R3 region starts at shape 1E9 (closed form from about 2E10).
- Needs a uniform asymptotic expansion of the incomplete gamma (Temme; DiDonato and Morris 1986) in
  tools/src/rustsheet_tools/accuracy/defs/gamma.py at 50 digits, cross-checked against the series where both run.
- The R3 marks go in eval/accuracy/math-over-excel.tsv (protected: a decision record citing 0188 R3's huge-shape case);
  the engine's points are known failures of T-3364 until it lands. A new batch is `r3`-style new files (SPEC-0029:
  committed batches are immutable).
- GAMMA.INV/CHISQ roots below the smallest double keep their math value (gamma.quantile's docstring, T-3578): Excel is 0
  there and the envelope's absolute floor accepts it.

## Acceptance criteria
- [x] The asymptotic expansion, with tests against the series where both converge.
- [x] A huge-shape batch for the three gamma quantiles, filled from excel-graph, with the R3 points marked and cited.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (T-3364).

## Outcome
- **Expansion.** tools/src/rustsheet_tools/accuracy/defs/gamma.py now computes the tails above `TEMME_SHAPE_MIN` (1E7)
  with Temme's uniform asymptotic expansion (DLMF 8.12.3, 8.12.8). The c_k(eta) are power series built at the working
  precision plus guard digits: mu(eta) by Lagrange reversion, then Temme's recursion, with each constant fixed by
  cancelling the 1/eta pole, so no table is copied. `SHAPE_CAP` and its NoValueError are gone. A huge shape is summed
  with log10(a) extra digits, because a eta^2 / 2 needs dps digits after the point.
  - Tests (test_accuracy_gamma_beta.py) check c_0(0), c_0'(0), c_1(0) and c_2(0) against DLMF 8.12.11, and check
    agreement with the series and continued fraction to 40 digits at shapes 1E4 and 1E6 from z = -37 to 37.
  - Against mpmath's `gammainc` at 1E4 they agree to 48 digits. At 1E4 to 3E4 the expansion matched `gammainc`
    (90 digits) to all 50; the remaining residual is the series'.
  - Excel's true quantiles `GAMMA.INV(0.5,5E9,1)`, `GAMMA.INV(1E-30,7E9,1)` and `CHISQ.INV(1E-100,6E9)` are reproduced
    to 16 digits.
  - Below 1E7 nothing changed: `rustsheet-accuracy verify` shows 0 differences on every s1 batch of the three functions.
- **Batch.** The extreme ranges now reach the parameters' limits: alpha up to 1E300, df up to CHISQ.INV's 1E10 and
  CHISQ.INV.RT's 1E16. The batch `e-T-3863` (`--stratum extreme`; 200 + 120 + 120 points, 170 of them at shapes above
  1E7) was filled from excel-graph on 2026-11-05 with nothing pending.
- **R3.** On 131 points (103 GAMMA.INV, 28 CHISQ.INV.RT; shapes 3.9E9 to 7E293) Excel is the closed form, 2.66e-4 off
  the math. They are marked in math-over-excel.tsv under the existing case-3 entry (decision 0289), and are known
  failures of T-3364 because the engine follows Excel's closed form.
  - Every other numeric point is within 1e-12 of the math, apart from four tiny-shape GAMMA.INV points where Excel's 0
    is under the absolute floor.
  - The 26 CHISQ.INV.RT `#NUM!` cells are Excel's band and the engine matches them.
  - Before the marks, the only two engine failures were CHISQ.INV.RT A21 and A58, where the engine is 21 and 37 ulps
    off Excel's closed form (the right-tail drift that statistics.md already notes). Both are now R3.
- **Gate A:** 18472 points, 0 failing, 271 known failures (+131, T-3364). Eval delta: none (no engine change).
- **Follow-up:** T-4329 fills the probe suites' math sidecars at huge shapes.
