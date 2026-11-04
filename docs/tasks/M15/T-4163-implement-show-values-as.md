---
id: T-4163
title: "Implement show-values-as"
milestone: M15
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4161, T-4149]
---

## Goal
PivotResult applies all fourteen show-values-as modes as the evidence says (leaves, subtotals, grand totals).

## Context
SPEC-0030; mog's three-pass dispatcher (oss-engines.md §1) is a design idea only. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] p-showas probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA of show-as values unless evidenced.
