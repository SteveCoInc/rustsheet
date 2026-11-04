---
id: T-4129
title: "Oracle capability matrix 1: layout, totals, aggregates and show-values-as rows"
milestone: M15
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4122, T-4125]
---

## Goal
For every catalog row in the layout, totals, aggregation and show-values-as families, a minimal probe records whether Excel for the web opens, refuses, repairs or renders it, in the catalog's `web` column.

## Context
Graph cannot create pivots; Excel for the web renders authored definitions (oracle-spike.md). The Excel 2010+ show-as modes live in x14 extensions: whether the web honours them is the first question (excel-docs.md). Rows the web cannot render go to M14 (decision 0039) and never block M15. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] `web` filled for every row of these families, each citing a probe.
- [ ] Refused rows have M14 tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Edge-pool coverage (later probe tasks).
