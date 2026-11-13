---
id: T-90015
title: "Legacy implicit intersection of ranges in lookups, AND/OR, 2-D ranges and range corners matches Excel (val11 probes)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: foundational
depends_on: []
---

## Goal
Every visibly-off cell of these cases matches Excel, or its verdict is recorded in docs/excel-semantics. Split into one implementation task per rule found.

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/legacy-lookup-value-intersection, probes/legacy-lookup-value-range-intersection, probes/legacy-and-or-operation-intersection, probes/implicit-intersection-two-d-range, probes/range-corner-right-operands, probes/at-unary-sign-range, probes/t-n-at-unary-plus-if-array, probes/subtotal-areas-lifted-pick, probes/3d-ref-one-sheet-span-row-delete (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
