---
id: T-4195
title: "GROUPBY and PIVOTBY multi-field keys on an intransitive = run: match Excel's dropped rows and shown keys (probes/t4088-groupby-multifield)"
milestone: M7
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY and PIVOTBY give Excel's groups for multi-field keys when Excel's `=` is intransitive across them. Or, if
the algorithm stays out of reach after probing, the cells are recorded as a DEVIATION (decision 0014).

## Context
- Evidence: probes/t4088-groupby-multifield (excel-graph, 2026-11-04, T-4088); coercion.md "Format characters,
  U+200B and the C1 controls (T-3662)", the T-4088 multi-field bullet. With the rows (`ab`,x), (`ab`&U+00AD,y),
  (U+00AD&`ab`,y) and values 1, 2, 4:
  - GROUPBY shows 2 groups, as UNIQUE does (Calc!B1:B2, B34): `{"ab","x",1;"ab­","y",2}`.
  - The third row is in no group: COUNTA counts 1, and ARRAYTOTEXT of the values is "2" (B30:B32). PIVOTBY is the
    same (B3, B38).
  - With the rows reversed, the shown keys are `{"ab","x",4;"ab","y",1}`: the `ab`&U+00AD row drops out, and the
    second group shows `ab`, not its first spelling U+00AD&`ab` (B8:B12, B33).
  - Each field alone groups by first `=` (B35:B36; probes/t4086-groupby-weightless).
- Ours (T-4088): each field is classed by `first_equal_lines` (functions/spill_functions.rs) and rows are grouped
  by the tuple of classes (`Grouping::new`, functions/lookup_and_reference/grouping.rs). Forward, that gives three
  groups.
- Probe first (excel-probe): more rows and orders, a third field, and which spelling a group shows. Find the rule
  for the group set, for membership and for the shown key. Candidates: whole-row keys with an exact-spelling
  membership test, or a per-field lookup into the key set's own field values.
- Owned cells: probes/t4088-groupby-multifield (10): Calc!B1, B3, B6, C6, B11, B30, B31, B32, B33, B38

## Acceptance criteria
- [ ] The cells pass and leave eval/baseline/probes.txt, with 0 regressions; or a DEVIATION entry records them with
      the probe evidence that the algorithm is out of reach.
- [ ] coercion.md states the rule (VERIFIED) and "Implemented in".
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT's order on intransitive cycles (T-3418).
