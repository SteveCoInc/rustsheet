---
id: T-4402
title: "PIVOTBY relative_to 3 and 4 on intransitive runs: the parent set is the parent subtotal's records, and an empty one is an error"
milestone: M7
lane: engine
status: in-progress
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
- [ ] Every probes/t4382-pivotby-intransitive-paths cell named in Context passes, or the misses are recorded in
      coercion.md and owned.
- [ ] The empty-parent errors are confirmed on a hold-out (PERCENTOF and a two-argument LAMBDA, both axes).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
