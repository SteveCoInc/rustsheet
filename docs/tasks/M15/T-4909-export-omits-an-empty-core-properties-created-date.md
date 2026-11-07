---
id: T-4909
title: "Export writes neither an empty core-properties created date nor an empty cellStyles (Excel refuses both)"
milestone: M15
lane: engine
status: done
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
- [x] An empty `created` is not written, and `creator` is escaped (unit tests).
- [x] An empty cell-style table is either not written or written with the Normal style; choose which from Excel's own
      saves (unit test).
- [x] The R1/all saves of fixtures valueless-cell-forms and poi link-external-workbook-a open in Excel for the web
      (`rustsheet-oracle resave`). Record this in both semantics entries.
- [x] `make check` green; reviewers approve.

## Out of scope
- Writing a real creation date when the source has none.

## Outcome
- **`created`.** `export/doc_props.rs` `get_core_xml` writes no `created` element when the model has no creation
  date. It now escapes the creator and the date.
- **`cellStyles`.** `export/styles.rs` `get_cell_styles_xml` writes no element for an empty cell-style table.
  - This keeps what such a source had. Excel's own save of such a source adds the Normal style (probe
    `csrc-excel`).
  - Excel opens the form without the element (T-4890 variant `5d941486…`), so the source's form is kept.
- **Found on the way.** `import/metadata.rs` read the app and core properties as one unit. A package without
  `docProps/app.xml` (poi/xlsx-jdbc.xlsx) therefore lost its creator and creation date, and the save wrote an empty
  date. Each part is now read on its own with its own defaults, and the fallback block in `import/mod.rs` is gone.
- **Tests.**
  - `export/test/test_doc_props.rs`: an unknown creation date is left out; the creator is escaped; an empty cell-style
    table is left out.
  - `import/metadata.rs`: core properties are kept without app properties.
- **Oracle** (excel-graph, 2026-11-07). The three saves T-4890 found refused now open: fixtures valueless-cell-forms
  R1/E5 and R1/all, and poi link-external-workbook-a R1/all (sha256 863551c9…, a76a541e…, 5ee42e7b…). Both semantics
  entries in docs/excel-semantics/xlsx-storage.md now say this.
- **Baselines.** Rerecorded with no regression. 14 fixtures and 8 poi entries shrank, and the fixtures R1 kept rate
  went from 93.65% to 94.57%.
