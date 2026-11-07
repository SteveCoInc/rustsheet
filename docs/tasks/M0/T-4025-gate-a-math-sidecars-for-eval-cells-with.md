---
id: T-4025
title: "Gate A math sidecars for eval cells with matrix arguments (MDETERM, and MINVERSE/MMULT through INDEX)"
milestone: M0
lane: harness
status: done
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
- [x] The sweep reader keeps the shape of an array literal or range for a `matrix` parameter, with tests.
- [x] One-call MDETERM cells and INDEX-of-MINVERSE/MMULT cells in probes and probes-numeric have math values.
      Their sidecars are regenerated and `make eval-math-check` is green.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change.

## Outcome
- `fitlab/sweep.py`: `parse_rows` reads an array literal or a reference row by row (a `;` inside a string literal is
  no row break), and `parse_matrix_arg` uses it for a matrix parameter; `parse_arg` still flattens, through the same
  helpers. `eval_math`: `matrix_rows` fits a matrix (a number is a 1x1 matrix, evidenced by `MDETERM(5)` = 5,
  stats-evidence-floor-t1623 S!Q112; a blank or logical element is skipped, Excel's `#VALUE!`; a text element is
  skipped with no math value because it is unprobed, follow-up T-4626; a matrix over 64x64 is skipped), `fit_arg` has
  separate array and matrix branches, `indexed_call` reads `=INDEX(FUNC(args),row,column)` of an indexed definition
  (also under an exact scale), an indexed definition called outside INDEX is skipped, and a `literal_arrays`
  definition's matrix parameter must be an array literal too (LINEST and friends gain nothing, as T-4096 said).
  `composite` keeps a matrix argument's rows the same way (`=SUM(MDETERM({1}))` gets a value).
- `defs/matrix.py`: MINVERSE of an exactly singular matrix is a domain error only where Excel's elimination in
  doubles meets an exactly-zero pivot (`excel_zero_pivot`, arrays.md "MINVERSE: singularity rule"); a residual pivot
  (`{1,2,3;4,5,6;7,8,9}`, which Excel inverts into a number) is NoValueError. The committed s1 tables verify with
  0 differences and MMULT's s1 draws are reproduced exactly. Carry-overs done: MMULT cites its own arrays.md entry,
  `_mmult_draws` folds `uniform`/`logged` into `shaped`, `row_column`'s docstring reworded, `index_of` raises
  TypeError.
- Sidecars (probes only; probes-numeric has no such cell), only gains: 40 new math cells, 29 values and 11 domain
  errors, each where the oracle cell shows `#NUM!` (minverse-singular-near-singular B1:B4, E1:E4 and H1,
  minverse-overflow-computed-pivot I5:I6). 20 cells listed as
  skipped (no single value): the 18 residual-pivot MINVERSE elements (minverse-singular-near-singular A1:A9,
  minverse-overflow-computed-pivot K1:K9) and 2 non-square MDETERM ranges (stats-evidence-floor-t1623 Q108, Q116).
  New sidecars: matrix-boolean-elements, minverse-overflow-computed-pivot, minverse-singular-near-singular,
  weeknum-stale-between-calls; extended: amordegrc-fraction-state-ca, math-argument-modes,
  stats-evidence-floor-t1623.
- Excel kept as the target (decision 0300): stats-evidence-floor-t1623 S!Q107 `MDETERM(G1:I3)` (1..9), math 0,
  Excel 6.661338147750939E-16 (the elimination residual, arrays.md "MDETERM: partial-pivot LU"). Inside Gate A's
  1E-300 absolute floor and not counted: minverse-singular-near-singular H5 (math 1E-400, Excel 0) and
  minverse-overflow-computed-pivot P3 (math 5.0E-309, the graph reports 0, decision 0042). Every other new value is
  within 6E-16 relative.
- After merging main, CHISQ.TEST and CHITEST (T-4098, whose two parameters are matrices) gain too: 56 cells, 37 values
  and 19 domain errors, in probes stat-argument-modes, t4098-tests-prob-domains and t2576-error-type-cell-paired and
  probes-numeric m9-legacy-chitest and t1049-chisqtest. Every error is where the oracle shows one (`#N/A`, `#DIV/0!` or
  `#NUM!`, the same code), and every value is within 6.1E-16 relative of Excel; a number is a 1x1 matrix there too
  (`CHISQ.TEST(5,5)` `#N/A`, t1049-chisqtest A36, as for one cell).
- Review minors: a text element has no math value because it is unprobed (`MDETERM("5")` is evidenced only as a direct
  argument), follow-up T-4626 probes text and text-number elements; a matrix over 64x64 has none either;
  `is_array_literal` (sweep.py) replaces four spelled-out brace checks (sweep.py twice, tables.py, and eval_math's
  `_range_for_literal`, where it also needs the closing brace: that changes only a skip reason); matrix.py notes the
  unmodelled reverse case (nonsingular exactly, an exactly-zero pivot in doubles).
- Gates: `make eval-math-check` current, `make check-fast` green (duplication 2791 after the merge, baseline 2876),
  tools ruff, mypy and pytest (2398) green; `verify` gives 0 differences for MDETERM, MINVERSE, MMULT, CHISQ.TEST and
  CHITEST.
