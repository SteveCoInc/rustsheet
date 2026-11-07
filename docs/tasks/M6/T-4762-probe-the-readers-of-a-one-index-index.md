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
- [ ] `make check` green; reviewers approve.

## Out of scope
- One index into a grid of several columns.

## Outcome
Added probes/t4762-index-one-index-readers (27 cells, excel-graph 2026-11-07): AVERAGE, MAX, MIN, PRODUCT and COUNT of
the one-index INDEX, ROWS and COLUMNS of the nested `INDEX(..,1,1)`, and SUM and ROWS of `INDEX(INDEX(r,2),1)`, over
the lifted OFFSET, lifted INDIRECT and mixed XLOOKUP grids, with data chosen so top-left and whole-element reads
differ (a descending column for MIN). Excel reads the top-left cell for AVERAGE/MAX/MIN/PRODUCT, counts nothing with
COUNT (0, a quirk), and the nested one-index INDEX hands the whole element back (SUM 6, ROWS 2). Ours matches all 27
cells with no code change; lookup.md records the results and names t4701's ISREF TRUE. Eval delta: +1 case, 27 new
passing cells, 0 regressions.
