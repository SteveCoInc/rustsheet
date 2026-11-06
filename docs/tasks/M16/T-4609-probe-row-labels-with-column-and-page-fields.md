---
id: T-4609
title: "Probe Row Labels with column and page fields, and compact versus outline on an outer row field"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md settles three open parts of the tabular Row Labels rule:
- whether a column or page field left at the default `compact` also gives `Row Labels`;
- whether `outline` alone matters on an axis field;
- which of `compact` and `outline` on the outer row field moves its subtotals onto the item line (T-4583 R1).

## Context
T-4583 (probes/t4583-pivot-row-labels R1–R6) covered row, data, unused and calculated fields. R1 left both attributes at the default on the outer row field, so it cannot tell `compact` from `outline` (Excel-fidelity review of T-4583). Variants: a column field at the default; a page field at the default; an outer row field with `compact=0` and outline at the default, and the reverse. Case builder v2 with excel-graph.

## Acceptance criteria
- [ ] Filled probe with the four variants; the pivots.md entry's UNVERIFIED part is closed or corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine rendering (T-4149).
