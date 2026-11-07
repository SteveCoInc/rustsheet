---
id: T-4733
title: "Pivot classic layout (gridDropZones) and subtotalCaption render"
milestone: M16
lane: engine
status: todo
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

Owner of these visibly-off pivot cells (decision 0245):
- probes/t4129-pivot-layout-flags GZ!A3:A5, GZ!A7:A12, GZ!B3:B10, GZ!C3:C12, SC!A7, SC!A10

## Acceptance criteria
- [ ] The engine renders GZ and SC as the sidecar shows (unit tests against the sidecar, and the cells pass in the eval).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Classic layout's UI-only drag-and-drop behaviour.
- gridDropZones in compact or outline form, unless a probe evidences it.
