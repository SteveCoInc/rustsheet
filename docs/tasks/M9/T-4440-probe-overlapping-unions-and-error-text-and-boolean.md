---
id: T-4440
title: "Probe overlapping unions and error, text and boolean cells in 3-D and union data of TRIMMEAN, Z.TEST and the PERCENTILE family"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An overlapping union and error, text, boolean and blank cells in a union or 3-D range as the data of TRIMMEAN, Z.TEST and the PERCENTILE/QUARTILE/PERCENTRANK family are probed against Excel, and the engine matches.

## Context
T-4309 probed the plain union and 3-D forms (probes/t4309-3d-union-trimmean-percentile). Overlap counting twice and which error wins across areas are by analogy with LARGE/SMALL only (`values_from_areas`). Rule: docs/excel-semantics/statistics.md, the T-4309 sentence.

## Acceptance criteria
- [ ] A probe case with oracle values for these forms is gated; the engine matches, or the deviation is fixed, and the statistics.md sentence loses its "unprobed".
- [ ] `make check` green; reviewers approve.

## Out of scope
-
