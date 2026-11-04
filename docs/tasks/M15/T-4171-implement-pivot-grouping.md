---
id: T-4171
title: "Implement pivot grouping"
milestone: M15
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4170, T-4141]
---

## Goal
Refresh and PivotResult support numeric, date and manual groups with Excel's items and labels.

## Context
SPEC-0030; evidence from `p-groups`. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] p-groups probes pass; corpus group pivots re-render.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auto date grouping on field drop (a UI action).
