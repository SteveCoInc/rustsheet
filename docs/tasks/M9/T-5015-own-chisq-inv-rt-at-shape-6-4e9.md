---
id: T-5015
title: "Mark the t3409 sweeps' closed-form-branch cells at shape 1E9 or more as R3 points (Calc!A234, Calc!A226 and their neighbours, decision 0337)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every closed-form-branch cell of probes-numeric/t3409-r4-gamma-inv-search-sweep and t3409-r4-gamma-inv-search-sweep-wide
at shape 1E9 or more that passes decision 0337's per-cell check is marked in eval/accuracy/math-over-excel.tsv, so
`make exit-owners MILESTONE=M9` no longer lists t3409-r4-gamma-inv-search-sweep Calc!A234.

## Context
The 2026-11-07 evening steward run (`make eval` on 04f97f7d, whose tree is c709e9d2's) found `make exit-owners
MILESTONE=M9` listing t3409-r4-gamma-inv-search-sweep Calc!A234 `CHISQ.INV.RT(1.55608746068665E-92,12890000000)`
under the CHISQ.INV.RT rate with no open M9 owner. Excel gives 12896697822.579903, ours 12893270849.57774, which equals
the 50-digit math sidecar: Excel/ours is 1.000266, the closed-form branch's signature.

This is not a new R3 case and needs no owner ruling. Its shape (df/2 = 6.445E9) is inside the committed R3 case of
decision 0337 ("From shape 1E9 this is R3") and the statistics.md entry "GAMMA.INV at a huge shape, closed-form branch"
(Status: MATH-OVER-EXCEL), which covers CHISQ.INV.RT too. T-4205 hands t3409-r4-gamma-inv-search-sweep Calc!A226 (shape
4.8E9) and t3409-r4-gamma-inv-search-sweep-wide Calc!A114 (shape 1.365E10) to R3 for the same reason. The cells only
lack their marks: eval/accuracy/math-over-excel.tsv has no line for either sweep.

Never fit Excel's start back into this region: T-4205 says "From 1E9 the math stays (R3), so do not put Excel's start
back there", and decision 0337 settles it.

## Acceptance criteria
- [ ] Calc!A234, Calc!A226, sweep-wide Calc!A114 and every other cell of both sweeps at shape 1E9 or more on the
  closed-form branch carry a math-over-excel.tsv mark pointing at the statistics.md closed-form-branch entry, each
  passing decision 0337's per-cell check: Excel off the committed math by more than 1e-4 relative, ours within 1e-13
  of it, and the cell on the underflow branch. A cell that fails the check is listed in the Outcome with its numbers
  instead.
- [ ] `make exit-owners MILESTONE=M9` no longer lists Calc!A234.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes, and cells below shape 1E9 (T-4205, T-4206).
