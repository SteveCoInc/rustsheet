---
id: T-4157
title: "xlsx import reads table and table-column dataDxfId from dataDxfId, not headerRowDxfId"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A table's data-area differential style, and each table column's, is read from `dataDxfId`.

## Context
Found by the 2026-11-04 round-trip audit (docs/research/xlsx-round-trip-2026-11-04.md §2.2): `crates/rustsheet-xlsx/src/import/tables.rs:85` (table) and
`:154` (table column) read `data_dxf_id` from the `headerRowDxfId` attribute, a copy-paste bug inherited from IronCalc.
A table with a data-area style therefore gets its header style instead, and loses it on export.

## Acceptance criteria
- [x] A test loads a table, and a table column, with distinct `headerRowDxfId` and `dataDxfId` and gets both
      (preferably from an Excel-saved corpus workbook; grep the corpora for `dataDxfId`).
- [x] Export writes them back unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other table style attributes.

## Outcome
`import/tables.rs` now reads a table's and each table column's `data_dxf_id` from `dataDxfId` (it read
`headerRowDxfId`). The parse moved into `parse_table` so a unit test can feed it XML:
`data_dxf_id_is_read_from_data_dxf_id` loads distinct header/data/totals ids on a table and on its columns, and a
column with only `headerRowDxfId` gets no data style. Export already writes `dataDxfId` from the field
(`export/tables.rs`), so the value now round-trips. No corpus workbook was searched for a `dataDxfId` sample; the
test uses inline XML. Eval delta: none expected (no baseline change).
