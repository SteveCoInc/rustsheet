---
id: T-4913
title: "exit_owners.py counts a cell a decision record lists as superseded (decision 0329 item 4) as owned, so unowned-off stops flagging gen-m9/17c110a53924beb8 Calc!A300"
milestone: M7
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`make unowned-off` and `make exit-owners` stop reporting visibly-off cells that a decision record has settled as
superseded F lines, starting with gen-m9/17c110a53924beb8 Calc!A300, without anyone filing a task that cannot be done.

## Context
The 2026-11-07 steward run found Calc!A300 with no open owner (`make unowned-off`) and first named it in T-4737, then
filed this task to make its F line leave eval/baseline/gen-m9.txt. Both were wrong: decision 0329 item 4 names this
cell (with probes/t2475-stat-subexpressions P!A62, A63) as superseded: "Ours already gives their November values, so
they are F lines against the October values today. They stay F lines, and they need no regression run." Item 3 forbids
re-filling or editing the October sidecar, so the line can never leave. The evidence is already recorded:
- October fill (eval/suites/gen-m9/cases/17c110a53924beb8.oracle.json): Calc!A300 `#N/A`.
- 2026-11-06 refill (probes/gen-m9-17c110a53924beb8-refill-1106.oracle.json): `#VALUE!`, which ours returns.
- docs/excel-semantics/arrays.md names the November `#VALUE!` for this cell and cites the refill.
- eval/baseline/gen-m9.txt keeps `F gen-m9/17c110a53924beb8.case.json Calc!A300`, as the decision says.
So the gap is in the owner tools: scripts/exit_owners.py treats only open tasks as owners. Decision 0329's
consequences leave excluding such cells from scoring to T-3610; this task changes only who counts as their owner.

The same applies to the cells T-4737 superseded under decision 0329 item 3 (its one `--allow-regressions` run). The
2026-11-07 evening steward run found `make exit-owners MILESTONE=M7` listing 33 of them with no open M7 owner. This
task owns them until the owner tools count them as settled:
- gen-m7/21b4eef18424ce94 Calc!B1505:E1514 (26 spill cells of Calc!A1504; November refill in probes/gen-m7-21b4eef18424ce94-refill-1107)
- probes/t2360-array-lambda-holidays (5 cells of column C on sheet P)
- probes/t2311-pivotby-value-columns-error P!C12
- probes/t2625-nested-consumers P!A32

## Acceptance criteria
- [ ] scripts/exit_owners.py reads the cells a decision record lists as superseded from one machine-readable place
      (for example an "Owned cell:"-style line in the decision, or the semantics entry decision 0329 item 3 requires)
      and counts them as owned by that decision in `--unattributed` and `--milestone` reports; the report names the
      decision as the owner.
- [ ] A unit test in scripts/ covers a superseded cell (owned, not reported) and an unlisted one (still reported).
- [ ] After `make eval`, `make unowned-off` does not list gen-m9/17c110a53924beb8 Calc!A300.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing the F line or changing the engine: decision 0329 keeps the line.
- Excluding superseded cells from scoring (T-3610).
