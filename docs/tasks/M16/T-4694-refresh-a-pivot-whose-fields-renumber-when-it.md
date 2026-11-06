---
id: T-4694
title: "Refresh a pivot whose fields renumber when it keeps verbatim parts"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A pivot loaded from a file refreshes when the refresh renames or renumbers its cache fields, as
Excel's refresh on open does for repeated, blank or formula headers, instead of keeping its saved
cells.

## Context
`remap` in crates/rustsheet-engine/src/pivot/grid.rs refuses with `PivotRefreshError::FieldsMoved`
whenever the field map is not the identity and the pivot keeps any verbatim part
(`PivotTable::opaque`). Every file Excel or the case builder saves keeps some (the root's version
attributes, `pivotTableStyleInfo`, field attributes), so any header the refresh renames (pivots.md
"Pivot field names for duplicate and blank source headers") leaves the pivot unrendered. Only the
verbatim parts that hold field indices (`formats`, `conditionalFormats`, `chartFormats`, `filters`,
`rowItems`/`colItems`, ...) block a renumbering; the entries owned by a field (`OpaqueOwner::Field`,
`Items`, `Item`) can follow it. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; moved from T-4152, whose adapter loads the
  case builder's parts, where the refresh fails with `FieldsMoved`): probes/t4585-pivot-field-names
  N1!A3:B6; probes/t4627-pivot-field-name-collisions P_SufA!A3:B6, P_SufB!A3:B6, P_FldA!A3:B6,
  P_FldB!A3:B6, P_Off!A3:B6, P_Empty!A3:B6, P_Const!A3:B6, P_Space!A3:B6, P_Fold!A3:B9.

## Acceptance criteria
- [ ] A refresh that renames fields renders the pivot when its verbatim parts hold no field index;
  entries owned by a field follow its new index; one that holds field indices still refuses.
- [ ] The cells above pass; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Remapping field indices inside verbatim `formats`, `filters` or layout lines.
