---
id: T-4992
title: "Assign owners to the unowned M16 catalog rows listed by the exit row"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4952]
---

## Goal
Every remaining in-scope row of eval/catalog/pivot-features.tsv that the M16 exit row lists as unowned either gets an
open M16 task in its `owner` column or is ruled out of scope by a decision, so the row's owner criterion
(`owners_met`) can read met.

## Context
- T-4952 made the M16 row (report.json `milestone_exit`, `pivot.stages[].unowned`; report.md's lines "PVn remaining rows
  with no open M16 owner") measure decision 0274 item 6's owner criterion. The rule (docs/architecture/eval-harness.md
  "Milestone exit"): a row is owned when its hand-filled `owner` column names an open M16 task; a done, missing or
  other-milestone task owns nothing; an `out` row is its scope decision's and never remains.
- At T-4952's run (2026-11-07) 487 remaining rows had no owner: PV1 358 of 360, PV2 8 of 8, PV3 78 of 78, PV5 43 of
  43. Only PV-1081 and PV-1113 were assigned (T-4915 names them as the attributes it renders).
- Candidates that need a ruling, not a bulk fill: T-4922 (missingCaption `-`, PV-1220?), T-4919 (classic layout,
  PV-1212?), T-4188 (slicers and timelines as pivot filters, the PV5 rows?), the PV1-PV3 implementation tasks of the
  plan (docs/research/pivot-tables-2026-11-04/plan.md §8). A row whose evidence is below the floor may need a probe
  task rather than an implementation task.
- Assign an owner only where the task clearly covers the row; file the missing owner tasks (one per coherent group
  of rows, e.g. per family and stage) and name them in the rows.

## Acceptance criteria
- [ ] Each row the exit row lists as unowned has an open M16 `owner` that covers it, or a decision rules it out
      (a `RULES` change in tools/src/rustsheet_tools/pivot_catalog.py citing it).
- [ ] `make eval` shows `owners met` for every stage on the M16 row, or the remaining unowned ids with why.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the rows.
- Changing the ownership rule (T-4952).
