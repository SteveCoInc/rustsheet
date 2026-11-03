---
id: T-3875
title: "GROUPBY/PIVOTBY snap: probe the two-argument aggregation LAMBDA and list the callers in snapping_root docs"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The GROUPBY/PIVOTBY aggregation-LAMBDA snap is probed for the two-argument form, and the `snapping_root` doc comment lists its callers.

## Context
- T-3617 (PR #2893) made GROUPBY/PIVOTBY aggregation LAMBDAs call through `LambdaCaller::snapping_root`. Its fidelity reviewer noted:
  - the snap also covers the two-argument form (`Call::Two`, `LAMBDA(v,a,SUM(v)-0.3)`), which neither t3112 probe exercises; it is inferred from the one-argument evidence and the T-3095 helpers;
  - the doc comment of `snapping_root` (functions/logical/lambda.rs) still lists only in-place calls, MAP, MAKEARRAY, BYROW, BYCOL, REDUCE and SCAN.
- Use the `excel-probe` skill for the probe.

## Acceptance criteria
- [ ] A probe case covers a two-parameter GROUPBY and PIVOTBY aggregation LAMBDA whose body snaps, with oracle values; the engine matches or the gap is recorded.
- [ ] The `snapping_root` doc comment names GROUPBY and PIVOTBY aggregation, citing probes/t3112-snap-groupby-eta.
- [ ] docs/excel-semantics/numbers.md "GROUPBY and PIVOTBY aggregation LAMBDAs" records the two-argument result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The PERCENTOF internal sum caller (stays unsnapped).

## Outcome
- Probe `probes/t3875-snap-groupby-two-arg` (30 cells, excel-graph 2026-11-03) covers two-parameter GROUPBY and PIVOTBY aggregation LAMBDAs over one and two groups; the engine already matched all 30 cells, so no engine change was needed. No Gate A sidecar is owed (no one-call numeric cells).
- `snapping_root` doc comment now names GROUPBY/PIVOTBY aggregation; numbers.md records the two-argument result.
- Eval delta: +30 passing cells (new probe), no regressions.
