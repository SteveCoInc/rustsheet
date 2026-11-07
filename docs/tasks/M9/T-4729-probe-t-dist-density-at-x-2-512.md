---
id: T-4729
title: "Probe T.DIST density at |x| >= 2^512 with a df that truncates to 0: #NUM! or #DIV/0! first (T-3554 review follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know which error Excel returns for `T.DIST(x, df, FALSE)` when |x| >= 2^512 (fl(x*x) overflows) and df truncates
to 0, so docs/excel-semantics/statistics.md's T-3554 entry can say "every df" or "every df >= 1" from evidence.

## Context
- T-3554 (#3753) made the density `#NUM!` where fl(x*x) overflows, from probes at df >= 1 only
  (eval/suites/probes/cases/t3554-tdist-density-huge-x.case.json).
- The engine checks `df == 0` first (crates/rustsheet-engine/src/functions/statistical/t_dist.rs), so
  `T.DIST(2^512, 0.5, FALSE)` returns `#DIV/0!`. No eval cell covers it. If Excel squares x first, it returns
  `#NUM!` instead (excel-fidelity review MINOR on #3753).

## Acceptance criteria
- [ ] A probe case (excel-probe skill, predictions committed before the fill) covers `T.DIST(±2^512, 0.5, FALSE)`,
      `T.DIST(±2^512, -0.5, FALSE)` and the same with `TDIST`-family neighbours where they take a df below 1.
- [ ] statistics.md's T-3554 entry states the df range the evidence covers; a mismatch gets an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine unless the probe shows a mismatch (then file it).
