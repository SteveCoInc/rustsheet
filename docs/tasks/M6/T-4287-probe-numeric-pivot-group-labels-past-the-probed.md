---
id: T-4287
title: "Probe numeric pivot group labels past the probed points: large and tiny bounds, non-whole end over whole data, label matching by case and spacing"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The numeric pivot range-group label points no probe covers yet are settled by an excel-graph probe, and the engine
matches it or the cells are owned.

## Context
- Follow-up of T-4249 and T-4253 (crates/rustsheet-engine/src/pivot/number_groups.rs `NumberGroups::new`, `labels`;
  docs/excel-semantics/web.md "Grouped pivot fields import as their group items", the UNVERIFIED (T-4287) list). The
  engine assumes:
  - a file item matches one of Excel's labels on its exact text (case and spacing unprobed: `" 0-3"`, `"0 - 3"`), a
    label the file lists twice keeps its first place, and a non-text item (`<n>`) is skipped;
  - a bound of 1e15 or more, below 1e-5 or at a 15-digit tie is written as text coercion writes it (`number_to_text`:
    `1E+20`); a bound of 1e20 or a tie tells that from plain decimal (probes/t4253-pivot-decimal-relabel agrees on
    the probed bounds only);
  - whole-number data with an `end` that is not whole (0 to 9.5 by 3) labels its last inner group up to
    `max(lo + interval - 1, end)` (`9-11`), and with a whole `end` the same (probed: 0 to 9 gives `6-9`, 0 to 10 gives
    `9-11`; probes/t4249-pivot-range-group-labels pivots G and K); a blank among whole data keeps whole labels;
  - date items (`CacheItem::Date`) among numbers count as numbers for the whole-data test.
- The case builder writes numeric groups (`group` key, tools/src/rustsheet_tools/pivot.py); S values as distinct
  powers of two let the rendered sums name each group's records (probes/t4249-pivot-range-group-edges does so).
  The builder refuses a group list of fewer than three distinct texts.
- Use the `excel-probe` skill.

## Acceptance criteria
- [ ] An excel-graph probe covers each point above, or names the ones excel-graph cannot show; web.md records the
      result.
- [ ] The engine matches every probe cell, or the cells are known failures owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date groups (T-4236) and the GETPIVOTDATA number rule (settled by T-4249).
