---
id: T-4128
title: "Commit the pivot spike observations as oracle probes"
milestone: M16
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-4122, T-4123]
---

## Goal
Each spike observation in oracle-spike.md is a committed probe with an excel-graph sidecar and a docs/excel-semantics/pivots.md entry: refreshOnLoad re-render with stale cells (closes T-2327), tabular two-field layout with subtotals and percentOfTotal, calculated field on sums, Top-N count filter changing subtotals, descending label sort, numeric group labels (fractional and integer data), and the `#SPILL!` anchor when the layout meets a foreign cell.

## Context
oracle-spike.md (2026-11-04) holds the runs; they are observations until committed (directive 1). T-2327 asked the re-render question; mark it done with this task's outcome. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases under eval/suites/probes/cases/pivot-*.case.json with sidecars.
- [ ] docs/excel-semantics/pivots.md created with these rules and their evidence; web.md's T-2327 UNVERIFIED note resolved.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
