---
id: T-4342
title: "GROUPBY and PIVOTBY group multi-field intransitive runs by T-4241's tree model (sorted UNIQUE keys, per-field lower-bound search, empty leaf aggregates every row)"
milestone: M7
lane: engine
status: done
priority: P0
size: M
tier: opus
depends_on: [T-4241]
---

## Goal
GROUPBY and PIVOTBY give Excel's groups, and each group's members, on multi-field keys where Excel's `=` is
intransitive. That includes the rows Excel puts in two groups and the groups that aggregate every row.

## Context
- The rule is in docs/excel-semantics/coercion.md, "GROUPBY on an intransitive run (T-4195, T-4241)". The reference
  model is docs/research/t4241/model.py, which fits 234 of 234 blocks of probes/t4195-groupby-intransitive and
  probes/t4241-groupby-intransitive-runs, sort directions included. In short:
  1. UNIQUE's kept rows (`coalesce_equal_lines`), sorted by SORT's merge (`sort_indices_pairwise`) on whole rows,
     each field in its sort direction.
  2. Classes are built field by field over that list. A key starts a new class under its parent unless its field
     `=` the field of the class's first key. A class shows that first key.
  3. Each row descends the tree with a lower-bound search over the parent's class keys, in the field's direction.
     It drops out when the found key is not `=` to it, or when the search runs off the end.
  4. A leaf no row reaches aggregates every row of the source (SUM counts each row once per group it is in).
- The sort direction enters the grouping, so `Grouping::new` must take it. The table sort (`field_relationship` 1)
  and the sort by a value column (`sort_order` 3 and above) on these runs are unprobed: probe them before
  extrapolating. `sort_order` 2 gave the ascending result on every probed block.
- `Grouping::new` is about 130 lines (T-4195 design MINOR). When this task replaces its body, split the
  whole-row pass and the group building into helpers, and share the text-pair `=` and cost closures.
- Before the engine change, write a confirmation probe that holds out data the model was not chosen on: seeded
  5-row runs, three fields, and PIVOTBY columns. Commit its case and the model's predictions before the fill, as
  docs/research/t4237 did (decision 0188 R4 hold-out for a structural model). T-4241 chose the model after it had
  seen the t4241 probe's values, so that probe is not a hold-out.
- Owned cells (taken over from T-4241): probes/t4195-groupby-intransitive (24): Calc!C8, C9, C11, C15, C16, C17,
  C20, C21, C23, C24, E46, E48, E49, E50, F8, F9, F11, F15, F16, F17, F20, F21, F23, F24; probes/t4088-groupby-
  multifield (2): Calc!B11, B33; probes/t4241-groupby-intransitive-runs (140): Calc!B2:B3, B5, B9:B11, B14:B15,
  B17:B18, B21:B23, B25, B27, B29, B31:B33, B36:B39, B41, B43, B45, B47, C1:C3, C5:C12, C14:C15, C17:C28, C30:C34,
  C36:C41, C43, C45, C47, D2:D3, D5, D9:D11, D14:D15, D17:D18, D21:D23, D25, D27, D29, D31:D33, D36:D39, D41, D43,
  D45, D47, E3:E5, E9:E11, E13, E16, E22:E23, E25, E29, E32:E33, E36:E39, E43, E46, F2:F3, F5, F9:F11, F14:F15,
  F17:F18, F21:F23, F25, F27, F29, F31:F33, F36:F39, F41, F43, F45, F47.

## Acceptance criteria
- [x] The confirmation probe's predictions are committed before its fill, and the model is Excel's on it (or the
      misses are recorded and the model revised).
- [x] The owned cells pass, with 0 regressions in eval/baseline, or the task records with evidence which ones stay
      known failures and why.
- [x] coercion.md "Implemented in" names the code.
- [x] `make check` green; reviewers approve.

## Out of scope
- SORT's order on intransitive cycles (T-3418).

## Outcome
- Hold-out: probes/t4342-groupby-tree-holdout (56 blocks, 206 GROUPBY/PIVOTBY cells: seeded five-row runs of two
  and three fields, ascending, `sort_order` -1, SUM, PIVOTBY by a column field, and ASCII case spellings). The
  model's predictions (docs/research/t4342/predictions.tsv) were committed before the fill (commit "hold-out probe
  ... before the fill"). The frozen T-4241 model scores 199 of 206. The six misses are one shape (two leaves
  showing one key); a lower-bound search over [0, n-1] instead of [0, n) fits them and keeps all 234 earlier
  blocks (t4195 C and E, t4241 B to E): 205 of 206 (`compare.py --revised`). That revision is a structural choice
  made on the hold-out (decision 0188 R4: no fitted constant). The miss left is Calc!C40 (three fields,
  `sort_order` -1), filed as T-4344.
- New finding: even in ASCII a group shows the first spelling in whole-row sorted order, not in source order:
  (`ABC`,y), (`abc`,x) shows `abc` twice (Calc!B51:B56). So the tree model applies to every multi-field key, not
  only to intransitive runs, and the engine now uses it everywhere. coercion.md records both findings.
- Engine: `Grouping::new` is the tree model (`Keys` for the comparisons and their cost, `Tree::build`,
  `Tree::descend`) and takes the tree directions (`tree_directions`: a hierarchy's key directions, ascending for a
  table or a value sort). UNIQUE's kept-row pass moved out of `unique_of` into `unique_first_lines`, which both use.
  T-4195's `classify` and the whole-row drop pass are gone. While no key text is outside ASCII, a row descends as
  its first equal row does, so there is one search per distinct key.
- Eval: probes known failures 6074 -> 5909 (166 fixed: all owned cells, 24 t4195 + 2 t4088 + 140 t4241), 0
  regressions. One new-coverage failure: t4342 Calc!C40 (T-4344).
- Follow-up: T-4344 (C40; PIVOTBY column fields, table sorts and value sorts on intransitive runs, all unprobed).
