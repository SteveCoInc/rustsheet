---
id: T-4161
title: "Probe all fourteen show-values-as modes"
milestone: M16
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4129]
---

## Goal
Evidence for % of grand/column/row total, % of, % of parent row/column/total, difference from, % difference from, running total in, % running total in, rank ascending and descending, and index: base item (previous, next, missing gives #N/A?), ties in rank, running totals across outer items, the values shown in subtotals and grand totals, and division by zero.

## Context
Engines disagree on rank (oss-engines.md §0.4); no corpus covers these modes. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Probe cases with sidecars; pivots.md entries per mode.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
Six probe cases, all filled by excel-graph on 2026-11-07: probes/t4161-pivot-show-as-base-items,
-running-rank, -zeros, -edges, -running-zero and -nested-parent. docs/excel-semantics/pivots.md "Show-values-as
edge cases (T-4161)" extends T-4129's entry mode by mode. Main findings:
- A base item missing from a parent gives `#N/A` on every row of that parent.
- (previous) and (next) do not cross outer items.
- An empty value is `#NULL!` under percent and percentDiff, even over an empty base. A present value over
  an empty base is empty.
- Difference counts empties as 0.
- Ranks are dense.
- Running totals run across outer items when the base field is the outer one.
- Division by zero is `#DIV/0!` for percent, percentDiff, the % of row/column/total modes and index. It is
  an empty cell for the % of parent modes. Under % running total it is empty only where the cell's own
  value is 0.
- A `baseItem` past the end of its field makes Excel for the web refuse the file (recorded in "Pivot flags
  Excel for the web refuses").

Still open, UNVERIFIED and owned by T-4750:
- hidden or sorted base items;
- an empty value over a base of 0;
- two data fields.

The off cells are owned by T-4163.
