---
id: T-4523
title: "YIELD is #NUM! in Excel where the engine's Newton converges, at a price and redemption far below par (Gate A YIELD/s1 A225)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine's YIELD is `#NUM!` where Excel's is on accuracy/YIELD/s1 A225, or the Outcome records, with probe evidence,
which part of Excel's solver (its start or its step test) leaves PRICE's domain there and why the engine does not
follow.

## Context
- Found by T-4113 (Gate A YIELD table, excel-graph, 2026-11-06): accuracy/YIELD/s1 A225
  `YIELD(1511252,1600178,0.693266194144484,0.0113829835620059,0.013387175518858,2,1)` (243 years semiannual, basis 1)
  is `#NUM!` in Excel; the engine returns 37.03781989946988, which is PRICE's root (the 50-digit math value). It is the
  table's only YIELD error mismatch and is listed in eval/accuracy/known-failures.tsv under this task.
- The point is far outside the range the engine's start was fitted on (`yield_start`, T-1592: rate 0..0.5, price 40..160,
  redemption 50..200): the price and the redemption are about 0.01, while the coupon (100·rate/f = 34.7 a period) is
  thousands of times the redemption. The fitted start takes 100 as par (`100·rate·T + (100 - pr)`), so at a price of
  0.01 it starts near the root of a par bond, not of this one. financial.md "YIELD's #NUM! cells and its 100-step cap"
  records Excel's `#NUM!` where its Newton leaves PRICE's domain; which start Excel takes at a redemption far from 100 is
  unprobed.
- Code: `yield_start` and `excel_yield_newton` in
  crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs.

## Acceptance criteria
- [ ] A probe (excel-graph) of YIELD at prices and redemptions from 1E-3 to 100 (rates 0.05 to 1, a few terms and
      bases) shows where Excel turns `#NUM!`; the finding is a financial.md entry.
- [ ] accuracy/YIELD/s1 A225 passes (its known-failures line removed), or the Outcome says why it cannot yet.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's last ulps at the root (T-2788).
