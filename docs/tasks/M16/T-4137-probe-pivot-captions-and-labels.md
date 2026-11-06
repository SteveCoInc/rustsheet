---
id: T-4137
title: "Probe pivot captions and labels"
milestone: M16
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-4122, T-4123]
---

## Goal
Evidence for every caption Excel writes: Row Labels, Column Labels, Values, Grand Total, `<item> Total`, `Total <data field>`, `(blank)`, `(All)`, `(Multiple Items)`, the default data field name per aggregate (`Count of`, `StdDev of`...), duplicate data field names, and custom caption attributes (rowHeaderCaption, grandTotalCaption, dataCaption, missingCaption, errorCaption).

## Context
T-2327 also asked for the Count/Max/Min captions. gridcraft and mog claims in oss-engines.md. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars; pivots.md entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
