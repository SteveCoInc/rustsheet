---
id: T-4285
title: "Owner ruling: NPER at rates 1E-14 to 1E-13, where Excel is still 7E-4 to 7E-3 off the math (R3 candidates outside the ruled region)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The owner rules whether decision 0188 R3's NPER region (0 < rate < 1E-14) extends to the rates up to 1E-13 where
Excel is still more than G = 1e-4 off the math; the engine then follows the ruling.


## Context
- Found by T-3367 (probes-numeric/t3367-nper-tiny-rate, excel-graph 2026-11-05, 50-digit math sidecar). Excel's
  relative error against the math, per rate (seven argument sets each, six numeric):
  - inside the ruled region: up to 4.7E-1 at 1.2E-16, 9.1E-3 at 9.9E-15; the engine returns the math there (T-3367);
  - outside it, still above G: 1E-14 (A99 to A104, 7.0E-4 to 1.3E-3), 1.5E-14 (A106 to A111, 6.5E-3 to 6.9E-3),
    3E-14 (A113 to A118, 7.8E-4 to 8.7E-4) and 1E-13 (A120 to A125, 7.8E-4 to 8.2E-4);
  - below G: 3E-13 (A127 to A132, 5.8E-5 to 8.7E-5) and 1E-12 (A134 to A139, 8.9E-5 to 9.1E-5).
- Cause: Excel takes ln of the doubles 1 + rate and ratio, each rounded to about 1.1E-16 absolute, so the relative
  error is about 1E-16/rate; it stays above G up to about 1E-12.
- Today the engine matches Excel to the bit at these 24 cells (`NPER_MATH_RATE` in
  crates/rustsheet-engine/src/functions/financial/annuity.rs), so they pass on Excel's side.
- Decision 0188 R3: a new case needs a decision record carrying an owner ruling. This task is that question; an agent
  must not widen the region on its own.

## Acceptance criteria
- [x] The owner's ruling is recorded in a decision record (widen the region to a stated bound, or keep it).
- [x] If widened: `NPER_MATH_RATE` moves to the ruled bound, the cells in the new region are R3-marked in
      eval/accuracy/math-over-excel.tsv, and financial.md's "NPER at a tiny positive rate" entry states the new region.
- [x] `make check` green; reviewers approve.

## Out of scope
- Negative tiny rates (unprobed).

## Outcome
- Owner ruling 2026-11-06, decision 0300: keep matching Excel. This is not an R3 case: no `MATH-OVER-EXCEL`
  entry, no R3 mark, no engine change (the engine already matches Excel here).
- Eval delta: none.
