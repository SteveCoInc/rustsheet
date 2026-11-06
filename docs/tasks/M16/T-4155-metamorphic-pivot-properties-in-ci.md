---
id: T-4155
title: "Metamorphic pivot properties in CI"
milestone: M16
lane: quality
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4151]
---

## Goal
Property tests over generated pivots: refresh is idempotent; GETPIVOTDATA of each value cell's tuple returns the cell; import-export-import is a fixed point (once export exists); permuting source rows changes only order-dependent outputs.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §6 layer 5. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Proptest-style tests committed and run in CI time budget.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Oracle-backed checks.
