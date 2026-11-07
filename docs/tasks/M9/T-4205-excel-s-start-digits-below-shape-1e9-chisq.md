---
id: T-4205
title: "Excel's start digits below shape 1E9: CHISQ.INV.RT(1E-300,df) for df 5E6 to 1E9 within 15 digits"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`CHISQ.INV.RT(1E-300,df)` for df 5E6 to 1E9 (probes/t3350-chisq-inv-huge-df A205, A217, A229, A241, A253, A265, A277, A289) is Excel's value within 15 significant digits.

## Context
Excel's value at these cells is its Wilson-Hilferty start x0, where the density at x0 underflows (T-3409 found
the rule holds on all 197 probed start cells from shape 1E5), 2.6E-4 above the true quantile. From shape 1E9 that
branch follows the math (decision 0188 R3 case 3, T-3364, decision 0337), and the engine returns the true quantile
wherever the start underflows (`huge_shape_quantile`, `huge_shape.rs`). Below 1E9 the owner ruled (2026-10-31, T-3409)
that Excel's start is the target, not the math. The start's higher terms (`WILSON_HILFERTY_SPREAD`,
`WILSON_HILFERTY_INVERSE_SHAPE`, `WILSON_HILFERTY_SQUARE`, `WILSON_HILFERTY_CUBE`) were fitted (T-3070) on closed-form
cells from shape 1.5E10: at shape 2.5E6 to 5E8 the start is 1E-9 to 3.4E-9 relative below Excel (A205: start
5119417.184081892, Excel 5119417.201267746), outside Gate A's envelope, so the true quantile stays the nearer value
for now. Moved here from T-3409's acceptance criteria. Find Excel's start formula (its 1/a and z terms) at these
shapes, or decide by decision record that these 8 cells stay off. Any start rule found applies only below shape 1E9.
From 1E9 the math stays (R3), so do not put Excel's start back there. docs/excel-semantics/statistics.md
"GAMMA.INV and CHISQ.INV at a huge shape". A fit owes decision 0188 R4's hold-out and fresh sweep.
The fresh sweep of T-3409 adds the same kind of cell below 1E9 (the start, right type, digits off): probes-numeric/
t3409-r4-gamma-inv-search-sweep Calc!A231, Calc!A262:A263, Calc!A266, Calc!A268, Calc!A271, Calc!A276:A279
(`CHISQ.INV.RT` at p from 1E-295 to 1E-251, df 3.7E7 to 9.7E8), and probes-numeric/
t3409-r4-gamma-inv-search-sweep-wide Calc!A63 `CHISQ.INV.RT(7.12382795097409E-296,18338000)`. Two sweep cells sit at
shape 1E9 or more and are R3's now, not this task's: t3409-r4-gamma-inv-search-sweep Calc!A226 (shape 4.8E9) and
t3409-r4-gamma-inv-search-sweep-wide Calc!A114 (shape 1.365E10).

## Acceptance criteria
- [ ] A205, A217, A229, A241, A253, A265, A277, A289 of probes/t3350-chisq-inv-huge-df and the sweep cells below 1E9
      above are within 15 digits of Excel (or a decision record keeps them off).
- [ ] From shape 1E9 the engine still returns the math on the closed-form branch (the R3 marks of
      eval/accuracy/math-over-excel.tsv keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
