---
id: T-4177
title: "Round-trip suite: engine-exported pivots re-rendered by Excel"
milestone: M16
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4176]
---

## Goal
A suite whose cases are exported by the engine and sent to the oracle: a refusal or repair fails; Excel's re-render and rowItems must equal the engine's.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §6 layer 4. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Suite and harness support; first cases filled.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
