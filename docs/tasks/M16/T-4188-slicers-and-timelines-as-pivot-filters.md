---
id: T-4188
title: "Slicers and timelines as pivot filters"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4165, T-4176]
---

## Goal
Slicer and timeline caches are imported as filter state on the pivots they drive, applied by PivotResult, and preserved on export.

## Context
excel-docs.md slicer/timeline rows; x14/x15 parts. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Drawing slicers.
