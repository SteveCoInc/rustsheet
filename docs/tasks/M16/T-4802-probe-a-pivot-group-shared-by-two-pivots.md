---
id: T-4802
title: "Probe a pivot group shared by two pivots of one cache, and the grouping shapes T-4170 left open"
milestone: M16
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4170]
---

## Goal
Excel for the web evidence for the grouping shapes T-4170 did not probe, so T-4171 implements none of them from
memory: one discrete or date-level group seen by two pivots of one cache, and the edge shapes listed below.

## Context
- T-4170 split to stay size M (its Outcome). It taught case builder v2 discrete (`discrete`) and date-level
  (`date_group`) groups (tools/src/rustsheet_tools/pivot_groups.py, the Rust mirror
  crates/rustsheet-eval/src/pivot_definition/groups.rs) and probed them in probes/t4170-pivot-manual-groups and
  probes/t4170-pivot-date-levels (docs/excel-semantics/pivots.md "Manual groups and date levels as Excel for the web
  renders them").
- The builder gives every pivot its own cache (`_defined` in pivot.py, `parts` in the Rust mirror). A shared group
  needs a definition key that puts a second pivot on an earlier pivot's cache (one `pivotCacheDefinition`, two
  `pivotTableDefinition`s with the same `cacheId`), in both builders with golden parts. Excel-authored evidence of the
  shape: ClosedXML's pivot-table/shared-group-field.xlsx and shared-nested-dategroup.xlsx (Excel 15.0300), two
  pivots on one grouped cache. Questions: does each pivot show the group; does a collapsed or hidden group item in one
  pivot (`sd="0"`, `h="1"`) change the other; GETPIVOTDATA against each.
- Shapes T-4170 left UNVERIFIED (pivots.md entry above): GETPIVOTDATA on an item of an ungrouped field whose
  subtotals are custom only (T-4170 saw `#REF!` on grouped fields, MS and DS); a group renamed through its pivotField
  item's `n` attribute rather than its `<groupItems>` text; a discrete group over numbers, dates, a blank or an error
  item; a group field on the columns or the pages; a date group with days, hours, minutes or seconds beside other levels, or levels that skip one
  (years and months only); the base field off the axes with only added levels shown; the 48357 shape whose in-place
  field has no `par`; `showAll="1"` on an added level.
- What Excel saves for a group's `par`, `base` and `discretePr`: T-4170 saw them kept as written only in the oracle's
  cached downloads (pivots.md entry above). A sidecar records group items and shared-item flags but not these, so a
  probe needs the snapshot's `PivotCacheField` (crates/rustsheet-eval/src/xlsx_snapshot/pivots.rs) to keep them, or
  another committed record of the saved parts.

## Acceptance criteria
- [ ] Builder v2 (Python and the Rust mirror, same refusals, goldens) puts two pivots on one cache.
- [ ] Probe cases with sidecars for the shared group, the saved `par`, `base` and `discretePr`, and the shapes above; entries in docs/excel-semantics/pivots.md;
  owners for every visibly-off cell (decision 0245).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4171).
