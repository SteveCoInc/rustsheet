---
id: T-4184
title: "Drill-down API: the records behind a pivot cell"
milestone: M16
lane: api
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4142, T-4178]
---

## Goal
The API returns the source records behind any value cell (Excel's Show Details content).

## Context
SPEC-0032. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Tests against probes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Creating the new sheet as Excel's UI does.
