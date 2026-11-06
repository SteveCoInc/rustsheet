---
id: T-4157
title: "xlsx import reads table and table-column dataDxfId from dataDxfId, not headerRowDxfId"
milestone: M11
lane: engine
status: todo
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
- [ ] A test loads a table, and a table column, with distinct `headerRowDxfId` and `dataDxfId` and gets both
      (preferably from an Excel-saved corpus workbook; grep the corpora for `dataDxfId`).
- [ ] Export writes them back unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other table style attributes.
