---
id: T-4750
title: "Probe show-values-as base items over hidden, sorted and zero bases"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4161]
---

## Goal
Excel for the web evidence for the show-values-as points T-4161 left open: (previous) and (next) when the base
field has hidden items or a custom sort (does the base skip hidden items, and does it follow display order
or cache order?); % of and % difference from where the value is empty and the base is 0 (`#NULL!` or
`#DIV/0!`); % running total in where the value is empty and the total is 0; whether Excel for
the web refuses every `baseItem` past its field's last item (only `baseItem="9"` on a two-item field is
evidenced); and the same modes with two data fields or with Values on rows.

Also the engine's own readings T-4163 implemented where no probe shows Excel's (pivots.md "Show-values-as
edge cases (T-4161)", the engine paragraph): a base field on no axis, a `baseItem` that names no item
(`#N/A`), errors in values, bases and denominators, a difference over an existing base item with a missing
combination, an empty value over a percentOfRunningTotal total of 0 (empty), a `showAll` line with no record,
`baseItem` as an `<items>` index versus a cache index, a data field with both `showDataAs` and an x14
`pivotShowAs` (the x14 mode wins), -0 shown as 0, a custom subtotal line under a shown field, and GETPIVOTDATA
against a shown field (today `#REF!`). The order of index's operations is T-4754's.

## Context
T-4161's probes (probes/t4161-pivot-show-as-*) settled the visible-item, single-data-field cases; the
points above are marked UNVERIFIED in docs/excel-semantics/pivots.md "Show-values-as edge cases". T-4163
implements the modes.

## Acceptance criteria
- [ ] Probe cases with sidecars; the UNVERIFIED points in pivots.md are verified or re-scoped.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4163).
