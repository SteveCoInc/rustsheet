---
id: T-4974
title: "Fill the 2026-11-07 probe suggestions (PR 4002) with excel-graph"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The 24 draft probe cases of PR #4002 (docs/research/probe-suggestions-2026-11-07, groups G01-G24) are in the probes
suite with excel-graph values, scored against the engine, and every visibly-off cell they add has an owner task.

## Context
PR #4002 lists probes that would confirm or falsify rules the 2026-11-07 Excel-parity fixes inferred rather than
measured (gamma Q-to-P switch and split complement, AS 109 stops, F back-transform, Student-t stops, LOGNORM density,
Loader store chain, SKEW/KURT x87 rounding (T-4328), IRR caps, BETA.INV boundaries, T.DIST at df 1000). The cases are
copied as `eval/suites/probes/cases/t4974-<group>.case.json` (the `val-` prefix dropped), unchanged otherwise.

## Acceptance criteria
- [ ] The 24 cases have excel-graph `.oracle.json` and Gate A `.math.json` sidecars.
- [ ] Their failing cells are baselined (`make eval-baseline`) and every visibly-off cell has an owner task.
- [ ] The per-group outcome (which inferred rules Excel confirms or falsifies) is in the Outcome note.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The merged-range scenarios M1-M6: they are operation scripts, not cases (`sheet_ops` has no unmerge, clear,
  paste, autofill or edge op); excel-graph cannot run copy, autofill or edge navigation.
- Fixing the engine where a probe falsifies a rule (owner tasks).
