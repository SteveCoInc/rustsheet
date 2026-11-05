---
id: T-4324
title: "Probe an empty numeric pivot group's start and end items under showAll, and share the unused-item checks (T-4311 review follow-ups)"
milestone: M6
lane: engine
status: blocked
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
- [ ] An excel-graph probe shows an empty numeric `<start` and `>end` under `showAll="1"` (layout and GETPIVOTDATA by
      label); the engine matches it or the cells are owned; web.md cites it.
- [ ] One helper each for the builder's unused-item validation and the facade's unused-item loop.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unused items of a grouped field (`PivotError::GroupedWithUnusedItems` stays).

## Blocked because
The code this task changes is in PR #3334 (T-4311's engine change: `row_layout`'s `LayoutCell::Empty`,
`spill_extent::group_rows`, `add_unused_items`, the test `a_grouped_pivot_with_show_all_shows_and_answers_its_empty_groups`,
probes/t4311-pivot-showall-grouped), which is approved but not merged as of 2026-11-05 13:56 UTC; on main T-4311 shows
done because its probe PR (#3337) merged. Unblock once #3334 merges.
