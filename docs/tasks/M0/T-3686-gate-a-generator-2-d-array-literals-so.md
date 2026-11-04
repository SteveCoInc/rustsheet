---
id: T-3686
title: "Gate A generator: 2-D array literals, so MDETERM, MINVERSE and MMULT get tables in place of their composite exemptions"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3551]
---

## Goal
The Gate A generator writes 2-D array literals (`{a,b;c,d}`), and MDETERM, MINVERSE and MMULT get oracle-filled `s1`
tables in place of their `composite` exemptions.

## Context
- Decision 0228 (T-3551) exempts MDETERM, MINVERSE and MMULT as `composite` only because `literal.Arg`'s array kind
  (T-3586) writes one row. They have real floating-point kernels (arrays.md "MDETERM: partial-pivot LU, no singularity
  cutoff", "MINVERSE: singularity rule"), so a table is the better evidence.
- MINVERSE and MMULT return arrays: a table point is one cell, so read one element (for example through INDEX) and say
  how in the definition; the generator writes `=FUNC(args)` today, so this needs a small extension too.
- Removing the three `exempt.tsv` lines changes a protected path: a decision record is needed.

## Acceptance criteria
- [x] `literal.Arg` and `parse_formula` handle 2-D array literals, with tests; existing tables regenerate byte-identical.
- [x] Definitions and filled `s1` tables for MDETERM, MINVERSE and MMULT; their `exempt.tsv` lines are removed under a
      decision record; `rustsheet-eval accuracy` stays at 0 failing with each failing point owned.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change.

## Outcome
- The generator writes 2-D array literals. `literal.Arg` gains `Matrix` (a tuple of rows, written `{a,b;c,d}`) and
  `model.Kind` gains `matrix`. `Sampler.finish`, `math_arg` and `parse_formula` read it, and a one-row `array`
  parameter refuses a matrix. One-row arrays are written as before.
- `Definition.indexed`: the last two parameters are INDEX's row and column, and the cell is
  `=INDEX(FUNC(args),row,column)`. `formula` and `parse_formula` write and read it. `formula` now takes the
  definition, not the function name.
- `defs/matrix.py`: MDETERM, MINVERSE and MMULT. The math is the exact rational determinant, inverse and product.
  Domain errors are only the evidenced ones (arrays.md): a pivot product past the double range is MDETERM `#NUM!`; an
  exactly-zero pivot (`{a,b;2a,2b}`) or an inverse element past the range is MINVERSE `#NUM!`. A non-conformable
  MMULT, an INDEX outside the result and a product past the range are not claimed and never drawn. Tests:
  `tools/tests/test_accuracy_matrix.py`.
- Tables `s1` (excel-graph, 2026-11-04): MDETERM 120, MINVERSE 200 and MMULT 200 points. All are in the envelope, with
  0 failing and 0 table defects. Excel is more than 1e-9 off the math on 1 MDETERM and 14 MINVERSE points
  (ill-conditioned matrices), and the engine matches Excel's 15 digits there. arrays.md records it.
- `exempt.tsv` loses the three `composite` lines under decision 0262. `rustsheet-eval accuracy`: 13658 points in 93
  tables, 87 known failures, 0 failing, 0 table defects, missing tables 200 (203 before).
- Byte-identical: 83 of the 90 existing `s1` tables regenerate byte-identical on this branch. For the other 7
  (CHISQ.INV, CHISQ.INV.RT, GAMMA.INV, T.DIST, T.INV, T.INV.2T and TINV), origin/main's generator gives the same bytes
  as this branch's, but not the committed ones. That gap predates this change (an earlier tool version wrote them).
- Follow-up: T-4025 (eval-cell math sidecars for matrix arguments; until then `eval_math` skips them as "a matrix
  argument").
