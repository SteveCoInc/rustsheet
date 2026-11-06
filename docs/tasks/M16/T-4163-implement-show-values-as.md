---
id: T-4163
title: "Implement show-values-as"
milestone: M16
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
SPEC-0032; mog's three-pass dispatcher (oss-engines.md §1) is a design idea only. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; moved from T-4152, whose adapter loads the
  case builder's parts through the importer, which skips a pivot with a `showDataAs` data field, "a data
  field shown as a calculation"): probes/t4123-pivot-capture T!A3:E12, T2!A3:E12, T3!A3:E12,
  T4!A3:E12.

## Acceptance criteria
- [ ] p-showas probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA of show-as values unless evidenced.
