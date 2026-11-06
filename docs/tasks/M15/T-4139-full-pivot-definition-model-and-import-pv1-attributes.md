---
id: T-4139
title: "Full pivot definition model and import (PV1 attributes typed, the rest verbatim)"
milestone: M15
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4121]
---

## Goal
The workbook model holds every pivot definition attribute that PV1 needs, typed (layout flags per pivot and per field, subtotal flags and placement, grand totals, items with h/sd/s, page field selections, data field names/aggregates/number formats, captions), and keeps every other element and attribute verbatim for round trip; the importer fills it under the SPEC-0015 limits.

## Context
SPEC-0032; crates/rustsheet-engine/src/pivot.rs; crates/rustsheet-xlsx/src/import/pivots.rs. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Unit tests importing the spike and corpus shapes.
- [ ] No GETPIVOTDATA regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rendering.
