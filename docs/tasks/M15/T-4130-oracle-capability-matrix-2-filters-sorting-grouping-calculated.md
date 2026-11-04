---
id: T-4130
title: "Oracle capability matrix 2: filters, sorting, grouping, calculated fields/items, slicers, timelines"
milestone: M15
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4122, T-4125]
---

## Goal
The same matrix for the remaining families, plus three questions: whether the Auto Refresh setting is stored in the file and honoured by Excel for the web, whether the web rewrites PivotChart caches (c:numCache/strCache) when it re-renders a pivot, and what GETPIVOTDATA returns against a pivot collapsed to `#SPILL!`.

## Context
excel-docs.md (AUTOREFRESH, PSPILL); plan §7 and §10. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] `web` filled for every row of these families, each citing a probe.
- [ ] The three questions answered in docs/excel-semantics/pivots.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any of it.
