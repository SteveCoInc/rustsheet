---
id: T-4952
title: "Measure the M16 owner criterion: map remaining pivot catalog rows to tasks"
milestone: M16
lane: harness
status: todo
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
- [ ] One rule decides whether a catalog row is owned, documented in docs/architecture/eval-harness.md "Milestone exit".
- [ ] `owners_met` is met or not met per stage, with the unowned ids listed; tests cover both.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Filing the owning tasks themselves.
