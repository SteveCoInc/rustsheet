---
id: T-4321
title: "LOGNORM.DIST density: find Excel's kernel (up to 34 ulp off the true density)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
LOGNORM.DIST(x, m, s, FALSE) reproduces Excel's digits, including the cells that are visibly off today.

## Context
Split from T-0975, which found the normal density kernel (`special::normal_pdf`, `EXP(-d*d/2) / s * K` with Excel's
EXP and K = RN(1/sqrt(2*pi)); docs/excel-semantics/statistics.md "Normal density is Excel's EXP times 1/sqrt(2*pi),
then / sigma (T-0975)") but not LOGNORM.DIST's. LOGNORM.DIST is a different kernel:
- `LOGNORM.DIST(1,0,1,FALSE)` is 0.39894228040143265 (probes-numeric m9-normpdf-lognorm-dist A4), one ulp under K, so
  at d = 0 it is not `EXP(0) * K`; it is what `1 / 2.5066282746310007` or `EXP(-0.9189385332046728)` gives.
- Against 80-digit math Excel's LOGNORM.DIST density is off by -13 to +10 ulp on the 120-cell m9-normpdf-lognorm-dist
  grid, the error growing with x (e.g. A73 `LOGNORM.DIST(8.25,1,0.5,FALSE)` +10, A37 `(9.25,0,1)` -11), and by 34 ulp
  at `LOGNORM.DIST(40,0,0.5,FALSE)` (gen-m9/b4173fe25109a238 Calc!A48: Excel 3.0218137820578166E-14, ours
  3.021813782057842E-14, visibly off). The error is larger than any single rounding of the exponent, so Excel's
  exponent carries more than one rounding of a quantity of the size of ln(x)^2 or larger (unverified).
- T-0975 scored about 60 forms (scratch scripts, not kept; `docs/research/t0975/collect.py` gathers the cells)
  against the 121 literal-argument density cells, none above 46: `exp(-d*d/2) / (x*sqrt(2 pi)*s)` in every association, with libm exp or Excel's
  EXP; `EXP(-d*d/2 - ln(x*s) - ln sqrt(2 pi))` and its sums in either order, with the stored or the extended (64-bit)
  intermediates; `NORM.DIST(ln x, m, s, FALSE) / x`; ln(x) as log10(x)*ln 10, log2(x)*ln 2 and log10(x)/log10(e).
  Untried: Excel's LN (`excel_ln`) in place of libm's, `(ln x - m)^2` through Excel's `^` (POWER), a sigma^2 form
  `exp(-(ln x - m)^2 / (2 s^2))` with POWER, and x^(-1) through POWER.
- Code: `functions/statistical/log_normal.rs` (the density branch keeps `exp / (x * SQRT_2PI * s)`). T-0975's design
  review: move `FRAC_1_SQRT_2PI` from incomplete_beta.rs into special.rs beside `SQRT_2PI`, the normal kernels' module.
- Owned cells: the 5 `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` `LOGNORM.DIST LOGNORM.INV` lines (E3, E5, E35, E46,
  E63) and the 4 gen-m9 LOGNORM.DIST lines (b4173fe25109a238 A48, visibly off; 3a25a03eb94fcfbd A220,
  85f48d5cdfb24555 A310, caa1eed48b1883fe A250) that T-0975 owned, plus the m9-normpdf-lognorm-dist probe cells.

Owned gen-m9 cells (moved from T-0975): 4 known-failure lines in eval/baseline/gen-m9.txt, 1 visibly off.
Cells:
- gen-m9/b4173fe25109a238.case.json (1): Calc!A48
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A220
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A310
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A250

## Acceptance criteria
- [ ] Visible parity: gen-m9/b4173fe25109a238 Calc!A48 is within half a unit of the 15th significant digit of Excel's
      value, and its baseline line leaves eval/baseline/gen-m9.txt.
- [ ] One LOGNORM.DIST density formulation fits every cached cell bit for bit, with the decision 0188 R4 guards
      (hold-out, fresh sweep) in the Outcome; otherwise the cells it misses are listed in a bit-level follow-up.
- [ ] docs/excel-semantics/statistics.md records the kernel (or the refuted candidates).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cumulative LOGNORM.DIST (T-0968) and LOGNORM.INV.
