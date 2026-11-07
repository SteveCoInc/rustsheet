---
id: T-4884
title: "F quantiles at a tiny p: Excel's search leaves the math by orders of magnitude below p 1E-175 at (30,3), at p = DBL_MIN and a subnormal p (T-4857 cells)"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Find the mechanism of Excel's F quantile search where it leaves the math at a tiny p, well enough to match Excel's
values on the T-4857 cells (Excel is the target there, decisions 0300 and 0267 item 3's ruling does not cover F), or
bring the evidence to the owner for a decision 0188 R3 ruling.

## Context
T-4857's probe (probes-numeric/t4857-f-classes, excel-graph 2026-11-07; docs/research/t4857/ANALYSIS.md section 6)
found a sharp regime change and many off-math cells (statistics.md "F quantiles at a tiny p leave the math abruptly
(T-4857)"):
- **A jump between p = 1E-172 and 1E-181 at (30, 3).** `F.INV(1E-k,30,3)` is within 3E-15 of the math for k = 1 to
  172 (sweep-p A2758:A2777) and 10 orders of magnitude below it from k = 181 (A2778: 3.66E-24 against 7.76E-14), the
  gap growing to 21 orders at k = 307. `F.INV.RT(1E-k,3,30)` is its exact reciprocal throughout (A2793:A2827). At
  (5, 7) the same sweep stays within 3E-14 of the math to k = 289, is 7E-13 off at 298, and 6% (F.INV) and 0.4%
  (F.INV.RT) off at 307 (A2722, A2757).
- **p = DBL_MIN and below.** `F.INV(DBL_MIN,5,7)` is 6% off the math, `F.INV.RT(DBL_MIN,5,7)` and `FINV` 6%,
  `FINV(DBL_MIN,30,3)` 24 orders of magnitude (edge-p A2219:A2230); a subnormal p from `EXP(-720)` is not read as 0:
  `F.INV(EXP(-720),5,7)` is 1.17E-124 where the math is 4.5E-126 (A1417), `F.INV.RT(EXP(-720),5,7)` and `FINV`
  2.40E+89 (A1435, A1453).
- **Pairwise cells** with Excel more than 1E-4 off the math, the engine on it: tiny p at df pairs never probed
  (A957, A966, A975, A984, A986, A1004, A1005, A1022, A1075, A1159, A1201, A1246, A1285 and the others listed below),
  including F.INV's lower tail, which T-4103 did not cover.
- **Excel `#NUM!`** where the engine has the math: `F.INV.RT(2.29499489338186E-230,16,80)` (A977),
  `FINV(2.09953717878393E-16,79,1E10)` (A1092), `F.INV(0.00149502660982554,1E10,79)` (A1312: the BETA.INV path at
  df 79 and 1E10, T-4044's band).
- **The df2 < 2 saturation** at a fractional df2 (`F.INV.RT(2.15E-276,1,1.1375)` = 3.696066238512854E+150, A884;
  A926 at df1 2): T-4662 owns the mechanism; these two cells are new evidence for it.
- Related: T-4103 (F.INV.RT and FINV tables), T-4568 (p = DBL_MIN), T-4662 and T-4675 (saturation), T-2555
  (BETA.INV's stop).

Cells (probes-numeric/t4857-f-classes): A884, A899, A926, A940, A944, A957, A961, A966, A975, A977, A984, A986, A1004:A1005, A1021:A1022, A1034, A1056, A1075, A1078, A1086, A1090, A1092, A1121, A1144, A1159, A1167, A1188, A1201, A1212, A1246, A1268, A1285, A1312, A1403, A1417, A1421, A1435, A1439, A1453, A2219:A2230, A2233, A2236, A2239, A2242, A2245, A2248, A2251, A2722, A2757, A2778:A2792, A2813:A2827

## Acceptance criteria
- [ ] A probe pins the (30, 3) jump between p 1E-172 and 1E-181 and maps it across df (where it starts as a
      function of df1 and df2).
- [ ] Either a model that matches Excel on these cells (with decision 0188 R4's guards), or a decision record with
      the owner's ruling for them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells where Excel is on the math (T-4883).
