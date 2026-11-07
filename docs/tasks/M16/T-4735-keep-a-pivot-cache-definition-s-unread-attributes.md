---
id: T-4735
title: "Keep a pivot cache definition's unread attributes and elements for export"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
An xlsx save keeps what a pivot cache definition holds that the model does not read (`refreshedBy`/`refreshedDate`, versions, `cacheField@numFmtId`, `sharedItems@minDate`/`maxDate`, `extLst`, `x14` extensions), as the pivot definition already keeps its verbatim rest.

## Context
T-4176's cache export writes the typed model only: the case builder's root attributes (`createdVersion="6"` ...), `numFmtId="0"` on every cacheField, every item as a shared item and every record as `<x v>`, with the `sharedItems` flags derived from the items (`SharedItemsFlags`). Excel opens that (T-4176 oracle runs, docs/excel-semantics/pivots.md "Excel for the web opens engine-exported pivot parts"), but a cacheField's number format (a date field's `numFmtId`, which the pivot's item labels may use) and Excel's extension lists are lost. Add an `OpaqueEntry`-style store to `PivotCache` (charged to the importer's `MAX_OPAQUE_BYTES` budget, SPEC-0032 "Model"), filled by `read_definition` in crates/rustsheet-xlsx/src/import/pivots.rs and written back by crates/rustsheet-xlsx/src/export/pivots/cache.rs, keeping the derived flags authoritative (a refresh recomputes them).

## Acceptance criteria
- [ ] Export then import keeps a cache's unread attributes and elements; an Excel-saved cache (crates/rustsheet-xlsx/tests/example.xlsx) round-trips with its `numFmtId`s and `extLst`.
- [ ] The store counts toward the import limits; a refresh drops what it can no longer vouch for (stale flags).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
