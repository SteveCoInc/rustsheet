---
id: T-4883
title: "F quantiles: the engine misses Excel where Excel is on the math (tiny p at df1 1, deep upper tails at df2 near 200, FINV near p = 1)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
F.INV, F.INV.RT and FINV return Excel's value on the T-4857 cells where Excel agrees with the 50-digit math and the
engine does not.

## Context
T-4857's probe (probes-numeric/t4857-f-classes and probes/t4857-f-kinds, excel-graph 2026-11-07; scorer
docs/research/t4857/score.py) reached quantile classes with no earlier Excel cell. Where Excel is within 1E-13 of the
math (or both are within 1E-14 of each other) the engine misses:
- `FINV(1-ε,1,37)` (A891, p just below 1): Excel 3.14E-31 = the math; ours 4.35E-15 (statrs' unrefined estimate of
  the rounded 1 - p).
- `F.INV(5.36895450533054E-18,1.8967,1)` (A904, df (1,1)): Excel 7.11E-35, the closed form tan(πp/2)^2; ours
  1.18E-16.
- `F.INV(6.67578851411039E-224,16,75753)` (A982): Excel 5.97E-29 = the math; ours 5.55E-17 (the bisection without a
  start crosses a deep tail the search kernel flushes).
- `F.INV(DBL_MIN,1.3165,11)` and `F.INV(6.87E-245,1.7718,79)` (A907, A913): ours 3E-323 and 2E-322; Excel 3.0E-150
  and 2.1E-149 (its saturation near 1E-150, statistics.md "F.INV.RT saturates ..."; the math underflows).
- `FINV(DBL_MIN,80,2)` (A1095) is Excel's 4.49E+307 = the math, ours 2.25E+306; `FINV(2.84E-305,81,201)` (A1128)
  and `F.INV.RT(1.63E-286,84,200)` (A1148) are 31% and 7.8E-7 off Excel, which is on the math.
- 15th-digit misses at moderate p (A1412, A1966, A1982, A1985, A1994, A1998, A2007, A3112, A3144, A3162, A916, A919;
  and the 26 kinds cells off past 4 ulp: F.INV at df 1 through TRUE, a TRUE reference, `(1=1)` or `SEQUENCE(3)/2` (A302, A308, A319, A331, A337, A348), their lifts, and the F.INV.RT and FINV rows from A389).
- T-1871 (in flight) reworks F.INV where statrs' inversion fails; re-score these cells after it lands before
  changing the search.
- Code: `fisher.rs` `f_quantile`, `f_quantile_tail`; `inverse_beta.rs` `f_inverse_cdf`; `special::refine_increasing`.

Cells (probes-numeric/t4857-f-classes): A891, A904, A907, A913, A916, A919, A982, A1095, A1128, A1148, A1412, A1966, A1982, A1985, A1994, A1998, A2007, A3112, A3144, A3162

Cells (probes/t4857-f-kinds): A302, A308, A319, A322:A323, A325, A331, A337, A348, A351:A352, A354, A389, A395, A406, A409:A410, A412, A421, A476, A482, A493, A496:A497, A499, A508

## Acceptance criteria
- [ ] Each cell above is within 15 digits of Excel or its residue is explained in the Outcome; no gated quantile
      cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells where Excel is far from the math (T-4884) and the huge-df BETA.INV path (T-4885).
