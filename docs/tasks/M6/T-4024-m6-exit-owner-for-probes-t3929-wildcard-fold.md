---
id: T-4024
title: "M6 exit owner for probes/t3929-wildcard-fold-xmatch Calc!E3, E7, E29, E30: they pass once T-3904 and T-4013 land"
milestone: M6
lane: engine
status: todo
priority: P0
size: S
tier: sonnet
depends_on: [T-3904, T-4013]
---

## Goal
The four M6 exit cells of probes/t3929-wildcard-fold-xmatch have an owner of their milestone (decision 0096), and they
pass once T-3904 and T-4013 land; if those two do not fix them, this task finds and fixes the M6 part that is left.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main e173e8a2 with calipers, corpus-bench and poi fetched):
  `python3 scripts/exit_owners.py --milestone M6` lists four cells, because the case is tagged M6 (it counts toward
  the M6 probe row) while its only owners, T-3904 and T-4013, are M2 tasks. Every other milestone's exit list is
  empty (`--milestone` M1, M2, M5, M7, M8, M10, M11; M9's 32 t3981 cells pass on main after T-3982).
- The cells, all `=IFNA(XMATCH(Data!Bn,Data!An,0),0)` (exact mode, so `*` and `?` are literal): Excel 1, ours 0
  (`NumericMismatch`):
  - E3: `a`&U+0524&`b` against `ab` (U+0524 is weightless in Excel's comparison): T-3904.
  - E7: U+0524 against U+10C7 (two weightless characters equal to each other): T-3904.
  - E29: `Ͷ` against `ϳ`, E30: `ϑ` against `ϴ` (T-3584's MATCH classes): T-4013.
- Evidence and rules: coercion.md "Weightless characters in `=`, COUNTIF and XMATCH (T-3584)"; lookup.md
  "Equivalence classes"; criteria.md "Case beyond ASCII" (T-3929). All four lines are in eval/baseline/probes.txt
  (`F probes/t3929-wildcard-fold-xmatch.case.json Calc!E3`, `Calc!E7`, `Calc!E29`, `Calc!E30`).
- This task records the split, as T-3997 does for M5: T-3904 and T-4013 change the comparison, this one confirms the
  M6 cells and owns them for the M6 exit. T-3904 and T-4013 both name these cells.
- Cells:
  - probes/t3929-wildcard-fold-xmatch.case.json (4): Calc!E3, E7, E29, E30

## Acceptance criteria
- [ ] After T-3904 and T-4013 are merged, `make eval-show CASE=probes/t3929-wildcard-fold-xmatch.case.json` shows
      Calc!E3, E7, E29 and E30 passing, and their four known-failure lines leave eval/baseline/probes.txt with no new
      lines in any suite (`make eval-baseline`).
- [ ] If any still fails, the Outcome gives Excel's and the engine's value and the cause, and the fix is made here
      when it is size S and needs no numeric fitting; otherwise the task is handed up (work-loop step 5b).
- [ ] `python3 scripts/exit_owners.py --milestone M6` no longer lists any of the four cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The comparison itself: the weightless characters (T-3904) and the MATCH classes (T-4013).
- The criteria cells of probes/t3929-wildcard-fold-criteria (M2 row; T-3904 and T-4013 own them).
