---
id: T-4269
title: "DateGroups: probe seconds, minutes and negative number items; harden the type (T-4246 review follow-ups)"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-4246]
---

## Goal
The unprobed GETPIVOTDATA number items on date-grouped fields are probed and matched, and `DateGroups` can no longer
panic, however it is built.

## Context
These are the MINOR findings from PR #3258 (T-4246), which all three reviewers approved:
- **Unprobed number items** (`DateGroups::number_item`, crates/rustsheet-engine/src/pivot/date_groups.rs):
  - A number item on a seconds or minutes group is taken as 0 to 59, by analogy with the hour rule. It is UNVERIFIED in
    web.md, and no probe groups by seconds or minutes.
  - The "integer part" is `trunc`, so a negative fraction (-0.5 on an hour group) is `12 AM` with no evidence. With
    `floor` it would be `#REF!`.
  - Probe both with excel-graph (spec pivots grouped by `seconds` and `minutes`; pivot.py `_DATE_GROUPS`), and match.
- **Hardening:** `DateGroups` has pub fields and derives `Decode`, while `item` and `number_item` compute
  `self.count - 1` and `self.count - 3`. These underflow for a value built or decoded with `count < 3`. Every call
  site goes through `DateGroups::new`, which enforces `count >= 3`, so nothing is reachable today. Make the fields
  private (with accessors), or use `saturating_sub`. Comment the bounded `as` casts in `in_range` and
  `split_serial`.
- **Naming:** `PivotRangeGroup` (crates/rustsheet/src/edit/pivot.rs, `#[doc(hidden)]`) now carries date groups too.
  Consider `PivotGroup`.
- **Importer:** it now refuses (skips) a date grouping with a wrong item count even when the base field holds no
  dates, where before the check ran only per grouped value. Record that in web.md's importer skip list.

## Acceptance criteria
- [ ] A probe case covers number items on seconds and minutes groups and a negative fractional number item; the code
      matches it, and web.md drops the UNVERIFIED note.
- [ ] `DateGroups` cannot underflow for any value of its fields.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4260 (a grouping over a blank) and T-4261 (cache rounding at a half second).
