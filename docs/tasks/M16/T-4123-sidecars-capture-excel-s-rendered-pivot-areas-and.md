---
id: T-4123
title: "Sidecars capture Excel's rendered pivot areas and rowItems/colItems"
milestone: M16
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
An oracle sidecar records, for every pivot Excel saved, its location, every cell of the location and of the page-field area (constants included), rowItems/colItems, group items and displayed texts, so the engine's render can be scored.

## Context
Sidecars hold formula cells only today (probes/pivot-rendered-cells reads the pivot through `=P!A3` formulas). Excel writes the pivot area as constants and rewrites the pivot parts on save (oracle-spike.md run 1, run 4). Values come from the snapshot reader (one parser, full precision). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] `rustsheet-eval snapshot` emits a `pivots` section; `rustsheet-oracle fill` stores it in `*.oracle.json` (format documented in eval-harness.md).
- [ ] A filled probe shows Excel's cells for a compact and a tabular pivot.
- [ ] Snapshot limits (XML-bomb, part size) apply; a malformed pivot part is skipped, never a panic.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scoring the captured cells (task `scoring`).
