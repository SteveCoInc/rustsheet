---
id: T-0950
title: "Legacy (CSE) array formulas fill a range larger than their result (repeat 1x1/row/column, #N/A elsewhere)"
milestone: M9
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A legacy (CSE) array formula entered over a range larger than its result fills that range as Excel
does: a 1x1 result fills every cell, a single-row result is repeated down and a single-column
result across, and every other extra cell is `#N/A`.

## Context
poi/MatrixFormulaEvalTestData.xlsx (Excel-cached, Excel 16), Sheet1 H46:J136: 95 known failures
(eval/baseline/poi.txt), a mix of repeated-value cells and 47 `#N/A` fill cells. Examples: `TRANSPOSE(B46)` over H46:J48 is 1 in
all 9 cells (also `MINVERSE(B54)` H54:J56, `MMULT(B58,E58)` H58:J60); `B62:C62+E62:F62` over
H62:J64 repeats the row down with J62:J64 `#N/A`; `TRANSPOSE(B66:C66)` over H66:J68 has row 68
`#N/A`. Rule recorded in docs/excel-semantics/arrays.md "Legacy (CSE) array formula over a larger
range". Found by the gap-triager while filing T-0943 (the MDETERM/MINVERSE blank-cell part).
Also (gap triage 2026-10-27): corpus-bench/13894/1_13894_input.xlsx Sheet1!F20:F25, the six extra cells of the CSE
array formula over F14:F25 (`IFERROR(INDEX(ucode,SMALL(IF(MATCH(ucode,ucode,0)=...),...)),"")`, ucode = J3:J8, a
6-row result): Excel `#N/A`, ours `#VALUE!` (6 visibly-off cells; the case's other 22 pass).
This is an engine change to how array-formula ranges are filled from a result (the xlsx importer
and the evaluator's array-formula path): check whether it needs a short spec first (CLAUDE.md
directive 4) and split if it is bigger than M.

## Acceptance criteria
- [x] The H46:J136 F lines of poi/MatrixFormulaEvalTestData.xlsx disappear from eval/baseline/poi.txt
      (those that stay need a documented reason) with no new F lines in any baseline.
- [x] corpus-bench/13894 F20:F25 pass, so its whole-case `F` line leaves eval/baseline/corpus-bench.txt
      if no other cell of the case fails.
- [x] Unit tests for the 1x1, row and column broadcast and the `#N/A` fill.
- [x] `make check` green; reviewers approve.

## Out of scope
- Dynamic-array spilling (`#SPILL!`), which is a different mechanism.

## Outcome
- The CSE branch of `Model::set_cells_with_result` (model.rs) now reads each cell of the declared range through
  `arithmetic::bcast_get`, the broadcast helper that already exists, instead of an exact-index lookup. A one-row or
  one-column result repeats (a 1x1 result fills every cell), and a cell past the result is `#N/A`. Before, it was
  `#VALUE!`. A scalar result already filled the range.
- Unit tests: test/array_formulas/test_cse_larger_range.rs covers the 1x1, row and column broadcasts, a 2x2 result's
  `#N/A` fill, and the fill that IFERROR does not catch.
- Eval delta (`make eval-check` with poi and corpus-bench fetched): poi known failures 111 -> 24 (fixed 87; every
  MatrixFormulaEvalTestData.xlsx H46:J136 F line is gone). corpus-bench 3 -> 2 (fixed 1: the whole-case F line of
  13894). 0 regressions in every suite.
- `make eval-baseline` wrote eval/baseline/poi.txt and corpus-bench.txt. It then refused at its roundtrip step, on
  roundtrip-poi/54436.xlsx, and that case fails the same way on `origin/main` without this change. Its evidence-floor
  step was run on its own afterwards (no change). Follow-up: T-4829.
