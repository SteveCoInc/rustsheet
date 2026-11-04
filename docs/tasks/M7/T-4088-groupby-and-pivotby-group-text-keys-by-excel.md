---
id: T-4088
title: "GROUPBY and PIVOTBY group text keys by Excel's = and order them pairwise (probes/t4086-groupby-weightless)"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY and PIVOTBY group text row and column keys by Excel's `=` and order the groups by Excel's pairwise
comparison, as UNIQUE and SORT do.

## Context
- Evidence: probes/t4086-groupby-weightless (excel-graph, 2026-11-04, T-4086); coercion.md "Format characters, U+200B
  and the C1 controls (T-3662)", the T-4086 GROUPBY bullet. Over `ab`, `ab`&U+00AD, `é`, `é`&U+00AD, U+00AD&`ab`
  and `AB`, Excel gives three groups, in this order:
  - {`ab`, U+00AD&`ab`, `AB`} = 49, its key the first spelling `ab` (EXACT: Calc!B11:B12);
  - {`ab`&U+00AD} = 2;
  - {`é`, `é`&U+00AD} = 12.
  This is `=`'s prefix rule. The engine gives five groups, as before T-3904.
- Ours: `group_key` (functions/lookup_and_reference/grouping.rs) keys a text by `cell_key`, its upper-case form. So it
  also keeps apart texts that `=` equates for other reasons (`ß` and `ss`, `á` and `a`&U+0301). `key_cmp`
  (group_order.rs) orders them with the total `cmp_text`.
- Model: UNIQUE (`unique_cell_key` and `unique_of`, functions/spill_functions.rs). It keys a text by its
  `CollationKey`, keeps a line unless `cmp_text_pairwise` finds it equal to a kept line of its bucket (the first
  one kept), and charges each comparison to the work budget. Reuse it, do not copy it (directive 3).
- Owned cells: probes/t4086-groupby-weightless (13): Calc!B1, B2, B4, B6, B7, B8, C5, C7, C8, C9, D6, D7, D8
- Related: T-3418 (SORT's order on intransitive cycles). The coercion.md "Not a total order" bullet says GROUPBY and
  PIVOTBY "still sort with the total `cmp_text`".

## Acceptance criteria
- [x] Every probes/t4086-groupby-weightless cell passes. `make eval-check` shows 0 regressions; in particular, the
      groupby-pivotby probe cases and gen-m9 keep every cell.
- [x] One grouping-by-`=` helper is shared with UNIQUE (no copy).
- [x] coercion.md states "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT's algorithm on intransitive cycles (T-3418).

## Outcome
- New `first_equal_lines` (functions/spill_functions.rs) holds UNIQUE's loop, now shared. It buckets lines by a key
  and has a line join the first kept line of its bucket that `same` finds equal, charging every comparison to the
  work budget. UNIQUE calls it.
- `Grouping::new` (functions/lookup_and_reference/grouping.rs) classes each key field with it, with
  `cmp_text_pairwise` for texts and `group_key` (now UNIQUE's `unique_cell_key`, -0 as 0) for the bucket. Groups,
  `prefix_ids`, the subtotal levels and `first_difference` compare those classes (`Group::cells: Vec<usize>`), so
  PIVOTBY's parents agree with the groups. `Grouping::new` takes the work budget as `spend`; GROUPBY and PIVOTBY pass
  `Model::spend_work`.
- The groups still sort with the total `cmp_text`. A class's members tie there, so first-appearance order holds
  (UNVERIFIED on intransitive cycles: T-3418).
- Folded in the #3142 MINORs: the collation module and `CollationKey` docs describe the six passes; coercion.md says
  why half-width kana stays a kana type (t3275 A99); new probe task T-4153 for `NARROW_NBSP`'s value.
- Unit test `keys_group_by_excel_equality`; grouping.rs's tests moved to grouping/tests.rs (the 1000-line limit).
- Eval delta: probes known failures 4239 → 4226 (the 13 owned cells), REGRESSIONS 0 in every suite (gen-m9's
  GROUPBY and PIVOTBY sweep and every groupby/pivotby/unique probe case keep each cell they passed).
