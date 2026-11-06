---
id: T-4165
title: "Implement pivot label/value/date filters, Top-N and value sorting"
milestone: M15
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4164, T-4149]
---

## Goal
PivotResult applies the filters and sorts of the catalog's filtering and sorting families as evidenced.

## Context
SPEC-0032; evidence from `p-filters`. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] p-filters probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Slicers and timelines (PV5).
