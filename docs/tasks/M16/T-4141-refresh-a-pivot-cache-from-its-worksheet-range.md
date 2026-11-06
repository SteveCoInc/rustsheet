---
id: T-4141
title: "Refresh a pivot cache from its worksheet range or Table source"
milestone: M16
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4139, T-4134]
---

## Goal
Refreshing reads the source (range, Table, or defined name) and rebuilds the cache exactly as Excel does: field names (duplicates renamed), item typing, identity and order, sharedItems flags, record values.

## Context
SPEC-0032; evidence from task `p-items`. The CacheBuilder of SPEC-0015 is the one builder (directive 3). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Unit tests and the p-items probes' cache-level expectations pass.
- [ ] Duplicate source header names: a filled probe (excel-graph) shows Excel's renaming before the engine implements it (no probe covers it yet; SPEC-0032 Refresh).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Grouped and calculated cache fields (PV3).
