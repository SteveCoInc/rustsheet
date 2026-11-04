---
id: T-4088
title: "GROUPBY and PIVOTBY group text keys by Excel's = and order them pairwise (probes/t4086-groupby-weightless)"
milestone: M7
lane: engine
status: todo
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
  - {`ab`, U+00AD&`ab`, `AB`} = 49, its key `ab`;
  - {`ab`&U+00AD} = 2;
  - {`é`, `é`&U+00AD} = 12.
  This is `=`'s prefix rule. The engine gives five groups, as before T-3904.
- Ours: `group_key` (functions/lookup_and_reference/grouping.rs) keys a text by `cell_key`, its upper-case form. So it
  also keeps apart texts that `=` equates for other reasons (`ß` and `ss`, `á` and `a`&U+0301). `key_cmp`
  (group_order.rs) orders them with the total `cmp_text`.
- Model: UNIQUE (`unique_cell_key` and `unique_of`, functions/spill_functions.rs). It keys a text by its
  `CollationKey`, keeps a line unless `cmp_text_pairwise` finds it equal to a kept line of its bucket (the first
  one kept), and charges each comparison to the work budget. Reuse it, do not copy it (directive 3).
- Owned cells: the failing cells of probes/t4086-groupby-weightless. The T-4086 PR lists them in
  eval/baseline/probes.txt.
- Related: T-3418 (SORT's order on intransitive cycles). The coercion.md "Not a total order" bullet says GROUPBY and
  PIVOTBY "still sort with the total `cmp_text`".

## Acceptance criteria
- [ ] Every probes/t4086-groupby-weightless cell passes. `make eval-check` shows 0 regressions; in particular, the
      groupby-pivotby probe cases and gen-m9 keep every cell.
- [ ] One grouping-by-`=` helper is shared with UNIQUE (no copy).
- [ ] coercion.md states "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT's algorithm on intransitive cycles (T-3418).
