---
id: T-4249
title: "Probe numeric pivot groups: other wrong item counts, partial last group, GETPIVOTDATA numbers past the end"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The numeric range-group points T-4235 extrapolated past its evidence are settled by an excel-graph probe, and the
engine matches it or the cells are owned.

## Context
- Follow-up of T-4235 (crates/rustsheet-engine/src/pivot/number_groups.rs `NumberGroups`; docs/excel-semantics/web.md
  "Grouped pivot fields import as their group items", the UNVERIFIED (T-4249) list). The engine assumes:
  - any `<groupItems>` count other than the layout's relabels with the end groups first (only one item too many is
    probed: probes/t4228-pivot-range-groups pivot 3), including a list that is too short;
  - a span that is not a whole number of intervals (0 to 10 by 3) has `ceil` groups, the last one `9-12`;
  - GETPIVOTDATA with a number past `endNum` is `#REF!`, and one below `endNum` whose quotient rounds to the group
    count picks `>end`.
- The case builder writes numeric groups already (`group` key, tools/src/rustsheet_tools/pivot.py); S values as
  distinct powers of two let the rendered sums name each group's records (probes/t4235-pivot-range-rounding does so).
- Use the `excel-probe` skill.

## Acceptance criteria
- [ ] An excel-graph probe covers each point above; web.md records the result.
- [ ] The engine matches every probe cell, or the cells are known failures owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date groups and decimal label formats (T-4236).
