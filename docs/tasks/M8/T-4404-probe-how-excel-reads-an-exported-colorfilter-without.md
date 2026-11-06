---
id: T-4404
title: "Probe how Excel reads an exported colorFilter without dxfId (a dxf index past the workbook's dxfs)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4004]
---

## Goal
We know, from Excel, whether a table whose `<colorFilter>` the export writes without its `dxfId` opens without
repair and still counts its hidden rows as filtered for SUBTOTAL.

## Context
- T-4004 keeps a table's color filter through import and export. When the imported `dxfId` is not one of the
  workbook's dxfs, `filter_column` in crates/rustsheet-xlsx/src/export/tables.rs drops the attribute, as
  `dxf_attribute` does for the table's own dxf ids, and writes `<colorFilter/>` (or `<colorFilter cellColor="0"/>`).
  The T-4004 excel-graph upload covered only `dxfId="0"`, a dxf that exists. The excel-fidelity review of T-4004
  (PR #3425) asked for this probe.
- Recipe: docs/excel-semantics/xlsx-storage.md, "Table parts in the export". Write the table with openpyxl and a
  `ColorFilter(dxfId=7)` on a workbook with no dxfs, round-trip it through `load_from_xlsx` and `save_to_xlsx`, then
  run `rustsheet-oracle recalc -b excel-graph` on the export. Upload a `dxfId`-less `<colorFilter/>` written directly
  as well.

## Acceptance criteria
- [ ] Both uploads are recorded in xlsx-storage.md "Table parts in the export", with whether Excel opens each file
      and its SUBTOTAL 9 and 109 values. If Excel refuses the file or reads the rows as manual hides, the import or
      export changes to match (for example by writing the column with no criterion), with a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a color filter.

## Outcome
excel-graph (2026-11-06): the dangling `dxfId=7` file is refused; the export (`<colorFilter/>`), a direct `<colorFilter/>`
and `<colorFilter cellColor="0"/>` open and read as manual hides (SUBTOTAL 9/109/3/103 = 21/15/6/4). Import now reads a
`<colorFilter>` without `dxfId` as criterion-less (unit test in filter_criteria.rs). Recorded in xlsx-storage.md.
No eval delta.
