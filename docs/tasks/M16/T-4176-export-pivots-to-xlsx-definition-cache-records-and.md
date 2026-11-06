---
id: T-4176
title: "Export pivots to xlsx: definition, cache, records and rowItems"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4149]
---

## Goal
xlsx export writes every pivot (typed attributes plus the verbatim rest), its cache and records and the engine's rowItems/colItems, so Excel opens it without repair and renders the same cells.

## Context
Pivots are dropped on export today (T-2081). ClosedXML's invariants (oss-engines.md §4) and the LibreOffice export bugs (libreoffice.md §4.11) are the checklist. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Coordinate with M15 (xlsx round-trip fidelity, decision 0275): its ownership matrix replays pivot parts verbatim until pivots have an export (docs/research/xlsx-round-trip-2026-11-04.md, `Replay` and `ReplayIfClean` rows); this task is that export, so it hands pivot parts from `Replay` to modelled and uses M15's package validator and roundtrip suite kind where they exist.

## Acceptance criteria
- [ ] Unit tests; round-trip suite (task `roundtrip`) passes for the spike and probe pivots.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Charts, slicers.
