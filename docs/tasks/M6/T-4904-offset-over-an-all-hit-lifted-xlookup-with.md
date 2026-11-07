---
id: T-4904
title: "OFFSET over an all-hit lifted XLOOKUP with a range if_not_found spills #VALUE! (T-4800 finding)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`OFFSET(XLOOKUP({2;3},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$G$1:$G$2),r,0)` (every lookup hits) spills a 2x1 of `#VALUE!`
(ROWS 2, COLUMNS 1), and a 1x1 for the one-element lookup `{2}`, as in Excel; ours is a scalar `#VALUE!`.

## Context
- Evidence: probes/t4800-offset-allhit-xlookup (excel-graph, T-4800); docs/excel-semantics/lookup.md "OFFSET over an
  all-hit lifted XLOOKUP"; SPEC-0014 "Per-element callers".
- Failing cells (Calc): A2, A5, B1, C1, B4, C4, B7, C7, B10, C10. The scalar control A13, A16 and the first spill cells pass.
- Code: functions/lookup_and_reference/offset.rs `offset_source` and `xlookup_reference` (T-4739 made the mixed grid lift).

## Acceptance criteria
- [ ] The 10 cells above pass and leave eval/baseline/probes.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- COUNTIF and SUMIF over all-hit XLOOKUP (T-4742).
