---
id: T-4402
title: "PIVOTBY relative_to 3 and 4 on intransitive runs: the parent set is the parent subtotal's records, and an empty one is an error"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PIVOTBY's relative_to 3 (parent column total) and 4 (parent row total) on an intransitive run give Excel's
values: the parent set is the parent subtotal's records by T-4401's rule, and an empty parent set gives
PERCENTOF `#DIV/0!` and a two-argument LAMBDA `#VALUE!`.

## Context
- From T-4382 (probes/t4382-pivotby-intransitive-paths, excel-graph, 2026-11-05; coercion.md "PIVOTBY's other
  paths on intransitive runs (T-4382)").
- `Grid::relative_of` (pivotby.rs) reads `RelativeSets::of_cell` (relative_to.rs), whose parent sets come from
  `Grouping::group_of`. Excel's parent column total of a cell is the records of the parent subtotal cell of the
  same row group (T-4378's reached-class rule): over Ay Ay Bx Cx Cy (q p p q q) the (`ab`,y) q cell's parent is
  `1, 4, 5`, ours `1, 5`. docs/research/t4382/score.py gives every leaf cell's parent set in Calc!D1:D15 with it.
- Parent row totals (relative_to 4) read the parent subtotal row's cell in the same column, the Total column
  included: over Ay Ax By Cx Bx (p p q p q) the (`ab`,x) Total cell's parent is `1, 2, 4`, ours `1, 2, 4, 5`
  (Calc!E17).
- Where the parent subtotal is empty (a column leaf no record reaches under an empty subtotal), Excel's
  PERCENTOF is `#DIV/0!` and the two-argument LAMBDA's cell `#VALUE!`; ours passes an empty set (`2, 4 / `,
  Calc!D7).
- Cells: probes/t4382-pivotby-intransitive-paths Calc!C2, C4, C5, C7, C12, C14, D2, D4, D5, D7, D10, D11, D12, D14,
  D17, D18, D19, D22, D23, D24, D25, D27, E17, E18, E19, E22, E23, E24, E25, E27 (D16, D21, D29, D30, E16, E21,
  E29 and E30 pass since T-4359).

## Acceptance criteria
- [x] Every probes/t4382-pivotby-intransitive-paths cell named in Context passes, or the misses are recorded in
      coercion.md and owned.
- [x] The empty-parent errors are confirmed on a hold-out (PERCENTOF and a two-argument LAMBDA, both axes).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `relative_to` 3 and 4 now read each cell's set from its parent's cell: `ParentSets` (relative_to.rs) builds,
  per row line, the records of the parent column item's cell in the row (3), or of the cell of the parent row
  item in the column (4), by the grid's own rules (`Grid::column_records` in pivotby.rs, which shares
  `Grid::reached_class` with `Grid::cell_rows`; row parents by `Grouping::subtotal_rows`). Sets are shared per
  parent (a `SetId::ParentCell` each, so PERCENTOF sums a parent once), the latest parent per row level kept;
  the `MAX_SET_ENTRIES` check (`check_set_entries`) still runs first. `relative_to` 0 to 2 keep `RelativeSets`.
- A two-argument function on an empty set is `#VALUE!` (`Model::aggregate_result`); PERCENTOF's `#DIV/0!` was
  already its total of 0.
- Eval: probes/t4382-pivotby-intransitive-paths 780 -> 810 of 819 (all 30 cells named in Context; the 9 left,
  B31:B33, B35, B39, B41, C31, C33, C39, are T-4401's/T-4388's). New hold-out
  probes/t4402-pivotby-parent-sets-holdout (excel-graph, 2026-11-05; generator and predictions in
  docs/research/t4402, predictions committed before the fill): 64 of 64 formulas, 544 of 544 cells, 36 of
  them the empty-parent errors on both axes (PERCENTOF `#DIV/0!`, LAMBDA `#VALUE!`). Not a numeric fit: the
  rule is a structural model chosen on T-4382's cells and checked here on fresh runs.
- coercion.md "PIVOTBY's other paths on intransitive runs (T-4382)" records the hold-out and the implementation.
