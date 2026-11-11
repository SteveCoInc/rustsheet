---
id: T-5057
title: "Probe which cache items Excel's on-open refresh of a refreshOnLoad pivot drops"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/pivots.md states, from committed excel-graph evidence, which shared items Excel keeps and which it drops when it refreshes a `refreshOnLoad` cache on open.

## Context
In T-4972's pivot-on-open sidecars (eval/suites/pivot-on-open, TemplateWithTableSourcePivotTables.xlsx), Excel rebuilt three caches from their one-row Table sources on open. Some fields' shared items shrank to the record's single `<m/>` (SchemeId went from 55 items to 1, CurveDate from 1543 to 1), while other fields of the same cache kept theirs (CurveType 3, ShockBond 22). The line indices were renumbered to match. Only one of the caches has `missingItemsLimit="0"`. pivots.md "Item casing when cache and source disagree" (Saved order) saw a refresh after a source edit keep the stale items. See pivots.md "Excel's on-open render of the corpus refreshOnLoad pivots is their saved render (T-4972)".

Probe cases (excel-probe skill) should vary `missingItemsLimit`, whether a field is on an axis, its `<items>` and the saved `count`, and compare the sidecar's `cache_fields` with the input's.

## Acceptance criteria
- [ ] Probe sidecars show which items an on-open refresh keeps, for each varied attribute.
- [ ] The pivots.md entry states the rule as VERIFIED, or names what stays open.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making the engine's refresh drop items (a follow-up engine task, once the rule is known).
