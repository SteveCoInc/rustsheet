---
id: T-4698
title: "Pivot repeat item labels (fillDownLabels): model, layout and render"
milestone: M16
lane: engine
status: done
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

## Acceptance criteria
- [x] A filled probe case covers repeat labels in tabular and outline form, table-wide and per field, and pivots.md records the result with the evidence cells.
- [x] The model reads the flag(s) typed (round trip kept), and the render passes the probe in unit tests against the sidecar.
- [x] `make check` green; reviewers approve.

## Out of scope
- Styles and indentation.

## Outcome
Done (2026-11-07). New probe probes/t4698-pivot-repeat-labels (excel-graph, 17 pivots) records the
findings in pivots.md "Repeat item labels":
- A field's x14 `fillDownLabels` repeats its label on every line of its children, inner
  subtotal lines included (T3R!A6). This holds in tabular and outline form (OST, OB, O3) and on
  columns (CC!B4:D4).
- It is per field, not inherited (T3P), and does nothing on the innermost field (TIN).
- Blank lines, collapsed lines and the field's own subtotal and grand lines are unchanged.
- Compact form fills nothing (CF). A cell belongs to the deepest level placed in it, as MX!A6
  shows.
- `(blank)` repeats (TBL).
- The table-wide `fillDownLabelsDefault` fills no existing field (TW, TWE, OW).
- `rowItems` and `colItems` do not change.

Changes:
- The importer reads the flag typed into `PivotField::fill_down_labels` from the field's kept
  `extLst`, through the one x14 extension reader (`read_extension`, formerly
  `read_data_extension` for `pivotShowAs`). The extension stays verbatim for export.
- `Cells::fill_down` renders it.
- The eval adapter already wrote the field's `extLst`, so it needed no change.
- Unit tests compare all 17 pivots and t4129 X14/X14F against the sidecars.

Eval: probes known failures went from 6301 to 6298 (t4129 X14F!A5, A6, A9 fixed). There were
0 regressions, and the new case passes in full.

The baseline's new `C` and `TC` lines for probes/t4698-pivot-repeat-labels are the new case's
generated count lines (its cells, all passing), not known failures. Unprobed readings are filed as
T-4764. `make check-fast` is green locally. In review, security and fidelity approved, and
the changes design requested are applied: one x14 extension walker, and `flag` for the boolean.

