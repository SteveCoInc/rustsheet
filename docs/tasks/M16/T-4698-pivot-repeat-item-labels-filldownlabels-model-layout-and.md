---
id: T-4698
title: "Pivot repeat item labels (fillDownLabels): model, layout and render"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4168]
---

## Goal
A pivot with Repeat Item Labels renders an outer item's label on every line of its children, as Excel does, in tabular and outline form.

## Context
Split from T-4168. "Repeat All Item Labels" is saved in the definition's extension list (`x14:pivotTableDefinition@fillDownLabelsDefault`, and per field `x14:pivotField@fillDownLabels` in the pivotField's `extLst`), which the model keeps only as opaque XML (`PivotTable::opaque`, crates/rustsheet-engine/src/pivot/model.rs; the importer crates/rustsheet-xlsx/src/import/pivots/table.rs). The renderer (crates/rustsheet-engine/src/pivot/layout/render.rs `Cells::lines`) writes an outer label on its first line only (pivots.md "Rendered pivot cells", t4123 T!A6). No probe covers the option: write `probes/t4698-...` with the case builder's raw `xml.extLst` slot (tools/src/rustsheet_tools/pivot_definition.py; it has no per-field extLst slot yet, so add one if Excel needs the field attribute) and fill it with the excel-graph oracle. Whether the lines (`rowItems`) change is part of the question.

T-4129 evidence (probes/t4129-pivot-layout-flags, excel-graph, 2026-11-06; pivots.md "Layout and totals flags as Excel
for the web renders them"). The case builder now has a per-field `xml.extLst` slot.
- X14F: tabular, with Region's `x14:pivotField fillDownLabels="1"`. `East` repeats on every East product row (A5:A6),
  and `West` on A9.
- X14: the table's `x14:pivotTableDefinition fillDownLabelsDefault="1"` (with `hideValuesRow="1"`). It fills nothing
  down for existing fields; the cells match the plain tabular render.

Owner of these visibly-off pivot cells (decision 0245):
- probes/t4129-pivot-layout-flags X14F!A5:A6, X14F!A9

## Acceptance criteria
- [ ] A filled probe case covers repeat labels in tabular and outline form, table-wide and per field, and pivots.md records the result with the evidence cells.
- [ ] The model reads the flag(s) typed (round trip kept), and the render passes the probe in unit tests against the sidecar.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Styles and indentation.
