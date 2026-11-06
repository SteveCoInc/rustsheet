---
id: T-4161
title: "Probe all fourteen show-values-as modes"
milestone: M16
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4129]
---

## Goal
Evidence for % of grand/column/row total, % of, % of parent row/column/total, difference from, % difference from, running total in, % running total in, rank ascending and descending, and index: base item (previous, next, missing gives #N/A?), ties in rank, running totals across outer items, the values shown in subtotals and grand totals, and division by zero.

## Context
Engines disagree on rank (oss-engines.md §0.4); no corpus covers these modes. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars; pivots.md entries per mode.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
