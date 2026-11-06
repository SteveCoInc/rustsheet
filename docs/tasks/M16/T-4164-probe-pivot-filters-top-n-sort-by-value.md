---
id: T-4164
title: "Probe pivot filters, Top-N, sort by value and their totals"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4130]
---

## Goal
Evidence for label, value and date filters, Top/Bottom N by items/percent/sum with ties, several filters on one field and their evaluation order, sort by a value field (ties, errors), manual item order, and whether subtotals and grand totals count filtered-out items.

## Context
Spike run 5 (Top-N changes subtotals); libreoffice.md (ties included). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars; pivots.md entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
