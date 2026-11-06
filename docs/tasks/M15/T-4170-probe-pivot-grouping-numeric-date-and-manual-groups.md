---
id: T-4170
title: "Probe the pivot grouping cases M6 left open: manual groups, nested date groups, shared groups"
milestone: M15
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4130]
---

## Goal
Evidence for the grouping shapes the M6 grouping work (T-4225..T-4324) did not probe: manual text groups (`<discretePr>`, `Group1` labels, ungrouped items beside them), several date levels on one field (Years + Quarters + Months together) and their layout, a group shared by two pivots of one cache, and grouping inside a tabular or outline layout with subtotals.

## Context
Rescoped 2026-11-06. M6 settled numeric range groups (record placement, labels, relabel order, GETPIVOTDATA number items) and single-level date groups, recorded in web.md "Grouped pivot fields import as their group items" with probes t4228-*, t4235-*, t4236-*, t4249-*, t4253-*, t4287-*, t4311-*, t4316-*; do not re-probe those. Use case builder v2 (`definition`, T-4122), extending it for `<discretePr>` and multi-level date groups. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars for each shape; entries in docs/excel-semantics/pivots.md (or web.md beside the M6 entry).
- [ ] Builder v2 writes manual and multi-level date groups (unit tests).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
- Numeric and single-level date groups (done in M6).
