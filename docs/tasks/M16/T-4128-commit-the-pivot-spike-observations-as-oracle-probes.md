---
id: T-4128
title: "Commit the pivot spike observations as oracle probes"
milestone: M16
lane: research
status: done
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
- [x] Probe cases under eval/suites/probes/cases/t4128-pivot-spike.case.json with sidecars.
- [x] docs/excel-semantics/pivots.md created with these rules and their evidence; web.md's T-2327 UNVERIFIED note resolved.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4128-pivot-spike (excel-graph, 2026-11-06) commits the spike observations as builder-v2
pivots, captured in the sidecar's `pivots` section:
- **R:** refresh on open over a stale cache (East/A 1004). Closes T-2327.
- **K:** a calculated field on sums (28.1), a Top-N filter that changes the subtotals (East Total 19),
  and a descending label sort.
- **G:** Excel's numeric group labels.
- **S:** a layout meeting a constant collapses to `#SPILL!` at the anchor, with location A3.

The tabular two-field layout with percentOfTotal was already in probes/t4123-pivot-capture.
docs/excel-semantics/pivots.md moves these entries to VERIFIED, and web.md's T-2327 note is resolved.
K also shows that a second, compact data field keeps `Row Labels`, recorded for T-4137 and T-4138.
The case-variant, flag and item-order observations stay UNVERIFIED for T-4134, whose probe they are.
