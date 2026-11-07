---
id: T-4745
title: "Pivot classic layout (gridDropZones) and subtotalCaption render"
milestone: M16
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot with `gridDropZones="1"` (classic layout), or with a field's `subtotalCaption`, renders the cells Excel for the
web renders.

## Context
- T-4129's capability matrix found both flags render on the web, and the engine ignores them. See
  docs/excel-semantics/pivots.md, "Layout and totals flags as Excel for the web renders them".
- **gridDropZones** (probes/t4129-pivot-layout-flags GZ, tabular):
  - the data caption `Sum of Sales` is at A3;
  - `Row Labels`, `Product` and `Total` are on row 4, with `Total` heading the value column;
  - the location's `firstHeaderRow` is 2.

  The engine renders the plain tabular header instead, starting at A3 with `Row Labels`.
- **subtotalCaption** (SC): the subtotal lines read the caption alone (`Tot`), with no item name. The engine writes
  `East Total`.
- Code: crates/rustsheet-engine/src/pivot/layout/ (geometry and `render.rs`). The importer is
  crates/rustsheet-xlsx/src/import/pivots/table.rs, which reads the flags or keeps them opaque.

## Acceptance criteria
- [x] The engine renders GZ and SC as the sidecar shows (unit tests against the sidecar, and the cells pass in the eval).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Classic layout's UI-only drag-and-drop behaviour.
- gridDropZones in compact or outline form, unless a probe evidences it.

## Outcome
Done. The model types `pivotTableDefinition@gridDropZones` (`PivotOptions::grid_drop_zones`) and
`pivotField@subtotalCaption` (`PivotField::subtotal_caption`); the importer reads them and the exporter writes
them back. The layout and the render follow Excel for the web:
- **Classic layout** changes a tabular pivot only when its column axis has no level. It then adds a
  column-header row (`firstHeaderRow` and `firstDataRow` 2), puts the data caption at the corner and heads the
  value column `Total`. With a column field, or with Values on columns, the layout is the plain one.
- **subtotalCaption** is the whole label of the field's default subtotal lines, on rows and columns, in tabular,
  outline and compact form, and a `?` in it stands for the item (`? Tot` reads `East Tot`; MS-OI29500's note says
  the field name). Custom subtotal lines keep `<item> <Function>`, and top subtotals keep the item's own line.

Evidence: t4129 GZ and SC, plus a new probe, probes/t4745-pivot-classic-subtotal-caption (excel-graph, 9
pivots: GC, GV, G1, SO, SB, SK, SF, SCC, SQ). Every cell, line and location passes in the render and lines unit
tests, and the t4129 GZ and SC cells pass in the eval (baseline updated). pivots.md has new sections, "Classic
layout (gridDropZones)" and "subtotalCaption labels a field's subtotal lines". The unprobed readings (classic
layout in compact or outline form, with no row field, beside colGrandTotals="0" or grandTotalCaption;
subtotalCaption with Values inside, empty, or with several `?`) are UNVERIFIED and go to T-4785.
