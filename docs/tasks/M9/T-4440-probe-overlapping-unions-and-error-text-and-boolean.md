---
id: T-4440
title: "Probe overlapping unions and error, text and boolean cells in 3-D and union data of TRIMMEAN, Z.TEST and the PERCENTILE family"
milestone: M9
lane: functions-stat
status: done
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

## Outcome
Probed (probes/t4440-union-overlap-error-text-trimmean-percentile, 64 cells, excel-graph, 2026-11-07): an overlapping union counts twice, text and boolean cells are skipped, the first error in area order wins, in 3-D and union data of TRIMMEAN, Z.TEST, PERCENTILE, QUARTILE.EXC, PERCENTRANK and PERCENTRANK.EXC. The engine already matched all 64 cells, so no engine change. statistics.md lost its "unprobed". Eval delta: +64 passing probe cells, no baseline entries needed.
