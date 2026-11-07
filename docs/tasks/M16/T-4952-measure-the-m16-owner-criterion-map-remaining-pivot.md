---
id: T-4952
title: "Measure the M16 owner criterion: map remaining pivot catalog rows to tasks"
milestone: M16
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4127]
---

## Goal
report.json's `milestone_exit` M16 row gives each stage's owner criterion a verdict: every remaining in-scope pivot catalog row (below the 30-oracle-cell floor, or with a failing cell) is owned by an open M16 task or has an `out` scope, and the row says which are not.

## Context
Decision 0274 item 6: every remaining row has a task or an out-of-scope ruling. T-4127 added the M16 row (crates/rustsheet-eval/src/milestone_exit/pivot.rs), whose `stages[].owners_met` is null (not measured) while any row remains, since eval/catalog/pivot-features.tsv names no task and almost no task names a catalog id. Options: an `owner` column in the catalog (tools/src/rustsheet_tools/pivot_catalog.py keeps hand-filled columns across regeneration), or tasks naming `PV-nnnn` ids read the way scripts/exit_owners.py reads cells. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] One rule decides whether a catalog row is owned, documented in docs/architecture/eval-harness.md "Milestone exit".
- [x] `owners_met` is met or not met per stage, with the unowned ids listed; tests cover both.
- [x] `make check` green; reviewers approve.

## Out of scope
- Filing the owning tasks themselves.

## Outcome
- Rule (one, option (a) of the brief): eval/catalog/pivot-features.tsv gains a hand-filled `owner` column, which
  tools/src/rustsheet_tools/pivot_catalog.py keeps across regeneration (`KEPT`). A remaining row is owned when its
  `owner` names a task; that task must be an open M16 task. The harness may not read docs/tasks (decision 0066: the
  docs-readers check refused the first draft, which read the task files from Rust), so `scripts/tasks.py validate`
  (`make hygiene`) enforces it as it enforces `TODO(T-nnnn)` owners: scripts/pivot_owners.py fails while an `owner`
  names a done, dropped, missing or other-milestone task, so the PR that closes a task re-points or clears its rows.
  An `owner` that is not `T-<digits>` fails the run. An `out` row is owned by its scope decision
  (`out (decision 0274)`) and never remains. Chosen over tasks naming `PV-nnnn` ids because the catalog is already
  the harness's input (no new Makefile step, generated file or CLI flag) and an explicit column cannot be claimed by
  a passing mention in a task's text. Documented in docs/architecture/eval-harness.md "Milestone exit" and decision
  0274's Consequences (scripts/ changed); docs/milestones.md and catalog-findings.md updated.
- `stages[].unowned` lists the remaining rows with no open owner; `owners_met` is `Some(unowned.is_empty())` per
  stage and the M16 `owners_met` is every stage's. report.md lists the unowned ids per stage below the table.
- Assigned: PV-1081 (`item@n`) and PV-1113 (`pivotField@name`) to T-4915, which names both rows as the attributes it
  renders. No other open task names a row unambiguously; T-4992 tracks the rest.
- At 8001d80b1 plus this change (make eval and make phase1-exit, 2026-11-07), owners **not met** in every stage:
  PV1 358 of 360 remaining rows unowned (2 owned), PV2 8 of 8, PV3 78 of 78, PV5 43 of 43 (487 unowned of 489).
  The M16 line: `gen-pivot not met; pivot-corpus-lo met; pivot-corpus-closedxml not met; probes not met; evidence
  not met (PV1 139/373, PV2 211/211, PV3 13/91, PV5 13/56 at the floor); owners not met; layout not met; corpus
  render n/a`.
