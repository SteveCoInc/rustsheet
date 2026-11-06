---
id: T-3862
title: "BETA.INV is #NUM! in Excel at a huge alpha and a tiny p where the engine returns 1-2^-53"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
BETA.INV returns `#NUM!` where Excel does at a huge alpha (above about 1E+160) and a tiny p, instead of 0.9999999999999999.

## Context
- Gate A table accuracy/BETA.INV/s1 (T-3578, excel-graph 2026-11-02): 12 log-uniform points A180, A181, A187, A188,
  A190, A191, A192, A193, A194, A196, A197 and A198, for example `BETA.INV(1.67153110503488E-76,1.06481105633372E+183,
  1.82075780727763E-162)`. Excel is `#NUM!`; the math is exactly 1 (the true quantile rounds to 1) and the engine returns
  1 - 2^-53. They are known failures in eval/accuracy/known-failures.tsv owned by this task.
- statistics.md "BETA.INV at a huge shape is AS 109's Newton search (T-1538)" says the search converges near 1 at a
  huge first shape (shapes up to 5E9 and 1E10 there). These points have alpha from 5E+161 to 4E+296 and p from 1E-277
  to 8E-10: find where Excel's search fails (a density or ln B that overflows, a start that cannot move), probe the
  boundary in alpha and p with the excel-probe skill, and record the rule in statistics.md.
- Code: `fn_beta_inv` (crates/rustsheet-engine/src/functions/statistical/beta.rs, inverse_beta.rs).

## Acceptance criteria
- [ ] The boundary of Excel's `#NUM!` is probed and recorded in statistics.md.
- [ ] The 12 known-failure lines of accuracy/BETA.INV/s1 owned by T-3862 pass and are removed.
- [ ] No eval cell regresses; `make check` green; reviewers approve.

## Out of scope
- BETA.INV near 1 at a tiny second shape (T-3351) and the 0.5 at a DBL_MIN shape (T-2808).

## Probe predictions (committed before the fill)
Case probes/t3862-beta-inv-huge-alpha (row r: alpha = 1E+(109+r), r = 1..51, alpha 1E+110..1E+160). Hypotheses:
- P1 (a threshold in alpha alone): every BETA.INV column (A-F, J, K) turns from a number to `#NUM!` at the same row,
  with alpha between 1E+115 and 1E+145 (the Gate A cells bracket it at 5.18E+114 / 1.55E+145), whatever p and beta.
- P2 (the search's cdf fails): BETA.DIST's cdf at x = 0.5 (H, I) is `#NUM!` from the same row as BETA.INV.
- P3 (AS 109's flip): G, `BETA.INV(0.9,1E-100,alpha)`, is 1 minus column J's mirror and errors where the
  unflipped call does.
A refutation of P1 (a row that depends on p or beta) means the rule is not in alpha alone.
