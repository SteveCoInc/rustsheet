---
id: T-0495
title: "An edited workbook evaluates to what a fresh load of it does (calipers pv26_014, pv23_019)"
milestone: M11
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A workbook evaluated after a series of edits holds the same values as the same workbook loaded
fresh, with the same edits applied, and evaluated once.

## Context
Found while verifying T-0453, and the same on main before it: in calipers
pv_goldens/pv26_014 (and pv26_2_014), loaded, evaluated, then edited 25 times (every
numeric constant at an even stride multiplied by 1.5 and incremented, one `evaluate()` per edit),
sheet 6 row 59 columns I, J, L read `OK` where the fresh load with the same edits reads `0.0`; in
pv23_019 (and pv23_027) sheet 3 row 97 columns G, H, I read `$ -` against `$ 0`. The first
divergence appears by the fifth edit. All four workbooks hold data tables and set
`calcPr iterate="1"`, so iterative calculation (`iteration.rs`), the data tables
(`data_tables.rs`) or a value carried over between evaluations (`FormulaValue` of a formula cell,
a stale spill) are the suspects. Reproduce with a release example that loads the golden, applies
the edits through `set_user_input` and compares `get_formatted_cell_value` of every cell with a
fresh load; find which evaluation first differs and which cell starts it.

## Acceptance criteria
- [x] The divergence's cause is named in the Outcome, with the cell that starts it.
- [x] A unit test reproduces it in a small workbook and passes; the four workbooks agree with a
      fresh load after the same 25 edits, or the task records which Excel behavior (evidence)
      makes the history matter, in docs/excel-semantics/.
- [x] `make check` green; reviewers approve. (`make check-fast` locally; CI runs `make check`.)

## Out of scope
- Data-table memoization (T-0453): the divergence is the same without it.

## Outcome
- **Cause:** iterative calculation's stop, not data tables and not a stale value.
  - Each recalculation resumes a cycle from its previous values, and stops at the first sweep whose largest change
    is below iterateDelta (0.001). Both rules are VERIFIED in recalc.md.
  - So a converging cycle stops at a different point when it is reached by another path.
- **Where it starts:** in calipers pv_goldens/pv23_019 the first divergence is at edit 10.
  - The edits were 25 numeric constants, every other one in cell order, each multiplied by 1.5 and incremented,
    with one `evaluate()` per edit. The scratch example `crates/rustsheet-xlsx/examples/t0495.rs` was not
    committed.
  - The cell is `Company1153 3-Statement`!H123 `=H121-H64`. In the cycle through H64 (`=H$121`) and H121, the
    fresh load stops with H64 trailing H121 by about 3E-10, while after the edits the two are equal.
  - So H123 shows "$ -" (0) after the edits and "$ 0" (2.9E-10) in the fresh load.
- **Iteration was the only cause:** with iterateDelta 0 and the iteration count at its maximum, all 25 edits agree
  with a fresh load.
- **pv26_014:** this edit sequence shows no divergence on current main, under the default or the tight settings.
- **Recorded:** recalc.md, "A converged cycle's value depends on the values it started from", records that Excel's
  rules make the history matter. Excel's own values for such an edit sequence are unprobed, because the oracle does
  not replay edits between calculations.
- **Unit test:** `a_converged_cycle_depends_on_the_previous_values` (test_iteration.rs).
- **Also in this PR:** T-4965, the follow-up task from PR #3991's review (T-4939).
- **Eval delta:** none (no engine change).
