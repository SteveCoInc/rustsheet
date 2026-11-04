---
id: T-4187
title: "PivotChart parts: round trip and cache rewrite on export"
milestone: M15
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
Depends on chart-part preservation (M11 unknown-part round trip). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Round-trip suite case with a PivotChart.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
