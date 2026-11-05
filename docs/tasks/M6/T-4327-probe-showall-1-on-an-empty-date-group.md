---
id: T-4327
title: "Probe showAll=1 on an empty date group; mark the unread pivot layouts UNVERIFIED; builder tests for column unused items (T-4311 review follow-ups)"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
GETPIVOTDATA on an empty date group under `showAll="1"` is backed by a probe, and the T-4311 notes and builder code
carry the review's documentation and test follow-ups.

## Context
These are the MINOR findings of the PR #3337 (T-4311) reviews, which all approved:
- T-4311 made an empty group follow `showAll` (`shown_group`, crates/rustsheet-engine/src/functions/lookup_and_reference/
  pivot_index.rs): `#REF!` at 0 and 0 at 1. Only a numeric range group is probed at `showAll="1"`
  (probes/t4311-pivot-showall-column-and-groups Calc E1, E3, E6). docs/excel-semantics/web.md's date-group entry says a
  date group is "taken to follow it, unprobed". Excel-fidelity asked for an **UNVERIFIED (probe pending)** marker until
  a probe settles it.
- web.md's column-field bullet (T-4248 entry, "The same rule holds on a column field ...") describes GETPIVOTDATA
  results that Excel computed, but the column layout itself (the empty "C" column) is the builder's, not read back,
  because the pivot has `refreshOnLoad` off. The row-field bullet tags this ("UNVERIFIED: the layout is the
  builder's"); the column bullet should too.
- tools/src/rustsheet_tools/pivot.py gained `column_unused_items` and `column_show_all` (T-4311), but
  tools/tests/test_spec2xlsx_pivots.py has no case for them. That means no test of the "need a column field" error,
  `_shows_all` on the column, or the column-axis `refreshOnLoad` and `colItems` output. T-4311's Outcome says "the
  builder's pivot tests" cover the change; the existing tests only pass on it.
- The spec builder already supports date groups (`"by"`) and `show_all` (pivot.py module doc; probes/t4236-pivot-date-
  groups-no-blank is a template).

## Acceptance criteria
- [ ] An excel-graph probe of a date-grouped row field (e.g. by months) with `showAll="1"` and an empty group: the
      engine matches it (GETPIVOTDATA on the empty group, and Excel's own layout read back), or an owner task is filed
      for the off cells.
- [ ] web.md: the date-group entry states the probed result; the column-field bullet says its layout is the builder's
      (UNVERIFIED), not Excel's.
- [ ] test_spec2xlsx_pivots.py covers `column_unused_items`/`column_show_all`: the column-field error, the column's
      `showAll` attribute, `colItems` count and `refreshOnLoad` off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unused items of a grouped field (`PivotError::GroupedWithUnusedItems` stays).
