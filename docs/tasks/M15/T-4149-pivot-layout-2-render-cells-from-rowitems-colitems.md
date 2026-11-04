---
id: T-4149
title: "Pivot layout 2: render cells from rowItems/colItems"
milestone: M15
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4147, T-4137]
---

## Goal
The engine writes the pivot's cells from its rowItems/colItems: captions, item labels (compact indentation is a style, the text is the item), values, page field area, `(blank)`, empty-cell and error captions.

## Context
Evidence from `p-captions` and `p-geometry`. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Pivot-area cells of the PV1 probes match the oracle.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number formats of data fields (PV2).
