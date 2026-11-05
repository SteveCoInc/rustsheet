---
id: T-4269
title: "DateGroups: probe seconds, minutes and negative number items; harden the type (T-4246 review follow-ups)"
milestone: M6
lane: engine
status: done
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
- [x] A probe case covers number items on seconds and minutes groups and a negative fractional number item; the code
      matches it, and web.md drops the UNVERIFIED note.
- [x] `DateGroups` cannot underflow for any value of its fields.
- [x] `make check` green; reviewers approve.

## Out of scope
- T-4260 (a grouping over a blank) and T-4261 (cache rounding at a half second).

## Outcome
- Probe `probes/t4269-pivot-time-group-number-items` (excel-graph, 252 cells): GETPIVOTDATA number and text items on
  pivots grouped by seconds, minutes and hours. A second or minute number item is 0 to 59, like the hour rule, and the
  integer part is truncated toward zero (`-0.5`, `-0.0001` pick `:00` / `12 AM`; `-1`, `60` are `#REF!`). The engine
  already matched every cell, so there is no code change for the probe. web.md drops the UNVERIFIED note and records the rule, Excel's
  `:00` labels for seconds, and the importer's count check over a base field without dates.
- Hardening: `DateGroups`' fields are private (only `DateGroups::new` builds one). `item` and `number_item` go
  through a checked `last_inner()`, so a decoded grouping of fewer than three items is `DateGroupError::Count` /
  `None`, never an underflow or a `clamp` panic. The bounded `as` casts are commented. Unit tests cover both.
- Not done: renaming `PivotRangeGroup` to `PivotGroup`. The task says "consider"; it is a `#[doc(hidden)]` name, and a
  rename touches the facade for no behavior change.
- Eval delta: +1 case (252 cells, all passing); no regressions.
