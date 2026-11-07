---
id: T-4176
title: "Export pivots to xlsx: definition, cache, records and rowItems"
milestone: M16
lane: engine
status: done
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
- [x] Unit tests; round-trip suite (task `roundtrip`) passes for the spike and probe pivots. (Export then import is a fixed point: `export/pivots/tests.rs` for tests/example.xlsx, tests/pivot_tables.xlsx and an API-built pivot; `rustsheet-eval` `pivot_definition/export_tests.rs` for all 159 probe pivots the importer loads, spike included; `roundtrip-fixtures` unchanged, as no fixture holds a pivot.)
- [x] Type the location geometry (`firstHeaderRow`/`firstDataRow`/`firstDataCol`, `rowPageCount`/`colPageCount`) in `PivotTable` and remove it from the `OpaqueOwner::Location` opaque set, unless T-4147 already did (T-4139 left it verbatim). (Done by T-4147: `PivotGeometry`; the export writes it from the model.)
- [x] `make check` green; reviewers approve. (`make check-fast` green locally; review by the lane's reviewers at PR time.)

## Out of scope
- Charts, slicers.

## Outcome
The regenerating xlsx export now writes pivots (crates/rustsheet-xlsx/src/export/pivots.rs, pivots/cache.rs,
pivots/table.rs) instead of dropping them; this moves pivot parts from the M15 ownership matrix's `Replay` row to
`Modelled` (docs/research/xlsx-round-trip-2026-11-04.md). The source-preserving tiers (SPEC-0034) still keep an
unchanged pivot's own parts byte for byte.
- Cache: every item is written as a shared item and every record as `<x v>` indices, with the `sharedItems` flags
  from `SharedItemsFlags`, the source range or `worksheetSource@name` (new `PivotCache::source_name`, read by the
  importer), `refreshOnLoad`, and dates as ISO text on the epoch the importer reads (1904-aware).
- Definition: typed attributes where they differ from the schema defaults, the verbatim rest on its owner, root
  children in schema order, and the engine's `rowItems`/`colItems` when its layout gives the location and geometry
  the pivot holds (the saved lines otherwise). For API-built pivots, `<items>` are generated on axis fields and
  `dataField="1"` is set (ClosedXML's invariants).
- Shared helpers: `Aggregate::consolidate_name`, `ShowDataAs::from_name`/`name` (the importer uses them too), and
  the importer's subtotal table, reused by the export.
- Excel check: three exported workbooks (example.xlsx, pivot_tables.xlsx, an API pivot) opened without repair in
  Excel for the web (`rustsheet-oracle recalc -b excel-graph`). Excel saved the same location and rowItems/colItems,
  and the pivot cells came back equal (docs/excel-semantics/pivots.md "Excel for the web opens engine-exported
  pivot parts").
- Dropped with a log line: a grouped cache and its pivots (T-4734), a cache with no worksheet source, and a pivot
  whose sheet or cache is gone. Follow-ups: T-4734 (grouped fields), T-4735 (the cache definition's unread rest),
  T-4177 (the Excel re-render suite).
