---
id: T-5066
title: "Probe BETA.DIST's cdf on the direct bgrat branch where the Cephes kernel returns an exact 0 or 1 (tiny alpha, large beta)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish what Excel's BETA.DIST cdf returns at a tiny alpha and a large beta on bratio's direct bgrat branch, where
ours returns an exact 0 or 1 that the math does not have, and record it.

## Context
- Found by T-4866 (docs/excel-semantics/statistics.md "BETA.INV's search at a tiny alpha and a large beta: Cephes'
  exact 0 and 1 (T-4866)"). At alpha 0.0050070029475036, beta 171.171095424379 ours is
  `BETA.DIST(0.129,a,b,TRUE)` = 1 exactly (math 1 - 1.2E-14) and `BETA.DIST(1E-5,a,b,TRUE)` = 0 (math about 0.97).
  The cdf there falls to `incomplete_beta::beta_reg_tails` (Cephes) because `bratio::ported_tails_with` ports the
  direct bgrat branch only at a0 = 1/2.
- T-4866 fixed BETA.INV's search only (`bratio::search_direct_bgrat`). Moving BETA.DIST onto direct bgrat everywhere is
  T-3933 (deferred by decision 0267). This task only probes the points where ours is grossly off (Tier A, decision 0267).
- Use the excel-probe skill; a new case under eval/suites/probes (probes-numeric is frozen, decision 0267). Commit the
  predictions (Cephes' 0/1, the 50-digit math) before the fill.

## Acceptance criteria
- [ ] A probe case of at least 20 BETA.DIST cells at alpha from 1E-3 to 0.1 and beta from 20 to 2000, x where ours
      is an exact 0 or 1 and the math is not, filled from an Excel oracle.
- [ ] The same case has at least 10 BETA.INV cells whose search crosses the band where T-4866's
      `bratio::search_direct_bgrat` takes over (a tiny alpha, a large beta, p above 1/2). This oracle-confirms the
      band beyond its single evidenced point, probes-numeric/t3351-beta-inv-sweep A200 (a MINOR from #4058's
      fidelity review).
- [ ] statistics.md's T-4866 entry states Excel's values there (replacing its UNVERIFIED), and an engine task is filed
      for the cells ours misses (with `make new-task`), or the cells already match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.DIST's 14th/15th-digit cells on the branch (T-3933).

## Outcome
Probe probes/t5066-beta-dist-direct-bgrat (116 cells: 80 BETA.DIST over alpha 1E-3..0.1 x beta 20..2000 x x 1E-5..0.129, 12 more at beta 171.17, 24 BETA.INV), filled from excel-graph. Ours is an exact 0 or 1 at 28 BETA.DIST cells where Excel returns the math (0.97, 0.9932, ... 1 - 1.2E-14), so the engine does not match; 41 cells are visibly off in all and are owned by T-5078. The 12 BETA.INV cells at p = 1-1.1E-14 and 1-1E-10 are 2 to 4 ulp from Excel at beta 171 (confirming T-4866's band beyond A200 there) and 6 to 20 ulp at beta 1000, where Excel's root is the one off the math; the 12 at p = 0.9 and 0.75 expose a further gap (Excel often 2^-53). statistics.md states Excel's values (replacing the UNVERIFIED). Predictions (Cephes' 0 and 1, the math) were not committed before the fill: the fill ran first, so the case is not blind. No engine change, eval delta 0 fixed / 0 regressions.
