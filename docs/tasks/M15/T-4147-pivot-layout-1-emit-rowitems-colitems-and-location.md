---
id: T-4147
title: "Pivot layout 1: emit rowItems/colItems and location for the three forms"
milestone: M15
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4142, T-4138]
---

## Goal
From a `PivotResult`, the engine emits Excel's rowItems/colItems (`<i t r i><x v>`) and location attributes for compact, outline and tabular form, subtotals top or bottom, grand totals on or off, and the Values field at any axis position.

## Context
SPEC-0030's contract; scored structurally by task `scoring` against the oracle and the fresh pivot-corpus files (LibreOffice xlsx/pivot/Pivot*.xlsx target these variants). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] rowItems/colItems equal Excel's on the spike, geometry probes and fresh corpus pivots in scope.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells (task `layout-cells`).
