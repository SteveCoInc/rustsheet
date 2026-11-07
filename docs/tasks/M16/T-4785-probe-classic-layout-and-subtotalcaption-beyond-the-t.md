---
id: T-4785
title: "Probe classic layout and subtotalCaption beyond the T-4745 shapes"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4745]
---

## Goal
The engine's readings of classic layout (`gridDropZones`) and `pivotField@subtotalCaption` beyond the shapes
probes/t4129-pivot-layout-flags and probes/t4745-pivot-classic-subtotal-caption show are evidenced, and corrected
where Excel differs.

## Context
Split from T-4745, which implemented the probed shapes (pivots.md "Classic layout (gridDropZones)" and
"subtotalCaption labels a field's subtotal lines"; crates/rustsheet-engine/src/pivot/layout/lines.rs `geometry`,
render.rs `render_within` and `Cells::total_label`). Engine readings left UNVERIFIED (T-4785):
- classic layout in compact and in outline form with no column level (the engine adds the column-header row, the
  corner data caption and the `Total` header as in tabular form);
- classic layout with no row field, with `colGrandTotals="0"`, and with a `grandTotalCaption` (does it replace the
  `Total` header? the engine always writes `Total`);
- classic layout with two or more data fields and Values on rows (`dataOnRows`): the engine adds the header row,
  writes `Total` and leaves the corner empty;
- classic layout with row fields and no data field: the engine adds the header row and writes `Total`;
- classic layout beside page fields: the engine places the page area above the grown location as usual;
- a `subtotalCaption` on a field with the Values field inside it (the engine keeps `<item> <data field>`), and an
  empty `subtotalCaption=""` (the engine writes an empty label), and a caption with several `?` (the engine
  replaces each with the item).
- whether desktop Excel also reads a `?` in `subtotalCaption` as the item (t4745 SQ shows the web does;
  MS-OI29500 says the field name, so the two may differ). This needs desktop Excel: put it in M14, Phase 4
  (decision 0039) if no desktop backend is available.
Write `probes/t4785-...` with the case builder and fill it with the excel-graph oracle.

## Acceptance criteria
- [ ] A filled probe case covers each reading above; pivots.md records the result with the evidence cells.
- [ ] The engine's rendered cells and geometry pass the probe in unit tests against the sidecar
      (crates/rustsheet-engine/src/pivot/layout/render/tests.rs, lines/tests.rs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Classic layout's UI-only drop-zone behaviour.
