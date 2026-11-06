---
id: T-4712
title: "Probe and implement how Excel remaps field indices in a renumbered pivot's formats, filters and sort scopes"
milestone: M16
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A `refreshOnLoad` pivot whose refresh renames or renumbers its cache fields renders as Excel renders
it when its definition keeps parts that hold field indices, and those indices are saved as Excel
saves them, instead of the refresh being refused and the pivot keeping its saved cells.

## Context
Since T-4694, `remap` (crates/rustsheet-engine/src/pivot/grid.rs) renumbers a pivot's fields under
verbatim parts that hold no field index, but refuses with `PivotRefreshError::FieldsMoved` when one
does (`OpaqueEntry::holds_field_index`, model.rs; docs/excel-semantics/pivots.md "The grid writer's
own readings"). What Excel does to those indices when the fields renumber on refresh is unprobed.
Probe it (`excel-probe` skill), one sheet per part, each over a source whose repeated, formula or
blank headers Excel renames (pivots.md "Pivot field names for duplicate and blank source headers")
or with a column inserted before the shown fields:
- `formats/format/pivotArea@field` and `references/reference@field`;
- `conditionalFormats/conditionalFormat/pivotAreas/pivotArea` (and the x14 extension's);
- `chartFormats/chartFormat/pivotArea` (a pivot chart over the pivot);
- `filters/filter@fld` (Top-N/value filters, whose `iMeasureFld` names a data field, and label
  filters), and `@mpFld`;
- a field's `autoSortScope/pivotArea` (sort by a value field);
- `pivotHierarchies` (`mps/mp@field`), if a worksheet-sourced pivot can hold any;
- outside the definition, the worksheet's `sheetView/pivotSelection/pivotArea@field` (the selected
  pivot cell), which also names fields by index.
Record what Excel saves (the sidecar's parts) and renders, then remap or drop each index as Excel
does, and narrow `holds_field_index` to what remains unmapped. Program:
docs/research/pivot-tables-2026-11-04/plan.md.

## Acceptance criteria
- [ ] A probe case pins, per part above, the indices Excel saves and the cells it renders after a
  renumbering refresh on open; the findings are in docs/excel-semantics/pivots.md.
- [ ] The refresh remaps (or drops) each probed part's field indices as Excel does and renders the
  pivot; a part still unprobed keeps refusing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Exporting the remapped parts (T-4176 writes the definition).
