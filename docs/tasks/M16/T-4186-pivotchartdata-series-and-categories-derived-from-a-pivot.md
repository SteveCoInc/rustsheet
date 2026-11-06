---
id: T-4186
title: "PivotChartData: series and categories derived from a pivot"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4149, T-4130]
---

## Goal
The engine exposes, for a PivotChart, its categories (multi-level from nested row fields), series (column tuples x data fields) and values (visible, show-as applied, no subtotals or grand totals), checked against Excel-written chart caches.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §7; POI WithChartSheet.xlsx and LibreOffice forum-mso-de-104083.xlsx hold Excel caches. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Matches the corpus chart caches; probes if the web rewrites caches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Drawing charts.
