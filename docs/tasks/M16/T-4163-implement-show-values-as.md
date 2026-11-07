---
id: T-4163
title: "Implement show-values-as"
milestone: M16
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4161, T-4149]
---

## Goal
PivotResult applies all fourteen show-values-as modes as the evidence says (leaves, subtotals, grand totals).

## Context
SPEC-0032; mog's three-pass dispatcher (oss-engines.md §1) is a design idea only. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

The owner paragraph (decision 0245) listed the visibly-off cells of probes/t4123-pivot-capture T to T4,
t4129-pivot-show-as(-x14) and the six t4161-pivot-show-as-* cases; every one of them now matches Excel, so
the list is gone.

## Acceptance criteria
- [x] p-showas probes pass.
- [x] `make check-fast` green locally; CI's `make check` and the reviewers are the PR's.

## Out of scope
- GETPIVOTDATA of show-as values unless evidenced.

## Outcome
`pivot::show_as` (crates/rustsheet-engine/src/pivot/show_as.rs) applies all fifteen modes (the ECMA nine and
the six x14 modes) as one post-pass over the values `compute` already made. Per data field with a mode it
computes every (row node, column node) cell, leaves, subtotals and grand totals alike, and stores the
non-empty ones. `PivotResult::value_as` reads them under the field's own function. Each mode family has one
structure:
- the base nodes of difference, percent and percentDiff ((previous), (next), a fixed item, `#N/A` for a base
  item missing from its parent, an empty cell for a missing combination);
- the runs of runTotal, percentOfRunningTotal and the dense ranks;
- the ancestor of percentOfParent;
- the parent of the two parent-row and parent-column modes;
- the totals of percentOfRow, percentOfCol, percentOfTotal and index.

index computes value × grand total / row total / column total, left to right. Only t4129 IX!B5 tells the
candidate orders apart, by 1 ulp, below the R4.4 web-versus-desktop floor, so the order is UNVERIFIED; T-4754
probes it with many order-discriminating cells.

The pass charges a work unit per cell, plus the run sorts, and counts each stored value against
`MAX_PIVOT_NODES`. The importer no longer skips a shown pivot, and it reads the x14 `pivotShowAs` from the
dataField extension it keeps verbatim. The export (T-4176) writes `showDataAs` only for the ECMA modes, since an
x14 mode is in that kept extension. GETPIVOTDATA on such a pivot stays `#REF!`.

Under the outer Region, percentOfRunningTotal divides a product row by that product's total over the
Regions; only subtotal rows use the Grand Total row (the corrected T-4161 reading, RR PTR).

Every cell of t4123 T to T4, t4129-pivot-show-as(-x14) and the six t4161 cases matches its sidecar
(`the_show_values_as_probes_cells_are_excels`, `the_capture_probes_cells_are_excels`). `make eval-baseline`:
probes known failures 8518 -> 6168 on main after T-4161 (fixed 3932, REGRESSIONS 0). The engine's own readings are UNVERIFIED in
pivots.md under T-4750, which now lists them.
