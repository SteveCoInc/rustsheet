---
id: T-4168
title: "Pivot layout options: custom subtotals, items with no data, repeat labels, blank rows, collapse"
milestone: M15
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4149]
---

## Goal
The layout honours custom subtotal functions (several per field), show items with no data, repeat item labels, a blank row after each item, per-field form mixes, and collapsed items (`sd="0"`).

## Context
Evidence from cap-layout and the geometry probes; probes added here where missing. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probes for each option pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Styles.
