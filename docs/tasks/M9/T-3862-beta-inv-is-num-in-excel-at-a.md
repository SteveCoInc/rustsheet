---
id: T-3862
title: "BETA.INV is #NUM! in Excel at a huge alpha and a tiny p where the engine returns 1-2^-53"
milestone: M9
lane: functions-stat
status: todo
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
  1.82075780727763E-162)`. Excel is `#NUM!`; the engine and the math are 1 - 2^-53 (the true quantile rounds to just
  below 1). They are known failures in eval/accuracy/known-failures.tsv owned by this task.
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
