---
id: T-4762
title: "Probe the readers of a one-index INDEX over a column of references (AVERAGE, MAX, ROWS of the nested INDEX, SUM of INDEX(INDEX(r,2),1))"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for the readers of a one-index INDEX over a column of references that T-4701's change reaches but its
probe did not cover, and the engine following it.

## Context
- T-4701 (probes/t4701-index-one-index-ref-grid, excel-graph 2026-11-07) keeps a reference element picked by
  `INDEX(r,2)` from a column of references (lifted OFFSET, lifted INDIRECT, a mixed XLOOKUP grid) as a 1x1 array of
  references wherever a reference is taken (`index_reference_arg`, `column_element` in
  crates/rustsheet-engine/src/functions/lookup_and_reference/index.rs). Probed: the nested `INDEX(..,1,1)` spill, ROWS
  of the one-index INDEX, ISREF of the nested INDEX, and `SUM` of the one-index INDEX (top-left cell only).
- The T-4701 Excel-fidelity review (PR #3817) found these cells changed from `#VALUE!` on main with no oracle data:
  `AVERAGE` and `MAX` of `INDEX(OFFSET(Data!$A$1,{0;1},0,2),2)` (now 2; 1 over the XLOOKUP grid), ROWS of
  `INDEX(INDEX(r,2),1,1)` (now 2), and `SUM(INDEX(INDEX(r,2),1))` (now 6, the whole element). They follow SPEC-0014
  rule 4 (top-left aggregates) and the probed spill, but are unprobed.
- Also name the ISREF TRUE result (t4701 A4) in docs/excel-semantics/lookup.md's entry.

## Acceptance criteria
- [x] A probes case (excel-graph) covers AVERAGE, MAX, MIN and PRODUCT of the one-index INDEX, ROWS and COLUMNS of the
      nested INDEX, and `SUM(INDEX(INDEX(r,2),1))`, over the three grid kinds.
- [x] lookup.md records the results; the engine passes the case, or an owner task is filed for each failing cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- One index into a grid of several columns.

## Outcome
New probe probes/t4762-index-ref-grid-readers (excel-graph, 27 cells; no engine change, every cell passes on main).
Over OFFSET, INDIRECT and the mixed XLOOKUP grid, AVERAGE, MAX, MIN and PRODUCT of `INDEX(r,2)` read the element's
top-left cell (2, 2, 1), COUNT of it is 0 (a quirk: it does not count the number the others read), ROWS and COLUMNS
of `INDEX(INDEX(r,2),1,1)` are 2 and 1, and `INDEX(INDEX(r,2),1)` is the whole element (SUM 6, 6, 3; ROWS 2).
lookup.md's T-4570 entry records them and the t4701 A4 ISREF TRUE. Eval delta: one new passing case, 0 regressions;
one generated case-count line in eval/baseline/probes.txt.
