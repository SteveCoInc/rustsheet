---
id: T-4627
title: "Probe pivot field-name suffix collisions"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
pivots.md settles the open point of "Pivot field names for duplicate and blank source headers": how Excel resolves a suffixed name (`Sales2`) or a `Field<n>` name that collides with a real header.

## Context
T-4585 (probes/t4585-pivot-field-names) shows `Sales2`, `Sales3`, `region2` and `Field4`. Cases:
- a source with `Sales`, `Sales2` and a second `Sales`;
- a blank header in column 4 beside a real `Field4` header.

Use the same formula-header trick: the case builder refuses duplicate or blank headers, so `=""` and `="Sales"` get past it. T-4141 implements the rule.

## Acceptance criteria
- [ ] Filled probe for both cases; the pivots.md entry's UNVERIFIED part closed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine refresh (T-4141).
