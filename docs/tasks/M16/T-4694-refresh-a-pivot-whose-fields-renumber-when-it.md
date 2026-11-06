---
id: T-4694
title: "Refresh a pivot whose fields renumber when it keeps verbatim parts"
milestone: M16
lane: engine
status: done
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

Cells this task owned (all pass now, see Outcome): probes/t4585-pivot-field-names N1!A3:B6;
probes/t4627-pivot-field-name-collisions P_SufA, P_SufB, P_FldA, P_FldB, P_Off, P_Empty, P_Const,
P_Space (each A3:B6) and P_Fold!A3:B9.

## Acceptance criteria
- [x] A refresh that renames fields renders the pivot when its verbatim parts hold no field index;
  entries owned by a field follow its new index; one that holds field indices still refuses.
- [x] The cells above pass; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Remapping field indices inside verbatim `formats`, `filters` or layout lines.

## Outcome
`remap` (crates/rustsheet-engine/src/pivot/grid.rs) now refuses a renumbering with
`FieldsMoved` only when a verbatim part holds a field index (`OpaqueEntry::holds_field_index`,
model.rs), and otherwise moves each entry a field owns (`OpaqueOwner::Field`, `Items`, `Item`;
`OpaqueOwner::field`/`with_field`) to its field's new index, dropping a dropped field's entries.
A part holds a field index when one of its attributes is named `field` or `fld` or ends in either,
ignoring case and prefix, read by a single linear pass that skips quoted values and text between
tags. Per ECMA-376 that catches `pivotArea@field` and `reference@field` (`formats`,
`conditionalFormats`, `chartFormats`, `autoSortScope`), `filter@fld`/`@mpFld`/`@iMeasureFld`,
`mp@field` (`pivotHierarchies`), and extension names such as `x14:sourceField`; an unknown
attribute so named refuses too. `rowItems`/`colItems` (`x@v` item indices of the axis's n-th field,
`i@i` data field indices), `pivotTableStyleInfo`, the root's and fields' attributes
(`pivotField@dataField` is a flag, the one exception the rule makes, for a field's own attributes)
hold none. pivots.md "The grid writer's own readings" says so.

Tests: `which_unread_parts_hold_field_indices` and `field_indices_follow_their_names` pin the
classification and the owner remap (refusing filters, formats, an `autoSortScope` and a
`sourceField`); `a_renamed_field_renders_the_pivot_under_its_unread_parts` builds the ten pivots as
the case builder saves them (`probe_cases::builder_cache`: header-text or `cache_names` field
names, formula cells not cached) with the builder's verbatim parts, refreshes them through
`plan_pivot_refresh` and compares every captured cell, the location and the geometry against the
t4585 and t4627 sidecars. `make eval-baseline`: probes known failures 6518 -> 6432 (fixed 86:
exactly the 86 cells above), REGRESSIONS 0.
