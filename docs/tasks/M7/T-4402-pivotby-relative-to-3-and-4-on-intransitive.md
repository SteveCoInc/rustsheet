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
- PIVOTBY's relative sets (relative_to.rs) now come from the classes each record reached, not each record's group
  (`AxisSets::class`, `Grouping::reached_class`). A cell's set is the records of its row item's class that are in its
  column item's class. A class's records are those whose descent reached it, or every record when none did, as a
  subtotal cell's are. While `=` is transitive these are the records of the item's groups, so nothing else changes.
- A two-argument LAMBDA given an empty relative set is `#VALUE!` (group_functions.rs). PERCENTOF's zero total
  already gave `#DIV/0!`.
- Every probes/t4382-pivotby-intransitive-paths cell named in Context passes. A unit test covers Calc!D10
  (`parent_column_total_is_the_parent_subtotal_s_records`).
- Hold-out probes/t4402-pivotby-parent-set-holdout (excel-graph, 2026-11-05) from docs/research/t4402/gen_probe.py:
  - 24 fresh seeded runs (seed "T-4402"): 12 with an empty parent set, 12 other intransitive runs.
  - PERCENTOF and the two-argument LAMBDA at relative_to 3 and 4, 96 cells.
  - The engine's values, with PR #3430 (T-4388) merged in, were committed in 353a65823 before the fill.
  - They matched 96 of 96. Main's bucketing matched 18.
  - The new case passes in full on this branch alone, without #3430.
  - This is a structural rule confirmed out of sample (decision 0188 R4), not a fit. Recorded in coercion.md.
- `docs/research/t4344/predict_engine.py` takes the case and output path as arguments. This is the same edit as
  #3430's, so the two merge cleanly, and this branch adds no copied wrapper.
- Eval delta: probes known failures 4911 -> 4881 (30 fixed, all in t4382). New case: 96 of 96 pass. No regressions.
