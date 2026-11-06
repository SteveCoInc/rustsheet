---
id: T-4195
title: "GROUPBY and PIVOTBY multi-field keys on an intransitive = run: match Excel's dropped rows and shown keys (probes/t4088-groupby-multifield)"
milestone: M7
lane: engine
status: done
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
- [x] The cells pass and leave eval/baseline/probes.txt, with 0 regressions: 8 of the 10 do; B11 and B33 move to
      T-4241 with the probe evidence (a DEVIATION is not open to this gap: README "Deliberate deviations",
      decision 0052).
- [x] coercion.md states the rule (VERIFIED) and "Implemented in".
- [x] `make check` green; reviewers approve.

## Out of scope
- SORT's order on intransitive cycles (T-3418).

## Outcome
- Probed (excel-graph, 2026-11-04): probes/t4195-groupby-intransitive (56 blocks: every order of `ab`,
  `ab`&U+00AD, U+00AD&`ab`, alone, with `AB` or `ab`&U+00AD&U+00AD, beside a second field, before it, and with a
  third) and probes/t4195-groupby-shown-key (18 blocks: which spelling a group shows). A simulator scored candidate
  models against every cell; the rule is in coercion.md "GROUPBY on an intransitive run (T-4195)".
- Excel's model, now ours (`Grouping::new`, `classify`, `Group::key_order` in
  functions/lookup_and_reference/grouping.rs): each field is classed within its parent (the rows sharing the
  earlier fields' classes); the distinct keys are the first-`=` ones, sorted by `<` (`sort_indices_pairwise`),
  and a text joins the key a lower-bound binary search finds equal to it; a group shows each class's first key;
  a group none of UNIQUE's kept rows is in is not shown (`coalesce_equal_lines`, T-4200's UNIQUE grouping, when a
  key holds a text outside ASCII); texts under one parent sort by that order
  (`AB` before `ab`&U+00AD).
- Eval delta: probes known failures 4046 -> 4064: 8 fixed (t4088-groupby-multifield B1, B3, B6, C6, B30, B31,
  B32, B38), 0 regressions, 2 new cases (-intransitive 24 known failures after the merge with main, where T-4200's UNIQUE passes
  D8 and D23; -shown-key 0).
- Not reached: on some multi-field runs Excel puts a row in two groups, and its UNIQUE keeps or drops a row
  depending on the rows around it. Filed T-4241 (opus), which owns the 24 -intransitive cells and t4088 B11, B33.
