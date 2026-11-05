---
id: T-4324
title: "Probe an empty numeric pivot group's start and end items under showAll, and share the unused-item checks (T-4311 review follow-ups)"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4311]
---

## Goal
Whether Excel lays out and answers an empty numeric group's `<start` and `>end` items under `showAll="1"` is settled by
an excel-graph probe, and the duplicated unused-item checks have one definition each.

## Context
MINOR findings of the T-4311 reviews (PR #3334), which all approved:
- excel-fidelity: probes/t4311-pivot-showall-grouped shows empty `<start`/`>end` items under `showAll="1"` only for a
  date group (M: `<1/1/2019`, `>1/1/2020` blank, GETPIVOTDATA 0). In the numeric pivot G both `<0` and `>9` hold a
  record. The engine (`row_layout`, `shown_group`), the unit test
  `a_grouped_pivot_with_show_all_shows_and_answers_its_empty_groups` (crates/rustsheet/src/edit/pivot.rs) and
  `spill_extent::group_rows` apply the date result to numeric groups. web.md "GETPIVOTDATA on a recordless item of a
  pivot field" gives the numeric pivot as the example of "`<start` and `>end` included"; reword it to fit the evidence.
- design: tools/src/rustsheet_tools/pivot.py validates `unused_items` and `column_unused_items` with two copies of the
  same rule (one helper for both axes), and `add_unused_items` (crates/rustsheet/src/edit/pivot.rs) repeats its
  add-shared-items loop per axis.
- Use the `excel-probe` skill: a numeric `showAll="1"` pivot whose data lies wholly inside `start`..`end`.

## Acceptance criteria
- [x] An excel-graph probe shows an empty numeric `<start` and `>end` under `showAll="1"` (layout and GETPIVOTDATA by
      label); the engine matches it or the cells are owned; web.md cites it.
- [x] One helper each for the builder's unused-item validation and the facade's unused-item loop.
- [x] `make check` green; reviewers approve.

## Out of scope
- Unused items of a grouped field (`PivotError::GroupedWithUnusedItems` stays).

## Outcome
Already met on main when this task was picked; nothing to change but this record.
- Evidence: probes/t4311-pivot-showall-column-and-groups (excel-graph, 2026-11-05), which T-4311 landed in place of
  the date-only probe the review saw, groups a numeric N by 0.1 from 0 to 0.4 with no record in `<0`, `0.1-0.2` and
  `>0.4`. With `showAll="1"` (pivot H) Excel lays out `<0` and `>0.4` with blank values (Calc!I2:J2, I7:J7) and
  GETPIVOTDATA on their labels is 0 (Calc!E1, E6); with `showAll="0"` they are not shown and are `#REF!` (D1, D6).
  The engine passes all 68 cells of the case, so `row_layout`, `shown_group`, `spill_extent::group_rows` and the unit
  test are right to apply the rule to numeric groups. docs/excel-semantics/web.md ("A row field grouped by 0.1 ..."
  and the grouped-field bullets) already cites that case for the numeric pivot.
- One helper each: tools/src/rustsheet_tools/pivot.py validates both axes with `PivotSpec._unused`, and
  `add_pivot_table` (crates/rustsheet/src/edit/pivot.rs) adds both axes' unused items in one loop over
  `(row, unused_items)` and `(column, column_unused_items)`.
- Eval delta: none.
