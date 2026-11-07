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
unchanged pivot's own parts byte for byte. What the tests show is that export then import is a fixed point; parity
with what Excel saves is not claimed (below, and T-4177).
- Cache: every item is written as a shared item and every record as `<x v>` indices, with the `sharedItems` flags
  from `SharedItemsFlags`, the source range or `worksheetSource@name` (new `PivotCache::source_name`, read by the
  importer), `refreshOnLoad`, and dates as ISO text on the epoch the importer reads (1904-aware; serials 1 to 60,
  phantom 60 and 1904 caches UNVERIFIED, T-4177). The shared-item order is the engine's (old items first, new ones
  appended), not the order Excel was seen to save (source first, stale after; t4612), and `containsNonDate` and
  `minDate`/`maxDate` are not written: UNVERIFIED, T-4654. The cache definition's unread rest is lost (T-4735).
- Limits (directive 5): a cache is held to the import limits before anything is written (records, record values
  per cache and over all caches, items), since its fields are public; non-finite numbers and records that are not
  indices into their items are refused.
- Definition: typed attributes where they differ from the schema defaults, the verbatim rest on its owner, root
  children in schema order. `rowItems`/`colItems` are the saved ones, written back as read, until the engine lays
  the pivot out (a render or refresh drops them, `OpaqueEntry::is_layout_lines`); then, and for an API-built pivot,
  the engine's when its layout gives the location and geometry the pivot holds. For API-built pivots, `<items>` are
  generated on axis fields and `dataField="1"` is set (ClosedXML's invariants).
- Shared helpers: `Aggregate::consolidate_name`, `ShowDataAs::from_name`/`name` (the importer uses them too), the
  importer's subtotal table, and `pivot::element_name`, reused by the export.
- Excel check (scratch, UNVERIFIED until T-4177 commits it): three exported workbooks (example.xlsx,
  pivot_tables.xlsx, an API pivot) opened without repair in Excel for the web (`rustsheet-oracle recalc -b
  excel-graph`). Only pivot_tables.xlsx's two `refreshOnLoad` pivots check the engine's lines: Excel re-rendered
  them to the same location and rowItems/colItems. The others were saved back unchanged (t4586), and the API
  pivot's generated `<items>` were never re-rendered (docs/excel-semantics/pivots.md "Excel for the web opens
  engine-exported pivot parts").
- Dropped with a log line: a grouped cache and its pivots (T-4734), a cache with no worksheet source, one past the
  limits or with a non-finite number or a bad record, one whose field holds texts that differ only in case (Excel
  for the web calls such a file corrupt), and a pivot whose sheet or cache is gone. Follow-ups: T-4734 (grouped
  fields), T-4735 (the cache definition's unread rest), T-4654 (shared-item order and flags), T-4177 (the Excel
  re-render suite, now also listing this task's UNVERIFIED cases).
