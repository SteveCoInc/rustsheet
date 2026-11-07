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
`#DIV/0!`); and the same modes with two data fields or with Values on rows.

## Context
T-4161's probes (probes/t4161-pivot-show-as-*) settled the visible-item, single-data-field cases; the
points above are marked UNVERIFIED in docs/excel-semantics/pivots.md "Show-values-as edge cases". T-4163
implements the modes.

## Acceptance criteria
- [ ] Probe cases with sidecars; the UNVERIFIED points in pivots.md are verified or re-scoped.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4163).
