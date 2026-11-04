---
id: T-4044
title: "F.DIST and F.DIST.RT are #NUM! at df1 79, df2 1E10 (16 t4017 cells the engine returns; owned only by deferred T-3209)"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: S
tier: opus
depends_on: [T-4030]
campaign: semantics
---

## Goal
F.DIST(x,df1,df2,TRUE) and F.DIST.RT(x,df1,df2) are `#NUM!` wherever Excel is, starting with every x at df1 = 79,
df2 = 1E10, from the rule T-4030 finds, not from a cut at the probed cells.

## Context
- Excel (excel-graph, T-4017) is `#NUM!` at all 16 cells of df1 = 79 in
  probes-numeric/t4017-f-huge-df2-above-mean-1e10, where the engine returns a number (`make eval` on origin/main
  a7b115e9e; gap triage 2026-11-04b):
  - probes-numeric/t4017-f-huge-df2-above-mean-1e10.case.json (16): Calc!A209, A210, A211, A212, A213, A214, A215, A216, A217, A218, A219, A220, A221, A222, A223, A224
  - e.g. A209 `F.DIST.RT(1.5,79,1E10)` Excel `#NUM!`, engine 0.0026850092067944196; A210 `F.DIST(1.5,79,1E10,TRUE)`
    Excel `#NUM!`, engine 0.9973149907932056; A223 `F.DIST.RT(50,79,1E10)` Excel `#NUM!`, engine 0.
- df1 up to 60 at df2 = 1E10 returns numbers in Excel (A1-A208 of the same case), and df1 = 79 at df2 1E7..1E9
  returns numbers (the other five t4017 cases). docs/excel-semantics/statistics.md "Incomplete beta at a huge first
  shape" (the T-4017 sweep paragraph) records the quirk; the per-df cut of "F-distribution df limit" (T-1530,
  `F_DF_MAX` in crates/rustsheet-engine/src/functions/statistical/fisher.rs: a df above 1E10 is `#NUM!`) does not
  explain it.
- These cells were reachable only through T-3209, which is `defer: phase-1b` (its subject is Excel's bgrat digits),
  so `make unowned-off ALL=1` listed them as unowned (`F.DIST (deferred)`, `F.DIST.RT (deferred)`). A wrong type is
  not a last-digit residue, hence this task.
- T-4030 (research) brackets the df1 boundary (61 to 78 at df2 1E10), probes df2 5E9 and 2E10, and BETA.DIST at the
  matching shapes, to tell a df cut that depends on both dfs from a bratio branch at b = 39.5. Implement what it
  finds; if T-4030 finds no rule that its cells pin, record that here and block on a further probe instead of fitting
  a cut to these 16 cells.
- Also check FDIST (legacy) and F.INV / F.INV.RT at the same shapes if T-4030 covers them: one definition for the
  check, shared by every F function it applies to.

## Acceptance criteria
- [ ] The 16 cells above are `#NUM!`; the other 1,329 t4017 cells keep their verdicts or improve (cell by cell,
      `rustsheet-eval show --all` on the six cases, before and after).
- [ ] T-4030's probe cells pass with the right type on both sides of the boundary it finds.
- [ ] The rule (and its evidence) in docs/excel-semantics/statistics.md, replacing "unprobed (T-4030)".
- [ ] No gated cell regresses (`make eval-baseline`); new failing cells have owners.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's digits at huge df2 (T-3209, T-4018) and its exact-0 tails (T-4045).

## Visible-parity stage
Decision 0083 and 0114 (`semantics`): done when every cell this task owns has Excel's type and error.

## Decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) makes this a Tier A task, worked first: its cells have the wrong type or error, or are first wrong at the 11th digit or earlier. Decision 0056's time-box applies. If two attempts do not move the cells, block the task as time-boxed, and decision 0267 accepts the cells that are left.
