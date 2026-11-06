---
id: T-4723
title: "Probe how Excel reads an empty t=\"str\" cell (no <v>, empty <v>) and an empty inlineStr (<t></t>, <t/>): text \"\" or blank"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/xlsx-storage.md "Empty `t=\"str\"` and empty inline-string cells" moves from UNVERIFIED to
VERIFIED or is corrected: what Excel reads for each of four cell shapes, text "" or a blank cell.

## Context
T-4148 review (excel-fidelity, 2026-11-06). The importer reads all four shapes as the text "", and
crates/rustsheet-xlsx/tests/empty_string_cells.rs pins that. A `t="d"` cell with no `<v>` is imported as blank
instead (xlsx-storage.md, also UNVERIFIED). The shapes appear in poi/bug66675.xlsx, which Excel did not write:
- `<c t="str"></c>`, with no `<v>`;
- `<c t="str"><v></v></c>`;
- `<c t="inlineStr"><is><t xml:space="preserve"></t></is></c>`;
- `<c t="inlineStr"><is><t/></is></c>`.
Code: `get_cell_from_excel` in crates/rustsheet-xlsx/src/import/worksheets.rs (`"str"` uses `unwrap_or("")`, and
`"inlineStr"` uses `rich_text_inline.unwrap_or_default()`).

## Acceptance criteria
- [ ] A fixture (excel-probe skill) has one cell of each shape. For each one, Excel for the web reports `ISBLANK`,
      `ISTEXT`, `LEN` and `COUNTA`. If Excel refuses the workbook, use one workbook per shape. A `t="d"` cell with no
      `<v>` is probed alongside.
- [ ] xlsx-storage.md records the result. If Excel reads a shape as blank, a follow-up task changes the importer and
      the T-4148 test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells that hold a formula.
