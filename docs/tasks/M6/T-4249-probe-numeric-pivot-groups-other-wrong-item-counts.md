---
id: T-4249
title: "Probe numeric pivot groups: other wrong item counts, partial last group, GETPIVOTDATA numbers past the end"
milestone: M6
lane: engine
status: done
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
  - any `<groupItems>` count other than the layout's relabels in T-4253's order (the file's items that are Excel's
    labels first, in file order; only one item too many is probed: probes/t4228-pivot-range-groups pivot 3,
    probes/t4253-pivot-relabel-order), including a list that is too short;
  - (from T-4253; moved to T-4287 by this task's Outcome) label matching by case and spacing, duplicates, non-text
    items and the text of bounds past the probed ones;
  - a span that is not a whole number of intervals (0 to 10 by 3) has `ceil` groups, the last one `9-12`;
  - GETPIVOTDATA with a number past `endNum` is `#REF!`, and one below `endNum` whose quotient rounds to the group
    count picks `>end`.
- The case builder writes numeric groups already (`group` key, tools/src/rustsheet_tools/pivot.py); S values as
  distinct powers of two let the rendered sums name each group's records (probes/t4235-pivot-range-rounding does so).
- Use the `excel-probe` skill.

## Acceptance criteria
- [x] An excel-graph probe covers each point above; web.md records the result.
- [x] The engine matches every probe cell, or the cells are known failures owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Date groups and decimal label formats (T-4236).

## Outcome
- Probes (excel-graph, 2026-11-05): probes/t4249-pivot-range-group-edges (six pivots: lists one short, two too long and
  two short of 0 to 9 by 3; 0 to 10 by 3 with 6 and 5 items; Excel's own labels; GETPIVOTDATA number items on two of
  them) and probes/t4249-pivot-range-group-labels (seven pivots: whole data 0 to 10 by 3, by 2.5 and from 0.5; decimal
  and whole 0 to 9 by 3; the layout's length in wrong text and permuted; number items below `startNum`).
- Findings (web.md "Grouped pivot fields import as their group items"):
  - Excel writes its own labels whatever the list says, of any length, the layout's own included, and keeps the
    file's order for the labels the file lists (T-4253's order rule holds for every length).
  - A partial last step counts as a group, and its `>` label names that group's upper bound (`>12` for 0 to 10 by 3)
    while records still split at `endNum`.
  - Whole data with a whole start and interval gets `lo-(lo + interval - 1)` labels, the last group ending at
    `max(that, endNum)` (`6-9`, `9-11`).
  - GETPIVOTDATA's number rule is truncation toward zero: k = trunc(q) matches when q <= k, k = -1 being `<start`
    and k = n `>end` (-1 by 3 gives `0-3`, -3 and -4.5 `<0`; 12 gives `>12` and 10 is `#REF!` for 0 to 10 by 3).
- Engine (crates/rustsheet-engine/src/pivot/number_groups.rs): `NumberGroups` gains `whole` and turns T-4253's
  `relabeled` into `order` (always computed, empty when it is the identity); `labels` writes whole-number labels and
  the `>` bound; `lower_bound_index` takes the truncation rule; `group_by_numbers` takes the file labels as a slice
  and always writes Excel's labels. Built on T-4253's branch (PR #3256, merged in here), whose "a list of the layout's
  length keeps its labels" the new pivots L and D contradict.
- Moved to T-4287 (new): the T-4253 points this probe does not cover (label match by case and spacing, duplicates,
  non-text items, bounds of 1e15 or more, below 1e-5 or at a tie), whole data with an `endNum` that is not whole, and
  a blank among whole data.
- Eval delta: two new cases, 252/252 and 212/212 cells passing; t4228 (168), t4235 (242), t4253 (196, 224) still pass;
  probes known failures 4390 -> 4390, no regressions.
- No fit: every rule is read off the probe cells; the truncation rule is the one reading that fits all 34 number-item
  cells (t4228 C30:C46, t4249 edges and labels).
