---
id: T-5066
title: "Probe BETA.DIST's cdf on the direct bgrat branch where the Cephes kernel returns an exact 0 or 1 (tiny alpha, large beta)"
milestone: M9
lane: functions-stat
status: todo
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
- [ ] statistics.md's T-4866 entry states Excel's values there (replacing its UNVERIFIED), and an engine task is filed
      for the cells ours misses (with `make new-task`), or the cells already match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.DIST's 14th/15th-digit cells on the branch (T-3933).
