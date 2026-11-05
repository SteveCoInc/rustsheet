---
id: T-4344
title: "GROUPBY's tree on a descending first field of three: the row t4342 Calc!C40 finds its class where the lower-bound search does not; and PIVOTBY columns, table sorts and value sorts on intransitive runs"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4342]
---

## Goal
GROUPBY and PIVOTBY match Excel on the intransitive-run cases T-4342's tree model does not cover: the descending
first field of a three-field key in probes/t4342-groupby-tree-holdout Calc!C40, PIVOTBY column fields on such
runs, and a table sort (`field_relationship` 1) or a sort by a value column on them.

## Context
- The model is in docs/excel-semantics/coercion.md, "GROUPBY on an intransitive run"; the code is
  `Grouping::new` (functions/lookup_and_reference/grouping.rs, `Tree`).
- Calc!C40: rows (B,y,A), (C,x,B), (B,x,C), (A,x,A), (A,x,B) with `sort_order` -1 (A = ab, B = ab&U+00AD,
  C = U+00AD&ab). The sorted keys are (A,x,A), (C,x,B), (B,y,A); the first field's classes are A and B. Excel puts
  row 1 in (B,y,A) alone (`"1"`), so its search for B over the classes [A, B] found B; the inclusive lower-bound
  search in that direction finds A (A < B is "after" B descending) and leaves the row in no group, and the leaf
  then aggregates every row. Row 3 (B,x,C) is in no group in Excel, which a search finding B also gives (no x
  class under B). An ascending search fits C40 but misses 24 of t4241's C cells
  (docs/research/t4342/compare.py, `docs/research/t4241/model.py` with `ascending_search=True`). Probe more
  descending three-field runs before choosing a rule.
- PIVOTBY's column fields: `Grouping::group_of` maps each record to one column group, so a record in a reached
  column and in a column no record reached is counted in the reached one only. Unprobed.
- `tree_directions` builds the tree ascending for a table sort and a value sort: unprobed on these runs.
- Owned cells: probes/t4342-groupby-tree-holdout (1): Calc!C40.

## Acceptance criteria
- [ ] A probe of descending three-field runs (and PIVOTBY columns on intransitive runs) is filled, and the model
      is revised to fit it and Calc!C40, or the cells stay known failures with the reason recorded.
- [ ] The probe commits the current model's predictions (the [0, n-1] search, chosen on the t4342 hold-out) before
      its fill, with blocks shaped like t4342 Calc!B14 and B48 (two leaves showing one key): that confirms the search
      revision out of sample.
- [ ] It covers ASCII case-spelling blocks (t4342 Calc!B51:B56's shape) under `sort_order` -1 and -2, `field_relationship`
      1 and a value sort: the shown spelling in those directions is unprobed though the engine builds the tree in them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT's order on intransitive cycles (T-3418).
