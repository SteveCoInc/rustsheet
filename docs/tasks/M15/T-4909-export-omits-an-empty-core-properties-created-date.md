---
id: T-4909
title: "Export writes neither an empty core-properties created date nor an empty cellStyles (Excel refuses both)"
milestone: M15
lane: engine
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A regenerated save opens in Excel even when the source records no creation date or has no cell styles. Today Excel
for the web refuses both.

## Context
Found by the T-4890 probe (docs/research/xlsx-round-trip/excel-resave-probe-2026-11-07.md §3;
docs/excel-semantics/xlsx-storage.md "An empty `dcterms:created` makes Excel refuse the package").
- `crates/rustsheet-xlsx/src/export/doc_props.rs` `get_core_xml` always writes
  `<dcterms:created xsi:type="dcterms:W3CDTF">{created}</dcterms:created>`. When the source has no
  `docProps/core.xml`, or no `created` in it, `created` is empty.
- Excel for the web refuses that package with `FileCorruptTryRepair`. Removing only that element makes the same
  package open (fixtures valueless-cell-forms R1-E5).
- `crates/rustsheet-xlsx/src/export/styles.rs` writes `<cellStyles count="{n}">…</cellStyles>` even when `n` is 0.
  Excel for the web refuses `<cellStyles count="0"></cellStyles>` (poi link-external-workbook-a R1/all). The same
  package opens with the element removed, and with the Normal style added (xlsx-storage.md "An empty `cellStyles`
  makes Excel refuse the package").
- In `get_core_xml`, `creator` is written without XML escaping. A `<` or `&` in the source's creator would
  break the part.

## Acceptance criteria
- [ ] An empty `created` is not written, and `creator` is escaped (unit tests).
- [ ] An empty cell-style table is either not written or written with the Normal style; choose which from Excel's own
      saves (unit test).
- [ ] The R1/all saves of fixtures valueless-cell-forms and poi link-external-workbook-a open in Excel for the web
      (`rustsheet-oracle resave`). Record this in both semantics entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing a real creation date when the source has none.
