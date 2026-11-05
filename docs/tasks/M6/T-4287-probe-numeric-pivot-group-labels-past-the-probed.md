---
id: T-4287
title: "Probe numeric pivot group labels past the probed points: large and tiny bounds, non-whole end over whole data, label matching by case and spacing"
milestone: M6
lane: engine
status: done
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
- [x] An excel-graph probe covers each point above, or names the ones excel-graph cannot show; web.md records the
      result.
- [x] The engine matches every probe cell, or the cells are known failures owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Date groups (T-4236) and the GETPIVOTDATA number rule (settled by T-4249).

## Outcome
- Four excel-graph probes (2026-11-05), sixteen pivots: probes/t4287-pivot-group-label-match, -bound-text, -whole-end
  and -dates-as-numbers. Findings (web.md "Grouped pivot fields import as their group items"):
  - A file label matches one of Excel's ignoring case (`>3e+20` matches `>3E+20`) but not spacing (`6 - 9`, ` 3-5`,
    `3-5 ` match nothing).
  - Bounds follow text coercion past the T-4253 points: 1e15 and 1e-15 plain, 1e20 as `1E+20` (whole data too), and a
    15-digit tie (100000000000000.5) writes its lower neighbour. The engine already matched.
  - Whole labels need a whole `endNum` too: whole data 0 to 9.5 by 3 reads `0-3` ... `9-12`, `>12`.
  - Quirk: a numeric grouping over a blank, or over date-formatted cells among numbers, is dropped on refresh; each
    value is its own item in record order.
- Engine: `NumberGroups::new` requires a whole `end` for whole labels and folds case (`fold_case`) when it matches file
  labels; `Txn::add_pivot_table` leaves a numeric field ungrouped over a blank or mixed date and number cells
  (`grouped`, `mixes_dates`), and `row_layout` writes a number item as its value. The eval adapter now sets spec number
  formats before it adds pivots as well as after the stored formulas (a stored formula's own format replaces an
  earlier one: probes/displayed-text-formats S!A16).
- Not probed (the case builder writes neither; follow-up T-4316): a label the file lists twice and a non-text group item; nor a numeric
  grouping over date cells alone (the engine keeps it). Recorded as UNVERIFIED in web.md.
- Eval delta: probes known failures 5713 -> 5713, 4 new cases (324 cells) all pass, 0 regressions in every suite.
