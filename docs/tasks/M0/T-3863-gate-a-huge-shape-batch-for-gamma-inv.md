---
id: T-3863
title: "Gate A huge-shape batch for GAMMA.INV, CHISQ.INV and CHISQ.INV.RT with the R3 points (uniform asymptotic incomplete gamma)"
milestone: M0
lane: harness
status: todo
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
- [ ] The asymptotic expansion, with tests against the series where both converge.
- [ ] A huge-shape batch for the three gamma quantiles, filled from excel-graph, with the R3 points marked and cited.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (T-3364).
