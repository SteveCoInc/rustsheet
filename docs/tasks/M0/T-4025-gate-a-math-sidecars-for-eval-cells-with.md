---
id: T-4025
title: "Gate A math sidecars for eval cells with matrix arguments (MDETERM, and MINVERSE/MMULT through INDEX)"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3686]
---

## Goal
`rustsheet-accuracy math` gives the probe suites' one-call MDETERM cells a math value. It does the same for
`INDEX(MINVERSE(...),r,c)` and `INDEX(MMULT(...),r,c)` cells, so R1(c) can count them inside the envelope.

## Context
- T-3686 added the `matrix` parameter kind and indexed definitions (`Definition.indexed`) for the Gate A tables.
  `eval_math.fit_arg` skips a matrix argument with the reason "a matrix argument". The fit lab's sweep reader
  (`fitlab/sweep.py` `_array` and range reads) returns array literals and ranges row-major and flat, so it loses
  their shape.
- `eval_math.single_call` matches only `=FUNC(args)`. An indexed definition's cell is `=INDEX(FUNC(args),r,c)`.
- `make eval-math-check` then expects the new cells in the probes and probes-numeric sidecars, so regenerate them
  (`rustsheet-accuracy math --suite ...`).

- Carry-over from T-3686's review (PR #3083, MINOR findings, the same module):
  - in `defs/matrix.py`, MMULT's `array1` and `array2` params cite `MDETERM_SOURCE`; give MMULT its own source string;
  - fold the near-identical `uniform`/`logged` closures of `_mmult_draws` into one helper that takes a matrix factory;
  - reword `_row_column`'s docstring: the two are INDEX's parameters, which the catalog-order test skips;
  - replace the `assert` type narrowing in `_index` with a TypeError, as `_exact` does;
  - `eval_math.fit_arg` tests the kind twice: give the array and matrix kinds separate branches once the
    matrix branch reads shapes.

## Acceptance criteria
- [ ] The sweep reader keeps the shape of an array literal or range for a `matrix` parameter, with tests.
- [ ] One-call MDETERM cells and INDEX-of-MINVERSE/MMULT cells in probes and probes-numeric have math values.
      Their sidecars are regenerated and `make eval-math-check` is green.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change.
