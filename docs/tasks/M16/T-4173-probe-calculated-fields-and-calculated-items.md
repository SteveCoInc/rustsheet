---
id: T-4173
title: "Probe calculated fields and calculated items"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4130]
---

## Goal
Evidence for calculated fields (on sums at every level, Count pivots, division by zero and other errors, references to non-value fields, constants, functions allowed and refused) and calculated items (formula over items per record, solve order, restrictions with grouping and several data fields).

## Context
Spike run 5; excel-docs.md quirks; [MS-OI29500] restrictions. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars; pivots.md entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
