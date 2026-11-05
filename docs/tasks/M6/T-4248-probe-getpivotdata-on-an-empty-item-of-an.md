---
id: T-4248
title: "Probe GETPIVOTDATA on an empty item of an ungrouped pivot field"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence on what GETPIVOTDATA answers for an item of an ungrouped pivot field that holds no record, so the engine can
match it.

## Context
- Follow-up of T-4235: on a range-grouped field, a group with no record is not shown (`showAll="0"`) and GETPIVOTDATA
  on its label is `#REF!` (probes/t4228-pivot-range-groups Calc B3, R1; docs/excel-semantics/web.md "Grouped pivot
  fields import as their group items"). The engine applies that only to range-grouped fields
  (crates/rustsheet-engine/src/functions/lookup_and_reference/pivot_index.rs `shown_group`); an ungrouped field's item
  with no record answers 0 as an empty intersection does (probes/getpivotdata-empty-intersection), unprobed.
- An ungrouped cache keeps an item with no record after a refresh that dropped its last record (`missingItemsLimit`),
  or when its `<sharedItems>` lists a value no record uses. A probe needs an xlsx-cache case (a `<sharedItems>` item
  no record points at, with `showAll="0"` and with `showAll="1"`), since a spec pivot's items come from its records.
- Use the `excel-probe` skill; record the finding in web.md.

## Acceptance criteria
- [ ] A probe case answers GETPIVOTDATA on a recordless item of an ungrouped field, with `showAll` 0 and 1, filled by
  excel-graph; web.md records the rule.
- [ ] If the rule differs from today's (0), the owning engine task is filed or the change is made.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date groups (T-4236).

## Outcome
- Probe `probes/t4248-getpivotdata-recordless-item` (excel-graph filled). Excel: recordless item of an ungrouped field is
  `#REF!` with `showAll="0"` and `0` with `showAll="1"` (case-insensitive); recorded in web.md.
- Builder (`tools/src/rustsheet_tools/pivot.py`) gained `unused_items` and `show_all`; `PivotTableSpec` the same keys.
- Differs from today's engine for `showAll="1"` only: filed T-4276 (owner of P!J1, P!J5). Eval: new probe cells 8 of 10
  pass; the 2 off are owned by T-4276.
