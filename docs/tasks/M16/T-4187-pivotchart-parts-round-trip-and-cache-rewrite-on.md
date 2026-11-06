---
id: T-4187
title: "PivotChart parts: round trip and cache rewrite on export"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4186, T-4176]
---

## Goal
Chart parts bound to pivots (c:pivotSource, c:pivotFmts, c14/c15 options) survive import and export, and their caches are rewritten from PivotChartData.

## Context
Depends on chart-part preservation (M11 unknown-part round trip). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Coordinate with M15 (xlsx round-trip fidelity, decision 0275): its ownership matrix replays pivot parts verbatim until pivots have an export (docs/research/xlsx-round-trip-2026-11-04.md, `Replay` and `ReplayIfClean` rows); this task is that export, so it hands pivot parts from `Replay` to modelled and uses M15's package validator and roundtrip suite kind where they exist.

## Acceptance criteria
- [ ] Round-trip suite case with a PivotChart.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
